package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.Page;
import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;

/** Test-only read-only gate; never change production timing or treat text length as chapter identity. */
final class RawEpubReadiness {
    private RawEpubReadiness() { }

    static void waitFor(Page page) {
        page.locator("iframe.epub-frame").waitFor();
        try (InputStream input = RawEpubReadiness.class.getResourceAsStream("/epub-frame-readiness.js")) {
            if (input == null) throw new AssertionError("Missing raw EPUB readiness gate");
            byte[] bytes = input.readNBytes(8193);
            if (bytes.length == 0 || bytes.length > 8192) throw new AssertionError("Invalid readiness gate size");
            // Inherit the existing Playwright timeout; do not extend the Worker or UI deadline.
            page.waitForFunction(new String(bytes, StandardCharsets.UTF_8));
        } catch (IOException failure) {
            throw new AssertionError("Cannot load raw EPUB readiness gate", failure);
        }
    }
}
