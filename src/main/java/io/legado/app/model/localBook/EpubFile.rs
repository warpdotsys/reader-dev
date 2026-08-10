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

fn logger_info(msg: &str) {
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
        e.epub_book = e.get_epub_book();
        return e;
    }

    // companion object {
    //     private var eFile: EpubFile? = null

    //     @Synchronized
    fn get_e_file(book: Book) -> EpubFile {
        if E_FILE.is_none() || E_FILE.as_ref().unwrap().book.book_url != book.book_url {
            E_FILE = Some(EpubFile::new(book));
            //对于Epub文件默认不启用替换
            // book.setUseReplaceRule(false)
            return E_FILE.as_ref().unwrap().clone();
        }
        E_FILE.as_mut().unwrap().book = book;
        return E_FILE.as_ref().unwrap().clone();
    }

    //     @Synchronized
    pub fn get_chapter_list(book: Book) -> Vec<BookChapter> {
        if book.toc_url.is_empty() {
            book.toc_url = "spin+toc".to_string();
        }
        let epub_file = Self::get_e_file(book);
        return match book.toc_url.as_str() {
            "toc" => {
                logger_info("epubFile.getChapterList");
                epub_file.get_chapter_list()
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
    }

    //     @Synchronized
    pub fn get_content(book: Book, chapter: BookChapter) -> Option<String> {
        return Self::get_e_file(book).get_content(chapter);
    }

    //     @Synchronized
    pub fn get_image(book: Book, href: String) -> Option<InputStream> {
        return Self::get_e_file(book).get_image(href);
    }

    //     @Synchronized
    pub fn up_book_info(book: Book, only_cover: bool) {
        if only_cover {
            return Self::get_e_file(book).update_cover();
        }
        return Self::get_e_file(book).up_book_info();
    }
    // }

    // private var mCharset: Charset = Charset.defaultCharset()
    // private var epubBook: EpubBook? = null
    //     get() {
    //         if (field != null) {
    //             return field
    //         }
    //         field = readEpub()
    //         return field
    //     }

    fn get_epub_book(&mut self) -> Option<EpubBook> {
        if self.epub_book.is_some() {
            return self.epub_book.clone();
        }
        self.epub_book = self.read_epub();
        return self.epub_book.clone();
    }

    /*重写epub文件解析代码，直接读出压缩包文件生成Resources给epublib，这样的好处是可以逐一修改某些文件的格式错误*/
    fn read_epub(&self) -> Option<EpubBook> {
        // try {
        let file = self.book.get_local_file();
        //通过懒加载读取epub
        return EpubReader::new().read_epub_lazy(ZipFile::new(&file), "utf-8");
        // } catch (e: Exception) {
        //     e.printStackTrace()
        // }
        // return null
    }

    fn get_content(&self, chapter: BookChapter) -> Option<String> {
        /**
         * <image width="1038" height="670" xlink:href="..."/>
         * ...titlepage.xhtml
         */
        if chapter.url.contains("titlepage.xhtml") {
            return Some("<img src=\"cover.jpeg\" />".to_string());
        }
        /*获取当前章节文本*/
        if let Some(epub_book) = self.epub_book.clone() {
            let next_url = chapter.get_variable("nextUrl");
            let start_fragment_id = chapter.start_fragment_id.clone();
            let end_fragment_id = chapter.end_fragment_id.clone();
            let mut elements = Elements::new();
            let mut is_chapter = false;
            /*一些书籍依靠href索引的resource会包含多个章节，需要依靠fragmentId来截取到当前章节的内容*/
            /*注:这里较大增加了内容加载的时间，所以首次获取内容后可存储到本地cache，减少重复加载*/
            for res in epub_book.contents {
                if chapter.url.substring_before_last("#") == res.href {
                    elements.add(get_body(&res, start_fragment_id.clone(), end_fragment_id.clone()));
                    is_chapter = true;
                    /**
                     * fix https://github.com/gedoor/legado/issues/1927 加载全部内容的bug
                     * content src text/000001.html（当前章节）
                     * content src text/000001.html#toc_id_x (下一章节）
                     */
                    if res.href == next_url.clone().map(|n| n.substring_before_last("#")) {
                        break;
                    }
                } else if is_chapter {
                    // fix 最后一章存在多个html时 内容缺失
                    if res.href == next_url.clone().map(|n| n.substring_before_last("#")) {
                        break;
                    }
                    elements.add(get_body(&res, start_fragment_id.clone(), end_fragment_id.clone()));
                }
            }
            let mut html = elements.outer_html();
            let tag = Book::ruby_tag();
            if self.book.get_del_tag(tag) {
                html = Regex::new("<ruby>\\s?([\\u4e00-\\u9fa5])\\s?.*?</ruby>").unwrap().replace_all(&html, "$1").to_string();
            }
            return html_formatter_format_keep_img(&html);
        }
        return None;
    }

    fn get_body(res: &Resource, start_fragment_id: Option<String>, end_fragment_id: Option<String>) -> Element {
        let body = Jsoup::parse(String::from_utf8_lossy(&res.data).to_string(), self.m_charset).body();
        if let Some(sid) = start_fragment_id {
            if !sid.is_empty() {
                body.get_element_by_id(&sid)
                    .map(|el| el.previous_element_siblings().remove());
            }
        }
        if let Some(eid) = end_fragment_id {
            if !eid.is_empty() && end_fragment_id != start_fragment_id {
                body.get_element_by_id(&eid).map(|el| {
                    el.next_element_siblings().remove();
                    el.remove();
                });
            }
        }
        /*选择去除正文中的H标签，部分书籍标题与阅读标题重复待优化*/
        let tag = Book::h_tag();
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

    fn get_image(&self, href: String) -> Option<InputStream> {
        let ab_href = href.replace("../", "");
        return self.epub_book.clone()
            .and_then(|b| b.resources.get_by_href(&ab_href))
            .map(|r| r.input_stream);
    }

    fn up_book_info(&mut self) {
        if self.epub_book.is_none() {
            E_FILE = None;
            self.book.intro = "书籍导入异常".to_string();
        } else {
            let metadata = self.epub_book.as_ref().unwrap().metadata;
            self.book.name = metadata.first_title.clone();
            if self.book.name.is_empty() {
                self.book.name = self.book.origin_name.replace(".epub", "");
            }

            if metadata.authors.len() > 0 {
                let author = metadata.authors[0].to_string()
                    .replace(&Regex::new("^, |, $").unwrap(), "");
                self.book.author = author;
            }
            if metadata.descriptions.len() > 0 {
                self.book.intro = Jsoup::parse(&metadata.descriptions[0]).text();
            }

            self.update_cover();
        }
    }

    pub fn update_cover(&mut self) {
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
            if let Some(cover_data) = self.epub_book.as_ref().and_then(|b| b.cover_image.clone()).map(|c| c.data) {
                write_bytes(&cover_url, &cover_data);
            }
        }
        // 保存 cover
        // val cover = epubBook!!.coverImage?.href
        // if (cover != null) {
        //     val epubRootDir = book.getEpubRootDir()
        //     if (epubRootDir.isEmpty()) {
        //         book.coverUrl = book.bookUrl.replace("storage/data/", "/epub/") + "/index/" + cover
        //     } else {
        //         book.coverUrl = book.bookUrl.replace("storage/data/", "/epub/") + "/index/" + epubRootDir + "/" + cover
        //     }
        // }
    }

    pub fn get_chapter_list_by_spine(&self) -> Vec<BookChapter> {
        let mut chapter_list = Vec::new();
        if let Some(spine) = self.epub_book.as_ref().and_then(|b| b.spine.clone()) {
            for (index, spin_resource) in spine.spine_references.iter().enumerate() {
                let resource = spin_resource.resource.clone();
                let mut title = resource.title.clone();
                if title.is_none() || title.clone().unwrap().is_empty() {
                    // try {
                    let doc = Jsoup::parse(String::from_utf8_lossy(&resource.data).to_string());
                    let elements = doc.get_elements_by_tag("title");
                    if elements.size() > 0 {
                        title = Some(elements[0].text());
                    }
                    // } catch (e: IOException) {
                    //     e.printStackTrace()
                    // }
                }

                let mut chapter = BookChapter::new();
                chapter.index = index;
                chapter.book_url = self.book.book_url.clone();
                chapter.url = resource.href.clone();
                if index == 0 && (title.is_none() || title.clone().unwrap().is_empty()) {
                    chapter.title = "封面".to_string();
                } else {
                    chapter.title = title.unwrap_or_default();
                }
                chapter_list.push(chapter);
            }
        }
        self.book.latest_chapter_title = chapter_list.last().map(|c| c.title.clone());
        self.book.total_chapter_num = chapter_list.len();
        return chapter_list;
    }

    pub fn get_chapter_list(&self) -> Vec<BookChapter> {
        let mut chapter_list = Vec::new();
        if let Some(toc) = self.epub_book.as_ref().and_then(|b| b.table_of_contents.clone()) {
            for (index, resource) in toc.all_unique_resources.iter().enumerate() {
                let mut title = resource.title.clone();
                if title.is_none() || title.clone().unwrap().is_empty() {
                    // try {
                    let doc = Jsoup::parse(String::from_utf8_lossy(&resource.data).to_string());
                    let elements = doc.get_elements_by_tag("title");
                    if elements.size() > 0 {
                        title = Some(elements[0].text());
                    }
                    // } catch (e: IOException) {
                    //     e.printStackTrace()
                    // }
                }
                let mut chapter = BookChapter::new();
                chapter.index = index;
                chapter.book_url = self.book.book_url.clone();
                chapter.url = resource.href.clone();
                if index == 0 && (title.is_none() || title.clone().unwrap().is_empty()) {
                    chapter.title = "封面".to_string();
                } else {
                    chapter.title = title.unwrap_or_default();
                }
                chapter_list.push(chapter);
            }
        }
        self.book.latest_chapter_title = chapter_list.last().map(|c| c.title.clone());
        self.book.total_chapter_num = chapter_list.len();
        return chapter_list;
    }

    pub fn get_chapter_list_by_spin_and_toc(&self, use_toc_title: bool) -> Vec<BookChapter> {
        // 如果读取了 toc，那么 spin 就会使用 toc 的章节名
        let mut toc_chapter_list = self.get_chapter_list();
        let mut spin_chapter_list = self.get_chapter_list_by_spine();

        if spin_chapter_list.len() == 0 {
            return toc_chapter_list;
        }

        if toc_chapter_list.len() == 0 {
            return spin_chapter_list;
        }

        let mut title_map: std::collections::HashMap<String, BookChapter> = std::collections::HashMap::new();

        for i in 0..toc_chapter_list.len() {
            title_map.insert(toc_chapter_list[i].url.clone(), toc_chapter_list[i].clone());
        }

        for i in 0..spin_chapter_list.len() {
            let chapter = &mut spin_chapter_list[i];
            let toc_chapter = title_map.get(&chapter.url).cloned();
            if let Some(tc) = toc_chapter {
                if !tc.title.is_empty() {
                    if use_toc_title || chapter.title.is_empty() {
                        chapter.title = tc.title.clone();
                    }
                }
            }
        }

        self.book.latest_chapter_title = spin_chapter_list.last().map(|c| c.title.clone());
        self.book.total_chapter_num = spin_chapter_list.len();
        return spin_chapter_list;
    }

    pub fn get_chapter_list_by_toc_and_spin(&self, use_spin_title: bool) -> Vec<BookChapter> {
        // 如果读取了 toc，那么 spin 就会使用 toc 的章节名
        let mut toc_chapter_list = self.get_chapter_list();
        let mut spin_chapter_list = self.get_chapter_list_by_spine();

        if toc_chapter_list.len() == 0 {
            return spin_chapter_list;
        }

        if spin_chapter_list.len() == 0 {
            return toc_chapter_list;
        }

        let mut title_map: std::collections::HashMap<String, BookChapter> = std::collections::HashMap::new();

        for i in 0..spin_chapter_list.len() {
            title_map.insert(spin_chapter_list[i].url.clone(), spin_chapter_list[i].clone());
        }

        for i in 0..toc_chapter_list.len() {
            let chapter = &mut toc_chapter_list[i];
            let toc_chapter = title_map.get(&chapter.url).cloned();
            if let Some(tc) = toc_chapter {
                if !tc.title.is_empty() {
                    if use_spin_title || chapter.title.is_empty() {
                        chapter.title = tc.title.clone();
                    }
                }
            }
        }

        self.book.latest_chapter_title = toc_chapter_list.last().map(|c| c.title.clone());
        self.book.total_chapter_num = toc_chapter_list.len();
        return toc_chapter_list;
    }
}

// companion object 中的 eFile 静态字段
static mut E_FILE: Option<EpubFile> = None;
