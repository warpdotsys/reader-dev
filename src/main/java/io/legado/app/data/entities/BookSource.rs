// package io.legado.app.data.entities

//import io.legado.app.App
// import io.legado.app.constant.AppConst
// import io.legado.app.constant.AppConst.userAgent
// import io.legado.app.data.entities.rule.*
// import io.legado.app.help.JsExtensions
// import io.legado.app.help.http.CookieStore
// import io.legado.app.help.CacheManager
// import io.legado.app.utils.GSON
// import io.legado.app.utils.fromJsonObject
//import io.legado.app.utils.getPrefString
// import io.legado.app.help.SourceAnalyzer
// import java.io.InputStream
// import java.util.*
// import javax.script.SimpleBindings
// import com.fasterxml.jackson.annotation.JsonIgnoreProperties

//@Parcelize
//@Entity(
//    tableName = "book_sources",
//    indices = [(Index(value = ["bookSourceUrl"], unique = false))]
//)
// @JsonIgnoreProperties(
//     "headerMap", "source", "_userNameSpace", "userNameSpace",
//     "loginHeader", "loginHeaderMap", "loginInfo", "loginInfoMap"
// )
pub struct BookSource {
    pub book_source_url: String,         // 地址，包括 http/https
    pub book_source_name: String,        // 名称
    pub book_source_group: Option<String>, // 分组
    //    @PrimaryKey
    pub book_source_type: i32,           // 类型，0 文本，1 音频
    pub book_url_pattern: Option<String>, //详情页url正则
    pub custom_order: i32,               // 手动排序编号
    pub enabled: bool,                   // 是否启用
    pub enabled_explore: bool,           //启用发现
    pub enabled_cookie_jar: Option<bool>,
    pub concurrent_rate: Option<String>, //并发率
    pub header: Option<String>,
    pub login_url: Option<String>,       // 登录地址
    pub login_ui: Option<String>,
    pub login_check_js: Option<String>,  // 登录检测js
    pub book_source_comment: Option<String>, // 注释
    pub variable_comment: Option<String>,
    pub last_update_time: i64,           // 最后更新时间，用于排序
    pub respond_time: i64,               // 响应时间，用于排序
    pub weight: i32,                     // 智能排序的权重
    pub explore_url: Option<String>,     // 发现url
    pub rule_explore: Option<ExploreRule>, // 发现规则
    pub search_url: Option<String>,      // 搜索url
    pub rule_search: Option<SearchRule>, // 搜索规则
    pub rule_book_info: Option<BookInfoRule>, // 书籍信息页规则
    pub rule_toc: Option<TocRule>,       // 目录页规则
    pub rule_content: Option<ContentRule>, // 正文页规则

    // @Transient
    // private var _userNameSpace: String = ""
    pub user_name_space: String,

    // @Transient
    // private var debugLog: io.legado.app.model.DebugLog? = null
    pub debug_log: Option<DebugLog>,

    //    @Ignore
    //    @IgnoredOnParcel
    pub search_rule_v: Option<SearchRule>,

    //    @Ignore
    //    @IgnoredOnParcel
    pub explore_rule_v: Option<ExploreRule>,

    //    @Ignore
    //    @IgnoredOnParcel
    pub book_info_rule_v: Option<BookInfoRule>,

    //    @Ignore
    //    @IgnoredOnParcel
    pub toc_rule_v: Option<TocRule>,

    //    @Ignore
    //    @IgnoredOnParcel
    pub content_rule_v: Option<ContentRule>,
}

impl BookSource {
    pub fn set_user_name_space(&mut self, value: String) {
        self.user_name_space = value;
    }

    pub fn get_user_name_space(&self) -> String {
        self.user_name_space.clone()
    }

    pub fn set_logger(&mut self, value: Option<DebugLog>) {
        self.debug_log = value;
    }

    pub fn get_logger(&self) -> Option<DebugLog> {
        self.debug_log.clone()
    }

    pub fn get_tag(&self) -> String {
        self.book_source_name.clone()
    }

    pub fn get_key(&self) -> String {
        self.book_source_url.clone()
    }

    pub fn get_search_rule(&self) -> SearchRule {
        match &self.rule_search {
            Some(rule) => rule.clone(),
            None => SearchRule::default(),
        }
    }

    pub fn get_explore_rule(&self) -> ExploreRule {
        match &self.rule_explore {
            Some(rule) => rule.clone(),
            None => ExploreRule::default(),
        }
    }

    pub fn get_book_info_rule(&self) -> BookInfoRule {
        match &self.rule_book_info {
            Some(rule) => rule.clone(),
            None => BookInfoRule::default(),
        }
    }

    pub fn get_toc_rule(&self) -> TocRule {
        match &self.rule_toc {
            Some(rule) => rule.clone(),
            None => TocRule::default(),
        }
    }

    pub fn get_content_rule(&self) -> ContentRule {
        match &self.rule_content {
            Some(rule) => rule.clone(),
            None => ContentRule::default(),
        }
    }

//    fun getExploreKinds(): ArrayList<ExploreKind>? {
//        val exploreKinds = arrayListOf<ExploreKind>()
//        exploreUrl?.let {
//            var a = it
//            if (a.isNotBlank()) {
//                try {
//                    if (it.startsWith("<js>", false)) {
//                        val aCache = ACache.get(App.INSTANCE, "explore")
//                        a = aCache.getAsString(bookSourceUrl) ?: ""
//                        if (a.isBlank()) {
//                            val bindings = SimpleBindings()
//                            bindings["baseUrl"] = bookSourceUrl
//                            bindings["java"] = JsExtensions
//                            a = AppConst.SCRIPT_ENGINE.eval(
//                                it.substring(4, it.lastIndexOf("<")),
//                                bindings
//                            ).toString()
//                            aCache.put(bookSourceUrl, a)
//                        }
//                    }
//                    val b = a.split("(&&|\n)+".toRegex())
//                    b.map { c ->
//                        val d = c.split("::")
//                        if (d.size > 1)
//                            exploreKinds.add(ExploreKind(d[0], d[1]))
//                    }
//                } catch (e: Exception) {
//                    exploreKinds.add(ExploreKind(e.localizedMessage))
//                }
//            }
//        }
//        return exploreKinds
//    }

    pub fn equal(&self, source: &BookSource) -> bool {
        self.equal_str(&self.book_source_name, &source.book_source_name)
            && self.equal_str(&self.book_source_url, &source.book_source_url)
            && self.equal_str(&self.book_source_group, &source.book_source_group)
            && self.book_source_type == source.book_source_type
            && self.equal_str(&self.book_url_pattern, &source.book_url_pattern)
            && self.enabled == source.enabled
            && self.enabled_explore == source.enabled_explore
            && self.enabled_cookie_jar == source.enabled_cookie_jar
            && self.equal_str(&self.header, &source.header)
            && self.equal_str(&self.login_url, &source.login_url)
            && self.equal_str(&self.explore_url, &source.explore_url)
            && self.equal_str(&self.search_url, &source.search_url)
            && self.get_search_rule() == source.get_search_rule()
            && self.get_explore_rule() == source.get_explore_rule()
            && self.get_book_info_rule() == source.get_book_info_rule()
            && self.get_toc_rule() == source.get_toc_rule()
            && self.get_content_rule() == source.get_content_rule()
    }

    // private fun equal(a: String?, b: String?): Boolean {
    //     return a == b || (a.isNullOrEmpty() && b.isNullOrEmpty())
    // }
    fn equal_str(&self, a: &Option<String>, b: &Option<String>) -> bool {
        a == b || (a.is_null_or_empty() && b.is_null_or_empty())
    }
}

impl Default for BookSource {
    fn default() -> Self {
        BookSource {
            book_source_url: String::new(),
            book_source_name: String::new(),
            book_source_group: None,
            book_source_type: 0,
            book_url_pattern: None,
            custom_order: 0,
            enabled: true,
            enabled_explore: true,
            enabled_cookie_jar: Some(false),
            concurrent_rate: None,
            header: None,
            login_url: None,
            login_ui: None,
            login_check_js: None,
            book_source_comment: None,
            variable_comment: None,
            last_update_time: 0,
            respond_time: 180000,
            weight: 0,
            explore_url: None,
            rule_explore: None,
            search_url: None,
            rule_search: None,
            rule_book_info: None,
            rule_toc: None,
            rule_content: None,
            user_name_space: String::new(),
            debug_log: None,
            search_rule_v: None,
            explore_rule_v: None,
            book_info_rule_v: None,
            toc_rule_v: None,
            content_rule_v: None,
        }
    }
}

impl PartialEq for BookSource {
    // override fun equals(other: Any?) =
    //     if (other is BookSource) other.bookSourceUrl == bookSourceUrl else false
    fn eq(&self, other: &Self) -> bool {
        other.book_source_url == self.book_source_url
    }
}

impl Eq for BookSource {}

impl std::hash::Hash for BookSource {
    // override fun hashCode(): Int {
    //     return bookSourceUrl.hashCode()
    // }
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.book_source_url.hash(state);
    }
}

// data class ExploreKind(
//     var title: String,
//     var url: String? = null
// )
pub struct ExploreKind {
    pub title: String,
    pub url: Option<String>,
}

// companion object {
//     fun fromJson(json: String): Result<BookSource> {
//         return SourceAnalyzer.jsonToBookSource(json)
//     }
//     fun fromJsonArray(json: String): Result<MutableList<BookSource>> {
//         return SourceAnalyzer.jsonToBookSources(json)
//     }
//     fun fromJsonArray(inputStream: InputStream): Result<MutableList<BookSource>> {
//         return SourceAnalyzer.jsonToBookSources(inputStream)
//     }
// }
impl BookSource {
    pub fn from_json(json: String) -> Result<BookSource> {
        SourceAnalyzer::json_to_book_source(json)
    }

    pub fn from_json_array(json: String) -> Result<Vec<BookSource>> {
        SourceAnalyzer::json_to_book_sources(json)
    }

    pub fn from_json_array_input_stream(input_stream: InputStream) -> Result<Vec<BookSource>> {
        SourceAnalyzer::json_to_book_sources(input_stream)
    }
}

// class Converters {
//     fun exploreRuleToString(exploreRule: ExploreRule?): String = GSON.toJson(exploreRule)
//     fun stringToExploreRule(json: String?) = GSON.fromJsonObject<ExploreRule>(json).getOrNull()
//     fun searchRuleToString(searchRule: SearchRule?): String = GSON.toJson(searchRule)
//     fun stringToSearchRule(json: String?) = GSON.fromJsonObject<SearchRule>(json).getOrNull()
//     fun bookInfoRuleToString(bookInfoRule: BookInfoRule?): String = GSON.toJson(bookInfoRule)
//     fun stringToBookInfoRule(json: String?) = GSON.fromJsonObject<BookInfoRule>(json).getOrNull()
//     fun tocRuleToString(tocRule: TocRule?): String = GSON.toJson(tocRule)
//     fun stringToTocRule(json: String?) = GSON.fromJsonObject<TocRule>(json).getOrNull()
//     fun contentRuleToString(contentRule: ContentRule?): String = GSON.toJson(contentRule)
//     fun stringToContentRule(json: String?) = GSON.fromJsonObject<ContentRule>(json).getOrNull()
// }
pub struct Converters;

impl Converters {
    pub fn explore_rule_to_string(explore_rule: Option<ExploreRule>) -> String {
        GSON::to_json(explore_rule)
    }

    pub fn string_to_explore_rule(json: Option<String>) -> Option<ExploreRule> {
        GSON::from_json_object::<ExploreRule>(json).get_or_null()
    }

    pub fn search_rule_to_string(search_rule: Option<SearchRule>) -> String {
        GSON::to_json(search_rule)
    }

    pub fn string_to_search_rule(json: Option<String>) -> Option<SearchRule> {
        GSON::from_json_object::<SearchRule>(json).get_or_null()
    }

    pub fn book_info_rule_to_string(book_info_rule: Option<BookInfoRule>) -> String {
        GSON::to_json(book_info_rule)
    }

    pub fn string_to_book_info_rule(json: Option<String>) -> Option<BookInfoRule> {
        GSON::from_json_object::<BookInfoRule>(json).get_or_null()
    }

    pub fn toc_rule_to_string(toc_rule: Option<TocRule>) -> String {
        GSON::to_json(toc_rule)
    }

    pub fn string_to_toc_rule(json: Option<String>) -> Option<TocRule> {
        GSON::from_json_object::<TocRule>(json).get_or_null()
    }

    pub fn content_rule_to_string(content_rule: Option<ContentRule>) -> String {
        GSON::to_json(content_rule)
    }

    pub fn string_to_content_rule(json: Option<String>) -> Option<ContentRule> {
        GSON::from_json_object::<ContentRule>(json).get_or_null()
    }
}
