package com.htmake.reader.utils;

import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

import java.io.File;
import java.nio.file.Files;
import java.nio.charset.StandardCharsets;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;

import static org.junit.Assert.*;

/** All files are generated inside the owned TemporaryFolder. */
public class ZipEntryBoundaryTest {
    @Rule public TemporaryFolder temporary = new TemporaryFolder();

    private File archive(String unsafeName) throws Exception {
        File file = temporary.newFile("generated.epub");
        try (ZipOutputStream zip = new ZipOutputStream(Files.newOutputStream(file.toPath()))) {
            zip.putNextEntry(new ZipEntry("valid-first.txt"));
            zip.write("generated valid bytes".getBytes(StandardCharsets.UTF_8));
            zip.closeEntry();
            zip.putNextEntry(new ZipEntry(unsafeName));
            zip.write("generated unsafe bytes".getBytes(StandardCharsets.UTF_8));
            zip.closeEntry();
        }
        return file;
    }

    @Test public void traversalRejectedBeforeWritingAnyEntry() throws Exception {
        File archive = archive("../outside.txt");
        File destination = temporary.newFolder("cache");
        assertFalse(ExtKt.unzip(archive, destination.getAbsolutePath()));
        assertFalse(new File(temporary.getRoot(), "outside.txt").exists());
        assertFalse(new File(destination, "valid-first.txt").exists());
    }

    @Test public void encodedTraversalIsNotAcceptedAsAnArchiveAlias() throws Exception {
        File archive = archive("%2e%2e/outside.txt");
        File destination = temporary.newFolder("cache");
        assertFalse(ExtKt.unzip(archive, destination.getAbsolutePath()));
        assertFalse(new File(destination, "valid-first.txt").exists());
    }
}
