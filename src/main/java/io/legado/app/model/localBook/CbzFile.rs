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
    pub info: Option<std::collections::HashMap<String, Box<dyn Any>>>,
    pub cover: Option<InputStream>,
}

impl CbzFile {

    pub fn new(book: Book) -> Self {
        CbzFile { book, info: None, cover: None }
    }

    // companion object {
    //     private var cFile: CbzFile? = null

    //     @Synchronized
    fn get_cbz_file(book: Book) -> CbzFile {
        if C_FILE.is_none() || C_FILE.as_ref().unwrap().book.book_url != book.book_url {
            C_FILE = Some(CbzFile::new(book));
            return C_FILE.as_ref().unwrap().clone();
        }
        C_FILE.as_mut().unwrap().book = book;
        return C_FILE.as_ref().unwrap().clone();
    }

    //     @Synchronized
    pub fn get_chapter_list(book: Book) -> Vec<BookChapter> {
        return Self::get_cbz_file(book).get_chapter_list();
    }

    //     @Synchronized
    pub fn get_content(book: Book, chapter: BookChapter) -> Option<String> {
        return Self::get_cbz_file(book).get_content(chapter);
    }

    //     @Synchronized
    pub fn up_book_info(book: Book, only_cover: bool) {
        if only_cover {
            return Self::get_cbz_file(book).update_cover();
        }
        return Self::get_cbz_file(book).up_book_info();
    }
    // }

    // init {
    // }

    fn parse_book_info(&mut self) -> (Option<std::collections::HashMap<String, Box<dyn Any>>>, Option<InputStream>) {
        if self.cover.is_some() || self.info.is_some() {
            return (self.info.clone(), self.cover.clone());
        }
        let zf = ZipFile::new(self.book.get_local_file());
        let mut entries = zf.entries();
        let image_ext = vec!["jpg", "jpeg", "gif", "png", "bmp", "webp", "svg"];

        while entries.has_more_elements() {
            let zip_entry: ZipEntry = entries.next_element();

            if !zip_entry.is_directory {
                let name = zip_entry.name;
                if name == "ComicInfo.xml" {
                    // 解析书籍信息
                    let input_stream = zf.get_input_stream(&zip_entry);
                    self.info = xml2map(input_stream);
                } else if self.cover.is_none() {
                    // 解析第一张图片
                    let ext = get_file_extetion(&name).to_lowercase();
                    if image_ext.contains(&ext.as_str()) {
                        self.cover = Some(zf.get_input_stream(&zip_entry));
                    }
                }
            }
            if self.cover.is_some() && self.info.is_some() {
                break;
            }
        }

        return (self.info.clone(), self.cover.clone());
    }

    fn up_book_info(&mut self) {
        let result = self.parse_book_info();
        if let Some(book_info_map) = result.0 {
            let book_info = book_info_map.get("ComicInfo")
                .map(|v| v as &dyn Any)
                .and_then(|v| v.downcast_ref::<std::collections::HashMap<String, Any>>());
            self.book.name = book_info
                .and_then(|info| info.get("Title"))
                .and_then(|v| v.downcast_ref::<String>())
                .unwrap_or(&self.book.name)
                .clone();
            self.book.author = book_info
                .and_then(|info| info.get("Writer"))
                .and_then(|v| v.downcast_ref::<String>())
                .unwrap_or(&self.book.author)
                .clone();
        }
        self.update_cover();
    }

    fn update_cover(&mut self) {
        let cover_file = format!("{}.jpg", md5_encode16(&self.book.book_url));
        let relative_cover_url = std::path::Path::new("assets")
            .join(self.book.get_user_name_space())
            .join("covers")
            .join(&cover_file)
            .to_string_lossy()
            .to_string();
        self.book.cover_url = "/".to_string() + &relative_cover_url.replace("\\", "/");
        let cover_url = std::path::Path::new(&self.book.work_root())
            .join("storage")
            .join(&relative_cover_url)
            .to_string_lossy()
            .to_string();
        if !File::new(&cover_url).exists() {
            let result = self.parse_book_info();
            if let Some(cover_stream) = result.1 {
                write_input_stream(&cover_url, &cover_stream);
            }
        }
    }

    fn get_content(&self, chapter: BookChapter) -> Option<String> {
        return Some(String::new());
    }

    fn get_chapter_list(&mut self) -> Vec<BookChapter> {
        let mut chapter_list = Vec::new();
        let zf = ZipFile::new(self.book.get_local_file());
        let entries = zf.entries();
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
            let mut chapter = BookChapter::new();
            chapter.title = name.clone();
            chapter.index = i;
            chapter.book_url = self.book.book_url.clone();
            chapter.url = name;
            chapter_list.push(chapter);
        }
        self.book.latest_chapter_title = chapter_list.last().map(|c| c.title.clone());
        self.book.total_chapter_num = chapter_list.len();
        return chapter_list;
    }
}

// companion object 中的 cFile 静态字段
static mut C_FILE: Option<CbzFile> = None;
