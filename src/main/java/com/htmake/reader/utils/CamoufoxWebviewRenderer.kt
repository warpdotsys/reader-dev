package com.htmake.reader.utils

import com.google.gson.Gson
import io.legado.app.help.http.CookieStore
import io.legado.app.help.http.StrResponse
import io.legado.app.utils.NetworkUtils
import kotlinx.coroutines.asCoroutineDispatcher
import kotlinx.coroutines.withContext
import java.io.OutputStreamWriter
import java.nio.charset.Charset
import java.nio.charset.StandardCharsets
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.attribute.PosixFilePermissions
import java.util.concurrent.Callable
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicInteger

/**
 * Camoufox is managed as a short-lived Python child process inside the Reader container.
 * The browser process is fresh for every render, while the worker itself owns no listener,
 * persistent profile, or user cookies. All browser traffic is forced through the same
 * SSRF-aware egress proxy used by the Chromium renderer.
 */
class CamoufoxWebviewRenderer(
    private val pythonExecutable: String = "python3",
    private val browserVersion: String = "152.0.4-beta.30",
    private val timeoutMs: Int = 20_000,
    allowPrivateNetworks: Boolean = System.getenv("READER_BROWSER_ALLOW_PRIVATE_NETWORKS")
        ?.equals("true", ignoreCase = true) == true
) : WebviewRenderer {
    private val pending = AtomicInteger(0)
    private val worker = Executors.newSingleThreadExecutor { task ->
        Thread(task, "reader-camoufox-webview").apply { isDaemon = true }
    }.asCoroutineDispatcher()
    private val outputReaders = Executors.newCachedThreadPool { task ->
        Thread(task, "reader-camoufox-stdout").apply { isDaemon = true }
    }
    private val processes = ConcurrentHashMap.newKeySet<Process>()
    private val networkPolicy = BrowserNetworkPolicy(allowPrivateNetworks)
    private val gson = Gson()
    @Volatile private var workerScript: Path? = null

    override suspend fun render(request: WebviewRequest): StrResponse {
        if (pending.incrementAndGet() > MAX_PENDING_REQUESTS) {
            pending.decrementAndGet()
            throw IllegalStateException("Camoufox WebView 请求队列已满")
        }
        try {
            return withContext(worker) { renderOnWorker(request) }
        } finally {
            pending.decrementAndGet()
        }
    }

    private fun renderOnWorker(request: WebviewRequest): StrResponse {
        val url = request.url ?: throw IllegalArgumentException("Camoufox WebView 需要 HTTP URL")
        networkPolicy.requireDocumentUrl(url)
        request.sourceRegex?.takeIf { it.isNotBlank() }?.let(::Regex)
        if (timeoutMs <= 0) throw IllegalArgumentException("browserTimeoutMs must be positive")

        val upstreamProxy = request.proxy?.takeIf { it.isNotBlank() }?.let(BrowserUpstreamProxy::parse)
        val blockedNetworkRequest = AtomicBoolean(false)
        val egressProxy = BrowserEgressProxy(
            networkPolicy,
            timeoutMs,
            { blockedNetworkRequest.set(true) },
            upstreamProxy
        )
        try {
            val proxyEndpoint = egressProxy.start()
            val headers = request.headerMap ?: emptyMap()
            val domain = NetworkUtils.getSubDomain(url)
            val cookieStore = CookieStore(request.userNameSpace)
            val cookies = cookieStore.cookieToMap(cookieStore.getCookie(domain))
            headers.entries.firstOrNull { it.key.equals("Cookie", ignoreCase = true) }
                ?.value?.let { cookies.putAll(cookieStore.cookieToMap(it)) }

            val payload = linkedMapOf<String, Any?>(
                "url" to url,
                "html" to request.html?.let { encodeHtmlForWebView(it, request.encode) },
                "headers" to headers.filterKeys { key ->
                    !key.equals("Cookie", ignoreCase = true) &&
                        !key.equals("User-Agent", ignoreCase = true) &&
                        !key.equals("Host", ignoreCase = true) &&
                        !key.equals("Content-Length", ignoreCase = true)
                },
                "userAgent" to headers.entries.firstOrNull { it.key.equals("User-Agent", ignoreCase = true) }
                    ?.value,
                "cookies" to cookies,
                "sourceRegex" to request.sourceRegex?.takeIf { it.isNotBlank() },
                "javaScript" to request.javaScript,
                "proxy" to proxyEndpoint,
                "post" to request.post,
                "body" to request.body,
                "timeoutMs" to timeoutMs,
                "browserVersion" to browserVersion
            )
            val response = invokeWorker(gson.toJson(payload))
            if (blockedNetworkRequest.get()) {
                throw BrowserNetworkPolicyViolation("Camoufox 渲染触及了本机或非公网网络资源，已中止")
            }

            if (domain.isNotEmpty()) {
                val stored = response.cookies.orEmpty().joinToString(";") { "${it.name}=${it.value}" }
                cookieStore.setCookie("${domain}_cookieJar", stored)
                cookieStore.setCookie(domain, stored)
            }
            return StrResponse(url, response.body ?: "")
        } finally {
            egressProxy.close()
        }
    }

    private fun invokeWorker(payload: String): WorkerResponse {
        val script = getWorkerScript()
        val process = try {
            ProcessBuilder(pythonExecutable, script.toString())
                .redirectError(ProcessBuilder.Redirect.INHERIT)
                .apply {
                    environment()["PYTHONUNBUFFERED"] = "1"
                    environment()["PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD"] = "1"
                }
                .start()
        } catch (e: Exception) {
            throw IllegalStateException("Camoufox Python worker 无法启动", e)
        }
        processes.add(process)
        val stdout = outputReaders.submit(Callable {
            process.inputStream.use { it.readBytes().toString(StandardCharsets.UTF_8) }
        })
        try {
            OutputStreamWriter(process.outputStream, StandardCharsets.UTF_8).use { writer ->
                writer.write(payload)
                writer.write("\n")
            }
            val processWatchdogMs = timeoutMs.toLong() * 2L + STARTUP_GRACE_MS
            if (!process.waitFor(processWatchdogMs, TimeUnit.MILLISECONDS)) {
                terminate(process)
                throw IllegalStateException("Camoufox WebView 超时，浏览器进程树已终止")
            }
            val output = stdout.get(OUTPUT_DRAIN_TIMEOUT_SECONDS, TimeUnit.SECONDS).trim()
            if (process.exitValue() != 0) {
                throw IllegalStateException("Camoufox Python worker 异常退出 (${process.exitValue()})")
            }
            if (output.isEmpty()) throw IllegalStateException("Camoufox Python worker 未返回结果")
            val response = gson.fromJson(output, WorkerResponse::class.java)
                ?: throw IllegalStateException("Camoufox Python worker 返回了无效 JSON")
            if (response.error == "BlockedScheme") {
                throw BrowserNetworkPolicyViolation("Camoufox WebView 已阻止不支持的资源协议")
            }
            response.error?.let { throw IllegalStateException("Camoufox 渲染失败 ($it)") }
            return response
        } catch (e: InterruptedException) {
            terminate(process)
            Thread.currentThread().interrupt()
            throw IllegalStateException("Camoufox WebView 请求已取消")
        } catch (e: Exception) {
            if (process.isAlive) terminate(process)
            throw e
        } finally {
            processes.remove(process)
            if (process.isAlive) terminate(process)
            if (!stdout.isDone) stdout.cancel(true)
        }
    }

    private fun getWorkerScript(): Path {
        workerScript?.takeIf { Files.isRegularFile(it) }?.let { return it }
        val resource = javaClass.getResourceAsStream("/camoufox/worker.py")
            ?: throw IllegalStateException("JAR 未包含 Camoufox worker 脚本")
        val path = Files.createTempFile("reader-camoufox-worker-", ".py")
        try {
            runCatching { Files.setPosixFilePermissions(path, PosixFilePermissions.fromString("rw-------")) }
            resource.use { input -> Files.copy(input, path, java.nio.file.StandardCopyOption.REPLACE_EXISTING) }
            workerScript = path
            return path
        } catch (e: Exception) {
            Files.deleteIfExists(path)
            throw IllegalStateException("无法准备 Camoufox worker 脚本", e)
        }
    }

    private fun terminate(process: Process) {
        runCatching { process.toHandle().descendants().forEach { it.destroyForcibly() } }
        runCatching { process.destroyForcibly() }
        runCatching { process.waitFor(2, TimeUnit.SECONDS) }
        runCatching { process.inputStream.close() }
        runCatching { process.outputStream.close() }
    }

    private fun encodeHtmlForWebView(html: String, encode: String?): String {
        if (encode.isNullOrBlank()) return html
        val charset = try {
            Charset.forName(encode)
        } catch (e: Exception) {
            throw IllegalArgumentException("不支持的 WebView HTML 字符集: $encode", e)
        }
        return String(html.toByteArray(charset), charset)
    }

    override suspend fun close() {
        processes.toList().forEach(::terminate)
        try {
            withContext(worker) {
                workerScript?.let { Files.deleteIfExists(it) }
                workerScript = null
            }
        } finally {
            worker.close()
            outputReaders.shutdownNow()
        }
    }

    private data class WorkerCookie(val name: String, val value: String)
    private data class WorkerResponse(
        val body: String?,
        val cookies: List<WorkerCookie>?,
        val error: String?
    )

    companion object {
        private const val MAX_PENDING_REQUESTS = 8
        private const val STARTUP_GRACE_MS = 15_000L
        private const val OUTPUT_DRAIN_TIMEOUT_SECONDS = 5L
    }
}
