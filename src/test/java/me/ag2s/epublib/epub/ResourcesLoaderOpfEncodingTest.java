package me.ag2s.epublib.epub;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.File;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.zip.ZipEntry;
import java.util.zip.ZipFile;
import java.util.zip.ZipInputStream;
import java.util.zip.ZipOutputStream;
import me.ag2s.epublib.domain.Resources;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

import static org.junit.Assert.assertArrayEquals;

public class ResourcesLoaderOpfEncodingTest {
    @Rule public TemporaryFolder temporaryFolder = new TemporaryFolder();

    @Test
    public void zipFileAndStreamPreserveDeclaredLatin1MetadataDuringNamespaceRepair() throws Exception {
        byte[] input = ("<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?>"
                + "<package smlns=\"http://www.idpf.org/2007/opf\">"
                + "<title>Café déjà vu</title></package>").getBytes(StandardCharsets.ISO_8859_1);
        byte[] expected = new String(input, StandardCharsets.ISO_8859_1)
                .replace("smlns=\"", "xmlns=\"")
                .getBytes(StandardCharsets.ISO_8859_1);
        ByteArrayOutputStream bytes = new ByteArrayOutputStream();
        try (ZipOutputStream output = new ZipOutputStream(bytes)) {
            output.putNextEntry(new ZipEntry("OEBPS/content.opf"));
            output.write(input);
            output.closeEntry();
        }
        File source = temporaryFolder.newFile("latin1.epub");
        Files.write(source.toPath(), bytes.toByteArray());

        try (ZipFile file = new ZipFile(source)) {
            Resources resources = ResourcesLoader.loadResources(file, "utf-8");
            assertArrayEquals(expected, resources.getByHref("OEBPS/content.opf").getData());
        }
        try (ZipInputStream stream = new ZipInputStream(new ByteArrayInputStream(bytes.toByteArray()))) {
            Resources resources = ResourcesLoader.loadResources(stream, "utf-8");
            assertArrayEquals(expected, resources.getByHref("OEBPS/content.opf").getData());
        }
    }
}
