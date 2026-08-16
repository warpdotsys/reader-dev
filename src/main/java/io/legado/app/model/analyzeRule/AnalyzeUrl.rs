use crate::prelude::*;
use std::sync::Mutex;
use crate::io_legado_app_help_http_okhttputils::{
    add_headers, get, new_call_response, new_call_response_body, new_call_str_response, post_form,
    post_json, post_multipart,
};
use crate::io_legado_app_help_http_strresponse::StrResponse;
use crate::stubs::{
    Any, Base64, HashMap, JsValue, MatchData, MediaType, Pattern, RequestBody,
    RequestBuilder, Response, ResponseBody, SCRIPT_ENGINE, SimpleBindings, System,
};
// package io.legado.app.model.analyzeRule
//
// import com.script.SimpleBindings
// import io.legado.app.constant.AppConst
// import io.legado.app.constant.AppConst.SCRIPT_ENGINE
// import io.legado.app.constant.AppConst.UA_NAME
// import io.legado.app.constant.AppPattern.JS_PATTERN
// import io.legado.app.constant.AppPattern.dataUriRegex
// import io.legado.app.data.entities.BaseSource
// import io.legado.app.data.entities.Book
// import io.legado.app.data.entities.BookChapter
// import io.legado.app.exception.ConcurrentException
// import io.legado.app.help.CacheManager
// import io.legado.app.help.JsExtensions
// import io.legado.app.help.http.*
// import io.legado.app.utils.*
// import kotlinx.coroutines.runBlocking
// import okhttp3.MediaType.Companion.toMediaType
// import okhttp3.RequestBody.Companion.toRequestBody
// import okhttp3.Response
// import java.net.URLEncoder
// import java.util.regex.Pattern
// import io.legado.app.model.DebugLog

/**
 * Created by GKF on 2018/1/24.
 * 搜索URL规则解析
 */
pub struct AnalyzeUrl {
    pub m_url: String,
    pub key: Option<String>,
    pub page: Option<i32>,
    pub speak_text: Option<String>,
    pub speak_speed: Option<i32>,
    pub base_url: String,
    // fix: BaseSource 因 JsExtensions::get_source 返回裸 trait 对象而非 dyn-compatible，
    //      用具体类型 BookSource 替代 Box<dyn BaseSource>（同文件可编译且不受 trait 影响）
    pub source: Option<BookSource>,
    pub rule_data: Option<Box<dyn RuleDataInterface>>,
    pub chapter: Option<BookChapter>,
    pub debug_log: Option<Box<dyn DebugLog>>,
    pub rule_url: String, // var ruleUrl = "" private set
    pub url: String, // var url: String = "" private set
    pub body: Option<String>, // var body: String? = None private set
    pub type_: Option<String>, // var type: String? = None private set
    pub header_map: HashMap<String, String>, // val headerMap = HashMap<String, String>()
    url_no_query: String, // private var urlNoQuery: String = ""
    query_str: Option<String>, // private var queryStr: String? = None
    field_map: LinkedHashMap<String, String>, // private val fieldMap = LinkedHashMap<String, String>()
    charset: Option<String>, // private var charset: String? = None
    method: RequestMethod, // private var method = RequestMethod.GET
    proxy: Option<String>, // private var proxy: String? = None
    retry: i32, // private var retry: Int = 0
    use_web_view: bool, // private var useWebView: Boolean = false
    web_js: Option<String>, // private var webJs: String? = None
    enabled_cookie_jar: bool, // private val enabledCookieJar = source?.enabledCookieJar ?: false
}

// fix: companion object 的 paramPattern / dataUriRegex（AppPattern.dataUriRegex）转为模块级函数
fn PARAM_PATTERN() -> Pattern {
    Pattern::compile(r"\s*,\s*(?=\{)")
}

fn DATA_URI_REGEX() -> Pattern {
    Pattern::compile(r"data:image/[a-zA-Z0-9.+-]+;base64,([A-Za-z0-9+/=]+)")
}

// fix: companion object 中 concurrentRecordMap 的转录占位（静态可变容器；HashMap::new 非 const，用 LazyLock）
static CONCURRENT_RECORD_MAP: std::sync::LazyLock<Mutex<HashMap<String, ConcurrentRecord>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

fn concurrent_record_map(key: String) -> Option<ConcurrentRecord> {
    CONCURRENT_RECORD_MAP.lock().unwrap().get(&key).cloned()
}

fn concurrent_record_map_put(key: String, record: ConcurrentRecord) {
    CONCURRENT_RECORD_MAP.lock().unwrap().insert(key, record);
}

fn concurrent_record_put(record: ConcurrentRecord) {
    // fix: 转录简化, 按 time 匹配已存在的记录
    let mut map = CONCURRENT_RECORD_MAP.lock().unwrap();
    for (_k, v) in map.iter_mut() {
        if v.time == record.time {
            *v = record;
            return;
        }
    }
}

fn concurrent_record_frequency_dec(record: ConcurrentRecord) {
    // concurrentRecord.frequency = concurrentRecord.frequency - 1
    let mut map = CONCURRENT_RECORD_MAP.lock().unwrap();
    for (_k, v) in map.iter_mut() {
        if v.time == record.time {
            v.frequency -= 1;
            return;
        }
    }
}

impl AnalyzeUrl {
    // companion object {
    //     val paramPattern: Pattern = Pattern.compile("\s*,\s*(?=\{)")
    //     private val pagePattern = Pattern.compile("<(.*?)>")
    //     private val concurrentRecordMap = hashMapOf<String, ConcurrentRecord>()
    //     // (静态可变容器, 对应 Java 的 HashMap)
    //     static CONCURRENT_RECORD_MAP: Lazy<Mutex<HashMap<String, ConcurrentRecord>>> = ...;
    // }
    // fix: 原 companion object 中的 Pattern 常量改为模块级函数（Pattern 非 const 构造）
    fn PAGE_PATTERN() -> Pattern {
        Pattern::compile("<(.*?)>")
    }
    //
    // class 构造 (含 init 块)
    /// fix: RSS 等场景（source 非 BookSource 无法传入）——构造后启用 cookie jar
    pub fn set_cookie_enabled(&mut self, enabled: bool) {
        self.enabled_cookie_jar = enabled;
    }

    pub fn new(
        m_url: String,
        key: Option<String>,
        page: Option<i32>,
        speak_text: Option<String>,
        speak_speed: Option<i32>,
        base_url: String,
        source: Option<BookSource>,
        rule_data: Option<Box<dyn RuleDataInterface>>,
        chapter: Option<BookChapter>,
        header_map_f: Option<HashMap<String, String>>,
        debug_log: Option<Box<dyn DebugLog>>,
    ) -> Self {
        // fix: BookSource 未实现 BaseSource，enabled_cookie_jar 为结构体字段而非方法
        let enabled_cookie_jar = source.as_ref().and_then(|s| s.enabled_cookie_jar).unwrap_or(false);
        let mut s = Self {
            m_url,
            key,
            page,
            speak_text,
            speak_speed,
            base_url,
            source,
            rule_data,
            chapter,
            debug_log,
            rule_url: String::new(),
            url: String::new(),
            body: None,
            type_: None,
            header_map: HashMap::new(),
            url_no_query: String::new(),
            query_str: None,
            field_map: LinkedHashMap::new(),
            charset: None,
            method: RequestMethod::GET,
            proxy: None,
            retry: 0,
            use_web_view: false,
            web_js: None,
            enabled_cookie_jar,
        };
        // init {
        if !is_data_url(&s.m_url) {
            // val urlMatcher = paramPattern.matcher(baseUrl)
            if let Some(url_matcher) = PARAM_PATTERN().find(&s.base_url) {
                s.base_url = s.base_url[..url_matcher.start()].to_string(); // baseUrl = baseUrl.substring(0, urlMatcher.start())
            }
            // (headerMapF ?: source?.getHeaderMap(true))?.let {
            // fix: 真实书源 header 解析（@js:/<js> 求值 + 登录头合并；原占位 JSON 原样塞 "header" 头）
            let header_map_f = header_map_f.or_else(|| {
                s.source.as_ref().map(|src| {
                    crate::stubs::parse_source_header_map(
                        src.header.clone(),
                        true,
                        Some(src.get_key()),
                        Some(s.get_user_name_space()),
                    )
                })
            });
            if let Some(it) = header_map_f {
                s.header_map.extend(it.clone()); // headerMap.putAll(it)
                if it.contains_key("proxy") { // it.containsKey("proxy")
                    s.proxy = it.get("proxy").cloned(); // proxy = it["proxy"]
                    s.header_map.remove("proxy");
                }
            }
            s.init_url();
        }
        // }
        s
    }

    // override fun getUserNameSpace(): String = ruleData?.getUserNameSpace() ?: "unknow"
    pub fn get_user_name_space(&self) -> String {
        self.rule_data.as_ref().map(|r| r.get_user_name_space()).unwrap_or_else(|| "unknow".to_string())
    }

    // override fun getLogger(): DebugLog? = debugLog
    pub fn get_logger(&self) -> Option<&dyn DebugLog> {
        self.debug_log.as_deref()
    }

    /**
     * 处理url
     */
    pub fn init_url(&mut self) {
        self.rule_url = self.m_url.clone();
        //执行@js,<js></js>
        self.analyze_js();
        //替换参数
        self.replace_key_page_js();
        //处理URL
        self.analyze_url();
    }

    /**
     * 执行@js,<js></js>
     * fix: 对齐 Kotlin analyzeJs（JS_PATTERN="<js>([\w\W]*?)</js>|@js:([\w\W]*)" 大小写不敏感）：
     *      ①匹配前文本段 trim 后 @result 替换为当前 ruleUrl（Kotlin L108-114）；
     *      ②<js> 非贪婪到 </js>、@js: 贪婪吃到字符串末尾（group2 优先）；
     *      ③eval 结果整体替换 ruleUrl（Kotlin L115 `ruleUrl = evalJS(...) as String`）
     *      原实现：@js: 要求闭合 @（无闭合段不执行——主流 "@js:计算URL" 写法直接失败）、拼接而非替换
     */
    fn analyze_js(&mut self) {
        let original = self.rule_url.clone();
        let mut rule_url = original.clone();
        let mut start = 0usize;
        loop {
            let lower = original[start..].to_lowercase();
            let rel = match (lower.find("<js>"), lower.find("@js:")) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            };
            let Some(rel) = rel else { break };
            let abs = start + rel;
            // 匹配前文本段（Kotlin substring(start, matchStart).trim；非空则 @result 替换为当前 ruleUrl）
            let prefix = if abs <= rule_url.len() { rule_url[..abs].trim().to_string() } else { String::new() };
            if !prefix.is_empty() {
                rule_url = prefix.replace("@result", &rule_url);
            }
            let is_tag = original[abs..].starts_with('<');
            let js = if is_tag {
                // <js>([\w\W]*?)</js> 非贪婪
                let rest = &original[abs + 4..];
                match rest.to_lowercase().find("</js>") {
                    Some(e) => rest[..e].to_string(),
                    None => break,
                }
            } else {
                // @js:([\w\W]*) 贪婪到串尾（group2）
                original[abs + 4..].to_string()
            };
            let val = self.eval_js(js, Some(&rule_url.clone())).map(|v| v.to_string()).unwrap_or_default();
            // Kotlin: ruleUrl = evalJS(...) as String（整体替换）
            rule_url = val;
            if is_tag {
                // 从 </js> 后继续（Kotlin start = matcher.end()）
                if let Some(e) = original[abs + 4..].to_lowercase().find("</js>") {
                    start = abs + 4 + e + 5;
                } else {
                    break;
                }
            } else {
                // @js: 吃到串尾——循环结束
                break;
            }
        }
        self.rule_url = rule_url;
    }

    /**
     * 替换关键字,页数,JS
     */
    fn replace_key_page_js(&mut self) {
        //先替换内嵌规则再替换页数规则，避免内嵌规则中存在大于小于号时，规则被切错
        //js
        if self.rule_url.contains("{{") && self.rule_url.contains("}}") {
            let mut analyze = RuleAnalyzer::new(self.rule_url.clone(), false); //创建解析
            //替换所有内嵌{{js}}
            let url = analyze.inner_rule_str("{{".to_string(), "}}".to_string(), |it| {
                // val jsEval = evalJS(it) ?: ""
                let js_eval = self.eval_js(it, None);
                if let Some(js_eval) = js_eval {
                    if let Some(s) = js_eval.as_string() {
                        Some(s)
                    } else if js_eval.is_double() {
                        let d = js_eval.as_double();
                        if d % 1.0 == 0.0 {
                            Some(format!("{:.0}", d)) // String.format("%.0f", jsEval)
                        } else {
                            Some(js_eval.to_string())
                        }
                    } else {
                        Some(js_eval.to_string())
                    }
                } else {
                    None
                }
            });
            if !url.is_empty() {
                self.rule_url = url;
            }
        }
        //page
        if let Some(page) = self.page {
            // fix: 原 pagePattern 匹配恒空 → 真实扫描 <页列表>（如 <1,2,3>）替换
            let mut rest = self.rule_url.clone();
            let mut result = String::new();
            loop {
                if let Some(start) = rest.find('<') {
                    result.push_str(&rest[..start]);
                    if let Some(end_rel) = rest[start..].find('>') {
                        let inner = rest[start + 1..start + end_rel].to_string();
                        let pages: Vec<&str> = inner.split(',').collect();
                        let matched = if (page as usize) < pages.len() {
                            pages[page as usize - 1].trim()
                        } else {
                            pages.last().unwrap_or(&"").trim()
                        };
                        result.push_str(matched);
                        rest = rest[start + end_rel + 1..].to_string();
                        continue;
                    }
                }
                result.push_str(&rest);
                break;
            }
            self.rule_url = result;
        }
    }

    /**
     * 解析Url
     */
    fn analyze_url(&mut self) {
        //replaceKeyPageJs已经替换掉额外内容，此处url是基础形式，可以直接切首个‘,’之前字符串。
        let url_matcher = PARAM_PATTERN().find(&self.rule_url);
        let (url_no_option, end) = if let Some(url_matcher) = url_matcher {
            (self.rule_url[..url_matcher.start()].to_string(), url_matcher.end())
        } else {
            (self.rule_url.clone(), 0)
        };
        self.url = NetworkUtils::getAbsoluteURL(Some(&self.base_url), &url_no_option); // url = NetworkUtils.getAbsoluteURL(baseUrl, urlNoOption)
        if let Some(b) = NetworkUtils::getBaseUrl(Some(&self.url)) { // NetworkUtils.getBaseUrl(url)?.let {
            self.base_url = b;
        }
        if url_no_option.len() != self.rule_url.len() {
            // GSON.fromJsonObject<UrlOption>(ruleUrl.substring(urlMatcher.end())).getOrNull()?.let { option ->
            let option = gson_from_json_object::<UrlOption>(self.rule_url[end..].to_string());
            if let Ok(option) = option {
                if let Some(it) = option.get_method() {
                    if it.eq_ignore_ascii_case("POST") { // it.equals("POST", true)
                        self.method = RequestMethod::POST;
                    }
                }
                if let Some(header_map) = option.get_header_map() {
                    for (k, v) in header_map {
                        self.header_map.insert(k.to_string(), v.to_string());
                    }
                }
                if let Some(it) = option.get_body() {
                    self.body = Some(it);
                }
                self.type_ = option.get_type();
                self.charset = option.get_charset();
                self.retry = option.get_retry();
                self.use_web_view = option.use_web_view();
                self.web_js = option.get_web_js();
                if let Some(js_str) = option.get_js() {
                    // option.getJs()?.let { jsStr -> evalJS(jsStr, url)?.toString()?.let { url = it } }
                    if let Some(it) = self.eval_js(js_str, Some(&self.url)).map(|v| v.to_string()) {
                        self.url = it;
                    }
                }
            }
        }
        // headerMap[UA_NAME] ?: let { headerMap[UA_NAME] = AppConst.userAgent }
        if !self.header_map.contains_key(AppConst::UA_NAME) {
            self.header_map.insert(AppConst::UA_NAME.to_string(), AppConst::userAgent());
        }
        self.url_no_query = self.url.clone();
        match self.method {
            RequestMethod::GET => {
                let pos = self.url.find('?');
                if let Some(pos) = pos {
                    self.analyze_fields(self.url[pos + 1..].to_string());
                    self.url_no_query = self.url[..pos].to_string();
                }
            }
            RequestMethod::POST => {
                if let Some(it) = self.body.clone() {
                    // if (!it.isJson() && !it.isXml() && headerMap["Content-Type"].isNullOrEmpty())
                    if !it.is_json() && !it.is_xml()
                        && self.header_map.get("Content-Type").map_or(true, |v| v.is_empty())
                    {
                        self.analyze_fields(it);
                    }
                }
            }
        }
    }

    /**
     * 解析QueryMap
     */
    fn analyze_fields(&mut self, fields_txt: String) {
        self.query_str = Some(fields_txt.clone());
        let query_s = fields_txt.split_not_blank("&");
        for query in query_s {
            let query_m = query.split_not_blank("=");
            let value = if query_m.len() > 1 { query_m[1].clone() } else { String::new() };
            if self.charset.as_deref().map_or(true, |c| c.is_empty()) {
                if NetworkUtils::hasUrlEncoded(&value) {
                    self.field_map.insert(query_m[0].to_string(), value.to_string());
                } else {
                    self.field_map.insert(query_m[0].to_string(), url_encode_charset(value, "UTF-8")); // URLEncoder.encode(value, "UTF-8")
                }
            } else if self.charset.as_deref() == Some("escape") {
                self.field_map.insert(query_m[0].to_string(), EncoderUtils::escape(&value));
            } else {
                self.field_map.insert(query_m[0].to_string(), url_encode_charset(value, self.charset.as_deref().unwrap_or("")));
            }
        }
    }

    /**
     * 执行JS
     */
    pub fn eval_js(&self, js_str: String, result: Option<&String>) -> Option<JsValue> {
        let mut bindings = SimpleBindings::new(); // val bindings = SimpleBindings()
        // fix: java 由全局对象提供（eval_js_script 注册扩展方法），此处不覆盖
        bindings.set("baseUrl", self.base_url.clone());
        // fix: CookieStore/CacheManager 真实实例绑定（JS 可调 cookie.getCookie()/cookie.setCookie()/cache.get()/cache.put()；
        //      原绑定字符串——方法调用 TypeError、cookie 登录流程全失效）
        let user_name_space = self.get_user_name_space();
        bindings.put("cookie", CookieStore::new(user_name_space.clone()));
        bindings.put("cache", CacheManager::new(user_name_space));
        bindings.set("page", self.page);
        bindings.set("key", self.key.clone());
        bindings.set("speakText", self.speak_text.clone());
        bindings.set("speakSpeed", self.speak_speed);
        // fix: book/source 绑定真实字段 JSON（JS 可访问 book.name / source.bookSourceUrl）
        if let Some(r) = &self.rule_data {
            if let Some(book) = r.as_any().downcast_ref::<Book>() {
                bindings.put("book", crate::stubs::Any::Str(crate::stubs::book_to_json(book).to_string()));
            } else {
                bindings.set("book", r.get_user_name_space());
            }
        } else {
            bindings.set("book", false);
        }
        match &self.source {
            Some(s) => {
                bindings.put("source", crate::stubs::Any::Str(crate::stubs::book_source_to_json(s).to_string()));
            }
            None => {
                bindings.set("source", false);
            }
        }
        // fix: result 以 Any::Str 绑定（JSON 对象形态自动解析为 JS 对象——loginCheckJs 的 result.url/result.body 可用；
        //      原 Option<String> 绑定恒为字符串字面）
        bindings.put("result", crate::stubs::Any::Str(result.cloned().unwrap_or_default()));
        // fix: JS 执行失败抛错（Kotlin SCRIPT_ENGINE.eval 抛 ScriptException→请求失败；
        //      原静默 None→URL 段变空→取到错误页面/规则无结果，难以排查）
        let js_head = js_str[..js_str.len().min(120)].to_string();
        match SCRIPT_ENGINE.eval_downcast_any(js_str, &mut bindings) {
            Some(a) => {
                let text = match &a {
                    crate::stubs::Any::Str(s) => s.clone(),
                    _ => crate::stubs::any_to_value(&a).to_string(),
                };
                Some(JsValue { value: Some(text) })
            }
            None => panic!("JS 执行失败: {}", js_head),
        }
    }

    pub fn put(&mut self, key: String, value: String) -> String {
        // chapter?.putVariable(key, value) ?: ruleData?.putVariable(key, value)
        if let Some(chapter) = &mut self.chapter {
            chapter.put_variable(key.clone(), Some(value.clone()));
        } else if let Some(rule_data) = &mut self.rule_data {
            rule_data.put_variable(&key, Some(&value));
        }
        return value;
    }

    pub fn get(&self, key: String) -> String {
        match key.as_str() {
            "bookName" => {
                // (ruleData as? Book)?.let { return it.name }
                if let Some(b) = self.rule_data.as_ref().and_then(|r| r.as_any().downcast_ref::<Book>()) {
                    return b.name.clone();
                }
            }
            "title" => {
                // chapter?.let { return it.title }
                if let Some(c) = &self.chapter {
                    return c.title.clone();
                }
            }
            _ => {}
        }
        // chapter?.getVariable(key) ?: ruleData?.getVariable(key) ?: ""
        return self.chapter.as_ref().and_then(|c| c.variable_map().get(&key).cloned())
            .or_else(|| self.rule_data.as_ref().and_then(|r| r.get_variable(&key)))
            .unwrap_or_default();
    }

    /**
     * 开始访问,并发判断
     */
    fn fetch_start(&self) -> Option<ConcurrentRecord> {
        let source = self.source.as_ref()?;
        // fix: BookSource 未实现 BaseSource，concurrent_rate 为结构体字段而非 trait 方法
        let concurrent_rate = source.concurrent_rate.as_deref()?; // concurrentRate ?: return null
        if concurrent_rate.is_empty() {
            return None;
        }
        let rate_index = concurrent_rate.find("/").map(|i| i as i32).unwrap_or(-1); // indexOf("/")
        let mut fetch_record = concurrent_record_map(source.get_key()); // concurrentRecordMap[source.getKey()]
        if fetch_record.is_none() {
            fetch_record = Some(ConcurrentRecord::new(rate_index != -1, System::now_millis(), 1));
            concurrent_record_map_put(source.get_key(), fetch_record.clone().unwrap());
            return fetch_record;
        }
        // Kotlin 智能转换: 上面的 null 检查后 fetchRecord 非空
        let mut fetch_record = fetch_record.unwrap();
        // val waitTime: Int = synchronized(fetchRecord) {
        let wait_time: i64 = {
            // synchronized(fetchRecord) {
            if rate_index == -1 {
                if fetch_record.frequency > 0 {
                    concurrent_rate.parse::<i64>().unwrap() // return@synchronized concurrentRate.toInt()
                } else {
                    let next_time = fetch_record.time + concurrent_rate.parse::<i64>().unwrap();
                    if System::now_millis() >= next_time {
                        fetch_record.time = System::now_millis();
                        fetch_record.frequency = 1;
                        0 // return@synchronized 0
                    } else {
                        next_time - System::now_millis() // return@synchronized (nextTime - System.currentTimeMillis()).toInt()
                    }
                }
            } else {
                let sj = &concurrent_rate[(rate_index + 1) as usize..]; // val sj = concurrentRate.substring(rateIndex + 1)
                let next_time = fetch_record.time + sj.parse::<i64>().unwrap();
                if System::now_millis() >= next_time {
                    fetch_record.time = System::now_millis();
                    fetch_record.frequency = 1;
                    0
                } else {
                    let cs = &concurrent_rate[..rate_index as usize]; // val cs = concurrentRate.substring(0, rateIndex)
                    if fetch_record.frequency > cs.parse::<i32>().unwrap() {
                        next_time - System::now_millis()
                    } else {
                        fetch_record.frequency = fetch_record.frequency + 1;
                        0
                    }
                }
            }
            // } catch (e: Exception) {
            //     return@synchronized 0
            // }
        };
        // }
        if wait_time > 0 {
            // fix: 原 panic 无捕获导致请求直接失败（Kotlin 抛 ConcurrentException 由上层等待重试）——
            //      等待并发窗口后重置记录并继续
            std::thread::sleep(std::time::Duration::from_millis(wait_time.max(0) as u64));
            fetch_record.time = System::now_millis();
            fetch_record.frequency = 0;
            concurrent_record_map_put(source.get_key(), fetch_record.clone());
        }
        return Some(fetch_record);
    }

    /**
     * 访问结束
     */
    fn fetch_end(&self, concurrent_record: Option<ConcurrentRecord>) {
        if let Some(concurrent_record) = concurrent_record {
            if !concurrent_record.concurrent {
                // synchronized(concurrentRecord) {
                concurrent_record_put(concurrent_record.clone()); // 实际由互斥锁保护
                concurrent_record_frequency_dec(concurrent_record); // concurrentRecord.frequency = concurrentRecord.frequency - 1
                // }
            }
        }
    }

    /**
     * 访问网站,返回StrResponse
     */
    pub async fn get_str_response_await(
        &mut self,
        js_str: Option<String>,
        source_regex: Option<String>,
        use_web_view: bool,
    ) -> StrResponse {
        if self.type_.is_some() {
            // fix: E0502 &self.url 不可变借用与 &mut self 的 get_byte_array_await 冲突，先取 bytes 再构造
            let bytes = self.get_byte_array_await().await;
            return StrResponse::new_url(&self.url, Some(byte_to_hex_string(&bytes)));
        }
        let concurrent_record = self.fetch_start();
        self.set_cookie(self.source.as_ref().map(|s| s.get_key()));
        let str_response: StrResponse;
        // fix: useWebView 需无头浏览器（服务端无）——回退普通请求（原 webview 占位返回空 body，整类书源失效）
        if self.use_web_view && use_web_view {
            let web_view_body = ReaderAdapterHelper::get_adapter().get_str_response_by_remote_webview(
                Some(&self.url),
                None,
                None,
                self.source.as_ref().map(|s| s.get_key()).as_deref(),
                Some(&self.header_map),
                source_regex.as_deref(),
                self.web_js.clone().or(js_str).as_deref(),
                None,
                matches!(self.method, RequestMethod::POST),
                self.body.as_deref(),
                &self.get_user_name_space(),
                self.debug_log.as_deref(),
            )
            .await
            .and_then(|r| r.body().cloned())
            .filter(|b| !b.is_empty());
            str_response = if let Some(body) = web_view_body {
                StrResponse::new_url(&self.url, Some(body))
            } else {
                // fix: 对齐 Kotlin——webview 适配器不可用（无头浏览器未配置）时整请求失败
                //      （Kotlin DefaultAdpater throw "不支持webview"；原回退普通 HTTP 拿未渲染 HTML——规则产出错内容/空内容）
                panic!("不支持webview: {}", &self.url[..self.url.len().min(100)]);
            };
        } else {
            str_response = new_call_str_response(
                &get_proxy_client(self.proxy.as_deref(), self.debug_log.as_deref()),
                self.retry,
                |builder: &mut RequestBuilder| {
                    add_headers(builder, &self.header_map);
                    match self.method {
                        RequestMethod::POST => {
                            builder.url(&self.url_no_query);
                            let content_type = self.header_map.get("Content-Type").cloned();
                            let body = self.body.clone();
                            if !self.field_map.is_empty() || body.as_deref().is_none_or(|b| b.trim().is_empty()) { // fieldMap.isNotEmpty() || body.isNullOrBlank()
                                post_form(builder, &self.field_map, true);
                            } else if content_type.as_deref().is_some_and(|ct| !ct.trim().is_empty()) { // !contentType.isNullOrBlank()
                                // val requestBody = body.toRequestBody(contentType.toMediaType())
                                let request_body = body.unwrap().to_request_body(content_type.unwrap().to_media_type());
                                builder.post(request_body);
                            } else {
                                post_json(builder, body.as_deref());
                            }
                        }
                        _ => get(builder, &self.url_no_query, &self.field_map, true),
                    }
                },
            )
            .await;
        }
        self.save_cookie_jar(str_response.raw()); // saveCookieJar(strResponse!!.raw)
        self.fetch_end(concurrent_record);
        return str_response;
    }

    pub fn save_cookie_jar(&self, response: &Response) {
        let cookie_list = response.headers("Set-Cookie");
        if cookie_list.is_empty() {
            return;
        }
        let cookie_store = CookieStore::new(self.get_user_name_space());
        let domain = NetworkUtils::getSubDomain(Some(&self.url)); // NetworkUtils.getSubDomain(url)
        for it in cookie_list {
            // cookieList.forEach { cookieStore.replaceCookie("${domain}_cookieJar", it) }
            cookie_store.replace_cookie(&format!("{}_cookieJar", domain), &it);
        }
    }

    // @JvmOverloads
    pub fn get_str_response(
        &mut self,
        js_str: Option<String>,
        source_regex: Option<String>,
        use_web_view: bool,
    ) -> StrResponse {
        // return runBlocking { getStrResponseAwait(jsStr, sourceRegex, useWebView) }
        block_on(self.get_str_response_await(js_str, source_regex, use_web_view))
    }

    /**
     * 访问网站,返回Response
     */
    pub async fn get_response_await(&mut self) -> Response {
        let concurrent_record = self.fetch_start();
        self.set_cookie(self.source.as_ref().map(|s| s.get_key()));
        // @Suppress("BlockingMethodInNonBlockingContext")
        let response = new_call_response(
            &get_proxy_client(self.proxy.as_deref(), None),
            self.retry,
            |builder: &mut RequestBuilder| {
                add_headers(builder, &self.header_map);
                match self.method {
                    RequestMethod::POST => {
                        builder.url(&self.url_no_query);
                        let content_type = self.header_map.get("Content-Type").cloned();
                        let body = self.body.clone();
                        if !self.field_map.is_empty() || body.as_deref().is_none_or(|b| b.trim().is_empty()) {
                            post_form(builder, &self.field_map, true);
                        } else if content_type.as_deref().is_some_and(|ct| !ct.trim().is_empty()) {
                            let request_body = body.unwrap().to_request_body(content_type.unwrap().to_media_type());
                            builder.post(request_body);
                        } else {
                            post_json(builder, body.as_deref());
                        }
                    }
                    _ => get(builder, &self.url_no_query, &self.field_map, true),
                }
            },
        )
        .await;
        self.fetch_end(concurrent_record);
        return response;
    }

    pub fn get_response(&mut self) -> Response {
        // return runBlocking { getResponseAwait() }
        block_on(self.get_response_await())
    }

    /**
     * 访问网站,返回ByteArray
     */
    pub async fn get_byte_array_await(&mut self) -> Vec<u8> {
        let concurrent_record = self.fetch_start();
        // @Suppress("RegExpRedundantEscape")
        // val dataUriFindResult = dataUriRegex.find(urlNoQuery)
        let data_uri_find_result = DATA_URI_REGEX().find(&self.url_no_query);
        // @Suppress("BlockingMethodInNonBlockingContext")
        if let Some(data_uri_find_result) = data_uri_find_result {
            // val dataUriBase64 = dataUriFindResult.groupValues[1]
            let data_uri_base64 = data_uri_find_result.group_values(1);
            let byte_array = Base64::decode(&data_uri_base64, Base64::DEFAULT); // Base64.decode(dataUriBase64, Base64.DEFAULT)
            self.fetch_end(concurrent_record);
            return byte_array;
        } else {
            self.set_cookie(self.source.as_ref().map(|s| s.get_key()));
            let byte_array = new_call_response_body(
                &get_proxy_client(self.proxy.as_deref(), None),
                self.retry,
                |builder: &mut RequestBuilder| {
                    add_headers(builder, &self.header_map);
                    match self.method {
                        RequestMethod::POST => {
                            builder.url(&self.url_no_query);
                            let content_type = self.header_map.get("Content-Type").cloned();
                            let body = self.body.clone();
                            if !self.field_map.is_empty() || body.as_deref().is_none_or(|b| b.trim().is_empty()) {
                                post_form(builder, &self.field_map, true);
                            } else if content_type.as_deref().is_some_and(|ct| !ct.trim().is_empty()) {
                                let request_body = body.unwrap().to_request_body(content_type.unwrap().to_media_type());
                                builder.post(request_body);
                            } else {
                                post_json(builder, body.as_deref());
                            }
                        }
                        _ => get(builder, &self.url_no_query, &self.field_map, true),
                    }
                },
            )
            .await
            .bytes(); // }.bytes()
            self.fetch_end(concurrent_record);
            return byte_array;
        }
    }

    pub fn get_byte_array(&mut self) -> Vec<u8> {
        // return runBlocking { getByteArrayAwait() }
        block_on(self.get_byte_array_await())
    }

    /**
     * 上传文件
     */
    pub async fn upload(&mut self, file_name: String, file: Any, content_type: String) -> StrResponse {
        return new_call_str_response(
            &get_proxy_client(self.proxy.as_deref(), None),
            self.retry,
            |builder: &mut RequestBuilder| {
                builder.url(&self.url_no_query);
                // GSON.fromJsonObject<HashMap<String, Any>>(body).getOrNull()!!
                let mut body_map: HashMap<String, Box<dyn std::any::Any>> =
                    gson_from_json_object::<HashMap<String, Any>>(self.body.clone().unwrap_or_default())
                        .get_or_null()
                        .unwrap_or_default()
                        .into_iter()
                        .map(|(k, v)| (k, Box::new(v) as Box<dyn std::any::Any>))
                        .collect();
                // fix: HashMap<String, Box<dyn Any>> 不可 Clone，先收集 key 再按 key 查询/插入
                for key in body_map.keys().cloned().collect::<Vec<String>>() {
                    // value == "fileRequest"
                    if body_map
                        .get(&key)
                        .and_then(|v| v.downcast_ref::<String>())
                        == Some(&"fileRequest".to_string())
                    {
                        let mut file_map: HashMap<String, Box<dyn std::any::Any>> = HashMap::new();
                        file_map.insert("fileName".to_string(), Box::new(file_name.clone()));
                        file_map.insert("file".to_string(), Box::new(file.clone()));
                        file_map.insert("contentType".to_string(), Box::new(content_type.clone()));
                        body_map.insert(key, Box::new(file_map));
                    }
                }
                post_multipart(builder, self.type_.as_deref(), &body_map);
            },
        )
        .await;
    }

    /**
     * 设置cookie urlOption的优先级大于书源保存的cookie
     * @param tag 书源url 缺省为传入的url
     */
    fn set_cookie(&mut self, tag: Option<String>) {
        let domain = NetworkUtils::getSubDomain(Some(tag.as_deref().unwrap_or(&self.url))); // NetworkUtils.getSubDomain(tag ?: url)
        if domain.is_empty() {
            return;
        }
        let cookie_store = CookieStore::new(self.get_user_name_space());
        if self.enabled_cookie_jar {
            // cookieStore.getCookie("${domain}_cookieJar")?.let { cookieStore.replaceCookie(domain, it) }
            let it = cookie_store.get_cookie(&format!("{}_cookieJar", domain));
            if !it.is_empty() {
                cookie_store.replace_cookie(&domain, &it);
            }
        }
        let cookie = cookie_store.get_cookie(&domain); // val cookie = cookieStore.getCookie(domain)
        if !cookie.is_empty() {
            let mut cookie_map = cookie_store.cookie_to_map(&cookie); // val cookieMap = cookieStore.cookieToMap(cookie)
            let custom_cookie = self.header_map.get("Cookie").cloned().unwrap_or_default(); // headerMap["Cookie"] ?: ""
            let custom_cookie_map = cookie_store.cookie_to_map(&custom_cookie); // cookieStore.cookieToMap(headerMap["Cookie"] ?: "")
            cookie_map.extend(custom_cookie_map); // cookieMap.putAll(customCookieMap)
            // val newCookie = cookieStore.mapToCookie(cookieMap)
            if let Some(new_cookie) = cookie_store.map_to_cookie(Some(&cookie_map)) {
                self.header_map.insert("Cookie".to_string(), new_cookie);
            }
        }
    }

    pub fn get_user_agent(&self) -> String {
        self.header_map.get(AppConst::UA_NAME).cloned().unwrap_or_else(AppConst::userAgent) // headerMap[UA_NAME] ?: AppConst.userAgent
    }

    pub fn is_post(&self) -> bool {
        return matches!(self.method, RequestMethod::POST);
    }

    // override fun getSource(): BaseSource? { return source }
    pub fn get_source(&self) -> Option<&BookSource> {
        return self.source.as_ref();
    }
}

// data class UrlOption(
//     private var method: String? = None,
//     private var charset: String? = None,
//     private var headers: Any? = None,
//     private var body: Any? = None,
//     private var retry: Int? = None,
//     private var type: String? = None,
//     private var webView: Any? = None,
//     private var webJs: String? = None,
//     private var js: String? = None,
// )
// fix: GSON.fromJsonObject<UrlOption> 需要 Deserialize; 字段名对应 Kotlin 属性名
#[derive(serde::Deserialize)]
pub struct UrlOption {
    method: Option<String>,
    charset: Option<String>,
    headers: Option<Any>,
    body: Option<Any>,
    retry: Option<i32>,
    #[serde(rename = "type")]
    type_: Option<String>,
    #[serde(rename = "webView")]
    web_view: Option<Any>,
    #[serde(rename = "webJs")]
    web_js: Option<String>,
    js: Option<String>,
}

impl UrlOption {
    pub fn new(
        method: Option<String>,
        charset: Option<String>,
        headers: Option<Any>,
        body: Option<Any>,
        retry: Option<i32>,
        type_: Option<String>,
        web_view: Option<Any>,
        web_js: Option<String>,
        js: Option<String>,
    ) -> Self {
        Self {
            method,
            charset,
            headers,
            body,
            retry,
            type_,
            web_view,
            web_js,
            js,
        }
    }

    pub fn set_method(&mut self, value: Option<String>) {
        // method = if (value.isNullOrBlank()) None else value
        self.method = value.and_then(|v| if v.trim().is_empty() { None } else { Some(v) });
    }

    pub fn get_method(&self) -> Option<String> {
        return self.method.clone();
    }

    pub fn set_charset(&mut self, value: Option<String>) {
        // charset = if (value.isNullOrBlank()) None else value
        self.charset = value.and_then(|v| if v.trim().is_empty() { None } else { Some(v) });
    }

    pub fn get_charset(&self) -> Option<String> {
        return self.charset.clone();
    }

    pub fn set_retry(&mut self, value: Option<String>) {
        // retry = if (value.isNullOrEmpty()) None else value.toIntOrNull()
        self.retry = value.and_then(|v| if v.is_empty() { None } else { v.parse().ok() });
    }

    pub fn get_retry(&self) -> i32 {
        return self.retry.unwrap_or(0); // retry ?: 0
    }

    pub fn set_type(&mut self, value: Option<String>) {
        // type = if (value.isNullOrBlank()) None else value
        self.type_ = value.and_then(|v| if v.trim().is_empty() { None } else { Some(v) });
    }

    pub fn get_type(&self) -> Option<String> {
        return self.type_.clone();
    }

    pub fn use_web_view(&self) -> bool {
        // return when (webView) { None, "", false, "false" -> false; else -> true }
        return match &self.web_view {
            None => false,
            Some(v) => {
                let s = v.to_string();
                if s.is_empty() || s == "false" || v.is_false_bool() {
                    false
                } else {
                    true
                }
            }
        };
    }

    pub fn use_web_view_boolean(&mut self, boolean: bool) {
        // webView = if (boolean) true else None
        self.web_view = if boolean { Some(Any::from(true)) } else { None };
    }

    pub fn set_headers(&mut self, value: Option<String>) {
        // headers = if (value.isNullOrBlank()) { None } else { GSON.fromJsonObject<Map<String, Any>>(value).getOrNull() }
        self.headers = match value {
            Some(v) if !v.trim().is_empty() => gson_from_json_object::<HashMap<String, Any>>(v).get_or_null().map(Any::from),
            _ => None,
        };
    }

    pub fn get_header_map(&self) -> Option<HashMap<String, Any>> {
        // return when (val value = headers) {
        //     is Map<*, *> -> value
        //     is String -> GSON.fromJsonObject<Map<String, Any>>(value).getOrNull()
        //     else -> None
        // }
        return match &self.headers {
            Some(value) if value.is_map() => value.as_map(),
            Some(value) if value.is_string() => gson_from_json_object::<HashMap<String, Any>>(value.to_string()).get_or_null(),
            _ => None,
        };
    }

    pub fn set_body(&mut self, value: Option<String>) {
        // body = when {
        //     value.isNullOrBlank() -> None
        //     value.isJsonObject() -> GSON.fromJsonObject<Map<String, Any>>(value)
        //     value.isJsonArray() -> GSON.fromJsonArray<Map<String, Any>>(value)
        //     else -> value
        // }
        self.body = match value {
            None => None,
            Some(v) if v.trim().is_empty() => None,
            Some(v) if v.is_json_object() => gson_from_json_object::<HashMap<String, Any>>(v).ok().map(Any::from),
            Some(v) if v.is_json_array() => gson_from_json_array::<Vec<HashMap<String, Any>>>(v).ok().map(Any::from),
            Some(v) => Some(Any::from(v)),
        };
    }

    pub fn get_body(&self) -> Option<String> {
        // return body?.let { if (it is String) it else GSON.toJson(it) }
        return self.body.as_ref().map(|it| {
            if it.is_string() {
                it.to_string()
            } else {
                gson_to_json(it)
            }
        });
    }

    pub fn set_web_js(&mut self, value: Option<String>) {
        // webJs = if (value.isNullOrBlank()) None else value
        self.web_js = value.and_then(|v| if v.trim().is_empty() { None } else { Some(v) });
    }

    pub fn get_web_js(&self) -> Option<String> {
        return self.web_js.clone();
    }

    pub fn set_js(&mut self, value: Option<String>) {
        // js = if (value.isNullOrBlank()) None else value
        self.js = value.and_then(|v| if v.trim().is_empty() { None } else { Some(v) });
    }

    pub fn get_js(&self) -> Option<String> {
        return self.js.clone();
    }
}

// data class ConcurrentRecord(
//     val concurrent: Boolean,
//     var time: Long,
//     var frequency: Int
// )
#[derive(Clone)]
pub struct ConcurrentRecord {
    pub concurrent: bool,
    pub time: i64,
    pub frequency: i32,
}

impl ConcurrentRecord {
    pub fn new(concurrent: bool, time: i64, frequency: i32) -> Self {
        Self {
            concurrent,
            time,
            frequency,
        }
    }
}
