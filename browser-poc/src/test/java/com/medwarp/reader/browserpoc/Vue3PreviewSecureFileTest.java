package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.Browser;
import com.microsoft.playwright.BrowserType;
import com.microsoft.playwright.Page;
import com.microsoft.playwright.Playwright;
import org.junit.Assume;
import org.junit.Test;

import java.net.URI;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import java.util.UUID;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

/** A real secure-mode Reader must accept the file dialog's retry and refresh the page. */
public class Vue3PreviewSecureFileTest {
    @Test
    public void secureKeyRetryCreatesLocalStoreDirectory() {
        String previewUrl = System.getenv("READER_VUE3_SECURE_URL");
        String managerKey = System.getenv("READER_TEST_MANAGER_KEY");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue(previewUrl != null && managerKey != null && !managerKey.isEmpty()
                && !executable.isEmpty() && Files.isRegularFile(Path.of(executable))
                && "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        URI uri = URI.create(previewUrl);
        assertEquals("http", uri.getScheme());
        assertTrue("Only an isolated loopback preview is allowed",
                "127.0.0.1".equals(uri.getHost()) || "localhost".equals(uri.getHost()));

        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")))) {
            Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                    .setExecutablePath(Path.of(executable)).setHeadless(true));
            try {
                Page page = browser.newPage();
                page.setDefaultTimeout(30000);
                page.navigate(previewUrl + "/login");
                page.locator(".mode-switch button").nth(1).click();
                page.locator("input[autocomplete=username]").fill("securefile" +
                        UUID.randomUUID().toString().replace("-", "").substring(0, 8));
                page.locator("input[autocomplete=current-password]").fill("SecureFileProbe-2026");
                page.locator(".submit-btn").click();
                page.locator(".bookshelf-page").waitFor();
                page.navigate(previewUrl + "/files");
                page.locator(".file-page").waitFor();
                String folder = "secure-folder-" + UUID.randomUUID().toString().substring(0, 8);
                try {
                    page.locator(".toolbar button:has-text('新建文件夹')").click();
                    page.locator(".dlg-overlay .dlg-input").fill(folder);
                    page.locator(".dlg-overlay .btn-primary").click();
                    page.locator(".dlg-overlay:has-text('管理密码')").waitFor();
                    page.locator(".dlg-overlay:has-text('管理密码') input[type=password]").fill("wrong-key");
                    page.locator(".dlg-overlay:has-text('管理密码') .btn-primary").click();
                    page.locator("[role=alert]:has-text('管理密码不正确')").waitFor();
                    page.locator(".dlg-overlay:has-text('管理密码') input[type=password]").fill(managerKey);
                    page.locator(".dlg-overlay:has-text('管理密码') .btn-primary").click();
                    page.locator(".row-name:text-is('" + folder + "')").waitFor();
                    assertEquals(0, page.locator(".dlg-overlay:has-text('管理密码')").count());
                } finally {
                    page.evaluate("async ({folder, key}) => {" +
                            "const token = localStorage.getItem('reader_access_token');" +
                            "const query = new URLSearchParams({accessToken:token,secureKey:key});" +
                            "const response = await fetch('/reader3/file/delete?' + query," +
                            "{method:'POST',headers:{'Content-Type':'application/json'}," +
                            "body:JSON.stringify({path:'/' + folder,home:'__LOCAL_STORE__'})});" +
                            "return (await response.json()).isSuccess; }",
                            Map.of("folder", folder, "key", managerKey));
                }
            } finally {
                browser.close();
            }
        }
    }
}
