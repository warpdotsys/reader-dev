use crate::prelude::*;
use crate::me_ag2s_epublib_epub_resourcesloader::ZipFile;
use crate::stubs::{Charset, Element, File, InputStream};
// package io.legado.app.model.localBook
//
// import io.legado.app.data.entities.Book
// import io.legado.app.data.entities.BookChapter
// import io.legado.app.help.BookHelp
// import io.legado.app.utils.*
// import me.ag2s.epublib.domain.EpubBook
// import me.ag2s.epublib.domain.Resource
// import me.ag2s.epublib.epub.EpubReader
// import org.jsoup.Jsoup
// import org.jsoup.nodes.Element
// import org.jsoup.select.Elements
// import java.io.File
// import java.io.FileOutputStream
// import java.io.IOException
// import java.io.InputStream
// import java.nio.charset.Charset
// import java.nio.file.Paths
// import java.util.*
// import java.util.zip.ZipFile
// import mu.KotlinLogging
// private val logger = KotlinLogging.logger {}

pub fn logger_info(msg: &str) {
    // mu.KotlinLogging
}

pub struct EpubFile {
    pub book: Book,
    pub m_charset: Charset,
    pub epub_book: Option<EpubBook>,
}

impl EpubFile {

    pub fn new(book: Book) -> Self {
        let mut e = EpubFile {
            book,
            m_charset: Charset::default(),
            epub_book: None,
        };
        // init {
        //     try {
        //         epubBook?.let {
        //             // if (book.coverUrl.isNullOrEmpty()) {
        //             //     book.coverUrl = FileUtils.getPath(
        //             //         appCtx.externalFiles,
        //             //         "covers",
        //             //         "${MD5Utils.md5Encode16(book.bookUrl)}.jpg"
        //             //     )
        //             // }
        //             // if (!File(book.coverUrl!!).exists()) {
        //             //     /*部分书籍DRM处理后，封面获取异常，待优化*/
        //             //     it.coverImage?.inputStream?.use { input ->
        //             //         val cover = BitmapFactory.decodeStream(input)
        //             //         val out = FileOutputStream(FileUtils.createFileIfNotExist(book.coverUrl!!))
        //             //         cover.compress(Bitmap.CompressFormat.JPEG, 90, out)
        //             //         out.flush()
        //             //         out.close()
        //             //     }
        //             // }
        //         }
        //     } catch (e: Exception) {
        //         e.printStackTrace()
        //     }
        // }
        // fix: Kotlin 懒加载 getter（EpubBook 非 Clone，直接读一次并缓存到字段）
        e.epub_book = e.read_epub();
        return e;
    }

    // companion object {
    //     private var eFile: EpubFile? = None

    //     @Synchronized
    // fix: Kotlin 共享同一 EpubFile 实例（EpubBook 不可 Clone），以 `&'static mut` 返回；
    //      Book 可 Clone，缓存实例时克隆一份，调用方持有的 book 在调用结束后同步回
    fn get_e_file(book: &Book) -> &'static mut EpubFile {
        unsafe {
            if E_FILE.is_none() || E_FILE.as_ref().unwrap().book.book_url != book.book_url {
                E_FILE = Some(EpubFile::new(book.clone()));
                //对于Epub文件默认不启用替换
                // book.setUseReplaceRule(false)
                return E_FILE.as_mut().unwrap();
            }
            E_FILE.as_mut().unwrap().book = book.clone();
            return E_FILE.as_mut().unwrap();
        }
    }

    //     @Synchronized
    pub fn get_chapter_list(book: &mut Book) -> Vec<BookChapter> {
        if book.toc_url.is_empty() {
            book.toc_url = "spin+toc".to_string();
        }
        let toc_url = book.toc_url.clone();
        let mut epub_file = Self::get_e_file(book);
        let result = match toc_url.as_str() {
            "toc" => {
                logger_info("epubFile.getChapterList");
                epub_file.get_chapter_list_inner()
            }
            "spin" => {
                logger_info("epubFile.getChapterListBySpine");
                epub_file.get_chapter_list_by_spine()
            }
            "spin<toc" => {
                logger_info("epubFile.getChapterListBySpinAndToc true");
                epub_file.get_chapter_list_by_spin_and_toc(true)
            }
            "spin+toc" => {
                logger_info("epubFile.getChapterListBySpinAndToc");
                epub_file.get_chapter_list_by_spin_and_toc(false)
            }
            "toc+spin" => {
                logger_info("epubFile.getChapterListByTocAndSpin");
                epub_file.get_chapter_list_by_toc_and_spin(false)
            }
            "toc<spin" => {
                logger_info("epubFile.getChapterListByTocAndSpin true");
                epub_file.get_chapter_list_by_toc_and_spin(true)
            }
            _ => {
                logger_info("epubFile.getChapterListBySpinAndToc");
                epub_file.get_chapter_list_by_spin_and_toc(false)
            }
        };
        // fix: 实例方法改写的是 static 缓存实例中的 book 副本，同步回调用方
        book.clone_from(&epub_file.book);
        return result;
    }

    //     @Synchronized
    pub fn get_content(book: &mut Book, chapter: &BookChapter) -> Option<String> {
        let mut epub_file = Self::get_e_file(book);
        let result = epub_file.get_content_inner(chapter);
        book.clone_from(&epub_file.book);
        return result;
    }

    //     @Synchronized
    pub fn get_image(book: &mut Book, href: String) -> Option<Box<dyn InputStream>> {
        let mut epub_file = Self::get_e_file(book);
        let result = epub_file.get_image_inner(href);
        book.clone_from(&epub_file.book);
        return result;
    }

    //     @Synchronized
    pub fn up_book_info(book: &mut Book, only_cover: bool) {
        let mut epub_file = Self::get_e_file(book);
        if only_cover {
            epub_file.update_cover();
        } else {
            epub_file.up_book_info_inner();
        }
        book.clone_from(&epub_file.book);
    }
    // }

    // private var mCharset: Charset = Charset.defaultCharset()
    // private var epubBook: EpubBook? = None
    //     get() {
    //         if (field != None) {
    //             return field
    //         }
    //         field = readEpub()
    //         return field
    //     }

    /*重写epub文件解析代码，直接读出压缩包文件生成Resources给epublib，这样的好处是可以逐一修改某些文件的格式错误*/
    fn read_epub(&mut self) -> Option<EpubBook> {
        // try {
        let file = self.book.get_local_file();
        //通过懒加载读取epub
        // fix: ZipFile 真实 zip 读取（原占位 todo!() panic）
        return EpubReader::new().read_epub_lazy(ZipFile::new(&file.file_path), "utf-8").ok();
        // } catch (e: Exception) {
        //     e.printStackTrace()
        // }
        // return None
    }

    // fix: 与 companion 静态 get_content(book, chapter) 重名（E0592），实例方法改名
    fn get_content_inner(&self, chapter: &BookChapter) -> Option<String> {
        /**
         * <image width="1038" height="670" xlink:href="..."/>
         * ...titlepage.xhtml
         */
        if chapter.url.contains("titlepage.xhtml") {
            return Some("<img src=\"cover.jpeg\" />".to_string());
        }
        /*获取当前章节文本*/
        if let Some(epub_book) = self.epub_book.as_ref() {
            let next_url = chapter.variable_map().get("nextUrl").cloned();
            let start_fragment_id = chapter.start_fragment_id.clone();
            let end_fragment_id = chapter.end_fragment_id.clone();
            let mut elements = Elements::new();
            let mut is_chapter = false;
            /*一些书籍依靠href索引的resource会包含多个章节，需要依靠fragmentId来截取到当前章节的内容*/
            /*注:这里较大增加了内容加载的时间，所以首次获取内容后可存储到本地cache，减少重复加载*/
            for res in epub_book.get_contents() {
                if StringUtil::substring_before_last(&chapter.url, '#') == *res.get_href() {
                    elements.add(self.get_body(&res, start_fragment_id.clone(), end_fragment_id.clone()));
                    is_chapter = true;
                    /**
                     * fix https://github.com/gedoor/legado/issues/1927 加载全部内容的bug
                     * content src text/000001.html（当前章节）
                     * content src text/000001.html#toc_id_x (下一章节）
                     */
                    if *res.get_href() == next_url.clone().map(|n| StringUtil::substring_before_last(&n, '#')).unwrap_or_default() {
                        break;
                    }
                } else if is_chapter {
                    // fix 最后一章存在多个html时 内容缺失
                    if *res.get_href() == next_url.clone().map(|n| StringUtil::substring_before_last(&n, '#')).unwrap_or_default() {
                        break;
                    }
                    elements.add(self.get_body(&res, start_fragment_id.clone(), end_fragment_id.clone()));
                }
            }
            let mut html = elements.outer_html();
            // fix: Kotlin `Book.rubyTag`（companion const = 4L）
            let tag = 4i64;
            if self.book.get_del_tag(tag) {
                html = Regex::new("<ruby>\\s?([\\u4e00-\\u9fa5])\\s?.*?</ruby>").unwrap().replace_all(&html, "$1").to_string();
            }
            return Some(HtmlFormatter::new().formatKeepImg(Some(html.as_str())));
        }
        return None;
    }

    fn get_body(&self, res: &Resource, start_fragment_id: Option<String>, end_fragment_id: Option<String>) -> Element {
        // fix: 按资源声明的编码解码（原仅 UTF-8 lossy；GBK 等 epub 乱码）
        let data = res.get_data().map(|d| d.as_slice()).unwrap_or(&[]);
        let encoding = res.get_input_encoding().clone();
        let text = if encoding.is_empty() || encoding.eq_ignore_ascii_case("utf-8") {
            String::from_utf8_lossy(data).to_string()
        } else {
            crate::io_legado_app_help_http_okhttputils::decode_bytes_with_charset(data, &encoding)
        };
        let body = Jsoup::parse(text).body();
        if let Some(sid) = start_fragment_id.as_ref() {
            if !sid.is_empty() {
                body.get_element_by_id(&sid)
                    .map(|el| el.previous_element_siblings().remove());
            }
        }
        if let Some(eid) = end_fragment_id.as_ref() {
            if !eid.is_empty() && end_fragment_id != start_fragment_id {
                body.get_element_by_id(eid).map(|mut el| {
                    el.next_element_siblings().remove();
                    el.remove();
                });
            }
        }
        /*选择去除正文中的H标签，部分书籍标题与阅读标题重复待优化*/
        // fix: Kotlin `Book.hTag`（companion const = 2L）
        let tag = 2i64;
        if self.book.get_del_tag(tag) {
            body.get_elements_by_tag("h1").remove();
            body.get_elements_by_tag("h2").remove();
            body.get_elements_by_tag("h3").remove();
            body.get_elements_by_tag("h4").remove();
            body.get_elements_by_tag("h5").remove();
            body.get_elements_by_tag("h6").remove();
            //body.getElementsMatchingOwnText(chapter.title)?.remove()
        }

        let children = body.children();
        children.select("script").remove();
        children.select("style").remove();
        return body;
    }

    // fix: 与 companion 静态 get_image(book, href) 重名（E0592），实例方法改名；
    //      InputStream 是 trait，返回 Box<dyn InputStream>
    fn get_image_inner(&self, href: String) -> Option<Box<dyn InputStream>> {
        let ab_href = href.replace("../", "");
        return self.epub_book.as_ref()
            .and_then(|b| b.get_resources().get_by_href(&ab_href))
            .and_then(|r| r.get_input_stream().ok())
            .map(|s| Box::new(s) as Box<dyn InputStream>);
    }

    // fix: 与 companion 静态 up_book_info(book, only_cover) 重名（E0592），实例方法改名
    fn up_book_info_inner(&mut self) {
        if self.epub_book.is_none() {
            unsafe { E_FILE = None; }
            self.book.intro = Some("书籍导入异常".to_string());
        } else {
            let metadata = self.epub_book.as_ref().unwrap().get_metadata();
            self.book.name = metadata.get_first_title();
            if self.book.name.is_empty() {
                self.book.name = self.book.origin_name.replace(".epub", "");
            }

            if metadata.get_authors().len() > 0 {
                // fix: String::replace 的 pattern 不支持 Regex，改用 Regex::replace_all
                let author = Regex::new("^, |, $").unwrap()
                    .replace_all(&metadata.get_authors()[0].to_string(), "")
                    .to_string();
                self.book.author = author;
            }
            if metadata.get_descriptions().len() > 0 {
                self.book.intro = Some(Jsoup::parse(metadata.get_descriptions()[0].clone()).text());
            }

            self.update_cover();
        }
    }

    pub fn update_cover(&mut self) {
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
            if let Some(cover_data) = self.epub_book.as_ref()
                .and_then(|b| b.get_cover_image().as_ref())
                .and_then(|c| c.get_data().ok())
                .map(|d| d.clone()) {
                write_bytes(&cover_url, &cover_data);
            }
        }
        // 保存 cover
        // val cover = epubBook!!.coverImage?.href
        // if (cover != None) {
        //     val epubRootDir = book.getEpubRootDir()
        //     if (epubRootDir.isEmpty()) {
        //         book.coverUrl = book.bookUrl.replace("storage/data/", "/epub/") + "/index/" + cover
        //     } else {
        //         book.coverUrl = book.bookUrl.replace("storage/data/", "/epub/") + "/index/" + epubRootDir + "/" + cover
        //     }
        // }
    }

    pub fn get_chapter_list_by_spine(&mut self) -> Vec<BookChapter> {
        let mut chapter_list = Vec::new();
        if let Some(spine) = self.epub_book.as_ref().map(|b| b.get_spine()) {
            for (index, spin_resource) in spine.get_spine_references().iter().enumerate() {
                if let Some(resource) = spin_resource.get_resource().as_ref() {
                    let mut title = resource.get_title().clone();
                    if title.is_empty() {
                        // try {
                        let doc = Jsoup::parse(String::from_utf8_lossy(resource.get_data().unwrap_or(&Vec::new())).to_string());
                        let elements = doc.get_elements_by_tag("title");
                        if elements.size() > 0 {
                            title = elements.get(0).text();
                        }
                        // } catch (e: IOException) {
                        //     e.printStackTrace()
                        // }
                    }

                    let mut chapter = BookChapter::default();
                    chapter.index = index as i32;
                    chapter.book_url = self.book.book_url.clone();
                    chapter.url = resource.get_href().clone();
                    if index == 0 && title.is_empty() {
                        chapter.title = "封面".to_string();
                    } else {
                        chapter.title = title;
                    }
                    chapter_list.push(chapter);
                }
            }
        }
        self.book.latest_chapter_title = chapter_list.last().map(|c| c.title.clone());
        self.book.total_chapter_num = chapter_list.len() as i32;
        return chapter_list;
    }

    // fix: 与 companion 静态 get_chapter_list(book) 重名（E0592），实例方法改名
    pub fn get_chapter_list_inner(&mut self) -> Vec<BookChapter> {
        let mut chapter_list = Vec::new();
        if let Some(toc) = self.epub_book.as_ref().map(|b| b.get_table_of_contents()) {
            for (index, resource) in toc.get_all_unique_resources().iter().enumerate() {
                let mut title = resource.get_title().clone();
                if title.is_empty() {
                    // try {
                    let doc = Jsoup::parse(String::from_utf8_lossy(resource.get_data().unwrap_or(&Vec::new())).to_string());
                    let elements = doc.get_elements_by_tag("title");
                    if elements.size() > 0 {
                        title = elements.get(0).text();
                    }
                    // } catch (e: IOException) {
                    //     e.printStackTrace()
                    // }
                }
                let mut chapter = BookChapter::default();
                chapter.index = index as i32;
                chapter.book_url = self.book.book_url.clone();
                chapter.url = resource.get_href().clone();
                if index == 0 && title.is_empty() {
                    chapter.title = "封面".to_string();
                } else {
                    chapter.title = title;
                }
                chapter_list.push(chapter);
            }
        }
        self.book.latest_chapter_title = chapter_list.last().map(|c| c.title.clone());
        self.book.total_chapter_num = chapter_list.len() as i32;
        return chapter_list;
    }

    pub fn get_chapter_list_by_spin_and_toc(&mut self, use_toc_title: bool) -> Vec<BookChapter> {
        // 如果读取了 toc，那么 spin 就会使用 toc 的章节名
        let mut toc_chapter_list = self.get_chapter_list_inner();
        let mut spin_chapter_list = self.get_chapter_list_by_spine();

        if spin_chapter_list.len() == 0 {
            return toc_chapter_list;
        }

        if toc_chapter_list.len() == 0 {
            return spin_chapter_list;
        }

        let mut title_map: std::collections::HashMap<String, BookChapter> = std::collections::HashMap::new();

        // fix: BookChapter 非 Clone，用 mem::take 移出（列表仅作去重/取标题用）
        for i in 0..toc_chapter_list.len() {
            title_map.insert(toc_chapter_list[i].url.clone(), std::mem::take(&mut toc_chapter_list[i]));
        }

        for i in 0..spin_chapter_list.len() {
            let chapter = &mut spin_chapter_list[i];
            if let Some(tc) = title_map.get(&chapter.url) {
                if !tc.title.is_empty() {
                    if use_toc_title || chapter.title.is_empty() {
                        chapter.title = tc.title.clone();
                    }
                }
            }
        }

        self.book.latest_chapter_title = spin_chapter_list.last().map(|c| c.title.clone());
        self.book.total_chapter_num = spin_chapter_list.len() as i32;
        return spin_chapter_list;
    }

    pub fn get_chapter_list_by_toc_and_spin(&mut self, use_spin_title: bool) -> Vec<BookChapter> {
        // 如果读取了 toc，那么 spin 就会使用 toc 的章节名
        let mut toc_chapter_list = self.get_chapter_list_inner();
        let mut spin_chapter_list = self.get_chapter_list_by_spine();

        if toc_chapter_list.len() == 0 {
            return spin_chapter_list;
        }

        if spin_chapter_list.len() == 0 {
            return toc_chapter_list;
        }

        let mut title_map: std::collections::HashMap<String, BookChapter> = std::collections::HashMap::new();

        // fix: BookChapter 非 Clone，用 mem::take 移出（列表仅作去重/取标题用）
        for i in 0..spin_chapter_list.len() {
            title_map.insert(spin_chapter_list[i].url.clone(), std::mem::take(&mut spin_chapter_list[i]));
        }

        for i in 0..toc_chapter_list.len() {
            let chapter = &mut toc_chapter_list[i];
            if let Some(tc) = title_map.get(&chapter.url) {
                if !tc.title.is_empty() {
                    if use_spin_title || chapter.title.is_empty() {
                        chapter.title = tc.title.clone();
                    }
                }
            }
        }

        self.book.latest_chapter_title = toc_chapter_list.last().map(|c| c.title.clone());
        self.book.total_chapter_num = toc_chapter_list.len() as i32;
        return toc_chapter_list;
    }
}

// companion object 中的 eFile 静态字段
pub static mut E_FILE: Option<EpubFile> = None;
