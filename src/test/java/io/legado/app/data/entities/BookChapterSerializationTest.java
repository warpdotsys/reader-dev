package io.legado.app.data.entities;

import com.google.gson.Gson;
import org.junit.Test;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

public class BookChapterSerializationTest {

    @Test
    public void userNamespaceIsNotExposedInChapterJson() {
        BookChapter chapter = new BookChapter();
        chapter.setTitle("第一章");
        chapter.setUserNameSpace("internal-user");

        String json = new Gson().toJson(chapter);

        assertTrue(json.contains("第一章"));
        assertFalse(json.contains("_userNameSpace"));
        assertFalse(json.contains("internal-user"));
    }
}
