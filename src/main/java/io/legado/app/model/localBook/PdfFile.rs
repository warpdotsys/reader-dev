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

    fn update_cover(&mut self) {
        // fix: 默认封面写入（原空实现——刷新本地书时封面不生成；1x1 深色 PNG，与 EPUB 导出默认封面一致）
        let cover_file = format!("{}.jpg", crate::stubs::md5_encode16(self.book.book_url.clone()));
        let relative_cover_url = std::path::Path::new("assets")
            .join(self.book.get_user_name_space())
            .join("covers")
            .join(&cover_file)
            .to_string_lossy()
            .to_string();
        self.book.cover_url = Some("/".to_string() + &relative_cover_url.replace("\\", "/"));
        let cover_url = std::path::Path::new(&self.book.work_root())
            .join("storage")
            .join(&relative_cover_url)
            .to_string_lossy()
            .to_string();
        if !std::path::Path::new(&cover_url).exists() {
            if let Some(parent) = std::path::Path::new(&cover_url).parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            const DEFAULT_COVER_PNG: &[u8] = &[
                0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
                0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
                0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00,
                0x00, 0x00, 0x03, 0x00, 0x01, 0x25, 0x45, 0x40, 0x5C, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
                0x44, 0xAE, 0x42, 0x60, 0x82,
            ];
            let _ = std::fs::write(&cover_url, DEFAULT_COVER_PNG);
        }
    }

    fn get_content_inner(&mut self, chapter: &BookChapter) -> Option<String> {
        // PDF 页文本提取（lopdf 解析；图片化渲染依赖系统级 PDF 渲染器，见已知限制）
        let local_file = self.book.get_local_file().path();
        let document = PDDocument::load(&local_file);
        let idx = chapter.start.unwrap_or(0).max(0) as usize;
        let text = document
            .document_catalog
            .pages
            .pages
            .get(idx)
            .map(|p| p.text.clone())
            .unwrap_or_default();
        close_quietly(&document);
        if text.is_empty() {
            None
        } else {
            Some(text)
        }
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
