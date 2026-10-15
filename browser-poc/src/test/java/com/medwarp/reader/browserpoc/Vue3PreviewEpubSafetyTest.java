package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.*;
import org.junit.Assume;
import org.junit.Test;
import java.net.URI;
import java.net.URLEncoder;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.io.ByteArrayOutputStream;
import java.util.*;
import java.util.zip.*;
import static org.junit.Assert.*;

/** Tiny generated books only; never target production or import a private book. */
public class Vue3PreviewEpubSafetyTest {
    @Test public void malformedSameNameImportsFailClearlyPreservePreviewAndAllowRetry() throws Exception {
        String base = System.getenv("READER_VUE3_PREVIEW_URL");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue(base != null && !executable.isEmpty() && Files.isRegularFile(Path.of(executable))
                && "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        URI address = URI.create(base);
        assertEquals("http", address.getScheme());
        assertTrue("Only isolated loopback data", "127.0.0.1".equals(address.getHost())
                || "localhost".equals(address.getHost()));
        Path folder = Files.createTempDirectory("generated-epub-safety-");
        Path fixture = folder.resolve("generated-safety.epub");
        byte[] valid = archive(null);
        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")))) {
            Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                    .setExecutablePath(Path.of(executable)).setHeadless(true));
            try {
                Page page = browser.newPage();
                page.setDefaultTimeout(30000);
                page.navigate(base + "/login");
                page.locator(".mode-switch button").nth(1).click();
                String username = "safe" + UUID.randomUUID().toString().replace("-", "").substring(0, 10);
                page.locator("input[autocomplete=username]").fill(username);
                page.locator("input[autocomplete=current-password]").fill("GeneratedSafety-2026");
                page.locator(".submit-btn").click();
                page.locator(".bookshelf-page").waitFor();
                // Prepare a legitimate same-name preview through the real multipart API.
                assertEquals(true, page.evaluate("async b64 => {"
                        + "const bytes=Uint8Array.from(atob(b64),c=>c.charCodeAt(0));const form=new FormData();"
                        + "form.append('file',new File([bytes],'generated-safety.epub',{type:'application/epub+zip'}));"
                        + "const token=localStorage.getItem('reader_access_token')||sessionStorage.getItem('reader_access_token');"
                        + "const response=await fetch('/reader3/importBookPreview?accessToken='+encodeURIComponent(token),"
                        + "{method:'POST',body:form});const body=await response.json();"
                        + "return response.status===200&&body.isSuccess===true&&Array.isArray(body.data)&&body.data.length===1;}",
                        Base64.getEncoder().encodeToString(valid)));
                assertPreviewUnchanged(page, username, valid);
                byte[] hugeDirectory = valid.clone();
                ByteBuffer.wrap(hugeDirectory).order(ByteOrder.LITTLE_ENDIAN)
                        .putInt(hugeDirectory.length - 22 + 12, 16 * 1024 * 1024 + 1);
                List<byte[]> invalid = Arrays.asList(archive("../escape.txt"), archive("%2e%2e/escape.txt"),
                        changedCentral(valid, 24, 1), changedCentral(valid, 16, 1),
                        changedCentral(valid, 24, 16 * 1024 * 1024 + 1), hugeDirectory,
                        Arrays.copyOf(valid, valid.length - 1));
                for (byte[] bytes : invalid) {
                    assertTrue("Fixtures must stay small, not a real ZIP bomb", bytes.length < 8192);
                    Files.write(fixture, bytes);
                    if (bytes == invalid.get(invalid.size() - 1)) page.setViewportSize(390, 844);
                    page.locator(".import-btn").click();
                    page.locator("[aria-label='导入本地书籍']").waitFor();
                    FileChooser chooser = page.waitForFileChooser(() -> page.locator(".dropzone").click());
                    Response preview = page.waitForResponse(response -> URI.create(response.url()).getPath()
                            .endsWith("/reader3/importBookPreview"), () -> chooser.setFiles(fixture));
                    assertEquals(200, preview.status());
                    assertEquals(true, page.evaluate("raw=>{const body=JSON.parse(raw);return body.isSuccess===false"
                            + "&&body.errorMsg.startsWith('EPUB 文件校验失败：')"
                            + "&&Object.keys(body).sort().join(',')==='errorMsg,isSuccess';}", preview.text()));
                    page.locator(".file-state.error").waitFor();
                    String shown = page.locator(".file-state.error").innerText();
                    assertTrue(shown.contains("EPUB 文件校验失败："));
                    assertFalse("Chinese error must not be replacement-character mojibake", shown.contains("\uFFFD"));
                    assertEquals("The whole error must fit without clipping its beginning", true,
                            page.locator(".file-state.error").evaluate("element=>element.scrollWidth<=element.clientWidth+1"
                                    + "&&getComputedStyle(element).whiteSpace==='normal'"));
                    assertEquals(0, page.locator(".preview-panel").count());
                    assertTrue(page.locator("[aria-label='导入本地书籍'] .dlg-close").isEnabled());
                    assertPreviewUnchanged(page, username, valid);
                    if (bytes == invalid.get(0) || bytes == invalid.get(invalid.size() - 1)) {
                        // Fast hosted uploads can finish during the dialog's real CSS transition.
                        // Wait for the user-visible settled state, not an arbitrary sleep or disabled animation.
                        page.waitForFunction("()=>{const overlay=document.querySelector('.dlg-overlay');"
                                + "const dialog=document.querySelector('[aria-label=\"导入本地书籍\"]');"
                                + "return overlay&&dialog&&[overlay,dialog].every(element=>"
                                + "getComputedStyle(element).opacity==='1'&&element.getAnimations()"
                                + ".every(animation=>animation.playState!=='running'&&animation.playState!=='pending'));}");
                        page.screenshot(new Page.ScreenshotOptions().setPath(Path.of(System.getenv()
                                .getOrDefault("RUNNER_TEMP", System.getProperty("java.io.tmpdir")),
                                bytes == invalid.get(0) ? "vue3-epub-safety-synthetic.png"
                                        : "vue3-epub-safety-mobile-synthetic.png")));
                    }
                    page.locator("[aria-label='导入本地书籍'] .dlg-close").click();
                }
                // A real valid retry must still import, save, and display generated Chinese text.
                page.setViewportSize(1280, 720);
                Files.write(fixture, valid);
                page.locator(".import-btn").click();
                FileChooser chooser = page.waitForFileChooser(() -> page.locator(".dropzone").click());
                Response retried = page.waitForResponse(response -> URI.create(response.url()).getPath()
                        .endsWith("/reader3/importBookPreview"), () -> chooser.setFiles(fixture));
                assertEquals(200, retried.status());
                assertEquals(true, page.evaluate("raw=>JSON.parse(raw).isSuccess===true", retried.text()));
                page.locator(".preview-panel").waitFor();
                Response saved = page.waitForResponse(response -> URI.create(response.url()).getPath()
                        .endsWith("/reader3/saveBook"),
                        () -> page.locator("[aria-label='导入本地书籍'] .accent-btn").click());
                assertEquals(200, saved.status());
                assertEquals(true, page.evaluate("raw=>JSON.parse(raw).isSuccess===true", saved.text()));
                String bookUrl = (String) page.evaluate("raw=>JSON.parse(raw).data.bookUrl", saved.text());
                page.locator(".bookshelf-page").waitFor();
                page.navigate(base + "/reader/" + URLEncoder.encode(bookUrl, StandardCharsets.UTF_8).replace("+", "%20"));
                page.locator(".reader-content").waitFor();
                page.waitForFunction("()=>document.querySelector('.reader-content')?.textContent.includes('合成安全测试正文')");
                assertFalse(page.locator(".reader-content").innerText().contains("\uFFFD"));
                System.out.println("Seven tiny malformed imports: HTTP 200, failed ReturnData/omitted null data, visible Chinese errors, unchanged valid preview; valid UI retry/read PASS");
            } finally { browser.close(); }
        } finally {
            Files.deleteIfExists(fixture);
            Files.deleteIfExists(folder);
        }
    }
    private static void assertPreviewUnchanged(Page page, String username, byte[] valid) {
        assertEquals(true, page.evaluate("async input=>{const response=await fetch('/assets/'+input.username"
                + "+'/book/generated-safety.epub/index.epub',{cache:'no-store'});if(response.status!==200)return false;"
                + "const got=new Uint8Array(await response.arrayBuffer());const expected=Uint8Array.from(atob(input.b64),c=>c.charCodeAt(0));"
                + "return got.length===expected.length&&got.every((value,index)=>value===expected[index]);}",
                Map.of("username", username, "b64", Base64.getEncoder().encodeToString(valid))));
    }
    private static byte[] changedCentral(byte[] valid, int offset, int value) {
        byte[] changed = valid.clone();
        ByteBuffer view = ByteBuffer.wrap(changed).order(ByteOrder.LITTLE_ENDIAN);
        for (int i = 0; i + 46 <= changed.length; i++) {
            if (view.getInt(i) == 0x02014b50) {
                int nameLength = Short.toUnsignedInt(view.getShort(i + 28));
                String name = new String(changed, i + 46, nameLength, StandardCharsets.UTF_8);
                if (name.equals("OEBPS/one.xhtml")) { view.putInt(i + offset, value); return changed; }
            }
        }
        throw new AssertionError("Generated fixture central directory absent");
    }
    private static byte[] archive(String unsafe) throws Exception {
        ByteArrayOutputStream bytes = new ByteArrayOutputStream();
        try (ZipOutputStream zip = new ZipOutputStream(bytes)) {
            entry(zip, "mimetype", "application/epub+zip");
            entry(zip, "META-INF/container.xml", "<container><rootfiles><rootfile full-path='OEBPS/book.opf'/></rootfiles></container>");
            entry(zip, "OEBPS/book.opf", "<package xmlns='http://www.idpf.org/2007/opf' version='2.0' unique-identifier='id'>"
                    + "<metadata xmlns:dc='http://purl.org/dc/elements/1.1/'><dc:identifier id='id'>generated-safety</dc:identifier>"
                    + "<dc:title>合成安全测试</dc:title><dc:creator>测试</dc:creator><dc:language>zh-CN</dc:language></metadata>"
                    + "<manifest><item id='ncx' href='toc.ncx' media-type='application/x-dtbncx+xml'/>"
                    + "<item id='one' href='one.xhtml' media-type='application/xhtml+xml'/></manifest>"
                    + "<spine toc='ncx'><itemref idref='one'/></spine></package>");
            entry(zip, "OEBPS/toc.ncx", "<ncx xmlns='http://www.daisy.org/z3986/2005/ncx/' version='2005-1'>"
                    + "<head/><docTitle><text>合成安全测试</text></docTitle><navMap><navPoint id='one' playOrder='1'>"
                    + "<navLabel><text>第一章</text></navLabel><content src='one.xhtml'/></navPoint></navMap></ncx>");
            entry(zip, "OEBPS/one.xhtml", "<html xmlns='http://www.w3.org/1999/xhtml'><head><title>生成</title></head>"
                    + "<body><h1>第一章</h1><p>合成安全测试正文</p></body></html>");
            if (unsafe != null) entry(zip, unsafe, "Generated only");
        }
        return bytes.toByteArray();
    }
    private static void entry(ZipOutputStream zip, String name, String text) throws Exception {
        ZipEntry entry = new ZipEntry(name);
        entry.setTime(1577836800000L);
        byte[] bytes = text.getBytes(StandardCharsets.UTF_8);
        if (name.equals("mimetype")) {
            CRC32 crc = new CRC32(); crc.update(bytes);
            entry.setMethod(ZipEntry.STORED); entry.setSize(bytes.length); entry.setCrc(crc.getValue());
        }
        zip.putNextEntry(entry); zip.write(bytes); zip.closeEntry();
    }
}
