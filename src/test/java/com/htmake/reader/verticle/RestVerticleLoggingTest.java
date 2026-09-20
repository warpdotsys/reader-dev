package com.htmake.reader.verticle;

import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;

public class RestVerticleLoggingTest {

    @Test
    public void queryParametersAreNeverIncludedInRequestLogs() {
        String target = "/reader3/getBookshelf?accessToken=user:secret&secureKey=secret";
        String sanitized = RestVerticleKt.sanitizeRequestTargetForLog(target);

        assertEquals("/reader3/getBookshelf", sanitized);
        assertFalse(sanitized.contains("accessToken"));
        assertFalse(sanitized.contains("secret"));
    }

    @Test
    public void controlCharactersCannotInjectExtraLogLines() {
        String sanitized = RestVerticleKt.sanitizeRequestTargetForLog(
                "/reader3/getUserInfo\r\nforged-log-entry\tvalue");

        assertEquals("/reader3/getUserInfo__forged-log-entry_value", sanitized);
        assertFalse(sanitized.contains("\r"));
        assertFalse(sanitized.contains("\n"));
        assertFalse(sanitized.contains("\t"));
    }
}
