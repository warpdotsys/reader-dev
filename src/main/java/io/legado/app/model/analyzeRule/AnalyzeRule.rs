// package io.legado.app.model.analyzeRule
//
// import com.script.SimpleBindings
// import io.legado.app.constant.AppConst.SCRIPT_ENGINE
// import io.legado.app.constant.AppPattern.JS_PATTERN
// import io.legado.app.data.entities.BaseBook
// import io.legado.app.data.entities.BookChapter
// import io.legado.app.data.entities.Book
// import io.legado.app.data.entities.BookSource
// import io.legado.app.data.entities.BaseSource
// import io.legado.app.help.CacheManager
// import io.legado.app.help.JsExtensions
// import io.legado.app.help.http.CookieStore
// import io.legado.app.utils.*
// import kotlinx.coroutines.runBlocking
// import kotlinx.coroutines.withTimeout
// import org.jsoup.nodes.Entities
// import org.mozilla.javascript.NativeObject
// import java.net.URL
// import java.util.*
// import java.util.regex.Pattern
// import kotlin.collections.HashMap
// import mu.KotlinLogging
// import io.legado.app.model.analyzeRule.RuleDataInterface
// import io.legado.app.model.webBook.WebBook

// private val logger = KotlinLogging.logger {}

/// 解析规则获取结果
// @Suppress("unused", "RegExpRedundantEscape")
pub struct AnalyzeRule {
    pub rule_data: Box<dyn RuleDataInterface>,
    pub source: Option<Box<dyn BaseSource>>,
    pub debug_log: Option<Box<DebugLog>>,

    pub chapter: Option<BookChapter>,
    pub next_chapter_url: Option<String>,
    pub content: Option<Box<dyn Any>>,
    pub base_url: Option<String>,
    pub redirect_url: Option<URL>,
    pub is_json: bool,
    pub is_regex: bool,

    pub analyze_by_x_path: Option<AnalyzeByXPath>,
    pub analyze_by_j_soup: Option<AnalyzeByJSoup>,
    pub analyze_by_j_son_path: Option<AnalyzeByJSonPath>,

    pub object_changed_xp: bool,
    pub object_changed_js: bool,
    pub object_changed_jp: bool,
}

impl AnalyzeRule {
    // override fun getUserNameSpace(): String = ruleData.getUserNameSpace()
    pub fn get_user_name_space(&self) -> String {
        self.rule_data.get_user_name_space()
    }

    // override fun getLogger(): io.legado.app.model.DebugLog? = debugLog
    pub fn get_logger(&self) -> Option<&DebugLog> {
        self.debug_log.as_ref()
    }

    // val book get() = ruleData as? BaseBook
    pub fn book(&self) -> Option<&BaseBook> {
        self.rule_data.as_any().downcast_ref::<BaseBook>()
    }

    // @JvmOverloads
    // fun setContent(content: Any?, baseUrl: String? = null): AnalyzeRule {
    pub fn set_content(&mut self, content: Option<Box<dyn Any>>, base_url: Option<String>) -> &mut Self {
        if content.is_none() {
            panic!("内容不可空（Content cannot be null）");
        }
        self.content = content;
        self.is_json = self.content.as_ref().unwrap().to_string().is_json();
        self.set_base_url(base_url);
        self.object_changed_xp = true;
        self.object_changed_js = true;
        self.object_changed_jp = true;
        self
    }

    // fun setBaseUrl(baseUrl: String?): AnalyzeRule {
    pub fn set_base_url(&mut self, base_url: Option<String>) -> &mut Self {
        if let Some(base_url) = base_url {
            self.base_url = Some(base_url);
        }
        self
    }

    // fun setRedirectUrl(url: String): URL? {
    pub fn set_redirect_url(&mut self, url: String) -> Option<URL> {
        match URL::new(url) {
            Ok(u) => {
                self.redirect_url = Some(u);
            }
            Err(e) => {
                self.log(format!("URL({}) error\n{}", url, e.localized_message()));
            }
        }
        self.redirect_url
    }

    /// 获取XPath解析类
    // private fun getAnalyzeByXPath(o: Any): AnalyzeByXPath {
    fn get_analyze_by_x_path(&mut self, o: Box<dyn Any>) -> AnalyzeByXPath {
        if o != self.content {
            AnalyzeByXPath::new(o)
        } else {
            if self.analyze_by_x_path.is_none() || self.object_changed_xp {
                self.analyze_by_x_path = Some(AnalyzeByXPath::new(self.content.as_ref().unwrap().clone()));
                self.object_changed_xp = false;
            }
            self.analyze_by_x_path.as_ref().unwrap().clone()
        }
    }

    /// 获取JSOUP解析类
    // private fun getAnalyzeByJSoup(o: Any): AnalyzeByJSoup {
    fn get_analyze_by_j_soup(&mut self, o: Box<dyn Any>) -> AnalyzeByJSoup {
        if o != self.content {
            AnalyzeByJSoup::new(o)
        } else {
            if self.analyze_by_j_soup.is_none() || self.object_changed_js {
                self.analyze_by_j_soup = Some(AnalyzeByJSoup::new(self.content.as_ref().unwrap().clone()));
                self.object_changed_js = false;
            }
            self.analyze_by_j_soup.as_ref().unwrap().clone()
        }
    }

    /// 获取JSON解析类
    // private fun getAnalyzeByJSonPath(o: Any): AnalyzeByJSonPath {
    fn get_analyze_by_j_son_path(&mut self, o: Box<dyn Any>) -> AnalyzeByJSonPath {
        if o != self.content {
            AnalyzeByJSonPath::new(o)
        } else {
            if self.analyze_by_j_son_path.is_none() || self.object_changed_jp {
                self.analyze_by_j_son_path = Some(AnalyzeByJSonPath::new(self.content.as_ref().unwrap().clone()));
                self.object_changed_jp = false;
            }
            self.analyze_by_j_son_path.as_ref().unwrap().clone()
        }
    }

    /// 获取文本列表
    // @JvmOverloads
    // fun getStringList(rule: String?, mContent: Any? = null, isUrl: Boolean = false): List<String>? {
    pub fn get_string_list(&mut self, rule: Option<String>, m_content: Option<Box<dyn Any>>, is_url: bool) -> Option<Vec<String>> {
        if rule.is_none() || rule.as_ref().unwrap().is_empty() {
            return None;
        }
        let rule_list = self.split_source_rule(rule, false);
        self.get_string_list_inner(rule_list, m_content, is_url)
    }

    // @JvmOverloads
    // fun getStringList(
    //     ruleList: List<SourceRule>,
    //     mContent: Any? = null,
    //     isUrl: Boolean = false
    // ): List<String>? {
    pub fn get_string_list_inner(
        &mut self,
        rule_list: Vec<SourceRule>,
        m_content: Option<Box<dyn Any>>,
        is_url: bool,
    ) -> Option<Vec<String>> {
        let mut result: Option<Box<dyn Any>> = None;
        let content: Option<Box<dyn Any>> = m_content.or_else(|| self.content.clone());
        if content.is_some() && !rule_list.is_empty() {
            result = content;
            if let Some(native_object) = result.as_ref().unwrap().downcast_ref::<NativeObject>() {
                result = native_object.get(&rule_list[0].rule).map(|v| v.to_string());
            } else {
                for source_rule in rule_list {
                    self.put_rule(source_rule.put_map);
                    source_rule.make_up_rule(result.as_ref());
                    if let Some(r) = result.as_ref() {
                        if !source_rule.rule.is_empty() {
                            result = match source_rule.mode {
                                Mode::Js => self.eval_js(source_rule.rule, Some(r.clone())),
                                Mode::Json => self.get_analyze_by_j_son_path(r.clone()).get_string_list(source_rule.rule),
                                Mode::XPath => self.get_analyze_by_x_path(r.clone()).get_string_list(source_rule.rule),
                                Mode::Default => self.get_analyze_by_j_soup(r.clone()).get_string_list(source_rule.rule),
                                _ => Some(source_rule.rule),
                            };
                        }
                        if !source_rule.replace_regex.is_empty() && result.is_some() && result.as_ref().unwrap().is_list() {
                            let mut new_list: Vec<String> = Vec::new();
                            for item in result.as_ref().unwrap().as_list() {
                                new_list.push(self.replace_regex(item.to_string(), &source_rule));
                            }
                            result = Some(Box::new(new_list));
                        } else if !source_rule.replace_regex.is_empty() {
                            result = Some(Box::new(self.replace_regex(result.as_ref().unwrap().to_string(), &source_rule)));
                        }
                    }
                }
            }
        }
        if result.is_none() {
            return None;
        }
        if result.as_ref().unwrap().is_string() {
            result = Some(Box::new(result.as_ref().unwrap().to_string().split('\n').map(String::from).collect::<Vec<String>>()));
        }
        if is_url {
            let mut url_list: Vec<String> = Vec::new();
            if result.as_ref().unwrap().is_list() {
                for url in result.as_ref().unwrap().as_list() {
                    let absolute_url = NetworkUtils::get_absolute_url(self.redirect_url.as_ref(), url.to_string());
                    if !absolute_url.is_empty() && !url_list.contains(&absolute_url) {
                        url_list.push(absolute_url);
                    }
                }
            }
            return Some(url_list);
        }
        // @Suppress("UNCHECKED_CAST")
        result.and_then(|r| r.as_list_string())
    }

    /// 获取文本
    // @JvmOverloads
    // fun getString(ruleStr: String?, mContent: Any? = null, isUrl: Boolean = false): String {
    pub fn get_string(&mut self, rule_str: Option<String>, m_content: Option<Box<dyn Any>>, is_url: bool) -> String {
        if TextUtils::is_empty(rule_str.as_ref()) {
            return String::new();
        }
        let rule_list = self.split_source_rule(rule_str, false);
        self.get_string_inner(rule_list, m_content, is_url)
    }

    // @JvmOverloads
    // fun getString(
    //     ruleList: List<SourceRule>,
    //     mContent: Any? = null,
    //     isUrl: Boolean = false
    // ): String {
    pub fn get_string_inner(
        &mut self,
        rule_list: Vec<SourceRule>,
        m_content: Option<Box<dyn Any>>,
        is_url: bool,
    ) -> String {
        let mut result: Option<Box<dyn Any>> = None;
        let content: Option<Box<dyn Any>> = m_content.or_else(|| self.content.clone());
        if content.is_some() && !rule_list.is_empty() {
            result = content;
            if let Some(native_object) = result.as_ref().unwrap().downcast_ref::<NativeObject>() {
                result = native_object.get(&rule_list[0].rule).map(|v| v.to_string());
            } else {
                for source_rule in rule_list {
                    self.put_rule(source_rule.put_map);
                    source_rule.make_up_rule(result.as_ref());
                    if let Some(r) = result.as_ref() {
                        if !source_rule.rule.is_blank() || source_rule.replace_regex.is_empty() {
                            result = match source_rule.mode {
                                Mode::Js => self.eval_js(source_rule.rule, Some(r.clone())),
                                Mode::Json => self.get_analyze_by_j_son_path(r.clone()).get_string(source_rule.rule),
                                Mode::XPath => self.get_analyze_by_x_path(r.clone()).get_string(source_rule.rule),
                                Mode::Default => if is_url {
                                    self.get_analyze_by_j_soup(r.clone()).get_string0(source_rule.rule)
                                } else {
                                    self.get_analyze_by_j_soup(r.clone()).get_string(source_rule.rule)
                                },
                                _ => Some(source_rule.rule),
                            };
                        }
                        if result.is_some() && !source_rule.replace_regex.is_empty() {
                            result = Some(Box::new(self.replace_regex(result.as_ref().unwrap().to_string(), &source_rule)));
                        }
                    }
                }
            }
        }
        if result.is_none() {
            result = Some(Box::new(String::new()));
        }
        let str = match Entities::unescape(result.as_ref().unwrap().to_string()) {
            Ok(v) => v,
            Err(e) => {
                self.log(format!("Entities.unescape() error\n{}", e.localized_message()));
                result.as_ref().unwrap().to_string()
            }
        };
        if is_url {
            return if str.is_blank() {
                self.base_url.clone().unwrap_or_else(|| String::new())
            } else {
                NetworkUtils::get_absolute_url(self.redirect_url.as_ref(), str)
            };
        }
        str
    }

    /// 获取Element
    // fun getElement(ruleStr: String): Any? {
    pub fn get_element(&mut self, rule_str: String) -> Option<Box<dyn Any>> {
        if TextUtils::is_empty(Some(&rule_str)) {
            return None;
        }
        let mut result: Option<Box<dyn Any>> = None;
        let content = self.content.clone();
        let rule_list = self.split_source_rule(Some(rule_str), true);
        if content.is_some() && !rule_list.is_empty() {
            result = content;
            for source_rule in rule_list {
                self.put_rule(source_rule.put_map);
                source_rule.make_up_rule(result.as_ref());
                if let Some(r) = result.as_ref() {
                    result = match source_rule.mode {
                        Mode::Regex => AnalyzeByRegex::get_element(
                            r.to_string(),
                            source_rule.rule.split_not_blank("&&"),
                        ),
                        Mode::Js => self.eval_js(source_rule.rule, Some(r.clone())),
                        Mode::Json => self.get_analyze_by_j_son_path(r.clone()).get_object(source_rule.rule),
                        Mode::XPath => self.get_analyze_by_x_path(r.clone()).get_elements(source_rule.rule),
                        _ => self.get_analyze_by_j_soup(r.clone()).get_elements(source_rule.rule),
                    };
                    if !source_rule.replace_regex.is_empty() {
                        result = Some(Box::new(self.replace_regex(result.as_ref().unwrap().to_string(), &source_rule)));
                    }
                }
            }
        }
        result
    }

    /// 获取列表
    // @Suppress("UNCHECKED_CAST")
    // fun getElements(ruleStr: String): List<Any> {
    pub fn get_elements(&mut self, rule_str: String) -> Vec<Box<dyn Any>> {
        let mut result: Option<Box<dyn Any>> = None;
        let content = self.content.clone();
        let rule_list = self.split_source_rule(Some(rule_str), true);
        if content.is_some() && !rule_list.is_empty() {
            result = content;
            for source_rule in rule_list {
                self.put_rule(source_rule.put_map);
                if let Some(r) = result.as_ref() {
                    result = match source_rule.mode {
                        Mode::Regex => AnalyzeByRegex::get_elements(
                            r.to_string(),
                            source_rule.rule.split_not_blank("&&"),
                        ),
                        Mode::Js => self.eval_js(source_rule.rule, Some(r.clone())),
                        Mode::Json => self.get_analyze_by_j_son_path(r.clone()).get_list(source_rule.rule),
                        Mode::XPath => self.get_analyze_by_x_path(r.clone()).get_elements(source_rule.rule),
                        _ => self.get_analyze_by_j_soup(r.clone()).get_elements(source_rule.rule),
                    };
                    if !source_rule.replace_regex.is_empty() {
                        result = Some(Box::new(self.replace_regex(result.as_ref().unwrap().to_string(), &source_rule)));
                    }
                }
            }
        }
        if let Some(r) = result {
            return r.as_list();
        }
        Vec::new()
    }

    /// 保存变量
    // private fun putRule(map: Map<String, String>) {
    fn put_rule(&mut self, map: HashMap<String, String>) {
        for (key, value) in map {
            self.put(key, self.get_string(Some(value), None, false));
        }
    }

    /// 分离put规则
    // private fun splitPutRule(ruleStr: String, putMap: HashMap<String, String>): String {
    fn split_put_rule(&self, rule_str: String, put_map: &mut HashMap<String, String>) -> String {
        let mut v_rule_str = rule_str;
        let mut put_matcher = put_pattern.matcher(v_rule_str);
        while put_matcher.find() {
            v_rule_str = v_rule_str.replace(put_matcher.group(), "");
            GSON::from_json_object::<HashMap<String, String>>(put_matcher.group(1))
                .get_or_none()
                .map(|it| put_map.extend(it));
        }
        v_rule_str
    }

    /// 正则替换
    // private fun replaceRegex(result: String, rule: SourceRule): String {
    fn replace_regex(&self, result: String, rule: &SourceRule) -> String {
        if rule.replace_regex.is_empty() {
            return result;
        }
        let mut v_result = result;
        v_result = if rule.replace_first {
            match (|| {
                let pattern = Pattern::compile(rule.replace_regex);
                let mut matcher = pattern.matcher(v_result);
                if matcher.find() {
                    matcher.group(0).unwrap().replace_first_regex(rule.replace_regex, rule.replacement)
                } else {
                    String::new()
                }
            })() {
                Ok(v) => v,
                Err(_) => v_result.replace_first_str(rule.replace_regex, rule.replacement),
            }
        } else {
            match (|| {
                v_result.replace_regex_all(rule.replace_regex, rule.replacement)
            })() {
                Ok(v) => v,
                Err(_) => v_result.replace_str(rule.replace_regex, rule.replacement),
            }
        };
        v_result
    }

    /// 分解规则生成规则列表
    // fun splitSourceRule(ruleStr: String?, allInOne: Boolean = false): List<SourceRule> {
    pub fn split_source_rule(&mut self, rule_str: Option<String>, all_in_one: bool) -> Vec<SourceRule> {
        if rule_str.is_none() || rule_str.as_ref().unwrap().is_empty() {
            return Vec::new();
        }
        let mut rule_list: Vec<SourceRule> = Vec::new();
        let mut m_mode: Mode = Mode::Default;
        let mut start = 0;
        // 仅首字符为:时为AllInOne，其实:与伪类选择器冲突，建议改成?更合理
        if all_in_one && rule_str.as_ref().unwrap().starts_with(":") {
            m_mode = Mode::Regex;
            self.is_regex = true;
            start = 1;
        } else if self.is_regex {
            m_mode = Mode::Regex;
        }
        let mut tmp: String;
        let mut js_matcher = JS_PATTERN.matcher(rule_str.clone());
        while js_matcher.find() {
            if js_matcher.start() > start {
                tmp = rule_str.as_ref().unwrap()[start..js_matcher.start()].trim().to_string();
                if !tmp.is_empty() {
                    rule_list.push(SourceRule::new(tmp, m_mode));
                }
            }
            rule_list.push(SourceRule::new(js_matcher.group(2).or(js_matcher.group(1)), Mode::Js));
            start = js_matcher.end();
        }

        if rule_str.as_ref().unwrap().len() > start {
            tmp = rule_str.as_ref().unwrap()[start..].trim().to_string();
            if !tmp.is_empty() {
                rule_list.push(SourceRule::new(tmp, m_mode));
            }
        }

        rule_list
    }

    // fun put(key: String, value: String): String {
    pub fn put(&mut self, key: String, value: String) -> String {
        self.chapter.as_ref().and_then(|c| c.put_variable(key, value))
            .or_else(|| self.book().and_then(|b| b.put_variable(key, value)))
            .unwrap_or_else(|| self.rule_data.put_variable(key, value));
        value
    }

    // fun get(key: String): String {
    pub fn get(&self, key: String) -> String {
        match key.as_str() {
            "bookName" => if let Some(book) = self.book() {
                return book.name.clone();
            },
            "title" => if let Some(chapter) = self.chapter.as_ref() {
                return chapter.title.clone();
            },
            _ => {}
        }
        self.chapter.as_ref().and_then(|c| c.get_variable(key.clone()))
            .or_else(|| self.book().and_then(|b| b.get_variable(key.clone())))
            .or_else(|| self.rule_data.get_variable(key))
            .unwrap_or_else(|| String::new())
    }

    /// 执行JS
    // fun evalJS(jsStr: String, result: Any? = null): Any? {
    pub fn eval_js(&mut self, js_str: String, result: Option<Box<dyn Any>>) -> Option<Box<dyn Any>> {
        let mut bindings = SimpleBindings::new();
        bindings.put("java", Box::new(self.clone()));
        bindings.put("cookie", CookieStore::new(self.get_user_name_space()));
        bindings.put("cache", CacheManager::new(self.get_user_name_space()));
        bindings.put("source", self.source.clone());
        bindings.put("book", self.book());
        bindings.put("result", result);
        bindings.put("baseUrl", self.base_url.clone());
        bindings.put("chapter", self.chapter.clone());
        bindings.put("title", self.chapter.as_ref().map(|c| c.title.clone()));
        bindings.put("src", self.content.clone());
        bindings.put("nextChapterUrl", self.next_chapter_url.clone());
        SCRIPT_ENGINE.eval(js_str, bindings)
    }

    // override fun getSource(): BaseSource? {
    pub fn get_source(&self) -> Option<&BaseSource> {
        self.source.as_ref()
    }

    /// js实现跨域访问,不能删
    // override fun ajax(urlStr: String): String? {
    pub fn ajax(&mut self, url_str: String) -> Option<String> {
        // runBlocking {
        //     kotlin.runCatching {
        let analyze_url = AnalyzeUrl::new(url_str, self.source.clone(), self.book(), self.debug_log.clone());
        match analyze_url.get_str_response_await().body {
            Ok(body) => Some(body),
            Err(e) => {
                self.log(format!("ajax({}) error\n{}", url_str, e.stack_trace_to_string()));
                // it.printStackTrace()
                e.msg
            }
        }
        // }
    }

    /// 章节数转数字
    // fun toNumChapter(s: String?): String? {
    pub fn to_num_chapter(&self, s: Option<String>) -> Option<String> {
        s?;
        let mut matcher = title_num_pattern.matcher(s);
        if matcher.find() {
            return Some(format!("{}{}{}", matcher.group(1), StringUtils::string_to_int(matcher.group(2)), matcher.group(3)));
        }
        Some(s)
    }

    /// 更新BookUrl,如果搜索结果有tocUrl也会更新,有些书源bookUrl定期更新,可以在js内调用更新
    // fun refreshBookUrl() {
    pub fn refresh_book_url(&mut self) {
        // runBlocking {
        let book_source = self.source.as_ref().and_then(|s| s.as_any().downcast_ref::<BookSource>());
        let book = self.book().and_then(|b| b.as_any().downcast_ref::<Book>());
        if book_source.is_none() || book.is_none() {
            return;
        }
        let books = WebBook::new(book_source.clone(), false, None, self.get_user_name_space()).search_book(book.name.clone());
        for it in books {
            if it.name == book.name && it.author == book.author {
                book.book_url = it.book_url;
                if !it.toc_url.is_blank() {
                    book.toc_url = it.toc_url;
                }
                return;
            }
        }
        // }
    }

    /// 更新tocUrl,有些书源目录url定期更新,可以在js调用更新
    // fun refreshTocUrl() {
    pub fn refresh_toc_url(&mut self) {
        // runBlocking {
        let book_source = self.source.as_ref().and_then(|s| s.as_any().downcast_ref::<BookSource>());
        let book = self.book().and_then(|b| b.as_any().downcast_ref::<Book>());
        if book_source.is_none() || book.is_none() {
            return;
        }
        WebBook::new(book_source.clone(), false, None, self.get_user_name_space()).get_book_info(book);
        // }
    }

    // fun reGetBook() {
    pub fn re_get_book(&mut self) {
        let book_source = self.source.as_ref().and_then(|s| s.as_any().downcast_ref::<BookSource>());
        let book = self.book().and_then(|b| b.as_any().downcast_ref::<Book>());
        if book_source.is_none() || book.is_none() {
            return;
        }
        // runBlocking {
        //     withTimeout(30 * 60 * 1000L) {
        let refreshed_book = WebBook::new(book_source.clone(), false, None, self.get_user_name_space())
            .precise_search(book.name.clone(), book.author.clone())
            .get_or_throw();
        book.book_url = refreshed_book.book_url;
        for (key, value) in refreshed_book.variable_map {
            book.put_variable(key, value);
        }
        WebBook::new(book_source.clone(), false, None, self.get_user_name_space()).get_book_info(book, false);
        //     }
        // }
    }
}

/// 规则类
// inner class SourceRule internal constructor(
//     ruleStr: String,
//     internal var mode: Mode = Mode.Default
// ) {
pub struct SourceRule {
    pub rule: String,
    pub mode: Mode,
    pub replace_regex: String,
    pub replacement: String,
    pub replace_first: bool,
    pub put_map: HashMap<String, String>,
    rule_param: Vec<String>,
    rule_type: Vec<i32>,
    get_rule_type: i32,
    js_rule_type: i32,
    default_rule_type: i32,
}

impl SourceRule {
    // init {
    pub fn new(rule_str: String, mode: Mode) -> SourceRule {
        let mut s = SourceRule {
            rule: String::new(),
            mode,
            replace_regex: String::new(),
            replacement: String::new(),
            replace_first: false,
            put_map: HashMap::new(),
            rule_param: Vec::new(),
            rule_type: Vec::new(),
            get_rule_type: -2,
            js_rule_type: -1,
            default_rule_type: 0,
        };
        s.rule = if s.mode == Mode::Js || s.mode == Mode::Regex {
            rule_str.clone()
        } else if rule_str.starts_with_ignore_case("@CSS:") {
            s.mode = Mode::Default;
            rule_str.clone()
        } else if rule_str.starts_with("@@") {
            s.mode = Mode::Default;
            rule_str[2..].to_string()
        } else if rule_str.starts_with_ignore_case("@XPath:") {
            s.mode = Mode::XPath;
            rule_str[7..].to_string()
        } else if rule_str.starts_with_ignore_case("@Json:") {
            s.mode = Mode::Json;
            rule_str[6..].to_string()
        } else if is_json_content || rule_str.starts_with("$.") || rule_str.starts_with("$[") {
            s.mode = Mode::Json;
            rule_str.clone()
        } else if rule_str.starts_with("/") {
            // XPath特征很明显,无需配置单独的识别标头
            s.mode = Mode::XPath;
            rule_str.clone()
        } else {
            rule_str.clone()
        };
        // 分离put
        s.rule = s.split_put_rule(s.rule.clone(), &mut s.put_map);
        // @get,{{ }}, 拆分
        let mut start = 0;
        let mut tmp: String;
        let mut eval_matcher = eval_pattern.matcher(s.rule.clone());

        if eval_matcher.find() {
            tmp = s.rule[start..eval_matcher.start()].to_string();
            if s.mode != Mode::Js && s.mode != Mode::Regex && (eval_matcher.start() == 0 || !tmp.contains("##")) {
                s.mode = Mode::Regex;
            }
            loop {
                if eval_matcher.start() > start {
                    tmp = s.rule[start..eval_matcher.start()].to_string();
                    s.split_regex(tmp);
                }
                tmp = eval_matcher.group();
                if tmp.starts_with_ignore_case("@get:") {
                    s.rule_type.push(s.get_rule_type);
                    s.rule_param.push(tmp[6..tmp.len() - 1].to_string());
                } else if tmp.starts_with("{{") {
                    s.rule_type.push(s.js_rule_type);
                    s.rule_param.push(tmp[2..tmp.len() - 2].to_string());
                } else {
                    s.split_regex(tmp);
                }
                start = eval_matcher.end();
                if !eval_matcher.find() {
                    break;
                }
            }
        }
        if s.rule.len() > start {
            tmp = s.rule[start..].to_string();
            s.split_regex(tmp);
        }
        s
    }

    /// 拆分\$\d{1,2}
    // private fun splitRegex(ruleStr: String) {
    fn split_regex(&mut self, rule_str: String) {
        let mut start = 0;
        let mut tmp: String;
        let rule_str_array: Vec<&str> = rule_str.split("##").collect();
        let mut regex_matcher = regex_pattern.matcher(rule_str_array[0].to_string());

        if regex_matcher.find() {
            if self.mode != Mode::Js && self.mode != Mode::Regex {
                self.mode = Mode::Regex;
            }
            loop {
                if regex_matcher.start() > start {
                    tmp = rule_str[start..regex_matcher.start()].to_string();
                    self.rule_type.push(self.default_rule_type);
                    self.rule_param.push(tmp);
                }
                tmp = regex_matcher.group();
                self.rule_type.push(tmp[1..].to_string().parse::<i32>().unwrap());
                self.rule_param.push(tmp);
                start = regex_matcher.end();
                if !regex_matcher.find() {
                    break;
                }
            }
        }
        if rule_str.len() > start {
            tmp = rule_str[start..].to_string();
            self.rule_type.push(self.default_rule_type);
            self.rule_param.push(tmp);
        }
    }

    /// 替换@get,{{ }}
    // fun makeUpRule(result: Any?) {
    pub fn make_up_rule(&mut self, result: Option<Box<dyn Any>>) {
        let mut info_val = String::new();
        if !self.rule_param.is_empty() {
            let mut index = self.rule_param.len();
            while index > 0 {
                index -= 1;
                let reg_type = self.rule_type[index];
                if reg_type > self.default_rule_type {
                    // @Suppress("UNCHECKED_CAST")
                    if let Some(list) = result.as_ref().and_then(|r| r.as_list_string_ref()) {
                        if list.len() > reg_type as usize {
                            if let Some(item) = &list[reg_type as usize] {
                                info_val.insert_str(0, item);
                            }
                        }
                    } else {
                        info_val.insert_str(0, &self.rule_param[index]);
                    }
                } else if reg_type == self.js_rule_type {
                    if self.is_rule(&self.rule_param[index]) {
                        let v = self.get_string(Some(self.rule_param[index].clone()), None, false);
                        info_val.insert_str(0, &v);
                    } else {
                        let js_eval = self.eval_js(self.rule_param[index].clone(), result.clone());
                        match js_eval {
                            None => {}
                            Some(js_eval) if js_eval.is_string() => info_val.insert_str(0, &js_eval.to_string()),
                            Some(js_eval) if js_eval.is_double() && js_eval.as_double() % 1.0 == 0.0 => info_val.insert_str(
                                0,
                                &format!("{:.0}", js_eval.as_double()),
                            ),
                            Some(js_eval) => info_val.insert_str(0, &js_eval.to_string()),
                        }
                    }
                } else if reg_type == self.get_rule_type {
                    info_val.insert_str(0, &self.get(&self.rule_param[index]));
                } else {
                    info_val.insert_str(0, &self.rule_param[index]);
                }
            }
            self.rule = info_val;
        }
        // 分离正则表达式
        let rule_str_s: Vec<&str> = self.rule.split("##").collect();
        self.rule = rule_str_s[0].trim().to_string();
        if rule_str_s.len() > 1 {
            self.replace_regex = rule_str_s[1].to_string();
        }
        if rule_str_s.len() > 2 {
            self.replacement = rule_str_s[2].to_string();
        }
        if rule_str_s.len() > 3 {
            self.replace_first = true;
        }
    }

    // private fun isRule(ruleStr: String): Boolean {
    fn is_rule(&self, rule_str: &String) -> bool {
        rule_str.starts_with('@') // js首个字符不可能是@，除非是装饰器，所以@开头规定为规则
            || rule_str.starts_with("$.")
            || rule_str.starts_with("$[")
            || rule_str.starts_with("//")
    }

    // private fun splitPutRule(ruleStr: String, putMap: HashMap<String, String>): String {
    fn split_put_rule(&self, rule_str: String, put_map: &mut HashMap<String, String>) -> String {
        let mut v_rule_str = rule_str;
        let mut put_matcher = put_pattern.matcher(v_rule_str);
        while put_matcher.find() {
            v_rule_str = v_rule_str.replace(&put_matcher.group(), "");
            if let Some(it) = GSON::from_json_object::<HashMap<String, String>>(put_matcher.group(1)).get_or_none() {
                put_map.extend(it);
            }
        }
        v_rule_str
    }
}

// enum class Mode {
//     XPath, Json, Default, Js, Regex
// }
#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    XPath,
    Json,
    Default,
    Js,
    Regex,
}

impl AnalyzeRule {
    // companion object {
    //     private val putPattern = Pattern.compile("@put:(\\{[^}]+?\\})", Pattern.CASE_INSENSITIVE)
    //     private val evalPattern =
    //         Pattern.compile("@get:\\{[^}]+?\\}|\\{\\{[\\w\\W]*?\\}\\}", Pattern.CASE_INSENSITIVE)
    //     private val regexPattern = Pattern.compile("\\$\\d{1,2}")
    //     private val titleNumPattern = Pattern.compile("(第)(.+?)(章)")
    // }
    pub const PUT_PATTERN: Pattern = Pattern::compile("@put:(\\{[^}]+?\\})", Pattern::CASE_INSENSITIVE);
    pub const EVAL_PATTERN: Pattern = Pattern::compile("@get:\\{[^}]+?\\}|\\{\\{[\\w\\W]*?\\}\\}", Pattern::CASE_INSENSITIVE);
    pub const REGEX_PATTERN: Pattern = Pattern::compile("\\$\\d{1,2}");
    pub const TITLE_NUM_PATTERN: Pattern = Pattern::compile("(第)(.+?)(章)");
}
