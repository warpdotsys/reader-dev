use crate::prelude::*;
// fix: 显式导入以覆盖 prelude 中多个 glob 重导出导致的同名歧义
use crate::io_legado_app_data_entities_booklogger::LOGGER;
use crate::stubs::{Charset, File, FileUtils, GSON};
use std::cell::RefMut;
// package io.legado.app.data.entities

// import io.legado.app.constant.BookType
// import io.legado.app.constant.AppPattern
// import io.legado.app.utils.GSON
// import io.legado.app.utils.fromJsonObject
// import io.legado.app.utils.MD5Utils
// import io.legado.app.utils.FileUtils
// import io.legado.app.model.localBook.LocalBook
// import io.legado.app.model.localBook.EpubFile
// import io.legado.app.model.localBook.UmdFile
// import io.legado.app.model.localBook.CbzFile
// import java.nio.charset.Charset
// import java.io.File
// import kotlin.math.max
// import kotlin.math.min
// import org.jsoup.Jsoup
// import com.fasterxml.jackson.annotation.JsonIgnoreProperties
// import com.fasterxml.jackson.annotation.JsonProperty

// @JsonIgnoreProperties("variableMap", "infoHtml", "tocHtml", "config", "rootDir", "localBook", "epub", "epubRootDir", "onLineTxt", "localTxt", "umd", "realAuthor", "unreadChapterNum", "folderName", "pdfImageWidth", "localFile", "kindList", "_userNameSpace", "bookDir", "userNameSpace")
// fix: Clone 已在 stubs.rs 手工实现（Book/ReadConfig），此处不再 derive
pub struct Book {
    pub book_url: String,                   // 详情页Url(本地书源存储完整文件路径)
    pub toc_url: String,                    // 目录页Url (toc=table of Contents)
    pub origin: String,                     // 书源URL(默认BookType.local)
    pub origin_name: String,                //书源名称
    pub name: String,                       // 书籍名称(书源获取)
    pub author: String,                     // 作者名称(书源获取)
    pub kind: Option<String>,               // 分类信息(书源获取)
    pub custom_tag: Option<String>,         // 分类信息(用户修改)
    pub cover_url: Option<String>,          // 封面Url(书源获取)
    pub custom_cover_url: Option<String>,   // 封面Url(用户修改)
    pub intro: Option<String>,              // 简介内容(书源获取)
    pub custom_intro: Option<String>,       // 简介内容(用户修改)
    pub charset: Option<String>,            // 自定义字符集名称(仅适用于本地书籍)
    pub r#type: i32,                        // @BookType
    pub group: i64,                         // 自定义分组索引号
    pub latest_chapter_title: Option<String>, // 最新章节标题
    pub latest_chapter_time: i64,           // 最新章节标题更新时间
    pub last_check_time: i64,               // 最近一次更新书籍信息的时间
    pub last_check_count: i32,              // 最近一次发现新章节的数量
    pub total_chapter_num: i32,             // 书籍目录总数
    pub dur_chapter_title: Option<String>,  // 当前章节名称
    pub dur_chapter_index: i32,             // 当前章节索引
    pub dur_chapter_pos: i32,               // 当前阅读的进度(首行字符的索引位置)
    pub dur_chapter_time: i64,              // 最近一次阅读书籍的时间(打开正文的时间)
    pub word_count: Option<String>,
    pub can_update: bool,                   // 刷新书架时更新书籍信息
    pub order: i32,                         // 手动排序
    pub origin_order: i32,                  //书源排序
    pub use_replace_rule: bool,             // 正文使用净化替换规则
    pub variable: Option<String>,           // 自定义书籍变量信息(用于书源规则检索书籍信息)
    pub read_config: RefCell<Option<ReadConfig>>,
    // @get:JsonProperty("isInShelf")
    pub is_in_shelf: bool,
    pub last_check_error: Option<String>,

    // override var infoHtml: String? = None
    pub info_html: Option<String>,
    // override var tocHtml: String? = None
    pub toc_html: Option<String>,

    // @Transient
    // private var rootDir: String = ""
    pub root_dir: String,

    // @Transient
    // private var _userNameSpace: String = ""
    pub user_name_space: String,

    // @delegate:Transient
    // override val variableMap: HashMap<String, String> by lazy { ... }
    pub variable_map_cache: RefCell<Option<HashMap<String, String>>>,
}

impl Book {
    pub fn is_local_book(&self) -> bool {
        self.origin == BookType::local
    }

    pub fn is_local_txt(&self) -> bool {
        self.is_local_book() && self.origin_name.to_lowercase().ends_with(".txt")
    }

    pub fn is_local_epub(&self) -> bool {
        self.is_local_book() && self.origin_name.to_lowercase().ends_with(".epub")
    }

    pub fn is_local_pdf(&self) -> bool {
        self.is_local_book() && self.is_pdf()
    }

    pub fn is_epub(&self) -> bool {
        self.origin_name.to_lowercase().ends_with(".epub")
    }

    pub fn is_cbz(&self) -> bool {
        self.origin_name.to_lowercase().ends_with(".cbz")
    }

    pub fn is_umd(&self) -> bool {
        self.origin_name.to_lowercase().ends_with(".umd")
    }

    pub fn is_pdf(&self) -> bool {
        self.origin_name.to_lowercase().ends_with(".pdf")
    }

    pub fn is_on_line_txt(&self) -> bool {
        !self.is_local_book() && self.r#type == 0
    }

    // override val variableMap: HashMap<String, String> by lazy {
    //     GSON.fromJsonObject<HashMap<String, String>>(variable).getOrNull() ?: hashMapOf()
    // }
    pub fn variable_map(&self) -> HashMap<String, String> {
        if let Some(cached) = self.variable_map_cache.borrow().as_ref() {
            return cached.clone();
        }
        let map = GSON::from_json_object::<HashMap<String, String>>(
            self.variable.clone().unwrap_or_default(),
        )
        .get_or_null()
        .unwrap_or_else(HashMap::new);
        *self.variable_map_cache.borrow_mut() = Some(map.clone());
        map
    }

    pub fn put_variable(&mut self, key: String, value: Option<String>) {
        let mut map = self.variable_map();
        if let Some(v) = value {
            map.insert(key, v);
        } else {
            map.remove(&key);
        }
        *self.variable_map_cache.borrow_mut() = Some(map.clone());
        self.variable = Some(GSON::to_json(map));
    }

    pub fn get_real_author(&self) -> String {
        AppPattern::authorRegex().replace_all(&self.author, "").to_string()
    }

    pub fn get_unread_chapter_num(&self) -> i32 {
        (self.total_chapter_num - self.dur_chapter_index - 1).max(0)
    }

    pub fn get_display_cover(&self) -> Option<String> {
        if self.custom_cover_url.is_null_or_empty() { self.cover_url.clone() } else { self.custom_cover_url.clone() }
    }

    pub fn get_display_intro(&self) -> Option<String> {
        if self.custom_intro.is_null_or_empty() { self.intro.clone() } else { self.custom_intro.clone() }
    }

    pub fn file_charset(&self) -> Charset {
        charset(self.charset.as_deref().unwrap_or("UTF-8"))
    }

    #[allow(elided_lifetimes_in_paths)]
    // private fun config(): ReadConfig
    fn config(&self) -> RefMut<'_, ReadConfig> {
        if self.read_config.borrow().is_none() {
            *self.read_config.borrow_mut() = Some(ReadConfig::default());
        }
        RefMut::map(self.read_config.borrow_mut(), |c| c.as_mut().unwrap())
    }

    pub fn set_del_tag(&self, tag: i64) {
        let mut config = self.config();
        config.del_tag =
            if (config.del_tag & tag) == tag { config.del_tag & !tag } else { config.del_tag | tag };
    }

    pub fn get_del_tag(&self, tag: i64) -> bool {
        (self.config().del_tag & tag) == tag
    }

    pub fn get_pdf_image_width(&self) -> f32 {
        self.config().pdf_image_width
    }

    pub fn set_pdf_image_width(&self, value: f32) {
        self.config().pdf_image_width = value;
    }

    pub fn get_folder_name(&self) -> String {
        //防止书名过长,只取9位
        let mut folder_name = AppPattern::fileNameRegex().replace_all(&self.name, "").to_string();
        folder_name = folder_name.chars().take(folder_name.len().min(9)).collect::<String>();
        folder_name + &MD5Utils::md5Encode16(&self.book_url)
    }

    pub fn set_root_dir(&mut self, root: String) {
        if !root.is_empty() && !root.ends_with(&File::separator()) {
            self.root_dir = root + &File::separator();
        } else {
            self.root_dir = root;
        }
    }

    pub fn get_local_file(&mut self) -> File {
        if self.origin_name.starts_with(&self.root_dir) {
            self.origin_name = self.origin_name.replacen(&self.root_dir, "", 1);
        }
        LOGGER.info(format!("getLocalFile rootDir: {} originName: {}", self.root_dir, self.origin_name));
        if self.is_epub() && self.origin_name.find("localStore").is_none() && self.origin_name.find("webdav").is_none() {
            // 非本地/webdav书仓的 epub文件
            return FileUtils::get_file(File::new(&(self.root_dir.clone() + &self.origin_name)), "index.epub");
        }
        if self.is_cbz() && self.origin_name.find("localStore").is_none() && self.origin_name.find("webdav").is_none() {
            // 非本地/webdav书仓的 cbz文件
            return FileUtils::get_file(File::new(&(self.root_dir.clone() + &self.origin_name)), "index.cbz");
        }
        if self.is_pdf() && self.origin_name.find("localStore").is_none() && self.origin_name.find("webdav").is_none() {
            return FileUtils::get_file(File::new(&(self.root_dir.clone() + &self.origin_name)), "index.pdf");
        }
        File::new(&(self.root_dir.clone() + &self.origin_name))
    }

    pub fn set_user_name_space(&mut self, name_space: String) {
        self.user_name_space = name_space;
    }

    pub fn get_user_name_space(&self) -> String {
        self.user_name_space.clone()
    }

    pub fn get_book_dir(&self) -> String {
        FileUtils::get_path(File::new(&self.root_dir), "storage", "data", &self.user_name_space, &(self.name.clone() + "_" + &self.author))
    }

    pub fn get_split_long_chapter(&self) -> bool {
        false
    }

    pub fn to_search_book(&mut self) -> SearchBook {
        let mut book = SearchBook {
            name: self.name.clone(),
            author: self.author.clone(),
            kind: self.kind.clone(),
            book_url: self.book_url.clone(),
            origin: self.origin.clone(),
            origin_name: self.origin_name.clone(),
            r#type: self.r#type,
            word_count: self.word_count.clone(),
            latest_chapter_title: self.latest_chapter_title.clone(),
            cover_url: self.cover_url.clone(),
            intro: self.intro.clone(),
            toc_url: self.toc_url.clone(),
            //                originOrder = originOrder,
            variable: self.variable.clone(),
            ..SearchBook::default()
        };
        book.info_html = self.info_html.clone();
        book.toc_html = self.toc_html.clone();
        book.set_user_name_space(self.get_user_name_space());
        book
    }

    pub fn get_epub_root_dir(&self) -> String {
        // 根据 content.opf 位置来确认root目录
        // var contentOPF = "OEBPS/content.opf"

        let default_path = "OEBPS".to_string();

        // 根据 META-INF/container.xml 来获取 contentOPF 位置
        let container_res = File::new(&(self.book_url.clone() + &File::separator() + "index" + &File::separator() + "META-INF" + &File::separator() + "container.xml"));
        if container_res.exists() {
            let result = (|| -> Option<String> {
                let document = Jsoup::parse(container_res.read_text());
                let root_file_element = document
                    .get_elements_by_tag("rootfiles").get(0)
                    .get_elements_by_tag("rootfile").get(0);
                let result = root_file_element.attr("full-path");
                println!("result: {}", result);
                if !result.is_empty() {
                    return File::new(&result).parent_file().map(|it| it.to_string());
                }
                None
            })();
            if result.is_some() {
                return result.unwrap();
            }
        }

        // 返回默认位置
        default_path
    }

    // only_cover: Boolean = false
    pub fn update_from_local(&mut self, only_cover: bool) {
        if self.is_epub() {
            EpubFile::up_book_info(self, only_cover);
        } else if self.is_umd() {
            UmdFile::up_book_info(self, only_cover);
        } else if self.is_cbz() {
            CbzFile::up_book_info(self, only_cover);
        }
    }

    pub fn work_root(&self) -> String {
        self.root_dir.clone()
    }
}

// companion object {
//     const val hTag = 2L
//     const val rubyTag = 4L
//     const val imgTag = 8L
//     const val imgStyleDefault = "DEFAULT"
//     const val imgStyleFull = "FULL"
//     const val imgStyleText = "TEXT"
// }
impl Book {
    pub const H_TAG: i64 = 2_i64;
    pub const RUBY_TAG: i64 = 4_i64;
    pub const IMG_TAG: i64 = 8_i64;
    pub const IMG_STYLE_DEFAULT: &'static str = "DEFAULT";
    pub const IMG_STYLE_FULL: &'static str = "FULL";
    pub const IMG_STYLE_TEXT: &'static str = "TEXT";

    // root_dir: String = ""
    pub fn init_local_book(book_url: String, local_path: String, root_dir: String) -> Book {
        let file_name = File::new(&local_path).name();
        let name_author = LocalBook::analyze_name_author(&file_name);
        let mut book = Book {
            book_url,
            toc_url: String::new(),
            origin: BookType::local.to_string(),
            origin_name: local_path,
            name: name_author.0,
            author: name_author.1,
            ..Book::default()
        };
        book.can_update = false;
        book.set_root_dir(root_dir);
        book.update_from_local(false);
        book
    }
}

// data class ReadConfig(
//     var reverseToc: Boolean = false,
//     var pageAnim: Int = -1,
//     var reSegment: Boolean = false,
//     var imageStyle: String? = None,
//     var useReplaceRule: Boolean = false,   // 正文使用净化替换规则
//     var delTag: Long = 0L,   //去除标签
//     var pdfImageWidth: Float = 800f
// )
// fix: 补 serde derive——Converters 经 GSON::to_json/from_json_object 序列化 ReadConfig
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ReadConfig {
    pub reverse_toc: bool,
    pub page_anim: i32,
    pub re_segment: bool,
    pub image_style: Option<String>,
    pub use_replace_rule: bool,   // 正文使用净化替换规则
    pub del_tag: i64,             //去除标签
    pub pdf_image_width: f32,
}

impl Default for ReadConfig {
    fn default() -> Self {
        ReadConfig {
            reverse_toc: false,
            page_anim: -1,
            re_segment: false,
            image_style: None,
            use_replace_rule: false,
            del_tag: 0,
            pdf_image_width: 800.0,
        }
    }
}

impl PartialEq for Book {
    // override fun equals(other: Any?): Boolean {
    //     if (other is Book) {
    //         return other.bookUrl == bookUrl
    //     }
    //     return false
    // }
    fn eq(&self, other: &Self) -> bool {
        other.book_url == self.book_url
    }
}

impl Eq for Book {}

impl std::hash::Hash for Book {
    // override fun hashCode(): Int {
    //     return bookUrl.hashCode()
    // }
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.book_url.hash(state);
    }
}

impl Default for Book {
    fn default() -> Self {
        Book {
            book_url: String::new(),
            toc_url: String::new(),
            origin: BookType::local.to_string(),
            origin_name: String::new(),
            name: String::new(),
            author: String::new(),
            kind: None,
            custom_tag: None,
            cover_url: None,
            custom_cover_url: None,
            intro: None,
            custom_intro: None,
            charset: None,
            r#type: 0,
            group: 0,
            latest_chapter_title: None,
            latest_chapter_time: System::current_time_millis(),
            last_check_time: System::current_time_millis(),
            last_check_count: 0,
            total_chapter_num: 0,
            dur_chapter_title: None,
            dur_chapter_index: 0,
            dur_chapter_pos: 0,
            dur_chapter_time: System::current_time_millis(),
            word_count: None,
            can_update: true,
            order: 0,
            origin_order: 0,
            use_replace_rule: true,
            variable: None,
            read_config: RefCell::new(None),
            is_in_shelf: false,
            last_check_error: None,
            info_html: None,
            toc_html: None,
            root_dir: String::new(),
            user_name_space: String::new(),
            variable_map_cache: RefCell::new(None),
        }
    }
}

// class Converters {
//     fun readConfigToString(config: ReadConfig?): String = GSON.toJson(config)
//     fun stringToReadConfig(json: String?) = GSON.fromJsonObject<ReadConfig>(json)
// }
pub struct Converters;

impl Converters {
    pub fn read_config_to_string(config: Option<ReadConfig>) -> String {
        GSON::to_json(config)
    }

    pub fn string_to_read_config(json: Option<String>) -> Option<ReadConfig> {
        GSON::from_json_object::<ReadConfig>(json.unwrap_or_default()).get_or_null()
    }
}

// 手写 Deserialize（GSON 语义：缺失字段用默认值；RefCell 内部字段忽略）
impl<'de> serde::Deserialize<'de> for Book {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = serde_json::Value::deserialize(deserializer)?;
        let gs = |k: &str| v.get(k).and_then(|x| x.as_str()).map(|s| s.to_string());
        let gi = |k: &str| v.get(k).and_then(|x| x.as_i64()).map(|i| i as i32).unwrap_or(0);
        let gl = |k: &str| v.get(k).and_then(|x| x.as_i64()).unwrap_or(0);
        let gb = |k: &str| v.get(k).and_then(|x| x.as_bool()).unwrap_or(false);
        Ok(Book {
            book_url: gs("bookUrl").unwrap_or_default(),
            toc_url: gs("tocUrl").unwrap_or_default(),
            origin: gs("origin").unwrap_or_else(|| BookType::local.to_string()),
            origin_name: gs("originName").unwrap_or_default(),
            name: gs("name").unwrap_or_default(),
            author: gs("author").unwrap_or_default(),
            r#type: gi("bookSourceType"),
            kind: gs("kind"),
            custom_tag: gs("customTag"),
            cover_url: gs("coverUrl"),
            custom_cover_url: gs("customCoverUrl"),
            intro: gs("intro"),
            custom_intro: gs("customIntro"),
            charset: gs("charset"),
            group: gl("group"),
            latest_chapter_title: gs("latestChapterTitle"),
            latest_chapter_time: gl("latestChapterTime"),
            last_check_time: gl("lastCheckTime"),
            last_check_count: gi("lastCheckCount"),
            total_chapter_num: gi("totalChapterNum"),
            dur_chapter_title: gs("durChapterTitle"),
            dur_chapter_index: gi("durChapterIndex"),
            dur_chapter_pos: gi("durChapterPos"),
            dur_chapter_time: gl("durChapterTime"),
            word_count: gs("wordCount"),
            can_update: v.get("canUpdate").and_then(|x| x.as_bool()).unwrap_or(true),
            order: gi("order"),
            origin_order: gi("originOrder"),
            use_replace_rule: gb("useReplaceRule"),
            variable: gs("variable"),
            read_config: std::cell::RefCell::new(None),
            is_in_shelf: gb("isInShelf"),
            last_check_error: gs("lastCheckError"),
            info_html: gs("infoHtml"),
            toc_html: gs("tocHtml"),
            root_dir: gs("rootDir").unwrap_or_default(),
            user_name_space: gs("userNameSpace").unwrap_or_default(),
            variable_map_cache: std::cell::RefCell::new(None),
        })
    }
}