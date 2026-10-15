package com.htmake.reader.api.controller

import io.legado.app.utils.ACache
import io.legado.app.data.entities.Book
import com.htmake.reader.utils.toMap
import com.htmake.reader.utils.toDataClass
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder

class BookInfoCacheTest {
    @get:Rule val temp = TemporaryFolder()
    private val key = "https://example.invalid/generated-book"

    @Test
    fun validMetadataKeepsExistingFieldsAndDefaults() {
        val cache = ACache.get(temp.newFolder("generated-valid"))
        cache.put(key, """{"bookUrl":"$key","origin":"https://example.invalid/source","name":"生成书名","author":"生成作者","durChapterIndex":4,"canUpdate":false,"wordCount":"123"}""")
        val book = requireNotNull(readCachedBookInfo(cache, key))
        assertEquals(key, book.bookUrl)
        assertEquals("https://example.invalid/source", book.origin)
        assertEquals("生成书名", book.name)
        assertEquals("生成作者", book.author)
        assertEquals(4, book.durChapterIndex)
        assertEquals(false, book.canUpdate)
        assertEquals("123", book.wordCount)
        assertEquals(0, book.type)
        assertEquals(0L, book.group)
    }

    @Test
    fun malformedMetadataIsAMissWithoutDeletingOrReturningAFakeBook() {
        val cache = ACache.get(temp.newFolder("generated-malformed"))
        for (invalid in listOf("{\"bookUrl\":", "{} trailing", "[]", "{}", "null")) {
            cache.put(key, invalid)
            assertNull(readCachedBookInfo(cache, key))
            assertEquals(invalid, cache.getAsString(key))
        }
        cache.put(key, """{"bookUrl":"$key","name":"重新获取的生成元数据"}""")
        assertEquals("重新获取的生成元数据", readCachedBookInfo(cache, key)?.name)
    }

    @Test
    fun legacyIntegerAdapterDefaultsAreNotSilentlyMadeStrict() {
        val cache = ACache.get(temp.newFolder("generated-legacy-integer"))
        val json = """{"bookUrl":"$key","name":"生成元数据","durChapterIndex":"invalid"}"""
        val original = json.toMap().toDataClass<Book>()
        cache.put(key, json)
        assertEquals(0, original.durChapterIndex)
        assertEquals(original.durChapterIndex, readCachedBookInfo(cache, key)?.durChapterIndex)
        assertEquals(json, cache.getAsString(key))
    }

    @Test
    fun wrongKeyMetadataIsNotReturnedOrDeleted() {
        val cache = ACache.get(temp.newFolder("generated-wrong-key"))
        val json = """{"bookUrl":"https://example.invalid/other-book","name":"其他生成元数据"}"""
        cache.put(key, json)
        assertNull(readCachedBookInfo(cache, key))
        assertEquals(json, cache.getAsString(key))
        assertNull(readCachedBookInfo(cache, ""))
    }

    @Test
    fun missingMetadataRemainsMissingAndDoesNotCreateAnEmptyFile() {
        val directory = temp.newFolder("generated-missing")
        val cache = ACache.get(directory)
        assertNull(readCachedBookInfo(cache, key))
        assertEquals(0, directory.listFiles()!!.size)
    }
}
