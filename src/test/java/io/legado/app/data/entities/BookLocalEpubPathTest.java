package io.legado.app.data.entities;

import io.legado.app.model.localBook.LocalBook;
import java.io.File;
import java.nio.file.Files;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

public class BookLocalEpubPathTest {

    @Rule public TemporaryFolder temporaryFolder = new TemporaryFolder();

    private Book book(String origin) {
        Book book = new Book();
        book.setRootDir(temporaryFolder.getRoot().getAbsolutePath());
        book.setOriginName(origin);
        return book;
    }

    @Test
    public void directEpubUsesSourceFileAndNeverDeletesSibling() throws Exception {
        File reading = temporaryFolder.newFolder("reading");
        File source = new File(reading, "probe.epub");
        File sibling = new File(reading, "keep.txt");
        Files.write(source.toPath(), new byte[] {1});
        Files.write(sibling.toPath(), new byte[] {2});

        Book book = book("reading/probe.epub");
        assertEquals(source.getCanonicalFile(), book.getLocalFile().getCanonicalFile());
        LocalBook.INSTANCE.deleteBook(book);
        assertFalse(source.exists());
        assertTrue(reading.isDirectory());
        assertTrue(sibling.isFile());
    }

    @Test
    public void legacyEpubDirectoryStillUsesIndexEpub() throws Exception {
        File reading = temporaryFolder.newFolder("reading");
        File legacy = new File(reading, "legacy.epub");
        assertTrue(legacy.mkdir());
        File source = new File(legacy, "index.epub");
        File sibling = new File(reading, "keep.txt");
        Files.write(source.toPath(), new byte[] {1});
        Files.write(sibling.toPath(), new byte[] {2});

        Book book = book("reading/legacy.epub");
        assertEquals(source.getCanonicalFile(), book.getLocalFile().getCanonicalFile());
        LocalBook.INSTANCE.deleteBook(book);
        assertFalse(legacy.exists());
        assertTrue(sibling.isFile());
    }
}
