package com.htmake.reader.utils

import org.junit.Assert.fail
import org.junit.Test
import java.net.InetAddress
import java.net.IDN

class BrowserNetworkPolicyTest {
    private val publicAddress = InetAddress.getByName("8.8.8.8")

    private fun assertBlocked(policy: BrowserNetworkPolicy, url: String) {
        try {
            policy.requireDocumentUrl(url)
            fail("Expected browser network policy to block $url")
        } catch (_: BrowserNetworkPolicyViolation) {
            // Expected.
        }
    }

    @Test
    fun allowsPublicTargetsAndRejectsMixedPublicAndPrivateDnsAnswers() {
        val publicOnly = BrowserNetworkPolicy(resolve = { arrayOf(publicAddress) })
        publicOnly.requireDocumentUrl("https://source.example/search")

        val mixedAnswers = BrowserNetworkPolicy(resolve = {
            arrayOf(publicAddress, InetAddress.getByName("10.0.0.8"))
        })
        assertBlocked(mixedAnswers, "https://source.example/search")
    }

    @Test
    fun normalizesInternationalizedHostnamesBeforeDnsValidation() {
        val expectedHost = IDN.toASCII("例子.测试", IDN.USE_STD3_ASCII_RULES)
        val policy = BrowserNetworkPolicy(resolve = { host ->
            if (host == expectedHost) arrayOf(publicAddress) else emptyArray()
        })
        policy.requireDocumentUrl("https://例子.测试/search")
    }

    @Test
    fun blocksLoopbackPrivateLinkLocalAndSpecialAddressRanges() {
        val policy = BrowserNetworkPolicy()
        listOf(
            "http://127.0.0.1/admin",
            "http://10.20.30.40/",
            "http://172.20.1.2/",
            "http://192.168.1.1/",
            "http://100.64.0.1/",
            "http://169.254.169.254/latest/meta-data",
            "http://198.51.100.1/",
            "http://[fd00::1]/",
            "http://[2001:db8::1]/"
        ).forEach { assertBlocked(policy, it) }
    }

    @Test
    fun rejectsLocalNamesCredentialsAndNonNetworkSchemes() {
        val policy = BrowserNetworkPolicy(resolve = { arrayOf(publicAddress) })
        assertBlocked(policy, "http://localhost/admin")
        assertBlocked(policy, "http://printer.local/")
        assertBlocked(policy, "https://user:secret@source.example/")
        assertBlocked(policy, "file:///etc/passwd")
        policy.requireRequestUrl("data:text/plain,ok")
        policy.requireRequestUrl("blob:https://source.example/id")
        assertBlockedRequest(policy, "ws://127.0.0.1:9000/socket")
    }

    @Test
    fun privateNetworksRequireExplicitOptInButSchemesRemainRestricted() {
        val policy = BrowserNetworkPolicy(allowPrivateNetworks = true)
        policy.requireDocumentUrl("http://127.0.0.1:18890/search")
        try {
            policy.requireRequestUrl("file:///etc/passwd")
            fail("file: must remain blocked when private network access is enabled")
        } catch (_: BrowserNetworkPolicyViolation) {
            // Expected.
        }
    }

    private fun assertBlockedRequest(policy: BrowserNetworkPolicy, url: String) {
        try {
            policy.requireRequestUrl(url)
            fail("Expected browser network policy to block $url")
        } catch (_: BrowserNetworkPolicyViolation) {
            // Expected.
        }
    }
}
