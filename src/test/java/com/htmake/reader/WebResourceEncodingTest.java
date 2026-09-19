package com.htmake.reader;

import org.junit.Test;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.ByteBuffer;
import java.nio.charset.CharacterCodingException;
import java.nio.charset.CodingErrorAction;
import java.nio.charset.StandardCharsets;

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
