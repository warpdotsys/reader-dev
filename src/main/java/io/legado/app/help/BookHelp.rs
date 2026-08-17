use crate::prelude::*;
// fix: 显式导入以覆盖 prelude 中多个 glob 重导出导致的同名歧义（File ← stubs/ResourceUtil；FileUtils ← stubs/FilesUtil）
use crate::io_legado_app_utils_filesutil::FileUtils;
use crate::stubs::File;
// package io.legado.app.help
//
// import io.legado.app.constant.AppPattern
// import io.legado.app.data.entities.Book
// import io.legado.app.data.entities.BookChapter
// import io.legado.app.data.entities.BookSource
// import io.legado.app.utils.FileUtils
// import io.legado.app.utils.NetworkUtils
// import io.legado.app.utils.MD5Utils
// import io.legado.app.utils.getFile
// import io.legado.app.model.analyzeRule.AnalyzeUrl
// import io.legado.app.model.localBook.LocalBook
// import java.io.File
// import java.util.concurrent.CopyOnWriteArraySet
// import kotlinx.coroutines.Deferred
// import kotlinx.coroutines.async
// import kotlinx.coroutines.delay
// import kotlinx.coroutines.Dispatchers
// import kotlinx.coroutines.CoroutineScope

//import org.apache.commons.text.similarity.JaccardSimilarity

// fix: Kotlin `private val downloadImages = CopyOnWriteArraySet<String>()` 的转录占位（并发去重；原 no-op 导致重复下载/写坏）
static download_images: std::sync::LazyLock<DownloadImages> = std::sync::LazyLock::new(|| DownloadImages {
    set: std::sync::Mutex::new(std::collections::HashSet::new()),
});

struct DownloadImages {
    set: std::sync::Mutex<std::collections::HashSet<String>>,
}

impl DownloadImages {
    fn contains(&self, src: &str) -> bool {
        self.set.lock().unwrap().contains(src)
    }
    fn add(&self, src: &str) {
        self.set.lock().unwrap().insert(src.to_string());
    }
    fn remove(&self, src: &str) {
        self.set.lock().unwrap().remove(src);
    }
}

pub struct BookHelp;

impl BookHelp {
    const cache_image_folder_name: &'static str = "images";

    pub fn format_folder_name(folder_name: &str) -> String {
        // return folderName.replace("[\\\\/:*?\"<>|.]".toRegex(), "")
        regex::Regex::new("[\\\\/:*?\"<>|.]").unwrap().replace_all(folder_name, "").into_owned()
    }

    pub fn format_author(author: Option<&str>) -> String {
        // return author
        //     ?.replace("作\\s*者[\\s:：]*".toRegex(), "")
        //     ?.replace("\\s+".toRegex(), " ")
        //     ?.trim { it <= ' ' }
        //     ?: ""
        match author {
            Some(author) => {
                let mut result = author.to_string();
                result = regex::Regex::new("作\\s*者[\\s:：]*").unwrap().replace_all(&result, "").into_owned();
                result = regex::Regex::new("\\s+").unwrap().replace_all(&result, " ").into_owned();
                result.trim_matches(|it: char| it <= ' ').to_string()
            }
            None => "".to_string(),
        }
    }

    /**
     * 格式化书名
     */
    pub fn format_book_name(name: &str) -> String {
        // return name
        //     .replace(AppPattern.nameRegex, "")
        //     .trim { it <= ' ' }
        AppPattern::nameRegex().replace_all(name, "").into_owned()
            .trim_matches(|it: char| it <= ' ').to_string()
    }

    /**
     * 格式化作者
     */
    pub fn format_book_author(author: &str) -> String {
        // return author
        //     .replace(AppPattern.authorRegex, "")
        //     .trim { it <= ' ' }
        AppPattern::authorRegex().replace_all(author, "").into_owned()
            .trim_matches(|it: char| it <= ' ').to_string()
    }

    pub fn get_book_cache_dir(book: &Book) -> File {
        // val md5Encode = MD5Utils.md5Encode(book.bookUrl).toString()
        let md5_encode = MD5Utils::md5Encode(Some(book.book_url.as_str())).to_string();
        let book_dir = book.get_book_dir();
        // if (bookDir.isEmpty()) {
        //     throw Exception("bookDir不能为空")
        // }
        if book_dir.is_empty() {
            panic!("bookDir不能为空")
        }
        let local_cache_dir = File::new(&book_dir).resolve(md5_encode.as_str());
        // if (!localCacheDir.exists()) {
        //     localCacheDir.mkdirs()
        // }
        if !local_cache_dir.exists() {
            let _ = std::fs::create_dir_all(&local_cache_dir.path());
        }
        local_cache_dir
    }

    /**
     * 读取章节内容
     */
    pub fn get_content(book: &mut Book, book_chapter: &BookChapter) -> Option<String> {
        let file = Self::get_book_cache_dir(book).resolve(&format!("{}.txt", book_chapter.index));
        if file.exists() {
            return Some(file.read_text());
        }
        if book.is_local_book() {
            let content = LocalBook::get_content(book, book_chapter);
            if content.is_some() && book.is_epub() {
                Self::save_text(book, book_chapter, content.as_ref().unwrap());
            }
            return content;
        }
        None
    }

    /**
     * 删除章节内容
     */
    pub fn del_content(book: &Book, book_chapter: &BookChapter) {
        // FileUtils.createFileIfNotExist(
        //     getBookCacheDir(book),
        //     String.format("%d.txt", bookChapter.index)
        // ).delete()
        FileUtils::createFileIfNotExist(
            &Self::get_book_cache_dir(book),
            &[format!("{}.txt", book_chapter.index).as_str()],
        ).delete();
    }

    pub async fn save_content(
        scope: &CoroutineScope,
        book_source: &BookSource,
        book: &Book,
        book_chapter: &BookChapter,
        content: &str,
    ) {
        Self::save_text(book, book_chapter, content);
        Self::save_images(scope, book_source, book, book_chapter, content).await;
    }

    pub fn save_text(
        book: &Book,
        book_chapter: &BookChapter,
        content: &str,
    ) {
        // if (content.isEmpty()) return
        //保存文本
        FileUtils::createFileIfNotExist(
            &Self::get_book_cache_dir(book),
            &[format!("{}.txt", book_chapter.index).as_str()],
        ).write_text(content);
    }

    pub async fn save_images(
        _scope: &CoroutineScope,
        book_source: &BookSource,
        book: &Book,
        book_chapter: &BookChapter,
        content: &str,
    ) {
        let mut img_urls: Vec<String> = Vec::new();
        for it in content.split('\n') {
            let matcher = AppPattern::imgPattern().find(it);
            if let Some(matcher) = matcher {
                let src = matcher.group_values(1);
                if !src.is_empty() {
                    let m_src = NetworkUtils::getAbsoluteURL(Some(book_chapter.url.as_str()), &src);
                    img_urls.push(m_src);
                }
            }
        }
        for url in img_urls {
            Self::save_image(Some(book_source), book, &url).await;
        }
    }

    pub async fn save_image(book_source: Option<&BookSource>, book: &Book, src: &str) {
        let mut wait_count = 0;
        while download_images.contains(src) && wait_count < 100 {
            delay(100).await;
            wait_count += 1;
        }
        if Self::get_image(book, src).exists() {
            return;
        }
        struct DownloadGuard<'a>(&'a str);
        impl<'a> Drop for DownloadGuard<'a> {
            fn drop(&mut self) {
                download_images.remove(self.0);
            }
        }
        download_images.add(src);
        let _guard = DownloadGuard(src);

        let mut analyze_url = AnalyzeUrl::new(
            src.to_string(),
            None,
            None,
            None,
            None,
            String::new(),
            book_source.cloned(),
            None,
            None,
            book_source.and_then(|bs| bs.get_header_map()),
            None,
        );
        let bytes = analyze_url.get_byte_array_await().await;
        FileUtils::createFileIfNotExist(
            &Self::get_book_cache_dir(book),
            &[
                Self::cache_image_folder_name,
                format!("{}.{}", MD5Utils::md5Encode16(src), Self::get_image_suffix(src)).as_str(),
            ],
        ).write_bytes(bytes);
    }

    pub fn get_image(book: &Book, src: &str) -> File {
        Self::get_book_cache_dir(book).resolve(
            Self::cache_image_folder_name,
        ).resolve(&format!("{}.{}", MD5Utils::md5Encode16(src), Self::get_image_suffix(src)))
    }

    pub fn get_image_suffix(src: &str) -> String {
        // var suffix = src.substringAfterLast(".").substringBefore(",")
        let mut suffix = match src.rfind('.') {
            Some(idx) => &src[idx + 1..],
            None => "",
        };
        // substringBefore(",")
        suffix = suffix.split(',').next().unwrap_or("");
        //检查截取的后缀字符是否合法 [a-zA-Z0-9]
        let file_suffix_regex = regex::Regex::new("(?i)^[a-z0-9]+$").unwrap();
        let mut suffix = suffix.to_string();
        if suffix.len() > 5 || !file_suffix_regex.is_match(&suffix) {
            suffix = "jpg".to_string();
        }
        suffix
    }
}
