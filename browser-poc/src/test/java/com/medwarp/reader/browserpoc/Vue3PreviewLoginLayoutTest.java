package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.*;
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

import static org.junit.Assert.*;

/** Real browser geometry, scrolling, keyboard and login; no private books or accounts. */
public class Vue3PreviewLoginLayoutTest {
    private static void settled(Page page) {
        page.locator(".login-page").waitFor();
        page.waitForFunction("() => document.fonts.status === 'loaded'"
                + " && getComputedStyle(document.querySelector('.login-page')).opacity === '1'"
                + " && document.getAnimations().every(a => a.playState !== 'running')");
    }

    private static boolean reachable(Locator control) {
        control.scrollIntoViewIfNeeded();
        return (Boolean) control.evaluate("el => { const r=el.getBoundingClientRect();"
                + "const x=r.left+r.width/2, y=r.top+r.height/2;"
                + "const hit=document.elementFromPoint(x,y);"
                + "return r.width>0 && r.height>0 && r.left>=0 && r.right<=innerWidth"
                + " && r.top>=0 && r.bottom<=innerHeight && (hit===el || el.contains(hit)); }");
    }

    private static void capture(Page page, Path directory, String scenario) {
        // Full-page screenshots can reposition fixed elements in Chromium. Keep
        // the actual viewport at both scroll ends as well as the full document.
        page.evaluate("window.scrollTo(0, 0)");
        page.screenshot(new Page.ScreenshotOptions()
                .setPath(directory.resolve("vue3-login-layout-generated-" + scenario + "-top.png")));
        page.screenshot(new Page.ScreenshotOptions().setFullPage(true)
                .setPath(directory.resolve("vue3-login-layout-generated-" + scenario + "-full.png")));
        page.evaluate("window.scrollTo(0, document.documentElement.scrollHeight)");
        page.screenshot(new Page.ScreenshotOptions()
                .setPath(directory.resolve("vue3-login-layout-generated-" + scenario + "-bottom.png")));
    }

    @Test(timeout = 180000)
    public void loginAndRegisterRemainReachableWithoutFooterOverlapAtShortAndNarrowSizes() throws Exception {
        String preview = System.getenv("READER_VUE3_PREVIEW_URL");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue(preview != null && !executable.isEmpty() && Files.isRegularFile(Path.of(executable))
                && "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        assertEquals("http", URI.create(preview).getScheme());
        assertTrue("Only isolated loopback Reader is permitted",
                List.of("127.0.0.1", "localhost").contains(URI.create(preview).getHost()));
        Path directory = Path.of(System.getenv().getOrDefault("READER_UI_EVIDENCE_DIR",
                System.getenv().getOrDefault("RUNNER_TEMP", "build")));
        Files.createDirectories(directory);
        List<String> problems = new ArrayList<>();
        List<String> facts = new ArrayList<>();
        List<String> pageErrors = new ArrayList<>();
        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")));
             Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                     .setExecutablePath(Path.of(executable)).setHeadless(true));
             BrowserContext context = browser.newContext()) {
            Page page = context.newPage();
            page.onPageError(pageErrors::add);
            page.setDefaultTimeout(10000);
            int[][] sizes = {{1280, 900}, {1280, 720}, {1280, 480}, {360, 640},
                    {320, 568}, {390, 844}, {360, 320}};
            for (int[] size : sizes) {
                page.setViewportSize(size[0], size[1]);
                page.navigate(preview + "/login");
                for (int mode = 0; mode < 2; mode++) {
                    page.locator(".mode-switch button").nth(mode).click();
                    settled(page);
                    assertEquals(mode == 0 ? 0 : 1, page.locator(".field-input[autocomplete=off]").count());
                    String scenario = size[0] + "x" + size[1] + (mode == 0 ? "-login" : "-register");
                    page.evaluate("window.scrollTo(0, 0)");
                    @SuppressWarnings("unchecked")
                    Map<String, Object> bounds = (Map<String, Object>) page.evaluate("() => {"
                            + "const rect=s=>{const r=document.querySelector(s).getBoundingClientRect();"
                            + "return {top:r.top+scrollY,bottom:r.bottom+scrollY};};"
                            + "return {width:document.documentElement.clientWidth,"
                            + "scrollWidth:document.documentElement.scrollWidth,"
                            + "height:document.documentElement.scrollHeight,"
                            + "brand:rect('.wordmark'),foot:rect('.login-foot'),footer:rect('.login-footer')};}");
                    facts.add((String) page.evaluate("({scenario,bounds}) => JSON.stringify({scenario,...bounds})",
                            Map.of("scenario", scenario, "bounds", bounds)));
                    @SuppressWarnings("unchecked") Map<String, Number> brand = (Map<String, Number>) bounds.get("brand");
                    @SuppressWarnings("unchecked") Map<String, Number> foot = (Map<String, Number>) bounds.get("foot");
                    @SuppressWarnings("unchecked") Map<String, Number> footer = (Map<String, Number>) bounds.get("footer");
                    if (((Number) bounds.get("scrollWidth")).intValue() > ((Number) bounds.get("width")).intValue())
                        problems.add(scenario + ": horizontal overflow");
                    if (brand.get("top").doubleValue() < 0) problems.add(scenario + ": brand starts above document");
                    if (footer.get("top").doubleValue() < foot.get("bottom").doubleValue() + 12)
                        problems.add(scenario + ": footer overlaps/truncates form content " + bounds);
                    if (footer.get("bottom").doubleValue() > ((Number) bounds.get("height")).doubleValue())
                        problems.add(scenario + ": footer not reachable by document scrolling");
                    for (String selector : List.of("input[autocomplete=username]", "input[autocomplete=current-password]",
                            ".remember-box", ".submit-btn", ".link-btn", ".tg-foot")) {
                        if (!reachable(page.locator(selector))) problems.add(scenario + ": unreachable " + selector);
                    }
                    if (mode == 1 && !reachable(page.locator(".field-input[autocomplete=off]")))
                        problems.add(scenario + ": unreachable invitation field");
                    capture(page, directory, scenario);
                    // Click the real bottom action only when native hit testing confirms it is reachable.
                    // Never open the external group link; it is checked geometrically above.
                    if (reachable(page.locator(".link-btn"))) {
                        page.locator(".link-btn").click();
                        assertEquals(mode == 0 ? 1 : 0, page.locator(".field-input[autocomplete=off]").count());
                    }
                }
            }
            String geometry = "[" + String.join(",", facts) + "]";
            Files.writeString(directory.resolve("vue3-login-layout-generated-geometry.json"), geometry,
                    StandardCharsets.UTF_8);
            assertTrue("Settled real browser layout failures: " + problems, problems.isEmpty());

            // Actual generated registration on the narrow screen, then existing-user login
            // with a reduced viewport. This models limited height, not a real phone keyboard.
            page.setViewportSize(320, 568);
            page.navigate(preview + "/login");
            settled(page);
            page.locator(".link-btn").click();
            String account = "loginlayout" + UUID.randomUUID().toString().replace("-", "").substring(0, 10);
            page.locator("input[autocomplete=username]").fill(account);
            page.locator("input[autocomplete=username]").press("Tab");
            assertEquals("current-password", page.evaluate("document.activeElement.autocomplete"));
            page.keyboard().type("Generated-LoginLayout-20261005");
            page.keyboard().press("Tab");
            assertEquals("off", page.evaluate("document.activeElement.autocomplete"));
            page.keyboard().press("Tab");
            assertEquals("checkbox", page.evaluate("document.activeElement.type"));
            page.keyboard().press("Space");
            assertFalse(page.locator(".remember-box").isChecked());
            page.keyboard().press("Tab");
            assertEquals("submit", page.evaluate("document.activeElement.type"));
            Response registered = page.waitForResponse(r -> r.url().contains("/reader3/login")
                    && "POST".equals(r.request().method()), () -> page.keyboard().press("Enter"));
            assertEquals(200, registered.status());
            assertEquals(true, page.evaluate("body=>JSON.parse(body).isSuccess===true", registered.text()));
            page.locator(".bookshelf-page").waitFor();
            assertEquals(account, page.locator(".user-chip").innerText());
            page.locator(".logout-btn").click();
            settled(page);
            page.setViewportSize(360, 320);
            page.locator("input[autocomplete=username]").fill(account);
            page.locator("input[autocomplete=current-password]").fill("Generated-LoginLayout-20261005");
            assertTrue(reachable(page.locator(".submit-btn")));
            Response loggedIn = page.waitForResponse(r -> r.url().contains("/reader3/login")
                    && "POST".equals(r.request().method()), () -> page.locator(".submit-btn").click());
            assertEquals(200, loggedIn.status());
            assertEquals(true, page.evaluate("body=>JSON.parse(body).isSuccess===true", loggedIn.text()));
            page.locator(".bookshelf-page").waitFor();
            assertEquals(account, page.locator(".user-chip").innerText());
            assertTrue("Unexpected browser exceptions: " + pageErrors, pageErrors.isEmpty());
            System.out.println("LOGIN_LAYOUT_GENERATED_PASSED sizes=7 modes=2 native-hit=checked scroll=checked"
                    + " keyboard=checked generated-register=passed generated-login=passed external-group=not-opened");
        }
    }
}
