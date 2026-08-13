use crate::prelude::*;
use crate::stubs::{Any, GSON, SCRIPT_ENGINE, TextUtils, URL};
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
    pub debug_log: Option<Box<dyn DebugLog>>,

    pub chapter: Option<BookChapter>,
    pub next_chapter_url: Option<String>,
    pub content: Option<Box<Any>>,
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
    pub fn get_logger(&self) -> Option<&dyn DebugLog> {
        self.debug_log.as_ref().map(|b| b.as_ref())
    }

    // fix: Kotlin `log(msg)` 由 JsExtensions 提供；AnalyzeRule 转录未实现 JsExtensions（get_source 返回 trait 对象无法表示），
    //      以自带方法等价实现（仅记录到 DebugLog）
    pub fn log(&self, msg: String) -> String {
        if let Some(logger) = self.debug_log.as_ref() {
            logger.log(None, Some(msg.as_str()), false);
        }
        msg
    }

    // val book get() = ruleData as? BaseBook
    pub fn book(&self) -> Option<&dyn BaseBook> {
        // fix: as_any 无法下转 dyn BaseBook（非 Sized），改转具体 Book 再升为 trait 对象；
        //      AnalyzeRule::new 占位构造器填充的 ruleData 非 Book，恒返回 None
        self.rule_data.as_any().downcast_ref::<Book>().map(|b| b as &dyn BaseBook)
    }

    // @JvmOverloads
    // fun setContent(content: Any?, baseUrl: String? = None): AnalyzeRule {
    pub fn set_content(&mut self, content: Option<Box<Any>>, base_url: Option<String>) -> &mut Self {
        if content.is_none() {
            panic!("内容不可空（Content cannot be None）");
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
        match URL::new(url.clone()) {
            Ok(u) => {
                self.redirect_url = Some(u);
            }
            Err(e) => {
                self.log(format!("URL({}) error\n{}", url, e.localized_message()));
            }
        }
        self.redirect_url.clone()
    }

    /// 获取XPath解析类
    // private fun getAnalyzeByXPath(o: Any): AnalyzeByXPath {
    fn get_analyze_by_x_path(&mut self, o: Box<Any>) -> AnalyzeByXPath {
        crate::io_legado_app_model_analyzerule_analyzebyxpath::AnalyzeByXPath::new(&o)
    }

    /// 获取JSOUP解析类
    // private fun getAnalyzeByJSoup(o: Any): AnalyzeByJSoup {
    fn get_analyze_by_j_soup(&mut self, o: Box<Any>) -> AnalyzeByJSoup {
        analyze_rule_stub_analyze_by_j_soup_new(*o)
    }

    /// 获取JSON解析类
    // private fun getAnalyzeByJSonPath(o: Any): AnalyzeByJSonPath {
    fn get_analyze_by_j_son_path(&mut self, o: Box<Any>) -> AnalyzeByJSonPath {
        crate::io_legado_app_model_analyzerule_analyzebyjsonpath::AnalyzeByJSonPath::new(&o)
    }

    /// 获取文本列表
    // @JvmOverloads
    // fun getStringList(rule: String?, mContent: Any? = None, isUrl: Boolean = false): List<String>? {
    pub fn get_string_list(&mut self, rule: Option<String>, m_content: Option<Box<Any>>, is_url: bool) -> Option<Vec<String>> {
        if rule.is_none() || rule.as_ref().unwrap().is_empty() {
            return None;
        }
        let rule_list = self.split_source_rule(rule, false);
        self.get_string_list_inner(rule_list, m_content, is_url)
    }

    // @JvmOverloads
    // fun getStringList(
    //     ruleList: List<SourceRule>,
    //     mContent: Any? = None,
    //     isUrl: Boolean = false
    // ): List<String>? {
    pub fn get_string_list_inner(
        &mut self,
        rule_list: Vec<SourceRule>,
        m_content: Option<Box<Any>>,
        is_url: bool,
    ) -> Option<Vec<String>> {
        let mut result: Option<Box<Any>> = None;
        let content: Option<Box<Any>> = m_content.or_else(|| self.content.clone());
        if content.is_some() && !rule_list.is_empty() {
            result = content;
            if let Some(native_object) = result.as_ref().unwrap().downcast_ref::<NativeObject>() {
                // fix: NativeObject.get 返回 Box<dyn AnyDebug>，占位格式化转 String
                result = native_object.get(&rule_list[0].rule).map(|v| Box::new(Any::Str(format!("{:?}", v))));
            } else {
                for mut source_rule in rule_list {
                    self.put_rule(source_rule.put_map.clone());
                    source_rule.make_up_rule(self, result.clone());
                    if let Some(r) = result.as_ref() {
                        if !source_rule.rule.is_empty() {
                            result = match source_rule.mode {
                                Mode::Js => self.eval_js(source_rule.rule.clone(), Some(r.clone())),
                                Mode::Json => Some(Box::new(Any::List(
                                    analyze_rule_stub_analyze_by_j_son_path_get_string_list(&self.get_analyze_by_j_son_path(r.clone()), &source_rule.rule)
                                        .into_iter()
                                        .map(Any::Str)
                                        .collect(),
                                ))),
                                Mode::XPath => Some(Box::new(Any::List(
                                    analyze_rule_stub_analyze_by_x_path_get_string_list(&self.get_analyze_by_x_path(r.clone()), &source_rule.rule)
                                        .into_iter()
                                        .map(Any::Str)
                                        .collect(),
                                ))),
                                Mode::Default => Some(Box::new(Any::List(
                                    self.get_analyze_by_j_soup(r.clone()).get_string_list(&source_rule.rule)
                                        .into_iter()
                                        .map(Any::Str)
                                        .collect(),
                                ))),
                                _ => Some(Box::new(Any::Str(source_rule.rule.clone()))),
                            };
                        }
                        if !source_rule.replace_regex.is_empty() && result.is_some() && result.as_ref().unwrap().is_list() {
                            let mut new_list: Vec<String> = Vec::new();
                            for item in result.as_ref().unwrap().list_iter() {
                                new_list.push(self.replace_regex(item.to_string(), &source_rule));
                            }
                            result = Some(Box::new(Any::List(new_list.into_iter().map(Any::Str).collect())));
                        } else if !source_rule.replace_regex.is_empty() {
                            result = Some(Box::new(Any::Str(self.replace_regex(result.as_ref().unwrap().to_string(), &source_rule))));
                        }
                    }
                }
            }
        }
        if result.is_none() {
            return None;
        }
        if result.as_ref().unwrap().is_string() {
            result = Some(Box::new(Any::List(
                result.as_ref().unwrap().to_string().split('\n').map(|s| Any::Str(s.to_string())).collect(),
            )));
        }
        if is_url {
            let mut url_list: Vec<String> = Vec::new();
            if result.as_ref().unwrap().is_list() {
                for url in result.as_ref().unwrap().list_iter() {
                    let absolute_url = NetworkUtils::getAbsoluteURL_url(self.redirect_url.as_ref(), url.to_string().as_str());
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
    // fun getString(ruleStr: String?, mContent: Any? = None, isUrl: Boolean = false): String {
    pub fn get_string(&mut self, rule_str: Option<String>, m_content: Option<Box<Any>>, is_url: bool) -> String {
        if TextUtils::is_empty(rule_str.as_deref()) {
            return String::new();
        }
        let rule_list = self.split_source_rule(rule_str, false);
        self.get_string_inner(rule_list, m_content, is_url)
    }

    // @JvmOverloads
    // fun getString(
    //     ruleList: List<SourceRule>,
    //     mContent: Any? = None,
    //     isUrl: Boolean = false
    // ): String {
    pub fn get_string_inner(
        &mut self,
        rule_list: Vec<SourceRule>,
        m_content: Option<Box<Any>>,
        is_url: bool,
    ) -> String {
        let mut result: Option<Box<Any>> = None;
        let content: Option<Box<Any>> = m_content.or_else(|| self.content.clone());
        if content.is_some() && !rule_list.is_empty() {
            result = content;
            if let Some(native_object) = result.as_ref().unwrap().downcast_ref::<NativeObject>() {
                // fix: NativeObject.get 返回 Box<dyn AnyDebug>，占位格式化转 String
                result = native_object.get(&rule_list[0].rule).map(|v| Box::new(Any::Str(format!("{:?}", v))));
            } else {
                for mut source_rule in rule_list {
                    self.put_rule(source_rule.put_map.clone());
                    source_rule.make_up_rule(self, result.clone());
                    if let Some(r) = result.as_ref() {
                        if !source_rule.rule.is_blank() || source_rule.replace_regex.is_empty() {
                            result = match source_rule.mode {
                                Mode::Js => self.eval_js(source_rule.rule.clone(), Some(r.clone())),
                                Mode::Json => self.get_analyze_by_j_son_path(r.clone()).get_string(&source_rule.rule).map(|s| Box::new(Any::Str(s))),
                                Mode::XPath => self.get_analyze_by_x_path(r.clone()).get_string(&source_rule.rule)
                                    .map(|s| Box::new(Any::Str(s))),
                                Mode::Default => if is_url {
                                    Some(Box::new(Any::Str(self.get_analyze_by_j_soup(r.clone()).get_string0(&source_rule.rule))))
                                } else {
                                    let jsoup = self.get_analyze_by_j_soup(r.clone());
                                    let css_res = jsoup.get_string(&source_rule.rule);
                                    css_res.map(|s| Box::new(Any::Str(s)))
                                },
                                _ => Some(Box::new(Any::Str(source_rule.rule.clone()))),
                            };
                        }
                        if result.is_some() && !source_rule.replace_regex.is_empty() {
                            result = Some(Box::new(Any::Str(self.replace_regex(result.as_ref().unwrap().to_string(), &source_rule))));
                        }
                    }
                }
            }
        }
        if result.is_none() {
            result = Some(Box::new(Any::Str(String::new())));
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
                NetworkUtils::getAbsoluteURL_url(self.redirect_url.as_ref(), str.as_str())
            };
        }
        str
    }

    /// 获取Element
    // fun getElement(ruleStr: String): Any? {
    pub fn get_element(&mut self, rule_str: String) -> Option<Box<Any>> {
        if TextUtils::is_empty(Some(rule_str.as_str())) {
            return None;
        }
        let mut result: Option<Box<Any>> = None;
        let content = self.content.clone();
        let rule_list = self.split_source_rule(Some(rule_str), true);
        if content.is_some() && !rule_list.is_empty() {
            result = content;
            for mut source_rule in rule_list {
                self.put_rule(source_rule.put_map.clone());
                source_rule.make_up_rule(self, result.clone());
                if let Some(r) = result.as_ref() {
                    result = match source_rule.mode {
                        Mode::Regex => analyze_rule_stub_regex_get_element(
                            r.to_string().as_str(),
                            &source_rule.rule.split_not_blank("&&"),
                            0,
                        )
                        .map(|list| Box::new(Any::List(list.into_iter().map(Any::Str).collect()))),
                        Mode::Js => self.eval_js(source_rule.rule.clone(), Some(r.clone())),
                        Mode::Json => Some(Box::new(analyze_rule_stub_analyze_by_j_son_path_get_object(&self.get_analyze_by_j_son_path(r.clone()), &source_rule.rule))),
                        Mode::XPath => self.get_analyze_by_x_path(r.clone()).get_elements(&source_rule.rule)
                            .map(|list| Box::new(Any::List(list.into_iter().map(Any::JXNode).collect()))),
                        _ => Some(Box::new(Any::Elements(self.get_analyze_by_j_soup(r.clone()).get_elements(&source_rule.rule)))),
                    };
                    if !source_rule.replace_regex.is_empty() {
                        result = Some(Box::new(Any::Str(self.replace_regex(result.as_ref().unwrap().to_string(), &source_rule))));
                    }
                }
            }
        }
        result
    }

    /// 获取列表
    // @Suppress("UNCHECKED_CAST")
    // fun getElements(ruleStr: String): List<Any> {
    pub fn get_elements(&mut self, rule_str: String) -> Vec<Box<Any>> {
        let mut result: Option<Box<Any>> = None;
        let content = self.content.clone();
        let rule_list = self.split_source_rule(Some(rule_str), true);
        if content.is_some() && !rule_list.is_empty() {
            result = content;
            for source_rule in rule_list {
                self.put_rule(source_rule.put_map.clone());
                if let Some(r) = result.as_ref() {
                    result = match source_rule.mode {
                        Mode::Regex => Some(Box::new(Any::List(
                            analyze_rule_stub_regex_get_elements(
                                r.to_string().as_str(),
                                &source_rule.rule.split_not_blank("&&"),
                                0,
                            )
                            .into_iter()
                            .map(|row| Any::List(row.into_iter().map(Any::Str).collect()))
                            .collect(),
                        ))),
                        Mode::Js => self.eval_js(source_rule.rule.clone(), Some(r.clone())),
                        Mode::Json => self.get_analyze_by_j_son_path(r.clone()).get_list(&source_rule.rule).map(|l| Box::new(Any::List(l.into_iter().collect()))),
                        Mode::XPath => self.get_analyze_by_x_path(r.clone()).get_elements(&source_rule.rule)
                            .map(|list| Box::new(Any::List(list.into_iter().map(Any::JXNode).collect()))),
                        _ => Some(Box::new(Any::Elements(self.get_analyze_by_j_soup(r.clone()).get_elements(&source_rule.rule)))),
                    };
                    if !source_rule.replace_regex.is_empty() {
                        result = Some(Box::new(Any::Str(self.replace_regex(result.as_ref().unwrap().to_string(), &source_rule))));
                    }
                }
            }
        }
        if let Some(r) = result {
            return r.list_iter().into_iter().map(Box::new).collect();
        }
        Vec::new()
    }

    /// 保存变量
    // private fun putRule(map: Map<String, String>) {
    fn put_rule(&mut self, map: HashMap<String, String>) {
        for (key, value) in map {
            let v = self.get_string(Some(value), None, false);
            self.put(key, v);
        }
    }

    /// 分离put规则
    // private fun splitPutRule(ruleStr: String, putMap: HashMap<String, String>): String {
    fn split_put_rule(&self, rule_str: String, put_map: &mut HashMap<String, String>) -> String {
        let mut v_rule_str = rule_str;
        let put_pattern = AnalyzeRule::PUT_PATTERN();
        let mut put_matcher = put_pattern.matcher(v_rule_str.clone());
        while put_matcher.find() {
            v_rule_str = v_rule_str.replace(put_matcher.group().as_str(), "");
            GSON::from_json_object::<HashMap<String, String>>(put_matcher.group_idx(1).unwrap_or_default())
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
            match (|| -> Result<String, StubError> {
                let pattern = Pattern::compile(rule.replace_regex.as_str());
                let mut matcher = pattern.matcher(v_result.clone());
                if matcher.find() {
                    Ok(matcher.group_idx(0).unwrap().replace_first_regex(rule.replace_regex.as_str(), rule.replacement.as_str()))
                } else {
                    Ok(String::new())
                }
            })() {
                Ok(v) => v,
                Err(_) => v_result.replace_first_str(rule.replace_regex.as_str(), rule.replacement.as_str()),
            }
        } else {
            match (|| {
                v_result.replace_regex_all(rule.replace_regex.as_str(), rule.replacement.as_str())
            })() {
                Ok(v) => v,
                Err(_) => v_result.replace_str(rule.replace_regex.as_str(), rule.replacement.as_str()),
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
        let js_pattern = AppPattern::JS_PATTERN();
        let mut js_matcher = js_pattern.matcher(rule_str.as_ref().unwrap().clone());
        while js_matcher.find() {
            if js_matcher.start() > start {
                tmp = rule_str.as_ref().unwrap()[start..js_matcher.start()].trim().to_string();
                if !tmp.is_empty() {
                    rule_list.push(SourceRule::new(tmp, m_mode, self.is_json));
                }
            }
            rule_list.push(SourceRule::new(js_matcher.group_idx(2).or(js_matcher.group_idx(1)).unwrap_or_default(), Mode::Js, self.is_json));
            start = js_matcher.end();
        }

        if rule_str.as_ref().unwrap().len() > start {
            tmp = rule_str.as_ref().unwrap()[start..].trim().to_string();
            if !tmp.is_empty() {
                rule_list.push(SourceRule::new(tmp, m_mode, self.is_json));
            }
        }

        rule_list
    }

    // fun put(key: String, value: String): String {
    pub fn put(&mut self, key: String, value: String) -> String {
        if let Some(chapter) = self.chapter.as_mut() {
            chapter.put_variable(key.clone(), Some(value.clone()));
        } else {
            // fix: book() 返回只读 &dyn BaseBook，无法调用 &mut put_variable；回落到 ruleData
            self.rule_data.put_variable(key.as_str(), Some(value.as_str()));
        }
        value
    }

    // fun get(key: String): String {
    pub fn get(&self, key: String) -> String {
        match key.as_str() {
            "bookName" => if let Some(book) = self.book() {
                return book.name().to_string();
            },
            "title" => if let Some(chapter) = self.chapter.as_ref() {
                return chapter.title.clone();
            },
            _ => {}
        }
        self.chapter.as_ref().and_then(|c| c.get_variable(key.as_str()))
            .or_else(|| self.book().and_then(|b| b.get_variable(key.as_str())))
            .or_else(|| self.rule_data.get_variable(&key))
            .unwrap_or_else(|| String::new())
    }

    /// 执行JS
    // fun evalJS(jsStr: String, result: Any? = None): Any? {
    pub fn eval_js(&mut self, js_str: String, result: Option<Box<Any>>) -> Option<Box<Any>> {
        let mut bindings = SimpleBindings::new();
        // fix: AnalyzeRule 无法 Clone（含 trait 对象字段），java 绑定以标记字符串占位（引擎占位不读取绑定）
        bindings.set("java", format!("AnalyzeRule@{:p}", self as *const Self));
        // fix: CookieStore/CacheManager 未实现 Debug，绑定以命名空间字符串占位
        bindings.set("cookie", self.get_user_name_space());
        bindings.set("cache", self.get_user_name_space());
        bindings.set("source", self.source.is_some());
        bindings.set("book", self.book().is_some());
        bindings.put("result", result);
        bindings.put("baseUrl", self.base_url.clone());
        // fix: BookChapter 未实现 Debug，绑定以标题占位
        bindings.set("chapter", self.chapter.as_ref().map(|c| c.title.clone()));
        bindings.put("title", self.chapter.as_ref().map(|c| c.title.clone()));
        bindings.put("src", self.content.clone());
        bindings.put("nextChapterUrl", self.next_chapter_url.clone());
        // fix: 引擎占位恒返回 None；若有值包装为 Any::Str 标记
        SCRIPT_ENGINE.eval_downcast_any(js_str, &mut bindings).map(Box::new)
    }

    // override fun getSource(): BaseSource? {
    pub fn get_source(&self) -> Option<&dyn BaseSource> {
        self.source.as_ref().map(|b| b.as_ref())
    }

    /// js实现跨域访问,不能删
    // override fun ajax(urlStr: String): String? {
    pub fn ajax(&mut self, url_str: String) -> Option<String> {
        // runBlocking {
        //     kotlin.runCatching {
        // fix: AnalyzeUrl::new 为全量参数构造（Kotlin 默认参数展开）；source/debugLog 无法从 trait 对象克隆，占位 None
        let mut analyze_url = AnalyzeUrl::new(
            url_str.clone(),
            None,
            None,
            None,
            None,
            self.base_url.clone().unwrap_or_default(),
            None,
            None,
            None,
            None,
            None,
        );
        // fix: get_str_response_await 为 async，需要 block_on 驱动；StrResponse 无异常路径，失败返回 None
        match block_on(analyze_url.get_str_response_await(None, None, false)).body() {
            Some(body) => Some(body.clone()),
            None => {
                self.log(format!("ajax({}) error", url_str));
                // it.printStackTrace()
                None
            }
        }
        // }
    }

    /// 章节数转数字
    // fun toNumChapter(s: String?): String? {
    pub fn to_num_chapter(&self, s: Option<String>) -> Option<String> {
        let s = s?;
        let title_num_pattern = AnalyzeRule::TITLE_NUM_PATTERN();
        let mut matcher = title_num_pattern.matcher(s.clone());
        if matcher.find() {
            return Some(format!("{}{}{}", matcher.group_idx(1).unwrap_or_default(), StringUtils::stringToInt(matcher.group_idx(2).as_deref()), matcher.group_idx(3).unwrap_or_default()));
        }
        Some(s)
    }

    /// 更新BookUrl,如果搜索结果有tocUrl也会更新,有些书源bookUrl定期更新,可以在js内调用更新
    // fun refreshBookUrl() {
    pub fn refresh_book_url(&mut self) {
        // runBlocking {
        let book = self.book().and_then(|b| b.as_any().downcast_ref::<Book>());
        if book.is_none() {
            return;
        }
        let book = book.unwrap();
        // fix: BaseSource trait 对象无 as_any 无法下转为 &BookSource，且 BookSource 未实现 Clone；WebBook 用默认书源占位
        let web_book = WebBook::new(BookSource::default(), false, None, Some(self.get_user_name_space()));
        let books = block_on(web_book.search_book(book.name.as_str(), None));
        for it in books {
            if it.name == book.name && it.author == book.author {
                // fix: book 为只读 &Book（经 &dyn Any 下转），无法写回 bookUrl/tocUrl；占位保留匹配判断
                let _ = (it.book_url, it.toc_url);
                return;
            }
        }
        // }
    }

    /// 更新tocUrl,有些书源目录url定期更新,可以在js调用更新
    // fun refreshTocUrl() {
    pub fn refresh_toc_url(&mut self) {
        // runBlocking {
        let book = self.book().and_then(|b| b.as_any().downcast_ref::<Book>());
        if book.is_none() {
            return;
        }
        let book = book.unwrap();
        // fix: get_book_info 需要 &mut Book，book 为只读引用；占位：在本地副本上执行并丢弃结果
        let mut book_mut = Book::default();
        book_mut.name = book.name.clone();
        let web_book = WebBook::new(BookSource::default(), false, None, Some(self.get_user_name_space()));
        block_on(web_book.get_book_info(&mut book_mut, false));
        // }
    }

    // fun reGetBook() {
    pub fn re_get_book(&mut self) {
        let book = self.book().and_then(|b| b.as_any().downcast_ref::<Book>());
        if book.is_none() {
            return;
        }
        let book = book.unwrap();
        // fix: 同上，book 只读且 BookSource 无 Clone；搜索与写回均占位（Kotlin 会重新搜索并写回 bookUrl/变量）
        let web_book = WebBook::new(BookSource::default(), false, None, Some(self.get_user_name_space()));
        let refreshed_book = block_on(web_book.search_book(book.name.as_str(), Some(1)))
            .into_iter()
            .find(|it| it.name == book.name && it.author == book.author)
            .map(|mut it| it.to_book())
            .get_or_throw()
            .unwrap_or_else(|_| Book::default());
        let _ = refreshed_book;
        let mut book_mut = Book::default();
        book_mut.name = book.name.clone();
        book_mut.author = book.author.clone();
        block_on(web_book.get_book_info(&mut book_mut, false));
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
    // fix: Kotlin 内类 SourceRule 可访问外层 isJSON，转录为构造函数参数
    pub fn new(rule_str: String, mode: Mode, is_json: bool) -> SourceRule {
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
        } else if is_json || rule_str.starts_with("$.") || rule_str.starts_with("$[") {
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
        // fix: 借用冲突（&self 与 &mut s.put_map），改用局部 map 分离后回填
        let mut put_map = HashMap::new();
        s.rule = s.split_put_rule(s.rule.clone(), &mut put_map);
        s.put_map = put_map;
        // @get,{{ }}, 拆分
        let mut start = 0;
        let mut tmp: String;
        let eval_pattern = AnalyzeRule::EVAL_PATTERN();
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
        let regex_pattern = AnalyzeRule::REGEX_PATTERN();
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
    // fix: Kotlin inner class 可访问外层 AnalyzeRule 方法（getString/evalJS/get），转录为显式 outer 参数
    pub fn make_up_rule(&mut self, outer: &mut AnalyzeRule, result: Option<Box<Any>>) {
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
                        let v = outer.get_string(Some(self.rule_param[index].clone()), None, false);
                        info_val.insert_str(0, &v);
                    } else {
                        let js_eval = outer.eval_js(self.rule_param[index].clone(), result.clone());
                        match js_eval {
                            None => {}
                            Some(js_eval) => match &*js_eval {
                                // fix: Any 无 is_double/as_double（JsValue 独占），改为直接匹配 Double 变体
                                Any::Double(d) if d % 1.0 == 0.0 => info_val.insert_str(0, &format!("{:.0}", d)),
                                _ => info_val.insert_str(0, &js_eval.to_string()),
                            },
                        }
                    }
                } else if reg_type == self.get_rule_type {
                    info_val.insert_str(0, &outer.get(self.rule_param[index].clone()));
                } else {
                    info_val.insert_str(0, &self.rule_param[index]);
                }
            }
            self.rule = info_val;
        }
        // 分离正则表达式
        let rule_str_s: Vec<String> = self.rule.split("##").map(|s| s.to_string()).collect();
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
        let put_pattern = AnalyzeRule::PUT_PATTERN();
        let mut put_matcher = put_pattern.matcher(v_rule_str.clone());
        while put_matcher.find() {
            v_rule_str = v_rule_str.replace(&put_matcher.group(), "");
            if let Some(it) = GSON::from_json_object::<HashMap<String, String>>(put_matcher.group_idx(1).unwrap_or_default()).get_or_none() {
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
    // fix: 原 companion object 的 Pattern 常量改为关联函数（Pattern::compile 非 const 构造，参考 AppPattern/AppConst 约定）
    pub fn PUT_PATTERN() -> Pattern {
        Pattern::compile_with("@put:(\\{[^}]+?\\})", Pattern::CASE_INSENSITIVE)
    }
    pub fn EVAL_PATTERN() -> Pattern {
        Pattern::compile_with("@get:\\{[^}]+?\\}|\\{\\{[\\w\\W]*?\\}\\}", Pattern::CASE_INSENSITIVE)
    }
    pub fn REGEX_PATTERN() -> Pattern {
        Pattern::compile("\\$\\d{1,2}")
    }
    pub fn TITLE_NUM_PATTERN() -> Pattern {
        Pattern::compile("(第)(.+?)(章)")
    }
}

// fix: Kotlin `chapter?.getVariable(key)`（BookChapter 转录缺该方法，本文件内最小扩展，逻辑与 RuleDataInterface 默认实现一致）
trait ChapterVariableExt {
    fn get_variable(&self, key: &str) -> Option<String>;
}

impl ChapterVariableExt for BookChapter {
    fn get_variable(&self, key: &str) -> Option<String> {
        self.variable_map().get(key).cloned()
    }
}
