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
        e.umd_book = e.get_umd_book();
        return e;
    }

    // companion object {
    //     private var uFile: UmdFile? = null

    //     @Synchronized
    fn get_u_file(book: Book) -> UmdFile {
        if U_FILE.is_none() || U_FILE.as_ref().unwrap().book.book_url != book.book_url {
            U_FILE = Some(UmdFile::new(book));
            return U_FILE.as_ref().unwrap().clone();
        }
        U_FILE.as_mut().unwrap().book = book;
        return U_FILE.as_ref().unwrap().clone();
    }

    //     @Synchronized
    pub fn get_chapter_list(book: Book) -> Vec<BookChapter> {
        return Self::get_u_file(book).get_chapter_list();
    }

    //     @Synchronized
    pub fn get_content(book: Book, chapter: BookChapter) -> Option<String> {
        return Self::get_u_file(book).get_content(chapter);
    }

    //     @Synchronized
    pub fn get_image(book: Book, href: String) -> Option<InputStream> {
        return Self::get_u_file(book).get_image(href);
    }

    //     @Synchronized
    pub fn up_book_info(book: Book, only_cover: bool) {
        if only_cover {
            return Self::get_u_file(book).update_cover();
        }
        return Self::get_u_file(book).up_book_info();
    }
    // }

    // private var umdBook: UmdBook? = null
    //     get() {
    //         if (field != null) {
    //             return field
    //         }
    //         field = readUmd()
    //         return field
    //     }

    fn get_umd_book(&mut self) -> Option<UmdBook> {
        if self.umd_book.is_some() {
            return self.umd_book.clone();
        }
        self.umd_book = self.read_umd();
        return self.umd_book.clone();
    }

    fn read_umd(&self) -> Option<UmdBook> {
        let input = LocalBook::get_book_input_stream(&self.book);
        return UmdReader::new().read(input);
    }

    fn up_book_info(&mut self) {
        if self.umd_book.is_none() {
            U_FILE = None;
            self.book.intro = "书籍导入异常".to_string();
        } else {
            let hd = self.umd_book.as_ref().unwrap().header.clone();
            self.book.name = hd.title;
            self.book.author = hd.author;
            self.book.kind = hd.book_type;

            self.update_cover();
        }
    }

    fn update_cover(&mut self) {
        if self.umd_book.is_none() {
            U_FILE = None;
            return;
        }
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
            write_bytes(&cover_url, &self.umd_book.as_ref().unwrap().cover.cover_data);
        }
    }

    fn get_content(&self, chapter: BookChapter) -> Option<String> {
        return self.umd_book.as_ref()
            .and_then(|b| b.chapters.as_ref())
            .map(|c| c.get_content_string(chapter.index));
    }

    fn get_chapter_list(&mut self) -> Vec<BookChapter> {
        let mut chapter_list = Vec::new();
        if let Some(chapters) = self.umd_book.as_ref().and_then(|b| b.chapters.as_ref()) {
            let titles = chapters.titles.clone();
            for (index, _) in titles.iter().enumerate() {
                let title = chapters.get_title(index);
                let mut chapter = BookChapter::new();
                chapter.title = title;
                chapter.index = index;
                chapter.book_url = self.book.book_url.clone();
                chapter.url = index.to_string();
                println!("UMD{}", chapter.url);
                chapter_list.push(chapter);
            }
        }
        self.book.latest_chapter_title = chapter_list.last().map(|c| c.title.clone());
        self.book.total_chapter_num = chapter_list.len();
        return chapter_list;
    }

    fn get_image(&self, href: String) -> Option<InputStream> {
        return None;
    }
}

// companion object 中的 uFile 静态字段
static mut U_FILE: Option<UmdFile> = None;
