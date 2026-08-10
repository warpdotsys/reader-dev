#[allow(dead_code)]
pub struct SourceAnalyzer {
    headerPattern: Pattern,
    jsPattern: Pattern,
}

impl SourceAnalyzer {
    pub fn new() -> SourceAnalyzer {
        SourceAnalyzer {
            headerPattern: Pattern::compile("@Header:\\{.+?\\}", Pattern::CASE_INSENSITIVE),
            jsPattern: Pattern::compile("\\{\\{.+?\\}\\}", Pattern::CASE_INSENSITIVE),
        }
    }

    pub fn jsonToBookSources(&self, json: &str) -> Result<Vec<BookSource>> {
        std::panic::catch_unwind(|| {
            let mut bookSources: Vec<BookSource> = Vec::new();
            if json.isJsonArray() {
                let items: Vec<Map<String, Any>> = jsonPath().parse(json).read("$");
                for item in items {
                    let jsonItem = jsonPath().parse(&item);
                    self.jsonToBookSource(&jsonItem.jsonString()).unwrap().map(|it| {
                        bookSources.push(it);
                    });
                }
            } else if json.isJsonObject() {
                self.jsonToBookSource(json).unwrap().map(|it| {
                    bookSources.push(it);
                });
            } else {
                panic!(NoStackTraceException::new("格式不对"));
            }
            bookSources
        })
    }

    pub fn jsonToBookSources_stream(&self, inputStream: &mut dyn InputStream) -> Result<Vec<BookSource>> {
        std::panic::catch_unwind(|| {
            let mut bookSources: Vec<BookSource> = Vec::new();
            let first: Result<(), ()> = (|| {
                let items: Vec<Map<String, Any>> = jsonPath().parse_stream(inputStream).read("$");
                for item in items {
                    let jsonItem = jsonPath().parse(&item);
                    self.jsonToBookSource(&jsonItem.jsonString()).unwrap().map(|it| {
                        bookSources.push(it);
                    });
                }
                Ok(())
            })();
            if first.is_err() {
                let item: Map<String, Any> = jsonPath().parse_stream(inputStream).read("$");
                let jsonItem = jsonPath().parse(&item);
                self.jsonToBookSource(&jsonItem.jsonString()).unwrap().map(|it| {
                    bookSources.push(it);
                });
            }
            bookSources
        })
    }

    pub fn jsonToBookSource(&self, json: &str) -> Result<BookSource> {
        let mut source = BookSource::new();
        let sourceAny: Option<BookSourceAny> = fromJsonObject::<BookSourceAny>(&GSON::new(), Some(json.trim()))
            .unwrap_or_else(|e| {
                Debug::log("转化书源出错", e.localizedMessage());
                Ok(None)
            })
            .ok()
            .flatten();
        std::panic::catch_unwind(|| {
            if sourceAny.ruleToc == None {
                let jsonItem = jsonPath().parse(json.trim());
                source.bookSourceUrl = jsonItem.readString("bookSourceUrl")
                    .unwrap_or_else(|| panic!(NoStackTraceException::new("格式不对")));
                source.bookSourceName = jsonItem.readString("bookSourceName").unwrap_or("".to_string());
                source.bookSourceGroup = jsonItem.readString("bookSourceGroup");
                // loginUrl = jsonItem.readString("loginUrl")
                // loginUi = jsonItem.readString("loginUi")
                // loginCheckJs = jsonItem.readString("loginCheckJs")
                source.bookSourceComment = jsonItem.readString("bookSourceComment").unwrap_or("".to_string());
                source.bookUrlPattern = jsonItem.readString("ruleBookUrlPattern");
                source.customOrder = jsonItem.readInt("serialNumber").unwrap_or(0);
                source.header = Self::uaToHeader(jsonItem.readString("httpUserAgent"));
                source.searchUrl = Self::toNewUrl(jsonItem.readString("ruleSearchUrl"));
                source.exploreUrl = Self::toNewUrls(jsonItem.readString("ruleFindUrl"));
                let sourceType = jsonItem.readString("bookSourceType");
                source.bookSourceType = match sourceType.as_deref() {
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
                if source.exploreUrl == None || source.exploreUrl.as_deref().unwrap().is_blank() {
                    source.enabledExplore = false;
                }
                source.ruleSearch = SearchRule::new(
                    bookList = Self::toNewRule(jsonItem.readString("ruleSearchList")),
                    name = Self::toNewRule(jsonItem.readString("ruleSearchName")),
                    author = Self::toNewRule(jsonItem.readString("ruleSearchAuthor")),
                    intro = Self::toNewRule(jsonItem.readString("ruleSearchIntroduce")),
                    kind = Self::toNewRule(jsonItem.readString("ruleSearchKind")),
                    bookUrl = Self::toNewRule(jsonItem.readString("ruleSearchNoteUrl")),
                    coverUrl = Self::toNewRule(jsonItem.readString("ruleSearchCoverUrl")),
                    lastChapter = Self::toNewRule(jsonItem.readString("ruleSearchLastChapter"))
                );
                source.ruleExplore = ExploreRule::new(
                    bookList = Self::toNewRule(jsonItem.readString("ruleFindList")),
                    name = Self::toNewRule(jsonItem.readString("ruleFindName")),
                    author = Self::toNewRule(jsonItem.readString("ruleFindAuthor")),
                    intro = Self::toNewRule(jsonItem.readString("ruleFindIntroduce")),
                    kind = Self::toNewRule(jsonItem.readString("ruleFindKind")),
                    bookUrl = Self::toNewRule(jsonItem.readString("ruleFindNoteUrl")),
                    coverUrl = Self::toNewRule(jsonItem.readString("ruleFindCoverUrl")),
                    lastChapter = Self::toNewRule(jsonItem.readString("ruleFindLastChapter"))
                );
                source.ruleBookInfo = BookInfoRule::new(
                    init = Self::toNewRule(jsonItem.readString("ruleBookInfoInit")),
                    name = Self::toNewRule(jsonItem.readString("ruleBookName")),
                    author = Self::toNewRule(jsonItem.readString("ruleBookAuthor")),
                    intro = Self::toNewRule(jsonItem.readString("ruleIntroduce")),
                    kind = Self::toNewRule(jsonItem.readString("ruleBookKind")),
                    coverUrl = Self::toNewRule(jsonItem.readString("ruleCoverUrl")),
                    lastChapter = Self::toNewRule(jsonItem.readString("ruleBookLastChapter")),
                    tocUrl = Self::toNewRule(jsonItem.readString("ruleChapterUrl"))
                );
                source.ruleToc = TocRule::new(
                    chapterList = Self::toNewRule(jsonItem.readString("ruleChapterList")),
                    chapterName = Self::toNewRule(jsonItem.readString("ruleChapterName")),
                    chapterUrl = Self::toNewRule(jsonItem.readString("ruleContentUrl")),
                    nextTocUrl = Self::toNewRule(jsonItem.readString("ruleChapterUrlNext"))
                );
                let mut content = Self::toNewRule(jsonItem.readString("ruleBookContent")).unwrap_or("".to_string());
                if content.starts_with("$") && !content.starts_with("$.") {
                    content = content[1..].to_string();
                }
                source.ruleContent = ContentRule::new(
                    content = content,
                    replaceRegex = Self::toNewRule(jsonItem.readString("ruleBookContentReplace")),
                    nextContentUrl = Self::toNewRule(jsonItem.readString("ruleContentUrlNext"))
                );
            } else {
                source.bookSourceUrl = sourceAny.bookSourceUrl;
                source.bookSourceName = sourceAny.bookSourceName;
                source.bookSourceGroup = sourceAny.bookSourceGroup;
                source.bookSourceType = sourceAny.bookSourceType;
                source.bookUrlPattern = sourceAny.bookUrlPattern;
                source.customOrder = sourceAny.customOrder;
                source.enabled = sourceAny.enabled;
                source.enabledExplore = sourceAny.enabledExplore;
                source.enabledCookieJar = sourceAny.enabledCookieJar;
                source.concurrentRate = sourceAny.concurrentRate;
                source.header = sourceAny.header;
                source.loginUrl = match sourceAny.loginUrl {
                    None => None,
                    Some(Any::String(s)) => Some(s),
                    Some(other) => JsonPath::parse(other).readString("url"),
                };
                // source.loginUi = if (sourceAny.loginUi is List<*>) {
                //     GSON.toJson(sourceAny.loginUi)
                // } else {
                //     sourceAny.loginUi?.toString()
                // }
                source.loginCheckJs = sourceAny.loginCheckJs;
                source.bookSourceComment = sourceAny.bookSourceComment;
                source.lastUpdateTime = sourceAny.lastUpdateTime;
                source.respondTime = sourceAny.respondTime;
                source.weight = sourceAny.weight;
                source.exploreUrl = sourceAny.exploreUrl;
                source.ruleExplore = if let Any::String(s) = sourceAny.ruleExplore {
                    fromJsonObject::<ExploreRule>(&GSON::new(), Some(s)).ok().flatten()
                } else {
                    fromJsonObject::<ExploreRule>(&GSON::new(), Some(&GSON::new().toJson(sourceAny.ruleExplore))).ok().flatten()
                };
                source.searchUrl = sourceAny.searchUrl;
                source.ruleSearch = if let Any::String(s) = sourceAny.ruleSearch {
                    fromJsonObject::<SearchRule>(&GSON::new(), Some(s)).ok().flatten()
                } else {
                    fromJsonObject::<SearchRule>(&GSON::new(), Some(&GSON::new().toJson(sourceAny.ruleSearch))).ok().flatten()
                };
                source.ruleBookInfo = if let Any::String(s) = sourceAny.ruleBookInfo {
                    fromJsonObject::<BookInfoRule>(&GSON::new(), Some(s)).ok().flatten()
                } else {
                    fromJsonObject::<BookInfoRule>(&GSON::new(), Some(&GSON::new().toJson(sourceAny.ruleBookInfo))).ok().flatten()
                };
                source.ruleToc = if let Any::String(s) = sourceAny.ruleToc {
                    fromJsonObject::<TocRule>(&GSON::new(), Some(s)).ok().flatten()
                } else {
                    fromJsonObject::<TocRule>(&GSON::new(), Some(&GSON::new().toJson(sourceAny.ruleToc))).ok().flatten()
                };
                source.ruleContent = if let Any::String(s) = sourceAny.ruleContent {
                    fromJsonObject::<ContentRule>(&GSON::new(), Some(s)).ok().flatten()
                } else {
                    fromJsonObject::<ContentRule>(&GSON::new(), Some(&GSON::new().toJson(sourceAny.ruleContent))).ok().flatten()
                };
            }
            source
        })
    }

    pub struct BookSourceAny {
        pub bookSourceName: String,                // 名称
        pub bookSourceGroup: Option<String>,            // 分组
        pub bookSourceUrl: String,                 // 地址，包括 http/https
        pub bookSourceType: i32,     // 类型，0 文本，1 音频
        pub bookUrlPattern: Option<String>,             // 详情页url正则
        pub customOrder: i32,                       // 手动排序编号
        pub enabled: bool,                    // 是否启用
        pub enabledExplore: bool,             // 启用发现
        pub enabledCookieJar: bool,          // 启用CookieJar
        pub concurrentRate: Option<String>,             // 并发率
        pub header: Option<String>,                     // 请求头
        pub loginUrl: Option<Any>,                      // 登录规则
        pub loginUi: Option<Any>,                       // 登录UI
        pub loginCheckJs: Option<String>,               //登录检测js
        pub bookSourceComment: Option<String>,            //书源注释
        pub lastUpdateTime: i64,                   // 最后更新时间，用于排序
        pub respondTime: i64,                // 响应时间，用于排序
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
                    let list = newRule.split("##");
                    if list[0].contains("|") {
                        newRule = list[0].replace("|", "||");
                        for i in 1..list.len() {
                            newRule += "##";
                            newRule += list[i];
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
            Self::toNewUrl(Some(it)).map(|s| s.replace_with_regex("\n\\s*", "")).unwrap_or_default()
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
        let header_pattern = Pattern::compile("@Header:\\{.+?\\}", Pattern::CASE_INSENSITIVE);
        let mut mather = header_pattern.matcher(&url);
        if mather.find() {
            let header = mather.group().unwrap();
            url = url.replace(header, "");
            map.insert("headers".to_string(), header[8..].to_string());
        }
        let mut urlList: Vec<&str> = url.split("|").collect();
        url = urlList[0].to_string();
        if urlList.len() > 1 {
            map.insert("charset".to_string(), urlList[1].split("=").nth(1).unwrap().to_string());
        }
        let js_pattern = Pattern::compile("\\{\\{.+?\\}\\}", Pattern::CASE_INSENSITIVE);
        mather = js_pattern.matcher(&url);
        let mut jsList: Vec<String> = Vec::new();
        while mather.find() {
            let group = mather.group().unwrap();
            jsList.push(group.to_string());
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
        urlList = url.split("@").collect();
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
