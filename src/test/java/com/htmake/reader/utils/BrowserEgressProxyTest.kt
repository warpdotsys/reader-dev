package com.htmake.reader.utils

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.io.ByteArrayOutputStream
import java.net.InetAddress
import java.net.ServerSocket
import java.net.Socket
import java.nio.charset.StandardCharsets
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicReference

class BrowserEgressProxyTest {
    @Test
    fun authenticatedHttpProxyReceivesPinnedAddressAndOriginalHost() {
        val upstream = ServerSocket(0, 1, InetAddress.getByName("127.0.0.1"))
        upstream.soTimeout = 3000
        val capturedRequest = AtomicReference("")
        val originProxy = Thread {
            upstream.accept().use { socket ->
                socket.soTimeout = 3000
                val request = readHead(socket)
                capturedRequest.set(request)
                socket.getOutputStream().write(
                    "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok".toByteArray(StandardCharsets.ISO_8859_1)
                )
                socket.getOutputStream().flush()
            }
        }.apply { isDaemon = true; start() }

        val loopback = InetAddress.getByName("127.0.0.1")
        val pinnedAddress = InetAddress.getByName("198.51.100.4")
        val policy = BrowserNetworkPolicy(allowPrivateNetworks = true) { host ->
            if (host == "target.example") arrayOf(pinnedAddress) else arrayOf(InetAddress.getAllByName(host).first())
        }
        val blocked = AtomicBoolean(false)
        val sourceProxy = BrowserUpstreamProxy.parse("http://127.0.0.1:${upstream.localPort}@reader@secret")
        val egress = BrowserEgressProxy(policy, 3000, { blocked.set(true) }, sourceProxy)
        var client: Socket? = null
        try {
            val proxyUrl = egress.start()
            val port = proxyUrl.substringAfterLast(':').toInt()
            client = Socket(loopback, port).apply { soTimeout = 3000 }
            client.getOutputStream().write(
                "GET http://target.example/path?q=1 HTTP/1.1\r\nHost: target.example\r\nConnection: close\r\n\r\n"
                    .toByteArray(StandardCharsets.ISO_8859_1)
            )
            client.getOutputStream().flush()
            val response = client.getInputStream().readBytes().toString(StandardCharsets.ISO_8859_1)
            originProxy.join(3000)

            assertTrue("proxied response body should be forwarded", response.endsWith("ok"))
            assertTrue("HTTP proxy request must use the pinned IP",
                capturedRequest.get().startsWith("GET http://198.51.100.4:80/path?q=1 HTTP/1.1"))
            assertTrue("original Host header must be preserved", capturedRequest.get().contains("Host: target.example"))
            assertTrue("proxy credentials must be sent only to the upstream proxy",
                capturedRequest.get().contains("Proxy-Authorization: Basic cmVhZGVyOnNlY3JldA=="))
            assertFalse("valid target should not trigger SSRF denial", blocked.get())
        } finally {
            runCatching { client?.close() }
            egress.close()
            runCatching { upstream.close() }
            originProxy.join(1000)
        }
    }

    @Test
    fun authenticatedSocks5UsesPinnedAddressAndDoesNotLeakCredentialsToOrigin() {
        val upstream = ServerSocket(0, 1, InetAddress.getByName("127.0.0.1"))
        upstream.soTimeout = 3000
        val socksTarget = AtomicReference("")
        val originRequest = AtomicReference("")
        val socksServer = Thread {
            upstream.accept().use { socket ->
                socket.soTimeout = 3000
                val input = socket.getInputStream()
                val output = socket.getOutputStream()
                val greeting = readExactly(input, 2)
                val methods = readExactly(input, greeting[1].toInt() and 0xff)
                if (greeting[0].toInt() != 5 || methods.none { it.toInt() and 0xff == 2 }) {
                    throw AssertionError("SOCKS5 authentication method was not offered")
                }
                output.write(byteArrayOf(5, 2))
                output.flush()

                val auth = readExactly(input, 2)
                val username = readExactly(input, auth[1].toInt() and 0xff).toString(StandardCharsets.UTF_8)
                val passwordLength = input.read()
                val password = readExactly(input, passwordLength).toString(StandardCharsets.UTF_8)
                if (auth[0].toInt() != 1 || username != "reader" || password != "secret") {
                    throw AssertionError("SOCKS5 credentials were not preserved")
                }
                output.write(byteArrayOf(1, 0))
                output.flush()

                val connect = readExactly(input, 4)
                val addressLength = when (connect[3].toInt() and 0xff) {
                    1 -> 4
                    4 -> 16
                    else -> throw AssertionError("SOCKS5 target was not an IP address")
                }
                val address = InetAddress.getByAddress(readExactly(input, addressLength)).hostAddress
                val portBytes = readExactly(input, 2)
                socksTarget.set("$address:${((portBytes[0].toInt() and 0xff) shl 8) or (portBytes[1].toInt() and 0xff)}")
                if (connect[0].toInt() != 5 || connect[1].toInt() != 1) {
                    throw AssertionError("SOCKS5 CONNECT command was invalid")
                }
                output.write(byteArrayOf(5, 0, 0, 1, 0, 0, 0, 0, 0, 0))
                output.flush()
                originRequest.set(readHead(socket))
                output.write(
                    "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok".toByteArray(StandardCharsets.ISO_8859_1)
                )
                output.flush()
            }
        }.apply { isDaemon = true; start() }

        val loopback = InetAddress.getByName("127.0.0.1")
        val pinnedAddress = InetAddress.getByName("198.51.100.7")
        val policy = BrowserNetworkPolicy(allowPrivateNetworks = true) { host ->
            if (host == "target.example") arrayOf(pinnedAddress) else arrayOf(InetAddress.getAllByName(host).first())
        }
        val egress = BrowserEgressProxy(
            policy, 3000, {},
            BrowserUpstreamProxy.parse("socks5://reader:secret@127.0.0.1:${upstream.localPort}")
        )
        var client: Socket? = null
        try {
            val port = egress.start().substringAfterLast(':').toInt()
            client = Socket(loopback, port).apply { soTimeout = 3000 }
            client.getOutputStream().write(
                "GET http://target.example/path HTTP/1.1\r\nHost: target.example\r\nConnection: close\r\n\r\n"
                    .toByteArray(StandardCharsets.ISO_8859_1)
            )
            client.getOutputStream().flush()
            val response = client.getInputStream().readBytes().toString(StandardCharsets.ISO_8859_1)
            socksServer.join(3000)

            assertTrue(response.endsWith("ok"))
            assertTrue(socksTarget.get().startsWith("198.51.100.7:80"))
            assertTrue(originRequest.get().startsWith("GET /path HTTP/1.1"))
            assertTrue(originRequest.get().contains("Host: target.example"))
            assertFalse("proxy credentials must not reach the origin", originRequest.get().contains("Proxy-Authorization"))
        } finally {
            runCatching { client?.close() }
            egress.close()
            runCatching { upstream.close() }
            socksServer.join(1000)
        }
    }

    @Test
    fun httpProxyUsesPinnedAddressForConnectAndRelaysTunnelBytes() {
        val upstream = ServerSocket(0, 1, InetAddress.getByName("127.0.0.1"))
        upstream.soTimeout = 3000
        val connectRequest = AtomicReference("")
        val tunnelPayload = AtomicReference("")
        val httpProxy = Thread {
            upstream.accept().use { socket ->
                socket.soTimeout = 3000
                val input = socket.getInputStream()
                val output = socket.getOutputStream()
                connectRequest.set(readHead(socket))
                output.write("HTTP/1.1 200 Connection Established\r\n\r\n".toByteArray(StandardCharsets.ISO_8859_1))
                output.flush()
                tunnelPayload.set(readExactly(input, 5).toString(StandardCharsets.ISO_8859_1))
                output.write("WORLD".toByteArray(StandardCharsets.ISO_8859_1))
                output.flush()
            }
        }.apply { isDaemon = true; start() }

        val pinnedAddress = InetAddress.getByName("198.51.100.9")
        val policy = BrowserNetworkPolicy(allowPrivateNetworks = true) { host ->
            if (host == "target.example") arrayOf(pinnedAddress) else arrayOf(InetAddress.getAllByName(host).first())
        }
        val egress = BrowserEgressProxy(policy, 3000, {},
            BrowserUpstreamProxy.parse("http://127.0.0.1:${upstream.localPort}"))
        var client: Socket? = null
        try {
            val port = egress.start().substringAfterLast(':').toInt()
            client = Socket(InetAddress.getByName("127.0.0.1"), port).apply { soTimeout = 3000 }
            client.getOutputStream().write(
                "CONNECT target.example:443 HTTP/1.1\r\nHost: target.example:443\r\n\r\n"
                    .toByteArray(StandardCharsets.ISO_8859_1)
            )
            client.getOutputStream().flush()
            val response = readHead(client)
            client.getOutputStream().write("HELLO".toByteArray(StandardCharsets.ISO_8859_1))
            client.getOutputStream().flush()
            val tunnelReply = readExactly(client.getInputStream(), 5).toString(StandardCharsets.ISO_8859_1)
            client.shutdownOutput()
            httpProxy.join(3000)

            assertTrue(response.startsWith("HTTP/1.1 200 Connection Established"))
            assertTrue(connectRequest.get().startsWith("CONNECT 198.51.100.9:443 HTTP/1.1"))
            assertTrue(connectRequest.get().contains("Host: 198.51.100.9:443"))
            assertEquals("HELLO", tunnelPayload.get())
            assertEquals("WORLD", tunnelReply)
        } finally {
            runCatching { client?.close() }
            egress.close()
            runCatching { upstream.close() }
            httpProxy.join(1000)
        }
    }

    @Test
    fun socks4UsesPinnedIpv4TargetAndPreservesOriginRequest() {
        val upstream = ServerSocket(0, 1, InetAddress.getByName("127.0.0.1"))
        upstream.soTimeout = 3000
        val socksTarget = AtomicReference("")
        val originRequest = AtomicReference("")
        val socksServer = Thread {
            upstream.accept().use { socket ->
                socket.soTimeout = 3000
                val input = socket.getInputStream()
                val output = socket.getOutputStream()
                val request = readExactly(input, 8)
                val user = ByteArrayOutputStream()
                while (true) {
                    val next = input.read()
                    if (next < 0 || next == 0) break
                    user.write(next)
                }
                val address = InetAddress.getByAddress(request.copyOfRange(4, 8)).hostAddress
                val port = ((request[2].toInt() and 0xff) shl 8) or (request[3].toInt() and 0xff)
                socksTarget.set("$address:$port")
                if (request[0].toInt() != 4 || request[1].toInt() != 1 || user.size() != 0) {
                    throw AssertionError("SOCKS4 CONNECT request was invalid")
                }
                output.write(byteArrayOf(0, 90, 0, 0, 0, 0, 0, 0))
                output.flush()
                originRequest.set(readHead(socket))
                output.write(
                    "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok".toByteArray(StandardCharsets.ISO_8859_1)
                )
                output.flush()
            }
        }.apply { isDaemon = true; start() }

        val pinnedAddress = InetAddress.getByName("198.51.100.5")
        val policy = BrowserNetworkPolicy(allowPrivateNetworks = true) { host ->
            if (host == "target.example") arrayOf(pinnedAddress) else arrayOf(InetAddress.getAllByName(host).first())
        }
        val egress = BrowserEgressProxy(policy, 3000, {},
            BrowserUpstreamProxy.parse("socks4://127.0.0.1:${upstream.localPort}"))
        var client: Socket? = null
        try {
            val port = egress.start().substringAfterLast(':').toInt()
            client = Socket(InetAddress.getByName("127.0.0.1"), port).apply { soTimeout = 3000 }
            client.getOutputStream().write(
                "GET http://target.example/path HTTP/1.1\r\nHost: target.example\r\nConnection: close\r\n\r\n"
                    .toByteArray(StandardCharsets.ISO_8859_1)
            )
            client.getOutputStream().flush()
            val response = client.getInputStream().readBytes().toString(StandardCharsets.ISO_8859_1)
            socksServer.join(3000)

            assertTrue(response.endsWith("ok"))
            assertEquals("198.51.100.5:80", socksTarget.get())
            assertTrue(originRequest.get().startsWith("GET /path HTTP/1.1"))
            assertTrue(originRequest.get().contains("Host: target.example"))
        } finally {
            runCatching { client?.close() }
            egress.close()
            runCatching { upstream.close() }
            socksServer.join(1000)
        }
    }

    private fun readHead(socket: Socket): String {
        val input = socket.getInputStream()
        val bytes = ByteArrayOutputStream()
        var matched = 0
        val terminator = byteArrayOf(13, 10, 13, 10)
        while (bytes.size() <= 64 * 1024) {
            val value = input.read()
            if (value < 0) break
            bytes.write(value)
            matched = if (value.toByte() == terminator[matched]) matched + 1
                else if (value == 13) 1 else 0
            if (matched == terminator.size) break
        }
        return bytes.toByteArray().toString(StandardCharsets.ISO_8859_1)
    }

    private fun readExactly(input: java.io.InputStream, count: Int): ByteArray {
        val bytes = ByteArray(count)
        var offset = 0
        while (offset < count) {
            val read = input.read(bytes, offset, count - offset)
            if (read < 0) throw java.io.EOFException("Unexpected end of SOCKS5 test stream")
            offset += read
        }
        return bytes
    }
}
