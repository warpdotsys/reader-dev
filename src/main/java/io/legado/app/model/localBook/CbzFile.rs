use crate::prelude::*;
// fix: 显式导入覆盖 prelude 中多个 glob 重导出导致的同名歧义（Any/File/ZipFile/ZipEntry/FileUtils 等）
use crate::io_legado_app_utils_filesutil::FileUtils;
use crate::stubs::{Any, File, FileInputStream, ZipEntry, ZipFile, xml2map, write_input_stream};
// package io.legado.app.model.localBook
//
// import io.legado.app.data.entities.Book
// import io.legado.app.data.entities.BookChapter
// import io.legado.app.utils.*
// import java.io.File
// import java.io.InputStream
// import java.util.*
// import java.nio.file.Paths
// import java.util.zip.ZipFile
// import java.util.zip.ZipEntry
// import java.util.zip.ZipOutputStream
// import io.legado.app.utils.FileUtils.getFileExtetion
// import io.legado.app.utils.XmlUtils.xml2map

pub struct CbzFile {
    pub book: Book,
    pub info: Option<std::collections::HashMap<String, Any>>,
    pub cover: Option<FileInputStream>,
}

impl CbzFile {

    pub fn new(book: Book) -> Self {
        CbzFile { book, info: None, cover: None }
    }

    // companion object {
    //     private var cFile: CbzFile? = None

    //     @Synchronized
    // fix: Kotlin `getCbzFile(book)`（@Synchronized 语义以 Mutex 实现；返回闭包结果以规避克隆不可克隆的输入流）；
    //      以 &mut Book 传入（与 EpubFile 一致），克隆进缓存实例，结束同步回调用方
    fn with_c_file<T>(book: &mut Book, f: impl FnOnce(&mut CbzFile) -> T) -> T {
        let mut guard = C_FILE.lock().unwrap();
        if guard.is_none() || guard.as_ref().unwrap().book.book_url != book.book_url {
            *guard = Some(CbzFile::new(book.clone()));
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

    // init {
    // }

    // fix: Kotlin `parseBookInfo(): Pair<HashMap<String, Any>?, InputStream?>`——封面流缓存于 self.cover，
    //      仅返回 info（流的读取方直接访问 self.cover），避免克隆不可克隆的 FileInputStream
    fn parse_book_info(&mut self) -> Option<std::collections::HashMap<String, Any>> {
        if self.cover.is_some() || self.info.is_some() {
            return self.info.clone();
        }
        let zf = ZipFile::new(&self.book.get_local_file());
        let mut entries = zf.entries();
        let image_ext = vec!["jpg", "jpeg", "gif", "png", "bmp", "webp", "svg"];

        while entries.has_more_elements() {
            let zip_entry: ZipEntry = entries.next_element();

            if !zip_entry.is_directory {
                let name = zip_entry.name.clone();
                if name == "ComicInfo.xml" {
                    // 解析书籍信息
                    let input_stream = zf.get_input_stream(&zip_entry);
                    self.info = Some(xml2map(&input_stream));
                } else if self.cover.is_none() {
                    // 解析第一张图片
                    let ext = FileUtils::getFileExtetion(&name).to_lowercase();
                    if image_ext.contains(&ext.as_str()) {
                        self.cover = Some(zf.get_input_stream(&zip_entry));
                    }
                }
            }
            if self.cover.is_some() && self.info.is_some() {
                break;
            }
        }

        return self.info.clone();
    }

    fn up_book_info_inner(&mut self) {
        if let Some(book_info_map) = self.parse_book_info() {
            let book_info = book_info_map.get("ComicInfo").and_then(|v| v.as_map());
            self.book.name = book_info
                .as_ref()
                .and_then(|info| info.get("Title"))
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| self.book.name.clone());
            self.book.author = book_info
                .as_ref()
                .and_then(|info| info.get("Writer"))
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| self.book.author.clone());
        }
        self.update_cover();
    }

    fn update_cover(&mut self) {
        let cover_file = format!("{}.jpg", md5_encode16(self.book.book_url.clone()));
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
        if !File::new(&cover_url).exists() {
            // fix: Kotlin `parseBookInfo().second?.let { writeInputStream(coverUrl, it) }`——封面流已缓存于 self.cover
            self.parse_book_info();
            if let Some(cover_stream) = self.cover.as_mut() {
                write_input_stream(&cover_url, cover_stream);
            }
        }
    }

    fn get_content_inner(&self, chapter: &BookChapter) -> Option<String> {
        // fix: 章节正文解析未转录，占位返回空串
        return Some(String::new());
    }

    fn get_chapter_list_inner(&mut self) -> Vec<BookChapter> {
        let mut chapter_list = Vec::new();
        let zf = ZipFile::new(&self.book.get_local_file());
        let mut entries = zf.entries();
        let mut image_file_list = Vec::<String>::new();
        while entries.has_more_elements() {
            let zip_entry: ZipEntry = entries.next_element();

            if !zip_entry.is_directory {
                let name = zip_entry.name;
                if !name.ends_with(".xml") {
                    // 只获取图片文件
                    image_file_list.push(name);
                }
            }
        }
        // 排序
        image_file_list.sort();
        for i in 0..image_file_list.len() {
            let name = image_file_list[i].clone();
            let mut chapter = BookChapter::default();
            chapter.title = name.clone();
            chapter.index = i as i32;
            chapter.book_url = self.book.book_url.clone();
            chapter.url = name;
            chapter_list.push(chapter);
        }
        self.book.latest_chapter_title = chapter_list.last().map(|c| c.title.clone());
        self.book.total_chapter_num = chapter_list.len() as i32;
        return chapter_list;
    }
}

// companion object 中的 cFile 静态字段
// fix: Kotlin `private var cFile`（@Synchronized 语义）——`static mut` 改 Mutex 以消除 E0133 并支持共享访问
pub static C_FILE: std::sync::Mutex<Option<CbzFile>> = std::sync::Mutex::new(None);
