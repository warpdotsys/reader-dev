package io.legado.app.model.localBook

import io.legado.app.data.entities.Book
import io.legado.app.data.entities.BookChapter
import org.apache.pdfbox.pdmodel.PDDocument
import org.apache.pdfbox.pdmodel.interactive.documentnavigation.outline.PDDocumentOutline
import org.apache.pdfbox.pdmodel.interactive.documentnavigation.outline.PDOutlineItem
import org.apache.pdfbox.pdmodel.interactive.documentnavigation.outline.PDOutlineNode
import java.io.InputStream

class PdfFile(book: Book) {
    var book: Book = book
    var info: Map<String, Any>? = null
    var cover: InputStream? = null

    companion object {
        private var cFile: PdfFile? = null

        @Synchronized
        private fun getPdfFile(book: Book): PdfFile {
            if (cFile == null || cFile!!.book.bookUrl != book.bookUrl) {
                cFile = PdfFile(book)
                return cFile!!
            }
            cFile?.book = book
            return cFile!!
        }

        @Synchronized
        fun getChapterList(book: Book): ArrayList<BookChapter> {
            return getPdfFile(book).getChapterList()
        }

        @Synchronized
        fun getContent(book: Book, chapter: BookChapter): String? {
            return getPdfFile(book).getContent(chapter)
        }

        @Synchronized
        fun upBookInfo(book: Book, onlyCover: Boolean = false) {
            if (onlyCover) {
                return getPdfFile(book).updateCover()
            }
            return getPdfFile(book).upBookInfo()
        }
    }

    private fun parseBookInfo(): Pair<Map<String, Any>?, InputStream?> {
        return Pair(info, cover)
    }

    private fun upBookInfo() {
        val result = parseBookInfo()
        if (result.first != null) {
            val bookInfo = result.first as Map<String, Any>
            val comicInfo = bookInfo["ComicInfo"] as Map<String, Any>?
            book.name = (comicInfo?.get("Title") ?: book.name) as String
            book.author = (comicInfo?.get("Writer") ?: book.author) as String
        }
        updateCover()
    }

    private fun updateCover() {
    }

    private fun getContent(@Suppress("UNUSED_PARAMETER") chapter: BookChapter): String {
        return ""
    }

    private fun getChapterList(): ArrayList<BookChapter> {
        if (book.tocUrl.isEmpty()) {
            book.tocUrl = "page"
        }
        if (book.tocUrl == "page") {
            return getChapterListByPage()
        }
        return getChapterListByOutline()
    }

    private fun getChapterListByPage(): ArrayList<BookChapter> {
        val chapterList = ArrayList<BookChapter>()
        val document = PDDocument.load(book.getLocalFile())
        val pageCount = document.numberOfPages
        var pageIndex = 0
        while (pageIndex < pageCount) {
            val name = "output-$pageIndex.png"
            val chapter = BookChapter()
            chapter.title = name
            chapter.index = pageIndex
            chapter.bookUrl = book.bookUrl
            chapter.url = name
            chapter.start = pageIndex.toLong()
            chapter.end = pageIndex.toLong()
            chapterList.add(chapter)
            pageIndex++
        }
        book.latestChapterTitle = chapterList.lastOrNull()?.title
        book.totalChapterNum = chapterList.size
        okhttp3.internal.Util.closeQuietly(document)
        return chapterList
    }

    private fun getChapterListByOutline(): ArrayList<BookChapter> {
        val chapterList = ArrayList<BookChapter>()
        val document = PDDocument.load(book.getLocalFile())
        val outline: PDDocumentOutline? = document.documentCatalog.documentOutline
        if (outline == null) {
            return chapterList
        }
        processOutline(document, chapterList, outline)
        if (chapterList.size > 0) {
            chapterList[chapterList.size - 1].end = document.numberOfPages.toLong()
        }
        okhttp3.internal.Util.closeQuietly(document)
        return chapterList
    }

    private fun processOutline(
        document: PDDocument,
        chapterList: ArrayList<BookChapter>,
        outline: PDOutlineNode
    ) {
        var current: PDOutlineItem? = outline.firstChild
        while (current != null) {
            val page = current.findDestinationPage(document)
            val pageIndex = document.documentCatalog.pages.indexOf(page)
            if (chapterList.size == 0 && pageIndex >= 1) {
                val firstChapter = BookChapter()
                firstChapter.title = "首章"
                firstChapter.index = 0
                firstChapter.bookUrl = book.bookUrl
                firstChapter.url = "chapter-0"
                firstChapter.start = 0L
                firstChapter.end = pageIndex.toLong()
                chapterList.add(firstChapter)
            }
            if (chapterList.size > 0) {
                val lastStart = chapterList[chapterList.size - 1].start
                if (lastStart != null && lastStart == pageIndex.toLong()) {
                    current = current.nextSibling
                    continue
                }
                val chapter = BookChapter()
                chapter.title = current.title
                chapter.index = chapterList.size
                chapter.bookUrl = book.bookUrl
                chapter.url = "chapter-${chapterList.size}"
                chapter.start = pageIndex.toLong()
                chapterList[chapterList.size - 1].end = pageIndex.toLong() - 1L
                chapterList.add(chapter)
            }
            if (current.hasChildren()) {
                processOutline(document, chapterList, current)
            }
            current = current.nextSibling
        }
    }
}
