package com.htmake.reader.api.controller;

import java.io.File;
import java.io.FileOutputStream;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.file.Files;
import java.util.Arrays;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

import static org.junit.Assert.assertArrayEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

public class DirectEpubExtractorTest {
    @Rule public TemporaryFolder temporaryFolder = new TemporaryFolder();

    private File archive(String entryName, byte[] body) throws Exception {
        File zip = temporaryFolder.newFile("source.epub");
        try (ZipOutputStream output = new ZipOutputStream(new FileOutputStream(zip))) {
            output.putNextEntry(new ZipEntry(entryName));
            output.write(body);
            output.closeEntry();
        }
        return zip;
    }

    @Test
    public void validArchiveExtractsToOwnedCache() throws Exception {
        byte[] body = "chapter content".getBytes("UTF-8");
        File source = archive("OEBPS/chapter.xhtml", body);
        File cache = new File(temporaryFolder.getRoot(), "cache");

        assertTrue(DirectEpubExtractor.INSTANCE.extract(source, cache, false, "book-url"));
        assertArrayEquals(body, Files.readAllBytes(new File(cache, "OEBPS/chapter.xhtml").toPath()));
        assertTrue(new File(cache, ".reader-epub-source").isFile());
        assertTrue(DirectEpubExtractor.INSTANCE.extract(source, cache, false, "book-url"));
    }

    @Test
    public void forgedDeclaredSizeCannotWriteBeyondLimitOrReplaceExistingCache() throws Exception {
        byte[] body = new byte[128 * 1024];
        Arrays.fill(body, (byte) 'x');
        File source = archive("OEBPS/chapter.xhtml", body);
        byte[] zip = Files.readAllBytes(source.toPath());
        int centralDirectory = -1;
        for (int i = 0; i < zip.length - 46; i++) {
            if (zip[i] == 'P' && zip[i + 1] == 'K' && zip[i + 2] == 1 && zip[i + 3] == 2) {
                centralDirectory = i;
                break;
            }
        }
        assertTrue(centralDirectory >= 0);
        ByteBuffer.wrap(zip, centralDirectory + 24, 4).order(ByteOrder.LITTLE_ENDIAN).putInt(1);
        Files.write(source.toPath(), zip);

        File cache = temporaryFolder.newFolder("cache");
        File marker = new File(cache, ".reader-epub-source");
        File existing = new File(cache, "keep.txt");
        Files.write(marker.toPath(), "book-url\nold".getBytes("UTF-8"));
        Files.write(existing.toPath(), "keep".getBytes("UTF-8"));

        assertFalse(DirectEpubExtractor.INSTANCE.extract(source, cache, true, "book-url"));
        assertArrayEquals("keep".getBytes("UTF-8"), Files.readAllBytes(existing.toPath()));
        assertFalse(new File(cache, "OEBPS/chapter.xhtml").exists());
    }

    @Test
    public void traversalEntryDoesNotEscapeCache() throws Exception {
        File source = archive("../outside.txt", new byte[] {1});
        File cache = new File(temporaryFolder.getRoot(), "cache");
        assertFalse(DirectEpubExtractor.INSTANCE.extract(source, cache, false, "book-url"));
        assertFalse(new File(temporaryFolder.getRoot(), "outside.txt").exists());
        assertFalse(cache.exists());
    }
}
