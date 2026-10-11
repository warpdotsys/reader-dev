package com.htmake.reader;

import org.junit.Test;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.ByteBuffer;
import java.nio.charset.CharacterCodingException;
import java.nio.charset.CodingErrorAction;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.Assert.assertArrayEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertTrue;

public class WebResourceEncodingTest {

    @Test
    public void mainVueBundleRemainsValidUtf8AfterLicenseUrlReplacement() throws Exception {
        byte[] bytes = readResource("/web/js/app.54619a3e.js");
        String script = decodeUtf8Strictly(bytes);

        assertTrue(script.contains("https://license.medwarp.cn"));
        assertFalse(script.contains("https://r.htmake.com"));
        assertTrue(script.contains("登录"));
        assertTrue(script.contains("书架"));
    }

    @Test
    public void serviceWorkerForcesAFullCleanPrecacheGeneration() throws Exception {
        String serviceWorker = decodeUtf8Strictly(readResource("/web/service-worker.js"));
        String manifest = decodeUtf8Strictly(readResource(
                "/web/precache-manifest.0d903434eaa73f94acefeef5d39c6628.js"));

        assertTrue(serviceWorker.contains("Encoding repair generation utf8fix1"));
        assertTrue(serviceWorker.contains("self.skipWaiting()"));
        assertTrue(serviceWorker.contains("workbox.core.clientsClaim()"));
        assertTrue(serviceWorker.contains("reader-utf8fix1"));
        assertTrue(manifest.contains("\"revision\": \"utf8fix1-"));
    }

    @Test
    public void readerBundlePreservesSupplementaryHanAcrossTheWholeLegacyTransform() throws Exception {
        String path = "web/js/reader.6e871769.js";
        byte[] source = readSource(path);
        String sourceText = decodeUtf8Strictly(source);
        String supplementaryHan = new String(Character.toChars(0x2B2AE));
        assertTrue("Source must retain U+2B2AE before resource processing", sourceText.contains(supplementaryHan));

        byte[] expected = canonicalLegacyText(sourceText, true);
        byte[] packaged = readResource("/" + path);
        assertArrayEquals("Whole JS resource must use only the established LF/license transform", expected, packaged);
        assertTrue("Packaged resource must retain U+2B2AE instead of a replacement glyph",
                decodeUtf8Strictly(packaged).contains(supplementaryHan));
    }

    @Test
    public void applicationYamlResourcesAreStrictUtf8LfExactCopies() throws Exception {
        for (String path : new String[] {"application.yml", "application-prod.yml"}) {
            String sourceText = decodeUtf8Strictly(readSource(path));
            byte[] expected = canonicalLegacyText(sourceText, false);
            byte[] packaged = readResource("/" + path);
            assertArrayEquals("Classpath " + path + " must be the complete LF-normalized source", expected, packaged);
            assertFalse("Classpath " + path + " must not retain CR bytes", containsCarriageReturn(packaged));
        }
    }

    private static byte[] readSource(String relativePath) throws IOException {
        return Files.readAllBytes(Path.of("src", "main", "resources").resolve(relativePath));
    }

    private static byte[] canonicalLegacyText(String source, boolean replaceLicenseUrl) {
        String canonical = source.replace("\r\n", "\n").replace("\r", "\n");
        if (replaceLicenseUrl) canonical = canonical.replace("https://r.htmake.com", "https://license.medwarp.cn");
        return canonical.getBytes(StandardCharsets.UTF_8);
    }

    private static boolean containsCarriageReturn(byte[] bytes) {
        for (byte value : bytes) if (value == '\r') return true;
        return false;
    }

    private static byte[] readResource(String path) throws IOException {
        InputStream input = WebResourceEncodingTest.class.getResourceAsStream(path);
        assertNotNull("Missing classpath resource " + path, input);
        try {
            ByteArrayOutputStream output = new ByteArrayOutputStream();
            byte[] buffer = new byte[8192];
            int count;
            while ((count = input.read(buffer)) != -1) {
                output.write(buffer, 0, count);
            }
            return output.toByteArray();
        } finally {
            input.close();
        }
    }

    private static String decodeUtf8Strictly(byte[] bytes) throws CharacterCodingException {
        return StandardCharsets.UTF_8.newDecoder()
                .onMalformedInput(CodingErrorAction.REPORT)
                .onUnmappableCharacter(CodingErrorAction.REPORT)
                .decode(ByteBuffer.wrap(bytes))
                .toString();
    }
}
