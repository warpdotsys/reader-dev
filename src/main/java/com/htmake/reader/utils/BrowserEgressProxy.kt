package com.htmake.reader.utils

import java.io.BufferedInputStream
import java.io.BufferedOutputStream
import java.io.ByteArrayOutputStream
import java.io.EOFException
import java.io.IOException
import java.io.InputStream
import java.io.OutputStream
import java.net.InetAddress
import java.net.InetSocketAddress
import java.net.ServerSocket
import java.net.Socket
import java.nio.charset.StandardCharsets
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.Executors
import java.util.concurrent.RejectedExecutionException
import java.util.concurrent.Semaphore
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Per-render HTTP forward proxy. Chromium uses this for HTTP, HTTPS CONNECT, and WebSocket
 * handshakes so every actual destination is re-resolved, checked, and connected by pinned IP.
 */
internal class BrowserEgressProxy(
    private val networkPolicy: BrowserNetworkPolicy,
    private val timeoutMs: Int,
    private val onBlocked: () -> Unit,
    private val upstreamProxy: BrowserUpstreamProxy? = null
) : AutoCloseable {
    private val running = AtomicBoolean(false)
    private val sockets = ConcurrentHashMap.newKeySet<Socket>()
    private val connectionLimit = Semaphore(MAX_CONNECTIONS)
    private val clients = Executors.newCachedThreadPool { task ->
        Thread(task, "reader-browser-egress-client").apply { isDaemon = true }
    }
    private var listener: ServerSocket? = null
    private var acceptor: Thread? = null

    fun start(): String {
        check(running.compareAndSet(false, true)) { "Browser egress proxy already started" }
        val server = ServerSocket()
        try {
            server.reuseAddress = true
            server.bind(InetSocketAddress(InetAddress.getByName("127.0.0.1"), 0), 64)
            listener = server
            acceptor = Thread({ acceptLoop(server) }, "reader-browser-egress-accept").apply {
                isDaemon = true
                start()
            }
            return "http://127.0.0.1:${server.localPort}"
        } catch (failure: Throwable) {
            running.set(false)
            runCatching { server.close() }
            throw failure
        }
    }

    private fun acceptLoop(server: ServerSocket) {
        while (running.get()) {
            try {
                val client = server.accept()
                if (!connectionLimit.tryAcquire()) {
                    runCatching { client.close() }
                    continue
                }
                sockets.add(client)
                try {
                    clients.execute {
                        try {
                            handle(client)
                        } finally {
                            sockets.remove(client)
                            runCatching { client.close() }
                            connectionLimit.release()
                        }
                    }
                } catch (_: RejectedExecutionException) {
                    sockets.remove(client)
                    runCatching { client.close() }
                    connectionLimit.release()
                }
            } catch (_: IOException) {
                if (running.get()) continue
                return
            }
        }
    }

    private fun handle(client: Socket) {
        client.soTimeout = timeoutMs
        val input = BufferedInputStream(client.getInputStream())
        val output = BufferedOutputStream(client.getOutputStream())
        try {
            val head = readHead(input) ?: return
            val requestLine = parseStartLine(head.startLine)
            if (requestLine.method.equals("CONNECT", ignoreCase = true)) {
                tunnelHttps(client, input, output, requestLine.target)
            } else {
                forwardHttp(client, input, output, requestLine, head.headers)
            }
        } catch (_: BrowserNetworkPolicyViolation) {
            onBlocked()
            runCatching { writeError(output, 403, "Forbidden") }
        } catch (_: Exception) {
            runCatching { writeError(output, 502, "Bad Gateway") }
        }
    }

    private fun tunnelHttps(
        client: Socket,
        clientInput: BufferedInputStream,
        clientOutput: BufferedOutputStream,
        authority: String
    ) {
        val target = parseAuthority(authority)
        val validated = networkPolicy.resolveRequestTarget(
            "https://${formatHost(target.first)}:${target.second}/"
        ) ?: throw BrowserNetworkPolicyViolation("本地 WebView 已阻止无效 CONNECT 目标")
        val upstream = connectTunnel(validated, target.second)
        sockets.add(upstream)
        try {
            upstream.soTimeout = 0
            client.soTimeout = 0
            clientOutput.write("HTTP/1.1 200 Connection Established\r\n\r\n".toByteArray(StandardCharsets.ISO_8859_1))
            clientOutput.flush()
            relay(client, upstream, clientInput, BufferedInputStream(upstream.getInputStream()), clientOutput,
                BufferedOutputStream(upstream.getOutputStream()))
        } finally {
            sockets.remove(upstream)
            runCatching { upstream.close() }
        }
    }

    private fun forwardHttp(
        client: Socket,
        clientInput: BufferedInputStream,
        clientOutput: BufferedOutputStream,
        request: StartLine,
        headers: List<Header>
    ) {
        val uri = requestUri(request.target, headers)
        if (!uri.scheme.equals("http", ignoreCase = true) && !uri.scheme.equals("ws", ignoreCase = true)) {
            throw BrowserNetworkPolicyViolation("本地 WebView 代理仅接受 HTTP、WebSocket 或 CONNECT 请求")
        }
        val validated = networkPolicy.resolveRequestTarget(uri.toString())
            ?: throw BrowserNetworkPolicyViolation("本地 WebView 已阻止无效 HTTP 目标")
        val requestIsChunked = isChunked(headers)
        val requestLength = contentLength(headers)
        if (requestIsChunked && requestLength != null) {
            throw IOException("Ambiguous request body framing")
        }
        val connection = connectHttpTarget(validated)
        val upstream = connection.socket
        sockets.add(upstream)
        try {
            upstream.soTimeout = timeoutMs
            val upstreamInput = BufferedInputStream(upstream.getInputStream())
            val upstreamOutput = BufferedOutputStream(upstream.getOutputStream())
            val upgrade = headers.any { it.name.equals("upgrade", true) } &&
                connectionTokens(headers).contains("upgrade")
            val connectionHeaders = connectionTokens(headers)
            val target = buildString {
                if (connection.usesHttpProxy) {
                    append("http://").append(formatHost(connection.destinationAddress.hostAddress))
                        .append(':').append(validated.port)
                }
                append(uri.rawPath?.takeIf { it.isNotEmpty() } ?: "/")
                uri.rawQuery?.let { append('?').append(it) }
            }
            upstreamOutput.write("${request.method} $target ${request.version}\r\n".toByteArray(StandardCharsets.ISO_8859_1))
            var forwardedHost = false
            headers.forEach { header ->
                val name = header.name.lowercase(java.util.Locale.ROOT)
                if (name in HOP_BY_HOP_REQUEST_HEADERS ||
                    (name in connectionHeaders && !(upgrade && name == "upgrade")) ||
                    name.equals("expect", true)) return@forEach
                if (name == "host") forwardedHost = true
                upstreamOutput.write("${header.name}: ${header.value}\r\n".toByteArray(StandardCharsets.ISO_8859_1))
            }
            if (!forwardedHost) {
                upstreamOutput.write("Host: ${formatHost(validated.host)}${uri.port.takeIf { it > 0 }?.let { ":$it" } ?: ""}\r\n"
                    .toByteArray(StandardCharsets.ISO_8859_1))
            }
            if (connection.usesHttpProxy) {
                upstreamProxy?.basicAuthorization?.let {
                    upstreamOutput.write("Proxy-Authorization: $it\r\n".toByteArray(StandardCharsets.ISO_8859_1))
                }
            }
            if (upgrade) {
                upstreamOutput.write("Connection: Upgrade\r\n".toByteArray(StandardCharsets.ISO_8859_1))
            } else {
                upstreamOutput.write("Connection: close\r\n".toByteArray(StandardCharsets.ISO_8859_1))
            }
            upstreamOutput.write("\r\n".toByteArray(StandardCharsets.ISO_8859_1))
            upstreamOutput.flush()
            copyRequestBody(clientInput, upstreamOutput, headers)
            upstreamOutput.flush()

            var response = readHead(upstreamInput) ?: throw EOFException("Origin closed before sending a response")
            var status = responseStatus(response.startLine)
            while (status in 100..199 && status != 101) {
                writeHead(clientOutput, response.startLine,
                    responseHeaders(response.headers, switchingProtocols = false, interim = true))
                clientOutput.flush()
                response = readHead(upstreamInput) ?: throw EOFException("Origin closed before final response")
                status = responseStatus(response.startLine)
            }
            if (status == 101 && !upgrade) throw IOException("Unexpected protocol upgrade response")
            val responseIsChunked = isChunked(response.headers)
            val responseLength = contentLength(response.headers)
            if (responseIsChunked && responseLength != null) {
                throw IOException("Ambiguous response body framing")
            }
            writeHead(clientOutput, response.startLine, responseHeaders(response.headers, switchingProtocols = status == 101))
            clientOutput.flush()
            if (status == 101 && upgrade) {
                upstream.soTimeout = 0
                client.soTimeout = 0
                relay(client, upstream, clientInput, upstreamInput, clientOutput, upstreamOutput)
                return
            }
            copyResponseBody(upstreamInput, clientOutput, request.method, status, response.headers)
            clientOutput.flush()
        } finally {
            sockets.remove(upstream)
            runCatching { upstream.close() }
        }
    }

    private fun requestUri(target: String, headers: List<Header>) = try {
        val parsed = java.net.URI(target)
        if (parsed.isAbsolute) parsed else {
            val host = headers.firstOrNull { it.name.equals("host", true) }?.value
                ?: throw BrowserNetworkPolicyViolation("本地 WebView HTTP 请求缺少 Host")
            java.net.URI("http://$host$target")
        }
    } catch (failure: BrowserNetworkPolicyViolation) {
        throw failure
    } catch (_: Exception) {
        throw BrowserNetworkPolicyViolation("本地 WebView 收到无效 HTTP 代理请求")
    }

    private fun parseAuthority(value: String): Pair<String, Int> {
        val uri = try {
            java.net.URI("http://$value")
        } catch (_: Exception) {
            throw BrowserNetworkPolicyViolation("本地 WebView 收到无效 CONNECT 目标")
        }
        val host = uri.host?.removePrefix("[")?.removeSuffix("]")
            ?: throw BrowserNetworkPolicyViolation("本地 WebView CONNECT 目标缺少主机名")
        val port = uri.port
        if (port !in 1..65535) throw BrowserNetworkPolicyViolation("本地 WebView CONNECT 端口无效")
        return host to port
    }

    private fun connectPinned(addresses: List<InetAddress>, port: Int): Socket {
        var lastFailure: IOException? = null
        addresses.forEach { address ->
            val socket = Socket()
            try {
                socket.connect(InetSocketAddress(address, port), timeoutMs)
                return socket
            } catch (failure: IOException) {
                lastFailure = failure
                runCatching { socket.close() }
            }
        }
        throw lastFailure ?: IOException("No validated address for browser request")
    }

    private fun connectHttpTarget(target: BrowserNetworkTarget): HttpConnection {
        val proxy = upstreamProxy ?: run {
            val address = target.addresses.firstOrNull() ?: throw IOException("No validated target address")
            return HttpConnection(connectPinned(target.addresses, target.port), address, false)
        }
        return when (proxy.protocol) {
            BrowserUpstreamProxy.Protocol.HTTP -> {
                val proxyTarget = resolveUpstreamProxy(proxy)
                val address = target.addresses.firstOrNull() ?: throw IOException("No validated target address")
                HttpConnection(connectPinned(proxyTarget.addresses, proxyTarget.port), address, true)
            }
            BrowserUpstreamProxy.Protocol.SOCKS4,
            BrowserUpstreamProxy.Protocol.SOCKS5 -> {
                val (socket, address) = connectSocksTarget(target, proxy)
                HttpConnection(socket, address, false)
            }
        }
    }

    private fun connectTunnel(target: BrowserNetworkTarget, port: Int): Socket {
        val proxy = upstreamProxy ?: return connectPinned(target.addresses, port)
        return when (proxy.protocol) {
            BrowserUpstreamProxy.Protocol.HTTP -> {
                val proxyTarget = resolveUpstreamProxy(proxy)
                val address = target.addresses.firstOrNull() ?: throw IOException("No validated target address")
                val socket = connectPinned(proxyTarget.addresses, proxyTarget.port)
                try {
                    socket.soTimeout = timeoutMs
                    val output = BufferedOutputStream(socket.getOutputStream())
                    val input = BufferedInputStream(socket.getInputStream())
                    val authority = "${formatHost(address.hostAddress)}:$port"
                    output.write("CONNECT $authority HTTP/1.1\r\nHost: $authority\r\n".toByteArray(StandardCharsets.ISO_8859_1))
                    proxy.basicAuthorization?.let {
                        output.write("Proxy-Authorization: $it\r\n".toByteArray(StandardCharsets.ISO_8859_1))
                    }
                    output.write("Proxy-Connection: Keep-Alive\r\n\r\n".toByteArray(StandardCharsets.ISO_8859_1))
                    output.flush()
                    val response = readHead(input) ?: throw EOFException("Upstream proxy closed during CONNECT")
                    val status = responseStatus(response.startLine)
                    if (status !in 200..299) throw IOException("Upstream HTTP proxy CONNECT failed with status $status")
                    socket
                } catch (failure: Exception) {
                    runCatching { socket.close() }
                    throw failure
                }
            }
            BrowserUpstreamProxy.Protocol.SOCKS4,
            BrowserUpstreamProxy.Protocol.SOCKS5 -> connectSocksTarget(target, proxy).first
        }
    }

    private fun connectSocksTarget(
        target: BrowserNetworkTarget,
        proxy: BrowserUpstreamProxy
    ): Pair<Socket, InetAddress> {
        val proxyTarget = resolveUpstreamProxy(proxy)
        var lastFailure: IOException? = null
        target.addresses.forEach { destination ->
            val socket = try {
                connectPinned(proxyTarget.addresses, proxyTarget.port)
            } catch (failure: IOException) {
                lastFailure = failure
                return@forEach
            }
            try {
                socket.soTimeout = timeoutMs
                when (proxy.protocol) {
                    BrowserUpstreamProxy.Protocol.SOCKS4 -> establishSocks4(socket, destination, target.port, proxy)
                    BrowserUpstreamProxy.Protocol.SOCKS5 -> establishSocks5(socket, destination, target.port, proxy)
                    else -> throw IOException("Invalid SOCKS upstream proxy protocol")
                }
                return socket to destination
            } catch (failure: IOException) {
                lastFailure = failure
                runCatching { socket.close() }
            }
        }
        throw lastFailure ?: IOException("No validated address for SOCKS proxy request")
    }

    private fun resolveUpstreamProxy(proxy: BrowserUpstreamProxy): BrowserNetworkTarget =
        networkPolicy.resolveRequestTarget(proxy.endpointUrl)
            ?: throw BrowserNetworkPolicyViolation("本地 WebView 已阻止无效上游代理地址")

    private fun establishSocks4(socket: Socket, destination: InetAddress, port: Int, proxy: BrowserUpstreamProxy) {
        if (destination !is java.net.Inet4Address) throw IOException("SOCKS4 cannot tunnel IPv6 targets")
        if (proxy.password != null) throw IOException("SOCKS4 proxy password authentication is unsupported")
        val user = (proxy.username ?: "").toByteArray(StandardCharsets.UTF_8)
        if (user.any { it == 0.toByte() }) throw IOException("Invalid SOCKS4 username")
        val output = socket.getOutputStream()
        output.write(byteArrayOf(4, 1, (port ushr 8).toByte(), port.toByte()))
        output.write(destination.address)
        output.write(user)
        output.write(0)
        output.flush()
        val response = readExactly(socket.getInputStream(), 8)
        if (response[0].toInt() != 0 || response[1].toInt() and 0xff != 90) {
            throw IOException("SOCKS4 proxy rejected the connection")
        }
    }

    private fun establishSocks5(
        socket: Socket,
        destination: InetAddress,
        port: Int,
        proxy: BrowserUpstreamProxy
    ) {
        val username = proxy.username?.toByteArray(StandardCharsets.UTF_8)
        val password = proxy.password?.toByteArray(StandardCharsets.UTF_8)
        if ((username == null) != (password == null)) throw IOException("Incomplete SOCKS5 credentials")
        if (username != null && (username.size > 255 || password!!.size > 255)) {
            throw IOException("SOCKS5 credentials exceed protocol limits")
        }
        val output = socket.getOutputStream()
        val input = socket.getInputStream()
        output.write(if (username == null) byteArrayOf(5, 1, 0) else byteArrayOf(5, 2, 0, 2))
        output.flush()
        val method = readExactly(input, 2)
        if (method[0].toInt() != 5) throw IOException("Invalid SOCKS5 negotiation response")
        when (method[1].toInt() and 0xff) {
            0 -> Unit
            2 -> {
                if (username == null) throw IOException("SOCKS5 proxy requires credentials")
                output.write(byteArrayOf(1, username.size.toByte()))
                output.write(username)
                output.write(password!!.size)
                output.write(password)
                output.flush()
                val auth = readExactly(input, 2)
                if (auth[0].toInt() != 1 || auth[1].toInt() != 0) {
                    throw IOException("SOCKS5 proxy authentication failed")
                }
            }
            else -> throw IOException("SOCKS5 proxy selected an unsupported authentication method")
        }
        val address = destination.address
        val addressType = if (address.size == 4) 1 else 4
        output.write(byteArrayOf(5, 1, 0, addressType.toByte()))
        output.write(address)
        output.write(byteArrayOf((port ushr 8).toByte(), port.toByte()))
        output.flush()
        val reply = readExactly(input, 4)
        if (reply[0].toInt() != 5 || reply[1].toInt() != 0) {
            throw IOException("SOCKS5 proxy rejected the target connection")
        }
        val boundAddressLength = when (reply[3].toInt() and 0xff) {
            1 -> 4
            3 -> readExactly(input, 1)[0].toInt() and 0xff
            4 -> 16
            else -> throw IOException("Invalid SOCKS5 bound address")
        }
        readExactly(input, boundAddressLength + 2)
    }

    private fun readExactly(input: InputStream, count: Int): ByteArray {
        val bytes = ByteArray(count)
        var offset = 0
        while (offset < bytes.size) {
            val read = input.read(bytes, offset, bytes.size - offset)
            if (read < 0) throw EOFException("Unexpected end of proxy handshake")
            offset += read
        }
        return bytes
    }

    private data class HttpConnection(
        val socket: Socket,
        val destinationAddress: InetAddress,
        val usesHttpProxy: Boolean
    )

    private fun relay(
        client: Socket,
        upstream: Socket,
        clientInput: InputStream,
        upstreamInput: InputStream,
        clientOutput: OutputStream,
        upstreamOutput: OutputStream
    ) {
        val toClient = clients.submit {
            runCatching { pump(upstreamInput, clientOutput) }
            runCatching { client.shutdownOutput() }
        }
        try {
            runCatching { pump(clientInput, upstreamOutput) }
            runCatching { upstream.shutdownOutput() }
            toClient.get(timeoutMs.toLong().coerceAtLeast(1L), TimeUnit.MILLISECONDS)
        } finally {
            toClient.cancel(true)
        }
    }

    private fun pump(input: InputStream, output: OutputStream) {
        val buffer = ByteArray(8192)
        while (true) {
            val count = input.read(buffer)
            if (count < 0) return
            output.write(buffer, 0, count)
            output.flush()
        }
    }

    private fun copyRequestBody(input: BufferedInputStream, output: BufferedOutputStream, headers: List<Header>) {
        if (isChunked(headers)) {
            copyChunked(input, output, flushChunks = false)
            return
        }
        val length = contentLength(headers) ?: return
        copyExactly(input, output, length, flush = false)
    }

    private fun copyResponseBody(
        input: BufferedInputStream,
        output: BufferedOutputStream,
        requestMethod: String,
        status: Int,
        headers: List<Header>
    ) {
        if (requestMethod.equals("HEAD", true) || status in 100..199 || status == 204 || status == 304) return
        if (isChunked(headers)) {
            copyChunked(input, output, flushChunks = true)
            return
        }
        val length = contentLength(headers)
        if (length != null) copyExactly(input, output, length.coerceAtLeast(0), flush = true)
        else pump(input, output)
    }

    private fun isChunked(headers: List<Header>): Boolean {
        val encodings = headers.filter { it.name.equals("transfer-encoding", true) }
            .flatMap { it.value.split(',') }
            .map { it.trim().lowercase(java.util.Locale.ROOT) }
        if (encodings.isEmpty()) return false
        if (encodings.any { it.isEmpty() } || encodings.last() != "chunked") {
            throw IOException("Unsupported Transfer-Encoding")
        }
        return true
    }

    private fun contentLength(headers: List<Header>): Long? {
        val rawValues = headers.filter { it.name.equals("content-length", true) }
            .flatMap { it.value.split(',') }
            .map { it.trim() }
        if (rawValues.isEmpty()) return null
        val lengths = rawValues.map { it.toLongOrNull() ?: throw IOException("Invalid Content-Length") }
        if (lengths.any { it < 0 } || lengths.distinct().size != 1) {
            throw IOException("Conflicting Content-Length")
        }
        return lengths.first()
    }

    private fun responseStatus(startLine: String): Int {
        val parts = startLine.split(' ', limit = 3)
        if (parts.size < 2 || parts[0] !in setOf("HTTP/1.0", "HTTP/1.1") ||
            parts.getOrNull(2).orEmpty().any { (it.code < 32 && it != '\t') || it.code == 127 }) {
            throw IOException("Invalid upstream response line")
        }
        val status = parts.getOrNull(1)?.toIntOrNull()
            ?: throw IOException("Invalid upstream response")
        if (status !in 100..599) throw IOException("Invalid upstream response status")
        return status
    }

    private fun responseHeaders(
        headers: List<Header>,
        switchingProtocols: Boolean,
        interim: Boolean = false
    ): List<Header> {
        val connectionHeaders = connectionTokens(headers)
        val forwarded = headers.filter { header ->
            val name = header.name.lowercase(java.util.Locale.ROOT)
            name !in HOP_BY_HOP_RESPONSE_HEADERS && name !in connectionHeaders &&
                (switchingProtocols || name != "upgrade")
        }.toMutableList()
        if (switchingProtocols) {
            val upgrade = headers.firstOrNull { it.name.equals("upgrade", true) }
            if (upgrade != null) forwarded.add(Header("Upgrade", upgrade.value))
            forwarded.add(Header("Connection", "Upgrade"))
        } else if (!interim) {
            forwarded.add(Header("Connection", "close"))
        }
        return forwarded
    }

    private fun connectionTokens(headers: List<Header>): Set<String> = headers
        .filter { it.name.equals("connection", true) }
        .flatMap { it.value.split(',') }
        .map { it.trim().lowercase(java.util.Locale.ROOT) }
        .filter { it.isNotEmpty() }
        .toSet()

    private fun copyChunked(input: BufferedInputStream, output: BufferedOutputStream, flushChunks: Boolean) {
        while (true) {
            val line = readLine(input) ?: throw EOFException("Unexpected end of chunked body")
            output.write(line)
            output.write(CRLF)
            if (flushChunks) output.flush()
            val size = line.toString(StandardCharsets.ISO_8859_1).substringBefore(';').trim().toLongOrNull(16)
                ?: throw IOException("Invalid HTTP chunk size")
            if (size == 0L) {
                var trailerBytes = 0
                while (true) {
                    val trailer = readLine(input) ?: throw EOFException("Unexpected end of chunk trailers")
                    trailerBytes += trailer.size + CRLF.size
                    if (trailerBytes > MAX_HEADER_BYTES) throw IOException("HTTP trailers too large")
                    output.write(trailer)
                    output.write(CRLF)
                    if (flushChunks) output.flush()
                    if (trailer.isEmpty()) return
                }
            }
            if (size < 0) throw IOException("Invalid HTTP chunk size")
            copyExactly(input, output, size, flush = flushChunks)
            if (input.read() != 13 || input.read() != 10) {
                throw IOException("Invalid HTTP chunk terminator")
            }
            output.write(CRLF)
        }
    }

    private fun copyExactly(input: BufferedInputStream, output: OutputStream, size: Long, flush: Boolean) {
        var remaining = size
        val buffer = ByteArray(8192)
        while (remaining > 0) {
            val count = input.read(buffer, 0, minOf(buffer.size.toLong(), remaining).toInt())
            if (count < 0) throw EOFException("Unexpected end of HTTP body")
            output.write(buffer, 0, count)
            if (flush) output.flush()
            remaining -= count
        }
    }

    private fun readHead(input: BufferedInputStream): Head? {
        val first = readLine(input) ?: return null
        val start = first.toString(StandardCharsets.ISO_8859_1)
        val headers = mutableListOf<Header>()
        var bytes = first.size + CRLF.size
        while (true) {
            val line = readLine(input) ?: throw EOFException("Unexpected end of HTTP headers")
            bytes += line.size + CRLF.size
            if (bytes > MAX_HEADER_BYTES) throw IOException("HTTP headers too large")
            if (line.isEmpty()) break
            val raw = line.toString(StandardCharsets.ISO_8859_1)
            val separator = raw.indexOf(':')
            if (separator <= 0) throw IOException("Invalid HTTP header")
            val name = raw.substring(0, separator).trim()
            val value = raw.substring(separator + 1).trim()
            if (name.isEmpty() || name.any { !isHeaderTokenChar(it) } ||
                value.any { (it.code < 32 && it != '\t') || it.code == 127 }) {
                throw IOException("Invalid HTTP header characters")
            }
            headers.add(Header(name, value))
        }
        return Head(start, headers)
    }

    private fun readLine(input: BufferedInputStream): ByteArray? {
        val line = ByteArrayOutputStream()
        var previous = -1
        while (line.size() <= MAX_HEADER_BYTES) {
            val value = input.read()
            if (value < 0) return if (line.size() == 0) null else throw EOFException("Unexpected end of HTTP line")
            if ((value == 10 && previous != 13) || (previous == 13 && value != 10)) {
                throw IOException("Invalid HTTP line ending")
            }
            if (previous == 13 && value == 10) {
                val bytes = line.toByteArray()
                return bytes.copyOf(bytes.size - 1)
            }
            line.write(value)
            previous = value
        }
        throw IOException("HTTP line too long")
    }

    private fun parseStartLine(value: String): StartLine {
        val parts = value.split(' ', limit = 3)
        if (parts.size != 3 || parts.any { it.isBlank() }) throw IOException("Invalid HTTP request line")
        if (parts[0].any { !isHeaderTokenChar(it) } || parts[1].any { it.code <= 32 || it.code == 127 } ||
            parts[2] !in setOf("HTTP/1.0", "HTTP/1.1")) {
            throw IOException("Unsupported HTTP request line")
        }
        return StartLine(parts[0], parts[1], parts[2])
    }

    private fun isHeaderTokenChar(value: Char): Boolean =
        value in 'a'..'z' || value in 'A'..'Z' || value in '0'..'9' ||
            value in "!#$%&'*+-.^_`|~"

    private fun writeHead(output: BufferedOutputStream, startLine: String, headers: List<Header>) {
        output.write("$startLine\r\n".toByteArray(StandardCharsets.ISO_8859_1))
        headers.forEach { output.write("${it.name}: ${it.value}\r\n".toByteArray(StandardCharsets.ISO_8859_1)) }
        output.write(CRLF)
    }

    private fun writeError(output: BufferedOutputStream, status: Int, reason: String) {
        val body = "Reader browser network policy rejected the request.".toByteArray(StandardCharsets.UTF_8)
        output.write("HTTP/1.1 $status $reason\r\n".toByteArray(StandardCharsets.ISO_8859_1))
        output.write("Content-Type: text/plain; charset=utf-8\r\n".toByteArray(StandardCharsets.ISO_8859_1))
        output.write("Content-Length: ${body.size}\r\nConnection: close\r\n\r\n".toByteArray(StandardCharsets.ISO_8859_1))
        output.write(body)
        output.flush()
    }

    private fun formatHost(host: String): String = if (host.contains(':')) "[$host]" else host

    override fun close() {
        if (!running.compareAndSet(true, false)) return
        runCatching { listener?.close() }
        sockets.forEach { runCatching { it.close() } }
        clients.shutdownNow()
        runCatching { clients.awaitTermination(1, TimeUnit.SECONDS) }
        runCatching { acceptor?.join(1000) }
        listener = null
        acceptor = null
    }

    private data class Head(val startLine: String, val headers: List<Header>)
    private data class Header(val name: String, val value: String)
    private data class StartLine(val method: String, val target: String, val version: String)

    private companion object {
        val CRLF = byteArrayOf(13, 10)
        const val MAX_HEADER_BYTES = 64 * 1024
        const val MAX_CONNECTIONS = 32
        val HOP_BY_HOP_REQUEST_HEADERS = setOf(
            "connection", "keep-alive", "proxy-connection", "proxy-authorization", "proxy-authenticate"
        )
        val HOP_BY_HOP_RESPONSE_HEADERS = setOf(
            "connection", "keep-alive", "proxy-connection", "proxy-authorization", "proxy-authenticate"
        )
    }
}
