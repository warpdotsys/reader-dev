package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.Browser;
import com.microsoft.playwright.BrowserType;
import com.microsoft.playwright.Download;
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

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertTrue;

/** The Vue 3 backup settings must follow the legacy empty-data response and user-home ZIP path. */
public class Vue3PreviewBackupTest {
    @Test
    public void backsUpThroughLegacyRouteAndDownloadsTheUserScopedZip() throws Exception {
        String previewUrl = System.getenv("READER_VUE3_PREVIEW_URL");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue(previewUrl != null && !executable.isEmpty()
                && Files.isRegularFile(Path.of(executable))
                && "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        URI uri = URI.create(previewUrl);
        assertEquals("http", uri.getScheme());
        assertTrue("Only loopback preview servers are allowed", "127.0.0.1".equals(uri.getHost())
                || "localhost".equals(uri.getHost()));

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
                page.locator("input[autocomplete=current-password]").fill("BackupProbe-2026");
                page.locator(".submit-btn").click();
                page.locator(".bookshelf-page").waitFor();

                page.navigate(previewUrl + "/settings");
                var backupCard = page.getByTestId("data-backup-card");
                backupCard.waitFor();
                Response backup = page.waitForResponse(
                        response -> URI.create(response.url()).getPath().endsWith("/reader3/backupToWebdav"),
                        () -> backupCard.getByRole(com.microsoft.playwright.options.AriaRole.BUTTON,
                                new com.microsoft.playwright.Locator.GetByRoleOptions().setName("立即备份")).click());
                assertEquals(200, backup.status());
                assertTrue("Legacy success has an empty-string data field",
                        backup.text().contains("\"data\":\"\""));
                page.getByTestId("backup-path").waitFor();
                assertTrue(page.getByTestId("backup-path").innerText()
                        .matches("webdav/legado/backup\\d{4}-\\d{2}-\\d{2}\\.zip"));

                Download download = page.waitForDownload(() -> backupCard.getByRole(
                        com.microsoft.playwright.options.AriaRole.BUTTON,
                        new com.microsoft.playwright.Locator.GetByRoleOptions().setName("导出数据")).click());
                assertNotNull(download.path());
                assertTrue(download.suggestedFilename().matches("backup\\d{4}-\\d{2}-\\d{2}\\.zip"));
                byte[] zip = Files.readAllBytes(download.path());
                assertTrue("Export must be a nonempty ZIP archive", zip.length > 4
                        && zip[0] == 'P' && zip[1] == 'K');

                String backupPath = page.getByTestId("backup-path").innerText();
                String backupName = backupPath.substring(backupPath.lastIndexOf('/') + 1);
                page.navigate(previewUrl + "/files");
                page.locator(".file-page").waitFor();
                page.locator(".home-pills button:has-text('用户数据')").click();
                page.locator(".row-name:text-is('webdav')").waitFor();
                page.locator(".row").filter(new com.microsoft.playwright.Locator.FilterOptions()
                        .setHasText("webdav")).locator(".row-main").click();
                page.locator(".row-name:text-is('legado')").waitFor();
                page.locator(".row").filter(new com.microsoft.playwright.Locator.FilterOptions()
                        .setHasText("legado")).locator(".row-main").click();
                page.locator(".row-name:text-is('" + backupName + "')").waitFor();
                page.locator(".row").filter(new com.microsoft.playwright.Locator.FilterOptions()
                        .setHasText(backupName)).locator(".row-select").click();
                page.locator(".toolbar button:has-text('还原备份')").click();
                assertTrue(page.locator(".dlg-overlay").innerText()
                        .contains("会替换备份中包含的当前用户数据"));
                Response restored = page.waitForResponse(
                        response -> URI.create(response.url()).getPath().endsWith("/reader3/file/restore"),
                        () -> page.locator(".dlg-overlay .btn-primary").click());
                assertEquals(200, restored.status());
                assertTrue("Restore must report the legacy ReturnData success envelope",
                        restored.text().contains("\"isSuccess\":true"));
            } finally {
                browser.close();
            }
        }
    }
}
