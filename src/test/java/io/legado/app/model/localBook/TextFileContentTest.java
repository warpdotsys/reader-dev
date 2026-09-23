package io.legado.app.model.localBook;

import io.legado.app.data.entities.Book;
import io.legado.app.data.entities.BookChapter;
import java.io.File;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

import static org.junit.Assert.assertEquals;

public class TextFileContentTest {

    @Rule public TemporaryFolder temporaryFolder = new TemporaryFolder();

    @Test
    public void chapterContentRetainsTitleAndOriginalWhitespace() throws Exception {
        String content = "第一章 开始\n这是第一章的正文。\n";
        File file = temporaryFolder.newFile("readable.txt");
        Files.write(file.toPath(), content.getBytes(StandardCharsets.UTF_8));

        Book book = new Book();
        book.setOriginName(file.getAbsolutePath());
        book.setCharset("UTF-8");
        BookChapter chapter = new BookChapter();
        chapter.setTitle("第一章 开始");
        chapter.setStart(0L);
        chapter.setEnd((long) content.getBytes(StandardCharsets.UTF_8).length);

        assertEquals(content, TextFile.Companion.getContent(book, chapter));
    }
}
