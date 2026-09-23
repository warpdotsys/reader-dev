package io.legado.app.data.entities;

import com.google.gson.Gson;
import org.junit.Test;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

public class SearchBookSerializationTest {

    @Test
    public void userNamespaceIsNotExposedInSearchResultJson() {
        SearchBook book = new SearchBook();
        book.setName("差分测试书");
        book.setUserNameSpace("internal-user");

        String json = new Gson().toJson(book);

        assertTrue(json.contains("差分测试书"));
        assertFalse(json.contains("_userNameSpace"));
        assertFalse(json.contains("internal-user"));
    }
}
