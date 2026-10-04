package com.htmake.reader.utils

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.net.InetAddress
import java.net.UnknownHostException
import java.nio.charset.StandardCharsets
import java.security.MessageDigest

class BrowserNetworkPolicyDiagnosticTest {
    @Test
    fun benchmarkDnsRangeIsDistinguishedWithoutPublishingUrlOrHost() {
        val policy = BrowserNetworkPolicy(resolve = { arrayOf(InetAddress.getByName("198.18.1.220")) })
        val first = denied(policy, "https://CDN.example./private/generated-path?session=generated-secret#fragment")
        val second = denied(policy, "https://cdn.example/another-path")
        assertTrue(first.contains("BENCHMARK_RANGE"))
        assertTrue(first.contains(fingerprint("cdn.example")))
        assertEquals(first, second)
        listOf("generated-secret", "generated-path", "session", "fragment", "cdn.example", "198.18.1.220")
            .forEach { assertFalse("Diagnostic must not contain $it", first.contains(it, ignoreCase = true)) }
    }

    @Test
    fun mixedAnswersStillFailClosedAndReportNonPublicAddress() {
        val policy = BrowserNetworkPolicy(resolve = {
            arrayOf(InetAddress.getByName("8.8.8.8"), InetAddress.getByName("10.0.0.8"))
        })
        val message = denied(policy, "https://mixed.example/path?credential=generated-secret")
        assertTrue(message.contains("NON_PUBLIC_ADDRESS"))
        assertTrue(message.contains(fingerprint("mixed.example")))
        assertFalse(message.contains("generated-secret"))
    }

    @Test
    fun unresolvedAndEmptyDnsHaveDifferentSanitizedReasons() {
        val unresolved = BrowserNetworkPolicy(resolve = { throw UnknownHostException("generated-secret") })
        val empty = BrowserNetworkPolicy(resolve = { emptyArray() })
        assertTrue(denied(unresolved, "https://source.example/").contains("DNS_FAILURE"))
        assertTrue(denied(empty, "https://source.example/").contains("EMPTY_DNS"))
        assertFalse(denied(unresolved, "https://source.example/").contains("generated-secret"))
    }

    @Test
    fun credentialsMalformedUrlsAndUnsupportedSchemesNeverEchoInput() {
        val policy = BrowserNetworkPolicy(resolve = { arrayOf(InetAddress.getByName("8.8.8.8")) })
        val cases = listOf(
            "https://generated-user:generated-secret@source.example/" to "URL_CREDENTIALS",
            "https://source.example/generated secret" to "MALFORMED_URL",
            "file:///generated-secret" to "UNSUPPORTED_SCHEME",
            "https://source.example:0/" to "INVALID_PORT",
            "http://generated-secret.local/" to "LOCAL_HOST"
        )
        cases.forEach { (url, reason) ->
            val message = denied(policy, url)
            assertTrue("Expected $reason", message.contains(reason))
            assertFalse(message.contains("generated-secret"))
            assertFalse(message.contains("generated-user"))
        }
    }

    private fun denied(policy: BrowserNetworkPolicy, url: String): String {
        try {
            policy.requireDocumentUrl(url)
            throw AssertionError("Expected a rejected generated URL")
        } catch (failure: BrowserNetworkPolicyViolation) {
            return failure.message.orEmpty()
        }
    }

    private fun fingerprint(host: String): String = MessageDigest.getInstance("SHA-256")
        .digest(host.toByteArray(StandardCharsets.UTF_8)).take(8)
        .joinToString("") { "%02x".format(it.toInt() and 0xff) }
}
