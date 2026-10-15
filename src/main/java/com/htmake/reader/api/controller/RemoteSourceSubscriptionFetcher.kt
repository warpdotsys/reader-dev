package com.htmake.reader.api.controller

import com.htmake.reader.ReaderApplication
import com.htmake.reader.utils.BrowserNetworkPolicy
import com.htmake.reader.utils.BrowserNetworkPolicyViolation
import io.vertx.core.Future
import io.vertx.core.buffer.Buffer
import io.vertx.core.http.HttpClient
import io.vertx.core.http.HttpClientOptions
import io.vertx.core.http.HttpMethod
import io.vertx.core.net.SocketAddress
import io.vertx.core.json.JsonArray
import io.vertx.kotlin.coroutines.awaitResult
import java.net.URI

/**
 * Fetches a Vue 3 subscription using one validated DNS answer instead of
 * resolving the hostname a second time during connect. Redirects are rejected
 * rather than followed, so every target must pass the same policy explicitly.
 */
internal class RemoteSourceSubscriptionFetcher(
    private val networkPolicy: BrowserNetworkPolicy = BrowserNetworkPolicy(),
    private val client: HttpClient = ReaderApplication.vertx().createHttpClient(
        HttpClientOptions().setTryUseCompression(false).setTrustAll(false),
    ),
) {
    suspend fun fetch(url: String): JsonArray {
        val target = validate(url)
        try {
            val body = awaitResult<Buffer> { completion ->
                var completed = false
                fun fail(error: Throwable) {
                    if (!completed) {
                        completed = true
                        completion.handle(Future.failedFuture(error))
                    }
                }
                lateinit var request: io.vertx.core.http.HttpClientRequest
                request = client.requestAbs(
                    HttpMethod.GET,
                    SocketAddress.inetSocketAddress(target.port, target.addresses.first().hostAddress),
                    url,
                ) { response ->
                    if (response.statusCode() !in 200..299) {
                        response.resume()
                        fail(RemoteSourceSubscriptionException("远程书源链接错误：HTTP ${response.statusCode()}"))
                        return@requestAbs
                    }
                    val length = response.getHeader("Content-Length")?.toLongOrNull()
                    if (length != null && length > SOURCE_SUBSCRIPTION_MAX_BYTES) {
                        response.resume()
                        fail(RemoteSourceSubscriptionException("远程书源文件超过 2 MiB 限制"))
                        return@requestAbs
                    }
                    val collected = Buffer.buffer()
                    response.exceptionHandler { fail(it) }
                    response.handler { chunk ->
                        if (completed) return@handler
                        if (collected.length() + chunk.length() > SOURCE_SUBSCRIPTION_MAX_BYTES) {
                            response.pause()
                            response.request().connection().close()
                            fail(RemoteSourceSubscriptionException("远程书源文件超过 2 MiB 限制"))
                        } else {
                            collected.appendBuffer(chunk)
                        }
                    }
                    response.endHandler {
                        if (!completed) {
                            completed = true
                            completion.handle(Future.succeededFuture(collected))
                        }
                    }
                }
                // A host header is still needed when a pinned IP is used for the TCP connection.
                request.putHeader("Host", target.uri.rawAuthority)
                request.setTimeout(SOURCE_SUBSCRIPTION_TIMEOUT_MS)
                request.exceptionHandler { fail(it) }
                request.end()
            }
            return try {
                parseRemoteBookSources(body.toString("UTF-8"))
            } catch (e: IllegalArgumentException) {
                throw RemoteSourceSubscriptionException("远程书源数据无效", e)
            }
        } catch (e: RemoteSourceSubscriptionException) {
            throw e
        } catch (e: Exception) {
            throw RemoteSourceSubscriptionException("远程书源链接或数据错误", e)
        }
    }

    private fun validate(url: String) = try {
        validateSourceSubscriptionTarget(url, networkPolicy)
    } catch (e: RemoteSourceSubscriptionException) {
        throw e
    }
}

internal fun validateSourceSubscriptionTarget(url: String, networkPolicy: BrowserNetworkPolicy) = try {
    val uri = URI(url)
    if (uri.fragment != null) throw RemoteSourceSubscriptionException("订阅链接不能包含片段")
    networkPolicy.resolveRequestTarget(url)
        ?: throw RemoteSourceSubscriptionException("订阅地址必须是 HTTP 或 HTTPS")
} catch (e: RemoteSourceSubscriptionException) {
    throw e
} catch (e: BrowserNetworkPolicyViolation) {
    throw RemoteSourceSubscriptionException("订阅地址被安全策略拒绝：${e.message}", e)
} catch (_: Exception) {
    throw RemoteSourceSubscriptionException("订阅地址无效")
}

internal class RemoteSourceSubscriptionException(message: String, cause: Throwable? = null) : IllegalArgumentException(message, cause)
