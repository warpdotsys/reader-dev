package com.htmake.reader.utils

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Test

class ChromiumLaunchPolicyTest {
    @Test
    fun backgroundFeatureOverridePreservesEveryPinnedDriverDefault() {
        val resource = javaClass.classLoader.getResourceAsStream("driver/package/lib/coreBundle.js")
        assertNotNull("The pinned Playwright driver resource is required", resource)
        val source = resource!!.bufferedReader(Charsets.UTF_8).use { it.readText() }
        val block = Regex("disabledFeatures = \\[(.*?)\\]\\.filter\\(Boolean\\)",
            RegexOption.DOT_MATCHES_ALL).find(source)
        assertNotNull("Review ChromiumLaunchPolicy after a Playwright bundle change", block)
        val actualDefaults = Regex("\"([A-Za-z][A-Za-z0-9]+)\"")
            .findAll(block!!.groupValues[1]).map { it.groupValues[1] }.toList()
        assertEquals("Do not silently remove or invent a pinned Playwright default",
            actualDefaults, ChromiumLaunchPolicy.playwrightDisabledFeatures)
        assertEquals(actualDefaults + listOf("PreconnectToSearch", "PreconnectFromKeyedService", "AimEnabled"),
            ChromiumLaunchPolicy.disabledFeaturesArgument().substringAfter('=').split(','))
    }
}
