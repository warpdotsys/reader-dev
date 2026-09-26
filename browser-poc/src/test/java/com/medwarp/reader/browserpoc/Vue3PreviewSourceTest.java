package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.Browser;
import com.microsoft.playwright.BrowserType;
import com.microsoft.playwright.Page;
import com.microsoft.playwright.Playwright;
import com.microsoft.playwright.Response;
import org.junit.Assume;
import org.junit.Test;

import java.net.URI;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import java.util.UUID;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

/** A remote preview must not write sources until the user confirms import. */
public class Vue3PreviewSourceTest {
    @Test
    public void remotePreviewIsReadOnlyThenImportAndBatchDelete() {
        String previewUrl = System.getenv("READER_VUE3_PREVIEW_URL");
        String fixtureUrl = System.getenv("READER_BOOK_FIXTURE_URL");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue(previewUrl != null && fixtureUrl != null
                && !executable.isEmpty() && Files.isRegularFile(Path.of(executable))
                && "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        for (String value : new String[]{previewUrl, fixtureUrl}) {
            URI uri = URI.create(value);
            assertEquals("http", uri.getScheme());
            assertTrue("Only loopback fixtures are allowed", "127.0.0.1".equals(uri.getHost())
                    || "localhost".equals(uri.getHost()));
        }

        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")))) {
            Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                    .setExecutablePath(Path.of(executable)).setHeadless(true));
            try {
                Page page = browser.newPage();
                page.setDefaultTimeout(30000);
                page.navigate(previewUrl + "/login");
                page.locator(".mode-switch button").nth(1).click();
                page.locator("input[autocomplete=username]").fill("vue" +
                        UUID.randomUUID().toString().replace("-", "").substring(0, 10));
                page.locator("input[autocomplete=current-password]").fill("SourceProbe-2026");
                Response registration = page.waitForResponse(
                        response -> URI.create(response.url()).getPath().endsWith("/reader3/login"),
                        () -> page.locator(".submit-btn").click());
                assertEquals("Registration HTTP status", 200, registration.status());
                String registrationBody = registration.text();
                Matcher registrationError = Pattern.compile("\\\"errorMsg\\\"\\s*:\\s*\\\"([^\\\"]*)\\\"")
                        .matcher(registrationBody);
                String errorMessage = registrationError.find() ? registrationError.group(1) : "missing errorMsg";
                assertTrue("Registration rejected: " + errorMessage,
                        Pattern.compile("\\\"isSuccess\\\"\\s*:\\s*true").matcher(registrationBody).find());
                try {
                    page.locator(".bookshelf-page").waitFor();
                } catch (RuntimeException failure) {
                    String state = String.valueOf(page.evaluate("() => JSON.stringify({" +
                            "path: location.pathname," +
                            "loggedInUser: localStorage.getItem('reader_username')," +
                            "loginVisible: !!document.querySelector('.login-page')," +
                            "message: document.querySelector('.el-message__content')?.textContent" +
                            "})"));
                    String runnerTemp = System.getenv("RUNNER_TEMP");
                    if (runnerTemp != null && !runnerTemp.isEmpty()) {
                        try {
                            page.screenshot(new Page.ScreenshotOptions()
                                    .setPath(Path.of(runnerTemp, "vue3-source-login-timeout.png"))
                                    .setFullPage(true));
                        } catch (RuntimeException screenshotFailure) {
                            failure.addSuppressed(screenshotFailure);
                        }
                    }
                    throw new AssertionError("Registration succeeded but shelf did not open: " + state, failure);
                }
                page.navigate(previewUrl + "/sources");
                page.locator(".sources-page").waitFor();
                assertEquals(0, sourceCount(page));

                page.getByText("远程导入", new Page.GetByTextOptions().setExact(true)).click();
                page.locator("input[placeholder='https://…/bookSource.json']")
                        .fill(fixtureUrl + "/source.json");
                Response preview = page.waitForResponse(
                        response -> URI.create(response.url()).getPath().endsWith("/reader3/previewRemoteBookSources"),
                        () -> page.locator("[aria-label='远程导入书源'] button[type=submit]").click());
                assertTrue(preview.text().contains("\"sources\""));
                page.locator(".preview-dlg").waitFor();
                assertEquals("Preview must not persist sources", 0, sourceCount(page));

                page.locator(".preview-dlg .dlg-actions .accent-btn").click();
                page.locator(".source-row").first().waitFor();
                assertEquals(1, sourceCount(page));

                page.locator(".head-actions button[title^='多选模式']").click();
                page.locator(".source-row .select-box").first().click();
                page.locator(".batch-bar .danger").click();
                page.locator(".el-message-box__btns .el-button--primary").click();
                page.waitForFunction("!document.querySelector('.source-row')");
                assertEquals(0, sourceCount(page));

                page.locator(".head-actions .accent-outline-btn").click();
                page.locator("[aria-label='新增书源'] input").nth(0).fill(fixtureUrl);
                page.locator("[aria-label='新增书源'] input").nth(1).fill("Vue3 CRUD source");
                page.locator("[aria-label='新增书源'] input").nth(2).fill("browser-check");
                Response saved = page.waitForResponse(
                        response -> URI.create(response.url()).getPath().endsWith("/reader3/saveBookSource"),
                        () -> page.locator("[aria-label='新增书源'] button[type=submit]").click());
                assertEquals(200, saved.status());
                page.waitForFunction("document.querySelector('.source-row .source-name')?.textContent.includes('Vue3 CRUD source')");
                assertEquals(1, sourceCount(page));

                try {
                    page.locator(".source-row button[title^='编辑书源']").click();
                } catch (RuntimeException failure) {
                    String state = String.valueOf(page.evaluate(
                            "() => JSON.stringify(Array.from(document.querySelectorAll('.source-row')).map(row => ({" +
                                    "name: row.querySelector('.source-name')?.textContent," +
                                    "buttons: Array.from(row.querySelectorAll('button')).map(button => {" +
                                    "const rect = button.getBoundingClientRect();" +
                                    "return {text: button.innerText, title: button.title," +
                                    "display: getComputedStyle(button).display," +
                                    "visibility: getComputedStyle(button).visibility," +
                                    "rect: {x: rect.x, y: rect.y, width: rect.width, height: rect.height}};" +
                                    "})})))"));
                    String runnerTemp = System.getenv("RUNNER_TEMP");
                    if (runnerTemp != null && !runnerTemp.isEmpty()) {
                        try {
                            page.screenshot(new Page.ScreenshotOptions()
                                    .setPath(Path.of(runnerTemp, "vue3-source-edit-timeout.png"))
                                    .setFullPage(true));
                        } catch (RuntimeException screenshotFailure) {
                            failure.addSuppressed(screenshotFailure);
                        }
                    }
                    throw new AssertionError("Could not click source edit button; source rows: " + state, failure);
                }
                page.locator("[aria-label='编辑书源'] .field-input").nth(1).fill("Vue3 CRUD renamed");
                page.locator("[aria-label='编辑书源'] .field-input").nth(2).fill("browser-check edited");
                Response edited = page.waitForResponse(
                        response -> URI.create(response.url()).getPath().endsWith("/reader3/saveBookSource"),
                        () -> page.locator("[aria-label='编辑书源'] button[type=submit]").click());
                assertEquals(200, edited.status());
                page.waitForFunction("document.querySelector('.source-row .source-name')?.textContent.includes('Vue3 CRUD renamed')");
                Map<?, ?> source = onlySource(page);
                assertEquals(fixtureUrl, source.get("bookSourceUrl"));
                assertEquals("Vue3 CRUD renamed", source.get("bookSourceName"));
                assertEquals("browser-check edited", source.get("bookSourceGroup"));
            } finally {
                browser.close();
            }
        }
    }

    private static int sourceCount(Page page) {
        return ((Number) page.evaluate("async () => {" +
                "const token = localStorage.getItem('reader_access_token');" +
                "const response = await fetch('/reader3/getBookSources?accessToken=' + encodeURIComponent(token));" +
                "const result = await response.json();" +
                "if (!result.isSuccess) throw new Error(result.errorMsg);" +
                "return result.data.length; }")) .intValue();
    }

    private static Map<?, ?> onlySource(Page page) {
        return (Map<?, ?>) page.evaluate("async () => {" +
                "const token = localStorage.getItem('reader_access_token');" +
                "const response = await fetch('/reader3/getBookSources?accessToken=' + encodeURIComponent(token));" +
                "const result = await response.json();" +
                "if (!result.isSuccess) throw new Error(result.errorMsg);" +
                "if (result.data.length !== 1) throw new Error('Expected exactly one source');" +
                "return result.data[0]; }");
    }
}
