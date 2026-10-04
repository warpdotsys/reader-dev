package com.htmake.reader.utils

import io.legado.app.adapters.DefaultAdpater
import io.legado.app.adapters.ReaderAdapterHelper
import io.legado.app.adapters.ReaderAdapterInterface
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.nio.file.Files
import java.nio.file.Path

/** Parent protocol regression; the tiny generated Java worker is not a Camoufox browser. */
class CamoufoxWebviewRendererNetworkDiagnosticTest {
    @get:Rule val temp = TemporaryFolder()
    private lateinit var originalUserDir: String
    private lateinit var originalAdapter: ReaderAdapterInterface

    @Before
    fun isolateStorage() {
        originalUserDir = System.getProperty("user.dir")
        originalAdapter = ReaderAdapterHelper.getAdapter()
        System.setProperty("user.dir", temp.root.absolutePath)
        ReaderAdapterHelper.setAdapter(DefaultAdpater())
    }

    @After
    fun restoreStorage() {
        System.setProperty("user.dir", originalUserDir)
        ReaderAdapterHelper.setAdapter(originalAdapter)
    }

    @Test
    fun workerNavigationErrorCannotHideAnEgressPolicyViolation() = runBlocking {
        val script = temp.root.toPath().resolve("FakeWorker.java")
        Files.writeString(script, """
            import java.io.*;
            import java.net.*;
            import java.nio.charset.StandardCharsets;
            import java.util.regex.*;
            public class FakeWorker {
                public static void main(String[] args) throws Exception {
                    String payload = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8)).readLine();
                    Matcher match = Pattern.compile("\"proxy\"\\s*:\\s*\"([^\"]+)\"").matcher(payload);
                    if (!match.find()) throw new AssertionError("No egress proxy in generated worker payload");
                    URI endpoint = URI.create(match.group(1));
                    try (Socket socket = new Socket(endpoint.getHost(), endpoint.getPort())) {
                        socket.setSoTimeout(3000);
                        socket.getOutputStream().write(("CONNECT 127.0.0.1:9 HTTP/1.1\r\n"
                            + "Host: 127.0.0.1:9\r\n\r\n").getBytes(StandardCharsets.ISO_8859_1));
                        socket.getOutputStream().flush();
                        String response = new String(socket.getInputStream().readAllBytes(), StandardCharsets.ISO_8859_1);
                        if (!response.startsWith("HTTP/1.1 403")) throw new AssertionError("Private target was not blocked");
                    }
                    System.out.print("{\"error\":\"NavigationTimeout\",\"cookies\":[]}");
                }
            }
        """.trimIndent())
        val java = Path.of(System.getProperty("java.home"), "bin",
            if (System.getProperty("os.name").startsWith("Windows", true)) "java.exe" else "java")
        val renderer = CamoufoxWebviewRenderer(java.toString(), timeoutMs = 8_000,
            allowPrivateNetworks = false, workerScriptOverride = script)
        try {
            val failure = try {
                // An IP literal avoids external DNS; the fake worker never navigates to it.
                renderer.render(WebviewRequest("https://8.8.8.8/", null, null, null, null, null,
                    null, null, false, null, "network-diagnostic-generated", null))
                null
            } catch (error: Exception) { error }
            assertTrue("The typed policy denial must take priority; observed ${failure?.javaClass?.simpleName}: ${failure?.message}",
                failure is BrowserNetworkPolicyViolation)
            assertTrue(failure?.message.orEmpty().contains("NON_PUBLIC_ADDRESS"))
        } finally {
            renderer.close()
        }
    }
}
