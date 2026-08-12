use crate::prelude::*;
use std::any::Any;
use crate::stubs::{PDDocument, PDDocumentOutline, PDOutlineItem, PDOutlineNode, FileInputStream};
// package io.legado.app.model.localBook
//
// import io.legado.app.data.entities.Book
// import io.legado.app.data.entities.BookChapter
// import org.apache.pdfbox.pdmodel.PDDocument
// import org.apache.pdfbox.pdmodel.interactive.documentnavigation.outline.PDDocumentOutline
// import org.apache.pdfbox.pdmodel.interactive.documentnavigation.outline.PDOutlineItem
// import org.apache.pdfbox.pdmodel.interactive.documentnavigation.outline.PDOutlineNode
// import java.io.InputStream

// fix: E0277 静态 C_FILE 需 Sync——`Box<dyn Any>` 无 Send，加 Send 约束
pub struct PdfFile {
    pub book: Book,
    pub info: Option<std::collections::HashMap<String, Box<dyn Any + Send>>>,
    pub cover: Option<FileInputStream>,
}

impl PdfFile {

    pub fn new(book: Book) -> Self {
        PdfFile { book, info: None, cover: None }
    }

    // companion object {
    //     private var cFile: PdfFile? = None

    //     @Synchronized
    // fix: Kotlin `getPdfFile(book)`——返回闭包结果以规避克隆不可克隆的 FileInputStream（与 CbzFile 转录一致）；
    //      以 &mut Book 传入（与 EpubFile 一致），克隆进缓存实例，结束同步回调用方
    fn with_c_file<T>(book: &mut Book, f: impl FnOnce(&mut PdfFile) -> T) -> T {
        let mut guard = C_FILE.lock().unwrap();
        if guard.is_none() || guard.as_ref().unwrap().book.book_url != book.book_url {
            *guard = Some(PdfFile::new(book.clone()));
        } else {
            guard.as_mut().unwrap().book = book.clone();
        }
        let result = f(guard.as_mut().unwrap());
        book.clone_from(&guard.as_ref().unwrap().book);
        result
    }

    //     @Synchronized
    pub fn get_chapter_list(book: &mut Book) -> Vec<BookChapter> {
        return Self::with_c_file(book, |file| file.get_chapter_list_inner());
    }

    //     @Synchronized
    pub fn get_content(book: &mut Book, chapter: &BookChapter) -> Option<String> {
        return Self::with_c_file(book, |file| file.get_content_inner(chapter));
    }

    //     @Synchronized
    pub fn up_book_info(book: &mut Book, only_cover: bool) {
        if only_cover {
            return Self::with_c_file(book, |file| file.update_cover());
        }
        return Self::with_c_file(book, |file| file.up_book_info_inner());
    }
    // }

    // fix: Kotlin `parseBookInfo(): Pair<HashMap<String, Any>?, InputStream?>`——封面流不可克隆，
    //      仅返回 info 引用（与 CbzFile 转录一致），避免克隆不可克隆的输入流
    fn parse_book_info(&self) -> &Option<std::collections::HashMap<String, Box<dyn Any + Send>>> {
        return &self.info;
    }

    fn up_book_info_inner(&mut self) {
        // fix: E0506 self.parse_book_info() 对 self 的整体不可变借用覆盖了后续 self.book.* 赋值——
        //      先取出克隆的 title/author，借用在 if let 块结束时释放，再赋值
        let mut new_name: Option<String> = None;
        let mut new_author: Option<String> = None;
        if let Some(book_info_map) = self.parse_book_info() {
            let comic_info = book_info_map
                .get("ComicInfo")
                .map(|v| v.as_ref())
                .and_then(|v| v.downcast_ref::<std::collections::HashMap<String, Box<dyn Any + Send>>>());
            new_name = comic_info
                .and_then(|info| info.get("Title"))
                .and_then(|v| v.downcast_ref::<String>())
                .cloned();
            new_author = comic_info
                .and_then(|info| info.get("Writer"))
                .and_then(|v| v.downcast_ref::<String>())
                .cloned();
        }
        if new_name.is_some() {
            self.book.name = new_name.unwrap();
        }
        if new_author.is_some() {
            self.book.author = new_author.unwrap();
        }
        self.update_cover();
    }

    fn update_cover(&self) {
    }

    fn get_content_inner(&self, chapter: &BookChapter) -> Option<String> {
        return Some(String::new());
    }

    fn get_chapter_list_inner(&mut self) -> Vec<BookChapter> {
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
        let document = PDDocument::load(&self.book.get_local_file().path());
        let page_count = document.number_of_pages;
        let mut page_index = 0;
        while page_index < page_count {
            let name = format!("output-{}.png", page_index);
            let mut chapter = BookChapter::default();
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
        self.book.total_chapter_num = chapter_list.len() as i32;
        close_quietly(&document);
        return chapter_list;
    }

    fn get_chapter_list_by_outline(&mut self) -> Vec<BookChapter> {
        let mut chapter_list = Vec::new();
        let document = PDDocument::load(&self.book.get_local_file().path());
        // fix: E0505 document_catalog.document_outline 非 Copy，partial move 后不能再借 &document——改借用
        let outline = document.document_catalog.document_outline.as_ref();
        if outline.is_none() {
            return chapter_list;
        }
        self.process_outline(&document, &mut chapter_list, outline.unwrap());
        if chapter_list.len() > 0 {
            // fix: E0499 索引表达式同时不可变/可变借用 chapter_list——先取索引
            let last_index = chapter_list.len() - 1;
            chapter_list[last_index].end = Some(document.number_of_pages as i64);
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
        let mut current: Option<&PDOutlineItem> = outline.first_child.as_deref();
        while let Some(cur) = current {
            let page = cur.find_destination_page(document);
            let page_index = document.document_catalog.pages.index_of(&page);
            if chapter_list.len() == 0 && page_index >= 1 {
                let mut first_chapter = BookChapter::default();
                first_chapter.title = "首章".to_string();
                first_chapter.index = 0;
                first_chapter.book_url = self.book.book_url.clone();
                first_chapter.url = "chapter-0".to_string();
                first_chapter.start = Some(0);
                first_chapter.end = Some(page_index as i64);
                chapter_list.push(first_chapter);
            }
            if chapter_list.len() > 0 {
                let last_start = chapter_list[chapter_list.len() - 1].start;
                if last_start.is_some() && last_start.unwrap() == page_index as i64 {
                    current = cur.next_sibling.as_deref();
                    continue;
                }
                let mut chapter = BookChapter::default();
                chapter.title = cur.title.clone();
                chapter.index = chapter_list.len() as i32;
                chapter.book_url = self.book.book_url.clone();
                chapter.url = format!("chapter-{}", chapter_list.len());
                chapter.start = Some(page_index as i64);
                // fix: E0502 索引表达式同时不可变/可变借用 chapter_list——先取索引
                let last_index = chapter_list.len() - 1;
                chapter_list[last_index].end = Some(page_index as i64 - 1);
                chapter_list.push(chapter);
            }
            if cur.has_children() {
                self.process_outline(document, chapter_list, cur);
            }
            current = cur.next_sibling.as_deref();
        }
    }
}

pub fn close_quietly(document: &PDDocument) {
    // try {
    document.close();
    // } catch (e: RuntimeException) {
    //     throw e
    // } catch (_: Exception) {
    // }
}

// companion object 中的 cFile 静态字段
// fix: Kotlin `private var cFile`（@Synchronized 语义）——`static mut` 改 Mutex 以消除 E0133 并支持共享访问
pub static C_FILE: std::sync::Mutex<Option<PdfFile>> = std::sync::Mutex::new(None);
