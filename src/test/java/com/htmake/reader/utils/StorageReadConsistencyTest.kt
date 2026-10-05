package com.htmake.reader.utils

import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.fail
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.io.File
import java.nio.file.Files
import java.nio.file.StandardCopyOption
import java.util.concurrent.CountDownLatch
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import java.util.concurrent.TimeoutException
import java.util.concurrent.locks.ReadWriteLock

/** Generated-only storage fixtures: no production user registry or book is opened. */
class StorageReadConsistencyTest {
    @get:Rule val temp = TemporaryFolder()
    private lateinit var originalStoragePath: String

    @Before
    fun setUp() {
        originalStoragePath = storageFinalPath
        storageFinalPath = temp.root.absolutePath
    }

    @After
    fun tearDown() {
        storageFinalPath = originalStoragePath
    }

    @Test
    fun readerMustWaitThroughTheWritersAtomicReplacementGap() {
        val payload = "{\"generated\":\"存储一致性\"}"
        saveStorage("generated", "item", value = payload)
        val file = getStorageFile("generated", "item")
        assertEquals(temp.root.canonicalFile, file.parentFile.parentFile.canonicalFile)
        // Hold the actual lock used by saveStorage, then reproduce its two-move
        // replacement window. Reflection avoids adding a production test hook.
        val lockMethod = Class.forName("com.htmake.reader.utils.ExtKt__VertExtKt")
            .declaredMethods.single { it.name.startsWith("storageLock") &&
                it.parameterTypes.contentEquals(arrayOf(File::class.java)) }.apply { isAccessible = true }
        val lock = lockMethod.invoke(null, file) as ReadWriteLock
        val backup = file.toPath().resolveSibling("item.generated-backup.json")
        val workers = Executors.newFixedThreadPool(2)
        val gapOpen = CountDownLatch(1)
        val restore = CountDownLatch(1)
        val readerStarted = CountDownLatch(1)
        try {
            val writer = workers.submit {
                lock.writeLock().lock()
                try {
                    Files.move(file.toPath(), backup, StandardCopyOption.ATOMIC_MOVE)
                    gapOpen.countDown()
                    check(restore.await(5, TimeUnit.SECONDS))
                    Files.move(backup, file.toPath(), StandardCopyOption.ATOMIC_MOVE)
                } finally {
                    lock.writeLock().unlock()
                }
            }
            check(gapOpen.await(5, TimeUnit.SECONDS))
            assertFalse(file.exists())
            val reader = workers.submit<String?> {
                readerStarted.countDown()
                getStorage("generated", "item")
            }
            check(readerStarted.await(5, TimeUnit.SECONDS))
            try {
                reader.get(250, TimeUnit.MILLISECONDS)
                fail("A reader must not treat the locked replacement gap as missing data")
            } catch (_: TimeoutException) {
                // Correct reader waits for the writer, including the existence check.
            } finally {
                restore.countDown()
            }
            writer.get(5, TimeUnit.SECONDS)
            assertEquals(payload, reader.get(5, TimeUnit.SECONDS))
        } finally {
            restore.countDown()
            workers.shutdownNow()
            check(workers.awaitTermination(5, TimeUnit.SECONDS))
        }
    }

    @Test
    fun trulyMissingGeneratedStorageStillReturnsNull() {
        assertNull(getStorage("generated", "missing"))
    }

    @Test
    fun generatedUserRegistryKeepsItsExistingIntegrityContract() {
        val payload = "{\"generated\":{\"username\":\"generated-user\"}}"
        saveStorage("data", "users", value = payload)
        assertEquals(payload, getStorage("data", "users"))
        getStorageFile("data", "users").writeText("{}", Charsets.UTF_8)
        try {
            getStorage("data", "users")
            fail("A changed generated user registry must still be rejected")
        } catch (_: Exception) {
            // Existing integrity check stays enabled; this is not a compatibility bypass.
        }
    }
}
