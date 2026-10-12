package com.htmake.reader.utils

import io.vertx.core.VertxOptions
import io.vertx.core.json.JsonObject
import org.junit.Assert.*
import org.junit.Test

class ReaderVertxOptionsTest {
    private val worker = "READER_VERTX_WORKER_POOL_SIZE"
    private val internal = "READER_VERTX_INTERNAL_BLOCKING_POOL_SIZE"
    private fun snapshot(options: VertxOptions): JsonObject = JsonObject.mapFrom(options)

    @Test fun absentSettingsPreserveEveryOriginalVertxOption() {
        val original = VertxOptions()
        assertEquals(20, original.workerPoolSize)
        assertEquals(20, original.internalBlockingPoolSize)
        assertEquals(snapshot(original), snapshot(readerVertxOptions(emptyMap())))
    }

    @Test fun boundedImageSettingsChangeOnlyTheTwoWorkerPools() {
        val expected = VertxOptions().setWorkerPoolSize(8).setInternalBlockingPoolSize(8)
        assertEquals(snapshot(expected), snapshot(readerVertxOptions(mapOf(worker to "8", internal to "8"))))
    }

    @Test fun oneSettingDoesNotSilentlyChangeTheOtherPool() {
        val options = readerVertxOptions(mapOf(worker to "6"))
        assertEquals(6, options.workerPoolSize)
        assertEquals(20, options.internalBlockingPoolSize)
    }

    @Test fun independentExplicitSettingsAndLimitsAreAccepted() {
        val options = readerVertxOptions(mapOf(worker to "1", internal to "128"))
        assertEquals(1, options.workerPoolSize)
        assertEquals(128, options.internalBlockingPoolSize)
    }

    @Test fun malformedValuesFailClosedForEitherPool() {
        for (name in listOf(worker, internal)) {
            for (raw in listOf("", "0", "-1", "01", "129", "1000", "8.0", " 8", "8\n", "secret-value")) {
                try {
                    readerVertxOptions(mapOf(name to raw))
                    fail("Expected malformed pool setting to be rejected")
                } catch (error: IllegalArgumentException) {
                    assertTrue(error.message!!.contains(name))
                    assertFalse(error.message!!.contains("secret-value"))
                }
            }
        }
    }

    @Test fun unrelatedEnvironmentDoesNotAffectLegacyOptions() {
        assertEquals(snapshot(VertxOptions()), snapshot(readerVertxOptions(mapOf("OTHER" to "ignored"))))
    }

    @Test fun eventLoopAndTimeoutPoliciesAreNotChangedByWorkerConfiguration() {
        val original = VertxOptions()
        val configured = readerVertxOptions(mapOf(worker to "8", internal to "8"))
        assertEquals(original.eventLoopPoolSize, configured.eventLoopPoolSize)
        assertEquals(original.blockedThreadCheckInterval, configured.blockedThreadCheckInterval)
        assertEquals(original.maxEventLoopExecuteTime, configured.maxEventLoopExecuteTime)
        assertEquals(original.maxWorkerExecuteTime, configured.maxWorkerExecuteTime)
    }
}
