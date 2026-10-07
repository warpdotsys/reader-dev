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
import java.util.ArrayList;
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
                + "document: document.documentElement.scrollWidth,"
                + "overflow: [...document.querySelectorAll('body *')].map(el => {"
                + "const r=el.getBoundingClientRect(); return {tag:el.tagName,"
                + "class:el.className,left:r.left,right:r.right};})"
                + ".filter(r => r.right > document.documentElement.clientWidth + 1"
                + " || r.left < -1).slice(0,12)})");
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

    private static void observeUncoveredPageTop(Page page, int width, int density,
                                               List<Map<String, Object>> observations,
                                               Path directory) throws Exception {
        page.mouse().move(0, 0);
        page.evaluate("window.scrollTo(0, 0)");
        page.waitForFunction("() => scrollY === 0 && document.fonts.status === 'loaded'"
                + " && getComputedStyle(document.querySelector('.bookshelf-page')).opacity === '1'");
        @SuppressWarnings("unchecked")
        Map<String, Object> geometry = (Map<String, Object>) page.evaluate("values => {"
                + "const header=document.querySelector('.topbar').getBoundingClientRect();"
                + "const heading=document.querySelector('.section-title');"
                + "const uncovered=el => { const r=el.getBoundingClientRect();"
                + "const hit=document.elementFromPoint(r.left+r.width/2,r.top+r.height/2);"
                + "return r.width>0 && r.height>0 && r.top>=0 && r.bottom<=innerHeight"
                + " && (hit===el || el.contains(hit)); };"
                + "return {width:values[0],density:values[1],scrollY,headerBottom:header.bottom,"
                + "headingTop:heading.getBoundingClientRect().top,"
                + "allNavInsideHeader:[...document.querySelectorAll('.user-area .nav-link')].every(el => {"
                + "const r=el.getBoundingClientRect();return r.top>=header.top && r.bottom<=header.bottom;}),"
                + "headingUncovered:uncovered(heading),"
                + "importUncovered:uncovered(document.querySelector('.import-btn')),"
                + "groupControlsOverlapTabs:[...document.querySelectorAll('.group-tab')].some(tab => {"
                + "const t=tab.getBoundingClientRect();return [...document.querySelectorAll('.group-manage')]"
                + ".some(control => {const c=control.getBoundingClientRect();"
                + "return Math.min(t.right,c.right)>Math.max(t.left,c.left)+1"
                + " && Math.min(t.bottom,c.bottom)>Math.max(t.top,c.top)+1;});}),"
                + "groupTabsUncovered:[...document.querySelectorAll('.group-tab')].every(uncovered)}; }",
                List.of(width, density));
        observations.add(geometry);
        Files.createDirectories(directory);
        Files.writeString(directory.resolve("vue3-shelf-layout-generated-geometry.json"),
                (String) page.evaluate("values => JSON.stringify({schemaVersion:1,"
                        + "scope:'generated page-top geometry; not production or real books',"
                        + "observations:values})", observations), StandardCharsets.UTF_8);
        if (density == 0 && (width == 1135 || width == 360)) {
            page.screenshot(new Page.ScreenshotOptions().setPath(directory.resolve(
                    "vue3-shelf-layout-generated-page-top-" + width + ".png")));
        }
        String scenario = "page top width=" + width + " density=" + density;
        assertTrue(scenario + ": " + geometry,
                ((Number) geometry.get("headingTop")).doubleValue()
                        >= ((Number) geometry.get("headerBottom")).doubleValue());
        assertEquals(scenario + " navigation outside header", true, geometry.get("allNavInsideHeader"));
        assertEquals(scenario + " heading covered", true, geometry.get("headingUncovered"));
        assertEquals(scenario + " import covered", true, geometry.get("importUncovered"));
        assertEquals(scenario + " group controls overlap filters", false, geometry.get("groupControlsOverlapTabs"));
        assertEquals(scenario + " group filter covered", true, geometry.get("groupTabsUncovered"));
    }

    @Test(timeout = 180000)
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
                        UUID.randomUUID().toString().replace("-", "").substring(0, 26));
                page.locator("input[autocomplete=current-password]").fill("LayoutFixture-2026");
                page.locator(".submit-btn").click();
                page.locator(".bookshelf-page").waitFor();
                String bookUrl = (String) page.evaluate(resource("vue3-reading-setup.js"), fixtureUrl);
                Number savedBooks = (Number) page.evaluate(resource("vue3-shelf-layout-setup.js"), bookUrl);
                assertEquals(15, savedBooks.intValue());
                page.reload();
                // A virtualized shelf need not mount all persisted books at once.
                page.getByText("共 15 本", new Page.GetByTextOptions().setExact(false)).waitFor();
                page.locator(".book-card").first().waitFor();

                Path evidence = Path.of(System.getenv().getOrDefault("RUNNER_TEMP", "build"));
                List<Map<String, Object>> observations = new ArrayList<>();
                for (int width : new int[]{1135, 1024, 768, 720, 360, 320}) {
                    page.setViewportSize(width, 865);
                    for (int density = 0; density < 3; density++) {
                        page.locator(".view-bar .sort-capsule").nth(density).click();
                        assertTrue("Requested density must be active", page.locator(".view-bar .sort-capsule")
                                .nth(density).getAttribute("class").contains("active"));
                        page.evaluate("window.scrollTo(0, 0)");
                        String scenario = "width=" + width + " density=" + density;
                        observeUncoveredPageTop(page, width, density, observations, evidence);
                        assertNoHorizontalOverflow(page, "hidden " + scenario);
                        assertEquals("Navigation must retain all real links", 12,
                                page.locator(".user-area .nav-link").count());
                        assertTrue("Search must not be crushed to make navigation fit",
                                ((Number) page.locator(".search-box").evaluate(
                                        "el => el.getBoundingClientRect().width")).doubleValue() >= 240);
                        if (width == 360 && density == 0) {
                            // The formerly covered last filter must accept a real click,
                            // then restoring All must bring the generated books back.
                            Locator lastFilter = page.locator(".group-tabs .group-tab").last();
                            lastFilter.click();
                            assertTrue(lastFilter.getAttribute("class").contains("active"));
                            page.locator(".empty-state").waitFor();
                            page.locator(".group-tabs .group-tab").first().click();
                            page.locator(".book-card").first().waitFor();
                        }

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
                                    .evaluate("el => getComputedStyle(el).visibility"))
                                    && "1".equals(preview.evaluate("el => getComputedStyle(el).opacity")));
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
                            // Capture the settled layout, not the previous card's
                            // fading preview during a legitimate hover transition.
                            page.waitForCondition(() -> (Boolean) page.evaluate("() => "
                                    + "[...document.querySelectorAll('.book-card:not(:hover) .hover-preview')]"
                                    + ".every(el => getComputedStyle(el).opacity === '0')"));
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
