package io.legado.app.data.entities;

import java.io.File;
import java.nio.file.Files;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

import static org.junit.Assert.assertEquals;

public class BookLocalPdfPathTest {

    @Rule public TemporaryFolder temporaryFolder = new TemporaryFolder();

    @Test
    public void directlyImportedPdfUsesTheSourceFile() throws Exception {
        File source = temporaryFolder.newFile("direct.pdf");
        Book book = new Book();
        book.setRootDir(temporaryFolder.getRoot().getAbsolutePath());
        book.setOriginName("direct.pdf");

        assertEquals(source.getCanonicalFile(), book.getLocalFile().getCanonicalFile());
    }

    @Test
    public void legacyPdfDirectoryStillUsesIndexPdf() throws Exception {
        File directory = temporaryFolder.newFolder("legacy.pdf");
        File source = new File(directory, "index.pdf");
        Files.write(source.toPath(), new byte[] {1});
        Book book = new Book();
        book.setRootDir(temporaryFolder.getRoot().getAbsolutePath());
        book.setOriginName("legacy.pdf");

        assertEquals(source.getCanonicalFile(), book.getLocalFile().getCanonicalFile());
    }
}
