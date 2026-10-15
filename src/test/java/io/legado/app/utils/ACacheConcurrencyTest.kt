package io.legado.app.utils

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.io.File
import java.io.IOException
import java.util.concurrent.CountDownLatch
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger

/** Generated cache bytes only; never open Reader storage or actual book metadata. */
class ACacheConcurrencyTest {
    @get:Rule val temp = TemporaryFolder()

    @Test
    fun stringBinaryAndHashLookupKeepLegacyFileNamesAndBytes() {
        val directory = temp.newFolder("generated-legacy-format")
        val cache = ACache.get(directory)
        val key = "generated-utf8-key"
        val text = "生成中文 UTF-8\nline two"
        cache.put(key, text)
        val file = File(directory, key.hashCode().toString())
        assertEquals(text, file.readText(Charsets.UTF_8))
        assertEquals(text, cache.getAsString(key))
        assertEquals(text, cache.getByHashCode(key.hashCode().toString()))
        val bytes = byteArrayOf(0, 1, -1, 80, 75)
        cache.put(key, bytes)
        assertTrue(bytes.contentEquals(file.readBytes()))
        assertTrue(bytes.contentEquals(cache.getAsBinary(key)))
        assertEquals(setOf(file.name), directory.listFiles()!!.map { it.name }.toSet())
    }

    @Test
    fun failedAtomicPublicationPreservesTheTargetAndRemovesOwnedTemporaryFile() {
        val directory = temp.newFolder("generated-failed-publication")
        val target = File(directory, "generated-directory-target").apply { mkdir() }
        val marker = File(target, "generated-retained-marker").apply { writeText("retain") }
        try {
            publishCacheBytes(target, byteArrayOf(1, 2))
            fail("A directory cannot be replaced by a cache file")
        } catch (_: IOException) {
            assertEquals("retain", marker.readText())
            assertEquals(setOf(target.name), directory.listFiles()!!.map { it.name }.toSet())
        }
    }

    @Test
    fun concurrentStringReadersOnlySeeOneCompletePublishedValue() {
        val cache = ACache.get(temp.newFolder("generated-concurrent-cache"), Long.MAX_VALUE, Int.MAX_VALUE)
        val key = "https://example.invalid/generated-book"
        val values = listOf(
            "{\"generated\":0,\"text\":\"" + "a".repeat(65536) + "\"}",
            "{\"generated\":1,\"text\":\"" + "b".repeat(262144) + "\"}",
            "{\"generated\":2,\"text\":\"" + "中文".repeat(32768) + "\"}",
        )
        cache.put(key, values[0])
        assertEquals(values[0], cache.getAsString(key))
        val allowed = values.toSet()
        val invalid = AtomicInteger()
        val workers = Executors.newFixedThreadPool(4)
        val ready = CountDownLatch(4)
        val start = CountDownLatch(1)
        try {
            val tasks = (0..3).map { index ->
                workers.submit {
                    ready.countDown()
                    check(start.await(5, TimeUnit.SECONDS))
                    if (index < 2) repeat(80) {
                        synchronized(cache) {
                            cache.put(key, values[index + 1])
                            assertEquals("Every normal writer update must really be published", values[index + 1], cache.getAsString(key))
                        }
                    }
                    else repeat(200) { if (cache.getAsString(key) !in allowed) invalid.incrementAndGet() }
                }
            }
            assertTrue(ready.await(5, TimeUnit.SECONDS))
            start.countDown()
            tasks.forEach { it.get(15, TimeUnit.SECONDS) }
            assertEquals("Cache readers must not receive truncated, interleaved, empty or missing bytes", 0, invalid.get())
            assertTrue("At least one new writer value must actually be published", cache.getAsString(key) in values.drop(1))
        } finally {
            start.countDown()
            workers.shutdownNow()
            assertTrue(workers.awaitTermination(5, TimeUnit.SECONDS))
        }
    }
}
