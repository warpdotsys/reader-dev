package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.Browser;
import com.microsoft.playwright.BrowserType;
import com.microsoft.playwright.Locator;
import com.microsoft.playwright.Page;
import com.microsoft.playwright.Playwright;
import org.junit.Assume;
import org.junit.Test;

import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import java.util.UUID;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

/** Real layout checks against synthetic books, never production data. */
public class Vue3PreviewShelfLayoutTest {
    private static void requireLoopback(String value) {
        URI uri = URI.create(value);
        assertEquals("http", uri.getScheme());
        assertTrue("Only loopback fixtures are allowed", "127.0.0.1".equals(uri.getHost())
                || "localhost".equals(uri.getHost()));
    }

    private static void assertNoHorizontalOverflow(Page page, String scenario) {
        @SuppressWarnings("unchecked")
        Map<String, Object> bounds = (Map<String, Object>) page.evaluate("() => ({"
                + "viewport: document.documentElement.clientWidth,"
                + "document: document.documentElement.scrollWidth})");
        assertTrue(scenario + ": " + bounds,
                ((Number) bounds.get("document")).doubleValue()
                        <= ((Number) bounds.get("viewport")).doubleValue());
    }

    private static String resource(String name) throws Exception {
        try (java.io.InputStream input = Vue3PreviewShelfLayoutTest.class
                .getResourceAsStream("/" + name)) {
            if (input == null) throw new IllegalStateException("Missing fixture resource: " + name);
            return new String(input.readAllBytes(), StandardCharsets.UTF_8);
        }
    }

    @Test
    public void hiddenAndHoveredPreviewsStayInsideShelfAtAllDensities() throws Exception {
        String previewUrl = System.getenv("READER_VUE3_PREVIEW_URL");
        String fixtureUrl = System.getenv("READER_BOOK_FIXTURE_URL");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue(previewUrl != null && fixtureUrl != null
                && !executable.isEmpty() && Files.isRegularFile(Path.of(executable))
                && "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        requireLoopback(previewUrl);
        requireLoopback(fixtureUrl);

        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")))) {
            Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                    .setExecutablePath(Path.of(executable)).setHeadless(true));
            try {
                Page page = browser.newPage();
                page.setDefaultTimeout(60000);
                page.setViewportSize(1135, 865);
                page.navigate(previewUrl + "/login");
                page.locator(".mode-switch button").nth(1).click();
                page.locator("input[autocomplete=username]").fill("layout" +
                        UUID.randomUUID().toString().replace("-", "").substring(0, 10));
                page.locator("input[autocomplete=current-password]").fill("LayoutFixture-2026");
                page.locator(".submit-btn").click();
                page.locator(".bookshelf-page").waitFor();
                String bookUrl = (String) page.evaluate(resource("vue3-reading-setup.js"), fixtureUrl);
                page.evaluate(resource("vue3-shelf-layout-setup.js"), bookUrl);
                page.reload();
                page.waitForCondition(() -> page.locator(".book-card").count() == 15);

                for (int width : new int[]{1135, 1024, 768}) {
                    page.setViewportSize(width, 865);
                    for (int density = 0; density < 3; density++) {
                        page.locator(".view-bar .sort-capsule").nth(density).click();
                        page.evaluate("window.scrollTo(0, 0)");
                        String scenario = "width=" + width + " density=" + density;
                        assertNoHorizontalOverflow(page, "hidden " + scenario);

                        @SuppressWarnings("unchecked")
                        List<String> edgeNames = (List<String>) page.locator(".book-card")
                                .evaluateAll("cards => { const visible = cards.filter(card => {"
                                        + "const r = card.getBoundingClientRect();"
                                        + "return r.top < innerHeight && r.bottom > 0; });"
                                        + "if (!visible.length) throw new Error('No visible fixture cards');"
                                        + "visible.sort((a,b) => a.getBoundingClientRect().left"
                                        + " - b.getBoundingClientRect().left);"
                                        + "return [visible[0], visible[visible.length-1]]"
                                        + ".map(card => card.querySelector('.book-name').textContent.trim()); }");
                        for (String name : edgeNames) {
                            Locator card = page.locator(".book-card")
                                    .filter(new Locator.FilterOptions().setHasText(name));
                            card.hover();
                            Locator preview = card.locator(".hover-preview");
                            page.waitForCondition(() -> "visible".equals(preview
                                    .evaluate("el => getComputedStyle(el).visibility")));
                            @SuppressWarnings("unchecked")
                            Map<String, Object> bounds = (Map<String, Object>) preview.evaluate("el => {"
                                    + "const p=el.getBoundingClientRect();"
                                    + "const c=el.closest('.book-card').getBoundingClientRect();"
                                    + "return {left:p.left,right:p.right,cardLeft:c.left,cardRight:c.right};}");
                            assertTrue(scenario + " preview crossed its card: " + bounds,
                                    ((Number) bounds.get("left")).doubleValue()
                                            >= ((Number) bounds.get("cardLeft")).doubleValue() - 1
                                            && ((Number) bounds.get("right")).doubleValue()
                                            <= ((Number) bounds.get("cardRight")).doubleValue() + 1);
                            assertTrue("Preview must retain the fixture introduction",
                                    preview.innerText().contains("长简介用于布局回归"));
                            assertNoHorizontalOverflow(page, "hovered " + scenario);
                        }
                        if (width == 1135 && density == 0) {
                            Path screenshot = Path.of(System.getenv().getOrDefault("RUNNER_TEMP", "build"))
                                    .resolve("vue3-shelf-layout-1135.png");
                            Files.createDirectories(screenshot.getParent());
                            page.screenshot(new Page.ScreenshotOptions().setPath(screenshot));
                        }
                    }
                }
            } finally {
                browser.close();
            }
        }
    }
}
