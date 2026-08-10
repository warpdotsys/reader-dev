// package io.legado.app.model.localBook
//
// import io.legado.app.data.entities.Book
// import io.legado.app.data.entities.BookChapter
// import org.apache.pdfbox.pdmodel.PDDocument
// import org.apache.pdfbox.pdmodel.interactive.documentnavigation.outline.PDDocumentOutline
// import org.apache.pdfbox.pdmodel.interactive.documentnavigation.outline.PDOutlineItem
// import org.apache.pdfbox.pdmodel.interactive.documentnavigation.outline.PDOutlineNode
// import java.io.InputStream

pub struct PdfFile {
    pub book: Book,
    pub info: Option<std::collections::HashMap<String, Box<dyn Any>>>,
    pub cover: Option<InputStream>,
}

impl PdfFile {

    pub fn new(book: Book) -> Self {
        PdfFile { book, info: None, cover: None }
    }

    // companion object {
    //     private var cFile: PdfFile? = null

    //     @Synchronized
    fn get_pdf_file(book: Book) -> PdfFile {
        if C_FILE.is_none() || C_FILE.as_ref().unwrap().book.book_url != book.book_url {
            C_FILE = Some(PdfFile::new(book));
            return C_FILE.as_ref().unwrap().clone();
        }
        C_FILE.as_mut().unwrap().book = book;
        return C_FILE.as_ref().unwrap().clone();
    }

    //     @Synchronized
    pub fn get_chapter_list(book: Book) -> Vec<BookChapter> {
        return Self::get_pdf_file(book).get_chapter_list();
    }

    //     @Synchronized
    pub fn get_content(book: Book, chapter: BookChapter) -> Option<String> {
        return Self::get_pdf_file(book).get_content(chapter);
    }

    //     @Synchronized
    pub fn up_book_info(book: Book, only_cover: bool) {
        if only_cover {
            return Self::get_pdf_file(book).update_cover();
        }
        return Self::get_pdf_file(book).up_book_info();
    }
    // }

    fn parse_book_info(&self) -> (Option<std::collections::HashMap<String, Box<dyn Any>>>, Option<InputStream>) {
        return (self.info.clone(), self.cover.clone());
    }

    fn up_book_info(&mut self) {
        let result = self.parse_book_info();
        if let Some(book_info_map) = result.0 {
            let comic_info = book_info_map
                .get("ComicInfo")
                .map(|v| v as &dyn Any)
                .and_then(|v| v.downcast_ref::<std::collections::HashMap<String, Any>>());
            self.book.name = comic_info
                .and_then(|info| info.get("Title"))
                .and_then(|v| v.downcast_ref::<String>())
                .unwrap_or(&self.book.name)
                .clone();
            self.book.author = comic_info
                .and_then(|info| info.get("Writer"))
                .and_then(|v| v.downcast_ref::<String>())
                .unwrap_or(&self.book.author)
                .clone();
        }
        self.update_cover();
    }

    fn update_cover(&self) {
    }

    fn get_content(&self, chapter: BookChapter) -> Option<String> {
        return Some(String::new());
    }

    fn get_chapter_list(&mut self) -> Vec<BookChapter> {
        if self.book.toc_url.is_empty() {
            self.book.toc_url = "page".to_string();
        }
        if self.book.toc_url == "page" {
            return self.get_chapter_list_by_page();
        }
        return self.get_chapter_list_by_outline();
    }

    fn get_chapter_list_by_page(&mut self) -> Vec<BookChapter> {
        let mut chapter_list = Vec::new();
        let document = PDDocument::load(self.book.get_local_file());
        let page_count = document.number_of_pages;
        let mut page_index = 0;
        while page_index < page_count {
            let name = format!("output-{}.png", page_index);
            let mut chapter = BookChapter::new();
            chapter.title = name.clone();
            chapter.index = page_index;
            chapter.book_url = self.book.book_url.clone();
            chapter.url = name;
            chapter.start = Some(page_index as i64);
            chapter.end = Some(page_index as i64);
            chapter_list.push(chapter);
            page_index += 1;
        }
        self.book.latest_chapter_title = chapter_list.last().map(|c| c.title.clone());
        self.book.total_chapter_num = chapter_list.len();
        close_quietly(&document);
        return chapter_list;
    }

    fn get_chapter_list_by_outline(&mut self) -> Vec<BookChapter> {
        let mut chapter_list = Vec::new();
        let document = PDDocument::load(self.book.get_local_file());
        let outline: Option<PDDocumentOutline> = document.document_catalog.document_outline;
        if outline.is_none() {
            return chapter_list;
        }
        self.process_outline(&document, &mut chapter_list, outline.as_ref().unwrap());
        if chapter_list.len() > 0 {
            chapter_list[chapter_list.len() - 1].end = Some(document.number_of_pages as i64);
        }
        close_quietly(&document);
        return chapter_list;
    }

    fn process_outline(
        &self,
        document: &PDDocument,
        chapter_list: &mut Vec<BookChapter>,
        outline: &PDOutlineNode
    ) {
        let mut current: Option<&PDOutlineItem> = outline.first_child.as_ref();
        while let Some(cur) = current {
            let page = cur.find_destination_page(document);
            let page_index = document.document_catalog.pages.index_of(&page);
            if chapter_list.len() == 0 && page_index >= 1 {
                let mut first_chapter = BookChapter::new();
                first_chapter.title = "首章".to_string();
                first_chapter.index = 0;
                first_chapter.book_url = self.book.book_url.clone();
                first_chapter.url = "chapter-0".to_string();
                first_chapter.start = Some(0L);
                first_chapter.end = Some(page_index as i64);
                chapter_list.push(first_chapter);
            }
            if chapter_list.len() > 0 {
                let last_start = chapter_list[chapter_list.len() - 1].start;
                if last_start.is_some() && last_start.unwrap() == page_index as i64 {
                    current = cur.next_sibling.as_ref();
                    continue;
                }
                let mut chapter = BookChapter::new();
                chapter.title = cur.title.clone();
                chapter.index = chapter_list.len();
                chapter.book_url = self.book.book_url.clone();
                chapter.url = format!("chapter-{}", chapter_list.len());
                chapter.start = Some(page_index as i64);
                chapter_list[chapter_list.len() - 1].end = Some(page_index as i64 - 1L);
                chapter_list.push(chapter);
            }
            if cur.has_children() {
                self.process_outline(document, chapter_list, cur);
            }
            current = cur.next_sibling.as_ref();
        }
    }
}

fn close_quietly(document: &PDDocument) {
    // try {
    document.close();
    // } catch (e: RuntimeException) {
    //     throw e
    // } catch (_: Exception) {
    // }
}

// companion object 中的 cFile 静态字段
static mut C_FILE: Option<PdfFile> = None;
