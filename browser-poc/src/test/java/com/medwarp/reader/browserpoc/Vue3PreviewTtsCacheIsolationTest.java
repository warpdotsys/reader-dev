package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.*;
import org.junit.Assume;
import org.junit.Test;
import java.net.URI;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicInteger;
import static org.junit.Assert.*;

/** Actual pages/Cache Storage/audio playback with generated text and a synthetic WAV response.
 * The TTS HTTP response is intentionally intercepted: this does not test an external voice provider. */
public class Vue3PreviewTtsCacheIsolationTest {
    private static final String PASSWORD = "Generated-Tts-Probe-20261007";
    private static void loopback(String url) {
        URI uri = URI.create(url);
        assertEquals("http", uri.getScheme());
        assertEquals("127.0.0.1", uri.getHost());
    }
    private static void signIn(Page page, String preview, String user, boolean register) {
        page.navigate(preview + "/login");
        if (register) page.locator(".mode-switch button").nth(1).click();
        page.locator("input[autocomplete=username]").fill(user);
        page.locator("input[autocomplete=current-password]").fill(PASSWORD);
        page.locator(".submit-btn").click();
        page.locator(".bookshelf-page").waitFor();
    }
    private static void logout(Page page, String preview) {
        page.navigate(preview + "/");
        page.locator(".bookshelf-page").waitFor();
        page.locator(".logout-btn").click();
        page.locator(".login-page").waitFor();
    }
    private static byte[] generatedWave() {
        int samples = 8000 * 8;
        ByteBuffer buffer = ByteBuffer.allocate(44 + samples * 2).order(ByteOrder.LITTLE_ENDIAN);
        buffer.put("RIFF".getBytes(StandardCharsets.US_ASCII)).putInt(36 + samples * 2);
        buffer.put("WAVEfmt ".getBytes(StandardCharsets.US_ASCII)).putInt(16).putShort((short) 1)
                .putShort((short) 1).putInt(8000).putInt(16000).putShort((short) 2).putShort((short) 16);
        buffer.put("data".getBytes(StandardCharsets.US_ASCII)).putInt(samples * 2);
        for (int i = 0; i < samples; i++) buffer.putShort((short) (500 * Math.sin(i * 2 * Math.PI * 220 / 8000)));
        return buffer.array();
    }
    private static void edit(Page page, String text) {
        Locator panel = page.locator(".tts-card");
        if (panel.isVisible()) {
            panel.locator("xpath=..").click(new Locator.ClickOptions().setPosition(8, 8));
            panel.waitFor(new Locator.WaitForOptions().setState(com.microsoft.playwright.options.WaitForSelectorState.HIDDEN));
        }
        page.locator("button[title='编辑本章正文并保存（服务器 + 本机缓存）']").click();
        page.locator(".edit-textarea").fill(text);
        Response response = page.waitForResponse(r -> URI.create(r.url()).getPath().endsWith("/reader3/saveBookContent"),
                () -> page.locator(".edit-card .pop-btn").click());
        assertEquals(200, response.status());
        assertTrue(response.text().contains("\"isSuccess\":true"));
        page.locator(".reader-content").getByText(text, new Locator.GetByTextOptions().setExact(true)).waitFor();
    }
    private static void play(Page page) {
        page.evaluate("() => { document.querySelector('audio').muted = true; }");
        page.locator(".tts-btn").click();
        page.waitForCondition(() -> Boolean.TRUE.equals(page.evaluate("() => { const a=document.querySelector('audio');"
                + "return a && a.src.startsWith('blob:') && !a.paused && a.currentTime > 0; }")));
    }

    private static void screenshot(Page page, String name) {
        String root = System.getenv("READER_UI_EVIDENCE_DIR");
        if (root == null) root = System.getenv("RUNNER_TEMP");
        if (root != null) page.screenshot(new Page.ScreenshotOptions().setPath(Path.of(root, name)));
    }
    @SuppressWarnings("unchecked")
    private static List<Map<String, Object>> cacheRecords(Page page) {
        return (List<Map<String, Object>>) page.evaluate("async () => { const records=[];"
                + "for(const name of await caches.keys()){ if(!name.startsWith('tts-audio-v2:')) continue;"
                + "const cache=await caches.open(name); const keys=await cache.keys();"
                + "records.push({name,count:keys.length,keys:keys.map(r=>new URL(r.url).pathname)});}"
                + "return records; }");
    }
    private static int entryCount(Page page) {
        return cacheRecords(page).stream().mapToInt(r -> ((Number) r.get("count")).intValue()).sum();
    }

    @Test
    public void fullTextAudioCacheAndSettingsRemainInTheirOwnAccount() throws Exception {
        String preview = System.getenv("READER_VUE3_PREVIEW_URL");
        String fixture = System.getenv("READER_BOOK_FIXTURE_URL");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue("This UI journey requires an explicitly isolated fixture",
                "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        // Opting in must fail on a missing browser/fixture, not masquerade as successful Gradle with a skipped test.
        assertNotNull("READER_VUE3_PREVIEW_URL is required", preview);
        assertNotNull("READER_BOOK_FIXTURE_URL is required", fixture);
        assertFalse("READER_BROWSER_EXECUTABLE is required", executable.isEmpty());
        assertTrue("The selected test browser must exist", Files.isRegularFile(Path.of(executable)));
        loopback(preview);
        loopback(fixture);
        String suffix = UUID.randomUUID().toString().replace("-", "").substring(0, 10);
        String first = "ttsa" + suffix, second = "ttsb" + suffix;
        String setup = new String(getClass().getResourceAsStream("/vue3-reading-setup.js").readAllBytes(), StandardCharsets.UTF_8);
        String reader = preview + "/reader/" + java.net.URLEncoder.encode(fixture + "/book", StandardCharsets.UTF_8) + "?chapter=0";
        String textA = "生成".repeat(2500) + "甲", textB = "生成".repeat(2500) + "乙";
        assertEquals(5001, textA.length());
        assertEquals(textA.length(), textB.length());
        byte[] wave = generatedWave();
        AtomicInteger requests = new AtomicInteger();
        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")));
             Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                     .setExecutablePath(Path.of(executable)).setHeadless(true));
             BrowserContext context = browser.newContext()) {
            Page page = context.newPage();
            page.setDefaultTimeout(15000);
            page.route("**/reader3/book/tts**", route -> {
                assertEquals("POST", route.request().method());
                assertTrue(route.request().postData().contains("\"text\":"));
                requests.incrementAndGet();
                route.fulfill(new Route.FulfillOptions().setStatus(200).setContentType("audio/wav").setBodyBytes(wave));
            });
            signIn(page, preview, first, true);
            page.evaluate(setup, fixture);
            page.evaluate("async () => { const c=await caches.open('tts-audio-v1');"
                    + "await c.put('/tts/generated-unowned.mp3',new Response('generated unowned old cache')); }");
            page.navigate(reader);
            page.locator(".reader-content").getByText("第一段，中文与 UTF-8。").waitFor();
            edit(page, textA);
            play(page);
            page.waitForCondition(() -> entryCount(page) == 1);
            assertEquals("A fresh profile must retain the 100% default volume", "+0%",
                    page.locator(".tts-card .set-row").filter(new Locator.FilterOptions().setHasText("音量"))
                            .locator(".set-value").innerText());
            screenshot(page, "tts-cache-generated-a-playing.png");
            assertEquals(1, requests.get());
            page.reload();
            page.locator(".reader-content").getByText(textA, new Locator.GetByTextOptions().setExact(true)).waitFor();
            play(page);
            assertEquals("Actual cache hit must not send another TTS request", 1, requests.get());
            page.locator(".tts-stop").click();
            edit(page, textB);
            play(page);
            page.waitForCondition(() -> entryCount(page) == 2);
            assertEquals("Equal-length changed tail must synthesize a distinct clip", 2, requests.get());
            Map<String,Object> firstRecord = cacheRecords(page).get(0);

            logout(page, preview);
            signIn(page, preview, second, true);
            page.evaluate(setup, fixture);
            page.navigate(reader);
            page.locator(".reader-content").getByText("第一段，中文与 UTF-8。").waitFor();
            edit(page, textA);
            play(page);
            page.waitForCondition(() -> entryCount(page) == 3);
            assertEquals("B cannot use A's audio at identical text and synthesis parameters", 3, requests.get());
            List<Map<String,Object>> records = cacheRecords(page);
            assertEquals(2, records.size());
            String secondCacheName = records.stream().map(record -> (String) record.get("name"))
                    .filter(name -> !name.equals(firstRecord.get("name"))).findFirst().orElseThrow();
            for (Map<String,Object> record : records) {
                assertTrue(((String) record.get("name")).matches("tts-audio-v2:[0-9a-f]{64}"));
                assertFalse(((String) record.get("name")).contains(first));
                for (Object key : (List<?>) record.get("keys"))
                    assertTrue(key.toString().matches("/tts/v2/[0-9a-f]{64}\\.mp3"));
            }
            page.navigate(preview + "/settings");
            Locator row = page.locator(".row").filter(new Locator.FilterOptions().setHasText("听书音频"));
            page.waitForCondition(() -> row.locator(".row-value").innerText().startsWith("1 条"));
            row.scrollIntoViewIfNeeded();
            screenshot(page, "tts-cache-generated-b-settings.png");
            row.locator(".cache-clear").click();
            page.waitForCondition(() -> entryCount(page) == 2);
            page.waitForCondition(() -> row.locator(".row-value").innerText().startsWith("0 条"));
            List<Map<String,Object>> afterClear = cacheRecords(page);
            assertEquals("A's keys and clips must remain unchanged", firstRecord,
                    afterClear.stream().filter(record -> record.get("name").equals(firstRecord.get("name")))
                            .findFirst().orElseThrow());
            // Refreshing the displayed statistics may reopen B's empty cache. Entries, not names, are the contract.
            for (Map<String,Object> record : afterClear) if (record.get("name").equals(secondCacheName))
                assertEquals(0, ((Number) record.get("count")).intValue());
            assertEquals(true, page.evaluate("async () => (await caches.keys()).includes('tts-audio-v1')"));
            screenshot(page, "tts-cache-generated-b-cleared.png");

            logout(page, preview);
            signIn(page, preview, first, false);
            page.navigate(reader);
            page.locator(".reader-content").getByText(textB, new Locator.GetByTextOptions().setExact(true)).waitFor();
            play(page);
            assertEquals("B clearing its own cache must not remove A's clips", 3, requests.get());
            assertEquals(2, entryCount(page));
            screenshot(page, "tts-cache-generated-a-returned.png");
        }
    }
}
