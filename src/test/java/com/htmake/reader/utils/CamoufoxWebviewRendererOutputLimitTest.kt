package com.htmake.reader.utils

import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.nio.file.Files
import java.nio.file.Path

/**
 * Parent-side protocol tests deliberately use Java source-file mode as a tiny fake
 * worker. They require a JDK but not a Python/Camoufox installation, so a corrupted
 * worker cannot make the output guard regress on developer machines.
 */
class CamoufoxWebviewRendererOutputLimitTest {
    @get:Rule val temp = TemporaryFolder()
    private val renderers = mutableListOf<CamoufoxWebviewRenderer>()

    @After
    fun tearDown() = runBlocking {
        renderers.forEach { it.close() }
    }

    @Test
    fun excessiveWorkerStdoutIsRejectedAndNextRequestStillRuns() = runBlocking {
        val renderer = renderer(fakeWorker("""
            public class FakeWorker {
                public static void main(String[] args) {
                    StringBuilder body = new StringBuilder(8 * 1024 * 1024 + 1);
                    for (int i = 0; i <= 8 * 1024 * 1024; i++) body.append('x');
                    System.out.print("{\"body\":\"" + body + "\",\"cookies\":[]}");
                }
            }
        """))

        val error = try {
            renderer.render(request())
            null
        } catch (expected: IllegalStateException) {
            expected
        }
        assertTrue(error?.message?.contains("输出超过 8 MiB 限制") == true)

        // A fresh short-lived child must still be accepted after the oversized one
        // was forcibly killed; this also proves the single render queue was released.
        val healthy = renderer(fakeWorker("""
            public class FakeWorker {
                public static void main(String[] args) {
                    System.out.print("{\"body\":\"ok\",\"cookies\":[]}");
                }
            }
        """))
        assertEquals("ok", healthy.render(request()).body)
    }

    private fun renderer(script: Path): CamoufoxWebviewRenderer {
        val java = Path.of(System.getProperty("java.home"), "bin", if (isWindows()) "java.exe" else "java")
        check(Files.isRegularFile(java)) { "JDK java executable is required for fake worker test" }
        return CamoufoxWebviewRenderer(
            pythonExecutable = java.toString(),
            timeoutMs = 8_000,
            allowPrivateNetworks = true,
            workerScriptOverride = script
        ).also(renderers::add)
    }

    private fun fakeWorker(source: String): Path {
        val file = temp.newFolder("worker-${System.nanoTime()}").toPath().resolve("FakeWorker.java")
        Files.writeString(file, source.trimIndent())
        return file
    }

    private fun request() = WebviewRequest(
        // The fake worker never navigates. Use an IP literal so this protocol
        // regression does not depend on external DNS resolution.
        url = "http://127.0.0.1:1/",
        html = null,
        encode = null,
        tag = null,
        headerMap = null,
        sourceRegex = null,
        javaScript = null,
        proxy = null,
        post = false,
        body = null,
        userNameSpace = "output-limit-test",
        debugLog = null
    )

    private fun isWindows() = System.getProperty("os.name").startsWith("Windows", ignoreCase = true)
}
