use crate::prelude::*;
// fix: GSON 为 io_legado_app_utils_gsonextensions::GSON（显式导入遮蔽 prelude 中 stubs::GSON 的 glob 歧义）
use crate::io_legado_app_utils_gsonextensions::GSON;
// fix: `Any` 被 stubs 与 analyzebyjsoup 两个 glob 同时导出，显式导入消歧义
use crate::stubs::Any;
#[allow(dead_code)]
pub struct SourceAnalyzer {
    headerPattern: Pattern,
    jsPattern: Pattern,
}

// fix: Kotlin `String.isJsonArray()/isJsonObject()` 扩展 → 本地辅助 trait
trait JsonStrExt {
    fn isJsonArray(&self) -> bool;
    fn isJsonObject(&self) -> bool;
}

impl JsonStrExt for str {
    fn isJsonArray(&self) -> bool {
        let s = self.trim();
        s.starts_with("[") && s.ends_with("]")
    }

    fn isJsonObject(&self) -> bool {
        let s = self.trim();
        s.starts_with("{") && s.ends_with("}")
    }
}

impl SourceAnalyzer {
    pub fn new() -> SourceAnalyzer {
        SourceAnalyzer {
            headerPattern: Pattern::compile_with("@Header:\\{.+?\\}", Pattern::CASE_INSENSITIVE),
            jsPattern: Pattern::compile_with("\\{\\{.+?\\}\\}", Pattern::CASE_INSENSITIVE),
        }
    }

    pub fn jsonToBookSources(&self, json: &str) -> Result<Vec<BookSource>, Box<dyn std::any::Any + Send>> {
        std::panic::catch_unwind(|| {
            let mut bookSources: Vec<BookSource> = Vec::new();
            if json.isJsonArray() {
                let items: Vec<Map<String, Any>> = JsonPath::parse(json).read("$").unwrap();
                for item in items {
                    let jsonItem = JsonPath::parse(serde_json::to_string(&item).unwrap_or_default());
                    bookSources.push(self.jsonToBookSource(&jsonItem.jsonString()).unwrap());
                }
            } else if json.isJsonObject() {
                bookSources.push(self.jsonToBookSource(json).unwrap());
            } else {
                panic!("格式不对");
            }
            bookSources
        })
    }

    pub fn jsonToBookSources_stream(&self, inputStream: &mut dyn InputStream) -> Result<Vec<BookSource>, Box<dyn std::any::Any + Send>> {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut bookSources: Vec<BookSource> = Vec::new();
            // fix: Kotlin `runCatching{...}.onFailure{...}` → 内部闭包返回 Result 传播错误
            let first: Result<(), StubError> = (|| {
                let items: Vec<Map<String, Any>> = JsonPath::parse_stream(inputStream)
                    .read("$")
                    .map_err(|e| StubError::new(e.to_string()))?;
                for item in items {
                    let jsonItem = JsonPath::parse(serde_json::to_string(&item).unwrap_or_default());
                    let it = self.jsonToBookSource(&jsonItem.jsonString())
                        .map_err(|_| StubError::new("jsonToBookSource 失败"))?;
                    bookSources.push(it);
                }
                Ok(())
            })();
            if first.is_err() {
                let item: Map<String, Any> = JsonPath::parse_stream(inputStream).read("$").unwrap();
                let jsonItem = JsonPath::parse(serde_json::to_string(&item).unwrap_or_default());
                bookSources.push(self.jsonToBookSource(&jsonItem.jsonString()).unwrap());
            }
            bookSources
        }))
    }

    pub fn jsonToBookSource(&self, json: &str) -> Result<BookSource, Box<dyn std::any::Any + Send>> {
        let mut source: BookSource = BookSource::default();
        let sourceAny: Option<BookSourceAny> = fromJsonObject::<BookSourceAny>(&GSON::new(), Some(json.trim()))
            .unwrap_or_else(|e| {

                // fix: Debug::log → logger().info（DebugLog 为转录损坏的 trait）
                let msg = e.downcast_ref::<StubError>().map(|se| se.msg.clone()).unwrap_or_default();
                logger().info(format!("转化书源出错: {}", msg));
                None
            });
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // fix: Kotlin `sourceAny?.ruleToc == null`（sourceAny 为 Option）→ 等价判空
            if sourceAny.as_ref().map(|s| s.ruleToc.is_none()).unwrap_or(true) {
                let jsonItem = JsonPath::parse(json.trim());
                source.book_source_url = jsonItem.readString("bookSourceUrl")
                    .unwrap_or_else(|| panic!("格式不对"));
                source.book_source_name = jsonItem.readString("bookSourceName").unwrap_or("".to_string());
                source.book_source_group = jsonItem.readString("bookSourceGroup");
                // fix: 旧格式书源导入（原注释掉——loginUrl/loginUi/loginCheckJs 配置丢失）
                source.login_url = jsonItem.readString("loginUrl");
                source.login_ui = jsonItem.readString("loginUi");
                source.login_check_js = jsonItem.readString("loginCheckJs");
                source.book_source_comment = Some(jsonItem.readString("bookSourceComment").unwrap_or_default());
                source.book_url_pattern = jsonItem.readString("ruleBookUrlPattern");
                source.custom_order = jsonItem.readInt("serialNumber").unwrap_or(0);
                source.header = Self::uaToHeader(jsonItem.readString("httpUserAgent").as_deref());
                source.search_url = Self::toNewUrl(jsonItem.readString("ruleSearchUrl").as_deref());
                source.explore_url = Self::toNewUrls(jsonItem.readString("ruleFindUrl").as_deref());
                let sourceType = jsonItem.readString("bookSourceType");
                source.book_source_type = match sourceType.as_deref() {
                    Some("AUDIO") => BookType::audio,
                    Some("audio") => BookType::audio,
                    Some("1") => BookType::audio,
                    Some("IMAGE") => BookType::image,
                    Some("image") => BookType::image,
                    Some("2") => BookType::image,
                    Some("FILE") => BookType::file,
                    Some("file") => BookType::file,
                    Some("3") => BookType::file,
                    _ => BookType::default,
                };
                source.enabled = jsonItem.readBool("enable").unwrap_or(true);
                // fix: Kotlin `exploreUrl.isNullOrBlank()` → OptionStringExt::is_null_or_empty
                if source.explore_url.is_null_or_empty() {
                    source.enabled_explore = false;
                }
                source.rule_search = Some(SearchRule {
                    book_list: Self::toNewRule(jsonItem.readString("ruleSearchList").as_deref()),
                    name: Self::toNewRule(jsonItem.readString("ruleSearchName").as_deref()),
                    author: Self::toNewRule(jsonItem.readString("ruleSearchAuthor").as_deref()),
                    intro: Self::toNewRule(jsonItem.readString("ruleSearchIntroduce").as_deref()),
                    kind: Self::toNewRule(jsonItem.readString("ruleSearchKind").as_deref()),
                    book_url: Self::toNewRule(jsonItem.readString("ruleSearchNoteUrl").as_deref()),
                    cover_url: Self::toNewRule(jsonItem.readString("ruleSearchCoverUrl").as_deref()),
                    last_chapter: Self::toNewRule(jsonItem.readString("ruleSearchLastChapter").as_deref()),
                    update_time: None,
                    word_count: None,
                });
                source.rule_explore = Some(ExploreRule {
                    book_list: Self::toNewRule(jsonItem.readString("ruleFindList").as_deref()),
                    name: Self::toNewRule(jsonItem.readString("ruleFindName").as_deref()),
                    author: Self::toNewRule(jsonItem.readString("ruleFindAuthor").as_deref()),
                    intro: Self::toNewRule(jsonItem.readString("ruleFindIntroduce").as_deref()),
                    kind: Self::toNewRule(jsonItem.readString("ruleFindKind").as_deref()),
                    book_url: Self::toNewRule(jsonItem.readString("ruleFindNoteUrl").as_deref()),
                    cover_url: Self::toNewRule(jsonItem.readString("ruleFindCoverUrl").as_deref()),
                    last_chapter: Self::toNewRule(jsonItem.readString("ruleFindLastChapter").as_deref()),
                    update_time: None,
                    word_count: None,
                });
                source.rule_book_info = Some(BookInfoRule {
                    init: Self::toNewRule(jsonItem.readString("ruleBookInfoInit").as_deref()),
                    name: Self::toNewRule(jsonItem.readString("ruleBookName").as_deref()),
                    author: Self::toNewRule(jsonItem.readString("ruleBookAuthor").as_deref()),
                    intro: Self::toNewRule(jsonItem.readString("ruleIntroduce").as_deref()),
                    kind: Self::toNewRule(jsonItem.readString("ruleBookKind").as_deref()),
                    cover_url: Self::toNewRule(jsonItem.readString("ruleCoverUrl").as_deref()),
                    last_chapter: Self::toNewRule(jsonItem.readString("ruleBookLastChapter").as_deref()),
                    toc_url: Self::toNewRule(jsonItem.readString("ruleChapterUrl").as_deref()),
                    update_time: None,
                    word_count: None,
                    can_re_name: None,
                });
                source.rule_toc = Some(TocRule {
                    chapter_list: Self::toNewRule(jsonItem.readString("ruleChapterList").as_deref()),
                    chapter_name: Self::toNewRule(jsonItem.readString("ruleChapterName").as_deref()),
                    chapter_url: Self::toNewRule(jsonItem.readString("ruleContentUrl").as_deref()),
                    next_toc_url: Self::toNewRule(jsonItem.readString("ruleChapterUrlNext").as_deref()),
                    pre_update_js: None,
                    is_volume: None,
                    is_vip: None,
                    update_time: None,
                });
                let mut content = Self::toNewRule(jsonItem.readString("ruleBookContent").as_deref()).unwrap_or("".to_string());
                if content.starts_with("$") && !content.starts_with("$.") {
                    content = content[1..].to_string();
                }
                source.rule_content = Some(ContentRule {
                    content: Some(content),
                    replace_regex: Self::toNewRule(jsonItem.readString("ruleBookContentReplace").as_deref()),
                    next_content_url: Self::toNewRule(jsonItem.readString("ruleContentUrlNext").as_deref()),
                    web_js: None,
                    source_regex: None,
                    image_style: None,
                });
            } else {
                // fix: else 分支等价于 Kotlin smart cast（sourceAny 必非空）
                let sourceAny = sourceAny.unwrap();
                source.book_source_url = sourceAny.bookSourceUrl;
                source.book_source_name = sourceAny.bookSourceName;
                source.book_source_group = sourceAny.bookSourceGroup;
                source.book_source_type = sourceAny.bookSourceType;
                source.book_url_pattern = sourceAny.bookUrlPattern;
                source.custom_order = sourceAny.customOrder;
                source.enabled = sourceAny.enabled;
                source.enabled_explore = sourceAny.enabledExplore;
                source.enabled_cookie_jar = Some(sourceAny.enabledCookieJar);
                source.concurrent_rate = sourceAny.concurrentRate;
                source.header = sourceAny.header;
                source.login_url = match sourceAny.loginUrl {
                    None => None,
                    Some(Any::Str(s)) => Some(s),
                    Some(other) => JsonPath::parse(other).readString("url"),
                };
                // fix: 真实解析 loginUi（原注释掉——书源登录 UI 配置丢失，前端登录表单无法按源渲染）
                source.login_ui = sourceAny.loginUi.as_ref().map(|ui| match ui {
                    Any::Str(s) => s.clone(),
                    other => GSON::new().toJson(other),
                });
                source.login_check_js = sourceAny.loginCheckJs;
                source.book_source_comment = sourceAny.bookSourceComment;
                source.last_update_time = sourceAny.lastUpdateTime;
                source.respond_time = sourceAny.respondTime;
                source.weight = sourceAny.weight;
                source.explore_url = sourceAny.exploreUrl;
                source.rule_explore = match sourceAny.ruleExplore {
                    Some(Any::Str(s)) => fromJsonObject::<ExploreRule>(&GSON::new(), Some(s.as_str())).ok().flatten(),
                    other => {
                        let json = GSON::new().toJson(other);
                        fromJsonObject::<ExploreRule>(&GSON::new(), Some(json.as_str())).ok().flatten()
                    }
                };
                source.search_url = sourceAny.searchUrl;
                source.rule_search = match sourceAny.ruleSearch {
                    Some(Any::Str(s)) => fromJsonObject::<SearchRule>(&GSON::new(), Some(s.as_str())).ok().flatten(),
                    other => {
                        let json = GSON::new().toJson(other);
                        fromJsonObject::<SearchRule>(&GSON::new(), Some(json.as_str())).ok().flatten()
                    }
                };
                source.rule_book_info = match sourceAny.ruleBookInfo {
                    Some(Any::Str(s)) => fromJsonObject::<BookInfoRule>(&GSON::new(), Some(s.as_str())).ok().flatten(),
                    other => {
                        let json = GSON::new().toJson(other);
                        fromJsonObject::<BookInfoRule>(&GSON::new(), Some(json.as_str())).ok().flatten()
                    }
                };
                source.rule_toc = match sourceAny.ruleToc {
                    Some(Any::Str(s)) => fromJsonObject::<TocRule>(&GSON::new(), Some(s.as_str())).ok().flatten(),
                    other => {
                        let json = GSON::new().toJson(other);
                        fromJsonObject::<TocRule>(&GSON::new(), Some(json.as_str())).ok().flatten()
                    }
                };
                source.rule_content = match sourceAny.ruleContent {
                    Some(Any::Str(s)) => fromJsonObject::<ContentRule>(&GSON::new(), Some(s.as_str())).ok().flatten(),
                    other => {
                        let json = GSON::new().toJson(other);
                        fromJsonObject::<ContentRule>(&GSON::new(), Some(json.as_str())).ok().flatten()
                    }
                };
            }
            source
        }))
    }

    // default规则适配
    // #正则#替换内容 替换成 ##正则##替换内容
    // | 替换成 ||
    // & 替换成 &&
    fn toNewRule(oldRule: Option<&str>) -> Option<String> {
        let oldRule = match oldRule {
            Some(s) if !s.is_blank() => s,
            _ => return None,
        };
        let mut newRule = oldRule.to_string();
        let mut reverse = false;
        let mut allinone = false;
        if oldRule.starts_with("-") {
            reverse = true;
            newRule = oldRule[1..].to_string();
        }
        if newRule.starts_with("+") {
            allinone = true;
            newRule = newRule[1..].to_string();
        }
        if !newRule.starts_with_ignore_case("@CSS:") &&
            !newRule.starts_with_ignore_case("@XPath:") &&
            !newRule.starts_with("//") &&
            !newRule.starts_with("##") &&
            !newRule.starts_with(":") &&
            !newRule.contains_ignore_case("@js:") &&
            !newRule.contains_ignore_case("<js>")
        {
            if newRule.contains("#") && !newRule.contains("##") {
                newRule = oldRule.replace("#", "##");
            }
            if newRule.contains("|") && !newRule.contains("||") {
                if newRule.contains("##") {
                    // fix: split 结果转为 owned Vec<String>（避免索引切片与可变赋值冲突）
                    let list: Vec<String> = newRule.split("##").map(|s| s.to_string()).collect();
                    if list[0].contains("|") {
                        newRule = list[0].replace("|", "||");
                        for i in 1..list.len() {
                            newRule += "##";
                            newRule += &list[i];
                        }
                    }
                } else {
                    newRule = newRule.replace("|", "||");
                }
            }
            if newRule.contains("&")
                && !newRule.contains("&&")
                && !newRule.contains("http")
                && !newRule.starts_with("/")
            {
                newRule = newRule.replace("&", "&&");
            }
        }
        if allinone {
            newRule = format!("+{}", newRule);
        }
        if reverse {
            newRule = format!("-{}", newRule);
        }
        Some(newRule)
    }

    fn toNewUrls(oldUrls: Option<&str>) -> Option<String> {
        let oldUrls = match oldUrls {
            Some(s) if !s.is_blank() => s,
            _ => return None,
        };
        if oldUrls.starts_with("@js:") || oldUrls.starts_with("<js>") {
            return Some(oldUrls.to_string());
        }
        if !oldUrls.contains("\n") && !oldUrls.contains("&&") {
            return Self::toNewUrl(Some(oldUrls));
        }
        let urls = oldUrls.split_with_regex("(&&|\r?\n)+");
        let mapped: Vec<String> = urls.iter().map(|it| {
            Self::toNewUrl(Some(it.as_str())).map(|s| s.replace_with_regex("\n\\s*", "")).unwrap_or_default()
        }).collect();
        Some(mapped.join("\n"))
    }

    fn toNewUrl(oldUrl: Option<&str>) -> Option<String> {
        let oldUrl = match oldUrl {
            Some(s) if !s.is_blank() => s,
            _ => return None,
        };
        let mut url: String = oldUrl.to_string();
        if oldUrl.starts_with_ignore_case("<js>") {
            url = url.replace("=searchKey", "={{key}}")
                .replace("=searchPage", "={{page}}");
            return Some(url);
        }
        let mut map = HashMap::<String, String>::new();
        let header_pattern = Pattern::compile_with("@Header:\\{.+?\\}", Pattern::CASE_INSENSITIVE);
        let mut mather = header_pattern.matcher(url.clone());
        if mather.find() {
            let header = mather.group();
            url = url.replace(&header, "");
            map.insert("headers".to_string(), header[8..].to_string());
        }
        // fix: split 结果转为 owned Vec<String>（原转录借用 url，与后续赋值冲突）
        let mut urlList: Vec<String> = url.split("|").map(|s| s.to_string()).collect();
        url = urlList[0].to_string();
        if urlList.len() > 1 {
            map.insert("charset".to_string(), urlList[1].split("=").nth(1).unwrap().to_string());
        }
        let js_pattern = Pattern::compile_with("\\{\\{.+?\\}\\}", Pattern::CASE_INSENSITIVE);
        mather = js_pattern.matcher(url.clone());
        let mut jsList: Vec<String> = Vec::new();
        while mather.find() {
            let group = mather.group();
            jsList.push(group.clone());
            url = url.replace(jsList.last().unwrap(), &format!("$${}", jsList.len() - 1));
        }
        url = url.replace("{", "<").replace("}", ">");
        url = url.replace("searchKey", "{{key}}");
        url = url.replace_with_regex("<searchPage([-+]1)>", "{{page$1}}")
            .replace_with_regex("searchPage([-+]1)", "{{page$1}}")
            .replace("searchPage", "{{page}}");
        for (index, item) in jsList.iter().enumerate() {
            url = url.replace(
                &format!("${index}"),
                &item.replace("searchKey", "key").replace("searchPage", "page")
            );
        }
        urlList = url.split("@").map(|s| s.to_string()).collect();
        url = urlList[0].to_string();
        if urlList.len() > 1 {
            map.insert("method".to_string(), "POST".to_string());
            map.insert("body".to_string(), urlList[1].to_string());
        }
        if map.len() > 0 {
            url += ",";
            url += &GSON::new().toJson(&map);
        }
        Some(url)
    }

    fn uaToHeader(ua: Option<&str>) -> Option<String> {
        let ua = match ua {
            Some(s) if !s.is_empty() => s,
            _ => return None,
        };
        let map = vec![(AppConst::UA_NAME.to_string(), ua.to_string())];
        Some(GSON::new().toJson(&map))
    }
}

// fix: BookSourceAny 补充 serde 反序列化（Gson fromJsonObject 等价），缺省值对齐 Kotlin data class 默认参数
fn default_true() -> bool {
    true
}

fn default_empty_string() -> Option<String> {
    Some(String::new())
}

fn default_respond_time() -> i64 {
    180000
}

#[derive(serde::Deserialize)]
pub struct BookSourceAny {
    #[serde(default)]
    pub bookSourceName: String,                // 名称
    pub bookSourceGroup: Option<String>,            // 分组
    #[serde(default)]
    pub bookSourceUrl: String,                 // 地址，包括 http/https
    #[serde(default)]
    pub bookSourceType: i32,     // 类型，0 文本，1 音频
    pub bookUrlPattern: Option<String>,             // 详情页url正则
    #[serde(default)]
    pub customOrder: i32,                       // 手动排序编号
    #[serde(default = "default_true")]
    pub enabled: bool,                    // 是否启用
    #[serde(default = "default_true")]
    pub enabledExplore: bool,             // 启用发现
    #[serde(default)]
    pub enabledCookieJar: bool,          // 启用CookieJar
    pub concurrentRate: Option<String>,             // 并发率
    pub header: Option<String>,                     // 请求头
    pub loginUrl: Option<Any>,                      // 登录规则
    pub loginUi: Option<Any>,                       // 登录UI
    pub loginCheckJs: Option<String>,               //登录检测js
    #[serde(default = "default_empty_string")]
    pub bookSourceComment: Option<String>,            //书源注释
    #[serde(default)]
    pub lastUpdateTime: i64,                   // 最后更新时间，用于排序
    #[serde(default = "default_respond_time")]
    pub respondTime: i64,                // 响应时间，用于排序
    #[serde(default)]
    pub weight: i32,                            // 智能排序的权重
    pub exploreUrl: Option<String>,                 // 发现url
    pub ruleExplore: Option<Any>,                   // 发现规则
    pub searchUrl: Option<String>,                  // 搜索url
    pub ruleSearch: Option<Any>,                    // 搜索规则
    pub ruleBookInfo: Option<Any>,                  // 书籍信息页规则
    pub ruleToc: Option<Any>,                       // 目录页规则
    pub ruleContent: Option<Any>,                    // 正文页规则
}

impl BookSourceAny {
    pub fn new() -> BookSourceAny {
        BookSourceAny {
            bookSourceName: "".to_string(),
            bookSourceGroup: None,
            bookSourceUrl: "".to_string(),
            bookSourceType: BookType::default,
            bookUrlPattern: None,
            customOrder: 0,
            enabled: true,
            enabledExplore: true,
            enabledCookieJar: false,
            concurrentRate: None,
            header: None,
            loginUrl: None,
            loginUi: None,
            loginCheckJs: None,
            bookSourceComment: Some("".to_string()),
            lastUpdateTime: 0,
            respondTime: 180000,
            weight: 0,
            exploreUrl: None,
            ruleExplore: None,
            searchUrl: None,
            ruleSearch: None,
            ruleBookInfo: None,
            ruleToc: None,
            ruleContent: None,
        }
    }
}
