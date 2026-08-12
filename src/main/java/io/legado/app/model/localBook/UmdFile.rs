use crate::prelude::*;
// fix: 显式导入覆盖 prelude 中多个 glob 重导出导致的同名歧义（File/FileInputStream）
use crate::stubs::{File, FileInputStream};
// package io.legado.app.model.localBook
//
// import io.legado.app.data.entities.Book
// import io.legado.app.data.entities.BookChapter
// import io.legado.app.utils.*
// import me.ag2s.umdlib.domain.UmdBook
// import me.ag2s.umdlib.umd.UmdReader
// import java.io.File
// import java.io.InputStream
// import java.util.*
// import java.nio.file.Paths

pub struct UmdFile {
    pub book: Book,
    pub umd_book: Option<UmdBook>,
}

impl UmdFile {

    pub fn new(book: Book) -> Self {
        let mut e = UmdFile {
            book,
            umd_book: None,
        };
        // init {
        //     try {
        //         umdBook?.let {
        //             // if (book.coverUrl.isNullOrEmpty()) {
        //             //     book.coverUrl = FileUtils.getPath(
        //             //         appCtx.externalFiles,
        //             //         "covers",
        //             //         "${MD5Utils.md5Encode16(book.bookUrl)}.jpg"
        //             //     )
        //             // }
        //             // if (!File(book.coverUrl!!).exists()) {
        //             //     FileUtils.writeBytes(book.coverUrl!!, it.cover.coverData)
        //             //
        //             // }
        //         }
        //     } catch (e: Exception) {
        //         e.printStackTrace()
        //     }
        // }
        // fix: Kotlin 惰性 getter `umdBook`——UmdBook 不可 Clone，构造时直接读取
        e.umd_book = e.read_umd();
        return e;
    }

    // companion object {
    //     private var uFile: UmdFile? = None

    //     @Synchronized
    // fix: Kotlin `getUFile(book)`——返回闭包结果以规避克隆（UmdBook 不可 Clone）
    fn with_u_file<T>(book: Book, f: impl FnOnce(&mut UmdFile) -> T) -> T {
        unsafe {
            if U_FILE.is_none() || U_FILE.as_ref().unwrap().book.book_url != book.book_url {
                U_FILE = Some(UmdFile::new(book));
            } else {
                U_FILE.as_mut().unwrap().book = book;
            }
            f(U_FILE.as_mut().unwrap())
        }
    }

    //     @Synchronized
    // fix: 调用方传 &mut Book（LocalBook/Book），克隆进静态缓存，调用结束后同步回
    pub fn get_chapter_list(book: &mut Book) -> Vec<BookChapter> {
        let result = Self::with_u_file(book.clone(), |file| file.get_chapter_list_inner());
        unsafe { book.clone_from(&U_FILE.as_ref().unwrap().book); }
        return result;
    }

    //     @Synchronized
    pub fn get_content(book: &mut Book, chapter: &BookChapter) -> Option<String> {
        let result = Self::with_u_file(book.clone(), |file| file.get_content_inner(chapter));
        unsafe { book.clone_from(&U_FILE.as_ref().unwrap().book); }
        return result;
    }

    //     @Synchronized
    pub fn get_image(book: &mut Book, href: String) -> Option<FileInputStream> {
        let result = Self::with_u_file(book.clone(), |file| file.get_image_inner(href));
        unsafe { book.clone_from(&U_FILE.as_ref().unwrap().book); }
        return result;
    }

    //     @Synchronized
    pub fn up_book_info(book: &mut Book, only_cover: bool) {
        if only_cover {
            Self::with_u_file(book.clone(), |file| file.update_cover());
        } else {
            Self::with_u_file(book.clone(), |file| file.up_book_info_inner());
        }
        unsafe { book.clone_from(&U_FILE.as_ref().unwrap().book); }
    }
    // }

    // private var umdBook: UmdBook? = None
    //     get() {
    //         if (field != None) {
    //             return field
    //         }
    //         field = readUmd()
    //         return field
    //     }

    // fix: 原 LocalBook::get_book_input_stream 返回 trait 类型不可用（该文件未修复），
    //      改为直接以 std::fs::File 打开本地文件（std::fs::File 实现 std::io::Read，满足 UmdReader::read）
    fn read_umd(&mut self) -> Option<UmdBook> {
        let file = self.book.get_local_file();
        if file.exists() {
            match std::fs::File::open(&file.path()) {
                Ok(mut f) => return Some(UmdReader::new().read(&mut f)),
                Err(_) => return None,
            }
        }
        return None;
    }

    // fix: 与 companion 静态 up_book_info(book, only_cover) 重名（E0592），实例方法改名
    fn up_book_info_inner(&mut self) {
        if self.umd_book.is_none() {
            unsafe { U_FILE = None; }
            self.book.intro = Some("书籍导入异常".to_string());
        } else {
            // fix: UmdHeader 不可 Clone，改借用 + 字段克隆
            let hd = &self.umd_book.as_ref().unwrap().header;
            self.book.name = hd.title.clone();
            self.book.author = hd.author.clone();
            self.book.kind = Some(hd.book_type.clone());

            self.update_cover();
        }
    }

    fn update_cover(&mut self) {
        if self.umd_book.is_none() {
            unsafe { U_FILE = None; }
            return;
        }
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
            write_bytes(&cover_url, &self.umd_book.as_ref().unwrap().cover.cover_data);
        }
    }

    // fix: 与 companion 静态 get_content(book, chapter) 重名（E0592），实例方法改名
    fn get_content_inner(&self, chapter: &BookChapter) -> Option<String> {
        return self.umd_book.as_ref()
            .map(|b| b.chapters.get_content_string(chapter.index as usize));
    }

    // fix: 与 companion 静态 get_chapter_list(book) 重名（E0592），实例方法改名
    fn get_chapter_list_inner(&mut self) -> Vec<BookChapter> {
        let mut chapter_list = Vec::new();
        if let Some(chapters) = self.umd_book.as_ref().map(|b| &b.chapters) {
            let titles = chapters.titles.clone();
            for (index, _) in titles.iter().enumerate() {
                let title = chapters.get_title(index);
                let mut chapter = BookChapter::default();
                chapter.title = title;
                chapter.index = index as i32;
                chapter.book_url = self.book.book_url.clone();
                chapter.url = index.to_string();
                println!("UMD{}", chapter.url);
                chapter_list.push(chapter);
            }
        }
        self.book.latest_chapter_title = chapter_list.last().map(|c| c.title.clone());
        self.book.total_chapter_num = chapter_list.len() as i32;
        return chapter_list;
    }

    // fix: 与 companion 静态 get_image(book, href) 重名（E0592），实例方法改名
    fn get_image_inner(&self, href: String) -> Option<FileInputStream> {
        return None;
    }
}

// companion object 中的 uFile 静态字段
// fix: update_cover/up_book_info 失败时置 None 重置缓存，保留 static mut 写法
pub static mut U_FILE: Option<UmdFile> = None;
