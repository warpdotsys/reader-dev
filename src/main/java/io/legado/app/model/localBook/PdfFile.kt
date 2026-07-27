package io.legado.app.model.localBook

import io.legado.app.data.entities.Book
import io.legado.app.data.entities.BookChapter
import mu.KotlinLogging
import org.apache.pdfbox.pdmodel.PDDocument
import org.apache.pdfbox.pdmodel.PDDocumentInformation
import org.apache.pdfbox.pdmodel.interactive.documentnavigation.outline.PDDocumentOutline
import org.apache.pdfbox.pdmodel.interactive.documentnavigation.outline.PDOutlineItem
import org.apache.pdfbox.pdmodel.interactive.documentnavigation.outline.PDOutlineNode
import org.apache.pdfbox.text.PDFTextStripper
import java.io.InputStream

private val logger = KotlinLogging.logger {}

class PdfFile(book: Book) {
    var book: Book = book
    var info: Map<String, Any> = emptyMap()
    var cover: InputStream? = null

    companion object {
        @JvmStatic
        private var cFile: PdfFile? = null

        @Synchronized
        private fun getPdfFile(book: Book): PdfFile {
            if (cFile == null || cFile!!.book.bookUrl != book.bookUrl) {
                cFile = PdfFile(book)
                cFile!!.upBookInfo()
            }
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
        fun upBookInfo(book: Book, isRefresh: Boolean = false) {
            if (isRefresh) {
                cFile = null
            }
            getPdfFile(book).upBookInfo()
        }
    }

    private fun parseBookInfo(): Pair<Map<String, Any>, InputStream?> {
        var doc: PDDocument? = null
        try {
            doc = PDDocument.load(book.getLocalFile())
            val docInfo: PDDocumentInformation = doc.documentInformation
            val infoMap = mutableMapOf<String, Any>()
            if (!docInfo.title.isNullOrBlank()) {
                infoMap["title"] = docInfo.title
            }
            if (!docInfo.author.isNullOrBlank()) {
                infoMap["author"] = docInfo.author
            }
            infoMap["pageCount"] = doc.numberOfPages
            return Pair(infoMap, null)
        } catch (e: Exception) {
            logger.error("parseBookInfo error: {}", e.message)
            return Pair(emptyMap(), null)
        } finally {
            doc?.close()
        }
    }

    private fun upBookInfo() {
        val (parsedInfo, parsedCover) = parseBookInfo()
        info = parsedInfo
        cover = parsedCover
        val title = info["title"] as? String
        if (!title.isNullOrBlank()) {
            book.name = title
        }
        val author = info["author"] as? String
        if (!author.isNullOrBlank()) {
            book.author = author
        }
        updateCover()
    }

    private fun updateCover() {
        // Cover extraction can be implemented if needed
    }

    private fun getContent(chapter: BookChapter): String? {
        var doc: PDDocument? = null
        try {
            doc = PDDocument.load(book.getLocalFile())
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

    private fun getChapterList(): ArrayList<BookChapter> {
        var doc: PDDocument? = null
        try {
            doc = PDDocument.load(book.getLocalFile())
            val chapters = getChapterListByOutline(doc)
            if (chapters.isNotEmpty()) {
                return chapters
            }
            return getChapterListByPage(doc)
        } catch (e: Exception) {
            logger.error("getChapterList error: {}", e.message)
            return arrayListOf()
        } finally {
            doc?.close()
        }
    }

    private fun getChapterListByPage(): ArrayList<BookChapter> {
        var doc: PDDocument? = null
        try {
            doc = PDDocument.load(book.getLocalFile())
            return getChapterListByPage(doc)
        } catch (e: Exception) {
            logger.error("getChapterListByPage error: {}", e.message)
            return arrayListOf()
        } finally {
            doc?.close()
        }
    }

    private fun getChapterListByPage(doc: PDDocument): ArrayList<BookChapter> {
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

    private fun getChapterListByOutline(): ArrayList<BookChapter> {
        var doc: PDDocument? = null
        try {
            doc = PDDocument.load(book.getLocalFile())
            return getChapterListByOutline(doc)
        } catch (e: Exception) {
            logger.error("getChapterListByOutline error: {}", e.message)
            return arrayListOf()
        } finally {
            doc?.close()
        }
    }

    private fun getChapterListByOutline(doc: PDDocument): ArrayList<BookChapter> {
        val chapters = ArrayList<BookChapter>()
        val outline: PDDocumentOutline? = doc.documentCatalog.documentOutline
        if (outline == null) return chapters
        processOutline(doc, chapters, outline)
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

    private fun processOutline(doc: PDDocument, chapters: ArrayList<BookChapter>, outlineNode: PDOutlineNode) {
        var current: PDOutlineItem? = outlineNode.getFirstChild() ?: return
        while (current != null) {
            val chapter = BookChapter()
            chapter.bookUrl = book.bookUrl
            chapter.title = current.title ?: "Untitled"
            val pageIndex = getPageIndex(current, doc)
            chapter.url = "pdf_page_${pageIndex}"
            chapter.start = pageIndex.toLong()
            chapter.end = pageIndex.toLong() + 1
            chapters.add(chapter)

            // Process children recursively
            val firstChild = current.getFirstChild()
            if (firstChild != null) {
                processOutline(doc, chapters, current)
            }
            current = current.nextSibling
        }
    }

    private fun getPageIndex(item: PDOutlineItem, doc: PDDocument): Int {
        try {
            val dest = item.destination
            if (dest != null && dest is org.apache.pdfbox.pdmodel.interactive.documentnavigation.destination.PDPageDestination) {
                val pageNum = dest.retrievePageNumber()
                if (pageNum >= 0) {
                    return pageNum
                }
            }
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
}
