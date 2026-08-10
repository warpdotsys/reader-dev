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
    pub source: Option<BaseSource>,
    pub rule_data: Option<Box<dyn RuleDataInterface>>,
    pub chapter: Option<BookChapter>,
    pub debug_log: Option<DebugLog>,
    pub rule_url: String, // var ruleUrl = "" private set
    pub url: String, // var url: String = "" private set
    pub body: Option<String>, // var body: String? = null private set
    pub type_: Option<String>, // var type: String? = null private set
    pub header_map: HashMap<String, String>, // val headerMap = HashMap<String, String>()
    url_no_query: String, // private var urlNoQuery: String = ""
    query_str: Option<String>, // private var queryStr: String? = null
    field_map: LinkedHashMap<String, String>, // private val fieldMap = LinkedHashMap<String, String>()
    charset: Option<String>, // private var charset: String? = null
    method: RequestMethod, // private var method = RequestMethod.GET
    proxy: Option<String>, // private var proxy: String? = null
    retry: i32, // private var retry: Int = 0
    use_web_view: bool, // private var useWebView: Boolean = false
    web_js: Option<String>, // private var webJs: String? = null
    enabled_cookie_jar: bool, // private val enabledCookieJar = source?.enabledCookieJar ?: false
}

impl AnalyzeUrl {
    // companion object {
    //     val paramPattern: Pattern = Pattern.compile("\s*,\s*(?=\{)")
    pub const PARAM_PATTERN: &'static str = r"\s*,\s*(?=\{)";
    //     private val pagePattern = Pattern.compile("<(.*?)>")
    const PAGE_PATTERN: &'static str = "<(.*?)>";
    //     private val concurrentRecordMap = hashMapOf<String, ConcurrentRecord>()
    //     // (静态可变容器, 对应 Java 的 HashMap)
    //     static CONCURRENT_RECORD_MAP: Lazy<Mutex<HashMap<String, ConcurrentRecord>>> = ...;
    // }
    //
    // class 构造 (含 init 块)
    pub fn new(
        m_url: String,
        key: Option<String>,
        page: Option<i32>,
        speak_text: Option<String>,
        speak_speed: Option<i32>,
        base_url: String,
        source: Option<BaseSource>,
        rule_data: Option<Box<dyn RuleDataInterface>>,
        chapter: Option<BookChapter>,
        header_map_f: Option<HashMap<String, String>>,
        debug_log: Option<DebugLog>,
    ) -> Self {
        let enabled_cookie_jar = source.as_ref().map(|s| s.enabled_cookie_jar).unwrap_or(false);
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
            if let Some(url_matcher) = PARAM_PATTERN.find(&s.base_url) {
                s.base_url = s.base_url[..url_matcher.start()].to_string(); // baseUrl = baseUrl.substring(0, urlMatcher.start())
            }
            // (headerMapF ?: source?.getHeaderMap(true))?.let {
            let header_map_f = header_map_f.or_else(|| s.source.as_ref().map(|src| src.get_header_map(true)));
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
    pub fn get_logger(&self) -> Option<DebugLog> {
        self.debug_log.clone()
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
     */
    fn analyze_js(&mut self) {
        let mut start = 0;
        let mut tmp: String;
        // val jsMatcher = JS_PATTERN.matcher(ruleUrl)
        let js_matcher: Vec<(usize, usize, Option<usize>, Option<usize>)> = Vec::new(); // JS_PATTERN 匹配结果: (start, end, group1, group2)
        for m in js_matcher {
            if m.0 > start {
                tmp = self.rule_url[m.0 - (m.0 - m.0)..m.0].trim().to_string();
                // 实际: ruleUrl.substring(start, jsMatcher.start()).trim { it <= ' ' }
                if !tmp.is_empty() {
                    self.rule_url = tmp.replace("@result", &self.rule_url);
                }
            }
            // ruleUrl = evalJS(jsMatcher.group(2) ?: jsMatcher.group(1), ruleUrl) as String
            let js = m.3.or(m.2);
            self.rule_url = self.eval_js(js.unwrap_or_default().to_string(), Some(&self.rule_url)).to_string() as String;
            start = m.1;
        }
        if self.rule_url.len() > start {
            tmp = self.rule_url[start..].trim().to_string();
            if !tmp.is_empty() {
                self.rule_url = tmp.replace("@result", &self.rule_url);
            }
        }
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
            let url = analyze.inner_rule("{{".to_string(), "}}".to_string(), |it| {
                // val jsEval = evalJS(it) ?: ""
                let js_eval = self.eval_js(it, None);
                if let Some(js_eval) = js_eval {
                    if let Some(s) = js_eval.as_string() {
                        Some(s)
                    } else if let Some(d) = js_eval.as_double() {
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
            let matcher: Vec<(usize, usize, String)> = Vec::new(); // pagePattern.matcher(ruleUrl) 匹配结果: (start, end, group1)
            for m in matcher {
                // val pages = matcher.group(1)!!.split(",")
                let pages: Vec<&str> = m.2.split(",").collect();
                let matched = &self.rule_url[m.0..m.1];
                self.rule_url = if (page as usize) < pages.len() {
                    // pages[pages.size - 1]等同于pages.last()
                    self.rule_url.replace(matched, pages[page as usize - 1].trim())
                } else {
                    self.rule_url.replace(matched, pages.last().unwrap().trim())
                };
            }
        }
    }

    /**
     * 解析Url
     */
    fn analyze_url(&mut self) {
        //replaceKeyPageJs已经替换掉额外内容，此处url是基础形式，可以直接切首个‘,’之前字符串。
        let url_matcher = PARAM_PATTERN.find(&self.rule_url);
        let (url_no_option, end) = if let Some(url_matcher) = url_matcher {
            (self.rule_url[..url_matcher.start()].to_string(), url_matcher.end())
        } else {
            (self.rule_url.clone(), 0)
        };
        self.url = get_absolute_url(&self.base_url, &url_no_option); // url = NetworkUtils.getAbsoluteURL(baseUrl, urlNoOption)
        if let Some(b) = get_base_url(&self.url) { // NetworkUtils.getBaseUrl(url)?.let {
            self.base_url = b;
        }
        if url_no_option.len() != self.rule_url.len() {
            // GSON.fromJsonObject<UrlOption>(ruleUrl.substring(urlMatcher.end())).getOrNull()?.let { option ->
            let option = gson_from_json_object::<UrlOption>(&self.rule_url[end..]);
            if let Some(option) = option {
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
        if !self.header_map.contains_key(UA_NAME) {
            self.header_map.insert(UA_NAME.to_string(), user_agent());
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
        let query_s = split_not_blank(&fields_txt, "&");
        for query in query_s {
            let query_m = split_not_blank(query, "=");
            let value = if query_m.len() > 1 { query_m[1] } else { "" };
            if self.charset.as_deref().map_or(true, |c| c.is_empty()) {
                if has_url_encoded(value) {
                    self.field_map.insert(query_m[0].to_string(), value.to_string());
                } else {
                    self.field_map.insert(query_m[0].to_string(), url_encode(value, "UTF-8")); // URLEncoder.encode(value, "UTF-8")
                }
            } else if self.charset.as_deref() == Some("escape") {
                self.field_map.insert(query_m[0].to_string(), encoder_utils_escape(value));
            } else {
                self.field_map.insert(query_m[0].to_string(), url_encode(value, self.charset.as_deref().unwrap_or("")));
            }
        }
    }

    /**
     * 执行JS
     */
    pub fn eval_js(&self, js_str: String, result: Option<&String>) -> Option<JValue> {
        let bindings = SimpleBindings::new(); // val bindings = SimpleBindings()
        bindings.set("java", self);
        bindings.set("baseUrl", self.base_url.clone());
        bindings.set("cookie", CookieStore::new(self.get_user_name_space()));
        bindings.set("cache", CacheManager::new(self.get_user_name_space()));
        bindings.set("page", self.page);
        bindings.set("key", self.key.clone());
        bindings.set("speakText", self.speak_text.clone());
        bindings.set("speakSpeed", self.speak_speed);
        bindings.set("book", self.rule_data.as_ref().and_then(|r| r.as_any().downcast_ref::<Book>())); // bindings["book"] = ruleData as? Book
        bindings.set("source", self.source.clone());
        bindings.set("result", result.cloned());
        SCRIPT_ENGINE.eval(js_str, bindings)
    }

    pub fn put(&mut self, key: String, value: String) -> String {
        // chapter?.putVariable(key, value) ?: ruleData?.putVariable(key, value)
        if let Some(chapter) = &mut self.chapter {
            chapter.put_variable(key, value);
        } else if let Some(rule_data) = &mut self.rule_data {
            rule_data.put_variable(key, value);
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
        return self.chapter.as_ref().and_then(|c| c.get_variable(&key))
            .or_else(|| self.rule_data.as_ref().and_then(|r| r.get_variable(&key)))
            .unwrap_or_default();
    }

    /**
     * 开始访问,并发判断
     */
    fn fetch_start(&self) -> Option<ConcurrentRecord> {
        let source = self.source.as_ref()?;
        let concurrent_rate = source.concurrent_rate.clone();
        if concurrent_rate.is_none_or(|r| r.is_empty()) {
            return None;
        }
        let rate_index = concurrent_rate.find("/"); // indexOf("/")
        let mut fetch_record = concurrent_record_map(source.get_key()); // concurrentRecordMap[source.getKey()]
        if fetch_record.is_none() {
            fetch_record = Some(ConcurrentRecord::new(rate_index.is_some(), System::now_millis(), 1));
            concurrent_record_map_put(source.get_key(), fetch_record.clone().unwrap());
            return fetch_record;
        }
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
                let sj = &concurrent_rate[rate_index + 1..]; // val sj = concurrentRate.substring(rateIndex + 1)
                let next_time = fetch_record.time + sj.parse::<i64>().unwrap();
                if System::now_millis() >= next_time {
                    fetch_record.time = System::now_millis();
                    fetch_record.frequency = 1;
                    0
                } else {
                    let cs = &concurrent_rate[..rate_index]; // val cs = concurrentRate.substring(0, rateIndex)
                    if fetch_record.frequency > cs.parse::<i64>().unwrap() {
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
            // throw ConcurrentException("根据并发率还需等待${waitTime}毫秒才可以访问", waitTime = waitTime)
            panic!("根据并发率还需等待{}毫秒才可以访问", wait_time);
        }
        return fetch_record;
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
            return StrResponse::new(self.url.clone(), byte_to_hex_string(self.get_byte_array_await().await));
        }
        let concurrent_record = self.fetch_start();
        self.set_cookie(self.source.as_ref().map(|s| s.get_key()));
        let str_response: StrResponse;
        if self.use_web_view && use_web_view {
            str_response = if self.method == RequestMethod::POST {
                // io.legado.app.adapters.ReaderAdapterHelper.getAdapter().getStrResponseByRemoteWebview(...)
                reader_adapter_helper().get_str_response_by_remote_webview(
                    url = self.url_no_query.clone(),
                    tag = self.source.as_ref().map(|s| s.get_key()),
                    header_map = self.header_map.clone(),
                    source_regex = source_regex,
                    java_script = self.web_js.clone().or(js_str), // webJs ?: jsStr
                    post = true,
                    body = self.body.clone(),
                    user_name_space = self.get_user_name_space(),
                    debug_log = self.debug_log.clone(),
                )
            } else {
                reader_adapter_helper().get_str_response_by_remote_webview(
                    url = self.url.clone(),
                    tag = self.source.as_ref().map(|s| s.get_key()),
                    header_map = self.header_map.clone(),
                    source_regex = source_regex,
                    java_script = self.web_js.clone().or(js_str),
                    user_name_space = self.get_user_name_space(),
                    debug_log = self.debug_log.clone(),
                )
            }
        } else {
            str_response = get_proxy_client(self.proxy.clone(), self.debug_log.clone()).new_call_str_response(self.retry, || {
                add_headers(self.header_map.clone());
                match self.method {
                    RequestMethod::POST => {
                        url(self.url_no_query.clone());
                        let content_type = self.header_map.get("Content-Type").cloned();
                        let body = self.body.clone();
                        if !self.field_map.is_empty() || body.as_deref().is_none_or(|b| b.trim().is_empty()) { // fieldMap.isNotEmpty() || body.isNullOrBlank()
                            post_form(self.field_map.clone(), true);
                        } else if content_type.as_deref().is_some_and(|ct| !ct.trim().is_empty()) { // !contentType.isNullOrBlank()
                            // val requestBody = body.toRequestBody(contentType.toMediaType())
                            let request_body = body.unwrap().to_request_body(content_type.unwrap().to_media_type());
                            post(request_body);
                        } else {
                            post_json(body);
                        }
                    }
                    _ => get(self.url_no_query.clone(), self.field_map.clone(), true),
                }
            });
        }
        self.save_cookie_jar(&str_response.raw); // saveCookieJar(strResponse!!.raw)
        self.fetch_end(concurrent_record);
        return str_response;
    }

    pub fn save_cookie_jar(&self, response: &Response) {
        let cookie_list = response.headers("Set-Cookie");
        if cookie_list.is_empty() {
            return;
        }
        let cookie_store = CookieStore::new(self.get_user_name_space());
        let domain = get_sub_domain(&self.url); // NetworkUtils.getSubDomain(url)
        for it in cookie_list {
            // cookieList.forEach { cookieStore.replaceCookie("${domain}_cookieJar", it) }
            cookie_store.replace_cookie(format!("{}_cookieJar", domain), it);
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
        let response = get_proxy_client(self.proxy.clone()).new_call_response(self.retry, || {
            add_headers(self.header_map.clone());
            match self.method {
                RequestMethod::POST => {
                    url(self.url_no_query.clone());
                    let content_type = self.header_map.get("Content-Type").cloned();
                    let body = self.body.clone();
                    if !self.field_map.is_empty() || body.as_deref().is_none_or(|b| b.trim().is_empty()) {
                        post_form(self.field_map.clone(), true);
                    } else if content_type.as_deref().is_some_and(|ct| !ct.trim().is_empty()) {
                        let request_body = body.unwrap().to_request_body(content_type.unwrap().to_media_type());
                        post(request_body);
                    } else {
                        post_json(body);
                    }
                }
                _ => get(self.url_no_query.clone(), self.field_map.clone(), true),
            }
        });
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
        let data_uri_find_result = DATA_URI_REGEX.find(&self.url_no_query);
        // @Suppress("BlockingMethodInNonBlockingContext")
        if let Some(data_uri_find_result) = data_uri_find_result {
            // val dataUriBase64 = dataUriFindResult.groupValues[1]
            let data_uri_base64 = data_uri_find_result.group_values(1);
            let byte_array = Base64::decode(&data_uri_base64, Base64::DEFAULT); // Base64.decode(dataUriBase64, Base64.DEFAULT)
            self.fetch_end(concurrent_record);
            return byte_array;
        } else {
            self.set_cookie(self.source.as_ref().map(|s| s.get_key()));
            let byte_array = get_proxy_client(self.proxy.clone()).new_call_response_body(self.retry, || {
                add_headers(self.header_map.clone());
                match self.method {
                    RequestMethod::POST => {
                        url(self.url_no_query.clone());
                        let content_type = self.header_map.get("Content-Type").cloned();
                        let body = self.body.clone();
                        if !self.field_map.is_empty() || body.as_deref().is_none_or(|b| b.trim().is_empty()) {
                            post_form(self.field_map.clone(), true);
                        } else if content_type.as_deref().is_some_and(|ct| !ct.trim().is_empty()) {
                            let request_body = body.unwrap().to_request_body(content_type.unwrap().to_media_type());
                            post(request_body);
                        } else {
                            post_json(body);
                        }
                    }
                    _ => get(self.url_no_query.clone(), self.field_map.clone(), true),
                }
            }).bytes(); // }.bytes()
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
        return get_proxy_client(self.proxy.clone()).new_call_str_response(self.retry, || {
            url(self.url_no_query.clone());
            // GSON.fromJsonObject<HashMap<String, Any>>(body).getOrNull()!!
            let mut body_map = gson_from_json_object::<HashMap<String, Any>>(self.body.clone()).get_or_null().unwrap();
            for (key, value) in body_map.clone() {
                if value.to_string() == "fileRequest" {
                    body_map.insert(key, map_of(
                        ("fileName", file_name.clone()),
                        ("file", file.clone()),
                        ("contentType", content_type.clone()),
                    ));
                }
            }
            post_multipart(self.type_.clone(), body_map);
        });
    }

    /**
     * 设置cookie urlOption的优先级大于书源保存的cookie
     * @param tag 书源url 缺省为传入的url
     */
    fn set_cookie(&mut self, tag: Option<String>) {
        let domain = get_sub_domain(tag.as_deref().unwrap_or(&self.url)); // NetworkUtils.getSubDomain(tag ?: url)
        if domain.is_empty() {
            return;
        }
        let cookie_store = CookieStore::new(self.get_user_name_space());
        if self.enabled_cookie_jar {
            // cookieStore.getCookie("${domain}_cookieJar")?.let { cookieStore.replaceCookie(domain, it) }
            if let Some(it) = cookie_store.get_cookie(format!("{}_cookieJar", domain)) {
                cookie_store.replace_cookie(domain.to_string(), it);
            }
        }
        let cookie = cookie_store.get_cookie(domain); // val cookie = cookieStore.getCookie(domain)
        if !cookie.is_empty() {
            let mut cookie_map = cookie_store.cookie_to_map(cookie); // val cookieMap = cookieStore.cookieToMap(cookie)
            let custom_cookie_map = cookie_store.cookie_to_map(self.header_map.get("Cookie").cloned().unwrap_or_default()); // cookieStore.cookieToMap(headerMap["Cookie"] ?: "")
            cookie_map.extend(custom_cookie_map); // cookieMap.putAll(customCookieMap)
            // val newCookie = cookieStore.mapToCookie(cookieMap)
            if let Some(new_cookie) = cookie_store.map_to_cookie(cookie_map) {
                self.header_map.insert("Cookie".to_string(), new_cookie);
            }
        }
    }

    pub fn get_user_agent(&self) -> String {
        self.header_map.get(UA_NAME).cloned().unwrap_or_else(|| user_agent()) // headerMap[UA_NAME] ?: AppConst.userAgent
    }

    pub fn is_post(&self) -> bool {
        return self.method == RequestMethod::POST;
    }

    // override fun getSource(): BaseSource? { return source }
    pub fn get_source(&self) -> Option<BaseSource> {
        return self.source.clone();
    }
}

// data class UrlOption(
//     private var method: String? = null,
//     private var charset: String? = null,
//     private var headers: Any? = null,
//     private var body: Any? = null,
//     private var retry: Int? = null,
//     private var type: String? = null,
//     private var webView: Any? = null,
//     private var webJs: String? = null,
//     private var js: String? = null,
// )
pub struct UrlOption {
    method: Option<String>,
    charset: Option<String>,
    headers: Option<Any>,
    body: Option<Any>,
    retry: Option<i32>,
    type_: Option<String>,
    web_view: Option<Any>,
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
        // method = if (value.isNullOrBlank()) null else value
        self.method = value.and_then(|v| if v.trim().is_empty() { None } else { Some(v) });
    }

    pub fn get_method(&self) -> Option<String> {
        return self.method.clone();
    }

    pub fn set_charset(&mut self, value: Option<String>) {
        // charset = if (value.isNullOrBlank()) null else value
        self.charset = value.and_then(|v| if v.trim().is_empty() { None } else { Some(v) });
    }

    pub fn get_charset(&self) -> Option<String> {
        return self.charset.clone();
    }

    pub fn set_retry(&mut self, value: Option<String>) {
        // retry = if (value.isNullOrEmpty()) null else value.toIntOrNull()
        self.retry = value.and_then(|v| if v.is_empty() { None } else { v.parse().ok() });
    }

    pub fn get_retry(&self) -> i32 {
        return self.retry.unwrap_or(0); // retry ?: 0
    }

    pub fn set_type(&mut self, value: Option<String>) {
        // type = if (value.isNullOrBlank()) null else value
        self.type_ = value.and_then(|v| if v.trim().is_empty() { None } else { Some(v) });
    }

    pub fn get_type(&self) -> Option<String> {
        return self.type_.clone();
    }

    pub fn use_web_view(&self) -> bool {
        // return when (webView) { null, "", false, "false" -> false; else -> true }
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
        // webView = if (boolean) true else null
        self.web_view = if boolean { Some(Any::from(true)) } else { None };
    }

    pub fn set_headers(&mut self, value: Option<String>) {
        // headers = if (value.isNullOrBlank()) { null } else { GSON.fromJsonObject<Map<String, Any>>(value).getOrNull() }
        self.headers = match value {
            Some(v) if !v.trim().is_empty() => gson_from_json_object::<HashMap<String, Any>>(v).get_or_null().map(Any::from),
            _ => None,
        };
    }

    pub fn get_header_map(&self) -> Option<&HashMap<String, Any>> {
        // return when (val value = headers) {
        //     is Map<*, *> -> value
        //     is String -> GSON.fromJsonObject<Map<String, Any>>(value).getOrNull()
        //     else -> null
        // }
        return match &self.headers {
            Some(value) if value.is_map() => value.as_map(),
            Some(value) if value.is_string() => gson_from_json_object::<HashMap<String, Any>>(value.to_string()).get_or_null().as_ref(),
            _ => None,
        };
    }

    pub fn set_body(&mut self, value: Option<String>) {
        // body = when {
        //     value.isNullOrBlank() -> null
        //     value.isJsonObject() -> GSON.fromJsonObject<Map<String, Any>>(value)
        //     value.isJsonArray() -> GSON.fromJsonArray<Map<String, Any>>(value)
        //     else -> value
        // }
        self.body = match value {
            None => None,
            Some(v) if v.trim().is_empty() => None,
            Some(v) if v.is_json_object() => gson_from_json_object::<HashMap<String, Any>>(v).map(Any::from),
            Some(v) if v.is_json_array() => gson_from_json_array::<Vec<HashMap<String, Any>>>(v).map(Any::from),
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
        // webJs = if (value.isNullOrBlank()) null else value
        self.web_js = value.and_then(|v| if v.trim().is_empty() { None } else { Some(v) });
    }

    pub fn get_web_js(&self) -> Option<String> {
        return self.web_js.clone();
    }

    pub fn set_js(&mut self, value: Option<String>) {
        // js = if (value.isNullOrBlank()) null else value
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
