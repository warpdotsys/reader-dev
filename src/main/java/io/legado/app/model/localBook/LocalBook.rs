// package io.legado.app.model.localBook
//
// import io.legado.app.constant.AppConst
// import io.legado.app.constant.AppPattern
// import io.legado.app.data.entities.Book
// import io.legado.app.data.entities.BookChapter
// import io.legado.app.help.BookHelp
// import io.legado.app.utils.*
// import io.legado.app.exception.TocEmptyException
// import java.io.File
// import java.io.FileInputStream
// import java.io.FileNotFoundException
// import java.io.InputStream
// import java.util.regex.Matcher
// import java.util.regex.Pattern
// import javax.script.SimpleBindings

pub struct LocalBook;

impl LocalBook {

    // private val nameAuthorPatterns = arrayOf(
    //     Pattern.compile("(.*?)《([^《》]+)》.*?作者：(.*)"),
    //     Pattern.compile("(.*?)《([^《》]+)》(.*)"),
    //     Pattern.compile("(^)(.+) 作者：(.+)$"),
    //     Pattern.compile("(^)(.+) by (.+)$")
    // )
    fn name_author_patterns() -> Vec<Regex> {
        vec![
            Regex::new("(.*?)《([^《》]+)》.*?作者：(.*)").unwrap(),
            Regex::new("(.*?)《([^《》]+)》(.*)").unwrap(),
            Regex::new("(^)(.+) 作者：(.+)$").unwrap(),
            Regex::new("(^)(.+) by (.+)$").unwrap(),
        ]
    }

    // @Throws(FileNotFoundException::class, SecurityException::class)
    pub fn get_book_input_stream(book: &Book) -> InputStream {
        let file = book.get_local_file();
        if file.exists() {
            return FileInputStream::new(&file);
        }
        panic!("{} 文件不存在", book.name);
    }

    // @Throws(Exception::class)
    pub fn get_chapter_list(book: &mut Book) -> Vec<BookChapter> {
        let chapters = if book.is_epub() {
            EpubFile::get_chapter_list(book)
        } else if book.is_umd() {
            UmdFile::get_chapter_list(book)
        } else if book.is_cbz() {
            CbzFile::get_chapter_list(book)
        } else if book.is_pdf() {
            PdfFile::get_chapter_list(book)
        } else {
            TextFile::get_chapter_list(book)
        };
        if chapters.is_empty() {
            panic!("Chapterlist is empty  {}", book.get_local_file());
        }
        return chapters;
    }

    pub fn get_content(book: &mut Book, chapter: &BookChapter) -> Option<String> {
        return if book.is_epub() {
            EpubFile::get_content(book, chapter)
        } else if book.is_umd() {
            UmdFile::get_content(book, chapter)
        } else if book.is_cbz() {
            CbzFile::get_content(book, chapter)
        } else if book.is_pdf() {
            PdfFile::get_content(book, chapter)
        } else {
            TextFile::get_content(book, chapter)
        };
    }

    pub fn analyze_name_author(file_name: &str) -> (String, String) {
        let temp_file_name = file_name.substring_before_last(".");
        let mut name: String;
        let mut author: String;

        for pattern in Self::name_author_patterns() {
            let matcher = pattern.matcher(temp_file_name);
            if matcher.find() {
                name = matcher.group(2);
                let group1 = matcher.group(1).unwrap_or("");
                let group3 = matcher.group(3).unwrap_or("");
                author = format_book_author(group1 + group3);
                return (name, author);
            }
        }

        name = format_book_name(temp_file_name);
        let candidate = format_book_author(temp_file_name.replace(&name, ""));
        author = if candidate.len() != temp_file_name.len() {
            candidate
        } else {
            String::new()
        };

        return (name, author);
    }

    pub fn delete_book(book: &mut Book) {
        // kotlin.runCatching {
        let mut book_file = book.get_local_file();
        if book.is_local_txt() || book.is_umd() {
            if book_file.exists() {
                book_file.delete();
            }
        }
        if book.is_epub() {
            book_file = book_file.parent_file();
            if book_file.exists() {
                delete_file(&book_file, true);
            }
        }
        // }
    }
}
