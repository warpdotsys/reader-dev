package io.legado.app.model.localBook

import io.legado.app.data.entities.Book
import io.legado.app.data.entities.BookChapter
import mu.KotlinLogging
import org.apache.pdfbox.pdmodel.PDDocument
import org.apache.pdfbox.pdmodel.PDDocumentInformation
import org.apache.pdfbox.pdmodel.interactive.documentnavigation.outline.PDDocumentOutline
import org.apache.pdfbox.pdmodel.interactive.documentnavigation.outline.PDOutlineItem
import org.apache.pdfbox.text.PDFTextStripper
import java.io.InputStream

private val logger = KotlinLogging.logger {}

object PdfFile {

    /**
     * Parse PDF metadata and update book info (title, author).
     */
    fun parseBookInfo(book: Book, inputStream: InputStream) {
        var doc: PDDocument? = null
        try {
            doc = PDDocument.load(inputStream)
            val info: PDDocumentInformation = doc.documentInformation
            if (!info.title.isNullOrBlank()) {
                book.name = info.title
            }
            if (!info.author.isNullOrBlank()) {
                book.author = info.author
            }
        } catch (e: Exception) {
            logger.error("parseBookInfo error: {}", e.message)
        } finally {
            doc?.close()
        }
    }

    /**
     * Get chapter list for a PDF book.
     * If the PDF has bookmarks/outline, use them as chapters.
     * Otherwise, create one chapter per page.
     */
    fun getChapterList(book: Book, inputStream: InputStream): ArrayList<BookChapter> {
        var doc: PDDocument? = null
        try {
            doc = PDDocument.load(inputStream)
            val outline: PDDocumentOutline? = doc.documentCatalog.documentOutline
            if (outline != null) {
                val chapters = getChapterListByOutline(book, doc, outline)
                if (chapters.isNotEmpty()) {
                    return chapters
                }
            }
            return getChapterListByPage(book, doc)
        } catch (e: Exception) {
            logger.error("getChapterList error: {}", e.message)
            return getChapterListByPageFallback(book, inputStream)
        } finally {
            doc?.close()
        }
    }

    /**
     * Create one chapter per page.
     */
    private fun getChapterListByPage(book: Book, doc: PDDocument): ArrayList<BookChapter> {
        val chapters = ArrayList<BookChapter>()
        val pageCount = doc.numberOfPages
        for (i in 0 until pageCount) {
            val chapter = BookChapter()
            chapter.index = i
            chapter.bookUrl = book.bookUrl
            chapter.title = "Page ${i + 1}"
            chapter.url = "pdf_page_${i}"
            chapter.start = i.toLong()
            chapter.end = (i + 1).toLong()
            chapters.add(chapter)
        }
        return chapters
    }

    /**
     * Fallback: create one chapter per page using stream reload.
     */
    private fun getChapterListByPageFallback(book: Book, inputStream: InputStream): ArrayList<BookChapter> {
        val chapters = ArrayList<BookChapter>()
        var doc: PDDocument? = null
        try {
            doc = PDDocument.load(inputStream)
            val pageCount = doc.numberOfPages
            for (i in 0 until pageCount) {
                val chapter = BookChapter()
                chapter.index = i
                chapter.bookUrl = book.bookUrl
                chapter.title = "Page ${i + 1}"
                chapter.url = "pdf_page_${i}"
                chapter.start = i.toLong()
                chapter.end = (i + 1).toLong()
                chapters.add(chapter)
            }
        } catch (e: Exception) {
            logger.error("getChapterListByPageFallback error: {}", e.message)
        } finally {
            doc?.close()
        }
        return chapters
    }

    /**
     * Use PDF bookmarks/outline as chapters.
     */
    private fun getChapterListByOutline(book: Book, doc: PDDocument, outline: PDDocumentOutline): ArrayList<BookChapter> {
        val chapters = ArrayList<BookChapter>()
        val firstChild = outline.getFirstChild() ?: return chapters
        processOutline(firstChild, chapters, book, doc)
        // Set start/end page indices for outline-based chapters
        if (chapters.isNotEmpty()) {
            for (i in 0 until chapters.size) {
                chapters[i].index = i
                if (i + 1 < chapters.size) {
                    chapters[i].end = chapters[i + 1].start
                } else {
                    chapters[i].end = doc.numberOfPages.toLong()
                }
            }
        }
        return chapters
    }

    /**
     * Recursively process PDF outline tree to build chapter list.
     */
    private fun processOutline(bookmark: PDOutlineItem, chapters: ArrayList<BookChapter>, book: Book, doc: PDDocument) {
        var current: PDOutlineItem? = bookmark
        while (current != null) {
            val chapter = BookChapter()
            chapter.bookUrl = book.bookUrl
            chapter.title = current.title ?: "Untitled"
            // Try to determine the page number from the destination
            val pageIndex = getPageIndex(current, doc)
            chapter.url = "pdf_page_${pageIndex}"
            chapter.start = pageIndex.toLong()
            chapter.end = pageIndex.toLong() + 1
            chapters.add(chapter)

            // Process children recursively
            val firstChild = current.getFirstChild()
            if (firstChild != null) {
                processOutline(firstChild, chapters, book, doc)
            }
            current = current.nextSibling
        }
    }

    /**
     * Get the page index for a given outline item.
     */
    private fun getPageIndex(item: PDOutlineItem, doc: PDDocument): Int {
        try {
            val dest = item.destination
            if (dest != null && dest is org.apache.pdfbox.pdmodel.interactive.documentnavigation.destination.PDPageDestination) {
                val pageNum = dest.retrievePageNumber()
                if (pageNum >= 0) {
                    return pageNum
                }
            }
            // Try action-based destination
            val action = item.action
            if (action != null && action is org.apache.pdfbox.pdmodel.interactive.action.PDActionGoTo) {
                val actionDest = action.destination
                if (actionDest != null && actionDest is org.apache.pdfbox.pdmodel.interactive.documentnavigation.destination.PDPageDestination) {
                    val pageNum = actionDest.retrievePageNumber()
                    if (pageNum >= 0) {
                        return pageNum
                    }
                }
            }
        } catch (e: Exception) {
            logger.warn("getPageIndex error: {}", e.message)
        }
        return 0
    }

    /**
     * Extract text content from a PDF for the given chapter.
     * The chapter's start/end fields indicate page range (0-based, end exclusive).
     */
    fun getContent(book: Book, chapter: BookChapter, inputStream: InputStream): String? {
        var doc: PDDocument? = null
        try {
            doc = PDDocument.load(inputStream)
            val stripper = PDFTextStripper()
            val startPage = (chapter.start ?: 0).toInt()
            val endPage = (chapter.end ?: (startPage + 1).toLong()).toInt()
            // PDFTextStripper uses 1-based page numbers
            stripper.startPage = startPage + 1
            stripper.endPage = endPage
            return stripper.getText(doc)
        } catch (e: Exception) {
            logger.error("getContent error: {}", e.message)
            return null
        } finally {
            doc?.close()
        }
    }

    /**
     * Get total page count of a PDF document.
     */
    fun getPageCount(inputStream: InputStream): Int {
        var doc: PDDocument? = null
        try {
            doc = PDDocument.load(inputStream)
            return doc.numberOfPages
        } catch (e: Exception) {
            logger.error("getPageCount error: {}", e.message)
            return 0
        } finally {
            doc?.close()
        }
    }
}
