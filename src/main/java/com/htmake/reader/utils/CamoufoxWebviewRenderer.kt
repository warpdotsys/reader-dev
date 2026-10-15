package com.htmake.reader.utils

import com.google.gson.Gson
import io.legado.app.help.http.CookieStore
import io.legado.app.help.http.StrResponse
import io.legado.app.utils.NetworkUtils
import kotlinx.coroutines.asCoroutineDispatcher
import kotlinx.coroutines.withContext
import java.io.OutputStreamWriter
import java.io.ByteArrayOutputStream
import java.io.InputStream
import java.nio.charset.Charset
import java.nio.charset.StandardCharsets
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.LinkOption
import java.nio.file.SimpleFileVisitor
import java.nio.file.FileVisitResult
import java.nio.file.attribute.BasicFileAttributes
import java.nio.file.attribute.PosixFilePermissions
import java.util.concurrent.Callable
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.ExecutionException
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger
import java.util.concurrent.atomic.AtomicReference

/**
 * Camoufox is managed as a short-lived Python child process inside the Reader container.
 * The browser process is fresh for every render, while the worker itself owns no listener,
 * persistent profile, or user cookies. All browser traffic is forced through the same
 * SSRF-aware egress proxy used by the Chromium renderer.
 */
class CamoufoxWebviewRenderer private constructor(
    private val pythonExecutable: String,
    private val browserVersion: String,
    private val timeoutMs: Int,
    allowPrivateNetworks: Boolean,
    private val workerScriptOverride: Path?,
    networkPolicyOverride: BrowserNetworkPolicy?
) : WebviewRenderer {
    constructor(
        pythonExecutable: String = "python3",
        browserVersion: String = "152.0.4-beta.30",
        timeoutMs: Int = 20_000,
        allowPrivateNetworks: Boolean = System.getenv("READER_BROWSER_ALLOW_PRIVATE_NETWORKS")
            ?.equals("true", ignoreCase = true) == true,
        /** Test-only escape hatch for exercising the parent/worker protocol without Camoufox. */
        workerScriptOverride: Path? = null
    ) : this(pythonExecutable, browserVersion, timeoutMs, allowPrivateNetworks, workerScriptOverride, null)

    /** Only the test fixture may allow its first origin while denying a redirected local host. */
    internal constructor(
        pythonExecutable: String, browserVersion: String, timeoutMs: Int, networkPolicy: BrowserNetworkPolicy
    ) : this(pythonExecutable, browserVersion, timeoutMs, false, null, networkPolicy)
    override val managesBrowserCookies: Boolean = true

    private val pending = AtomicInteger(0)
    private val worker = Executors.newSingleThreadExecutor { task ->
        Thread(task, "reader-camoufox-webview").apply { isDaemon = true }
    }.asCoroutineDispatcher()
    private val outputReaders = Executors.newCachedThreadPool { task ->
        Thread(task, "reader-camoufox-stdout").apply { isDaemon = true }
    }
    private val processes = ConcurrentHashMap.newKeySet<Process>()
    private val networkPolicy = networkPolicyOverride ?: BrowserNetworkPolicy(allowPrivateNetworks)
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
        val blockedNetworkRequest = AtomicReference<BrowserNetworkPolicyViolation?>(null)
        val egressProxy = BrowserEgressProxy(
            networkPolicy,
            timeoutMs,
            { blockedNetworkRequest.compareAndSet(null, it) },
            upstreamProxy
        )
        try {
            val proxyEndpoint = egressProxy.start()
            val headers = request.headerMap ?: emptyMap()
            val cookieStore = CookieStore(request.userNameSpace)
            val requestCookieHeader = headers.entries.firstOrNull { it.key.equals("Cookie", ignoreCase = true) }
                ?.value.orEmpty()
            val legacyScope = NetworkUtils.getSubDomain(url)
            val cookies = BrowserCookieJar.cookiesForBrowserRequest(
                cookieStore,
                url,
                // This key is written only by the legacy enabledCookieJar flow.
                // It is migrated once into structured host-only records before the
                // browser request; it is never treated as a response cookie again.
                legacyCookieHeader = cookieStore.getCookie("${legacyScope}_cookieJar"),
                explicitCookieHeader = requestCookieHeader
            )

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
            blockedNetworkRequest.get()?.let {
                throw it.withRenderContext("Camoufox 渲染触及了本机或非公网网络资源，已中止")
            }

            // Do not flatten browser cookies into CookieStore's legacy domain key.
            // That format has no Path/Secure/Domain/expiry semantics and can both
            // disclose a scoped cookie and recreate one that a response deleted.
            BrowserCookieJar.merge(cookieStore, url, response.cookies.orEmpty().map(WorkerCookie::toStoredCookie))
            return StrResponse(url, response.body ?: "")
        } catch (failure: Exception) {
            // A denied subresource commonly makes the worker report a navigation
            // error first. Keep the parent's sanitized denial, not that secondary symptom.
            blockedNetworkRequest.get()?.let {
                throw it.withRenderContext("Camoufox 渲染触及了本机或非公网网络资源，已中止")
            }
            throw failure
        } finally {
            egressProxy.close()
        }
    }

    private fun invokeWorker(payload: String): WorkerResponse {
        // The parent owns this directory so a killed worker cannot leave its
        // temporary origin-header credentials behind until container shutdown.
        val directory = Files.createTempDirectory("reader-camoufox-request-").toRealPath()
        try {
            if (Files.getFileStore(directory).supportsFileAttributeView("posix")) {
                Files.setPosixFilePermissions(directory, PosixFilePermissions.fromString("rwx------"))
            }
            return invokeWorker(payload, directory)
        } finally {
            removeOwnedRequestDirectory(directory)
        }
    }

    private fun removeOwnedRequestDirectory(directory: Path) {
        // Never follow an addon/browser-created symlink to files outside this
        // exact fresh directory. No environment-expanded or broad deletion root.
        check(directory.isAbsolute && directory.fileName.toString().startsWith("reader-camoufox-request-"))
        check(!Files.isSymbolicLink(directory) && Files.isDirectory(directory, LinkOption.NOFOLLOW_LINKS))
        check(directory.toRealPath(LinkOption.NOFOLLOW_LINKS) == directory)
        Files.walkFileTree(directory, object : SimpleFileVisitor<Path>() {
            override fun visitFile(file: Path, attributes: BasicFileAttributes): FileVisitResult {
                check(file.toAbsolutePath().normalize().startsWith(directory))
                Files.delete(file)
                return FileVisitResult.CONTINUE
            }
            override fun postVisitDirectory(path: Path, failure: java.io.IOException?): FileVisitResult {
                if (failure != null) throw failure
                check(path.toAbsolutePath().normalize().startsWith(directory))
                Files.delete(path)
                return FileVisitResult.CONTINUE
            }
        })
    }

    private fun invokeWorker(payload: String, requestDirectory: Path): WorkerResponse {
        val script = getWorkerScript()
        val process = try {
            ProcessBuilder(pythonExecutable, script.toString())
                .redirectError(ProcessBuilder.Redirect.INHERIT)
                .apply {
                    environment()["PYTHONUNBUFFERED"] = "1"
                    environment()["PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD"] = "1"
                    for (name in listOf("TMPDIR", "TEMP", "TMP")) environment()[name] = requestDirectory.toString()
                }
                .start()
        } catch (e: Exception) {
            throw IllegalStateException("Camoufox Python worker 无法启动", e)
        }
        processes.add(process)
        val stdout = outputReaders.submit(Callable {
            readBoundedWorkerOutput(process.inputStream) {
                // Do not wait for the normal watchdog after an abusive or corrupt
                // worker response. It can otherwise keep the single render queue
                // occupied while descendants retain inherited stdout handles.
                terminate(process)
            }
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
            val output = try {
                stdout.get(OUTPUT_DRAIN_TIMEOUT_SECONDS, TimeUnit.SECONDS).trim()
            } catch (e: ExecutionException) {
                val cause = e.cause
                if (cause is WorkerOutputLimitExceeded) {
                    throw IllegalStateException("Camoufox Python worker 输出超过 ${MAX_WORKER_STDOUT_BYTES / MEBIBYTE} MiB 限制，浏览器进程树已终止")
                }
                throw IllegalStateException("Camoufox Python worker 输出读取失败", cause)
            }
            if (process.exitValue() != 0) {
                throw IllegalStateException("Camoufox Python worker 异常退出 (${process.exitValue()})")
            }
            if (output.isEmpty()) throw IllegalStateException("Camoufox Python worker 未返回结果")
            val response = gson.fromJson(output, WorkerResponse::class.java)
                ?: throw IllegalStateException("Camoufox Python worker 返回了无效 JSON")
            if (response.error == "BlockedScheme") {
                throw BrowserNetworkPolicyViolation("Camoufox WebView 已阻止不支持的资源协议")
            }
            response.error?.let { error ->
                // The child is a protocol boundary. Only known hints accompanying
                // its navigation Error may select these fixed messages; never
                // echo a raw hint, URL or certificate text, or relax TLS policy.
                val message = when (response.certificateError.takeIf { error == "Error" }) {
                    "SSL_ERROR_BAD_CERT_DOMAIN" -> "Camoufox HTTPS 证书域名不匹配 (SSL_ERROR_BAD_CERT_DOMAIN)"
                    "SEC_ERROR_UNKNOWN_ISSUER" -> "Camoufox HTTPS 证书签发机构不受信任 (SEC_ERROR_UNKNOWN_ISSUER)"
                    else -> "Camoufox 渲染失败 ($error)"
                }
                throw IllegalStateException(message)
            }
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
        workerScriptOverride?.let { return it }
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

    /**
     * The worker protocol is a single JSON line, but it is still an untrusted child
     * process boundary. Never use InputStream.readBytes() here: a malformed worker
     * could otherwise allocate until the JVM is killed before the request timeout.
     */
    private fun readBoundedWorkerOutput(input: InputStream, onLimit: () -> Unit): String {
        val output = ByteArrayOutputStream(minOf(MAX_WORKER_STDOUT_BYTES, 64 * 1024))
        val buffer = ByteArray(8 * 1024)
        var total = 0
        input.use { stream ->
            while (true) {
                val read = stream.read(buffer)
                if (read < 0) break
                if (read == 0) continue
                if (read > MAX_WORKER_STDOUT_BYTES - total) {
                    runCatching(onLimit)
                    throw WorkerOutputLimitExceeded()
                }
                output.write(buffer, 0, read)
                total += read
            }
        }
        return output.toString(StandardCharsets.UTF_8)
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

    private data class WorkerCookie(
        val name: String = "",
        val value: String = "",
        val domain: String = "",
        val path: String = "/",
        val hostOnly: Boolean = true,
        val secure: Boolean = false,
        val httpOnly: Boolean = false,
        val sameSite: String? = null,
        val expires: Double = -1.0,
        val deleted: Boolean = false
    ) {
        fun toStoredCookie() = BrowserCookieJar.Cookie(
            name = name,
            value = value,
            domain = domain,
            path = path,
            hostOnly = hostOnly,
            secure = secure,
            httpOnly = httpOnly,
            sameSite = sameSite,
            expires = expires,
            deleted = deleted
        )
    }
    private data class WorkerResponse(
        val body: String?,
        val cookies: List<WorkerCookie>?,
        val error: String?,
        val certificateError: String?
    )

    private class WorkerOutputLimitExceeded : IllegalStateException()

    companion object {
        private const val MAX_PENDING_REQUESTS = 8
        private const val STARTUP_GRACE_MS = 15_000L
        private const val OUTPUT_DRAIN_TIMEOUT_SECONDS = 5L
        private const val MEBIBYTE = 1024 * 1024
        private const val MAX_WORKER_STDOUT_BYTES = 8 * MEBIBYTE
    }
}
