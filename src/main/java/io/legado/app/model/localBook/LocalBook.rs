use crate::prelude::*;
// fix: 显式导入消解 prelude 多 glob 歧义（FileInputStream ← resourceutil/stubs；FileUtils ← stubs/FilesUtil）
use crate::stubs::FileInputStream;
use crate::io_legado_app_utils_filesutil::FileUtils;
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
    // fix: E0599 `regex::Regex` 无 matcher()——改用 stubs::Pattern（与 TextFile 转录一致）
    fn name_author_patterns() -> Vec<Pattern> {
        vec![
            Pattern::compile("(.*?)《([^《》]+)》.*?作者：(.*)"),
            Pattern::compile("(.*?)《([^《》]+)》(.*)"),
            Pattern::compile("(^)(.+) 作者：(.+)$"),
            Pattern::compile("(^)(.+) by (.+)$"),
        ]
    }

    // @Throws(FileNotFoundException::class, SecurityException::class)
    // fix: InputStream 为 trait，返回类型改用具体 FileInputStream；get_local_file(&mut self) 故需 &mut Book
    pub fn get_book_input_stream(book: &mut Book) -> FileInputStream {
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
        // fix: E0599 String 无 substring_before_last()——用 StringUtil 关联函数（与 EpubFile 转录一致）
        let temp_file_name = StringUtil::substring_before_last(file_name, '.');
        let mut name: String;
        let mut author: String;

        for pattern in Self::name_author_patterns() {
            let mut matcher = pattern.matcher(temp_file_name.clone());
            if matcher.find() {
                name = matcher.group_idx(2).unwrap_or_default();
                let group1 = matcher.group_idx(1).unwrap_or_default();
                let group3 = matcher.group_idx(3).unwrap_or_default();
                author = BookHelp::format_book_author(&format!("{}{}", group1, group3));
                return (name, author);
            }
        }

        name = BookHelp::format_book_name(&temp_file_name);
        let candidate = BookHelp::format_book_author(&temp_file_name.replace(&name, ""));
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
            // fix: E0308 parent_file() 返回 Option<File>——if-let 解包（Kotlin `?: return` 语义）
            if let Some(parent) = book_file.parent_file() {
                book_file = parent;
                if book_file.exists() {
                    FileUtils::delete_deleteRootDir(&book_file, true);
                }
            }
        }
        // }
    }
}
