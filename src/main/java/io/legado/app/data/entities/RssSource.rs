// package io.legado.app.data.entities

// import com.jayway.jsonpath.DocumentContext
// import com.fasterxml.jackson.annotation.JsonIgnoreProperties
// import io.legado.app.help.CacheManager
// import io.legado.app.help.JsExtensions
// import io.legado.app.help.http.CookieStore
// import io.legado.app.constant.AppConst
// import javax.script.SimpleBindings
// import io.legado.app.utils.*

// @JsonIgnoreProperties("headerMap", "userNameSpace", "loginHeader", "loginHeaderMap", "loginInfo", "loginInfoMap")
pub struct RssSource {
    pub source_url: String,
    pub source_name: String,
    pub source_icon: String,
    pub source_group: Option<String>,
    pub source_comment: Option<String>,
    pub enabled: bool,
    pub variable_comment: Option<String>,
    pub enabled_cookie_jar: Option<bool>,
    pub concurrent_rate: Option<String>,    //并发率
    pub header: Option<String>,             // 请求头
    pub login_url: Option<String>,          // 登录地址
    pub login_ui: Option<String>,
    pub login_check_js: Option<String>,     //登录检测js
    pub sort_url: Option<String>,
    pub single_url: bool,
    //列表规则
    pub article_style: i32,                 //列表样式,0,1,2
    pub rule_articles: Option<String>,
    pub rule_next_page: Option<String>,
    pub rule_title: Option<String>,
    pub rule_pub_date: Option<String>,
    //webView规则
    pub rule_description: Option<String>,
    pub rule_image: Option<String>,
    pub rule_link: Option<String>,
    pub rule_content: Option<String>,
    pub style: Option<String>,
    pub enable_js: bool,
    pub load_with_base_url: bool,
    pub custom_order: i32,

    // @Transient
    // private var _userNameSpace: String = ""
    pub user_name_space: String,

    // @Transient
    // private var debugLog: io.legado.app.model.DebugLog? = null
    pub debug_log: Option<DebugLog>,
}

impl RssSource {
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
        self.source_name.clone()
    }

    pub fn get_key(&self) -> String {
        self.source_url.clone()
    }

    pub fn equal(&self, source: &RssSource) -> bool {
        self.equal_str(&self.source_url, &source.source_url)
            && self.equal_str(&self.source_icon, &source.source_icon)
            && self.enabled == source.enabled
            && self.equal_str(&self.source_group, &source.source_group)
            && self.equal_str(&self.rule_articles, &source.rule_articles)
            && self.equal_str(&self.rule_next_page, &source.rule_next_page)
            && self.equal_str(&self.rule_title, &source.rule_title)
            && self.equal_str(&self.rule_pub_date, &source.rule_pub_date)
            && self.equal_str(&self.rule_description, &source.rule_description)
            && self.equal_str(&self.rule_link, &source.rule_link)
            && self.equal_str(&self.rule_content, &source.rule_content)
            && self.enable_js == source.enable_js
            && self.load_with_base_url == source.load_with_base_url
    }

    // private fun equal(a: String?, b: String?): Boolean {
    //     return a == b || (a.isNullOrEmpty() && b.isNullOrEmpty())
    // }
    fn equal_str(&self, a: &Option<String>, b: &Option<String>) -> bool {
        a == b || (a.is_null_or_empty() && b.is_null_or_empty())
    }

    // fun sortUrls(): List<Pair<String, String>> = arrayListOf<Pair<String, String>>().apply {
    //     kotlin.runCatching {
    //         var a = sortUrl
    //         if (sortUrl?.startsWith("<js>", false) == true
    //             || sortUrl?.startsWith("@js:", false) == true
    //         ) {
    //             val jsStr = if (sortUrl!!.startsWith("@")) {
    //                 sortUrl!!.substring(4)
    //             } else {
    //                 sortUrl!!.substring(4, sortUrl!!.lastIndexOf("<"))
    //             }
    //             a = evalJS(jsStr).toString()
    //         }
    //         a?.split("(&&|\n)+".toRegex())?.forEach { c ->
    //             val d = c.split("::")
    //             if (d.size > 1)
    //                 add(Pair(d[0], d[1]))
    //         }
    //         if (isEmpty()) {
    //             add(Pair("", sourceUrl))
    //         }
    //     }
    // }
    pub fn sort_urls(&self) -> Vec<(String, String)> {
        let mut result: Vec<(String, String)> = Vec::new();
        // kotlin.runCatching { ... }
        let caught = (|| -> Result<()> {
            let mut a = self.sort_url.clone();
            if self.sort_url.as_deref().map_or(false, |s| starts_with_ignore_case(s, "<js>"))
                || self.sort_url.as_deref().map_or(false, |s| starts_with_ignore_case(s, "@js:")) {
                let js_str = if self.sort_url.as_ref().unwrap().starts_with("@") {
                    self.sort_url.as_ref().unwrap()[4..].to_string()
                } else {
                    self.sort_url.as_ref().unwrap()[4..self.sort_url.as_ref().unwrap().rfind("<").unwrap()].to_string()
                };
                a = Some(self.eval_js(js_str, None).map(|v| v.to_string()).unwrap_or_default());
            }
            if let Some(a_str) = a {
                for c in a_str.split(|ch| ch == '&' || ch == '\n') {
                    let d: Vec<&str> = c.split("::").collect();
                    if d.len() > 1 {
                        result.push((d[0].to_string(), d[1].to_string()));
                    }
                }
            }
            if result.is_empty() {
                result.push((String::new(), self.source_url.clone()));
            }
            Ok(())
        })();
        let _ = caught;
        result
    }
}

impl Default for RssSource {
    fn default() -> Self {
        RssSource {
            source_url: String::new(),
            source_name: String::new(),
            source_icon: String::new(),
            source_group: None,
            source_comment: None,
            enabled: true,
            variable_comment: None,
            enabled_cookie_jar: Some(false),
            concurrent_rate: None,
            header: None,
            login_url: None,
            login_ui: None,
            login_check_js: None,
            sort_url: None,
            single_url: false,
            article_style: 0,
            rule_articles: None,
            rule_next_page: None,
            rule_title: None,
            rule_pub_date: None,
            rule_description: None,
            rule_image: None,
            rule_link: None,
            rule_content: None,
            style: None,
            enable_js: true,
            load_with_base_url: true,
            custom_order: 0,
            user_name_space: String::new(),
            debug_log: None,
        }
    }
}

impl PartialEq for RssSource {
    // override fun equals(other: Any?): Boolean {
    //     if (other is RssSource) {
    //         return other.sourceUrl == sourceUrl
    //     }
    //     return false
    // }
    fn eq(&self, other: &Self) -> bool {
        other.source_url == self.source_url
    }
}

impl Eq for RssSource {}

impl std::hash::Hash for RssSource {
    // override fun hashCode() = sourceUrl.hashCode()
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.source_url.hash(state);
    }
}

// it.startsWith(prefix, ignoreCase = true) 的等价实现
fn starts_with_ignore_case(s: &str, prefix: &str) -> bool {
    s.to_lowercase().starts_with(&prefix.to_lowercase())
}

// @Suppress("MemberVisibilityCanBePrivate")
// companion object {
//     fun fromJsonDoc(doc: DocumentContext): Result<RssSource> {
//         return kotlin.runCatching {
//             // val loginUi = doc.read<Any>("$.loginUi")
//             RssSource(...)
//         }
//     }
//     fun fromJson(json: String): Result<RssSource> {
//         return fromJsonDoc(jsonPath.parse(json))
//     }
//     fun fromJsonArray(jsonArray: String): Result<ArrayList<RssSource>> {
//         return kotlin.runCatching {
//             val sources = arrayListOf<RssSource>()
//             val doc = jsonPath.parse(jsonArray).read<List<*>>("$")
//             doc.forEach {
//                 val jsonItem = jsonPath.parse(it)
//                 fromJsonDoc(jsonItem).getOrThrow().let { source ->
//                     sources.add(source)
//                 }
//             }
//             sources
//         }
//     }
// }
impl RssSource {
    pub fn from_json_doc(doc: DocumentContext) -> Result<RssSource> {
        // kotlin.runCatching { ... }
        (|| -> Result<RssSource> {
            // val loginUi = doc.read<Any>("$.loginUi")
            let mut source = RssSource {
                source_url: doc.read_string("$.sourceUrl").expect("sourceUrl"),
                source_name: doc.read_string("$.sourceName").expect("sourceName"),
                source_icon: doc.read_string("$.sourceIcon").unwrap_or_default(),
                source_group: doc.read_string("$.sourceGroup"),
                source_comment: doc.read_string("$.sourceComment"),
                enabled: doc.read_bool("$.enabled").unwrap_or(true),
                enabled_cookie_jar: doc.read_bool("$.enabledCookieJar").unwrap_or(false),
                concurrent_rate: doc.read_string("$.concurrentRate"),
                header: doc.read_string("$.header"),
                login_url: doc.read_string("$.loginUrl"),
                // loginUi = if (loginUi is List<*>) GSON.toJson(loginUi) else loginUi?.toString(),
                login_check_js: doc.read_string("$.loginCheckJs"),
                sort_url: doc.read_string("$.sortUrl"),
                single_url: doc.read_bool("$.singleUrl").unwrap_or(false),
                article_style: doc.read_int("$.articleStyle").unwrap_or(0),
                rule_articles: doc.read_string("$.ruleArticles"),
                rule_next_page: doc.read_string("$.ruleNextPage"),
                rule_title: doc.read_string("$.ruleTitle"),
                rule_pub_date: doc.read_string("$.rulePubDate"),
                rule_description: doc.read_string("$.ruleDescription"),
                rule_image: doc.read_string("$.ruleImage"),
                rule_link: doc.read_string("$.ruleLink"),
                rule_content: doc.read_string("$.ruleContent"),
                style: doc.read_string("$.style"),
                enable_js: doc.read_bool("$.enableJs").unwrap_or(true),
                load_with_base_url: doc.read_bool("$.loadWithBaseUrl").unwrap_or(true),
                custom_order: doc.read_int("$.customOrder").unwrap_or(0),
                ..RssSource::default()
            };
            Ok(source)
        })()
    }

    pub fn from_json(json: String) -> Result<RssSource> {
        Self::from_json_doc(json_path::parse(json))
    }

    pub fn from_json_array(json_array: String) -> Result<Vec<RssSource>> {
        // kotlin.runCatching { ... }
        (|| -> Result<Vec<RssSource>> {
            let mut sources: Vec<RssSource> = Vec::new();
            let doc = json_path::parse(json_array).read::<Vec<Box<dyn Any>>>("$");
            for it in doc {
                let json_item = json_path::parse(it);
                let source = Self::from_json_doc(json_item)?.ok_or(Error)?;
                sources.push(source);
            }
            Ok(sources)
        })()
    }
}
