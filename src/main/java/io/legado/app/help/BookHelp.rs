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
        AppPattern::name_regex.replace_all(name, "").into_owned()
            .trim_matches(|it: char| it <= ' ').to_string()
    }

    /**
     * 格式化作者
     */
    pub fn format_book_author(author: &str) -> String {
        // return author
        //     .replace(AppPattern.authorRegex, "")
        //     .trim { it <= ' ' }
        AppPattern::author_regex.replace_all(author, "").into_owned()
            .trim_matches(|it: char| it <= ' ').to_string()
    }

    pub fn get_book_cache_dir(book: &Book) -> File {
        // val md5Encode = MD5Utils.md5Encode(book.bookUrl).toString()
        let md5_encode = MD5Utils::md5_encode(&book.book_url).to_string();
        let book_dir = book.get_book_dir();
        // if (bookDir.isEmpty()) {
        //     throw Exception("bookDir不能为空")
        // }
        if book_dir.is_empty() {
            panic!("bookDir不能为空")
        }
        let local_cache_dir = File::new(&book_dir).get_file(&md5_encode);
        // if (!localCacheDir.exists()) {
        //     localCacheDir.mkdirs()
        // }
        if !local_cache_dir.exists() {
            let _ = std::fs::create_dir_all(&local_cache_dir.path);
        }
        local_cache_dir
    }

    /**
     * 读取章节内容
     */
    pub fn get_content(book: &Book, book_chapter: &BookChapter) -> Option<String> {
        let file = get_book_cache_dir(book).get_file(format!("%d.txt", book_chapter.index));
        if file.exists() {
            return Some(file.read_text());
        }
        if book.is_local_book() {
            let content = LocalBook::get_content(book, book_chapter);
            if content.is_some() && book.is_epub() {
                save_text(book, book_chapter, content.as_ref().unwrap());
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
        FileUtils::create_file_if_not_exist(
            get_book_cache_dir(book),
            format!("%d.txt", book_chapter.index),
        ).delete();
    }

    pub async fn save_content(
        scope: &CoroutineScope,
        book_source: &BookSource,
        book: &Book,
        book_chapter: &BookChapter,
        content: &str,
    ) {
        save_text(book, book_chapter, content);
        save_images(scope, book_source, book, book_chapter, content).await;
    }

    pub fn save_text(
        book: &Book,
        book_chapter: &BookChapter,
        content: &str,
    ) {
        // if (content.isEmpty()) return
        //保存文本
        FileUtils::create_file_if_not_exist(
            get_book_cache_dir(book),
            format!("%d.txt", book_chapter.index),
        ).write_text(content);
    }

    pub async fn save_images(
        scope: &CoroutineScope,
        book_source: &BookSource,
        book: &Book,
        book_chapter: &BookChapter,
        content: &str,
    ) {
        // val awaitList = arrayListOf<Deferred<Int>>()
        let mut await_list: Vec<Deferred<i32>> = Vec::new();
        // content.split("\n").forEach {
        //     val matcher = AppPattern.imgPattern.matcher(it)
        //     if (matcher.find()) {
        //         matcher.group(1)?.let { src ->
        //             val mSrc = NetworkUtils.getAbsoluteURL(bookChapter.url, src)
        //             val req: Deferred<Int> = scope.async {
        //                 saveImage(bookSource, book, mSrc)
        //                 return@async 1
        //             }
        //             awaitList.add(req)
        //         }
        //     }
        // }
        for it in content.split("\n") {
            let matcher = AppPattern::img_pattern.find(it);
            if let Some(matcher) = matcher {
                if let Some(src) = matcher.group(1) {
                    let m_src = NetworkUtils::get_absolute_url(&book_chapter.url, src);
                    let req: Deferred<i32> = scope.async(|| async move {
                        save_image(book_source, book, m_src).await;
                        return 1;
                    });
                    await_list.push(req);
                }
            }
        }
        // awaitList.forEach {
        //     it.await()
        // }
        for it in await_list {
            it.await().await;
        }
    }

    pub async fn save_image(book_source: Option<&BookSource>, book: &Book, src: &str) {
        // while (downloadImages.contains(src)) {
        //     delay(100)
        // }
        while download_images.contains(src) {
            delay(100).await;
        }
        if get_image(book, src).exists() {
            return;
        }
        download_images.add(src);
        let analyze_url = AnalyzeUrl::new(src, source = book_source);
        // try {
        //     analyzeUrl.getByteArrayAwait().let {
        //         FileUtils.createFileIfNotExist(
        //             getBookCacheDir(book),
        //             cacheImageFolderName,
        //             "${MD5Utils.md5Encode16(src)}.${getImageSuffix(src)}"
        //         ).writeBytes(it)
        //     }
        // } catch (e: Exception) {
        //     e.printStackTrace()
        // } finally {
        //     downloadImages.remove(src)
        // }
        match analyze_url.get_byte_array_await().await {
            Ok(it) => {
                FileUtils::create_file_if_not_exist(
                    get_book_cache_dir(book),
                    Self::cache_image_folder_name,
                    format!("{}.{}", MD5Utils::md5_encode16(src), get_image_suffix(src)),
                ).write_bytes(it);
            }
            Err(e) => {
                e.print_stack_trace();
            }
        }
        download_images.remove(src);
    }

    pub fn get_image(book: &Book, src: &str) -> File {
        get_book_cache_dir(book).get_file(
            Self::cache_image_folder_name,
            format!("{}.{}", MD5Utils::md5_encode16(src), get_image_suffix(src)),
        )
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
