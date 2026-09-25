package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.Browser;
import com.microsoft.playwright.BrowserType;
import com.microsoft.playwright.Playwright;
import com.microsoft.playwright.PlaywrightException;
import com.sun.net.httpserver.HttpServer;
import org.junit.AfterClass;
import org.junit.Assume;
import org.junit.BeforeClass;
import org.junit.Test;

import java.io.IOException;
import java.net.InetSocketAddress;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Collections;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

public class BrowserProbeTest {
    private static HttpServer server;
    private static Playwright playwright;
    private static Browser browser;
    private static BrowserProbe probe;
    private static String baseUrl;

    @BeforeClass
    public static void start() throws IOException {
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue("Set READER_BROWSER_EXECUTABLE to run browser tests",
                !executable.isEmpty() && Files.isRegularFile(Path.of(executable)));
        server = HttpServer.create(new InetSocketAddress("127.0.0.1", 0), 0);
        server.createContext("/", exchange -> {
            if (exchange.getRequestURI().getPath().equals("/slow")) {
                try {
                    Thread.sleep(1500);
                } catch (InterruptedException e) {
                    Thread.currentThread().interrupt();
                }
            }
            String body = new String(exchange.getRequestBody().readAllBytes(), StandardCharsets.UTF_8);
            String cookie = exchange.getRequestHeaders().getFirst("Cookie");
            String marker = exchange.getRequestMethod() + "|" + body + "|"
                    + exchange.getRequestHeaders().getFirst("X-Probe") + "|" + cookie;
            if (exchange.getRequestURI().getPath().equals("/seed")) {
                exchange.getResponseHeaders().add("Set-Cookie", "sid=alpha; Path=/; HttpOnly");
            }
            exchange.getResponseHeaders().set("Content-Type", "text/html; charset=UTF-8");
            byte[] response = ("<!doctype html><html><body><div id='result'>" + marker
                    + "</div><script>document.querySelector('#result').dataset.ready='yes'</script>"
                    + "</body></html>").getBytes(StandardCharsets.UTF_8);
            try {
                exchange.sendResponseHeaders(200, response.length);
                exchange.getResponseBody().write(response);
            } finally {
                exchange.close();
            }
        });
        server.start();
        baseUrl = "http://127.0.0.1:" + server.getAddress().getPort();
        playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Collections.singletonMap("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")));
        browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                .setExecutablePath(Path.of(executable))
                .setHeadless(true));
        probe = new BrowserProbe(browser);
    }

    @AfterClass
    public static void stop() {
        if (browser != null) browser.close();
        if (playwright != null) playwright.close();
        if (server != null) server.stop(0);
    }

    @Test
    public void getHeadersAndScript() {
        BrowserProbe.Result result = probe.render(new BrowserProbe.Request(
                baseUrl + "/echo", null, "get-user", null,
                Collections.singletonMap("X-Probe", "header-ok"),
                "document.querySelector('#result').textContent", null, 10000));
        assertEquals(200, result.status);
        assertTrue(result.html.contains("data-ready=\"yes\""));
        assertTrue(String.valueOf(result.scriptResult).contains("GET||header-ok"));
    }

    @Test
    public void postBody() {
        BrowserProbe.Result result = probe.render(new BrowserProbe.Request(
                baseUrl + "/echo", null, "post-user", "page=2",
                Collections.singletonMap("Content-Type", "application/x-www-form-urlencoded"),
                "document.querySelector('#result').textContent", null, 10000));
        assertEquals(200, result.status);
        assertTrue(String.valueOf(result.scriptResult).contains("POST|page=2"));
    }

    @Test
    public void cookiesDoNotCrossNamespaces() {
        probe.render(new BrowserProbe.Request(baseUrl + "/seed", null, "alice", null,
                null, null, null, 10000));
        BrowserProbe.Result sameUser = probe.render(new BrowserProbe.Request(
                baseUrl + "/echo", null, "alice", null, null,
                "document.querySelector('#result').textContent", null, 10000));
        BrowserProbe.Result otherUser = probe.render(new BrowserProbe.Request(
                baseUrl + "/echo", null, "bob", null, null,
                "document.querySelector('#result').textContent", null, 10000));
        assertTrue(String.valueOf(sameUser.scriptResult).contains("sid=alpha"));
        assertFalse(String.valueOf(otherUser.scriptResult).contains("sid=alpha"));
    }

    @Test(expected = PlaywrightException.class)
    public void navigationTimesOut() {
        probe.render(new BrowserProbe.Request(baseUrl + "/slow", null, "slow-user", null,
                null, null, null, 500));
    }
}
