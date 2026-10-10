package io.legado.app.data.entities

import com.google.gson.Gson
import org.junit.Assert.assertFalse
import org.junit.Test

class RssArticleSerializationTest {

    @Test
    fun responseDoesNotExposeInternalUserNamespace() {
        val article = RssArticle(title = "Fixture article", link = "https://example.invalid/article")
        article.setUserNameSpace("private-user")

        val json = Gson().toJson(article)
        assertFalse(json.contains("_userNameSpace"))
        assertFalse(json.contains("private-user"))
    }
}
