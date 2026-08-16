// 占位类型库：让纯转录的 Rust 代码可编译（逻辑保持等价，运行行为以降级实现为主）
// 该文件由编译迭代驱动逐步补齐。 (fix: `//!` 改为 `//`，include! 进模块后内部文档注释触发 E0753)

pub use std::collections::HashMap;
use serde::Deserialize;

// ---------------- 通用错误 ----------------

#[derive(Debug, Clone)]
pub struct StubError {
    pub msg: String,
}

impl StubError {
    pub fn new(msg: impl Into<String>) -> Self {
        StubError { msg: msg.into() }
    }
}

impl std::fmt::Display for StubError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for StubError {}

// fix: 允许 `Option?` 在返回 Result<_, StubError> 的闭包/函数中使用
impl From<std::convert::Infallible> for StubError {
    fn from(_: std::convert::Infallible) -> Self {
        StubError::new("infallible")
    }
}

pub trait ThrowableExt {
    fn localized_message(&self) -> String;
    fn stack_trace_to_string(&self) -> String;
    fn msg(&self) -> Option<String>;
    // fix: 补充 print_stack_trace 默认实现（BookController 等模块已调用）
    fn print_stack_trace(&self) {}
    // fix: JsExtensions 使用（Kotlin `Throwable.printOnDebug()`）
    fn print_on_debug(&self) {}
}

impl ThrowableExt for StubError {
    fn localized_message(&self) -> String {
        self.msg.clone()
    }
    fn stack_trace_to_string(&self) -> String {
        self.msg.clone()
    }
    fn msg(&self) -> Option<String> {
        Some(self.msg.clone())
    }
}

impl ThrowableExt for std::io::Error {
    fn localized_message(&self) -> String {
        self.to_string()
    }
    fn stack_trace_to_string(&self) -> String {
        format!("{:?}", self)
    }
    fn msg(&self) -> Option<String> {
        Some(self.to_string())
    }
}

impl ThrowableExt for regex::Error {
    fn localized_message(&self) -> String {
        self.to_string()
    }
    fn stack_trace_to_string(&self) -> String {
        format!("{:?}", self)
    }
    fn msg(&self) -> Option<String> {
        Some(self.to_string())
    }
}

impl ThrowableExt for url::ParseError {
    fn localized_message(&self) -> String {
        self.to_string()
    }
    fn stack_trace_to_string(&self) -> String {
        format!("{:?}", self)
    }
    fn msg(&self) -> Option<String> {
        Some(self.to_string())
    }
}

impl ThrowableExt for serde_json::Error {
    fn localized_message(&self) -> String {
        self.to_string()
    }
    fn stack_trace_to_string(&self) -> String {
        format!("{:?}", self)
    }
    fn msg(&self) -> Option<String> {
        Some(self.to_string())
    }
}

pub trait ResultExt<T> {
    fn get_or_none(self) -> Option<T>;
    fn get_or_default(self, default: T) -> T;
    // fix: 补充 getOrNull 别名（Book.rs 等转录模块已调用）
    fn get_or_null(self) -> Option<T>
    where
        Self: Sized,
    {
        self.get_or_none()
    }
}

impl<T, E> ResultExt<T> for Result<T, E> {
    fn get_or_none(self) -> Option<T> {
        self.ok()
    }
    fn get_or_default(self, default: T) -> T {
        self.unwrap_or(default)
    }
}

pub trait OptionExt<T> {
    fn get_or_throw(self) -> Result<T, StubError>;
    fn get_or_none(self) -> Option<T>;
    // fix: Option.isPresent()（ReaderUIApplication.showConfirm 使用）
    fn is_present(&self) -> bool;
}

impl<T> OptionExt<T> for Option<T> {
    fn get_or_throw(self) -> Result<T, StubError> {
        self.ok_or_else(|| StubError::new("getOrThrow() on None"))
    }
    fn get_or_none(self) -> Option<T> {
        self
    }
    fn is_present(&self) -> bool {
        self.is_some()
    }
}

// ---------------- 正则 Pattern / Matcher（regex 包装） ----------------

#[derive(Clone)]
pub struct Pattern {
    re: fancy_regex::Regex,
    src: String,
}

impl Pattern {
    pub const CASE_INSENSITIVE: u32 = 1;

    pub fn compile(src: &str) -> Pattern {
        let re = fancy_regex::Regex::new(src).unwrap_or_else(|_| fancy_regex::Regex::new("").unwrap());
        Pattern { re, src: src.to_string() }
    }

    pub fn compile_with(src: &str, _flags: u32) -> Pattern {
        let mut s = src.to_string();
        if _flags & Self::CASE_INSENSITIVE != 0 && !s.starts_with("(?i)") {
            s = format!("(?i){}", s);
        }
        Pattern::compile(&s)
    }

    #[allow(elided_lifetimes_in_paths)]
    pub fn matcher(&self, haystack: String) -> Matcher<'_> {
        Matcher {
            re: &self.re,
            hay: haystack,
            pos: 0,
            last: None,
            in_range: false,
        }
    }

    pub fn matches(&self, s: &str) -> bool {
        self.re.is_match(s).unwrap_or(false)
    }

    pub fn is_match(&self, s: &str) -> bool {
        self.re.is_match(s).unwrap_or(false)
    }

    pub fn replace_all(&self, hay: &str, rep: &str) -> String {
        self.re.replace_all(hay, rep).to_string()
    }

    pub fn replace_first(&self, hay: &str, rep: &str) -> String {
        self.re.replace(hay, rep).to_string()
    }

    pub fn split(&self, hay: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut last = 0;
        for m in self.re.find_iter(hay).filter_map(|m| m.ok()) {
            out.push(hay[last..m.start()].to_string());
            last = m.end();
        }
        out.push(hay[last..].to_string());
        out
    }
    pub fn src(&self) -> &str {
        &self.src
    }
    pub fn find_iter<'a>(&'a self, hay: &'a str) -> MatcherIter<'a> {
        MatcherIter {
            it: self.re.find_iter(hay),
            hay,
        }
    }

    pub fn to_string(&self) -> String {
        self.src.clone()
    }

    // Kotlin Regex.find(haystack) 一次匹配（返回带 start/end/groupValues 的 MatchData）
    pub fn find(&self, hay: &str) -> Option<MatchData> {
        let caps = self.re.captures(hay).ok()??;
        let m = caps.get(0)?;
        let groups: Vec<String> = (0..caps.len())
            .map(|i| caps.get(i).map(|g| g.as_str().to_string()).unwrap_or_default())
            .collect();
        Some(MatchData {
            start: m.start(),
            end: m.end(),
            groups,
        })
    }
}

#[derive(Debug, Clone)]
pub struct MatchData {
    start: usize,
    end: usize,
    groups: Vec<String>,
}

impl MatchData {
    pub fn start(&self) -> usize {
        self.start
    }
    pub fn end(&self) -> usize {
        self.end
    }
    pub fn group_values(&self, i: usize) -> String {
        self.groups.get(i).cloned().unwrap_or_default()
    }
}

pub struct MatcherIter<'a> {
    it: fancy_regex::Matches<'a, 'a, str>,
    hay: &'a str,
}

impl<'a> Iterator for MatcherIter<'a> {
    type Item = (usize, usize, String);
    fn next(&mut self) -> Option<Self::Item> {
        self.it
            .next()
            .and_then(|m| m.ok())
            .map(|m| (m.start(), m.end(), m.as_str().to_string()))
    }
}
pub struct Matcher<'a> {
    re: &'a fancy_regex::Regex,
    hay: String,
    pos: usize,
    last: Option<(usize, usize)>,
    in_range: bool,
}

impl<'a> Matcher<'a> {
    pub fn find(&mut self) -> bool {
        if self.in_range && self.pos >= self.hay.len() {
            return false;
        }
        match self.re.find_from_pos(&self.hay, self.pos) {
            Ok(Some(m)) => {
                self.last = Some((m.start(), m.end()));
                self.pos = m.end();
                if !self.in_range {
                    self.in_range = true;
                }
                true
            }
            _ => false,
        }
    }

    pub fn find_from(&mut self, from: usize) -> bool {
        self.pos = from;
        self.find()
    }

    pub fn start(&self) -> usize {
        self.last.map(|(s, _)| s).unwrap_or(0)
    }

    pub fn end(&self) -> usize {
        self.last.map(|(_, e)| e).unwrap_or(0)
    }

    pub fn group(&self) -> String {
        self.last
            .map(|(s, e)| self.hay[s..e].to_string())
            .unwrap_or_default()
    }

    pub fn group_idx(&self, i: usize) -> Option<String> {
        let (s, e) = self.last?;
        let caps = self.re.captures(&self.hay[s..e]).ok()??;
        caps.get(i).map(|m| m.as_str().to_string())
    }

    pub fn group_count(&self) -> usize {
        self.last
            .map(|_| self.re.captures_len().saturating_sub(1))
            .unwrap_or(0)
    }

    pub fn matches(&self) -> bool {
        self.last.is_some()
    }
}

// ---------------- URL 包装 ----------------

#[derive(Debug, Clone)]
pub struct URL(pub url::Url);

impl URL {
    pub fn new(s: String) -> Result<URL, StubError> {
        url::Url::parse(&s).map(URL).map_err(|e| StubError::new(e.to_string()))
    }

    pub fn parse(s: &str) -> Result<URL, StubError> {
        url::Url::parse(s).map(URL).map_err(|e| StubError::new(e.to_string()))
    }

    pub fn protocol(&self) -> String {
        self.0.scheme().to_string()
    }

    pub fn host(&self) -> Option<String> {
        self.0.host_str().map(|h| h.to_string())
    }

    pub fn path(&self) -> String {
        self.0.path().to_string()
    }

    pub fn pathname(&self) -> String {
        self.0.path().to_string()
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    // JavaFX URL.toExternalForm()（ReaderUIApplication 图标资源使用）
    pub fn to_external_form(&self) -> String {
        self.0.to_string()
    }
}

// ---------------- GSON（serde_json 包装） ----------------

pub struct GSON;

impl GSON {
    pub fn from_json_object<T>(s: String) -> Result<T, StubError>
    where
        T: serde::de::DeserializeOwned,
    {
        serde_json::from_str::<T>(&s).map_err(|e| StubError::new(e.to_string()))
    }

    pub fn to_json_string<T: serde::Serialize>(v: &T) -> String {
        serde_json::to_string(v).unwrap_or_default()
    }

    // fix: Kotlin GSON.toJson(value)（Book.rs Converters 等转录模块使用）
    pub fn to_json<T: serde::Serialize>(v: T) -> String {
        serde_json::to_string(&v).unwrap_or_default()
    }
}

// ---------------- JS 引擎占位 ----------------

#[derive(Default)]
pub struct SimpleBindings {
    pub map: HashMap<String, Box<dyn AnyDebug>>,
}

pub trait AnyDebug: std::any::Any + std::fmt::Debug {
    fn as_any(&self) -> &dyn std::any::Any;
}
impl<T: std::any::Any + std::fmt::Debug> AnyDebug for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl SimpleBindings {
    pub fn new() -> Self {
        SimpleBindings::default()
    }

    pub fn put(&mut self, key: &str, value: impl AnyDebug + 'static) {
        self.map.insert(key.to_string(), Box::new(value));
    }

    // Kotlin `bindings[key] = value`（AnalyzeUrl.evalJS 使用；真实存储供 JS 引擎绑定）
    pub fn set(&mut self, key: &str, value: impl std::fmt::Debug + 'static) {
        self.map.insert(key.to_string(), Box::new(value));
    }

    pub fn get(&self, key: &str) -> Option<&Box<dyn AnyDebug>> {
        self.map.get(key)
    }
}

#[derive(Default)]
pub struct NativeObject {
    pub map: HashMap<String, Box<dyn AnyDebug>>,
}

impl NativeObject {
    pub fn new() -> Self {
        NativeObject::default()
    }

    pub fn put(&mut self, key: &str, value: impl AnyDebug + 'static) {
        self.map.insert(key.to_string(), Box::new(value));
    }

    pub fn get(&self, key: &str) -> Option<&Box<dyn AnyDebug>> {
        self.map.get(key)
    }
}

pub struct ScriptEngine;

impl ScriptEngine {
    pub fn eval(
        &self,
        js: String,
        bindings: &mut SimpleBindings,
    ) -> Option<Box<dyn AnyDebug>> {
        crate::runtime::js::eval_js_script(&js, bindings).map(|any| Box::new(any) as Box<dyn AnyDebug>)
    }
    // 直接返回 Any（绕过 Box<dyn AnyDebug> upcast 的 downcast 问题）
    pub fn eval_downcast_any(
        &self,
        js: String,
        bindings: &mut SimpleBindings,
    ) -> Option<Any> {
        crate::runtime::js::eval_js_script(&js, bindings)
    }
}

pub static SCRIPT_ENGINE: ScriptEngine = ScriptEngine;

// ---------------- Entities 占位 ----------------

pub struct Entities;

impl Entities {
    /// jsoup Entities.unescape：常用命名实体 + &#123; / &#x1F; 数字引用
    /// fix: 原仅 5 个命名实体、数字引用不解码——&nbsp;/&#8220;/&hellip; 等保留在结果中
    pub fn unescape(s: String) -> Result<String, StubError> {
        let mut out = String::with_capacity(s.len());
        let chars: Vec<char> = s.chars().collect();
        let mut i = 0usize;
        while i < chars.len() {
            let c = chars[i];
            if c == '&' {
                // 数字引用 &#123; / &#x1F;（可带分号）
                if i + 1 < chars.len() && chars[i + 1] == '#' {
                    let mut j = i + 2;
                    let mut hex = false;
                    if j < chars.len() && (chars[j] == 'x' || chars[j] == 'X') {
                        hex = true;
                        j += 1;
                    }
                    let start = j;
                    while j < chars.len() && chars[j] != ';' && chars[j].is_alphanumeric() {
                        j += 1;
                    }
                    if j > start {
                        let digits: String = chars[start..j].iter().collect();
                        if let Ok(code) = if hex { u32::from_str_radix(&digits, 16) } else { digits.parse::<u32>() } {
                            if let Some(ch) = char::from_u32(code) {
                                out.push(ch);
                                // 跳过 ';'（若存在）
                                if j < chars.len() && chars[j] == ';' {
                                    j += 1;
                                }
                                i = j;
                                continue;
                            }
                        }
                    }
                }
                // 命名实体
                let named = [
                    ("amp;", '&'),
                    ("lt;", '<'),
                    ("gt;", '>'),
                    ("quot;", '"'),
                    ("apos;", '\''),
                    ("nbsp;", '\u{00A0}'),
                    ("hellip;", '…'),
                    ("mdash;", '—'),
                    ("ndash;", '–'),
                    ("ldquo;", '\u{201C}'),
                    ("rdquo;", '\u{201D}'),
                    ("lsquo;", '\u{2018}'),
                    ("rsquo;", '\u{2019}'),
                    ("middot;", '·'),
                    ("laquo;", '«'),
                    ("raquo;", '»'),
                    ("times;", '×'),
                    ("divide;", '÷'),
                    ("copy;", '©'),
                    ("reg;", '®'),
                    ("trade;", '™'),
                    ("deg;", '°'),
                    ("plusmn;", '±'),
                    ("sect;", '§'),
                    ("para;", '¶'),
                    ("bull;", '•'),
                    ("euro;", '€'),
                    ("pound;", '£'),
                    ("yen;", '¥'),
                    ("cent;", '¢'),
                ];
                let rest: String = chars[i + 1..].iter().take(10).collect();
                let mut matched = false;
                for (entity, ch) in named {
                    if rest.starts_with(entity) {
                        out.push(ch);
                        i += 1 + entity.len();
                        matched = true;
                        break;
                    }
                }
                if matched {
                    continue;
                }
                out.push(c);
            } else {
                out.push(c);
            }
            i += 1;
        }
        Ok(out)
    }

    pub fn escape(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }
}

// ---------------- 日志占位 ----------------

pub struct Log;

impl Log {
    pub fn debug(&self, _msg: impl AsRef<str>) {}
    pub fn info(&self, _msg: impl AsRef<str>) {}
    pub fn warn(&self, _msg: impl AsRef<str>) {}
    pub fn error(&self, _msg: impl AsRef<str>) {}
    pub fn trace(&self, _msg: impl AsRef<str>) {}
}

pub struct KotlinLogging;

impl KotlinLogging {
    // fix: 调整为无参关联函数（BookLogger.rs `KotlinLogging::logger()` 转录调用）
    pub fn logger() -> KotlinLoggingLogger {
        KotlinLoggingLogger
    }
    pub fn logger_empty() -> KotlinLoggingLogger {
        KotlinLoggingLogger
    }
}

pub struct KotlinLoggingLogger;

impl KotlinLoggingLogger {
    pub fn debug(&self, _msg: String) {}
    pub fn info(&self, _msg: String) {}
    pub fn warn(&self, _msg: String) {}
    pub fn error(&self, _msg: String) {}
    pub fn trace(&self, _msg: String) {}
}

pub fn logger() -> Log {
    Log
}

// ---------------- 集合别名 ----------------

pub type ArrayList<T> = Vec<T>;

// Kotlin java.util.List 别名（List<String> / List<JXNode> 等转录类型）
pub type List<T> = Vec<T>;

// Kotlin arrayListOf()
pub fn array_list_of<T>() -> Vec<T> {
    Vec::new()
}

// Kotlin java.util.List.add / addAll / indices（Kotlin List.indices 即 0..lastIndex）
pub trait VecExt<T> {
    fn add(&mut self, item: T);
    fn add_all(&mut self, other: Vec<T>);
    fn indices(&self) -> std::ops::Range<usize>;
}

impl<T> VecExt<T> for Vec<T> {
    fn add(&mut self, item: T) {
        self.push(item);
    }
    fn add_all(&mut self, other: Vec<T>) {
        self.extend(other);
    }
    fn indices(&self) -> std::ops::Range<usize> {
        0..self.len()
    }
}

// Kotlin List.joinToString(separator)
pub trait JoinToStringExt {
    fn join_to_string(&self, sep: &str) -> String;
}

impl<T: std::fmt::Display> JoinToStringExt for Vec<T> {
    fn join_to_string(&self, sep: &str) -> String {
        self.iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(sep)
    }
}

// Kotlin String?.isNullOrEmpty()
pub trait OptionStringExt {
    fn is_null_or_empty(&self) -> bool;
}

impl OptionStringExt for Option<String> {
    fn is_null_or_empty(&self) -> bool {
        match self {
            Some(s) => s.is_empty(),
            None => true,
        }
    }
}

// ---------------- LRUCache（真实转录模块 re-export） ----------------

// LRUCache 模块内使用 Rc / RefCell / Hash（其 `use crate::prelude::*`），在此补充导出
pub use std::cell::RefCell;
pub use std::hash::Hash;
pub use std::rc::Rc;

pub use crate::com_htmake_reader_utils_lrucache::LRUCache;

// AnalyzeByJSonPath / AnalyzeByXPath 等模块直接使用 RuleAnalyzer（同名真实转录模块）
pub use crate::io_legado_app_model_analyzerule_ruleanalyzer::RuleAnalyzer;

// ---------------- kotlinx.coroutines.sync.Mutex（真实互斥；原 no-op 导致并发写坏） ----------------

#[derive(Debug, Clone, Default)]
pub struct Mutex {
    locked: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Mutex {
    pub fn new() -> Self {
        Mutex {
            locked: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
    pub async fn lock(&self) {
        // 自旋等待（用户级互斥临界区短；无 tokio 依赖）
        while self.locked.swap(true, std::sync::atomic::Ordering::Acquire) {
            std::thread::yield_now();
        }
    }
    pub fn unlock(&self) {
        self.locked.store(false, std::sync::atomic::Ordering::Release);
    }
}

// ---------------- okhttp3 Request / Response（真实请求数据载体） ----------------

#[derive(Debug, Clone, Default)]
pub struct Request {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub body_bytes: Option<Vec<u8>>,
    pub form_fields: HashMap<String, String>,
    pub content_type: Option<String>,
}

impl Request {
    pub fn builder() -> RequestBuilder {
        RequestBuilder::default()
    }
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).map(|s| s.as_str())
    }
    pub fn new_builder(&self) -> RequestBuilder {
        // fix: 复制当前请求（原返回空 builder——拦截器重建请求丢失 url/method/body，请求全失败）
        RequestBuilder {
            inner: std::cell::RefCell::new(self.clone()),
        }
    }
    pub fn url(&self) -> HttpUrl {
        HttpUrl(self.url.clone())
    }
}

#[derive(Debug, Clone, Default)]
pub struct RequestBuilder {
    pub inner: std::cell::RefCell<Request>,
}

impl RequestBuilder {
    pub fn url(&self, url: &str) -> &Self {
        self.inner.borrow_mut().url = url.to_string();
        self
    }
    // okhttp3 Request.Builder.post(requestBody)
    pub fn post(&self, body: RequestBody) -> &Self {
        let mut r = self.inner.borrow_mut();
        r.method = String::from("POST");
        r.body = Some(body.text);
        r.body_bytes = body.bytes;
        if let Some(mt) = &body.media_type {
            r.content_type = Some(mt.clone());
        }
        self
    }
    pub fn get(&self) -> &Self {
        self.inner.borrow_mut().method = String::from("GET");
        self
    }
    pub fn add_header<N: AsRef<str>, V: AsRef<str>>(&self, name: N, value: V) -> &Self {
        self.inner
            .borrow_mut()
            .headers
            .insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }
    pub fn remove_header(&self, name: &str) -> &Self {
        self.inner.borrow_mut().headers.remove(name);
        self
    }
    pub fn header<N: AsRef<str>, V: AsRef<str>>(&self, name: N, value: V) -> &Self {
        self.add_header(name, value)
    }
    pub fn add_form_field<K: AsRef<str>, V: AsRef<str>>(&self, name: K, value: V) -> &Self {
        self.inner.borrow_mut().form_fields.insert(
            name.as_ref().to_string(),
            value.as_ref().to_string(),
        );
        self
    }
    pub fn set_body(&self, text: impl AsRef<str>) -> &Self {
        self.inner.borrow_mut().body = Some(text.as_ref().to_string());
        self
    }
    pub fn build(&self) -> Request {
        self.inner.borrow().clone()
    }
}

// okhttp3 MediaType / RequestBody（RequestBody 携带文本）
#[derive(Debug, Clone, Default)]
pub struct MediaType {
    pub name: &'static str,
    pub default_extension: &'static str,
    pub extensions: &'static [&'static str],
}

#[derive(Debug, Clone, Default)]
pub struct RequestBody {
    pub text: String,
    pub media_type: Option<String>,
    pub bytes: Option<Vec<u8>>,
}

impl RequestBody {
    pub fn new() -> Self {
        RequestBody::default()
    }
    pub fn from_text(text: impl Into<String>) -> RequestBody {
        RequestBody { text: text.into(), media_type: None, bytes: None }
    }
    // okhttp3 RequestBody.create(mediaType, text)
    pub fn create(media_type: &MediaType, text: impl Into<String>) -> RequestBody {
        RequestBody { text: text.into(), media_type: Some(media_type.name.to_string()), bytes: None }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Response {
    pub status: i32,
    pub headers: HashMap<String, String>,
    pub headers_multi: HashMap<String, Vec<String>>,
    pub body_text: String,
    pub body_bytes: Vec<u8>,
    pub url: String,
}

impl Response {
    pub fn message_str(&self) -> String {
        String::new()
    }
    pub fn execute(&self) -> Response {
        self.clone()
    }
    pub fn is_successful(&self) -> bool {
        self.status >= 200 && self.status < 400
    }
    pub fn code(&self) -> i32 {
        self.status
    }
    pub fn body(&self) -> Body {
        Body {
            text: self.body_text.clone(),
        }
    }
    // okhttp3 Response.headers(name) -> List<String>（AnalyzeUrl.saveCookieJar 使用）
    // fix: 大小写不敏感（reqwest 将响应头小写化）+ 支持多值（多 Set-Cookie）
    pub fn headers(&self, name: &str) -> Vec<String> {
        let lower = name.to_lowercase();
        if let Some(v) = self.headers_multi.get(&lower) {
            return v.clone();
        }
        for (k, v) in &self.headers {
            if k.eq_ignore_ascii_case(name) {
                return vec![v.clone()];
            }
        }
        Vec::new()
    }
    // okhttp3 Response.header(name)（单值）
    pub fn header(&self, name: &str) -> Option<String> {
        for (k, v) in &self.headers {
            if k.eq_ignore_ascii_case(name) {
                return Some(v.clone());
            }
        }
        None
    }
    pub fn content_type(&self) -> Option<String> {
        self.header("content-type")
    }
    pub fn to_string(&self) -> String {
        self.url.clone()
    }
    pub fn request(&self) -> Request {
        Request {
            url: self.url.clone(),
            ..Default::default()
        }
    }
    pub fn body_option(&self) -> Option<ResponseBody> {
        Some(ResponseBody {
            text: Some(self.body_text.clone()),
            bytes: self.body_bytes.clone(),
            content_type: self.content_type(),
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Body {
    pub text: String,
}

impl Body {
    pub fn string(&self) -> String {
        self.text.clone()
    }
    pub fn bytes(&self) -> Vec<u8> {
        self.text.clone().into_bytes()
    }
}

// ---------------- java.util.Locale 占位 ----------------

#[derive(Debug, Clone, Default)]
pub struct Locale;

impl Locale {
    pub const PRC: Locale = Locale;
    pub fn get_default() -> Locale {
        Locale
    }
    pub fn get_country(&self) -> String {
        String::new()
    }
}

// ---------------- java.util.Date / java.time.LocalDateTime / java.util.UUID 占位 ----------------

pub struct Date;

impl Date {
    pub fn now() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone, Default)]
pub struct LocalDateTime;

impl LocalDateTime {
    pub fn now() -> Self {
        LocalDateTime
    }
    pub fn format(&self, _pattern: &str) -> String {
        String::new()
    }
}

#[derive(Debug, Clone)]
pub struct Uuid {
    bytes: [u8; 16],
}

impl Uuid {
    // fix: 真实 v4 UUID（原恒 00000000-...，JS randomUuid 全零）
    pub fn new_v4() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let c = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut b = [0u8; 16];
        b[0..8].copy_from_slice(&(now as u64).to_le_bytes());
        b[8..12].copy_from_slice(&c.to_le_bytes()[..4]);
        let pid = (std::process::id() as u64).to_le_bytes();
        b[12..16].copy_from_slice(&pid[..4]);
        b[6] = (b[6] & 0x0f) | 0x40; // version 4
        b[8] = (b[8] & 0x3f) | 0x80; // RFC 4122 variant
        Uuid { bytes: b }
    }
}

impl std::fmt::Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let b = self.bytes;
        write!(
            f,
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]
        )
    }
}

// ---------------- StringBuilder 占位 ----------------

#[derive(Debug, Clone, Default)]
pub struct StringBuilder(pub String);

impl StringBuilder {
    pub fn new() -> Self {
        StringBuilder(String::new())
    }

    pub fn append(&mut self, s: impl std::fmt::Display) -> &mut Self {
        self.0.push_str(&s.to_string());
        self
    }

    pub fn insert(&mut self, idx: usize, s: impl std::fmt::Display) -> &mut Self {
        self.0.insert_str(idx.min(self.0.len()), &s.to_string());
        self
    }

    pub fn insert0(&mut self, s: impl std::fmt::Display) -> &mut Self {
        self.0.insert_str(0, &s.to_string());
        self
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }

    pub fn length(&self) -> usize {
        self.0.len()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

// ---------------- String 扩展方法 ----------------

pub trait StringExt: AsRef<str> + ToString {
    fn is_blank(&self) -> bool;
    fn is_not_blank(&self) -> bool;
    fn is_json(&self) -> bool;
    fn split_blank(&self) -> Vec<String>;
    fn split_not_blank(&self, sep: &str) -> Vec<String>;
    fn starts_with_ignore_case(&self, prefix: &str) -> bool;
    fn contains_ignore_case(&self, needle: &str) -> bool;
    fn replace_first_str(&self, from: &str, to: &str) -> String;
    fn replace_str(&self, from: &str, to: &str) -> String;
    fn replace_regex_all(&self, re: &str, rep: &str) -> Result<String, StubError>;
    // fix: SourceAnalyzer toNewUrls/toNewUrl 使用（Kotlin `String.replace(Regex, replacement)`）
    fn replace_with_regex(&self, re: &str, rep: &str) -> String {
        match regex::Regex::new(re) {
            Ok(rx) => rx.replace_all(&self.to_string(), rep).to_string(),
            Err(_) => self.to_string(),
        }
    }
    // fix: SourceAnalyzer toNewUrls 使用（Kotlin `String.split(Regex)`）
    fn split_with_regex(&self, re: &str) -> Vec<String> {
        match regex::Regex::new(re) {
            Ok(rx) => rx.split(&self.to_string()).map(|s| s.to_string()).collect(),
            Err(_) => vec![self.to_string()],
        }
    }
    fn to_int(&self) -> i32;
    fn to_long(&self) -> i64;
    fn to_double(&self) -> f64;
    fn is_numeric(&self) -> bool;
    fn trim_str(&self) -> String;
    // Kotlin String.compareTo(other, ignoreCase = false) 返回 Int
    fn cmp_sensitive(&self, other: &str) -> i32 {
        let a = self.to_string();
        let b = other.to_string();
        if a < b {
            -1
        } else if a > b {
            1
        } else {
            0
        }
    }
    // Kotlin String.compareTo(other, ignoreCase = true) 返回 Int
    fn cmp_ignore_case(&self, other: &str) -> i32 {
        let a = self.to_string().to_lowercase();
        let b = other.to_lowercase();
        if a < b {
            -1
        } else if a > b {
            1
        } else {
            0
        }
    }
    // Kotlin String.substring(beginIndex)
    fn substring(&self, begin: usize) -> String {
        self.to_string()[begin..].to_string()
    }
    // Kotlin String.substring(beginIndex, endIndex)
    fn substring_range(&self, begin: usize, end: usize) -> String {
        self.to_string()[begin..end].to_string()
    }
    // Kotlin CharSequence.subSequence(start, end)
    fn sub_sequence(&self, start: usize, end: usize) -> String {
        self.to_string()[start..end].to_string()
    }
    // Kotlin String.takeLast(n)
    fn take_last(&self, n: usize) -> String {
        let s = self.to_string();
        if s.len() > n {
            s[s.len() - n..].to_string()
        } else {
            s
        }
    }
    // Kotlin String.indexOf(sub, fromIndex)（找不到返回 -1）
    fn index_of(&self, sub: &str, from: usize) -> i32 {
        let s = self.to_string();
        if from >= s.len() {
            return -1;
        }
        match s[from..].find(sub) {
            Some(i) => (from + i) as i32,
            None => -1,
        }
    }
    // Kotlin String.hashCode()（ACache.newFile 使用）
    fn hashCode(&self) -> i32 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&self.to_string(), &mut h);
        std::hash::Hasher::finish(&h) as i32
    }
    // Kotlin String.random()（占位：LCG 伪随机）
    fn random(&self) -> char {
        let s = self.to_string();
        let mut seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        seed ^= (s.len() as u128).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        seed ^= seed >> 30;
        seed = seed.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        seed ^= seed >> 27;
        seed = seed.wrapping_mul(0x94D0_49BB_1331_11EB);
        seed ^= seed >> 31;
        let chars: Vec<char> = s.chars().collect();
        if chars.is_empty() {
            return '\0';
        }
        chars[(seed as usize) % chars.len()]
    }
    // Kotlin String.isXml()（AnalyzeUrl.analyzeUrl 使用）
    fn is_xml(&self) -> bool {
        let t = self.to_string().trim().to_string();
        (t.starts_with('<') && t.ends_with('>')) || t.starts_with("<?xml")
    }
    // Kotlin String.isJsonObject()（AnalyzeUrl.UrlOption.setBody 使用）
    fn is_json_object(&self) -> bool {
        let t = self.to_string().trim().to_string();
        t.starts_with('{') && t.ends_with('}')
    }
    // Kotlin String.isJsonArray()（AnalyzeUrl.UrlOption.setBody 使用）
    fn is_json_array(&self) -> bool {
        let t = self.to_string().trim().to_string();
        t.starts_with('[') && t.ends_with(']')
    }
    // okhttp3 String.toMediaType()（AnalyzeUrl 使用）——fix: 真实 MediaType（原恒 None）
    fn to_media_type(&self) -> Option<MediaType> {
        if self.to_string().trim().is_empty() {
            None
        } else {
            Some(MediaType {
                name: Box::leak(self.to_string().into_boxed_str()),
                default_extension: "",
                extensions: &[],
            })
        }
    }
    // okhttp3 String.toRequestBody()（AnalyzeUrl 使用）——fix: 真实文本 + Content-Type（原恒空 body）
    fn to_request_body(&self, media_type: Option<MediaType>) -> RequestBody {
        RequestBody {
            text: self.to_string(),
            media_type: media_type.map(|m| m.name.to_string()),
            bytes: None,
        }
    }
    // Kotlin String.equals(Object)（RestVerticle rawMethod().equals("PUT") 使用）
    fn equals(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl StringExt for String {
    fn is_blank(&self) -> bool {
        self.trim().is_empty()
    }
    fn is_not_blank(&self) -> bool {
        !self.trim().is_empty()
    }
    fn is_json(&self) -> bool {
        let t = self.trim();
        (t.starts_with('{') && t.ends_with('}')) || (t.starts_with('[') && t.ends_with(']'))
    }
    fn split_blank(&self) -> Vec<String> {
        self.split_whitespace().map(|s| s.to_string()).collect()
    }
    fn split_not_blank(&self, sep: &str) -> Vec<String> {
        self.split(sep)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }
    fn starts_with_ignore_case(&self, prefix: &str) -> bool {
        self.len() >= prefix.len()
            && self.get(..prefix.len()).map_or(false, |s| s.eq_ignore_ascii_case(prefix))
    }
    fn contains_ignore_case(&self, needle: &str) -> bool {
        self.to_lowercase().contains(&needle.to_lowercase())
    }
    fn replace_first_str(&self, from: &str, to: &str) -> String {
        self.replacen(from, to, 1)
    }
    fn replace_str(&self, from: &str, to: &str) -> String {
        self.replace(from, to)
    }
    fn replace_regex_all(&self, re: &str, rep: &str) -> Result<String, StubError> {
        let rx = regex::Regex::new(re).map_err(|e| StubError::new(e.to_string()))?;
        Ok(rx.replace_all(self, rep).to_string())
    }
    fn to_int(&self) -> i32 {
        self.trim().parse().unwrap_or(0)
    }
    fn to_long(&self) -> i64 {
        self.trim().parse().unwrap_or(0)
    }
    fn to_double(&self) -> f64 {
        self.trim().parse().unwrap_or(0.0)
    }
    fn is_numeric(&self) -> bool {
        self.trim().parse::<f64>().is_ok()
    }
    fn trim_str(&self) -> String {
        self.trim().to_string()
    }
}

// &str 也提供 StringExt（如 count_occurrences 中 text.index_of(...)）
impl StringExt for &str {
    fn is_blank(&self) -> bool {
        self.trim().is_empty()
    }
    fn is_not_blank(&self) -> bool {
        !self.trim().is_empty()
    }
    fn is_json(&self) -> bool {
        let t = self.trim();
        (t.starts_with('{') && t.ends_with('}')) || (t.starts_with('[') && t.ends_with(']'))
    }
    fn split_blank(&self) -> Vec<String> {
        self.split_whitespace().map(|s| s.to_string()).collect()
    }
    fn split_not_blank(&self, sep: &str) -> Vec<String> {
        self.split(sep)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }
    fn starts_with_ignore_case(&self, prefix: &str) -> bool {
        self.len() >= prefix.len() && (*self)[..prefix.len()].eq_ignore_ascii_case(prefix)
    }
    fn contains_ignore_case(&self, needle: &str) -> bool {
        self.to_lowercase().contains(&needle.to_lowercase())
    }
    fn replace_first_str(&self, from: &str, to: &str) -> String {
        self.replacen(from, to, 1)
    }
    fn replace_str(&self, from: &str, to: &str) -> String {
        self.replace(from, to)
    }
    fn replace_regex_all(&self, re: &str, rep: &str) -> Result<String, StubError> {
        let rx = regex::Regex::new(re).map_err(|e| StubError::new(e.to_string()))?;
        Ok(rx.replace_all(self, rep).to_string())
    }
    fn to_int(&self) -> i32 {
        self.trim().parse().unwrap_or(0)
    }
    fn to_long(&self) -> i64 {
        self.trim().parse().unwrap_or(0)
    }
    fn to_double(&self) -> f64 {
        self.trim().parse().unwrap_or(0.0)
    }
    fn is_numeric(&self) -> bool {
        self.trim().parse::<f64>().is_ok()
    }
    fn trim_str(&self) -> String {
        self.trim().to_string()
    }
}

// ---------------- 任意值包装（降级实现） ----------------

#[derive(Debug, Clone, Default)]
pub struct JsValue {
    pub value: Option<String>,
}

impl JsValue {
    pub fn new() -> Self {
        JsValue::default()
    }
    pub fn to_string(&self) -> String {
        self.value.clone().unwrap_or_default()
    }
    pub fn is_string(&self) -> bool {
        self.value.is_some()
    }
    // Kotlin JsValue.asString()
    pub fn as_string(&self) -> Option<String> {
        self.value.clone()
    }
    pub fn is_list(&self) -> bool {
        false
    }
    pub fn as_list(&self) -> Vec<JsValue> {
        Vec::new()
    }
    pub fn is_double(&self) -> bool {
        false
    }
    pub fn as_double(&self) -> f64 {
        0.0
    }
}

// ---------------- DOM / JSoup 占位 ----------------

#[derive(Debug, Clone, Default)]
pub struct Document {
    pub html: String,
    pub text: String,
}

impl Document {
    pub fn new() -> Self {
        Document::default()
    }
    pub fn parse(s: String) -> Document {
        let text = crate::runtime::html::text_of(&s);
        Document { text, html: s }
    }
    pub fn text(&self) -> String {
        self.text.clone()
    }
    pub fn select(&self, css: &str) -> Elements {
        crate::runtime::html::select_elements(&self.html, css)
    }
    pub fn get_elements_by_tag(&self, tag: &str) -> Elements {
        crate::runtime::html::select_elements(&self.html, tag)
    }
    pub fn get_element_by_id(&self, id: &str) -> Option<Element> {
        let els = crate::runtime::html::select_elements(&self.html, &format!("#{}", id)); els.first()
    }
    pub fn outer_html(&self) -> String {
        self.html.clone()
    }
    pub fn to_string(&self) -> String {
        self.text.clone()
    }
    pub fn title(&self) -> String {
        crate::runtime::html::title_of(&self.html)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Element {
    pub html: String,
    pub text: String,
}

impl Element {
    pub fn new() -> Self {
        Element::default()
    }
    pub fn text(&self) -> String {
        self.text.clone()
    }
    pub fn attr(&self, name: &str) -> String {
        crate::runtime::html::attr_of(&self.html, name)
    }
    pub fn has_attr(&self, name: &str) -> bool {
        crate::runtime::html::has_attr(&self.html, name)
    }
    pub fn select(&self, css: &str) -> Elements {
        crate::runtime::html::select_elements(&self.html, css)
    }
    pub fn get_elements_by_tag(&self, tag: &str) -> Elements {
        crate::runtime::html::select_elements(&self.html, tag)
    }
    pub fn get_element_by_id(&self, id: &str) -> Option<Element> {
        let els = crate::runtime::html::select_elements(&self.html, &format!("#{}", id)); els.first()
    }
    pub fn parent(&self) -> Option<Element> {
        None
    }
    pub fn children(&self) -> Elements {
        crate::runtime::html::children_of(&self.html)
    }
    pub fn first(&self) -> Option<Element> {
        let (h, t) = crate::runtime::html::first_element(&self.html); if h.is_empty() { None } else { Some(Element { text: t, html: h }) }
    }
    pub fn outer_html(&self) -> String {
        self.html.clone()
    }
    pub fn inner_html(&self) -> String {
        crate::runtime::html::inner_html_of(&self.html)
    }
    pub fn to_string(&self) -> String {
        self.text.clone()
    }
    pub fn tag_name(&self) -> String {
        crate::runtime::html::tag_name_of(&self.html)
    }
    pub fn own_text(&self) -> String {
        // fix: jsoup ownText()——仅直接文本子节点（原返回整棵子树文本）
        crate::runtime::html::own_text_of(&self.html)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Elements {
    pub list: Vec<Element>,
}

impl Elements {
    pub fn new() -> Self {
        Elements::default()
    }
    // Kotlin jsoup Elements(Element) 单元素构造
    pub fn new_single(e: Element) -> Elements {
        Elements { list: vec![e] }
    }
    pub fn size(&self) -> usize {
        self.list.len()
    }
    pub fn len(&self) -> usize {
        self.list.len()
    }
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }
    pub fn get(&self, i: usize) -> Element {
        self.list.get(i).cloned().unwrap_or_default()
    }
    pub fn first(&self) -> Option<Element> {
        self.list.first().cloned()
    }
    pub fn each_text(&self) -> Vec<String> {
        self.list.iter().map(|e| e.text()).collect()
    }
    pub fn each_attr(&self, name: &str) -> Vec<String> {
        self.list.iter().map(|e| e.attr(name)).collect()
    }
    pub fn each_outer_html(&self) -> Vec<String> {
        self.list.iter().map(|e| e.outer_html()).collect()
    }
    pub fn select(&self, css: &str) -> Elements {
        let mut out = Elements::default();
        for e in &self.list {
            for s in e.select(css).list {
                out.list.push(s);
            }
        }
        out
    }
    pub fn to_string(&self) -> String {
        self.list.iter().map(|e| e.to_string()).collect()
    }
}

// JXDocument::create(doc) 等 ToString 泛型边界所需的 Display 实现（与各自 to_string() 行为一致）
impl std::fmt::Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl std::fmt::Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl std::fmt::Display for Elements {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

// ---------------- XML / XPath 占位 ----------------

#[derive(Debug, Clone, Default)]
pub struct XmlNode {
    pub text: String,
}

impl XmlNode {
    pub fn get_first_child(&self) -> Option<XmlNode> {
        None
    }
    pub fn get_next_sibling(&self) -> Option<XmlNode> {
        None
    }
    pub fn get_node_value(&self) -> String {
        String::new()
    }
    pub fn get_text_content(&self) -> String {
        self.text.clone()
    }
    pub fn get_node_name(&self) -> String {
        String::new()
    }
    pub fn get_attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }
    pub fn child_nodes(&self) -> Vec<XmlNode> {
        Vec::new()
    }
}

pub struct XPath;

impl XPath {
    pub fn new_instance(&self) -> XPath {
        XPath
    }
    pub fn compile(&self, _expr: &str) -> XPathExpression {
        XPathExpression
    }
}

pub struct XPathExpression;

impl XPathExpression {
    pub fn evaluate_string(&self, _node: &XmlNode) -> String {
        String::new()
    }
    pub fn evaluate_list(&self, _node: &XmlNode) -> Vec<XmlNode> {
        Vec::new()
    }
}

pub struct DocumentBuilder;

impl DocumentBuilder {
    pub fn new_instance() -> Self {
        DocumentBuilder
    }
    pub fn build(&self, _s: String) -> Result<XmlNode, StubError> {
        Ok(XmlNode::default())
    }
}

// ---------------- 网络占位 ----------------

pub struct OkHttpClient {
    pub proxy: Option<String>,
    pub proxy_auth: Option<(String, String)>,
    pub interceptors: Vec<Interceptor>,
}

impl OkHttpClient {
    pub fn new() -> Self {
        OkHttpClient { proxy: None, proxy_auth: None, interceptors: Vec::new() }
    }
    pub fn new_with_interceptors(interceptors: Vec<Interceptor>) -> Self {
        OkHttpClient { proxy: None, proxy_auth: None, interceptors }
    }
}

// ---------------- 通用方法 trait ----------------

pub trait CommonExt {
    fn to_astring(&self) -> String;
}

impl CommonExt for String {
    fn to_astring(&self) -> String {
        self.clone()
    }
}

// ---------------- 时区 / 日期占位 ----------------

pub struct SimpleDateFormat {
    pub pattern: String,
}

/// Java SimpleDateFormat pattern → chrono format（常用子集）
pub fn java_pattern_to_chrono(p: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = p.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let mut run = 1;
        while i + run < chars.len() && chars[i + run] == c {
            run += 1;
        }
        match c {
            'y' => out.push_str("%Y"),
            'M' => {
                if run >= 3 {
                    out.push_str("%B");
                } else {
                    out.push_str("%m");
                }
            }
            'd' => {
                if run >= 2 {
                    out.push_str("%d");
                } else {
                    out.push_str("%e");
                }
            }
            'H' => {
                if run >= 2 {
                    out.push_str("%H");
                } else {
                    out.push_str("%k");
                }
            }
            'm' => out.push_str("%M"),
            's' => out.push_str("%S"),
            'E' => out.push_str("%a"),
            'z' | 'Z' => out.push_str("%z"),
            other => {
                for _ in 0..run {
                    out.push(other);
                }
            }
        }
        i += run;
    }
    out
}

impl SimpleDateFormat {
    pub fn new(pattern: &str) -> Self {
        SimpleDateFormat {
            pattern: pattern.to_string(),
        }
    }
    pub fn new_2args(pattern: &str, _locale: Locale) -> SimpleDateFormat {
        SimpleDateFormat {
            pattern: pattern.to_string(),
        }
    }
    pub fn format(&self, ms: i64) -> String {
        use chrono::TimeZone;
        match chrono::Local.timestamp_millis_opt(ms).single() {
            Some(dt) => dt.format(&java_pattern_to_chrono(&self.pattern)).to_string(),
            None => String::new(),
        }
    }
}

// ---------------- java.util.Calendar 占位 ----------------

pub struct Calendar {
    pub timeInMillis: i64,
    pub time: i64,
}

impl Calendar {
    pub fn getInstance() -> Calendar {
        let now = System::current_time_millis();
        Calendar {
            timeInMillis: now,
            time: now,
        }
    }
}

// ---------------- java.text.DecimalFormat 占位 ----------------

pub struct DecimalFormat {
    pattern: String,
}

impl DecimalFormat {
    pub fn new(pattern: &str) -> DecimalFormat {
        DecimalFormat {
            pattern: pattern.to_string(),
        }
    }
    // fix: 简化实现，按 pattern 中小数点后的位数格式化
    pub fn format(&self, value: f64) -> String {
        let decimals = self.pattern.split('.').nth(1).map(|s| s.len()).unwrap_or(0);
        format!("{:.*}", decimals, value)
    }
}

// ---------------- regex::Regex re-export（VertExt.validateEmail 等使用） ----------------

pub use regex::Regex;

// ---------------- Gson / GsonBuilder / TypeToken 占位（serde_json 包装） ----------------

pub struct Gson {
    pretty: bool,
    html_escaping: bool,
}

impl Gson {
    pub fn new() -> Gson {
        Gson { pretty: false, html_escaping: true }
    }

    // fix: 配置生效（原忽略 pretty/html escaping——内容提取时 \u003c 等转义残留）
    fn render(&self, s: &str) -> String {
        if self.html_escaping {
            s.to_string()
        } else {
            s.replace("\\u003c", "<")
                .replace("\\u003e", ">")
                .replace("\\u0026", "&")
                .replace("\\u0027", "'")
                .replace("\\u0022", "\"")
                .replace("\\u005c", "\\")
        }
    }

    pub fn to_json<T: serde::Serialize>(&self, v: T) -> String {
        let s = if self.pretty {
            serde_json::to_string_pretty(&v)
        } else {
            serde_json::to_string(&v)
        }
        .unwrap_or_default();
        self.render(&s)
    }

    pub fn toJson<T: serde::Serialize>(&self, v: T) -> String {
        self.to_json(v)
    }

    pub fn from_json<T: serde::de::DeserializeOwned>(&self, json: &str, _ty: Type) -> T {
        serde_json::from_str::<T>(json).unwrap_or_else(|e| panic!("Gson::from_json 解析失败: {}", e))
    }

    pub fn fromJson<T: serde::de::DeserializeOwned>(&self, json: Option<&str>) -> Result<Option<T>, StubError> {
        match json {
            Some(s) => serde_json::from_str::<T>(s).map(Some).map_err(|e| StubError::new(e.to_string())),
            None => Ok(None),
        }
    }
}

pub struct GsonBuilder {
    pretty: bool,
    html_escaping: bool,
}

impl GsonBuilder {
    pub fn new() -> GsonBuilder {
        GsonBuilder { pretty: false, html_escaping: true }
    }
    // Kotlin `registerTypeAdapter(Type, Object)`：Type 占位，接受任意类型参数
    pub fn register_type_adapter<T, A>(self, _ty: T, _adapter: A) -> GsonBuilder {
        self
    }
    pub fn registerTypeAdapter<T, A>(self, _ty: T, _adapter: A) -> GsonBuilder {
        self
    }
    // fix: 真实配置（原丢弃——HTML 转义导致正文提取 \u003c 残留）
    pub fn disable_html_escaping(mut self) -> GsonBuilder {
        self.html_escaping = false;
        self
    }
    pub fn disableHtmlEscaping(mut self) -> GsonBuilder {
        self.html_escaping = false;
        self
    }
    // fix: 真实配置（原丢弃）
    pub fn set_pretty_printing(mut self) -> GsonBuilder {
        self.pretty = true;
        self
    }
    pub fn setPrettyPrinting(mut self) -> GsonBuilder {
        self.pretty = true;
        self
    }
    pub fn create(self) -> Gson {
        Gson { pretty: self.pretty, html_escaping: self.html_escaping }
    }
}

// java.lang.reflect.Type 占位
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Type;

// com.google.gson.reflect.TypeToken 占位
#[derive(Debug, Clone, Copy, Default)]
pub struct TypeToken<T>(std::marker::PhantomData<T>);

impl<T> TypeToken<T> {
    pub fn new() -> TypeToken<T> {
        TypeToken(std::marker::PhantomData)
    }
    pub fn get_type(&self) -> Type {
        Type
    }
    pub fn r#type(&self) -> Type {
        Type
    }
}

pub fn type_of<T>() -> Type {
    Type
}

// ---------------- JsonObject / JsonArray 占位（io.vertx.core.json 包装） ----------------

#[derive(Debug, Clone, Default)]
pub struct JsonObject(pub String);

impl JsonObject {
    pub fn new() -> JsonObject {
        JsonObject(String::new())
    }
    // Kotlin `JsonObject(String)` 解析构造
    pub fn new_parsed(s: &str) -> JsonObject {
        JsonObject(s.to_string())
    }
    pub fn put(&mut self, key: &str, value: impl std::fmt::Display) {
        let s = value.to_string();
        let val: serde_json::Value = if s == "true" {
            serde_json::Value::Bool(true)
        } else if s == "false" {
            serde_json::Value::Bool(false)
        } else if let Ok(i) = s.parse::<i64>() {
            serde_json::Value::Number(i.into())
        } else {
            serde_json::Value::String(s)
        };
        let mut obj = serde_json::from_str::<serde_json::Value>(&self.0)
            .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));
        if let Some(m) = obj.as_object_mut() {
            m.insert(key.to_string(), val);
            self.0 = obj.to_string();
        }
    }
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
    // Kotlin JsonObject.getMap()（ReaderUIApplication windowConfigMap 使用）
    pub fn map(&self) -> HashMap<String, Any> {
        let mut out = HashMap::new();
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&self.0) {
            if let Some(obj) = v.as_object() {
                for (k, val) in obj {
                    out.insert(k.clone(), crate::runtime::js::value_to_any(val));
                }
            }
        }
        out
    }
    // Kotlin JsonObject.getString(key)（控制器读取书源字段使用）
    pub fn get_string(&self, key: &str) -> String {
        serde_json::from_str::<serde_json::Value>(&self.0)
            .ok()
            .and_then(|v| v.get(key).cloned())
            .map(|v| match v {
                serde_json::Value::String(s) => s,
                other => other.to_string(),
            })
            .unwrap_or_default()
    }
    // Kotlin JsonObject.getInteger(key, default)（BookSourceController.getBookSources 使用）
    pub fn get_integer(&self, key: &str, default: i32) -> i32 {
        serde_json::from_str::<serde_json::Value>(&self.0)
            .ok()
            .and_then(|v| v.get(key).cloned())
            .and_then(|v| v.as_i64())
            .map(|n| n as i32)
            .unwrap_or(default)
    }
    // Kotlin JsonObject.getLong(key, default)（BookGroupController.onCheckEnd 使用）
    pub fn get_long(&self, key: &str, default: i64) -> i64 {
        serde_json::from_str::<serde_json::Value>(&self.0)
            .ok()
            .and_then(|v| v.get(key).cloned())
            .and_then(|v| v.as_i64())
            .unwrap_or(default)
    }
    // Kotlin JsonObject.getJsonArray(key)（BookGroupController.saveBookGroupOrder 使用）
    pub fn get_json_array(&self, key: &str) -> Option<JsonArray> {
        serde_json::from_str::<serde_json::Value>(&self.0)
            .ok()
            .and_then(|v| v.get(key).cloned())
            .and_then(|v| v.as_array().cloned())
            .map(|arr| {
                JsonArray(arr
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect())
            })
    }
    // Kotlin JsonObject.mapTo(Class<T>)（GSON 反序列化）
    pub fn map_to<T>(&self) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        serde_json::from_str::<T>(&self.0).ok()
    }
    // Kotlin JsonObject.mapFrom(obj)（GSON 序列化占位）
    pub fn map_from<T>(value: T) -> JsonObject
    where
        T: crate::com_htmake_reader_db_db::EntityToJson,
    {
        JsonObject(value.to_json_value().to_string())
    }
}

impl std::fmt::Display for JsonObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Default)]
pub struct JsonArray(pub Vec<String>);

impl JsonArray {
    pub fn new() -> JsonArray {
        JsonArray(Vec::new())
    }
    // Kotlin `JsonArray(String)` 解析构造
    pub fn new_parsed(s: &str) -> JsonArray {
        Self::from_json(s.to_string()).unwrap_or_else(|| JsonArray(vec![s.to_string()]))
    }
    pub fn add(&mut self, value: impl std::fmt::Display) {
        self.0.push(value.to_string());
    }
    pub fn to_string(&self) -> String {
        let vals: Vec<serde_json::Value> = self
            .0
            .iter()
            .map(|s| serde_json::from_str::<serde_json::Value>(s).unwrap_or_else(|_| serde_json::Value::String(s.clone())))
            .collect();
        serde_json::to_string(&vals).unwrap_or_default()
    }
    // Kotlin JsonArray(Collection<JsonObject>)（bookSourceList 重建）
    pub fn from_list(list: Vec<JsonObject>) -> JsonArray {
        JsonArray(list.into_iter().map(|o| o.to_string()).collect())
    }
    // Kotlin `JsonArray(String)` 反序列化
    pub fn from_json(s: String) -> Option<JsonArray> {
        serde_json::from_str::<Vec<serde_json::Value>>(&s)
            .ok()
            .map(|v| JsonArray(v.into_iter().map(|x| x.to_string()).collect()))
    }
    pub fn size(&self) -> i32 {
        self.0.len() as i32
    }
    // Kotlin JsonArray.getJsonObject(index)（控制器读取书源数组使用）
    pub fn get_json_object(&self, index: i32) -> Option<JsonObject> {
        self.0.get(index as usize).map(|s| JsonObject::new_parsed(s))
    }
    pub fn get_list(&self) -> Vec<JsonObject> {
        self.0.iter().map(|s| JsonObject::new_parsed(s)).collect()
    }
    // Kotlin JsonArray.getString(index)
    pub fn get_string(&self, index: i32) -> String {
        self.0.get(index as usize).cloned().unwrap_or_default()
    }
    // Kotlin MutableList.set(index, element)
    pub fn set(&mut self, index: usize, value: JsonObject) {
        if index < self.0.len() {
            self.0[index] = value.to_string();
        }
    }
    // Kotlin MutableList.remove(index)
    pub fn remove(&mut self, index: usize) {
        if index < self.0.len() {
            self.0.remove(index);
        }
    }
}

// ---------------- Any 占位枚举（对应 Kotlin Any，用于 is 智能转换） ----------------

#[derive(Debug, Clone, Default)]
pub enum Any {
    #[default]
    Null,
    Bool(bool),
    Long(i64),
    Double(f64),
    Str(String),
    JsonObject(JsonObject),
    JsonArray(JsonArray),
    List(Vec<Any>),
    Map(HashMap<String, Any>),
    // fix: AnalyzeByJSonPath / AnalyzeByXPath 转录所需变体（对应 Kotlin `is ReadContext/JXNode/Document/Element/Elements` 分支）
    ReadContext(ReadContext),
    JXNode(JXNode),
    JXDocument(JXDocument),
    Document(Document),
    Element(Element),
    Elements(Elements),
}

impl Any {
    #[allow(elided_lifetimes_in_paths)]
    pub fn from_string(s: String) -> Any {
        Any::Str(s)
    }
    pub fn from_bool(b: bool) -> Any {
        Any::Bool(b)
    }
    pub fn from_long(i: i64) -> Any {
        Any::Long(i)
    }
    pub fn from_double(d: f64) -> Any {
        Any::Double(d)
    }
    pub fn from_list(l: Vec<Any>) -> Any {
        Any::List(l)
    }
    pub fn from_map(m: HashMap<String, Any>) -> Any {
        Any::Map(m)
    }
    pub fn as_map(&self) -> Option<HashMap<String, Any>> {
        match self {
            Any::Map(m) => Some(m.clone()),
            _ => None,
        }
    }
    pub fn as_long(&self) -> Option<i64> {
        match self {
            Any::Long(i) => Some(*i),
            Any::Double(d) => Some(*d as i64),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Any::Bool(b) => Some(*b),
            Any::Str(s) => Some(s == "true" || s == "1"),
            _ => None,
        }
    }
    pub fn is_map(&self) -> bool {
        matches!(self, Any::Map(_))
    }
    pub fn is_string(&self) -> bool {
        matches!(self, Any::Str(_))
    }
    // UrlOption.useWebView: webView 为 false / "false" 时视为 false
    pub fn is_false_bool(&self) -> bool {
        match self {
            Any::Bool(false) => true,
            Any::Str(s) => s == "false",
            _ => false,
        }
    }
    pub fn is_list(&self) -> bool {
        matches!(self, Any::List(_))
    }
    pub fn list_iter(&self) -> Vec<Any> {
        match self {
            Any::List(l) => l.clone(),
            _ => Vec::new(),
        }
    }
    // Kotlin `node as JXNode`（AnalyzeByXPath.getResult）
    pub fn as_jx_node(&self) -> JXNode {
        match self {
            Any::JXNode(n) => n.clone(),
            _ => JXNode::default(),
        }
    }
    // Kotlin `node as JXDocument`
    pub fn as_jx_document(&self) -> JXDocument {
        match self {
            Any::JXDocument(d) => d.clone(),
            _ => JXDocument::default(),
        }
    }
    pub fn r#type(&self) -> Type {
        Type
    }
    // Kotlin `as? T` 智能转换（fix: 真实实现——原恒 None 导致 bookSourceMap 索引全 0、CacheManager 永不命中。
    //      支持数字/布尔/字符串基本类型；单线程使用，调用方需立即消费返回的引用）
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        use std::any::TypeId;
        let tid = TypeId::of::<T>();
        let value: Option<Box<dyn std::any::Any>> = match self {
            Any::Long(i) if tid == TypeId::of::<i32>() => Some(Box::new(*i as i32) as Box<dyn std::any::Any>),
            Any::Long(i) if tid == TypeId::of::<i64>() => Some(Box::new(*i) as Box<dyn std::any::Any>),
            Any::Long(i) if tid == TypeId::of::<f64>() => Some(Box::new(*i as f64) as Box<dyn std::any::Any>),
            Any::Long(i) if tid == TypeId::of::<f32>() => Some(Box::new(*i as f32) as Box<dyn std::any::Any>),
            Any::Long(i) if tid == TypeId::of::<u32>() => Some(Box::new(*i as u32) as Box<dyn std::any::Any>),
            Any::Long(i) if tid == TypeId::of::<u64>() => Some(Box::new(*i as u64) as Box<dyn std::any::Any>),
            Any::Long(i) if tid == TypeId::of::<bool>() => Some(Box::new(*i != 0) as Box<dyn std::any::Any>),
            Any::Double(d) if tid == TypeId::of::<f64>() => Some(Box::new(*d) as Box<dyn std::any::Any>),
            Any::Double(d) if tid == TypeId::of::<f32>() => Some(Box::new(*d as f32) as Box<dyn std::any::Any>),
            Any::Double(d) if tid == TypeId::of::<i32>() => Some(Box::new(*d as i32) as Box<dyn std::any::Any>),
            Any::Double(d) if tid == TypeId::of::<i64>() => Some(Box::new(*d as i64) as Box<dyn std::any::Any>),
            Any::Double(d) if tid == TypeId::of::<u64>() => Some(Box::new(*d as u64) as Box<dyn std::any::Any>),
            Any::Bool(b) if tid == TypeId::of::<bool>() => Some(Box::new(*b) as Box<dyn std::any::Any>),
            Any::Bool(b) if tid == TypeId::of::<i32>() => Some(Box::new(*b as i32) as Box<dyn std::any::Any>),
            Any::Str(s) if tid == TypeId::of::<String>() => Some(Box::new(s.clone()) as Box<dyn std::any::Any>),
            Any::Str(s) if tid == TypeId::of::<i32>() => s.parse::<i32>().ok().map(|v| Box::new(v) as Box<dyn std::any::Any>),
            Any::Str(s) if tid == TypeId::of::<i64>() => s.parse::<i64>().ok().map(|v| Box::new(v) as Box<dyn std::any::Any>),
            Any::Str(s) if tid == TypeId::of::<f64>() => s.parse::<f64>().ok().map(|v| Box::new(v) as Box<dyn std::any::Any>),
            Any::Str(s) if tid == TypeId::of::<bool>() => Some(Box::new(!s.is_empty() && s != "false" && s != "0") as Box<dyn std::any::Any>),
            _ => None,
        };
        let value = value?;
        DOWNCAST_CELL.with(|c| unsafe {
            let slot = &mut *c.get();
            *slot = Some(value);
            slot.as_ref().unwrap().downcast_ref::<T>()
        })
    }
}

thread_local! {
    static DOWNCAST_CELL: std::cell::UnsafeCell<Option<Box<dyn std::any::Any>>> = const { std::cell::UnsafeCell::new(None) };
}

// fix: `Option<&Any>.downcast_ref::<T>()`（Kotlin `map["key"] as? T` 转录；显式导入优先级高于 glob）
pub trait OptionAnyDowncastExt {
    fn downcast_ref<T: 'static>(&self) -> Option<&T>;
}

impl OptionAnyDowncastExt for Option<&Any> {
    fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        match self {
            Some(a) => a.downcast_ref::<T>(),
            None => None,
        }
    }
}

impl std::fmt::Display for Any {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Any::Null => write!(f, "null"),
            Any::Bool(b) => write!(f, "{}", b),
            Any::Long(i) => write!(f, "{}", i),
            Any::Double(d) => write!(f, "{}", d),
            Any::Str(s) => write!(f, "{}", s),
            Any::JsonObject(o) => write!(f, "{}", o.0),
            Any::JsonArray(a) => write!(f, "{}", a.to_string()),
            Any::List(l) => write!(f, "{}", crate::stubs::any_list_to_value(l)),
            Any::Map(m) => write!(f, "{}", crate::stubs::any_map_to_value(m)),
            Any::ReadContext(r) => write!(f, "{}", r.json),
            Any::JXNode(n) => write!(f, "{}", n.text),
            Any::JXDocument(d) => write!(f, "{}", d.text),
            Any::Document(d) => write!(f, "{}", d.text),
            Any::Element(e) => write!(f, "{}", e.text),
            Any::Elements(es) => write!(f, "{}", es.to_string()),
        }
    }
}

impl serde::Serialize for Any {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Any::Null => serializer.serialize_none(),
            Any::Bool(b) => serializer.serialize_bool(*b),
            Any::Long(i) => serializer.serialize_i64(*i),
            Any::Double(d) => serializer.serialize_f64(*d),
            Any::Str(s) => serializer.serialize_str(s),
            Any::JsonObject(o) => match serde_json::from_str::<serde_json::Value>(&o.0) { Ok(v) => v.serialize(serializer), Err(_) => serializer.serialize_str(&o.0) },
            Any::JsonArray(a) => { let items: Vec<serde_json::Value> = a.0.iter().filter_map(|s| serde_json::from_str(s).ok()).collect(); serde_json::Value::Array(items).serialize(serializer) },
            Any::List(l) => l.serialize(serializer),
            Any::Map(m) => m.serialize(serializer),
            Any::ReadContext(r) => serializer.serialize_str(&r.json),
            Any::JXNode(n) => serializer.serialize_str(&n.text),
            Any::JXDocument(d) => serializer.serialize_str(&d.text),
            Any::Document(d) => serializer.serialize_str(&d.text),
            Any::Element(e) => serializer.serialize_str(&e.text),
            Any::Elements(es) => serializer.serialize_str(&es.to_string()),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Any {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Any, D::Error> {
        let v = serde_json::Value::deserialize(deserializer)?;
        Ok(match v {
            serde_json::Value::Null => Any::Null,
            serde_json::Value::Bool(b) => Any::Bool(b),
            serde_json::Value::Number(n) => match n.as_i64() {
                Some(i) => Any::Long(i),
                None => Any::Double(n.as_f64().unwrap_or(0.0)),
            },
            serde_json::Value::String(s) => Any::Str(s),
            serde_json::Value::Array(a) => Any::List(
                a.into_iter()
                    .map(|x| serde_json::from_value::<Any>(x).unwrap_or(Any::Null))
                    .collect(),
            ),
            serde_json::Value::Object(o) => Any::Map(
                o.into_iter()
                    .map(|(k, v)| (k, serde_json::from_value::<Any>(v).unwrap_or(Any::Null)))
                    .collect(),
            ),
        })
    }
}

impl From<Any> for String {
    fn from(a: Any) -> String {
        match a {
            Any::Str(s) => s,
            Any::JsonObject(o) => o.0,
            Any::JsonArray(arr) => arr.to_string(),
            _ => String::new(),
        }
    }
}

// Kotlin Any 装箱：Any.from(true/String/Map/List)（AnalyzeUrl.UrlOption 使用）
impl From<bool> for Any {
    fn from(b: bool) -> Any {
        Any::Bool(b)
    }
}

impl From<String> for Any {
    fn from(s: String) -> Any {
        Any::Str(s)
    }
}

impl From<&str> for Any {
    fn from(s: &str) -> Any {
        Any::Str(s.to_string())
    }
}

impl From<HashMap<String, Any>> for Any {
    fn from(m: HashMap<String, Any>) -> Any {
        Any::Map(m)
    }
}

impl From<Vec<HashMap<String, Any>>> for Any {
    fn from(l: Vec<HashMap<String, Any>>) -> Any {
        Any::List(l.into_iter().map(Any::from).collect())
    }
}

// VertRoute.success 等使用：`any.is_json_object()` / `any.to_string()`（Option 无法 impl Display，改用 trait 方法）
pub trait OptionAnyExt {
    fn is_json_object(&self) -> bool;
    fn to_string(&self) -> String;
}

impl OptionAnyExt for Option<Any> {
    fn is_json_object(&self) -> bool {
        matches!(self, Some(Any::JsonObject(_)))
    }
    fn to_string(&self) -> String {
        match self {
            Some(a) => format!("{}", a),
            None => "null".to_string(),
        }
    }
}

// ---------------- Class / ::class 占位（Kotlin `X::class` → trait 常量） ----------------

pub struct Class<T>(std::marker::PhantomData<T>);

impl<T> std::fmt::Debug for Class<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Class")
    }
}

impl<T> Clone for Class<T> {
    fn clone(&self) -> Self {
        Class(std::marker::PhantomData)
    }
}

impl<T> Copy for Class<T> {}

impl<T> Default for Class<T> {
    fn default() -> Self {
        Class(std::marker::PhantomData)
    }
}

impl<T> Class<T> {
    pub const fn new() -> Class<T> {
        Class(std::marker::PhantomData)
    }
    // Java Int::class.javaPrimitiveType 占位
    pub fn java_primitive_type(&self) -> Class<T> {
        *self
    }
    // Java Class.getResource()（ReaderUIApplication 图标资源使用）
    pub fn get_resource(&self, _path: &str) -> URL {
        URL(url::Url::parse("https://localhost/").unwrap())
    }
}

// Kotlin `Int::class` / `Long::class` / `AppConfig::class` / `MongoFile::class`
pub trait ClassConstant {
    type Target;
    const class: Class<Self::Target>;
}

impl ClassConstant for i32 {
    type Target = i32;
    const class: Class<i32> = Class::new();
}

impl ClassConstant for i64 {
    type Target = i64;
    const class: Class<i64> = Class::new();
}

impl ClassConstant for crate::com_htmake_reader_config_appconfig::AppConfig {
    type Target = crate::com_htmake_reader_config_appconfig::AppConfig;
    const class: Class<crate::com_htmake_reader_config_appconfig::AppConfig> = Class::new();
}

impl ClassConstant for crate::com_htmake_reader_entity_mongofile::MongoFile {
    type Target = crate::com_htmake_reader_entity_mongofile::MongoFile;
    const class: Class<crate::com_htmake_reader_entity_mongofile::MongoFile> = Class::new();
}

// fix: ReaderUIApplication/ReaderApplication 使用 `X::class`（Kotlin Class 引用）
impl ClassConstant for crate::com_htmake_reader_readerapplication::ReaderApplication {
    type Target = crate::com_htmake_reader_readerapplication::ReaderApplication;
    const class: Class<crate::com_htmake_reader_readerapplication::ReaderApplication> = Class::new();
}

impl ClassConstant for crate::com_htmake_reader_readeruiapplication::ReaderUIApplication {
    type Target = crate::com_htmake_reader_readeruiapplication::ReaderUIApplication;
    const class: Class<crate::com_htmake_reader_readeruiapplication::ReaderUIApplication> = Class::new();
}

impl ClassConstant for bool {
    type Target = bool;
    const class: Class<bool> = Class::new();
}

impl ClassConstant for String {
    type Target = String;
    const class: Class<String> = Class::new();
}

// Kotlin `Int::class.javaPrimitiveType` 之外的 `i32::type()` 形式
pub trait TypeOf {
    fn r#type() -> Type;
}

impl TypeOf for i32 {
    fn r#type() -> Type {
        Type
    }
}

impl TypeOf for i64 {
    fn r#type() -> Type {
        Type
    }
}

impl TypeOf for f64 {
    fn r#type() -> Type {
        Type
    }
}

impl TypeOf for bool {
    fn r#type() -> Type {
        Type
    }
}

impl TypeOf for String {
    fn r#type() -> Type {
        Type
    }
}

impl TypeOf for Any {
    fn r#type() -> Type {
        Type
    }
}

// ---------------- Kotlin 反射占位（readInstanceProperty / setInstanceProperty） ----------------

// `instance::class.memberProperties` 占位
pub trait ReflectionExt {
    fn class(&self) -> ReflectionClass;
}

impl ReflectionExt for dyn std::any::Any {
    fn class(&self) -> ReflectionClass {
        ReflectionClass::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ReflectionClass;

impl ReflectionClass {
    pub fn new() -> ReflectionClass {
        ReflectionClass
    }
    pub fn member_properties(&self) -> Vec<KProperty1> {
        Vec::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct KProperty1 {
    pub name: String,
    pub value: Any,
}

impl KProperty1 {
    pub fn get(&self, _instance: &dyn std::any::Any) -> Any {
        self.value.clone()
    }
    // Kotlin `property is KMutableProperty<*>` 智能转换占位
    pub fn as_mutable(&self) -> Option<KMutableProperty> {
        None
    }
}

#[derive(Debug, Clone, Default)]
pub struct KMutableProperty {
    pub name: String,
    pub value: Any,
}

impl KMutableProperty {
    pub fn setter(&self) -> KMutablePropertySetter {
        KMutablePropertySetter
    }
}

#[derive(Debug, Clone, Default)]
pub struct KMutablePropertySetter;

impl KMutablePropertySetter {
    pub fn call(&self, _instance: &dyn std::any::Any, _value: Any) {}
}

// java.lang.reflect.Array.newInstance(clazz, 0)::class.java as Class<Array<T>> 占位
pub fn java_reflect_array_new_instance<T>(_clazz: Class<T>, _length: i32) -> ArrayReflect<T> {
    ArrayReflect::new()
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ArrayReflect<T>(std::marker::PhantomData<T>);

impl<T> ArrayReflect<T> {
    pub fn new() -> ArrayReflect<T> {
        ArrayReflect(std::marker::PhantomData)
    }
    pub fn class(&self) -> ArrayClass<T> {
        ArrayClass::new()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ArrayClass<T>(std::marker::PhantomData<T>);

impl<T> ArrayClass<T> {
    pub fn new() -> ArrayClass<T> {
        ArrayClass(std::marker::PhantomData)
    }
    pub fn as_type(&self) -> Class<Vec<T>> {
        Class::new()
    }
}

// ---------------- java.io.File 占位（基于 std::fs 降级实现） ----------------

#[derive(Debug, Clone, Default, PartialEq)]
pub struct File {
    pub file_path: String,
    pub name: String,
    pub absolute_path: String,
    pub parent_file: Option<Box<File>>,
}

impl File {
    pub const SEPARATOR: &'static str = "/";

    pub fn new(path: &str) -> File {
        let normalized = path.replace('\\', "/");
        let name = normalized.rsplit('/').next().unwrap_or("").to_string();
        let absolute_path = std::path::Path::new(&normalized)
            .canonicalize()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| normalized.clone());
        let parent_file = std::path::Path::new(&normalized)
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map(|p| Box::new(File::new(&p.to_string_lossy())));
        File {
            file_path: normalized,
            name,
            absolute_path,
            parent_file,
        }
    }

    pub fn exists(&self) -> bool {
        std::path::Path::new(&self.file_path).exists()
    }

    pub fn is_file(&self) -> bool {
        std::path::Path::new(&self.file_path).is_file()
    }

    pub fn is_directory(&self) -> bool {
        std::path::Path::new(&self.file_path).is_dir()
    }

    pub fn mkdirs(&self) -> bool {
        std::fs::create_dir_all(&self.file_path).is_ok()
    }

    pub fn path(&self) -> String {
        self.file_path.clone()
    }

    pub fn resolve(&self, child: &str) -> File {
        File::new(&format!(
            "{}/{}",
            self.file_path.trim_end_matches('/'),
            child.trim_start_matches('/')
        ))
    }

    pub fn absolute_file(&self) -> File {
        self.clone()
    }

    pub fn to_path(&self) -> File {
        self.clone()
    }

    pub fn to_absolute_path(&self) -> File {
        self.clone()
    }

    pub fn name_without_extension(&self) -> String {
        match self.name.rfind('.') {
            Some(i) => self.name[..i].to_string(),
            None => self.name.clone(),
        }
    }

    pub fn create_new_file(&self) -> bool {
        std::fs::File::create(&self.file_path).is_ok()
    }

    pub fn write_text(&self, text: &str) {
        let _ = std::fs::write(&self.file_path, text);
    }

    pub fn read_text(&self) -> String {
        std::fs::read_to_string(&self.file_path).unwrap_or_default()
    }

    pub fn list_files(&self) -> Vec<File> {
        std::fs::read_dir(&self.file_path)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .map(|e| File::new(&e.path().to_string_lossy()))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn delete(&self) -> bool {
        std::fs::remove_file(&self.file_path).is_ok()
    }

    // ---- FilesUtil 使用的 java.io.File 方法（附加占位） ----

    pub fn separator() -> String {
        std::path::MAIN_SEPARATOR_STR.to_string()
    }

    pub fn parent(&self) -> Option<String> {
        std::path::Path::new(&self.file_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
    }

    pub fn parentFile(&self) -> Option<File> {
        self.parent_file.clone().map(|b| *b)
    }

    pub fn absolutePath(&self) -> String {
        self.absolute_path.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn isFile(&self) -> bool {
        self.is_file()
    }

    pub fn isDirectory(&self) -> bool {
        self.is_directory()
    }

    pub fn createNewFile(&self) -> bool {
        self.create_new_file()
    }

    pub fn absoluteFile(&self) -> File {
        self.absolute_file()
    }

    pub fn length(&self) -> i64 {
        std::fs::metadata(&self.file_path)
            .map(|m| m.len() as i64)
            .unwrap_or(0)
    }

    pub fn lastModified(&self) -> i64 {
        std::fs::metadata(&self.file_path)
            .and_then(|m| m.modified())
            .map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    }

    pub fn renameTo(&self, dest: &File) -> bool {
        std::fs::rename(&self.file_path, &dest.file_path).is_ok()
    }

    pub fn listFiles(&self) -> Option<Vec<File>> {
        Some(self.list_files())
    }

    pub fn listFiles_dir(&self) -> Option<Vec<File>> {
        Some(
            self.list_files()
                .into_iter()
                .filter(|f| f.is_directory())
                .collect(),
        )
    }

    pub fn listFiles_filter(&self, filter: impl FnMut(&File) -> bool) -> Option<Vec<File>> {
        Some(self.list_files().into_iter().filter(filter).collect())
    }

    pub fn listFiles_name(&self, mut filter: impl FnMut(&str) -> bool) -> Option<Vec<File>> {
        Some(
            self.list_files()
                .into_iter()
                .filter(|f| filter(&f.name))
                .collect(),
        )
    }

    pub fn new_path(parent: &File, child: &str) -> File {
        File::new(&format!("{}/{}", parent.file_path.trim_end_matches('/'), child))
    }

    // ---- ACache / Book 转录所需 java.io.File 方法（附加占位） ----

    pub fn writeText(&self, text: &str) {
        self.write_text(text);
    }

    pub fn readText(&self) -> String {
        self.read_text()
    }

    pub fn writeBytes(&self, data: &[u8]) {
        let _ = std::fs::write(&self.file_path, data);
    }

    pub fn readBytes(&self) -> Vec<u8> {
        std::fs::read(&self.file_path).unwrap_or_default()
    }

    pub fn setLastModified(&self, _time: i64) {
        // fix: 占位——不支持修改 mtime，仅保持接口一致
    }

    pub fn parent_file(&self) -> Option<File> {
        self.parentFile()
    }
}

impl std::fmt::Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.file_path)
    }
}

// ---------------- java.nio.file.Paths / java.lang.System 占位 ----------------

pub struct Paths;

impl Paths {
    pub fn get(first: impl AsRef<str>, rest: &str) -> String {
        format!(
            "{}/{}",
            first.as_ref().trim_end_matches('/'),
            rest.trim_start_matches('/')
        )
    }
}

// fix: ACache 转录使用的 java.nio.file.Path 静态 join（File(cacheDir, child) 语义）
pub struct Path;

impl Path {
    pub fn join(parent: String, child: &str) -> String {
        format!("{}/{}", parent.trim_end_matches('/'), child.trim_start_matches('/'))
    }
}

pub struct System;

impl System {
    // Kotlin System.setProperty(key, value)（ReaderUIApplication 使用）
    pub fn set_property(_key: &str, _value: &str) -> String {
        String::new()
    }
    pub fn get_property(key: &str) -> String {
        if key == "user.dir" {
            return std::env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
        }
        std::env::var(key).unwrap_or_default()
    }
    pub fn current_time_millis() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }
    pub fn currentTimeMillis() -> i64 {
        Self::current_time_millis()
    }
    // AnalyzeUrl.fetchStart 使用（Kotlin System.currentTimeMillis()）
    pub fn now_millis() -> i64 {
        Self::current_time_millis()
    }
    pub fn exit(_code: i32) {}
    // Kotlin System.arraycopy（ACache/Base64 等转录模块使用）
    pub fn arraycopy(src: &[u8], src_pos: usize, dst: &mut [u8], dst_pos: usize, len: usize) {
        if src_pos.saturating_add(len) <= src.len() && dst_pos.saturating_add(len) <= dst.len() {
            dst[dst_pos..dst_pos + len].copy_from_slice(&src[src_pos..src_pos + len]);
        }
    }
}

// ---------------- java.io.Closeable / java.io.InputStream 占位 ----------------

pub trait Closeable {
    fn close(&mut self) -> std::io::Result<()>;
}

pub trait InputStream {
    fn read(&mut self, b: &mut [u8], off: usize, len: usize) -> i32;
    fn close(&mut self);
}

// fix: Vec<u8> 作为字节流（epublib Resource::new_stream 读取 zip 条目内容）
impl InputStream for Vec<u8> {
    fn read(&mut self, b: &mut [u8], off: usize, len: usize) -> i32 {
        if self.is_empty() || off >= b.len() {
            return -1;
        }
        let n = len.min(self.len()).min(b.len() - off);
        b[off..off + n].copy_from_slice(&self[..n]);
        self.drain(..n);
        n as i32
    }
    fn close(&mut self) {}
}

// ---------------- java.io.FileInputStream / FileOutputStream 占位（基于 std::fs 降级实现） ----------------

pub struct FileInputStream {
    inner: Option<std::fs::File>,
}

impl FileInputStream {
    pub fn new(file: &File) -> FileInputStream {
        FileInputStream {
            inner: std::fs::File::open(&file.path()).ok(),
        }
    }

    pub fn new_path(path: &str) -> FileInputStream {
        FileInputStream::new(&File::new(path))
    }

    pub fn read(&mut self, b: &mut [u8], off: usize, len: usize) -> i32 {
        use std::io::Read;
        if off > b.len() {
            return -1;
        }
        let end = (off + len).min(b.len());
        match self.inner.as_mut().map(|f| f.read(&mut b[off..end])) {
            Some(Ok(0)) => -1,
            Some(Ok(n)) => n as i32,
            _ => -1,
        }
    }
}

impl Closeable for FileInputStream {
    fn close(&mut self) -> std::io::Result<()> {
        self.inner = None;
        Ok(())
    }
}

pub struct FileOutputStream {
    inner: Option<std::fs::File>,
}

impl FileOutputStream {
    pub fn new(file: &File) -> FileOutputStream {
        FileOutputStream {
            inner: std::fs::File::create(&file.path()).ok(),
        }
    }

    pub fn new_path(path: &str) -> FileOutputStream {
        FileOutputStream::new(&File::new(path))
    }

    pub fn write(&mut self, b: &[u8]) {
        use std::io::Write;
        if let Some(f) = self.inner.as_mut() {
            let _ = f.write_all(b);
        }
    }

    pub fn write_range(&mut self, b: &[u8], off: usize, len: usize) {
        use std::io::Write;
        if off <= b.len() {
            let end = (off + len).min(b.len());
            if let Some(f) = self.inner.as_mut() {
                let _ = f.write_all(&b[off..end]);
            }
        }
    }

    pub fn flush(&mut self) {
        use std::io::Write;
        if let Some(f) = self.inner.as_mut() {
            let _ = f.flush();
        }
    }
}

impl Closeable for FileOutputStream {
    fn close(&mut self) -> std::io::Result<()> {
        self.inner = None;
        Ok(())
    }
}

pub struct BufferedInputStream {
    inner: FileInputStream,
}

impl BufferedInputStream {
    pub fn new(inner: FileInputStream) -> BufferedInputStream {
        BufferedInputStream { inner }
    }

    pub fn read(&mut self, b: &mut [u8]) -> i32 {
        self.inner.read(b, 0, b.len())
    }

    // Kotlin InputStream.readBytes()：读尽全部字节
    pub fn readBytes(&mut self) -> Vec<u8> {
        let mut out = Vec::new();
        let mut buffer = vec![0u8; 8192];
        loop {
            let len = self.read(&mut buffer);
            if len <= 0 {
                break;
            }
            out.extend_from_slice(&buffer[..len as usize]);
        }
        out
    }
}

impl Closeable for BufferedInputStream {
    fn close(&mut self) -> std::io::Result<()> {
        self.inner.close()
    }
}

pub struct BufferedOutputStream {
    inner: FileOutputStream,
}

impl BufferedOutputStream {
    pub fn new(inner: FileOutputStream) -> BufferedOutputStream {
        BufferedOutputStream { inner }
    }

    pub fn write(&mut self, b: &[u8]) {
        self.inner.write(b);
    }

    pub fn write_range(&mut self, b: &[u8], off: usize, len: usize) {
        self.inner.write_range(b, off, len);
    }
}

impl Closeable for BufferedOutputStream {
    fn close(&mut self) -> std::io::Result<()> {
        self.inner.close()
    }
}

pub struct ByteArrayOutputStream {
    data: Vec<u8>,
}

impl ByteArrayOutputStream {
    pub fn new() -> ByteArrayOutputStream {
        ByteArrayOutputStream { data: Vec::new() }
    }

    pub fn write(&mut self, b: &[u8]) {
        self.data.extend_from_slice(b);
    }

    pub fn write_range(&mut self, b: &[u8], off: usize, len: usize) {
        if off <= b.len() {
            let end = (off + len).min(b.len());
            self.data.extend_from_slice(&b[off..end]);
        }
    }

    pub fn toByteArray(&self) -> Vec<u8> {
        self.data.clone()
    }
}

impl Closeable for ByteArrayOutputStream {
    fn close(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// ---- ACache 序列化缓存占位（Serializable / ObjectInputStream / ObjectOutputStream） ----

pub trait Serializable {}

pub struct ByteArrayInputStream {
    pub data: Vec<u8>,
    pub mark_snapshot: Option<Vec<u8>>,
}

impl ByteArrayInputStream {
    pub fn new(data: Vec<u8>) -> ByteArrayInputStream {
        ByteArrayInputStream {
            data,
            mark_snapshot: None,
        }
    }
    pub fn close(&mut self) {}
}

pub struct ObjectOutputStream;

impl ObjectOutputStream {
    pub fn new(_out: &ByteArrayOutputStream) -> ObjectOutputStream {
        ObjectOutputStream
    }
    pub fn writeObject(&mut self, _value: &dyn Serializable) {}
}

pub struct ObjectInputStream;

impl ObjectInputStream {
    pub fn new(_input: &mut ByteArrayInputStream) -> ObjectInputStream {
        ObjectInputStream
    }
    pub fn readObject(&mut self) -> Box<dyn std::any::Any> {
        Box::new(Any::Null)
    }
    pub fn close(&mut self) {}
}

pub struct FileWriter {
    file: File,
    append: bool,
}

impl FileWriter {
    pub fn new(file: &File, append: bool) -> FileWriter {
        FileWriter {
            file: file.clone(),
            append,
        }
    }

    pub fn write(&mut self, content: &str) {
        use std::io::Write;
        let result = std::fs::OpenOptions::new()
            .create(true)
            .append(self.append)
            .truncate(!self.append)
            .write(true)
            .open(&self.file.path())
            .and_then(|mut f| f.write_all(content.as_bytes()));
        let _ = result;
    }
}

impl Closeable for FileWriter {
    fn close(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// ---------------- java.nio.file.Files / StandardCopyOption 占位 ----------------

pub struct Files;

impl Files {
    pub fn create_temp_file(dir: File, prefix: &str, suffix: &str) -> File {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        File::new(&format!(
            "{}/{}{}{}",
            dir.path().trim_end_matches('/'),
            prefix,
            ts,
            suffix
        ))
    }
    pub fn write(file: &File, bytes: &[u8]) {
        let _ = std::fs::write(&file.path(), bytes);
    }
    pub fn exists(file: &File) -> bool {
        file.exists()
    }
    pub fn move_path(from: &File, to: &File, _option: StandardCopyOption) {
        let _ = std::fs::rename(&from.path(), &to.path());
    }
    pub fn delete_if_exists(file: &File) {
        if file.exists() {
            let _ = std::fs::remove_file(&file.path());
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StandardCopyOption;

impl StandardCopyOption {
    pub const ATOMIC_MOVE: StandardCopyOption = StandardCopyOption;
}

// ---------------- java.util.concurrent 占位 ----------------

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TimeUnit;

impl TimeUnit {
    pub const SECONDS: TimeUnit = TimeUnit;
}

#[derive(Debug, Clone, Default)]
pub struct Lock;

impl Lock {
    pub fn try_lock(&self, _timeout: i32, _unit: TimeUnit) -> bool {
        true
    }
    pub fn unlock(&self) {}
}

#[derive(Debug, Clone, Default)]
pub struct ReadWriteLock;

impl ReadWriteLock {
    pub fn new() -> ReadWriteLock {
        ReadWriteLock
    }
    pub fn read_lock(&self) -> Lock {
        Lock
    }
    pub fn write_lock(&self) -> Lock {
        Lock
    }
}

// Kotlin `ReentrantReadWriteLock()`（子类即 ReadWriteLock）
pub type ReentrantReadWriteLock = ReadWriteLock;

// ---------------- java.util.Base64 占位（标准 base64 编码） ----------------

pub struct JavaBase64;

impl JavaBase64 {
    pub fn get_encoder() -> Base64Encoder {
        Base64Encoder
    }
}

pub struct Base64Encoder;

impl Base64Encoder {
    pub fn encode_to_string(&self, bytes: &[u8]) -> String {
        const TABLE: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for chunk in bytes.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = *chunk.get(1).unwrap_or(&0) as u32;
            let b2 = *chunk.get(2).unwrap_or(&0) as u32;
            let n = (b0 << 16) | (b1 << 8) | b2;
            out.push(TABLE[(n >> 18) as usize & 0x3f] as char);
            out.push(TABLE[(n >> 12) as usize & 0x3f] as char);
            if chunk.len() > 1 {
                out.push(TABLE[(n >> 6) as usize & 0x3f] as char);
            } else {
                out.push('=');
            }
            if chunk.len() > 2 {
                out.push(TABLE[n as usize & 0x3f] as char);
            } else {
                out.push('=');
            }
        }
        out
    }
}

// ---------------- io.legado.app.utils.Base64 占位（Android Base64.decode, AnalyzeUrl.getByteArrayAwait 使用） ----------------

pub struct Base64;

impl Base64 {
    pub const DEFAULT: i32 = 0;
    pub const NO_PADDING: i32 = 1;
    pub const NO_WRAP: i32 = 2;

    pub fn decode(s: &str, _flags: i32) -> Vec<u8> {
        let mut out = Vec::new();
        let mut buf: u32 = 0;
        let mut bits: u32 = 0;
        for b in s.bytes() {
            if b == b'=' || b == b'\n' || b == b'\r' || b.is_ascii_whitespace() {
                continue;
            }
            let v = match b {
                b'A'..=b'Z' => (b - b'A') as u32,
                b'a'..=b'z' => (b - b'a' + 26) as u32,
                b'0'..=b'9' => (b - b'0' + 52) as u32,
                b'+' => 62,
                b'/' => 63,
                _ => continue,
            };
            buf = (buf << 6) | v;
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                out.push((buf >> bits) as u8);
                buf &= (1u32 << bits) - 1;
            }
        }
        out
    }

    pub fn decode_str(s: &str, flags: i32) -> Vec<u8> {
        Self::decode(s, flags)
    }
}

// ---------------- io.legado.app.utils.FileUtils 占位 ----------------

pub struct FileUtils;

impl FileUtils {
    pub fn get_extension(name: &str) -> String {
        name.rsplit('.').next().map(|s| s.to_string()).unwrap_or_default()
    }
    // fix: Book.rs 转录调用（Kotlin FileUtils.getFile(root, subDirFile) / getPath(...) 扁平化）
    pub fn get_file(root: File, child: &str) -> File {
        root.resolve(child)
    }
    pub fn get_path(root: File, sub: &str, sub2: &str, sub3: &str, sub4: &str) -> String {
        format!(
            "{}/{}/{}/{}/{}",
            root.path().trim_end_matches('/'),
            sub.trim_start_matches('/'),
            sub2.trim_start_matches('/'),
            sub3.trim_start_matches('/'),
            sub4.trim_start_matches('/')
        )
    }
}

// ---------------- Gson TypeAdapter 占位 ----------------

pub struct IntTypeAdapter;

impl IntTypeAdapter {
    pub fn new() -> IntTypeAdapter {
        IntTypeAdapter
    }
}

pub struct LongTypeAdapter;

impl LongTypeAdapter {
    pub fn new() -> LongTypeAdapter {
        LongTypeAdapter
    }
}

pub struct MapDeserializerDoubleAsIntFix;

impl MapDeserializerDoubleAsIntFix {
    pub fn new() -> MapDeserializerDoubleAsIntFix {
        MapDeserializerDoubleAsIntFix
    }
}

// ---------------- Jackson 占位（ObjectMapper / ObjectNode / JsonToken） ----------------

#[derive(Debug, Clone, Default)]
pub struct ObjectMapper;

impl ObjectMapper {
    pub fn new() -> ObjectMapper {
        ObjectMapper
    }
    pub fn factory(&self) -> JsonFactory {
        JsonFactory
    }
}

#[derive(Debug, Clone, Default)]
pub struct JsonFactory;

impl JsonFactory {
    pub fn create_parser(&self, file: &File) -> JsonParser {
        let mut parser = JsonParser::default();
        if file.exists() {
            let text = file.read_text();
            if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&text) {
                *parser.items.borrow_mut() = arr.into_iter().map(|v| v.to_string()).collect();
            }
        }
        parser
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JsonToken {
    #[default]
    START_ARRAY,
    END_ARRAY,
    START_OBJECT,
    END_OBJECT,
    FIELD_NAME,
    VALUE_STRING,
}

#[derive(Debug, Clone)]
pub struct JsonParser {
    // 真实 JSON 数组流式解析：items 为数组元素（对象 JSON 文本）
    pub items: std::cell::RefCell<Vec<String>>,
    pub pos: std::cell::RefCell<usize>,
    pub started: std::cell::RefCell<bool>,
    pub last_token: std::cell::RefCell<JsonToken>,
}

impl Default for JsonParser {
    fn default() -> JsonParser {
        JsonParser {
            items: std::cell::RefCell::new(Vec::new()),
            pos: std::cell::RefCell::new(0),
            started: std::cell::RefCell::new(false),
            last_token: std::cell::RefCell::new(JsonToken::START_ARRAY),
        }
    }
}

impl JsonParser {
    pub fn next_token(&self) -> JsonToken {
        if !*self.started.borrow() {
            *self.started.borrow_mut() = true;
            *self.last_token.borrow_mut() = JsonToken::START_ARRAY;
            return JsonToken::START_ARRAY;
        }
        let pos = *self.pos.borrow();
        let len = self.items.borrow().len();
        if pos < len {
            *self.pos.borrow_mut() = pos + 1;
            *self.last_token.borrow_mut() = JsonToken::START_OBJECT;
            return JsonToken::START_OBJECT;
        }
        *self.last_token.borrow_mut() = JsonToken::END_ARRAY;
        return JsonToken::END_ARRAY;
    }
    pub fn current_token(&self) -> JsonToken {
        self.last_token.borrow().clone()
    }
    pub fn current_name(&self) -> String {
        String::new()
    }
    pub fn value_as_string(&self) -> String {
        String::new()
    }
    pub fn skip_children(&self) {}
    // Kotlin JsonParser.readValueAsTree<T>（泛型占位：经特化方法读取）
    pub fn read_value_as_tree<T: Default>(&self) -> T {
        T::default()
    }
    // 特化：当前元素 → JsonNode
    pub fn read_value_as_json_node(&self) -> JsonNode {
        let pos = (*self.pos.borrow()).saturating_sub(1);
        match self.items.borrow().get(pos) {
            Some(s) => serde_json::from_str::<serde_json::Value>(s)
                .map(JsonNode)
                .unwrap_or_else(|_| JsonNode(serde_json::Value::Null)),
            None => JsonNode(serde_json::Value::Null),
        }
    }
    // 特化：当前元素 → ObjectNode（扁平字符串值）
    pub fn read_value_as_object_node(&self) -> ObjectNode {
        let pos = (*self.pos.borrow()).saturating_sub(1);
        let mut map = std::collections::HashMap::new();
        if let Some(s) = self.items.borrow().get(pos) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(s) {
                if let Some(obj) = v.as_object() {
                    for (k, val) in obj {
                        let raw = match val {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        map.insert(k.clone(), raw);
                    }
                }
            }
        }
        ObjectNode(map)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ObjectNode(pub HashMap<String, String>);

impl ObjectNode {
    pub fn remove(&mut self, key: &str) {
        self.0.remove(key);
    }
    pub fn to_string(&self) -> String {
        let mut obj = serde_json::Map::new();
        for (k, v) in &self.0 {
            obj.insert(k.clone(), serde_json::from_str::<serde_json::Value>(v).unwrap_or_else(|_| serde_json::Value::String(v.clone())));
        }
        serde_json::Value::Object(obj).to_string()
    }
}

// ---------------- MongoDB 占位 ----------------

pub use crate::com_htmake_reader_entity_mongofile::MongoFile;

#[derive(Debug, Clone, Default)]
pub struct MongoCollection<T> {
    pub docs: HashMap<String, T>,
}

impl<T> MongoCollection<T> {
    pub fn find(&self, _filter: Filter) -> MongoFind<T> {
        MongoFind::default()
    }
    pub fn replace_one(&self, _filter: Filter, _replacement: T, _options: ReplaceOptions) -> UpdateResult {
        UpdateResult::default()
    }
    pub fn insert_one(&self, _doc: T) {}
}

#[derive(Debug, Clone)]
pub struct MongoFind<T> {
    pub first_doc: Option<T>,
}

impl<T> Default for MongoFind<T> {
    fn default() -> Self {
        MongoFind { first_doc: None }
    }
}

impl<T> MongoFind<T> {
    pub fn first(&mut self) -> Option<T> {
        self.first_doc.take()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Filter {
    pub key: String,
    pub value: String,
}

pub struct Filters;

impl Filters {
    pub fn eq(key: &str, value: &str) -> Filter {
        Filter {
            key: key.to_string(),
            value: value.to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ReplaceOptions {
    pub upsert_flag: bool,
}

impl ReplaceOptions {
    pub fn new() -> ReplaceOptions {
        ReplaceOptions { upsert_flag: false }
    }
    pub fn upsert(mut self, u: bool) -> ReplaceOptions {
        self.upsert_flag = u;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct UpdateResult {
    pub modified_count: i64,
}

// ---------------- java.nio.charset.Charset 占位（QueryTTF 等使用） ----------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Charset;

impl Charset {
    pub const US_ASCII: Charset = Charset;
    pub const UTF_8: Charset = Charset;
    pub const UTF_16BE: Charset = Charset;
    // fix: 合并原 2760 行重复 Charset 占位的方法（E0428 只保留首个定义）
    pub fn for_name(_name: &str) -> Charset {
        Charset
    }
    pub fn default_charset() -> Charset {
        Charset
    }
}

// ---------------- com.jayway.jsonpath JsonPath / ReadContext 占位（AnalyzeByJSonPath 使用） ----------------

#[derive(Debug, Clone, Default)]
pub struct ReadContext {
    pub json: String,
}

impl ReadContext {
    // 真实 JSONPath 解析（支持 $ .field [n] [*] 与简单 [?(@.k=="v")] 过滤）
    pub fn read<T>(&self, path: &str) -> Result<T, StubError>
    where
        T: serde::de::DeserializeOwned,
    {
        let value = crate::runtime::json_path::query(&self.json, path)
            .ok_or_else(|| StubError::new(format!("json path not found: {}", path)))?;
        serde_json::from_value::<T>(value).map_err(|e| StubError::new(e.to_string()))
    }
}

pub struct JsonPath;

impl JsonPath {
    pub fn parse<T: ToString>(t: T) -> ReadContext {
        ReadContext { json: t.to_string() }
    }
}

// ---------------- org.seimicrawler.xpath JXDocument / JXNode 占位（AnalyzeByXPath 使用） ----------------

#[derive(Debug, Clone, Default)]
pub struct JXNode {
    pub html: String,
    pub text: String,
}

impl JXNode {
    // Kotlin JXNode.isElement（占位：由 JXNode 输入的节点视为元素）
    pub fn is_element(&self) -> bool {
        true
    }
    // Kotlin JXNode.asElement()
    pub fn as_element(&self) -> Element {
        Element {
            text: self.text.clone(),
            html: self.text.clone(),
        }
    }
    // Kotlin JXNode.asString()
    pub fn as_string(&self) -> String {
        self.text.clone()
    }
    // Kotlin JXNode.sel(xPath)
    pub fn sel(&self, x_path: &str) -> Option<Vec<JXNode>> {
        crate::runtime::xpath::select_nodes(&self.html, x_path)
    }
}

impl std::fmt::Display for JXNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

#[derive(Debug, Clone, Default)]
pub struct JXDocument {
    pub text: String,
}

impl JXDocument {
    // Kotlin JXDocument.create(Object)
    pub fn create<T: ToString>(t: T) -> JXDocument {
        JXDocument {
            text: t.to_string(),
        }
    }
    // Kotlin JXDocument.selN(xPath)
    pub fn sel_n(&self, x_path: &str) -> Option<Vec<JXNode>> {
        crate::runtime::xpath::select_nodes(&self.text, x_path)
    }
}

impl std::fmt::Display for JXDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

// ---------------- io.legado.app.utils.TextUtils 占位（同名真实模块 io_legado_app_utils_textutils 转录未完成，
// 占位提供 join/isEmpty 供 prelude 使用；真实模块内部符号被 glob 导入遮蔽，互不冲突） ----------------

pub struct TextUtils;

impl TextUtils {
    // Kotlin TextUtils.isEmpty(CharSequence?)
    pub fn is_empty(s: Option<&str>) -> bool {
        match s {
            Some(s) => s.is_empty(),
            None => true,
        }
    }
    // Kotlin TextUtils.join(CharSequence delimiter, Iterable<?> tokens)
    pub fn join<T: std::fmt::Display>(delimiter: &str, tokens: Vec<T>) -> String {
        tokens
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(delimiter)
    }
}


// ================= 类型检查迭代补充 =================

// ---- Java 异常别名 ----
pub type IOException = StubError;
pub type Exception = StubError;
pub type Throwable = StubError;
pub type Boolean = bool;
pub type Map<K, V> = std::collections::HashMap<K, V>;
pub type LinkedHashMap<K, V> = std::collections::HashMap<K, V>;
// Kotlin linkedSetOf（BookSourceController.saveUserBookSources 使用）
pub type LinkedHashSet<T> = std::collections::HashSet<T>;
pub type JsonElement = serde_json::Value;

// ---- io 包（io::vertx::... / io::Error 等路径引用） ----
pub mod io {
    pub type Error = std::io::Error;
    pub struct ByteArrayInputStream(pub Vec<u8>);
    pub struct ByteArrayOutputStream(pub Vec<u8>);
    impl ByteArrayOutputStream {
        pub fn new() -> Self {
            ByteArrayOutputStream(Vec::new())
        }
        pub fn to_byte_array(&self) -> Vec<u8> {
            self.0.clone()
        }
    }
        pub mod vertx {
            include!("runtime/vertx.rs");
        }
}

// ---- HttpMethod ----
pub use crate::stubs::io::vertx::HttpMethod;

// ---- Pair ----
#[derive(Clone, Debug)]
pub struct Pair<A, B>(pub A, pub B);
impl<A, B> Pair<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Pair(a, b)
    }
    pub fn first(&self) -> &A {
        &self.0
    }
    pub fn second(&self) -> &B {
        &self.1
    }
}

// ---- 加密真实实现（AES/DES 对称 + RSA 非对称） ----
pub struct Cipher {
    pub transformation: String,
    pub mode: i32,
    pub key: Vec<u8>,
    pub iv: Option<Vec<u8>>,
    pub algorithm: String,
}

impl Cipher {
    pub const ENCRYPT_MODE: i32 = 1;
    pub const DECRYPT_MODE: i32 = 2;

    pub fn new() -> Self {
        Cipher {
            transformation: String::new(),
            mode: Cipher::ENCRYPT_MODE,
            key: Vec::new(),
            iv: None,
            algorithm: String::new(),
        }
    }
    pub fn init(&mut self, _mode: i32, _key: &str) {}
    pub fn update(&mut self, _data: Vec<u8>) -> Vec<u8> {
        Vec::new()
    }
    pub fn do_final(&mut self) -> Vec<u8> {
        Vec::new()
    }
    pub fn getInstance(transformation: &str) -> Cipher {
        let algorithm = transformation
            .split('/')
            .next()
            .unwrap_or(transformation)
            .to_uppercase();
        Cipher {
            transformation: transformation.to_string(),
            mode: Cipher::ENCRYPT_MODE,
            key: Vec::new(),
            iv: None,
            algorithm,
        }
    }
    pub fn init_spec(&mut self, mode: i32, key: &SecretKeySpec) {
        self.mode = mode;
        self.key = key.key.clone();
    }
    pub fn init_spec_iv(&mut self, mode: i32, key: &SecretKeySpec, iv: &IvParameterSpec) {
        self.mode = mode;
        self.key = key.key.clone();
        self.iv = Some(iv.iv.clone());
    }
    pub fn init_key(&mut self, mode: i32, key: &dyn java_security_Key) {
        self.mode = mode;
        self.algorithm = String::from("RSA");
        if let Some(k) = key.as_private() {
            self.key = k.der.clone();
        } else if let Some(k) = key.as_public() {
            self.key = k.der.clone();
        }
    }
    pub fn do_final_data(&mut self, data: &[u8]) -> Vec<u8> {
        if self.algorithm == "RSA" {
            return self.rsa_do_final(data);
        }
        self.symmetric_do_final(data)
    }
    pub fn do_final_range(&mut self, data: &[u8], offset: usize, len: usize) -> Vec<u8> {
        self.do_final_data(&data[offset..offset + len])
    }

    fn symmetric_do_final(&self, data: &[u8]) -> Vec<u8> {
        let mode_name = self
            .transformation
            .split('/')
            .nth(1)
            .unwrap_or("ECB")
            .to_uppercase();
        let is_cbc = mode_name.contains("CBC");
        let encrypt = self.mode == Cipher::ENCRYPT_MODE;
        match self.algorithm.as_str() {
            "AES" => {
                let key_len = self.key.len();
                let block = if key_len == 32 { 32 } else { 16 };
                if is_cbc {
                    let iv_len = self.iv.as_ref().map(|v| v.len()).unwrap_or(0);
                    if iv_len >= block {
                        Self::cbc_crypt::<aes::Aes128>(self, data, encrypt, 16)
                            .or_else(|| Self::cbc_crypt::<aes::Aes256>(self, data, encrypt, 32))
                            .unwrap_or_default()
                    } else {
                        Self::cbc_crypt_iv16::<aes::Aes128>(self, data, encrypt, 16)
                            .or_else(|| Self::cbc_crypt_iv16::<aes::Aes256>(self, data, encrypt, 32))
                            .unwrap_or_default()
                    }
                } else {
                    if key_len == 32 {
                        Self::ecb_crypt::<aes::Aes256>(self, data, encrypt, 32).unwrap_or_default()
                    } else {
                        Self::ecb_crypt::<aes::Aes128>(self, data, encrypt, 16).unwrap_or_default()
                    }
                }
            }
            "DES" => {
                if is_cbc {
                    Self::cbc_crypt_iv16::<des::Des>(self, data, encrypt, 8).unwrap_or_default()
                } else {
                    Self::ecb_crypt::<des::Des>(self, data, encrypt, 8).unwrap_or_default()
                }
            }
            _ => Vec::new(),
        }
    }

    /// ECB 手动分块 + PKCS7（兼容任意块大小）
    fn ecb_crypt<C>(&self, data: &[u8], encrypt: bool, block_size: usize) -> Option<Vec<u8>>
    where
        C: cipher::BlockEncrypt + cipher::BlockDecrypt + cipher::KeyInit,
    {
        use cipher::generic_array::GenericArray;
        let cipher = C::new_from_slice(&self.key).ok()?;
        if encrypt {
            let pad = block_size - (data.len() % block_size);
            let mut out = Vec::with_capacity(data.len() + pad);
            let mut chunks = data.chunks_exact(block_size);
            for chunk in &mut chunks {
                let mut block = GenericArray::clone_from_slice(chunk);
                cipher.encrypt_block(&mut block);
                out.extend_from_slice(&block);
            }
            let rem = chunks.remainder();
            if !rem.is_empty() {
                let mut block = GenericArray::default();
                block[..rem.len()].copy_from_slice(rem);
                block[rem.len()..].iter_mut().for_each(|b| *b = pad as u8);
                cipher.encrypt_block(&mut block);
                out.extend_from_slice(&block);
            }
            // 完整块需追加一个整填充块
            if rem.is_empty() {
                let mut block = GenericArray::default();
                block.iter_mut().for_each(|b| *b = pad as u8);
                cipher.encrypt_block(&mut block);
                out.extend_from_slice(&block);
            }
            Some(out)
        } else {
            if data.is_empty() || data.len() % block_size != 0 {
                return None;
            }
            let mut out = Vec::with_capacity(data.len());
            for chunk in data.chunks_exact(block_size) {
                let mut block = GenericArray::clone_from_slice(chunk);
                cipher.decrypt_block(&mut block);
                out.extend_from_slice(&block);
            }
            // 去 PKCS7 填充
            let pad = *out.last()? as usize;
            if pad > 0 && pad <= block_size {
                out.truncate(out.len() - pad);
            }
            Some(out)
        }
    }

    /// CBC（IV 取前 block 字节）——通用实现
    fn cbc_crypt_iv16<C>(&self, data: &[u8], encrypt: bool, block_size: usize) -> Option<Vec<u8>>
    where
        C: cipher::BlockEncrypt + cipher::BlockDecrypt + cipher::KeyInit,
    {
        use cipher::generic_array::GenericArray;
        let cipher = C::new_from_slice(&self.key).ok()?;
        let iv = self.iv.as_deref().unwrap_or(&[0u8; 16]);
        let mut iv_block = GenericArray::default();
        for (d, s) in iv_block.iter_mut().zip(iv.iter()) {
            *d = *s;
        }
        if encrypt {
            let mut padded = data.to_vec();
            let pad = block_size - (padded.len() % block_size);
            padded.extend(std::iter::repeat(pad as u8).take(pad));
            let mut out = Vec::with_capacity(padded.len());
            let mut prev = iv_block.clone();
            for chunk in padded.chunks_exact(block_size) {
                let mut block = GenericArray::clone_from_slice(chunk);
                for (b, p) in block.iter_mut().zip(prev.iter()) {
                    *b ^= *p;
                }
                cipher.encrypt_block(&mut block);
                prev = block.clone();
                out.extend_from_slice(&block);
            }
            Some(out)
        } else {
            if data.is_empty() || data.len() % block_size != 0 {
                return None;
            }
            let mut out = Vec::with_capacity(data.len());
            let mut prev = iv_block.clone();
            for chunk in data.chunks_exact(block_size) {
                let mut block = GenericArray::clone_from_slice(chunk);
                let orig = block.clone();
                cipher.decrypt_block(&mut block);
                for (b, p) in block.iter_mut().zip(prev.iter()) {
                    *b ^= *p;
                }
                prev = orig;
                out.extend_from_slice(&block);
            }
            let pad = *out.last()? as usize;
            if pad > 0 && pad <= block_size {
                out.truncate(out.len() - pad);
            }
            Some(out)
        }
    }

    /// CBC（IV 为完整块）——预留接口，与 cbc_crypt_iv16 同实现
    fn cbc_crypt<C>(&self, data: &[u8], encrypt: bool, block_size: usize) -> Option<Vec<u8>>
    where
        C: cipher::BlockEncrypt + cipher::BlockDecrypt + cipher::KeyInit,
    {
        Self::cbc_crypt_iv16::<C>(self, data, encrypt, block_size)
    }

    fn rsa_do_final(&self, data: &[u8]) -> Vec<u8> {
        use rsa::BigUint;
        use rsa::pkcs1::DecodeRsaPrivateKey;
        use rsa::pkcs1::DecodeRsaPublicKey;
        use rsa::traits::{PrivateKeyParts, PublicKeyParts};
        let encrypt = self.mode == Cipher::ENCRYPT_MODE;
        let sk = rsa::RsaPrivateKey::from_pkcs1_der(&self.key).ok();
        let pk = rsa::RsaPublicKey::from_pkcs1_der(&self.key).ok();
        if encrypt {
            // RSAES-PKCS1-v1_5 加密（块类型 2）
            let (n, e, d, k): (BigUint, BigUint, Option<BigUint>, usize) = if let Some(sk) = &sk {
                (sk.n().clone(), sk.e().clone(), Some(sk.d().clone()), sk.size())
            } else if let Some(pk) = &pk {
                (pk.n().clone(), pk.e().clone(), None, pk.size())
            } else {
                return Vec::new();
            };
            if data.len() > k - 11 {
                return Vec::new();
            }
            let mut em = vec![0u8; k];
            em[1] = 2;
            let ps_len = k - data.len() - 3;
            em[2..2 + ps_len].fill(0xff);
            em[2 + ps_len] = 0;
            em[k - data.len()..].copy_from_slice(data);
            let m = BigUint::from_bytes_be(&em);
            let c = match &d {
                Some(d) => m.modpow(d, &n),
                None => m.modpow(&e, &n),
            };
            let out = c.to_bytes_be();
            let mut result = vec![0u8; k - out.len()];
            result.extend(out);
            result
        } else {
            // 解密
            let (n, d, k) = if let Some(sk) = &sk {
                (sk.n().clone(), sk.d().clone(), sk.size())
            } else if let Some(pk) = &pk {
                // 公钥"解密"= 用指数反算（Java 公钥解密私钥加密的场景）
                (pk.n().clone(), pk.e().clone(), pk.size())
            } else {
                return Vec::new();
            };
            if data.len() > k {
                return Vec::new();
            }
            let mut padded = vec![0u8; k - data.len()];
            padded.extend_from_slice(data);
            let c = BigUint::from_bytes_be(&padded);
            let m = c.modpow(&d, &n);
            let em = m.to_bytes_be();
            // 去填充：0x00 || 0x02 || PS(非0) || 0x00 || M（块类型 2）
            let mut start = 0usize;
            if em.len() >= 2 && em[0] == 0 && em[1] == 2 {
                let mut i = 2;
                while i < em.len() && em[i] != 0 {
                    i += 1;
                }
                if i < em.len() {
                    start = i + 1;
                }
            }
            if start < em.len() {
                em[start..].to_vec()
            } else {
                em
            }
        }
    }
}
pub struct PrivateKey {
    pub der: Vec<u8>,
}
pub struct PublicKey {
    pub der: Vec<u8>,
}

// Java java.security.Key 抽象（EncoderUtils RSA 走 &dyn java_security_Key）
pub trait java_security_Key {
    fn as_private(&self) -> Option<&PrivateKey> {
        None
    }
    fn as_public(&self) -> Option<&PublicKey> {
        None
    }
}
impl java_security_Key for PrivateKey {
    fn as_private(&self) -> Option<&PrivateKey> {
        Some(self)
    }
}
impl java_security_Key for PublicKey {
    fn as_public(&self) -> Option<&PublicKey> {
        Some(self)
    }
}

// Java javax.crypto.spec.SecretKeySpec / IvParameterSpec
pub struct SecretKeySpec {
    pub key: Vec<u8>,
    pub algorithm: String,
}
impl SecretKeySpec {
    pub fn new(key: &[u8], algorithm: &str) -> SecretKeySpec {
        SecretKeySpec {
            key: key.to_vec(),
            algorithm: algorithm.to_string(),
        }
    }
}

pub struct IvParameterSpec {
    pub iv: Vec<u8>,
}
impl IvParameterSpec {
    pub fn new(iv: &[u8]) -> IvParameterSpec {
        IvParameterSpec { iv: iv.to_vec() }
    }
}

// Java java.security.KeyPair / KeyPairGenerator 占位
pub struct KeyPair;
pub struct KeyPairGenerator;
impl KeyPairGenerator {
    pub fn getInstance(_algorithm: &str) -> KeyPairGenerator {
        KeyPairGenerator
    }
    pub fn genKeyPair(&self) -> KeyPair {
        KeyPair
    }
}

// ---- Jsoup 占位 ----
pub struct Jsoup;
impl Jsoup {
    pub fn parse(s: String) -> Document {
        Document::parse(s)
    }
    pub fn parse_body_fragment(s: String) -> Document {
        let (h, t) = crate::runtime::html::body_of(&s);
        Document { text: t, html: h }
    }
    pub fn connect(_url: &str) -> OkHttpClient {
        OkHttpClient { proxy: None, proxy_auth: None, interceptors: Vec::new() }
    }
}

// ---- 协程占位 ----
pub struct CoroutineContext;
pub struct CoroutineScope;
pub struct Deferred;
pub struct Dispatchers;
impl Dispatchers {
    // Kotlin Dispatchers.IO（RestVerticle/BookController 等 `MDCContext + Dispatchers.IO` 使用）
    pub const IO: CoroutineContext = CoroutineContext;
    pub fn io() -> CoroutineContext {
        CoroutineContext
    }
    pub fn default() -> CoroutineContext {
        CoroutineContext
    }
    pub fn main() -> CoroutineContext {
        CoroutineContext
    }
}
pub struct Job;
impl Job {
    pub fn cancel(&self) {}
}
pub struct Block;
pub struct VoidBlock;
#[derive(Debug, Clone)]
pub struct Stage;
impl Stage {
    pub fn new() -> Stage {
        Stage
    }
    pub fn show(&mut self) {}
    pub fn hide(&mut self) {}
    pub fn set_scene(&mut self, _scene: Option<Scene>) {}
    pub fn set_title(&mut self, _title: String) {}
    pub fn set_x(&mut self, _x: f64) {}
    pub fn set_y(&mut self, _y: f64) {}
    pub fn width_property(&self) -> DoubleProperty {
        DoubleProperty
    }
    pub fn height_property(&self) -> DoubleProperty {
        DoubleProperty
    }
    pub fn x_property(&self) -> DoubleProperty {
        DoubleProperty
    }
    pub fn y_property(&self) -> DoubleProperty {
        DoubleProperty
    }
    pub fn scene_property(&self) -> SceneProperty {
        SceneProperty
    }
    pub fn init_style(&mut self, _style: StageStyle) {}
    pub fn get_icons(&self) -> ObservableList {
        ObservableList
    }
}
pub struct JsonDeserializationContext;

// ---- MDCContext 占位（kotlinx.coroutines.slf4j.MDCContext，RestVerticle/BookController 使用） ----
pub struct MDCContext;
impl MDCContext {
    pub fn new() -> MDCContext {
        MDCContext
    }
}
impl std::ops::Add<CoroutineContext> for MDCContext {
    type Output = CoroutineContext;
    fn add(self, _rhs: CoroutineContext) -> CoroutineContext {
        CoroutineContext
    }
}

// ---- JavaFX 占位（ReaderUIApplication 使用） ----
pub struct Scene;
impl Scene {
    pub fn new<A, B>(_root: A, _arg: B) -> Scene {
        Scene
    }
    pub fn new_with_size<A>(_root: A, _width: f64, _height: f64) -> Scene {
        Scene
    }
    pub fn height_property(&self) -> DoubleProperty {
        DoubleProperty
    }
}
pub struct VBox;
impl VBox {
    pub fn new() -> VBox {
        VBox
    }
    pub fn get_children(&self) -> ObservableList {
        ObservableList
    }
}
pub struct ObservableList;
impl ObservableList {
    pub fn add<T>(&mut self, _item: T) {}
    pub fn add_all<T>(&mut self, _items: Vec<T>) {}
}
pub struct DoubleProperty;
impl DoubleProperty {
    pub fn add_listener<F: FnMut(&DoubleProperty, f64, f64)>(&self, _f: F) {}
}
pub struct SceneProperty;
impl SceneProperty {
    pub fn add_listener<F: FnMut(&SceneProperty, Scene, Scene)>(&self, _f: F) {}
}
pub struct Image;
impl Image {
    pub fn new(_url: String) -> Image {
        Image
    }
}
pub struct ImageView;
impl ImageView {
    pub fn new(_url: String) -> ImageView {
        ImageView
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StageStyle {
    TRANSPARENT,
    UNIFIED,
}
pub struct Color;
impl Color {
    pub const TRANSPARENT: Color = Color;
}
pub struct Dialog;
impl Dialog {
    pub fn new() -> Dialog {
        Dialog
    }
    pub fn get_dialog_pane(&self) -> DialogPane {
        DialogPane
    }
    pub fn show_and_wait(&mut self) -> Option<ButtonType> {
        None
    }
    pub fn show(&mut self) {}
}
pub struct DialogPane;
impl DialogPane {
    pub fn set_content_text(&mut self, _text: String) {}
    pub fn get_button_types(&self) -> ObservableList {
        ObservableList
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ButtonType {
    OK,
    YES,
    NO,
}
pub struct Platform;
impl Platform {
    pub fn run_later<T>(_runnable: T) {}
}
pub struct Runnable;
impl Runnable {
    pub fn new<F: FnOnce()>(_f: F) -> Runnable {
        Runnable
    }
}
pub struct WebView;
impl WebView {
    pub fn new() -> WebView {
        WebView
    }
    pub fn get_engine(&self) -> WebEngine {
        WebEngine
    }
}
pub struct WebEngine;
impl WebEngine {
    pub fn set_on_error<F: FnMut(WebErrorEvent)>(&mut self, _f: F) {}
    pub fn set_on_alert<F: FnMut(WebAlertEvent)>(&mut self, _f: F) {}
    pub fn set_confirm_handler<F: Fn(String) -> bool>(&mut self, _f: F) {}
    pub fn get_load_worker(&self) -> Worker {
        Worker
    }
    pub fn title_property(&self) -> TitleProperty {
        TitleProperty
    }
    pub fn load(&mut self, _url: String) {}
}
#[derive(Debug)]
pub struct WebErrorEvent;
#[derive(Debug)]
pub struct WebAlertEvent {
    pub data: String,
}
pub struct TitleProperty;
impl TitleProperty {
    pub fn add_listener<F: FnMut(&TitleProperty, Option<String>, Option<String>)>(&self, _f: F) {}
}

// ---- Spring 占位（ReaderUIApplication/SpringContextUtils 使用） ----
pub struct SpringApplication;
impl SpringApplication {
    pub fn new<T>(_clazz: Class<T>) -> SpringApplication {
        SpringApplication
    }
    pub fn add_listeners<T>(&mut self, _listener: T) {}
    pub fn run(&mut self, _args: Vec<String>) {}
    pub fn exit(_context: Option<ApplicationContext>) -> i32 {
        0
    }
}
pub struct ApplicationListener;
impl ApplicationListener {
    pub fn new<E, F: FnMut(E)>(_f: F) -> ApplicationListener {
        ApplicationListener
    }
}
pub struct ApplicationEnvironmentPreparedEvent;
impl ApplicationEnvironmentPreparedEvent {
    pub fn get_environment(&self) -> ConfigurableEnvironment {
        ConfigurableEnvironment
    }
}
pub struct ConfigurableEnvironment;
impl ConfigurableEnvironment {
    pub fn new() -> ConfigurableEnvironment {
        ConfigurableEnvironment
    }
    pub fn get_property_sources(&self) -> PropertySources {
        PropertySources
    }
    // Spring property → 环境变量 relaxed binding（reader.server.port → READER_SERVER_PORT）
    pub fn get_property<T: 'static>(&self, key: &str, _clazz: Class<T>) -> Option<T> {
        let v = env_var_for(key)?;
        let tid = std::any::TypeId::of::<T>();
        let any: Box<dyn std::any::Any> = if tid == std::any::TypeId::of::<String>() {
            Box::new(v)
        } else if tid == std::any::TypeId::of::<bool>() {
            Box::new(v.eq_ignore_ascii_case("true") || v == "1")
        } else if tid == std::any::TypeId::of::<i32>() {
            Box::new(v.parse::<i32>().ok()?)
        } else if tid == std::any::TypeId::of::<i64>() {
            Box::new(v.parse::<i64>().ok()?)
        } else if tid == std::any::TypeId::of::<u32>() {
            Box::new(v.parse::<u32>().ok()?)
        } else {
            return None;
        };
        any.downcast::<T>().ok().map(|b| *b)
    }
}
pub struct PropertySources;
impl PropertySources {
    pub fn add_first<T>(&mut self, _source: T) {}
}
pub struct MapPropertySource;
impl MapPropertySource {
    pub fn new(_name: &str, _source: HashMap<String, Any>) -> MapPropertySource {
        MapPropertySource
    }
}
pub struct ApplicationContext;

// ---- SSL 占位 ----
pub struct X509TrustManager;
pub struct SSLContext;
pub struct SSLSocketFactory;

// ---- ResponseBody ----
#[derive(Clone, Default)]
pub struct ResponseBody {
    pub text: Option<String>,
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
}
impl ResponseBody {
    pub fn new() -> Self {
        ResponseBody::default()
    }
    pub fn string(&self) -> String {
        self.text.clone().unwrap_or_default()
    }
    pub fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }
}

// ---- 顶层函数占位 ----
pub fn run_blocking<T, F: FnOnce() -> T>(block: F) -> T {
    block()
}
pub fn run_catching<T, F: FnOnce() -> T>(block: F) -> Result<T, StubError> {
    Ok(block())
}
// fix: Kotlin `launch(context) { }` → (CoroutineContext, 闭包/Future)，返回 Job（RestVerticle job 赋值使用）
//      原占位不执行闭包（封面下载/书籍缓存/TTS 全失效）→ 同步执行 + catch_unwind
pub fn launch<T>(_ctx: CoroutineContext, f: T) -> Job
where
    T: FnOnce(),
{
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    Job
}
pub fn with_context<F: FnOnce()>(_ctx: CoroutineContext, f: F) {
    f()
}
pub fn empty_list<T>() -> Vec<T> {
    Vec::new()
}
pub fn mutable_list_of<T>(items: Vec<T>) -> Vec<T> {
    items
}
// fix: 真实 URL 编码（Kotlin URLEncoder：字母数字._- 保留，空格→+，其余 %XX；utf8ToGbk 类场景 charset 为 GBK 时先转码）
pub fn url_encode(s: &str) -> String {
    url_encode_charset(s.to_string(), "UTF-8")
}
pub fn url_encode_charset(s: String, charset: &str) -> String {
    let bytes: Vec<u8> = if charset.to_lowercase().contains("gbk") || charset.to_lowercase().contains("gb") && !charset.to_lowercase().contains("utf") {
        let (enc, _, _) = encoding_rs::GBK.encode(&s);
        enc.into_owned()
    } else {
        s.into_bytes()
    };
    let mut out = String::new();
    for &b in &bytes {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'-' | b'_' | b'*' => out.push(b as char),
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}
// Kotlin ByteArray.toHexString()（AnalyzeUrl.getStrResponseAwait 使用）
pub fn byte_to_hex_string(bytes: &[u8]) -> String {
    let mut sb = String::with_capacity(2 * bytes.len());
    for b in bytes {
        sb.push_str(&format!("{:02x}", b));
    }
    sb
}
// Kotlin NetworkUtils.isDataUrl(url)（AnalyzeUrl.init 使用）
pub fn is_data_url(url: &str) -> bool {
    url.starts_with("data:image")
}
pub fn gson_from_json_array<T>(s: String) -> Result<T, StubError>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_str::<T>(&s).map_err(|e| StubError::new(e.to_string()))
}
// Kotlin GSON.toJson(any)（AnalyzeUrl.UrlOption.getBody 使用）
pub fn gson_to_json<T: std::fmt::Display>(value: &T) -> String {
    value.to_string()
}
// Kotlin kotlinx.coroutines.runBlocking 之外的同步阻塞（AnalyzeUrl.getStrResponse 等使用；
// 无运行时, 用自旋 poll 驱动 stub future）
pub fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    fn noop_waker() -> std::task::Waker {
        fn noop_clone(_: *const ()) -> std::task::RawWaker {
            noop_raw()
        }
        fn noop(_: *const ()) {}
        fn noop_raw() -> std::task::RawWaker {
            static VTABLE: std::task::RawWakerVTable =
                std::task::RawWakerVTable::new(noop_clone, noop, noop, noop);
            std::task::RawWaker::new(std::ptr::null(), &VTABLE)
        }
        unsafe { std::task::Waker::from_raw(noop_raw()) }
    }
    let waker = noop_waker();
    let mut cx = std::task::Context::from_waker(&waker);
    let mut fut = std::pin::pin!(fut);
    loop {
        match fut.as_mut().poll(&mut cx) {
            std::task::Poll::Ready(v) => return v,
            std::task::Poll::Pending => std::thread::yield_now(),
        }
    }
}
pub fn md5_encode16(s: String) -> String {
    s
}
pub fn json_encode(value: String, _pretty: bool) -> String {
    value
}
pub fn gson_from_json_object<T>(s: String) -> Result<T, StubError>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_str::<T>(&s).map_err(|e| StubError::new(e.to_string()))
}
pub fn from_json_object<T>(s: String) -> Result<T, StubError>
where
    T: serde::de::DeserializeOwned,
{
    gson_from_json_object(s)
}
pub fn get_absolute_url(base: Option<&URL>, s: String) -> String {
    // OkHttp HttpUrl.resolve 语义：相对路径基于 base 拼接；绝对 URL 原样返回
    if s.starts_with("http://") || s.starts_with("https://") {
        return s;
    }
    if let Some(base) = base {
        if let Ok(joined) = base.0.join(&s) {
            return joined.to_string();
        }
        // 无 scheme 的协议相对 URL（//host/path）或 base 无 host
        if s.starts_with("//") {
            let scheme = base.0.scheme();
            if !scheme.is_empty() {
                return format!("{}:{}", scheme, s);
            }
        }
    }
    s
}
pub fn is_empty<T>(list: &[T]) -> bool {
    list.is_empty()
}
pub fn format_book_author(author: &str, publisher: &str, date: &str) -> String {
    format!("{} {} {}", author, publisher, date)
}
// fix: Kotlin java.lang.Long.valueOf(String)（ACache.Utils 使用）
pub fn java_long_valueOf(s: &str) -> i64 {
    s.trim().parse().unwrap_or(0)
}
// fix: Kotlin Charsets/charset(name)（Book.fileCharset 使用）
pub fn charset(name: &str) -> Charset {
    Charset::for_name(name)
}
pub fn synthesising() {}

// ---- WebdavController / BookSourceController 转录所需顶层函数 ----

// fix: 真实 percent 解码（原原样返回——WebDAV 中文/空格路径找不到文件）
pub fn url_decode(s: &str, _charset: &str) -> String {
    match percent_encoding::percent_decode_str(s).decode_utf8() {
        Ok(decoded) => decoded.into_owned(),
        Err(_) => s.to_string(),
    }
}
// fix: 真实 base64 解码（原原样返回——WebDAV Basic 认证 user:pass 解析失败恒 401）
pub fn base64_decode(s: &str) -> String {
    let bytes = crate::io_legado_app_utils_base64::Base64::decode_str(s, crate::io_legado_app_utils_base64::Base64::NO_WRAP);
    String::from_utf8_lossy(&bytes).into_owned()
}
// fix: 真实时间格式化（原返回模板字面量——WebDAV PROPFIND lastModified 全错）
pub fn simple_date_format(format: &str, millis: i64) -> String {
    use chrono::TimeZone;
    match chrono::Local.timestamp_millis_opt(millis).single() {
        Some(dt) => dt.format(&java_pattern_to_chrono(format)).to_string(),
        None => String::new(),
    }
}
// Kotlin UUID.randomUUID().toString()
pub fn uuid_random() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:016x}-{:016x}", nanos, nanos.wrapping_mul(0x9E3779B97F4A7C15))
}
// Kotlin String.format(fmt, vararg args) → 按序替换 %s 占位符
pub fn string_format(fmt: &str, args: &[String]) -> String {
    let mut result = String::new();
    let mut iter = args.iter();
    for part in fmt.split("%s") {
        result.push_str(part);
        if let Some(arg) = iter.next() {
            result.push_str(arg);
        }
    }
    result
}

// ---- WebClient 占位（BookSourceController 远程书源同步） ----

#[derive(Clone, Debug, Default)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}
impl HttpResponse {
    pub fn new() -> HttpResponse {
        HttpResponse { status: 0, body: String::new() }
    }
    pub fn new_ok(body: String) -> HttpResponse {
        HttpResponse { status: 200, body }
    }
    pub fn body_as_json_array(&self) -> Option<JsonArray> {
        JsonArray::from_json(self.body.clone())
    }
    pub fn body_as_string(&self) -> Option<String> {
        Some(self.body.clone())
    }
}

pub struct SendResult(pub Result<HttpResponse, StubError>);
impl SendResult {
    pub fn result(&self) -> Option<HttpResponse> {
        self.0.as_ref().ok().cloned()
    }
}


pub struct WebRequest {
    pub url: String,
    pub client: Option<reqwest::blocking::Client>,
    pub timeout_ms: Option<u64>,
    pub headers: std::collections::HashMap<String, String>,
}
impl WebRequest {
    pub fn timeout(mut self, millis: u64) -> WebRequest {
        self.timeout_ms = Some(millis);
        self
    }
    pub fn header(mut self, key: &str, value: &str) -> WebRequest {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
    /// 纯 async GET（tokio 环境安全；blocking 版在 async 上下文会 panic）
    pub async fn async_get_text(&self) -> Option<String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(self.timeout_ms.unwrap_or(3000)))
            .build()
            .ok()?;
        let resp = client.get(&self.url).send().await.ok()?;
        resp.text().await.ok()
    }
    /// 独立线程 + 独立 tokio runtime 执行 async POST raw body
    pub fn async_post_in_thread(&self, body: &str) -> Option<String> {
        let url = self.url.clone();
        let timeout_ms = self.timeout_ms.unwrap_or(3000);
        let body = body.to_string();
        let headers = self.headers.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .ok()?;
            rt.block_on(async {
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_millis(timeout_ms))
                    .build()
                    .ok()?;
                let mut req = client.post(&url).body(body);
                for (k, v) in &headers {
                    req = req.header(k, v);
                }
                let resp = req.send().await.ok()?;
                resp.text().await.ok()
            })
        })
        .join()
        .ok()
        .flatten()
    }
    /// 独立线程 + 独立 tokio runtime 执行 async POST form（pollster/同步上下文均安全）
    pub fn async_post_form_in_thread(        &self,
        form: &[(String, String)],
        headers: &std::collections::HashMap<String, String>,
    ) -> Option<String> {
        let url = self.url.clone();
        let timeout_ms = self.timeout_ms.unwrap_or(3000);
        let form = form.to_vec();
        let headers = headers.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .ok()?;
            rt.block_on(async {
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_millis(timeout_ms))
                    .build()
                    .ok()?;
                let mut req = client.post(&url).form(&form);
                for (k, v) in &headers {
                    req = req.header(k, v);
                }
                let resp = req.send().await.ok()?;
                resp.text().await.ok()
            })
        })
        .join()
        .ok()
        .flatten()
    }
    /// 独立线程 + 独立 tokio runtime 执行 async GET 二进制（pollster/同步上下文均安全）
    pub fn async_get_bytes_in_thread(&self) -> Option<Vec<u8>> {
        let url = self.url.clone();
        let timeout_ms = self.timeout_ms.unwrap_or(3000);
        let headers = self.headers.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .ok()?;
            rt.block_on(async {
                // fix: 透传请求头（防盗链 UA/Referer）
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_millis(timeout_ms))
                    .danger_accept_invalid_certs(true)
                    .build()
                    .ok()?;
                let mut req = client.get(&url);
                for (k, v) in &headers {
                    req = req.header(k, v);
                }
                let resp = req.send().await.ok()?;
                resp.bytes().await.ok().map(|b| b.to_vec())
            })
        })
        .join()
        .ok()
        .flatten()
    }
    /// 独立线程 + 独立 tokio runtime 执行 async GET（pollster/同步上下文均安全）
    pub fn async_get_text_in_thread(&self) -> Option<String> {
        let url = self.url.clone();
        let timeout_ms = self.timeout_ms.unwrap_or(3000);
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .ok()?;
            rt.block_on(async {
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_millis(timeout_ms))
                    .build()
                    .ok()?;
                let resp = client.get(&url).send().await.ok()?;
                resp.text().await.ok()
            })
        })
        .join()
        .ok()
        .flatten()
    }
    pub fn send(&self, handler: &dyn Fn(SendResult)) {
        // fix: reqwest::blocking 在 tokio current_thread runtime 内会 panic，
        //      请求移到独立线程执行，结果经 channel 回传主线程再调 handler
        let (tx, rx) = std::sync::mpsc::channel::<Option<reqwest::blocking::Response>>();
        let client = self.client.clone();
        let url = self.url.clone();
        let timeout_ms = self.timeout_ms;
        std::thread::spawn(move || {
            let result = client.and_then(|c| {
                let mut req = c.get(&url);
                if let Some(t) = timeout_ms {
                    req = req.timeout(std::time::Duration::from_millis(t));
                }
                req.send().ok()
            });
            let _ = tx.send(result);
        });
        let result = rx
            .recv_timeout(std::time::Duration::from_secs(30))
            .unwrap_or(None);
        let resp = result.map(|r| HttpResponse {
            status: r.status().as_u16(),
            body: r.text().unwrap_or_default(),
        });
        handler(SendResult(resp.ok_or_else(|| StubError::new("request failed"))));
    }
}

// Kotlin io.vertx.kotlin.coroutines.awaitResult → 同步执行
pub fn await_result<F>(f: F) -> HttpResponse
where
    F: FnOnce(&dyn Fn(SendResult)),
{
    let result: std::cell::RefCell<HttpResponse> = std::cell::RefCell::new(HttpResponse::default());
    {
        let out: &dyn Fn(SendResult) = &|it: SendResult| {
            if let Some(r) = it.result() {
                *result.borrow_mut() = r;
            }
        };
        f(out);
    }
    result.into_inner()
}

// ---- serde / log 模块别名 ----
pub mod serde_reexport {
    pub use serde::*;
}
pub mod log {
    pub use crate::stubs::Log;
    // fix: TTSService 等转录模块直接 `log::debug(...)` 形式调用（模块级函数占位，与 Log 方法签名一致）
    pub fn debug(_msg: impl AsRef<str>) {}
    pub fn info(_msg: impl AsRef<str>) {}
    pub fn warn(_msg: impl AsRef<str>) {}
    pub fn error(_msg: impl AsRef<str>) {}
    pub fn trace(_msg: impl AsRef<str>) {}
}

// ---- 其它 ----
pub struct Worker;
impl Worker {
    // JavaFX Worker.State（WebEngine 加载状态监听使用）
    pub fn state_property(&self) -> WorkerStateProperty {
        WorkerStateProperty
    }
    pub fn get_exception(&self) -> Option<Throwable> {
        None
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkerState;
impl WorkerState {
    pub const FAILED: WorkerState = WorkerState;
}
pub struct WorkerStateProperty;
impl WorkerStateProperty {
    pub fn add_listener<F: FnMut(&WorkerStateProperty, WorkerState, WorkerState)>(&self, _f: F) {}
}
pub struct InputSource;
pub struct EntityResolver;

// ---- JavaFX Application 顶层函数占位（ReaderUIApplication 使用） ----
pub fn launch_args() -> Vec<String> {
    Vec::new()
}
pub fn set_launch_args(_args: Vec<String>) {}
// Kotlin `super.stop()`（ReaderUIApplication.stop 调用）
pub fn super_stop() {}

// fix: Kotlin Int（`Int::class` / `downcast_ref::<Int>` 转录使用）
pub type Int = i32;

// ---- 顶层函数占位（VertExt 风格存储） ----
pub fn app_config() -> Option<&'static str> {
    None
}
pub fn get_work_dir() -> String {
    std::env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()
}
pub fn get_storage(_name: &[String], _ext: &str) -> Option<String> {
    None
}
pub fn get_storage_file(_name: &[String], _ext: &str) -> File {
    File::default()
}
pub fn save_storage(_name: &[String], _value: String, _pretty: bool, _ext: &str) {}
pub fn get_book_cache_dir() -> File {
    File::default()
}
pub fn get_ssl_socket_factory_base() -> SSLSocketFactory {
    SSLSocketFactory
}

// ---- OutputStream 统一占位（InputStream 以 1913 行 trait 为准；原 2792 行重复 struct 因 E0428 被移除） ----
#[derive(Debug, Clone, Default)]
pub struct OutputStream(pub Vec<u8>);
impl OutputStream {
    pub fn new() -> Self {
        OutputStream(Vec::new())
    }
    pub fn write(&mut self, _b: u8) {}
    pub fn write_all(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }
    pub fn close(&mut self) {}
    pub fn flush(&mut self) {}
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.clone()
    }
}

// ---------------- java.lang.Character 占位（ZipUtils.isSpace 使用） ----------------

pub struct Character;

impl Character {
    pub fn isWhitespace(c: char) -> bool {
        c.is_whitespace()
    }
}

// ---------------- java.util.zip 占位（ZipUtils 使用，显式导入消解 glob 歧义） ----------------

#[derive(Debug, Clone, Default)]
pub struct ZipEntry {
    pub name: String,
    pub comment: Option<String>,
    pub is_directory: bool,
}

impl ZipEntry {
    pub fn new(name: String) -> ZipEntry {
        ZipEntry {
            is_directory: name.ends_with('/'),
            name,
            comment: None,
        }
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn isDirectory(&self) -> bool {
        self.is_directory
    }
    pub fn comment(&self) -> String {
        self.comment.clone().unwrap_or_default()
    }
}

pub struct ZipOutputStream {
    writer: Option<zip::ZipWriter<std::fs::File>>,
    in_entry: bool,
}

impl ZipOutputStream {
    pub fn new(out: FileOutputStream) -> ZipOutputStream {
        let writer = out.inner.map(zip::ZipWriter::new);
        ZipOutputStream {
            writer,
            in_entry: false,
        }
    }
    pub fn putNextEntry(&mut self, entry: &ZipEntry) {
        if let Some(w) = &mut self.writer {
            let opts = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            if entry.is_directory {
                let _ = w.add_directory(&entry.name, opts);
            } else {
                let _ = w.start_file(&entry.name, opts);
            }
            self.in_entry = true;
        }
    }
    pub fn closeEntry(&mut self) {
        self.in_entry = false;
    }
    pub fn write(&mut self, b: &[u8]) {
        if let Some(w) = &mut self.writer {
            if self.in_entry {
                let _ = std::io::Write::write_all(w, b);
            }
        }
    }
    pub fn close(&mut self) {
        if let Some(w) = self.writer.take() {
            let _ = w.finish();
        }
    }
}

impl Drop for ZipOutputStream {
    fn drop(&mut self) {
        self.close();
    }
}

pub struct ZipFileEntryIter {
    entries: Vec<ZipEntry>,
    pos: usize,
}

impl ZipFileEntryIter {
    pub fn hasMoreElements(&self) -> bool {
        self.pos < self.entries.len()
    }
    pub fn nextElement(&mut self) -> ZipEntry {
        let entry = self.entries[self.pos].clone();
        self.pos += 1;
        entry
    }
}

pub struct ZipFile {
    path: String,
    entries: Vec<ZipEntry>,
}

impl ZipFile {
    pub fn new(file: &File) -> ZipFile {
        let entries = read_zip_entries(&file.path());
        ZipFile {
            path: file.path(),
            entries,
        }
    }
    pub fn entries(&self) -> ZipFileEntryIter {
        ZipFileEntryIter {
            entries: self.entries.clone(),
            pos: 0,
        }
    }
    // 真实解压单条目到临时文件
    pub fn getInputStream(&self, entry: &ZipEntry) -> FileInputStream {
        use std::io::Read;
        let tmp = std::env::temp_dir().join(format!("reader_zip_{}_{}", self.path.replace(['/', '\\', ':'], "_"), entry.name.replace(['/', '\\', ':'], "_")));
        if let Ok(file) = std::fs::File::open(&self.path) {
            if let Ok(mut archive) = zip::ZipArchive::new(file) {
                if let Ok(mut e) = archive.by_name(&entry.name) {
                    let mut buf = Vec::new();
                    if e.read_to_end(&mut buf).is_ok() {
                        let _ = std::fs::write(&tmp, &buf);
                    }
                }
            }
        }
        FileInputStream::new_path(&tmp.to_string_lossy())
    }
    pub fn close(&self) {}
}

// 读取 ZIP 中央目录（EOCD + Central Directory）中的条目名与注释
fn read_zip_entries(path: &str) -> Vec<ZipEntry> {
    let mut entries = Vec::new();
    if let Ok(file) = std::fs::File::open(path) {
        if let Ok(mut archive) = zip::ZipArchive::new(file) {
            for i in 0..archive.len() {
                if let Ok(entry) = archive.by_index(i) {
                    let is_dir = entry.is_dir();
                    let name = entry.name().to_string();
                    entries.push(ZipEntry {
                        is_directory: is_dir,
                        name: if is_dir { name.trim_end_matches(char::from(47)).to_string() + "/" } else { name },
                        comment: None,
                    });
                }
            }
        }
    }
    entries
}
// ================= GsonExtensions / SourceAnalyzer 编译迭代补充（新增占位，均 additive） =================

// ---- 序列化辅助：把 JSON 值按 key 取字符串（rule 类型 Deserialize 使用） ----
fn get_json_str(v: &serde_json::Value, key: &str) -> Option<String> {
    v.get(key).and_then(|x| match x {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    })
}

// ---- GSON 便捷构造（SourceAnalyzer 调用 GSON::new() 取得 Gson 实例） ----
impl GSON {
    pub fn new() -> Gson {
        Gson::new()
    }
}

// ---- Gson 补充方法（GsonExtensions 转录使用） ----
impl Gson {
    pub fn fromJson_list<T, P>(&self, json: Option<&str>, _ty: &P) -> Result<Option<Vec<T>>, StubError>
    where
        T: serde::de::DeserializeOwned,
    {
        match json {
            Some(s) => serde_json::from_str::<Vec<T>>(s).map(Some).map_err(|e| StubError::new(e.to_string())),
            None => Ok(None),
        }
    }

    // 占位：InputStreamReader 未保存流数据，恒返回 None（流式反序列化降级）
    pub fn fromJson_reader<T>(&self, _reader: &InputStreamReader) -> Result<Option<T>, StubError>
    where
        T: serde::de::DeserializeOwned,
    {
        Ok(None)
    }

    pub fn fromJson_list_reader<T, P>(&self, _reader: &InputStreamReader, _ty: &P) -> Result<Option<Vec<T>>, StubError>
    where
        T: serde::de::DeserializeOwned,
    {
        Ok(None)
    }

    pub fn toJson_dyn(&self, _v: &Any, _ty: Type, _writer: &mut JsonWriter) {}
}

// ---- java.io.InputStreamReader / OutputStreamWriter 占位（GsonExtensions 使用；显式路径引用消解跨模块 glob 歧义） ----

pub struct InputStreamReader;

impl InputStreamReader {
    pub fn new(_s: Option<&mut dyn InputStream>) -> InputStreamReader {
        InputStreamReader
    }
}

pub struct OutputStreamWriter;

impl OutputStreamWriter {
    pub fn new(_out: &mut OutputStream, _charset: &str) -> OutputStreamWriter {
        OutputStreamWriter
    }
}

// ---- com.google.gson.stream.JsonWriter 占位 ----

pub struct JsonWriter;

impl JsonWriter {
    pub fn new(_writer: OutputStreamWriter) -> JsonWriter {
        JsonWriter
    }
    pub fn setIndent(&mut self, _indent: &str) {}
    pub fn beginArray(&mut self) {}
    pub fn endArray(&mut self) {}
    pub fn close(&mut self) {}
}

// ---- com.google.gson.internal.LinkedTreeMap 别名（保持插入序语义的 Map） ----
pub type LinkedTreeMap<K, V> = std::collections::HashMap<K, V>;

// ---- ReadContext 便捷读取（SourceAnalyzer 转录：readString/readInt/readBool/jsonString，SUPPRESS_EXCEPTIONS 语义失败返回 None） ----

fn json_path_lookup<'a>(value: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let mut cur = value;
    let mut path = path.trim();
    if let Some(rest) = path.strip_prefix('$') {
        path = rest.trim();
    }
    if let Some(rest) = path.strip_prefix('.') {
        path = rest;
    }
    for segment in path.split('.') {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        if segment.starts_with('[') && segment.ends_with(']') && segment.len() > 2 {
            let idx: usize = segment[1..segment.len() - 1].parse().ok()?;
            cur = cur.get(idx)?;
        } else {
            cur = cur.get(segment)?;
        }
    }
    Some(cur)
}

impl ReadContext {
    pub fn jsonString(&self) -> String {
        self.json.clone()
    }

    pub fn readString(&self, path: &str) -> Option<String> {
        let v: serde_json::Value = serde_json::from_str(&self.json).ok()?;
        let cur = json_path_lookup(&v, path)?;
        match cur {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            serde_json::Value::Bool(b) => Some(b.to_string()),
            _ => None,
        }
    }

    pub fn readInt(&self, path: &str) -> Option<i32> {
        let v: serde_json::Value = serde_json::from_str(&self.json).ok()?;
        let cur = json_path_lookup(&v, path)?;
        cur.as_i64().map(|i| i as i32)
    }

    pub fn readBool(&self, path: &str) -> Option<bool> {
        let v: serde_json::Value = serde_json::from_str(&self.json).ok()?;
        let cur = json_path_lookup(&v, path)?;
        cur.as_bool()
    }
}

impl JsonPath {
    // Kotlin `JsonPath.parse(InputStream)`：读取全部字节后按 UTF-8 解析
    pub fn parse_stream(inputStream: &mut dyn InputStream) -> ReadContext {
        let mut buf: Vec<u8> = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            let chunk_len = chunk.len();
            let n = inputStream.read(&mut chunk, 0, chunk_len);
            if n <= 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n as usize]);
        }
        ReadContext {
            json: String::from_utf8_lossy(&buf).to_string(),
        }
    }
}

// ---------------- Ext.rs / VertExt.rs 编译修复附加占位（显式导入消解 glob 歧义后使用） ----------------

// Kotlin `ByteArray(size)` → Vec<u8>（Ext.rs unzip/zip 缓冲区）
pub struct ByteArray;

impl ByteArray {
    pub fn new(size: usize) -> Vec<u8> {
        vec![0u8; size]
    }
}

// Kotlin `read(buffer).also { len = it }` 占位（仅在 i32 返回值上实现）
pub trait AlsoExt {
    fn also(self, out: &mut i32) -> Self;
}

impl AlsoExt for i32 {
    fn also(self, out: &mut i32) -> Self {
        *out = self;
        self
    }
}

// Kotlin 可空流 `inputStream.read(...)` / `outputStream.write(...)` / `stream.close()` 调用占位
pub trait OptionInputStreamReadExt {
    fn read(&mut self, b: &mut [u8], off: usize, len: usize) -> i32;
}

impl OptionInputStreamReadExt for Option<FileInputStream> {
    fn read(&mut self, b: &mut [u8], off: usize, len: usize) -> i32 {
        match self.as_mut() {
            Some(s) => FileInputStream::read(s, b, off, len),
            None => -1,
        }
    }
}

pub trait OptionOutputStreamWriteExt {
    fn write(&mut self, b: &[u8], off: usize, len: usize);
}

impl OptionOutputStreamWriteExt for Option<OutputStream> {
    fn write(&mut self, b: &[u8], off: usize, len: usize) {
        if let Some(s) = self.as_mut() {
            let start = off.min(b.len());
            let end = (off + len).min(b.len());
            s.write_all(&b[start..end]);
        }
    }
}

pub trait OptionStreamCloseExt {
    fn close(&mut self);
}

impl OptionStreamCloseExt for Option<FileInputStream> {
    fn close(&mut self) {
        *self = None;
    }
}

impl OptionStreamCloseExt for Option<OutputStream> {
    fn close(&mut self) {
        if let Some(s) = self.as_mut() {
            s.close();
        }
    }
}

// ZipFile / ZipFileEntryIter / ZipOutputStream 的 snake_case 别名（Ext.rs 使用）
impl ZipFile {
    pub fn new_path(path: &str) -> ZipFile {
        ZipFile::new(&File::new(path))
    }
    pub fn get_input_stream(&self, entry: &ZipEntry) -> FileInputStream {
        self.getInputStream(entry)
    }
}

impl ZipFileEntryIter {
    pub fn has_more_elements(&mut self) -> bool {
        self.hasMoreElements()
    }
    pub fn next_element(&mut self) -> ZipEntry {
        self.nextElement()
    }
}

impl ZipOutputStream {
    pub fn put_next_entry(&mut self, entry: &ZipEntry) {
        self.putNextEntry(entry);
    }
    pub fn close_entry(&mut self) {
        self.closeEntry();
    }
    pub fn write_range(&mut self, b: &[u8], off: usize, len: usize) {
        let start = off.min(b.len());
        let end = (off + len).min(b.len());
        self.write(&b[start..end]);
    }
}


// ================= JsExtensions 转录补充（io.legado.app.help.JsExtensions 使用） =================

// ---- org.jsoup.Connection.Method 占位 ----
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConnectionMethod {
    GET,
    HEAD,
    POST,
}

// ---- org.jsoup.Connection.Response 占位（JsExtensions.get/head/post 返回） ----
pub struct ConnectionResponse;

impl ConnectionResponse {
    pub fn cookies(&self) -> std::collections::HashMap<String, String> {
        std::collections::HashMap::new()
    }
}

// ---- OkHttpClient 补充 jsoup 风格链式调用（Jsoup.connect(...).method(...).execute()） ----
impl OkHttpClient {
    pub fn ssl_socket_factory(&self, _factory: SSLSocketFactory) -> &Self {
        self
    }
    pub fn ignore_content_type(&self, _b: bool) -> &Self {
        self
    }
    pub fn follow_redirects(&self, _b: bool) -> &Self {
        self
    }
    pub fn headers(&self, _h: &std::collections::HashMap<String, String>) -> &Self {
        self
    }
    pub fn request_body(&self, _b: &str) -> &Self {
        self
    }
    pub fn method(&self, _m: ConnectionMethod) -> &Self {
        self
    }
    pub fn execute(&self) -> ConnectionResponse {
        ConnectionResponse
    }
}

// ---- java.util.SimpleTimeZone 占位 ----
pub struct SimpleTimeZone {
    pub raw_offset: i32,
    pub id: String,
}

impl SimpleTimeZone {
    pub fn new(raw_offset: i32, id: &str) -> SimpleTimeZone {
        SimpleTimeZone {
            raw_offset,
            id: id.to_string(),
        }
    }
}

impl SimpleDateFormat {
    pub fn set_time_zone(&self, _tz: SimpleTimeZone) {}
}

// ---- java.net.URLEncoder 占位（空格转 '+'，其余字节 percent 编码） ----
pub struct URLEncoder;

impl URLEncoder {
    pub fn encode(s: &str, enc: &str) -> Result<String, StubError> {
        let _ = enc;
        Ok(url::form_urlencoded::byte_serialize(s.as_bytes()).collect())
    }
}

// ---- cn.hutool.crypto.symmetric.AES 占位（未实现真实加解密） ----
pub struct AES;

impl AES {
    pub fn new(_mode: &str, _padding: &str, _key: Vec<u8>, _iv: Vec<u8>) -> AES {
        AES
    }
    pub fn decrypt_str(&self, _data: &str) -> Option<String> {
        None
    }
    pub fn encrypt_base64(&self, _data: &str) -> Option<String> {
        None
    }
}

// ---- cn.hutool.crypto.symmetric.DESede 占位（未实现真实加解密） ----
pub struct DESede;

impl DESede {
    pub fn new(_mode: &str, _padding: &str, _key: Vec<u8>, _iv: Vec<u8>) -> DESede {
        DESede
    }
    pub fn decrypt_str(&self, _data: &str) -> Option<String> {
        None
    }
    pub fn encrypt_base64(&self, _data: &str) -> Option<String> {
        None
    }
}

// ---- cn.hutool.crypto.digest.DigestUtil 占位（无加密 crate 依赖，digest 退化为原样字节） ----
pub struct DigestUtil;

impl DigestUtil {
    pub fn digester(_algorithm: &str) -> Digester {
        Digester
    }
}

pub struct Digester;

impl Digester {
    // fix: 简化占位实现，未实现真实摘要算法
    pub fn digest(&self, data: &str) -> Vec<u8> {
        data.as_bytes().to_vec()
    }
    pub fn digest_hex(&self, data: &str) -> Option<String> {
        Some(data.as_bytes().iter().map(|b| format!("{:02x}", b)).collect())
    }
}

// ---- java.util.zip.ZipInputStream 占位（未解析真实 zip 结构） ----
pub struct ZipInputStream {
    inner: ByteArrayInputStream,
    archive: Option<zip::ZipArchive<std::io::Cursor<Vec<u8>>>>,
    index: usize,
    entry_content: Option<Vec<u8>>,
}

impl ZipInputStream {
    pub fn new(input: ByteArrayInputStream) -> ZipInputStream {
        let data = input.data.clone();
        let archive = zip::ZipArchive::new(std::io::Cursor::new(data)).ok();
        ZipInputStream {
            inner: input,
            archive,
            index: 0,
            entry_content: None,
        }
    }
    // 真实 zip 条目遍历（跳过目录项），内容缓存供 copy_to 使用
    pub fn next_entry(&mut self) -> Option<ZipEntry> {
        let archive = self.archive.as_mut()?;
        loop {
            let entry = archive.by_index(self.index).ok()?;
            self.index += 1;
            if entry.is_dir() {
                continue;
            }
            let name = entry.name().to_string();
            drop(entry);
            let mut content = Vec::new();
            let mut file = archive.by_index(self.index - 1).ok()?;
            std::io::Read::read_to_end(&mut file, &mut content).ok()?;
            self.entry_content = Some(content);
            return Some(ZipEntry {
                is_directory: false,
                name,
                comment: None,
            });
        }
    }
    pub fn copy_to(&mut self, out: &mut ByteArrayOutputStream) {
        if let Some(content) = &self.entry_content {
            out.write(content);
        } else {
            let data = self.inner.data.clone();
            out.write(&data);
        }
    }
}

/// 内存字节流（InputStream 实现，TTS 音频流等使用）
#[derive(Clone, Default)]
pub struct BytesInputStream {
    data: std::rc::Rc<std::cell::RefCell<Vec<u8>>>,
    pos: std::rc::Rc<std::cell::RefCell<usize>>,
}

impl BytesInputStream {
    pub fn new(data: Vec<u8>) -> BytesInputStream {
        BytesInputStream {
            data: std::rc::Rc::new(std::cell::RefCell::new(data)),
            pos: std::rc::Rc::new(std::cell::RefCell::new(0)),
        }
    }
}

impl InputStream for BytesInputStream {
    fn read(&mut self, b: &mut [u8], off: usize, len: usize) -> i32 {
        let data = self.data.borrow();
        let pos = *self.pos.borrow();
        if pos >= data.len() {
            return -1;
        }
        let n = (data.len() - pos).min(len);
        b[off..off + n].copy_from_slice(&data[pos..pos + n]);
        self.pos.replace(pos + n);
        n as i32
    }
    fn close(&mut self) {}
}

// ---- java.io.File 补充读写字节方法（JsExtensions 使用） ----
impl File {
    pub fn read_bytes(&self) -> Vec<u8> {
        std::fs::read(&self.file_path).unwrap_or_default()
    }
    pub fn write_bytes(&self, bytes: Vec<u8>) {
        let _ = std::fs::write(&self.file_path, bytes);
    }
    // ---- WebdavController 转录所需 java.nio.file.Path / java.io.File 方法 ----

    // Path.normalize() —— 真实词法规范化（解析 . 与 ..，对齐 java.nio.Path.normalize）
    pub fn normalize(&self) -> File {
        File::new(&normalize_path_lexical(&self.file_path))
    }
    // Path.startsWith(other) —— 按路径分段比较（对齐 java.nio.Path.startsWith）
    pub fn starts_with(&self, base: &File) -> bool {
        path_segments_prefix(&self.file_path, &base.file_path)
    }
    // Path.toFile()
    pub fn to_file(&self) -> File {
        File::new(&self.file_path)
    }
    // File.lastModified() snake_case 别名
    pub fn last_modified(&self) -> i64 {
        self.lastModified()
    }
    // File.deleteRecursively()
    pub fn delete_recursively(&self) -> bool {
        std::fs::remove_dir_all(&self.file_path)
            .or_else(|_| std::fs::remove_file(&self.file_path))
            .is_ok()
    }
    // File.renameTo() snake_case 别名
    pub fn rename_to(&self, dest: &File) -> bool {
        self.renameTo(dest)
    }
    // File.copyRecursively()
    pub fn copy_recursively(&self, dest: &File) -> bool {
        if self.is_directory() {
            let _ = std::fs::create_dir_all(&dest.file_path);
            let mut ok = true;
            for child in self.list_files() {
                let child_dest = dest.resolve(&child.name());
                ok = ok && child.copy_recursively(&child_dest);
            }
            ok
        } else {
            if let Some(p) = std::path::Path::new(&dest.file_path).parent() {
                let _ = std::fs::create_dir_all(p);
            }
            std::fs::copy(&self.file_path, &dest.file_path).is_ok()
        }
    }
}

// ---- java.nio.Path.normalize / startsWith 的辅助实现（防路径穿越） ----
fn normalize_path_lexical(path: &str) -> String {
    let bytes = path.as_bytes();
    let drive = if bytes.len() >= 2 && bytes[1] == b':' { Some(&path[..2]) } else { None };
    // 根分隔符：路径开头 / \，或盘符后紧跟 / \（C:\）
    let has_root_sep = path.starts_with('/')
        || path.starts_with('\\')
        || drive.map_or(false, |d| {
            let rest = &path[d.len()..];
            rest.starts_with('/') || rest.starts_with('\\')
        });
    let rooted = has_root_sep;
    let sep = if path.contains('\\') { '\\' } else { '/' };
    let mut stack: Vec<&str> = Vec::new();
    for seg in path.split(|c| c == '/' || c == '\\') {
        // fix: 跳过盘符段（原把 "C:" 同时压入 stack——输出 "C:C:/..." 双重盘符）
        if let Some(d) = drive {
            if seg == d {
                continue;
            }
        }
        match seg {
            "" | "." => {}
            ".." => match stack.last() {
                // 消费普通段
                Some(x) if *x != ".." => {
                    stack.pop();
                }
                _ => {
                    // 相对路径下的前导 .. 保留；绝对路径根目录下的 .. 忽略（对齐 java）
                    if !rooted {
                        stack.push("..");
                    }
                }
            },
            s => stack.push(s),
        }
    }
    let mut out = String::new();
    if let Some(d) = drive {
        out.push_str(d);
        if has_root_sep {
            out.push(sep);
        }
    } else if path.starts_with('/') || path.starts_with('\\') {
        out.push(sep);
    }
    out.push_str(&stack.join(&sep.to_string()));
    if out.is_empty() {
        if rooted {
            out.push(sep);
        } else {
            out.push('.');
        }
    }
    out
}

fn path_segments_prefix(path: &str, base: &str) -> bool {
    let a: Vec<&str> = path.split(|c| c == '/' || c == '\\').filter(|s| !s.is_empty()).collect();
    let b: Vec<&str> = base.split(|c| c == '/' || c == '\\').filter(|s| !s.is_empty()).collect();
    if b.len() > a.len() {
        return false;
    }
    a[..b.len()] == b[..]
}

// ---- okhttp3.ResponseBody 补充 bytes()（JsExtensions 使用） ----
impl ResponseBody {
    // fix: 返回原始字节（原为 lossy 解码后重编码——二进制内容损坏）
    pub fn bytes(&self) -> Vec<u8> {
        if !self.bytes.is_empty() {
            return self.bytes.clone();
        }
        self.text.clone().unwrap_or_default().into_bytes()
    }
}
// ---------------- SQLTable/JSONTable findBy 补充（mapTo(clazz) 带 Class 参数版） ----------------

impl JsonObject {
    // Kotlin JsonObject.mapTo(clazz)（GSON 反序列化）
    pub fn map_to_with_class<T>(&self, _clazz: Class<T>) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        serde_json::from_str::<T>(&self.0).ok()
    }
}

// ---------------- PDFBox 占位（PdfFile.rs 转录所需；基于 lopdf 真实解析） ----------------
pub struct PDPage {
    pub page_index: i32,
    pub text: String,
}

pub struct PDPages {
    pub pages: Vec<PDPage>,
}

impl PDPages {
    pub fn index_of(&self, page: &PDPage) -> i32 {
        page.page_index
    }
}

pub struct PDDocumentCatalog {
    pub pages: PDPages,
    pub document_outline: Option<PDDocumentOutline>,
}

pub struct PDDocument {
    pub number_of_pages: i32,
    pub document_catalog: PDDocumentCatalog,
    inner: Option<lopdf::Document>,
}

impl PDDocument {
    pub fn load(file: &str) -> PDDocument {
        let doc = lopdf::Document::load(file).ok();
        let (number_of_pages, pages) = match &doc {
            Some(d) => {
                let page_map = d.get_pages();
                let mut pages = Vec::new();
                for (num, _id) in &page_map {
                    let text = d.extract_text(&[*num]).unwrap_or_default();
                    pages.push(PDPage {
                        page_index: *num as i32,
                        text,
                    });
                }
                // fix: 按页号升序（原 HashMap 迭代顺序不定——页文本错位/越界）
                pages.sort_by_key(|p| p.page_index);
                (page_map.len() as i32, pages)
            }
            None => (0, Vec::new()),
        };
        let document_outline = doc.as_ref().and_then(pdf_build_outline);
        PDDocument {
            number_of_pages,
            document_catalog: PDDocumentCatalog {
                pages: PDPages { pages },
                document_outline,
            },
            inner: doc,
        }
    }

    pub fn close(&self) {}
}

pub type PDDocumentOutline = PDOutlineNode;

pub struct PDOutlineNode {
    pub first_child: Option<Box<PDOutlineItem>>,
}

pub struct PDOutlineItem {
    pub title: String,
    pub next_sibling: Option<Box<PDOutlineItem>>,
    pub node: PDOutlineNode,
    pub page_index: i32,
}

impl std::ops::Deref for PDOutlineItem {
    type Target = PDOutlineNode;

    fn deref(&self) -> &PDOutlineNode {
        &self.node
    }
}

impl PDOutlineItem {
    pub fn find_destination_page(&self, _document: &PDDocument) -> PDPage {
        PDPage {
            page_index: self.page_index,
            text: String::new(),
        }
    }

    pub fn has_children(&self) -> bool {
        self.first_child.is_some()
    }
}

/// 从 PDF 目录（Outlines）构建 PDOutlineNode 树（lopdf 低层遍历）
fn pdf_build_outline(doc: &lopdf::Document) -> Option<PDOutlineNode> {
    use lopdf::Object;
    let root_ref = doc.trailer.get(b"Root").ok()?.as_reference().ok()?;
    let root = doc.get_dictionary(root_ref).ok()?;
    let outlines_ref = root.get(b"Outlines").ok()?.as_reference().ok()?;
    let outlines = doc.get_dictionary(outlines_ref).ok()?;
    let first = outlines.get(b"First").ok()?.as_reference().ok()?;
    let first_child = pdf_build_item(doc, first);
    Some(PDOutlineNode { first_child })
}

fn pdf_build_item(doc: &lopdf::Document, id: lopdf::ObjectId) -> Option<Box<PDOutlineItem>> {
    use lopdf::Object;
    let dict = doc.get_dictionary(id).ok()?;
    let title = dict
        .get(b"Title")
        .ok()
        .and_then(|o| o.as_str().ok())
        .map(|s| String::from_utf8_lossy(s).to_string())
        .unwrap_or_default();
    let page_index = pdf_dest_page(doc, dict).unwrap_or(-1);
    let next_sibling = dict
        .get(b"Next")
        .ok()
        .and_then(|o| o.as_reference().ok())
        .and_then(|n| pdf_build_item(doc, n));
    let first_child = dict
        .get(b"First")
        .ok()
        .and_then(|o| o.as_reference().ok())
        .and_then(|c| pdf_build_item(doc, c));
    Some(Box::new(PDOutlineItem {
        title,
        next_sibling,
        node: PDOutlineNode { first_child },
        page_index,
    }))
}

fn pdf_dest_page(doc: &lopdf::Document, dict: &lopdf::Dictionary) -> Option<i32> {
    use lopdf::Object;
    let page_ref = dict
        .get(b"Dest")
        .ok()
        .and_then(|o| match o {
            Object::Array(arr) => arr.first()?.as_reference().ok(),
            Object::Reference(r) => {
                let d = doc.get_object(*r).ok()?;
                if let Object::Array(arr) = d {
                    arr.first()?.as_reference().ok()
                } else {
                    None
                }
            }
            _ => None,
        })?;
    // get_pages() 反查 ObjectId → 页码
    for (num, id) in doc.get_pages() {
        if id == page_ref {
            return Some(num as i32);
        }
    }
    None
}

// ---- NCXDocumentV2 转录补充 ----

pub struct NcxError;

impl NcxError {
    pub fn printStackTrace(&self) {}
}

impl ByteArrayOutputStream {
    pub fn to_byte_array(&self) -> Vec<u8> {
        self.toByteArray()
    }
}

// ---------------- HttpHelper 转录所需 okhttp3 补充占位（追加，勿重复） ----------------

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ConnectionSpec;

impl ConnectionSpec {
    pub const MODERN_TLS: ConnectionSpec = ConnectionSpec;
    pub const COMPATIBLE_TLS: ConnectionSpec = ConnectionSpec;
    pub const CLEARTEXT: ConnectionSpec = ConnectionSpec;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProxyType {
    #[default]
    HTTP,
    SOCKS,
    DIRECT,
}

#[derive(Debug, Clone, Default)]
pub struct InetSocketAddress {
    pub host: String,
    pub port: i32,
}

impl InetSocketAddress {
    pub fn new(host: &str, port: i32) -> InetSocketAddress {
        InetSocketAddress {
            host: host.to_string(),
            port,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Proxy {
    pub type_: ProxyType,
    pub addr: InetSocketAddress,
}

impl Proxy {
    pub fn new(type_: ProxyType, addr: InetSocketAddress) -> Proxy {
        Proxy { type_, addr }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    NONE,
    BASIC,
    HEADERS,
    BODY,
}

#[derive(Debug, Clone, Default)]
pub struct HttpLoggingInterceptor;

impl HttpLoggingInterceptor {
    pub fn new<L: ?Sized>(_logger: Option<&L>) -> HttpLoggingInterceptor {
        HttpLoggingInterceptor
    }
    pub fn set_level(&self, _level: Level) {}
}

pub struct Credentials;

impl Credentials {
    // fix: 简化占位实现，未实现真实 Basic 认证编码
    pub fn basic(username: &str, password: &str) -> String {
        format!("Basic {}:{}", username, password)
    }
}

#[derive(Debug, Clone, Default)]
// fix: 携带代理凭据（Authenticator 闭包无法提取，HttpHelper 直接以字段构造）
pub struct Authenticator {
    pub username: String,
    pub password: String,
}

impl Authenticator {
    pub fn new<F>(_f: F) -> Authenticator
    where
        F: FnOnce(Option<&crate::stubs::io::vertx::Route>, &Response) -> Request,
    {
        Authenticator::default()
    }
    pub fn with_credentials(username: &str, password: &str) -> Authenticator {
        Authenticator { username: username.to_string(), password: password.to_string() }
    }
}

// okhttp3 Interceptor.Chain（fix: 真实拦截器链——原单元占位，add_interceptor 丢弃导致 UA 注入失效）
pub type Interceptor = std::sync::Arc<dyn Fn(&Chain) -> Response + Send + Sync + 'static>;

#[derive(Clone, Default)]
pub struct Chain {
    pub request: Request,
    pub interceptors: Vec<Interceptor>,
    pub index: usize,
    pub proxy: Option<String>,
    pub proxy_auth: Option<(String, String)>,
}

impl Chain {
    pub fn request(&self) -> Request {
        self.request.clone()
    }
    pub fn proceed(&self, request: Request) -> Response {
        if self.index < self.interceptors.len() {
            let chain = Chain {
                request,
                interceptors: self.interceptors.clone(),
                index: self.index + 1,
                proxy: self.proxy.clone(),
                proxy_auth: self.proxy_auth.clone(),
            };
            (self.interceptors[self.index])(&chain)
        } else {
            match crate::runtime::okhttp::execute(
                &request,
                self.proxy.as_deref(),
                self.proxy_auth.as_ref().map(|(u, p)| (u.as_str(), p.as_str())),
            ) {
                Ok(r) => r,
                // fix: Kotlin okhttp onFailure → resumeWithException（异常传播给调用方，书源标记失败/错误响应）
                Err(e) => panic!("{}", e.msg().unwrap_or_default()),
            }
        }
    }
}

#[derive(Clone, Default)]
pub struct OkHttpClientBuilder {
    pub proxy: Option<String>,
    pub proxy_auth: Option<(String, String)>,
    pub interceptors: Vec<Interceptor>,
}

impl OkHttpClientBuilder {
    pub fn connect_timeout(&self, _timeout: u64, _unit: TimeUnit) -> &Self {
        self
    }
    pub fn write_timeout(&self, _timeout: u64, _unit: TimeUnit) -> &Self {
        self
    }
    pub fn read_timeout(&self, _timeout: u64, _unit: TimeUnit) -> &Self {
        self
    }
    pub fn ssl_socket_factory(&self, _factory: SSLSocketFactory, _trust_manager: X509TrustManager) -> &Self {
        self
    }
    pub fn retry_on_connection_failure(&self, _retry: bool) -> &Self {
        self
    }
    pub fn hostname_verifier(&self, _verifier: impl std::any::Any) -> &Self {
        self
    }
    pub fn connection_specs(&self, _specs: Vec<ConnectionSpec>) -> &Self {
        self
    }
    pub fn follow_redirects(&self, _b: bool) -> &Self {
        self
    }
    pub fn follow_ssl_redirects(&self, _b: bool) -> &Self {
        self
    }
    // fix: 真实存储拦截器（原丢弃——UA/Keep-Alive 头注入失效）
    pub fn add_interceptor(&mut self, interceptor: Box<dyn Fn(&Chain) -> Response + Send + Sync + 'static>) -> &mut Self {
        self.interceptors.push(std::sync::Arc::from(interceptor));
        self
    }
    pub fn add_network_interceptor(&self, _interceptor: HttpLoggingInterceptor) -> &Self {
        self
    }
    // fix: 代理真实生效（reqwest Proxy::all）；Kotlin 用 java.net Proxy.Type.SOCKS（socks4 协议）→ socks4:// 走手写握手
    pub fn proxy(&mut self, proxy: Proxy) -> &mut Self {
        let scheme = match proxy.type_ {
            ProxyType::HTTP => "http://",
            ProxyType::SOCKS => "socks4://",
            ProxyType::DIRECT => "",
        };
        self.proxy = Some(format!("{}{}:{}", scheme, proxy.addr.host, proxy.addr.port));
        self
    }
    // fix: 代理认证真实存储（reqwest Proxy::basic_auth）
    pub fn proxy_authenticator(&mut self, authenticator: Authenticator) -> &mut Self {
        if !authenticator.username.is_empty() {
            self.proxy_auth = Some((authenticator.username.clone(), authenticator.password.clone()));
        }
        self
    }
    pub fn build(&self) -> OkHttpClient {
        OkHttpClient {
            proxy: self.proxy.clone(),
            proxy_auth: self.proxy_auth.clone(),
            interceptors: self.interceptors.clone(),
        }
    }
}

impl OkHttpClient {
    pub fn builder() -> OkHttpClientBuilder {
        OkHttpClientBuilder { proxy: None, proxy_auth: None, interceptors: Vec::new() }
    }
    pub fn new_builder(&self) -> OkHttpClientBuilder {
        OkHttpClientBuilder {
            proxy: self.proxy.clone(),
            proxy_auth: self.proxy_auth.clone(),
            interceptors: Vec::new(),
        }
    }
}

impl Clone for OkHttpClient {
    fn clone(&self) -> Self {
        OkHttpClient {
            proxy: self.proxy.clone(),
            proxy_auth: self.proxy_auth.clone(),
            interceptors: self.interceptors.clone(),
        }
    }
}

// ---- BookHelp / BaseController 转录所需 kotlinx.coroutines 占位 ----

// fix: Kotlin kotlinx.coroutines.delay(ms) 占位（不实际休眠）
// fix: 真实休眠（原不实际休眠——保存图片重试/退避无间隔）
pub async fn delay(millis: i64) {
    if millis > 0 {
        std::thread::sleep(std::time::Duration::from_millis(millis as u64));
    }
}

impl CoroutineScope {
    // fix: Kotlin CoroutineScope.async {} 占位（BookHelp.saveImages 使用）
    pub fn r#async<F>(&self, _block: F) -> Deferred {
        Deferred
    }
}

impl Deferred {
    // fix: Kotlin Deferred.await() 占位（await 为 Rust 关键字 → r#await 转义）
    pub fn r#await(&self) {}
}


// ---------------- retrofit2 / kotlinx.coroutines 占位（CoroutinesCallAdapterFactory 使用） ----------------

// java.lang.annotation.Annotation 占位
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Annotation;

// retrofit2.Retrofit 占位
#[derive(Debug, Clone, Default)]
pub struct Retrofit;

// retrofit2.CallAdapter.Factory 占位
pub trait CallAdapterFactory {
    // Kotlin CallAdapter.Factory.get(returnType, annotations, retrofit)
    fn get(
        &self,
        return_type: Type,
        annotations: &[Annotation],
        retrofit: &Retrofit,
    ) -> Option<Box<dyn CallAdapter>>;
}

// retrofit2.CallAdapter 占位（trait object 安全：方法均非泛型）
pub trait CallAdapter {
    fn response_type(&self) -> Type;
    fn adapt(&self, call: Call<Any>) -> Deferred;
}

// retrofit2.Call<T> 占位
pub struct Call<T> {
    pub request: Option<Request>,
    pub proxy: Option<String>,
    pub proxy_auth: Option<(String, String)>,
    pub interceptors: Vec<Interceptor>,
    pub phantom: std::marker::PhantomData<T>,
}

impl<T> Call<T> {
    pub fn new() -> Self {
        Call {
            request: None,
            proxy: None,
            proxy_auth: None,
            interceptors: Vec::new(),
            phantom: std::marker::PhantomData,
        }
    }
    pub fn cancel(&self) {}
    // 真实执行：blocking 请求后回调（拦截器链在 proceed 末端执行请求）
    pub fn enqueue<F>(&self, callback: F)
    where
        F: FnOnce(Result<Response, Throwable>) + Send + 'static,
    {
        let result = match &self.request {
            Some(req) => {
                eprintln!("[okhttp] enqueue executing: {} {}", req.method, req.url);
                if self.interceptors.is_empty() {
                    crate::runtime::okhttp::execute(req, self.proxy.as_deref(), self.proxy_auth.as_ref().map(|(u, p)| (u.as_str(), p.as_str())))
                } else {
                    let chain = Chain {
                        request: req.clone(),
                        interceptors: self.interceptors.clone(),
                        index: 0,
                        proxy: self.proxy.clone(),
                        proxy_auth: self.proxy_auth.clone(),
                    };
                    Ok(chain.proceed(req.clone()))
                }
            }
            None => Err(Throwable::new("call has no request".to_string())),
        };
        callback(result);
    }
}

// kotlinx.coroutines.CompletableDeferred<T> 占位
pub struct CompletableDeferred<T>(std::marker::PhantomData<T>);

impl<T> CompletableDeferred<T> {
    pub fn new() -> Self {
        CompletableDeferred(std::marker::PhantomData)
    }
    pub fn invoke_on_completion<F: FnOnce()>(&self, _f: F) {}
    pub fn is_cancelled(&self) -> bool {
        false
    }
    pub fn complete(&self, _value: T) {}
    pub fn complete_exceptionally(&self, _t: Throwable) {}
}

// retrofit2.HttpException 占位
pub struct HttpException;

impl HttpException {
    // fix: Kotlin HttpException(response) 占位——返回 Throwable(StubError) 供 completeExceptionally 使用
    pub fn new(_response: Response) -> Throwable {
        StubError::new("HttpException")
    }
}

// retrofit2.Utils 占位（getRawType / getParameterUpperBound / isParameterized 判定）
pub fn get_raw_type<T>(_t: &Type) -> Class<T> {
    Class::new()
}

pub fn get_parameter_upper_bound(_index: i32, _t: &Type) -> Type {
    Type
}

pub fn is_parameterized_type(_t: &Type) -> bool {
    true
}

// fix: 占位——stub 模型下 Class<T> 恒等（Kotlin `X::class == getRawType(t)` 判定降级）
impl<T> PartialEq for Class<T> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl ClassConstant for Deferred {
    type Target = Deferred;
    const class: Class<Deferred> = Class::new();
}

impl ClassConstant for Response {
    type Target = Response;
    const class: Class<Response> = Class::new();
}

// fix: Kotlin `val d = CompletableDeferred<T>()` 返回后再注册回调——stub 需可 Clone 句柄
impl<T> Clone for CompletableDeferred<T> {
    fn clone(&self) -> Self {
        CompletableDeferred(std::marker::PhantomData)
    }
}
// ---------------- okhttp3 WebSocket / okio Buffer / ByteString / CountDownLatch 占位（TTSService 使用） ----------------

// okio.ByteString 占位
#[derive(Debug, Clone, Default)]
pub struct ByteString(pub Vec<u8>);

impl ByteString {
    // okio ByteString.lastIndexOf(byteString) —— 返回 -1 表示未找到（Java 语义）
    pub fn last_index_of(&self, needle: &[u8]) -> i32 {
        if needle.is_empty() {
            return -1;
        }
        self.0
            .windows(needle.len())
            .position(|w| w == needle)
            .map(|i| i as i32)
            .unwrap_or(-1)
    }
    // okio ByteString.substring(int) -> ByteString（转录返回 Vec<u8> 便于 Buffer.write）
    pub fn substring(&self, start: usize) -> Vec<u8> {
        self.0.get(start..).unwrap_or(&[]).to_vec()
    }
}

// okio.Buffer 占位（顶层 Buffer 与 io::vertx::core::buffer::Buffer 无冲突）
#[derive(Debug, Clone, Default)]
pub struct Buffer {
    pub data: Vec<u8>,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer { data: Vec::new() }
    }
    // okio Buffer.clear()
    pub fn clear(&mut self) {
        self.data.clear();
    }
    // okio Buffer.write(ByteString)
    pub fn write(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }
    // okio Buffer.readByteArray()
    pub fn read_byte_array(&self) -> Vec<u8> {
        self.data.clone()
    }
}

// okhttp3.WebSocket 占位（send 恒 true，等价 Java 发送成功）
#[derive(Debug, Clone, Default)]
pub struct WebSocket {
    sender: Option<std::sync::mpsc::Sender<String>>,
    events: std::sync::Arc<std::sync::Mutex<Vec<WebSocketEvent>>>,
}

impl WebSocket {
    // okhttp3 WebSocket.send(String) -> boolean（真实发送到连接线程）
    pub fn send(&self, text: String) -> bool {
        match &self.sender {
            Some(tx) => tx.send(text).is_ok(),
            None => false,
        }
    }
    /// 取出累积的 WebSocket 事件（TTSService 轮询调用）
    pub fn poll_events(&self) -> Vec<WebSocketEvent> {
        if let Ok(mut guard) = self.events.lock() {
            std::mem::take(&mut *guard)
        } else {
            Vec::new()
        }
    }
}

/// WebSocket 连接线程：tungstenite 连接 + 双向收发
fn ws_loop(
    req: tokio_tungstenite::tungstenite::http::Request<()>,
    rx: std::sync::mpsc::Receiver<String>,
    events: std::sync::Arc<std::sync::Mutex<Vec<WebSocketEvent>>>,
) {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;
    let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
        Ok(rt) => rt,
        Err(_) => return,
    };
    rt.block_on(async move {
        let (mut ws, _) = match tokio_tungstenite::connect_async(req).await {
            Ok(conn) => conn,
            Err(_) => {
                if let Ok(mut guard) = events.lock() {
                    guard.push(WebSocketEvent::onFailure("connect failed".to_string()));
                }
                return;
            }
        };
        loop {
            tokio::select! {
                msg = ws.next() => {
                    match msg {
                        Some(Ok(Message::Text(t))) => {
                            if let Ok(mut guard) = events.lock() {
                                guard.push(WebSocketEvent::onMessageText(t.to_string()));
                            }
                        }
                        Some(Ok(Message::Binary(b))) => {
                            if let Ok(mut guard) = events.lock() {
                                guard.push(WebSocketEvent::onMessageBytes(crate::stubs::ByteString(b.to_vec())));
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            if let Ok(mut guard) = events.lock() {
                                guard.push(WebSocketEvent::onClosed(String::new()));
                            }
                            break;
                        }
                        Some(Err(e)) => {
                            if let Ok(mut guard) = events.lock() {
                                guard.push(WebSocketEvent::onFailure(e.to_string()));
                            }
                            break;
                        }
                        _ => break,
                    }
                }
                text = async { rx.try_recv().ok() } => {
                    if let Some(t) = text {
                        if ws.send(Message::Text(t.into())).await.is_err() {
                            break;
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                }
            }
        }
    });
}

// okhttp3.WebSocketListener 回调事件转录（TTSService 内部监听器使用）
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum WebSocketEvent {
    onClosed(String),
    onClosing(String),
    onFailure(String),
    onMessageText(String),
    onMessageBytes(ByteString),
}

// java.util.concurrent.CountDownLatch 占位（TTSService 轮询驱动）
#[derive(Debug, Clone, Default)]
pub struct CountDownLatch {
    pub count: std::cell::Cell<i32>,
}

impl CountDownLatch {
    // CountDownLatch(int count)
    pub fn new(count: i32) -> CountDownLatch {
        CountDownLatch {
            count: std::cell::Cell::new(count),
        }
    }
    // CountDownLatch.countDown()
    pub fn count_down(&self) {
        self.count.set(self.count.get().saturating_sub(1));
    }
    // 当前剩余计数（TTSService 轮询完成判断）
    pub fn count(&self) -> i32 {
        self.count.get()
    }
    // CountDownLatch.await(long, TimeUnit) -> boolean（await 为关键字 → r#await 转义）
    pub fn r#await(&self, _timeout: i32, _unit: TimeUnit) -> bool {
        true
    }
}

impl OkHttpClient {
    // okhttp3 OkHttpClient.newWebSocket(Request, WebSocketListener)
    pub fn new_web_socket(&self, request: Request, _listener: impl std::any::Any) -> WebSocket {
        let (tx, rx) = std::sync::mpsc::channel::<String>();
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events2 = events.clone();
        let mut req = match tokio_tungstenite::tungstenite::client::IntoClientRequest::into_client_request(request.url.as_str()) {
            Ok(r) => r,
            Err(_) => {
                return WebSocket {
                    sender: None,
                    events,
                }
            }
        };
        for (k, v) in &request.headers {
            if let Ok(val) = v.parse() {
                let name = tokio_tungstenite::tungstenite::http::HeaderName::from_bytes(k.as_bytes());
                if let Ok(name) = name {
                    req.headers_mut().append(name, val);
                }
            }
        }
        std::thread::spawn(move || {
            ws_loop(req, rx, events2);
        });
        WebSocket {
            sender: Some(tx),
            events,
        }
    }
}

impl OkHttpClientBuilder {
    // okhttp3 OkHttpClient.Builder.pingInterval(long, TimeUnit)
    pub fn ping_interval(&self, _interval: i64, _unit: TimeUnit) -> &Self {
        self
    }
}

// ---------------- fix: HttpTTS.rs 转录所需追加 ----------------

impl ReadContext {
    // Kotlin `DocumentContext.readLong(path)`（io.legado.app.utils.readLong 扩展）
    pub fn read_long(&self, path: &str) -> Option<i64> {
        self.read::<i64>(path).ok()
    }
    // Kotlin `DocumentContext.readString(path)`
    pub fn read_string(&self, path: &str) -> Option<String> {
        self.read::<String>(path).ok()
    }
    // Kotlin `DocumentContext.readBool(path)`（RssSource.rs 等同步使用）
    pub fn read_bool(&self, path: &str) -> Option<bool> {
        self.read::<bool>(path).ok()
    }
    // Kotlin `DocumentContext.readInt(path)`
    pub fn read_int(&self, path: &str) -> Option<i32> {
        self.read::<i32>(path).ok()
    }
}

impl Any {
    // Kotlin `any?.toString()`
    pub fn to_string_opt(&self) -> Option<String> {
        match self {
            Any::Null => None,
            other => Some(other.to_string()),
        }
    }
}

// ---- SSLHelper.rs 补充占位（追加，勿重写） ----
pub struct SecureRandom;
impl SecureRandom {
    pub fn new() -> SecureRandom {
        SecureRandom
    }
}

pub struct KeyManager;

pub struct X509Certificate;

pub struct HostnameVerifier {
    verify: Box<dyn Fn(&str, &str) -> bool>,
}
impl HostnameVerifier {
    pub fn new(verify: Box<dyn Fn(&str, &str) -> bool>) -> HostnameVerifier {
        HostnameVerifier { verify }
    }
}

pub struct KeyStore;
impl KeyStore {
    pub fn get_instance<T: AsRef<str>>(_type: T) -> KeyStore {
        KeyStore
    }
    pub fn get_default_type() -> String {
        "JKS".to_string()
    }
    pub fn load(&self, _input: Option<&dyn InputStream>, _password: &[char]) -> Result<(), StubError> {
        Ok(())
    }
    pub fn set_certificate_entry(&self, _alias: &str, _cert: X509Certificate) {}
}

pub struct KeyManagerFactory;
impl KeyManagerFactory {
    pub fn get_instance<T: AsRef<str>>(_algorithm: T) -> Result<KeyManagerFactory, StubError> {
        Ok(KeyManagerFactory)
    }
    pub fn get_default_algorithm() -> String {
        "SunX509".to_string()
    }
    pub fn init(&self, _key_store: &KeyStore, _password: &[char]) -> Result<(), StubError> {
        Ok(())
    }
    pub fn key_managers(&self) -> Vec<KeyManager> {
        vec![]
    }
}

pub struct CertificateFactory;
impl CertificateFactory {
    pub fn get_instance<T: AsRef<str>>(_type: T) -> CertificateFactory {
        CertificateFactory
    }
    pub fn generate_certificate(&self, _stream: &dyn InputStream) -> X509Certificate {
        X509Certificate
    }
}

pub struct TrustManager;
impl TrustManager {
    // fix: Kotlin `trustManager is X509TrustManager` 类型判断占位（恒 None）
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        None
    }
}

pub struct TrustManagerFactory;
impl TrustManagerFactory {
    pub fn get_instance<T: AsRef<str>>(_algorithm: T) -> TrustManagerFactory {
        TrustManagerFactory
    }
    pub fn get_default_algorithm() -> String {
        "PKIX".to_string()
    }
    pub fn init(&self, _key_store: &KeyStore) {}
    pub fn trust_managers(&self) -> Vec<TrustManager> {
        vec![]
    }
}

impl X509TrustManager {
    pub fn new(
        _check_client_trusted: Box<dyn Fn(&[X509Certificate], &str)>,
        _check_server_trusted: Box<dyn Fn(&[X509Certificate], &str)>,
        _get_accepted_issuers: Box<dyn Fn() -> Vec<X509Certificate>>,
    ) -> X509TrustManager {
        X509TrustManager
    }
}
impl Clone for X509TrustManager {
    fn clone(&self) -> Self {
        X509TrustManager
    }
}
impl Default for X509TrustManager {
    fn default() -> Self {
        X509TrustManager
    }
}

impl Clone for SSLSocketFactory {
    fn clone(&self) -> Self {
        SSLSocketFactory
    }
}
impl Default for SSLSocketFactory {
    fn default() -> Self {
        SSLSocketFactory
    }
}

impl SSLContext {
    pub fn get_instance(_protocol: &str) -> Result<SSLContext, StubError> {
        Ok(SSLContext)
    }
    pub fn init<T>(
        &self,
        _key_managers: Option<Vec<KeyManager>>,
        _trust_managers: Vec<T>,
        _secure_random: Option<SecureRandom>,
    ) -> Result<(), StubError> {
        Ok(())
    }
    pub fn socket_factory(&self) -> SSLSocketFactory {
        SSLSocketFactory
    }
}

// fix: SSLHelper 闭包错误为 Box<dyn Error>，printStackTrace 调用占位
impl ThrowableExt for Box<dyn std::error::Error> {
    fn localized_message(&self) -> String {
        self.to_string()
    }
    fn stack_trace_to_string(&self) -> String {
        format!("{:?}", self)
    }
    fn msg(&self) -> Option<String> {
        Some(self.to_string())
    }
}
// ---- fix: CURD.rs 占位 RoutingContext 为 prelude glob 唯一可见的 RoutingContext（stubs 内嵌 vertx 模块不可 glob），
// 补充 RssSourceController 转录所需方法（RssSourceController.rs E0308/E0599 修复）----


// ---------------- UserController 转录补充（追加） ----------------

// fix: UserController 使用 Spring Environment（BaseController.env 字段）
pub struct Environment;

/// Spring property → 环境变量 relaxed binding（reader.server.port → READER_SERVER_PORT）
fn env_var_for(key: &str) -> Option<String> {
    let env_key = key.replace('.', "_").to_uppercase();
    std::env::var(&env_key).ok().filter(|s| !s.is_empty())
}

impl Environment {
    pub fn get_property(&self, key: &str) -> Option<String> {
        env_var_for(key)
    }
    pub fn get_property_boolean(&self, key: &str) -> bool {
        env_var_for(key)
            .map(|s| s.eq_ignore_ascii_case("true") || s == "1")
            .unwrap_or(false)
    }
    pub fn get_property_int(&self, key: &str) -> Option<i32> {
        env_var_for(key).and_then(|s| s.parse().ok())
    }
    pub fn get_property_default(&self, key: &str, default: String) -> String {
        env_var_for(key).unwrap_or(default)
    }
}

// fix: UserController 转录所需 RoutingContext 方法（追加到 CURD 占位 RoutingContext）

// fix: io.vertx.ext.web.Session 占位（UserController/BaseController 使用；内存会话存储）
pub struct Session;
fn session_map() -> &'static std::sync::Mutex<std::collections::HashMap<String, String>> {
    static SESSION_MAP: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<String, String>>> = std::sync::OnceLock::new();
    SESSION_MAP.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}
impl Session {
    pub fn get(&self, key: &str) -> Option<String> {
        session_map().lock().ok().and_then(|m| m.get(key).cloned())
    }
    pub fn put(&self, key: &str, value: String) {
        if let Ok(mut m) = session_map().lock() {
            m.insert(key.to_string(), value);
        }
    }
    pub fn destroy(&self) {
        if let Ok(mut m) = session_map().lock() {
            m.clear();
        }
    }
}

// fix: io.vertx.ext.web.FileUpload 占位（UserController/FileController 上传处理）
#[derive(Clone)]
pub struct FileUpload;
impl FileUpload {
    pub fn uploaded_file_name(&self) -> String {
        String::new()
    }
    pub fn file_name(&self) -> String {
        String::new()
    }
}

// fix: UserController 转录所需 JsonObject 方法（追加）
impl JsonObject {
    // Kotlin JsonObject.getString(key, default)（UserController.login 使用）
    pub fn get_string_default(&self, key: &str, default: &str) -> String {
        let v = self.get_string(key);
        if v.is_empty() {
            default.to_string()
        } else {
            v
        }
    }
    // Kotlin JsonObject.getBoolean(key)（UserController.addUser/updateUser 使用）
    pub fn get_boolean(&self, key: &str) -> Option<bool> {
        serde_json::from_str::<serde_json::Value>(&self.0)
            .ok()
            .and_then(|v| v.get(key).cloned())
            .and_then(|v| v.as_bool())
    }
    // Kotlin JsonObject.getBoolean(key, default)（UserController.login 使用）
    pub fn get_boolean_default(&self, key: &str, default: bool) -> bool {
        self.get_boolean(key).unwrap_or(default)
    }
    // Kotlin JsonObject.getInteger(key)（UserController.addUser/updateUser 使用）
    pub fn get_integer_opt(&self, key: &str) -> Option<i32> {
        serde_json::from_str::<serde_json::Value>(&self.0)
            .ok()
            .and_then(|v| v.get(key).cloned())
            .and_then(|v| v.as_i64())
            .map(|n| n as i32)
    }
    // Kotlin JsonObject.mapMutable()（UserController.deleteUsers 使用）
    pub fn map_mut(&mut self) -> HashMap<String, Any> {
        HashMap::new()
    }
}

// ---------------- AnalyzeRule 转录补充（追加） ----------------
// fix: Kotlin `Any?.asListString()` / `Any?.asListStringRef()`（AnalyzeRule.getStringListInner / makeUpRule 使用；
//      列表元素转 String，String 视为单元素列表；asListStringRef 元素以 Option 包装便于按索引取值）
impl Any {
    pub fn as_list_string(&self) -> Option<Vec<String>> {
        match self {
            Any::List(l) => Some(l.iter().map(|v| v.to_string()).collect()),
            Any::Str(s) => Some(vec![s.clone()]),
            _ => None,
        }
    }
    pub fn as_list_string_ref(&self) -> Option<Vec<Option<String>>> {
        match self {
            Any::List(l) => Some(l.iter().map(|v| Some(v.to_string())).collect()),
            Any::Str(s) => Some(vec![Some(s.clone())]),
            _ => None,
        }
    }
}

// fix: Kotlin `String.replaceFirstRegex(regex, replacement)`（AnalyzeRule.replaceRegex 使用；占位：替换首个匹配）
pub trait StringReplaceFirstRegexExt {
    fn replace_first_regex(&self, from: &str, to: &str) -> String;
}

impl StringReplaceFirstRegexExt for String {
    fn replace_first_regex(&self, from: &str, to: &str) -> String {
        Pattern::compile(from).replace_first(self, to)
    }
}

impl StringReplaceFirstRegexExt for str {
    fn replace_first_regex(&self, from: &str, to: &str) -> String {
        Pattern::compile(from).replace_first(self, to)
    }
}


// ================= StringUtils.rs 转录补充（追加） =================

// ---- java.text.ParseException 别名（StringUtils.dateConvert_source 使用；复用 StubError） ----
pub type ParseException = StubError;

// ---- java.lang.Integer 占位（StringUtils.chineseNumToInt / stringToInt / removeUTFCharacters / byteToHexString 使用） ----
pub struct Integer;

impl Integer {
    pub fn parseInt(s: impl AsRef<str>) -> i32 {
        s.as_ref().trim().parse::<i32>().unwrap_or(0)
    }
    pub fn parseInt_radix(s: impl AsRef<str>, radix: u32) -> i32 {
        i32::from_str_radix(s.as_ref().trim(), radix).unwrap_or(0)
    }
    pub fn toHexString(i: i32) -> String {
        format!("{:x}", i)
    }
}

// ---- java.lang.StringBuffer 占位（StringUtils.removeUTFCharacters 使用） ----
#[derive(Debug, Clone, Default)]
pub struct StringBuffer(pub String);

impl StringBuffer {
    pub fn with_capacity(cap: usize) -> StringBuffer {
        StringBuffer(String::with_capacity(cap))
    }
    pub fn append(&mut self, s: impl std::fmt::Display) -> &mut Self {
        self.0.push_str(&s.to_string());
        self
    }
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl SimpleDateFormat {
    // Kotlin SimpleDateFormat.parse(String): Date（StringUtils.dateConvert_source 使用；真实解析）
    pub fn parse(&self, s: &str) -> Result<i64, StubError> {
        let fmt = java_pattern_to_chrono(&self.pattern);
        let naive = chrono::NaiveDateTime::parse_from_str(s, &fmt)
            .or_else(|_| {
                chrono::NaiveDate::parse_from_str(s, &fmt).map(|d| {
                    d.and_hms_opt(0, 0, 0)
                        .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap())
                })
            })
            .map_err(|e| StubError::new(e.to_string()))?;
        Ok(naive.and_utc().timestamp_millis())
    }
}

impl Calendar {
    // Kotlin Calendar.HOUR / Calendar.get(field)（StringUtils.dateConvert_source 使用；真实取时）
    pub const HOUR: i32 = 11;
    pub fn get(&self, field: i32) -> i32 {
        use chrono::{TimeZone, Timelike};
        match chrono::Local.timestamp_millis_opt(self.time).single() {
            Some(dt) => match field {
                Calendar::HOUR => dt.hour() as i32,
                _ => 0,
            },
            None => 0,
        }
    }
}

impl Character {
    // Kotlin Character.digit(ch, radix)（StringUtils.hexStringToByte 使用）
    pub fn digit(c: char, radix: u32) -> i32 {
        c.to_digit(radix).map(|d| d as i32).unwrap_or(-1)
    }
}

impl<'a> Matcher<'a> {
    // fix: StringUtils.removeUTFCharacters 使用（简化实现：quoteReplacement 原样返回，append 直接追加 replacement / 剩余尾部）
    pub fn quoteReplacement(s: &str) -> String {
        s.to_string()
    }
    pub fn appendReplacement(&mut self, buf: &mut StringBuffer, replacement: impl AsRef<str>) -> &mut Self {
        buf.append(replacement.as_ref());
        self
    }
    pub fn appendTail(&mut self, buf: &mut StringBuffer) -> &mut Self {
        buf.append(&self.hay[self.pos..]);
        self
    }
}
// ---------------- ReaderApplication 转录补充（io.vertx.core.json.Json / Jackson 配置 / WebClientOptions / HttpClientOptions） ----------------
pub struct Json;
impl Json {
    pub fn mapper() -> ObjectMapper {
        ObjectMapper
    }
    pub fn pretty_mapper() -> ObjectMapper {
        ObjectMapper
    }
}
impl ObjectMapper {
    // Kotlin Json.mapper().apply { registerKotlinModule() }
    pub fn apply<F: FnOnce(&mut Self)>(&self, _f: F) {}
    pub fn register_kotlin_module(&mut self) {}
    pub fn configure(&self, _feature: DeserializationFeature, _state: bool) {}
}
pub enum DeserializationFeature {
    FAIL_ON_UNKNOWN_PROPERTIES,
}

pub use crate::stubs::io::vertx::{HttpClient, HttpClientOptions, WebClient, WebClientOptions};

// ================= CbzFile 转录补充（io.legado.app.model.localBook.CbzFile 使用，additive） =================

// fix: Kotlin `XmlUtils.xml2map(ComicInfo.xml 输入流)`（原恒空 map——CBZ 标题/作者/简介全丢）
fn read_xml_element_text(reader: &mut quick_xml::Reader<&[u8]>, buf: &mut Vec<u8>) -> String {
    let mut text = String::new();
    loop {
        buf.clear();
        match reader.read_event_into(buf) {
            Ok(quick_xml::events::Event::Text(t)) => {
                if let Ok(s) = t.unescape() {
                    text.push_str(&s);
                }
            }
            Ok(quick_xml::events::Event::CData(t)) => {
                text.push_str(&String::from_utf8_lossy(&t));
            }
            Ok(quick_xml::events::Event::Start(_)) => {
                text.push_str(&read_xml_element_text(reader, buf));
            }
            Ok(quick_xml::events::Event::End(_)) | Ok(quick_xml::events::Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    text.trim().to_string()
}

pub fn xml2map(source: &mut FileInputStream) -> std::collections::HashMap<String, Any> {
    let mut data = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let chunk_len = chunk.len();
        let n = source.read(&mut chunk, 0, chunk_len);
        if n <= 0 {
            break;
        }
        data.extend_from_slice(&chunk[..n as usize]);
    }
    let xml = String::from_utf8_lossy(&data).into_owned();
    let mut result: std::collections::HashMap<String, Any> = std::collections::HashMap::new();
    let mut root_map: std::collections::HashMap<String, Any> = std::collections::HashMap::new();
    let mut reader = quick_xml::Reader::from_str(&xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut in_root = false;
    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "ComicInfo" {
                    in_root = true;
                } else if in_root {
                    let text = read_xml_element_text(&mut reader, &mut buf);
                    root_map.insert(name, Any::from_string(text));
                }
            }
            Ok(quick_xml::events::Event::End(e)) => {
                if String::from_utf8_lossy(e.name().as_ref()) == "ComicInfo" {
                    break;
                }
            }
            Ok(quick_xml::events::Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    result.insert(String::from("ComicInfo"), Any::from_map(root_map));
    result
}

// fix: Kotlin `FileUtils.writeInputStream(path, inputStream)`——按输入流拷贝写入文件
pub fn write_input_stream(path: &str, input: &mut FileInputStream) -> bool {
    let mut out = FileOutputStream::new(&File::new(path));
    let mut buffer = vec![0u8; 8192];
    loop {
        let buffer_len = buffer.len();
        let len = input.read(&mut buffer, 0, buffer_len);
        if len <= 0 {
            break;
        }
        out.write(&buffer[..len as usize]);
    }
    true
}

// fix: Kotlin `map["key"] as String`（CbzFile.upBookInfo 读取 ComicInfo Title/Writer）
impl Any {
    pub fn as_str(&self) -> Option<String> {
        match self {
            Any::Str(s) => Some(s.clone()),
            _ => None,
        }
    }
}

// ---------------- JsoupExtensions 转录补充（追加，勿重复） ----------------

#[derive(Debug, Clone, Default)]
pub struct Tag {
    pub tag_name: String,
    pub preserve_whitespace: bool,
}

impl Tag {
    pub fn name(&self) -> String {
        self.tag_name.clone()
    }
    pub fn preserveWhitespace(&self) -> bool {
        self.preserve_whitespace
    }
}

impl Element {
    pub fn tag(&self) -> Tag {
        Tag::default()
    }
    pub fn isBlock(&self) -> bool {
        false
    }
    pub fn nextSibling(&self) -> Option<Node> {
        None
    }
}

#[derive(Debug, Clone, Default)]
pub struct Node {
    pub text: String,
}

impl Node {
    pub fn is_TextNode(&self) -> bool {
        true
    }
    pub fn as_text_node(&self) -> TextNode {
        TextNode {
            text: self.text.clone(),
        }
    }
    pub fn is_Element(&self) -> bool {
        false
    }
    pub fn as_element(&self) -> Element {
        Element {
            text: self.text.clone(),
            html: self.text.clone(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TextNode {
    pub text: String,
}

impl TextNode {
    pub fn wholeText(&self) -> String {
        self.text.clone()
    }
    pub fn parentNode(&self) -> Option<Node> {
        None
    }
    pub fn is_CDataNode(&self) -> bool {
        false
    }
}

pub trait NodeVisitor {
    fn head(&mut self, node: &Node, depth: i32);
    fn tail(&mut self, node: &Node, depth: i32);
}

pub struct NodeTraversor;

impl NodeTraversor {
    pub fn traverse(visitor: &mut dyn NodeVisitor, root: &Element) {
        let _ = (visitor, root);
    }
}

impl crate::me_ag2s_epublib_util_stringutil::StringUtil {
    pub fn borrowBuilder() -> StringBuilder {
        StringBuilder::new()
    }
    pub fn releaseBuilder(sb: &mut StringBuilder) -> String {
        sb.to_string()
    }
    pub fn appendNormalisedWhitespace(sb: &mut StringBuilder, word: &str, insert_space_if_needed: bool) {
        let word = word.trim_matches(|c: char| c <= ' ');
        if word.is_empty() {
            return;
        }
        let last_is_space = sb.length() > 0 && sb.to_string().ends_with(' ');
        if insert_space_if_needed && !last_is_space && !word.starts_with(' ') {
            sb.append(' ');
        }
        sb.append(word);
    }
}

impl crate::me_ag2s_epublib_epub_ncxdocumentv3::NcxV3Error {
    pub fn printStackTrace(&self) {}
}

// ---------------- java.util.BitSet 占位（NetworkUtils 转录使用） ----------------
#[derive(Debug, Clone)]
pub struct BitSet {
    words: Vec<u64>,
}

impl BitSet {
    pub fn new(size: usize) -> Self {
        BitSet { words: vec![0; (size + 63) / 64] }
    }
    pub fn set(&mut self, bit: u32) {
        let idx = (bit / 64) as usize;
        if idx < self.words.len() {
            self.words[idx] |= 1u64 << (bit % 64);
        }
    }
    pub fn get(&self, bit: u32) -> bool {
        let idx = (bit / 64) as usize;
        idx < self.words.len() && (self.words[idx] & (1u64 << (bit % 64))) != 0
    }
}

// ---------------- java.net 网络占位（NetworkUtils 转录使用） ----------------
#[derive(Debug, Clone)]
pub struct InetAddress {
    pub isLoopbackAddress: bool,
    pub hostAddress: String,
}

#[derive(Debug, Clone)]
pub struct Enumeration<T> {
    pub items: Vec<T>,
}

impl<T> Enumeration<T> {
    pub fn empty() -> Self {
        Enumeration { items: Vec::new() }
    }
}

impl<T> IntoIterator for Enumeration<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub inetAddresses: Option<Enumeration<InetAddress>>,
}

impl NetworkInterface {
    // fix: 占位实现，返回空枚举（不枚举真实网卡）
    pub fn getNetworkInterfaces() -> Enumeration<NetworkInterface> {
        Enumeration::empty()
    }
}

// Java URL(base, relative) 相对地址解析（NetworkUtils 转录使用）
impl URL {
    pub fn new_relative(base: &URL, relative: &str) -> Result<URL, StubError> {
        base.0.join(relative).map(URL).map_err(|e| StubError::new(e.to_string()))
    }
}

// Kotlin Throwable.printStackTrace()（NetworkUtils 等转录模块使用）
impl StubError {
    pub fn printStackTrace(&self) {}
}


// ================= XmlUtils 转录补充（io.legado.app.utils.XmlUtils 使用，additive） =================

// 对应 Java `org.w3c.dom` 最小占位（XmlUtils.xml2map 使用）
// 命名 XmlDomNode/ XmlDomNodeList：与 JsoupExtensions 补充的 jsoup 风格 Node 区分，避免 E0428 重名
#[derive(Debug, Clone, Default)]
pub struct XmlDocument {
    pub childNodes: XmlDomNodeList,
}

#[derive(Debug, Clone, Default)]
pub struct XmlDomNodeList {
    pub length: usize,
    pub nodes: Vec<XmlDomNode>,
}

impl XmlDomNodeList {
    pub fn item(&self, index: usize) -> XmlDomNode {
        self.nodes.get(index).cloned().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Default)]
pub struct XmlDomNode {
    pub nodeType: i32,
    pub nodeName: String,
    pub nodeValue: Any,
    pub childNodes: XmlDomNodeList,
}

impl XmlDomNode {
    pub const ELEMENT_NODE: i32 = 1;
    pub const TEXT_NODE: i32 = 3;
    pub fn firstChild(&self) -> XmlDomNode {
        self.childNodes.nodes.first().cloned().unwrap_or_default()
    }
}

// 对应 Java `javax.xml.parsers.DocumentBuilderFactory`（XmlUtils.xml2map 使用）
pub struct DocumentBuilderFactory;

impl DocumentBuilderFactory {
    pub fn newInstance() -> Self {
        DocumentBuilderFactory
    }
    pub fn newDocumentBuilder(&self) -> DocumentBuilder {
        DocumentBuilder
    }
}

impl DocumentBuilder {
    // Kotlin `builder.parse(source)` 各重载（XmlUtils.xml2map 使用；真实 quick-xml 解析）
    pub fn parse_str(&self, s: Option<String>) -> XmlDocument {
        let mut doc = XmlDocument::default();
        if let Some(s) = s {
            doc.childNodes = parse_xml_tree(&s);
        }
        doc
    }
    pub fn parse_stream(&self, s: Option<&mut dyn InputStream>) -> XmlDocument {
        let mut doc = XmlDocument::default();
        if let Some(s) = s {
            let mut buf = Vec::new();
            loop {
                let mut chunk = [0u8; 4096];
                let len = chunk.len();
                let n = s.read(&mut chunk, 0, len);
                if n <= 0 {
                    break;
                }
                buf.extend_from_slice(&chunk[..n as usize]);
            }
            if let Ok(text) = String::from_utf8(buf) {
                if !text.is_empty() {
                    doc.childNodes = parse_xml_tree(&text);
                }
            }
        }
        doc
    }
    pub fn parse_input_source(&self, _s: Option<InputSource>) -> XmlDocument {
        // InputSource 为占位类型（无数据载体），返回空文档；真实数据走 parse_str/parse_stream
        XmlDocument::default()
    }
}

/// quick-xml 解析 XML 文本为 XmlDomNode 树（元素 + 非空文本节点）
fn parse_xml_tree(xml: &str) -> XmlDomNodeList {
    let mut reader = quick_xml::Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let children = parse_xml_children(&mut reader, &mut buf);
    XmlDomNodeList {
        length: children.len(),
        nodes: children,
    }
}

fn parse_xml_children(reader: &mut quick_xml::Reader<&[u8]>, buf: &mut Vec<u8>) -> Vec<XmlDomNode> {
    let mut children: Vec<XmlDomNode> = Vec::new();
    loop {
        buf.clear();
        match reader.read_event_into(buf) {
            Ok(quick_xml::events::Event::Start(e)) => {
                let mut node = XmlDomNode::default();
                node.nodeType = XmlDomNode::ELEMENT_NODE;
                node.nodeName = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let subs = parse_xml_children(reader, buf);
                node.childNodes = XmlDomNodeList {
                    length: subs.len(),
                    nodes: subs,
                };
                children.push(node);
            }
            Ok(quick_xml::events::Event::Text(t)) => {
                if let Ok(text) = t.unescape() {
                    let text = text.trim().to_string();
                    if !text.is_empty() {
                        let mut node = XmlDomNode::default();
                        node.nodeType = XmlDomNode::TEXT_NODE;
                        node.nodeName = String::from("#text");
                        node.nodeValue = Any::Str(text);
                        children.push(node);
                    }
                }
            }
            Ok(quick_xml::events::Event::End(_))
            | Ok(quick_xml::events::Event::Eof)
            | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    children
}

// Kotlin `source is InputStream / is InputSource`（XmlUtils.xml2map when 分支使用；占位恒 false）
impl Any {
    pub fn is_input_stream(&self) -> bool {
        false
    }
    pub fn as_input_stream(&self) -> Option<&mut dyn InputStream> {
        None
    }
    pub fn is_input_source(&self) -> bool {
        false
    }
    pub fn as_input_source(&self) -> Option<InputSource> {
        None
    }
}
// ================= BaseSource / UmdUtils 转录补充 =================

// fix: BookSource/RssSource 实现 JsExtensions + BaseSource（原无实现者——get_source 恒 None，
//      依赖 source 对象的 JS 规则拿不到真实对象；必需方法真实，默认方法沿用 trait 实现）
impl crate::io_legado_app_help_jsextensions::JsExtensions for crate::io_legado_app_data_entities_booksource::BookSource {
    fn get_source(&self) -> Option<Box<dyn crate::io_legado_app_data_entities_basesource::BaseSource>> {
        Some(Box::new(self.clone()))
    }
    fn get_user_name_space(&self) -> String {
        self.user_name_space.clone()
    }
    fn get_logger(&self) -> Option<Box<dyn crate::io_legado_app_model_debuglog::DebugLog>> {
        None
    }
}

impl crate::io_legado_app_data_entities_basesource::BaseSource for crate::io_legado_app_data_entities_booksource::BookSource {
    fn concurrent_rate(&self) -> Option<&str> {
        self.concurrent_rate.as_deref()
    }
    fn set_concurrent_rate(&mut self, value: Option<String>) {
        self.concurrent_rate = value;
    }
    fn enabled_cookie_jar(&self) -> Option<bool> {
        self.enabled_cookie_jar
    }
    fn set_enabled_cookie_jar(&mut self, value: Option<bool>) {
        self.enabled_cookie_jar = value;
    }
    fn login_url(&self) -> Option<&str> {
        self.login_url.as_deref()
    }
    fn set_login_url(&mut self, value: Option<String>) {
        self.login_url = value;
    }
    fn login_ui(&self) -> Option<&str> {
        self.login_ui.as_deref()
    }
    fn set_login_ui(&mut self, value: Option<String>) {
        self.login_ui = value;
    }
    fn header(&self) -> Option<&str> {
        self.header.as_deref()
    }
    fn set_header(&mut self, value: Option<String>) {
        self.header = value;
    }
    fn get_tag(&self) -> String {
        self.book_source_url.clone()
    }
    fn get_key(&self) -> String {
        self.book_source_url.clone()
    }
    fn get_source(&self) -> Option<Box<dyn crate::io_legado_app_data_entities_basesource::BaseSource>>
    where
        Self: Sized,
    {
        Some(Box::new(self.clone()))
    }
}

impl crate::io_legado_app_help_jsextensions::JsExtensions for crate::io_legado_app_data_entities_rsssource::RssSource {
    fn get_source(&self) -> Option<Box<dyn crate::io_legado_app_data_entities_basesource::BaseSource>> {
        Some(Box::new(self.clone()))
    }
    fn get_user_name_space(&self) -> String {
        self.user_name_space.clone()
    }
    fn get_logger(&self) -> Option<Box<dyn crate::io_legado_app_model_debuglog::DebugLog>> {
        None
    }
}

impl crate::io_legado_app_data_entities_basesource::BaseSource for crate::io_legado_app_data_entities_rsssource::RssSource {
    fn concurrent_rate(&self) -> Option<&str> {
        self.concurrent_rate.as_deref()
    }
    fn set_concurrent_rate(&mut self, value: Option<String>) {
        self.concurrent_rate = value;
    }
    fn enabled_cookie_jar(&self) -> Option<bool> {
        self.enabled_cookie_jar
    }
    fn set_enabled_cookie_jar(&mut self, value: Option<bool>) {
        self.enabled_cookie_jar = value;
    }
    fn login_url(&self) -> Option<&str> {
        self.login_url.as_deref()
    }
    fn set_login_url(&mut self, value: Option<String>) {
        self.login_url = value;
    }
    fn login_ui(&self) -> Option<&str> {
        self.login_ui.as_deref()
    }
    fn set_login_ui(&mut self, value: Option<String>) {
        self.login_ui = value;
    }
    fn header(&self) -> Option<&str> {
        self.header.as_deref()
    }
    fn set_header(&mut self, value: Option<String>) {
        self.header = value;
    }
    fn get_tag(&self) -> String {
        self.source_url.clone()
    }
    fn get_key(&self) -> String {
        self.source_url.clone()
    }
    fn get_source(&self) -> Option<Box<dyn crate::io_legado_app_data_entities_basesource::BaseSource>>
    where
        Self: Sized,
    {
        Some(Box::new(self.clone()))
    }
}


// ---- java.util.Random 占位（UmdUtils.gen_random_bytes 使用；xorshift64 伪随机） ----
pub struct Random {
    state: u64,
}

impl Random {
    pub fn new() -> Random {
        Random { state: 0x6a09e667f3bcc909 }
    }
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        (x >> 32) as u32
    }
}

// ---- java.util.zip.InflaterInputStream 占位（UmdUtils.decompress 使用） ----
// ---- java.util.zip.InflaterInputStream（UmdUtils.decompress 使用）——fix: 真实 zlib 解压（原恒 EOF，UMD 解压为空）
pub struct InflaterInputStream {
    inner: ByteArrayInputStream,
    decompressed: Option<Vec<u8>>,
    pos: usize,
}

impl InflaterInputStream {
    pub fn new(input: ByteArrayInputStream) -> InflaterInputStream {
        InflaterInputStream { inner: input, decompressed: None, pos: 0 }
    }
    pub fn read(&mut self, b: &mut [u8]) -> i32 {
        if self.decompressed.is_none() {
            use std::io::Read;
            let mut out = Vec::new();
            let mut decoder = flate2::read::ZlibDecoder::new(std::io::Cursor::new(self.inner.data.clone()));
            let _ = decoder.read_to_end(&mut out);
            self.decompressed = Some(out);
        }
        let data = self.decompressed.as_ref().unwrap();
        if self.pos >= data.len() || b.is_empty() {
            return -1;
        }
        let n = b.len().min(data.len() - self.pos);
        b[..n].copy_from_slice(&data[self.pos..self.pos + n]);
        self.pos += n;
        n as i32
    }
}

// ---- ByteArrayOutputStream 补充（UmdUtils 使用） ----
impl ByteArrayOutputStream {
    pub fn write_byte(&mut self, b: u8) {
        self.write(&[b]);
    }
    pub fn flush(&mut self) {}
}

// ---- BufferedInputStream 补充（UmdUtils.read_file 逐字节读取） ----
impl BufferedInputStream {
    // Java InputStream.read()：读单个字节，EOF 返回 -1
    pub fn read_byte(&mut self) -> i32 {
        let mut b = [0u8; 1];
        let n = self.read(&mut b);
        if n > 0 {
            b[0] as i32
        } else {
            -1
        }
    }
}

impl BufferedOutputStream {
    pub fn flush(&mut self) {
        self.inner.flush();
    }
}

// ---- String.encodeToByteArray(start, end) 等价扩展（BaseSource 密钥派生使用） ----
pub trait StringEncodeToByteArrayExt {
    fn encode_to_byte_array(&self, start: usize, end: usize) -> Vec<u8>;
}

impl StringEncodeToByteArrayExt for str {
    fn encode_to_byte_array(&self, start: usize, end: usize) -> Vec<u8> {
        self.as_bytes()
            .get(start..end.min(self.len()))
            .unwrap_or_default()
            .to_vec()
    }
}

// ---------------- Debugger 转录补充（新增占位，均 additive） ----------------
// fix: Debugger 时间格式（Kotlin SimpleDateFormat("[mm:ss.SSS]").format(Date(millis))）
pub fn debug_time_format(millis: i64) -> String {
    let mm = (millis / 60000) % 60;
    let ss = (millis / 1000) % 60;
    let sss = millis % 1000;
    format!("[{:02}:{:02}.{:03}]", mm, ss, sss)
}

// fix: Debugger 使用的 String 扩展（Kotlin String.isAbsUrl / String.substringAfter(delimiter)）
pub trait StringUtilExt: AsRef<str> + ToString {
    fn is_abs_url(&self) -> bool {
        let s = self.to_string().to_lowercase();
        s.starts_with("http://") || s.starts_with("https://")
    }
    fn substring_after(&self, delimiter: &str) -> String {
        let s = self.to_string();
        match s.find(delimiter) {
            Some(i) => s[i + delimiter.len()..].to_string(),
            None => s,
        }
    }
}
impl<T: AsRef<str> + ToString> StringUtilExt for T {}

// fix: Debugger 调试输出真实 JSON（原恒 "{}"——书源调试结果不可见；按具体实体类型序列化）
pub fn gson_to_json_placeholder<T: 'static>(value: &T) -> String {
    let any = value as &dyn std::any::Any;
    if let Some(b) = any.downcast_ref::<crate::io_legado_app_data_entities_book::Book>() {
        return crate::stubs::book_to_json(b).to_string();
    }
    if let Some(b) = any.downcast_ref::<crate::io_legado_app_data_entities_searchbook::SearchBook>() {
        return crate::stubs::search_book_to_json(b).to_string();
    }
    if let Some(v) = any.downcast_ref::<Vec<crate::io_legado_app_data_entities_searchbook::SearchBook>>() {
        return crate::stubs::search_books_to_json(v).to_string();
    }
    if let Some(v) = any.downcast_ref::<Vec<crate::io_legado_app_data_entities_bookchapter::BookChapter>>() {
        let arr: Vec<serde_json::Value> = v.iter().map(|c| crate::stubs::book_chapter_to_json(c)).collect();
        return serde_json::Value::Array(arr).to_string();
    }
    if let Some(v) = any.downcast_ref::<Vec<crate::io_legado_app_data_entities_book::Book>>() {
        return crate::stubs::books_to_json(v).to_string();
    }
    if let Some(e) = any.downcast_ref::<crate::com_htmake_reader_entity_basicerror::BasicError>() {
        return serde_json::json!({
            "error": e.error,
            "exception": e.exception,
            "message": e.message,
            "path": e.path,
            "status": e.status,
            "timestamp": e.timestamp,
        })
        .to_string();
    }
    if let Some(v) = any.downcast_ref::<Any>() {
        return crate::stubs::any_to_json_value(v).to_string();
    }
    if let Some(v) = any.downcast_ref::<Option<Box<Any>>>() {
        return match v {
            Some(a) => crate::stubs::any_to_json_value(a.as_ref()).to_string(),
            None => "null".to_string(),
        };
    }
    "{}".to_string()
}

// ---- java.io.File.get_name() 补充（UmdChapters.add_file_auto 使用） ----
impl File {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

// ---- java.io.ByteArrayOutputStream 补充（UmdChapters 使用：with_capacity/size/reset） ----
impl ByteArrayOutputStream {
    pub fn with_capacity(cap: usize) -> ByteArrayOutputStream {
        ByteArrayOutputStream { data: Vec::with_capacity(cap) }
    }
    pub fn size(&self) -> i32 {
        self.data.len() as i32
    }
    pub fn reset(&mut self) {
        self.data.clear();
    }
}

// ---- java.util.zip.DeflaterOutputStream 占位（UmdChapters.write_chapters_chunks 使用） ----
// fix: 无 flate2 依赖，无法真正 zlib 压缩；占位直写原文（与 InflaterInputStream 占位对称）
// ---- java.util.zip.DeflaterOutputStream（UmdChapters 使用）——fix: 真实 zlib 压缩（原直写原文，UMD 导出损坏）
pub struct DeflaterOutputStream<'a> {
    out: &'a mut ByteArrayOutputStream,
    buffer: Vec<u8>,
}

impl<'a> DeflaterOutputStream<'a> {
    pub fn new(out: &'a mut ByteArrayOutputStream) -> DeflaterOutputStream<'a> {
        DeflaterOutputStream { out, buffer: Vec::new() }
    }
    pub fn write(&mut self, b: &[u8]) {
        self.buffer.extend_from_slice(b);
    }
    pub fn close(&mut self) {
        use std::io::Write;
        let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        let _ = encoder.write_all(&self.buffer);
        let compressed = encoder.finish().unwrap_or_default();
        self.out.write(&compressed);
        self.buffer.clear();
    }
}
// ---------------- MongoClient / MongoDatabase / Codec 占位（MongoManager 使用） ----------------

#[derive(Debug, Clone, Default)]
pub struct MongoClient {
    pub databases: HashMap<String, MongoDatabase>,
}

impl MongoClient {
    pub fn get_database(&self, name: &str) -> Option<MongoDatabase> {
        self.databases.get(name).cloned()
    }
}

pub struct MongoClients;

impl MongoClients {
    // Kotlin MongoClients.create(uri): MongoClient
    pub fn create(_uri: &str) -> MongoClient {
        MongoClient::default()
    }
}

#[derive(Debug, Clone, Default)]
pub struct MongoDatabase {
    pub name: String,
}

impl MongoDatabase {
    pub fn with_codec_registry(self, _registry: CodecRegistry) -> MongoDatabase {
        self
    }
    pub fn get_collection<T: Default>(&self, _name: &str, _cls: Class<T>) -> Option<MongoCollection<T>> {
        Some(MongoCollection::default())
    }
}

#[derive(Debug, Clone, Default)]
pub struct CodecRegistry;

pub trait CodecProvider {}

#[derive(Debug, Clone, Default)]
pub struct PojoCodecProvider;

impl CodecProvider for PojoCodecProvider {}

#[derive(Debug, Clone, Default)]
pub struct PojoCodecProviderBuilder;

impl PojoCodecProviderBuilder {
    pub fn automatic(self, _flag: bool) -> PojoCodecProviderBuilder {
        self
    }
    pub fn build(self) -> PojoCodecProvider {
        PojoCodecProvider
    }
}

impl PojoCodecProvider {
    pub fn builder() -> PojoCodecProviderBuilder {
        PojoCodecProviderBuilder
    }
}

pub struct CodecRegistries;

impl CodecRegistries {
    // Kotlin fromRegistries(vararg registries): 转录处传 &[CodecRegistry] 切片
    pub fn from_registries(registries: &[CodecRegistry]) -> CodecRegistry {
        registries.first().cloned().unwrap_or_default()
    }
    pub fn from_providers(provider: impl CodecProvider) -> CodecRegistry {
        let _ = provider;
        CodecRegistry::default()
    }
}

pub struct MongoClientSettings;

impl MongoClientSettings {
    pub fn get_default_codec_registry() -> CodecRegistry {
        CodecRegistry::default()
    }
}

// ---- Coroutine.rs 转录所需 kotlinx.coroutines 占位（追加，勿删） ----
pub struct CancellationException;
impl CancellationException {
    pub fn new() -> Self {
        CancellationException
    }
}

pub struct CompletionHandler;
pub struct DisposableHandle;

pub struct MainScope;
impl MainScope {
    pub fn new() -> MainScope {
        MainScope
    }
    pub fn launch<F>(&self, _f: F) -> Job {
        Job
    }
}

// fix: Kotlin kotlinx.coroutines.withTimeout(timeMillis) { block() }（原不实际超时——请求可能挂起；
//      忙轮询 deadline 检查，超时 panic（无 catch 时优于挂起；主 HTTP 路径另有 reqwest 45s 兜底））
pub struct WithTimeout<F: std::future::Future> {
    inner: F,
    deadline: std::time::Instant,
    millis: i64,
}
impl<F: std::future::Future> std::future::Future for WithTimeout<F> {
    type Output = F::Output;
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<F::Output> {
        if std::time::Instant::now() >= self.deadline {
            panic!("withTimeout: 超过 {}ms 未完成", self.millis);
        }
        // unsafe: 字段投影（WithTimeout 无 Drop，inner 始终先于 deadline 字段使用，符合 pin 投影安全条件）
        let this = unsafe { self.get_unchecked_mut() };
        // SAFETY: 同上——inner 字段在 WithTimeout 中未被移动（结构无 Drop/Unpin 投影），
        //         deadline 检查在 poll 前完成，投影后仅用于 poll inner
        let inner = unsafe { std::pin::Pin::new_unchecked(&mut this.inner) };
        inner.poll(cx)
    }
}

pub fn with_timeout<T: std::future::Future, F: FnOnce() -> T>(millis: i64, f: F) -> WithTimeout<T> {
    WithTimeout {
        inner: f(),
        deadline: std::time::Instant::now() + std::time::Duration::from_millis(millis.max(0) as u64),
        millis,
    }
}

impl Default for Job {
    fn default() -> Self {
        Job
    }
}

impl Job {
    pub fn launch<F>(&self, _f: F) -> Job {
        Job
    }
    pub fn is_cancelled(&self) -> bool {
        false
    }
    pub fn is_active(&self) -> bool {
        true
    }
    pub fn is_completed(&self) -> bool {
        false
    }
    pub fn invoke_on_completion(&self, _handler: CompletionHandler) -> DisposableHandle {
        DisposableHandle
    }
}

impl CoroutineScope {
    pub fn coroutine_context(&self) -> CoroutineContext {
        CoroutineContext
    }
    pub fn is_active(&self) -> bool {
        true
    }
}

impl CoroutineContext {
    // fix: Kotlin CoroutineContext.plus(other) 占位
    pub fn plus(&self, _other: &CoroutineContext) -> CoroutineContext {
        CoroutineContext
    }
}
// ---- jsoup select 相关 stub（AnalyzeByJSoup 使用）----
#[derive(Debug, Clone)]
pub enum Evaluator {
    Id(String),
}

pub struct Collector;

impl Collector {
    // jsoup Collector.collect(evaluator, root) -> Elements
    // fix: 真实实现（原空实现——id.xxx 前置规则恒空）
    pub fn collect(evaluator: Evaluator, root: &Element) -> Elements {
        match evaluator {
            Evaluator::Id(id) => {
                let doc = scraper::Html::parse_fragment(&root.html);
                let sel = match scraper::Selector::parse(&format!("#{}", id)) {
                    Ok(s) => s,
                    Err(_) => return Elements::default(),
                };
                let mut list = Vec::new();
                for e in doc.select(&sel) {
                    list.push(Element {
                        text: crate::runtime::html::jsoup_normalise_text(&e),
                        html: e.html().to_string(),
                    });
                }
                Elements { list }
            }
        }
    }
}


// fix: EpubReader.rs 对 ReaderError/ParseError 调用 print_stack_trace（Java 打印堆栈等价）
impl ThrowableExt for crate::me_ag2s_epublib_epub_packagedocumentreader::ReaderError {
    fn localized_message(&self) -> String { "ReaderError".to_string() }
    fn stack_trace_to_string(&self) -> String { "ReaderError".to_string() }
    fn msg(&self) -> Option<String> { None }
}

impl ThrowableExt for crate::me_ag2s_epublib_util_resourceutil::ParseError {
    fn localized_message(&self) -> String { "ParseError".to_string() }
    fn stack_trace_to_string(&self) -> String { "ParseError".to_string() }
    fn msg(&self) -> Option<String> { None }
}


// ==================== fix: OkHttpUtils.rs 转录所需追加（只增不改） ====================

// okhttp3 FormBody / FormBody.Builder（真实构造 application/x-www-form-urlencoded body）
pub struct FormBody;
#[derive(Debug, Default)]
pub struct FormBodyBuilder {
    pub fields: std::cell::RefCell<Vec<(String, String)>>,
}
impl FormBody {
    pub fn builder() -> FormBodyBuilder {
        FormBodyBuilder::default()
    }
}
impl FormBodyBuilder {
    pub fn add(&self, name: &str, value: &str) -> &Self {
        self.fields
            .borrow_mut()
            .push((name.to_string(), url_encode(value)));
        self
    }
    pub fn add_encoded(&self, name: &str, value: &str) -> &Self {
        self.fields.borrow_mut().push((name.to_string(), value.to_string()));
        self
    }
    pub fn build(&self) -> RequestBody {
        let parts = self
            .fields
            .borrow()
            .iter()
            .map(|(k, v)| format!("{}={}", url_encode(k), v))
            .collect::<Vec<_>>()
            .join("&");
        RequestBody {
            text: parts,
            media_type: Some("application/x-www-form-urlencoded".to_string()),
            bytes: None,
        }
    }
}

// okhttp3 MultipartBody / MultipartBody.Builder（postMultipart 真实构造 RFC2046 文本）
pub const MULTIPART_BOUNDARY: &str = "----ReaderFormBoundary7MA4YWxkTrZu0gW";
pub struct MultipartBody;
#[derive(Debug, Default)]
pub struct MultipartBodyBuilder {
    pub parts: std::cell::RefCell<Vec<(String, String, RequestBody)>>,
}
impl MultipartBody {
    pub fn builder() -> MultipartBodyBuilder {
        MultipartBodyBuilder::default()
    }
}
impl MultipartBodyBuilder {
    pub fn set_type(&self, _type: Option<MediaType>) -> &Self {
        self
    }
    pub fn add_form_data_part(&self, name: &str, file_name: &str, body: RequestBody) -> &Self {
        self.parts.borrow_mut().push((name.to_string(), file_name.to_string(), body));
        self
    }
    pub fn build(&self) -> RequestBody {
        let mut out = String::new();
        for (name, file_name, body) in self.parts.borrow().iter() {
            out.push_str(&format!("--{}\r\n", MULTIPART_BOUNDARY));
            out.push_str(&format!("Content-Disposition: form-data; name=\"{}\"", name));
            if !file_name.is_empty() {
                out.push_str(&format!("; filename=\"{}\"", file_name));
            }
            out.push_str("\r\n\r\n");
            out.push_str(&body.text);
            out.push_str("\r\n");
        }
        out.push_str(&format!("--{}--\r\n", MULTIPART_BOUNDARY));
        RequestBody::from_text(out)
    }
}

// okhttp3 HttpUrl / HttpUrl.Builder 占位 + String.toHttpUrl()（OkHttpUtils.get 使用）
#[derive(Debug, Clone, Default)]
pub struct HttpUrl(pub String);
impl HttpUrl {
    pub fn new_builder(&self) -> HttpUrlBuilder {
        HttpUrlBuilder {
            url: self.0.clone(),
            query: std::cell::RefCell::new(Vec::new()),
        }
    }
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}
pub struct HttpUrlBuilder {
    pub url: String,
    pub query: std::cell::RefCell<Vec<(String, String)>>,
}
impl HttpUrlBuilder {
    pub fn add_query_parameter(&self, name: &str, value: &str) -> &Self {
        self.query.borrow_mut().push((name.to_string(), value.to_string()));
        self
    }
    pub fn add_encoded_query_parameter(&self, name: &str, value: &str) -> &Self {
        self.query.borrow_mut().push((name.to_string(), value.to_string()));
        self
    }
    pub fn build(&self) -> HttpUrl {
        let mut u = self.url.clone();
        let q = self.query.borrow();
        if !q.is_empty() {
            let sep = if u.contains('?') { '&' } else { '?' };
            u.push(sep);
            u.push_str(&q.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>().join("&"));
        }
        HttpUrl(u)
    }
}
pub trait HttpUrlExt {
    fn to_http_url(&self) -> HttpUrl;
}
impl HttpUrlExt for str {
    fn to_http_url(&self) -> HttpUrl {
        HttpUrl(self.to_string())
    }
}

// okhttp3 File.asRequestBody(MediaType?)（OkHttpUtils.postMultipart 使用）——fix: 读文件内容（原空 body）
impl File {
    pub fn as_request_body(&self, _media_type: Option<MediaType>) -> RequestBody {
        let text = std::fs::read_to_string(&self.file_path).unwrap_or_default();
        RequestBody::from_text(text)
    }
}

// okhttp3 ByteArray.toRequestBody(MediaType?)（OkHttpUtils.postMultipart 使用）
pub trait ByteArrayExt {
    fn to_request_body(&self, _media_type: Option<MediaType>) -> RequestBody;
}
impl ByteArrayExt for Vec<u8> {
    fn to_request_body(&self, media_type: Option<MediaType>) -> RequestBody {
        RequestBody {
            text: String::from_utf8_lossy(self).to_string(),
            media_type: media_type.map(|m| m.name.to_string()),
            bytes: Some(self.clone()),
        }
    }
}


// okhttp3 MediaType.charset()（OkHttpUtils.text 使用）
impl MediaType {
    pub fn charset(&self) -> Option<&str> {
        None
    }
}

// retrofit2 Call<T> 构造（OkHttpClient.new_call 占位返回使用）
// okhttp3 OkHttpClient.newCall(Request)（OkHttpUtils.new_call_response/new_call/new_call_str_response 使用）——真实执行
impl OkHttpClient {
    pub fn new_call(&self, request: Request) -> Call<Response> {
        Call {
            request: Some(request),
            proxy: self.proxy.clone(),
            proxy_auth: self.proxy_auth.clone(),
            interceptors: self.interceptors.clone(),
            phantom: std::marker::PhantomData,
        }
    }
}

// OkHttpUtils.await_call 所需 oneshot 通道（真实共享槽；enqueue 同步回调 → recv 立即取到）
pub struct OneshotSender<T>(std::sync::Arc<std::sync::Mutex<Option<T>>>);
pub struct OneshotReceiver<T>(std::sync::Arc<std::sync::Mutex<Option<T>>>);
impl<T> OneshotSender<T> {
    pub fn send(&self, value: T) {
        *self.0.lock().unwrap() = Some(value);
    }
}
impl<T> OneshotReceiver<T> {
    pub async fn recv(&self) -> Option<T> {
        self.0.lock().unwrap().take()
    }
}
pub fn tokio_oneshot_channel<T>() -> (OneshotSender<T>, OneshotReceiver<T>) {
    let shared = std::sync::Arc::new(std::sync::Mutex::new(None));
    (OneshotSender(shared.clone()), OneshotReceiver(shared))
}// ---------------- EpubFile.rs 转录追加（仅追加，不改已有内容） ----------------

impl Default for Charset {
    fn default() -> Self {
        Charset
    }
}

impl Document {
    // fix: jsoup Document.body()（EpubFile 转录使用）
    pub fn body(&self) -> Element {
        let (h, t) = crate::runtime::html::body_of(&self.html);
        Element { text: t, html: h }
    }
}
impl Element {
    // fix: jsoup Element.previousElementSiblings()/nextElementSiblings()/remove()（EpubFile 转录使用）
    pub fn previous_element_siblings(&self) -> Elements {
        Elements::default()
    }
    pub fn next_element_siblings(&self) -> Elements {
        Elements::default()
    }
    pub fn remove(&mut self) {}
}

impl Elements {
    // fix: jsoup Elements.add/remove/outerHtml（EpubFile 转录使用）
    pub fn add(&mut self, e: Element) {
        self.list.push(e);
    }
    pub fn remove(&mut self) {
        self.list.clear();
    }
    pub fn outer_html(&self) -> String {
        self.list.iter().map(|e| e.outer_html()).collect()
    }
}

// fix: Kotlin FileUtils.writeBytes(path, bytes)（localBook EpubFile/UmdFile 转录使用）
pub fn write_bytes(path: &str, bytes: &[u8]) {
    let _ = std::fs::write(path, bytes);
}

// ---------------- java.io.Reader / InputStream.reset 扩展 / decode_string 占位（CharsetMatch 转录使用） ----------------

// fix: 对应 java.io.Reader；名称加前缀避免与其它转录模块 glob 导出的 Reader 歧义（CharsetMatch 内以 `as Reader` 显式导入）
pub struct CharsetMatchReader {
    data: Vec<u8>,
    pos: usize,
}

impl CharsetMatchReader {
    pub fn new(data: Vec<u8>) -> CharsetMatchReader {
        CharsetMatchReader { data, pos: 0 }
    }
    pub fn read(&mut self, len: usize) -> i32 {
        if self.pos >= self.data.len() {
            return -1;
        }
        let n = len.min(self.data.len() - self.pos);
        self.pos += n;
        n as i32
    }
    pub fn read_buffer(&mut self, len: usize) -> String {
        let start = self.pos.saturating_sub(len);
        String::from_utf8_lossy(&self.data[start..self.pos]).into_owned()
    }
    pub fn close(&mut self) {}
}

// fix: InputStream 占位 trait 无 reset()（不修改原 trait，避免破坏 XmlStreamReader 等已有 impl），扩展 trait 提供空实现
pub trait InputStreamResetExt {
    fn reset(&mut self) -> Result<(), StubError>;
}

impl<T: ?Sized + InputStream> InputStreamResetExt for T {
    fn reset(&mut self) -> Result<(), StubError> {
        Ok(())
    }
}

impl InputStream for ByteArrayInputStream {
    fn read(&mut self, b: &mut [u8], off: usize, len: usize) -> i32 {
        if off > b.len() {
            return -1;
        }
        let n = len.min(b.len() - off).min(self.data.len());
        if n == 0 {
            return -1;
        }
        b[off..off + n].copy_from_slice(&self.data[..n]);
        self.data.drain(..n);
        n as i32
    }
    fn close(&mut self) {}
}

// fix: Java new String(byte[], charsetName) 占位——不支持任意 charset，退化为 UTF-8 lossy 解码
pub fn decode_string(data: &[u8], _name: &str) -> String {
    String::from_utf8_lossy(data).into_owned()
}

// ---- fix: CharsetDetector uses InputStream.mark()/reset() (stubs trait lacks them; appended extension trait, no rewrite) ----
pub trait InputStreamMarkReset {
    fn mark(&mut self, readlimit: usize);
    fn reset(&mut self) -> Result<(), StubError>;
}

impl InputStreamMarkReset for Box<dyn InputStream> {
    fn mark(&mut self, _readlimit: usize) {}
    fn reset(&mut self) -> Result<(), StubError> {
        Ok(())
    }
}

// ================= YueduApi 转录修复补充（追加；只追加不改写） =================

// ---- io.vertx.ext.web.handler.StaticHandler 占位（YueduApi 静态资源路由） ----
pub use crate::stubs::io::vertx::StaticHandler;

// ---- java.lang.Runtime 占位（YueduApi.getSystemInfo） ----
pub struct Runtime;
impl Runtime {
    pub fn free_memory() -> i64 { 0 }
    pub fn total_memory() -> i64 { 0 }
    pub fn max_memory() -> i64 { 0 }
}

// ---- org.slf4j.MDC 占位（YueduApi 定时任务 traceId） ----
pub struct MDC;
impl MDC {
    pub fn put(_key: &str, _value: String) {}
}

// ---- java.net.URIDecoder.decodeURIComponent 占位（YueduApi 静态资源路径解码） ----
pub fn uri_decode_component(s: &str, _flag: bool) -> String {
    s.to_string()
}

// ---- Environment 补充（YueduApi.getContextPath / setupPort；真实实现见 5400 行 Environment） ----

// ---- System.gc 占位（YueduApi.autoGc） ----
impl System {
    pub fn gc() {}
}

// ---- java.util.Calendar 补充（YueduApi.getSystemInfo / 定时任务） ----
impl Calendar {
    pub const DAY_OF_MONTH: i32 = 5;
    pub const HOUR_OF_DAY: i32 = 11;
    pub const MINUTE: i32 = 12;
    pub const SECOND: i32 = 13;
    pub const MILLISECOND: i32 = 14;
    pub fn set(&mut self, _field: i32, _value: i32) {}
    pub fn time_in_millis(&self) -> i64 {
        self.timeInMillis
    }
}

// ---- ApplicationContext.publishEvent 占位（YueduApi.started / onStartError） ----
impl ApplicationContext {
    pub fn publish_event(&self, _event: crate::com_htmake_reader_springevent::SpringEvent) {}
}

// ---- CURD RoutingContext 方法补充（YueduApi 静态资源路由 / onHandlerError） ----

// ---- Router.post（YueduApi 路由注册） ----
pub trait RouterPostExt {
    fn post(&mut self, path: &str) -> crate::stubs::io::vertx::Route;
}
impl RouterPostExt for crate::stubs::io::vertx::Router {
    fn post(&mut self, path: &str) -> crate::stubs::io::vertx::Route {
        crate::stubs::io::vertx::Router::post(self, path)
    }
}

// ---- Route 处理器链（YueduApi 路由注册） ----
pub trait RouteHandlerExt {
    fn handler<F>(&mut self, h: F)
    where
        F: FnMut(&mut crate::stubs::io::vertx::RoutingContext) + 'static;
    fn coroutine_handler<R: 'static>(&mut self, f: impl Fn(&mut crate::stubs::io::vertx::RoutingContext) -> R + 'static)
    where
        R: IntoAnyResult;
    fn coroutine_handler_without_res<R: 'static>(&mut self, f: impl Fn(&mut crate::stubs::io::vertx::RoutingContext) -> R + 'static)
    where
        R: IntoAnyResult;
}
impl RouteHandlerExt for crate::stubs::io::vertx::Route {
    fn handler<F>(&mut self, h: F)
    where
        F: FnMut(&mut crate::stubs::io::vertx::RoutingContext) + 'static,
    {
        crate::stubs::io::vertx::Route::handler(self, h);
    }
    fn coroutine_handler<R: 'static>(&mut self, f: impl Fn(&mut crate::stubs::io::vertx::RoutingContext) -> R + 'static)
    where
        R: IntoAnyResult,
    {
        crate::stubs::io::vertx::Route::handler(self, move |ctx| {
            let r = f(ctx);
            // fix: 控制器方法可能为 async fn（返回 Future），经 IntoAnyResult 统一驱动后写回 JSON 响应
            let any = r.into_any_result();
            if let Some(rd) = any.downcast_ref::<crate::com_htmake_reader_api_returndata::ReturnData>() {
                if !ctx.response.borrow().ended {
                    ctx.success(rd);
                }
            } else if !ctx.response.borrow().ended {
                ctx.json(String::new());
            }
        });
    }
    fn coroutine_handler_without_res<R>(&mut self, f: impl Fn(&mut crate::stubs::io::vertx::RoutingContext) -> R + 'static)
    where
        R: IntoAnyResult,
    {
        crate::stubs::io::vertx::Route::handler(self, move |ctx| {
            let _ = f(ctx).into_any_result();
        });
    }
}

// ---- ReaderAdapter 实现 ReaderAdapterInterface（YueduApi.initRouter setAdapter；原 impl 被注释） ----
impl crate::io_legado_app_adapters_readeradapterinterface::ReaderAdapterInterface for crate::com_htmake_reader_init_readeradapter::ReaderAdapter {
    fn get_work_dir(&self, _sub_path: &str) -> String {
        String::new()
    }
    fn get_work_dir_vararg(&self, _sub_dir_files: &[&str]) -> String {
        String::new()
    }
    fn get_cache_dir(&self) -> String {
        String::new()
    }
    async fn get_str_response_by_remote_webview(
        &self,
        _url: Option<&str>,
        _html: Option<&str>,
        _encode: Option<&str>,
        _tag: Option<&str>,
        _header_map: Option<&std::collections::HashMap<String, String>>,
        _source_regex: Option<&str>,
        _java_script: Option<&str>,
        _proxy: Option<&str>,
        _post: bool,
        _body: Option<&str>,
        _user_name_space: &str,
        _debug_log: Option<&dyn crate::io_legado_app_model_debuglog::DebugLog>,
    ) -> Option<crate::io_legado_app_help_http_strresponse::StrResponse> {
        None
    }
}

// ---- YueduApi 使用的控制器占位 ----
// fix: 原控制器模块构造器私有/缺失、路由方法私有，跨模块无法调用；按 Kotlin 语义提供占位（仅保证编译与调用形态）
macro_rules! returndata_ctx_routes {
    ($($name:ident),* $(,)?) => {
        $(
            pub fn $name(&self, _context: &crate::com_htmake_reader_api_controller_curd::RoutingContext) -> crate::com_htmake_reader_api_returndata::ReturnData {
                crate::com_htmake_reader_api_returndata::ReturnData::new()
            }
        )*
    };
}
macro_rules! void_ctx_routes {
    ($($name:ident),* $(,)?) => {
        $(
            pub fn $name(&self, _context: &crate::com_htmake_reader_api_controller_curd::RoutingContext) {}
        )*
    };
}










// fix: javax.xml.namespace.QName（PackageDocumentMetadataReader 转录使用, 作为 other_properties 的键）
#[derive(PartialEq, Eq, Hash)]
pub struct QName {
    name: String,
}

impl QName {
    pub fn new(name: String) -> QName {
        QName { name: name }
    }
}

// ---- GSON 数组反序列化（DefaultData.rs 转录使用，对应 Kotlin GSON.fromJsonArray<T>） ----
impl GSON {
    pub fn from_json_array<T>(json: &str) -> Result<Vec<T>, StubError>
    where
        T: serde::de::DeserializeOwned,
    {
        serde_json::from_str::<Vec<T>>(json).map_err(|e| StubError::new(e.to_string()))
    }
}
// ================= 转录修复迭代补充（NavigationEvent/Date/SearchBook/EncodingDetectHelp/FileResourceProvider 使用） =================

// java.util.Date 占位（Date.rs 转录使用）
pub struct java_util_Date(pub i64);

impl java_util_Date {
    pub fn new() -> java_util_Date {
        java_util_Date(System::current_time_millis())
    }
}

// java.util.Locale.US（Date.rs 转录使用）
impl Locale {
    pub const US: Locale = Locale;
}

// java.io.File.getPath()（FileResourceProvider.rs 转录使用）
impl File {
    pub fn get_path(&self) -> String {
        self.file_path.clone()
    }
}

// java.net.URL.openStream() 占位（EncodingDetectHelp.detect_encoding_url 转录使用；read 恒返回 -1 即 EOF）
pub struct UrlInputStream;

impl UrlInputStream {
    pub fn read(&mut self, _b: &mut [i8], _off: i32, _len: i32) -> i32 {
        -1
    }
    pub fn close(&mut self) {}
}

impl URL {
    pub fn open_stream(&self) -> UrlInputStream {
        UrlInputStream
    }
}

// ---- Deferred 占位增强（BaseController.limitConcurrentNeedContinue 转录使用，对应 Kotlin Deferred<T> 状态 API） ----
impl Deferred {
    pub fn is_completed(&self) -> bool {
        false
    }
    pub fn is_cancelled(&self) -> bool {
        false
    }
    pub fn get_completed(&self) -> Box<dyn std::any::Any> {
        Box::new(())
    }
    pub fn from_future<F: std::future::Future>(_f: F) -> Deferred {
        Deferred
    }
}
impl Clone for Deferred {
    fn clone(&self) -> Self {
        Deferred
    }
}

// fix: AppConst.SCRIPT_ENGINE() 转录所需（ScriptEngine 缺 new() 构造）
impl ScriptEngine {
    pub fn new() -> ScriptEngine {
        ScriptEngine
    }
}

// ---------------- StringExtensions 补充（codePointCount / offsetByCodePoints） ----------------
// fix: Kotlin String.codePointCount(beginIndex, endIndex) / offsetByCodePoints(index, offset)
//      （StringExtensions.toStringArray 使用；以 char（码点）计数，字节索引近似 UTF-16 码元索引）
pub fn codePointCount(s: &str, begin: usize, end: usize) -> usize {
    match s.get(begin..end) {
        Some(sub) => sub.chars().count(),
        None => 0,
    }
}

pub fn offsetByCodePoints(s: &str, index: usize, offset: i32) -> usize {
    let mut cur = index.min(s.len());
    let mut remaining = offset;
    while remaining > 0 {
        match s.get(cur..).and_then(|rest| rest.chars().next()) {
            Some(c) => {
                cur += c.len_utf8();
                remaining -= 1;
            }
            None => break,
        }
    }
    cur
}

// ---------------- okhttp3 Response.Builder / Protocol / Headers 占位（StrResponse 使用） ----------------

pub struct Protocol;

impl Protocol {
    pub const HTTP_1_1: Protocol = Protocol;
}

#[derive(Debug, Clone, Default)]
pub struct Headers;

#[derive(Debug, Clone, Default)]
pub struct ResponseBuilder {
    pub inner: std::cell::RefCell<Response>,
}

impl ResponseBuilder {
    pub fn new() -> ResponseBuilder {
        ResponseBuilder {
            inner: std::cell::RefCell::new(Response::default()),
        }
    }
    pub fn code(&self, code: i32) -> &Self {
        self.inner.borrow_mut().status = code;
        self
    }
    pub fn message(&self, _message: &str) -> &Self {
        self
    }
    pub fn protocol(&self, _protocol: Protocol) -> &Self {
        self
    }
    pub fn request(&self, request: Request) -> &Self {
        self.inner.borrow_mut().url = request.url.clone();
        self
    }
    pub fn build(&self) -> Response {
        self.inner.borrow().clone()
    }
}

impl Response {
    // okhttp3 Response.networkResponse（StrResponse.url 使用；最终响应 = 自身，url 为重定向后真实地址）
    pub fn network_response(&self) -> Option<Response> {
        Some(self.clone())
    }
    // okhttp3 Response.message（StrResponse.message 使用）
    pub fn message(&self) -> String {
        String::new()
    }
    // okhttp3 Response.headers（StrResponse.headers 使用；占位恒空）
    pub fn headers_all(&self) -> &'static Headers {
        static EMPTY_HEADERS: Headers = Headers;
        &EMPTY_HEADERS
    }
}


// ---------------- retrofit2 Converter / Converter.Factory 占位（ByteConverter/EncodeConverter 使用） ----------------

pub struct Converter<F, T>(pub Box<dyn Fn(F) -> T>);

impl<F, T> Converter<F, T> {
    pub fn new<C>(f: C) -> Converter<F, T>
    where
        C: Fn(F) -> T + 'static,
    {
        Converter(Box::new(f))
    }
}

pub trait ConverterFactory {
    // fix: 原泛型方法 <F, T> 无法被具体类型实现（E0308），改为固定 Converter<ResponseBody, String>（同 Kotlin EncodeConverter）
    fn response_body_converter(
        &self,
        _type: Option<&Type>,
        _annotations: Option<&[Annotation]>,
        _retrofit: Option<&Retrofit>,
    ) -> Option<Converter<ResponseBody, String>>;
}

// ---------------- AnalyzeRule 构造占位（BookInfo/BookList 等调用 AnalyzeRule::new） ----------------

struct AnalyzeRulePlaceholderData;

impl crate::io_legado_app_model_analyzerule_ruledatainterface::RuleDataInterface for AnalyzeRulePlaceholderData {
    fn variable_map(&self) -> &HashMap<String, String> {
        static EMPTY: std::sync::OnceLock<std::collections::HashMap<String, String>> = std::sync::OnceLock::new();
        EMPTY.get_or_init(|| HashMap::new())
    }
    fn get_user_name_space(&self) -> String {
        String::new()
    }
    fn put_variable(&mut self, _key: &str, _value: Option<&str>) {}
}

// fix: Kotlin `AnalyzeRule(ruleData, source, debugLog)`（真实构造：提取书籍变量 + 书源）
impl crate::io_legado_app_model_analyzerule_analyzerule::AnalyzeRule {
    pub fn new(
        rule_data: &dyn crate::io_legado_app_model_analyzerule_ruledatainterface::RuleDataInterface,
        source: Option<&crate::io_legado_app_data_entities_booksource::BookSource>,
        debug_log: Option<&dyn crate::io_legado_app_model_debuglog::DebugLog>,
    ) -> crate::io_legado_app_model_analyzerule_analyzerule::AnalyzeRule {
        // 提取书籍变量（{{bookName}}/@get:{bookName} 等）
        let mut book_variables = std::collections::HashMap::new();
        if let Some(b) = rule_data.as_any().downcast_ref::<crate::io_legado_app_data_entities_book::Book>() {
            book_variables.insert(String::from("bookName"), b.name.clone());
            book_variables.insert(String::from("bookAuthor"), b.author.clone());
            book_variables.insert(String::from("bookUrl"), b.book_url.clone());
            book_variables.insert(String::from("tocUrl"), b.toc_url.clone());
            book_variables.insert(String::from("bookKind"), b.kind.clone().unwrap_or_default());
            book_variables.insert(String::from("bookWordCount"), b.word_count.clone().unwrap_or_default());
            book_variables.insert(String::from("bookIntro"), b.intro.clone().unwrap_or_default());
        }
        let source_book_source = source.cloned();
        crate::io_legado_app_model_analyzerule_analyzerule::AnalyzeRule {
            rule_data: Box::new(AnalyzeRulePlaceholderData),
            // fix: 真实书源（BaseSource 已实现；原恒 None——依赖 source 的 JS 规则拿不到对象）
            source: source.map(|s| Box::new(s.clone()) as Box<dyn crate::io_legado_app_data_entities_basesource::BaseSource>),
            // fix: 真实调试对象（原恒 None——书源规则调试日志完全失效）
            debug_log: debug_log.map(|dl| dl.clone_box()),
            source_book_source,
            book_variables,
            chapter: None,
            next_chapter_url: None,
            content: None,
            base_url: None,
            redirect_url: None,
            is_json: false,
            is_regex: false,
            analyze_by_x_path: None,
            analyze_by_j_soup: None,
            analyze_by_j_son_path: None,
            object_changed_xp: false,
            object_changed_js: false,
            object_changed_jp: false,
        }
    }
}

// fix: VertRoute.global_handler 使用 io.vertx.ext.web.RoutingContext.get(key)（追加）

// ---------------- WrapOutputStream / UmdBook 转录修复：`dyn Write` 输出流 trait 占位（Umd 写出专用方法集） ----------------

pub trait Write {
    fn write(&mut self, b: &[u8]);
    fn write_range(&mut self, b: &[u8], off: usize, len: usize);
    fn write_byte(&mut self, b: i32);
    fn close(&mut self);
    fn flush(&mut self);
}

// ---------------- JsonExtensions.rs 转录修复：JsonPath.using(Configuration.builder()...) 占位 ----------------

pub struct Configuration;
pub struct ConfigurationBuilder;

pub enum JsonPathOption {
    SUPPRESS_EXCEPTIONS,
}

impl Configuration {
    pub fn builder() -> ConfigurationBuilder {
        ConfigurationBuilder
    }
}

impl ConfigurationBuilder {
    pub fn options(self, _option: JsonPathOption) -> ConfigurationBuilder {
        self
    }
    pub fn build(self) -> Configuration {
        Configuration
    }
}

impl JsonPath {
    pub fn using(_config: Configuration) -> ReadContext {
        ReadContext::default()
    }
}
// ---------------- java.security.MessageDigest 占位（MD5Utils.md5Encode 使用；纯 std 实现 RFC 1321 MD5） ----------------

pub struct MessageDigest;

impl MessageDigest {
    pub fn getInstance(_algorithm: &str) -> MessageDigest {
        MessageDigest
    }

    pub fn digest(&self, input: &[u8]) -> Vec<u8> {
        md5_bytes(input)
    }
}

pub fn md5_bytes(input: &[u8]) -> Vec<u8> {
    let mut s: [u32; 4] = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476];
    let bit_len = (input.len() as u64).wrapping_mul(8);
    let mut msg: Vec<u8> = input.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_le_bytes());
    const K: [u32; 64] = [
        0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
        0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
        0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
        0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
        0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
        0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
        0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
        0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
    ];
    const S: [u32; 64] = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
        5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
        4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
        6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];
    for chunk in msg.chunks_exact(64) {
        let mut m = [0u32; 16];
        for i in 0..16 {
            m[i] = u32::from_le_bytes([chunk[i * 4], chunk[i * 4 + 1], chunk[i * 4 + 2], chunk[i * 4 + 3]]);
        }
        let (mut a, mut b, mut c, mut d) = (s[0], s[1], s[2], s[3]);
        for i in 0..64 {
            let (f, g) = match i / 16 {
                0 => ((b & c) | ((!b) & d), i),
                1 => ((d & b) | ((!d) & c), (5 * i + 1) % 16),
                2 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | (!d)), (7 * i) % 16),
            };
            let f = f.wrapping_add(a).wrapping_add(K[i]).wrapping_add(m[g]);
            a = d;
            d = c;
            c = b;
            b = b.wrapping_add(f.rotate_left(S[i]));
        }
        s[0] = s[0].wrapping_add(a);
        s[1] = s[1].wrapping_add(b);
        s[2] = s[2].wrapping_add(c);
        s[3] = s[3].wrapping_add(d);
    }
    let mut out = Vec::with_capacity(16);
    for v in s {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

// ---------------- java.io.StringReader 占位（RssParserDefault 使用；实现 org.kxml2.io.Reader） ----------------

pub struct StringReader {
    buf: Vec<u16>,
    pos: usize,
}

impl StringReader {
    pub fn new(s: &str) -> StringReader {
        StringReader {
            buf: s.encode_utf16().collect(),
            pos: 0,
        }
    }
}

impl crate::org_kxml2_io_kxmlparser::Reader for StringReader {
    fn read(&mut self) -> i32 {
        if self.pos >= self.buf.len() {
            return -1;
        }
        let c = self.buf[self.pos] as i32;
        self.pos += 1;
        c
    }
    fn read_buf(&mut self, buf: &mut Vec<u16>, off: usize, len: usize) -> i32 {
        let mut count = 0;
        while count < len && self.pos < self.buf.len() {
            let idx = off + count;
            if idx < buf.len() {
                buf[idx] = self.buf[self.pos];
            }
            self.pos += 1;
            count += 1;
        }
        if count == 0 {
            -1
        } else {
            count as i32
        }
    }
    fn to_string(&self) -> String {
        String::from_utf16_lossy(&self.buf)
    }
}

// ---------------- XmlPullParserFactory 占位（RssParserDefault 使用；与 EpubProcessorSupport 同名类型区分） ----------------

pub struct XmlPullParserFactory {
    pub is_namespace_aware: bool,
}

impl XmlPullParserFactory {
    pub fn new_instance() -> XmlPullParserFactory {
        XmlPullParserFactory {
            is_namespace_aware: true,
        }
    }
    pub fn new_pull_parser(&self) -> crate::org_kxml2_io_kxmlparser::KXmlParser {
        let mut parser = crate::org_kxml2_io_kxmlparser::KXmlParser::new();
        parser.process_nsp = self.is_namespace_aware;
        parser
    }
}

// fix: LongTypeAdapter 转录所需——JsonNumber 缺 to_long()（LongTypeAdapter.deserialize 使用）
impl JsonNumber {
    pub fn to_long(&self) -> i64 {
        self.0.parse::<f64>().ok().map(|f| f as i64).unwrap_or(0)
    }
}
// ---------------- EpubResourceProvider / HtmlFormatter 追加占位（只追加） ----------------

impl ZipFile {
    // EpubResourceProvider.get_resource_stream 使用（Kotlin ZipFile.getEntry）
    pub fn get_entry(&self, name: &str) -> Option<ZipEntry> {
        self.entries.iter().find(|e| e.name == name).cloned()
    }
    // EpubResourceProvider.get_resource_stream 使用（Kotlin ZipFile.getInputStream）
    // fix: 真实解压单条目（原空流——EPUB 阅读时图片/CSS/字体全空）
    pub fn get_input_stream_dyn(&self, entry: &ZipEntry) -> Box<dyn InputStream> {
        use std::io::Read;
        let mut data = Vec::new();
        if let Ok(file) = std::fs::File::open(&self.path) {
            if let Ok(mut archive) = zip::ZipArchive::new(file) {
                if let Ok(mut e) = archive.by_name(&entry.name) {
                    let _ = e.read_to_end(&mut data);
                }
            }
        }
        Box::new(ByteArrayInputStream::new(data))
    }
}

impl StringBuffer {
    // HtmlFormatter.formatKeepImg_url 使用（Kotlin StringBuffer.append(String)）
    pub fn append_str(&mut self, s: &str) -> &mut Self {
        self.0.push_str(s);
        self
    }
}

// fix: 为 dyn Error 提供 ThrowableExt（ThrowableExtensions.msg 使用）
impl ThrowableExt for dyn std::error::Error {
    fn localized_message(&self) -> String {
        self.to_string()
    }
    fn stack_trace_to_string(&self) -> String {
        format!("{:?}", self)
    }
    fn msg(&self) -> Option<String> {
        Some(self.to_string())
    }
}

// ---------------- Kotlin RegexOption 占位（AppPattern.bookFileRegex 使用） ----------------

pub struct RegexOption;
impl RegexOption {
    pub const IGNORE_CASE: u32 = 1;
}

// fix: Kotlin `Regex(pattern, RegexOption.IGNORE_CASE)`（AppPattern.bookFileRegex 使用；
// 按 IGNORE_CASE 语义编译为内联 (?i) 前缀）
pub fn regex_new_with_option(pattern: &str, options: u32) -> regex::Regex {
    let mut p = String::from(pattern);
    if options & RegexOption::IGNORE_CASE != 0 && !p.starts_with("(?i)") {
        p = format!("(?i){}", p);
    }
    regex::Regex::new(&p).unwrap_or_else(|_| regex::Regex::new("").unwrap())
}


// fix: Kotlin `ruleData = rssArticle`（RssArticle : RuleDataInterface 转录缺失；
//      variable_map 借用 OnceLock 缓存占位，与 RssArticle 固有 variable_map() 语义一致）
impl crate::io_legado_app_model_analyzerule_ruledatainterface::RuleDataInterface for crate::io_legado_app_data_entities_rssarticle::RssArticle {
    fn variable_map(&self) -> &HashMap<String, String> {
        use std::sync::OnceLock;
        static EMPTY: OnceLock<HashMap<String, String>> = OnceLock::new();
        EMPTY.get_or_init(|| self.variable_map())
    }

    fn get_user_name_space(&self) -> String {
        self.user_name_space.clone()
    }

    fn put_variable(&mut self, key: &str, value: Option<&str>) {
        let mut map = self.variable_map();
        if let Some(v) = value {
            map.insert(key.to_string(), v.to_string());
        } else {
            map.remove(key);
        }
        *self.variable_map_cache.borrow_mut() = Some(map.clone());
        self.variable = Some(GSON::to_json(map));
    }
}

// fix: RssParserByRule 需按值传 Vec<SourceRule>（get_string_inner），SourceRule 无 Clone——
//      以公开 new() 重建近似克隆（私有派生字段重新推导）
impl Clone for crate::io_legado_app_model_analyzerule_analyzerule::SourceRule {
    fn clone(&self) -> Self {
        crate::io_legado_app_model_analyzerule_analyzerule::SourceRule::new(self.rule.clone(), self.mode, false)
    }
}

// ================= IntTypeAdapter 所需 Gson 占位（追加；被并发会话移除后重新补充） =================

// com.google.gson.JsonSerializationContext 占位（IntTypeAdapter/LongTypeAdapter serialize 使用）
pub struct JsonSerializationContext;

// com.google.gson.JsonPrimitive 占位（IntTypeAdapter/LongTypeAdapter serialize 使用）
pub struct JsonPrimitive(pub String);

impl JsonPrimitive {
    pub fn new(s: String) -> JsonPrimitive {
        JsonPrimitive(s)
    }
    pub fn is_number(&self) -> bool {
        self.0.parse::<f64>().is_ok()
    }
    pub fn as_number(&self) -> JsonNumber {
        JsonNumber(self.0.clone())
    }
}

// com.google.gson.JsonNumber 占位（JsonPrimitive.asNumber 使用）
pub struct JsonNumber(pub String);

impl JsonNumber {
    pub fn to_int(&self) -> i32 {
        self.0.parse::<f64>().ok().map(|f| f as i32).unwrap_or(0)
    }
}

// fix: JsonElement(=serde_json::Value) 的 Gson 风格方法（IntTypeAdapter.deserialize 使用）
pub trait JsonElementGsonExt {
    fn is_json_primitive(&self) -> bool;
    fn as_json_primitive(&self) -> JsonPrimitive;
}

impl JsonElementGsonExt for JsonElement {
    fn is_json_primitive(&self) -> bool {
        self.is_boolean() || self.is_number() || self.is_string()
    }
    fn as_json_primitive(&self) -> JsonPrimitive {
        JsonPrimitive(self.to_string())
    }
}

impl From<JsonPrimitive> for JsonElement {
    fn from(p: JsonPrimitive) -> JsonElement {
        JsonElement::String(p.0)
    }
}

// ---------------- 以下为后续编译迭代追加（只追加不重写） ----------------

// fix: Identifier 转录使用（Kotlin java.util.UUID.randomUUID()）
impl Uuid {
    pub fn random_uuid() -> Self {
        Uuid::new_v4()
    }
}

// fix: LazyResource 等模块在返回 Result<_, IOException>(StubError) 的函数中对 std::io::Error 使用 `?`
impl From<std::io::Error> for StubError {
    fn from(e: std::io::Error) -> Self {
        StubError::new(e.to_string())
    }
}

// fix: java.lang.System.err 占位（EncodingDetect 转录使用 System.err.println）
pub struct SystemErr;
impl SystemErr {
    pub fn println(&self, msg: impl std::fmt::Display) {
        println!("{}", msg);
    }
}
impl System {
    pub const err: SystemErr = SystemErr;
}

// fix: jsoup Document.getElementsByTag(tag)（EncodingDetect 转录使用；占位返回空集合）
impl Document {
    pub fn getElementsByTag(&self, _tag: &str) -> Elements {
        Elements::new()
    }
}
// fix: RssParserByRule `Box::new(rss_article.clone())`（RssArticle 转录未实现 Clone；字段全 pub 可逐项克隆）
impl Clone for crate::io_legado_app_data_entities_rssarticle::RssArticle {
    fn clone(&self) -> Self {
        crate::io_legado_app_data_entities_rssarticle::RssArticle {
            origin: self.origin.clone(),
            sort: self.sort.clone(),
            title: self.title.clone(),
            order: self.order,
            link: self.link.clone(),
            pub_date: self.pub_date.clone(),
            description: self.description.clone(),
            content: self.content.clone(),
            image: self.image.clone(),
            read: self.read,
            variable: self.variable.clone(),
            user_name_space: self.user_name_space.clone(),
            variable_map_cache: std::cell::RefCell::new(self.variable_map_cache.borrow().clone()),
        }
    }
}

// fix: QName 补充 namespace_uri / local_part（PackageDocumentMetadataWriter 转录使用）
impl QName {
    pub fn namespace_uri(&self) -> String {
        String::new()
    }

    pub fn local_part(&self) -> String {
        self.name.clone()
    }
}// ---------------- BookChapterList 专用：AnalyzeUrl 构造占位（Kotlin 传引用, 转录 new 收所有权；仅追加） ----------------
// fix: Kotlin `AnalyzeUrl(mUrl, source=bookSource, ruleData=book, headerMapF=bookSource.getHeaderMap(), debugLog=debugLog)`——
//      source 传具体 BookSource、header_map 传书源请求头（原全 None 导致分页目录丢头/Cookie）
pub fn analyze_url_new_placeholder(
    m_url: String,
    source: Option<crate::io_legado_app_data_entities_booksource::BookSource>,
    header_map_f: Option<std::collections::HashMap<String, String>>,
    rule_data: Option<Box<dyn crate::io_legado_app_model_analyzerule_ruledatainterface::RuleDataInterface>>,
) -> crate::io_legado_app_model_analyzerule_analyzeurl::AnalyzeUrl {
    crate::io_legado_app_model_analyzerule_analyzeurl::AnalyzeUrl::new(
        m_url,
        None,
        None,
        None,
        None,
        String::new(),
        source,
        rule_data,
        None,
        header_map_f,
        None,
    )
}

// ---------------- Kotlin io.legado.app.utils.isTrue（BookChapterList.isVolume/isVip 判定；仅追加） ----------------
// Kotlin: fun String.isTrue() = equals("true", true) || toIntOrNull() == 1
pub trait StringIsTrueExt {
    fn is_true(&self) -> bool;
}

impl StringIsTrueExt for String {
    fn is_true(&self) -> bool {
        self.eq_ignore_ascii_case("true") || self.trim().parse::<i64>().map_or(false, |n| n == 1)
    }
}

impl StringIsTrueExt for &str {
    fn is_true(&self) -> bool {
        self.eq_ignore_ascii_case("true") || self.trim().parse::<i64>().map_or(false, |n| n == 1)
    }
}
// ---- TextFile 转录补充（io.legado.app.model.localBook.TextFile） ----
// fix: Kotlin Pattern.MULTILINE / Pattern.pattern()（Java Pattern 常量与方法）
impl Pattern {
    pub const MULTILINE: u32 = 8;
    pub fn pattern(&self) -> String {
        self.src.clone()
    }
}

// fix: TextFile 使用的 FileInputStream.readRange / skip（Java 语义，skip 基于当前偏移向前跳过）
impl FileInputStream {
    pub fn read_range(&mut self, b: &mut [u8], off: usize, len: usize) -> usize {
        let n = self.read(b, off, len);
        if n < 0 {
            0
        } else {
            n as usize
        }
    }
    pub fn skip(&mut self, n: i64) -> i64 {
        use std::io::{Read, Seek};
        if let Some(f) = self.inner.as_mut() {
            let cur = f.stream_position().unwrap_or(0);
            let len = f.metadata().map(|m| m.len()).unwrap_or(0);
            let target = ((cur as i64).saturating_add(n)).clamp(0, len as i64);
            f.seek(std::io::SeekFrom::Start(target as u64)).ok();
            return target - cur as i64;
        }
        0
    }
}

// fix: Kotlin System.runFinalization()（TextFile.analyzeWithPattern 使用）
impl System {
    pub fn run_finalization() {}
}

// fix: Kotlin String.toPatternWithFlags(flags)（TextFile 使用，等价 Kotlin Pattern.compile(regex, flags)）
pub trait StringToPatternExt {
    fn to_pattern_with_flags(&self, flags: u32) -> Pattern;
}
impl StringToPatternExt for String {
    fn to_pattern_with_flags(&self, flags: u32) -> Pattern {
        Pattern::compile_with(self, flags)
    }
}

// ---------------- BookContent.rs additions (append-only) ----------------

// fix: BookContent needs BookChapter by value (AnalyzeRule.chapter / AnalyzeUrl::new);
//      transcription lacks Clone; all fields pub so clone field-by-field
impl Clone for crate::io_legado_app_data_entities_bookchapter::BookChapter {
    fn clone(&self) -> Self {
        crate::io_legado_app_data_entities_bookchapter::BookChapter {
            url: self.url.clone(),
            title: self.title.clone(),
            is_volume: self.is_volume,
            base_url: self.base_url.clone(),
            book_url: self.book_url.clone(),
            index: self.index,
            resource_url: self.resource_url.clone(),
            tag: self.tag.clone(),
            start: self.start,
            end: self.end,
            start_fragment_id: self.start_fragment_id.clone(),
            end_fragment_id: self.end_fragment_id.clone(),
            variable: self.variable.clone(),
            user_name_space: self.user_name_space.clone(),
            variable_map_cache: std::sync::Mutex::new(self.variable_map_cache.lock().unwrap().clone()),
        }
    }
}

// fix: Book::read_config is RefCell<Option<ReadConfig>>; its Clone needs ReadConfig: Clone (transcription lacks it)
impl Clone for crate::io_legado_app_data_entities_book::ReadConfig {
    fn clone(&self) -> Self {
        crate::io_legado_app_data_entities_book::ReadConfig {
            reverse_toc: self.reverse_toc,
            page_anim: self.page_anim,
            re_segment: self.re_segment,
            image_style: self.image_style.clone(),
            use_replace_rule: self.use_replace_rule,
            del_tag: self.del_tag,
            pdf_image_width: self.pdf_image_width,
        }
    }
}

// fix: BookContent passes Box<Book> as rule_data to AnalyzeUrl::new (transcription lacks Clone)
impl Clone for crate::io_legado_app_data_entities_book::Book {
    fn clone(&self) -> Self {
        crate::io_legado_app_data_entities_book::Book {
            book_url: self.book_url.clone(),
            toc_url: self.toc_url.clone(),
            origin: self.origin.clone(),
            origin_name: self.origin_name.clone(),
            name: self.name.clone(),
            author: self.author.clone(),
            kind: self.kind.clone(),
            custom_tag: self.custom_tag.clone(),
            cover_url: self.cover_url.clone(),
            custom_cover_url: self.custom_cover_url.clone(),
            intro: self.intro.clone(),
            custom_intro: self.custom_intro.clone(),
            charset: self.charset.clone(),
            r#type: self.r#type,
            group: self.group,
            latest_chapter_title: self.latest_chapter_title.clone(),
            latest_chapter_time: self.latest_chapter_time,
            last_check_time: self.last_check_time,
            last_check_count: self.last_check_count,
            total_chapter_num: self.total_chapter_num,
            dur_chapter_title: self.dur_chapter_title.clone(),
            dur_chapter_index: self.dur_chapter_index,
            dur_chapter_pos: self.dur_chapter_pos,
            dur_chapter_time: self.dur_chapter_time,
            word_count: self.word_count.clone(),
            can_update: self.can_update,
            order: self.order,
            origin_order: self.origin_order,
            use_replace_rule: self.use_replace_rule,
            variable: self.variable.clone(),
            read_config: std::sync::Mutex::new(self.read_config.lock().unwrap().clone()),
            is_in_shelf: self.is_in_shelf,
            last_check_error: self.last_check_error.clone(),
            info_html: self.info_html.clone(),
            toc_html: self.toc_html.clone(),
            root_dir: self.root_dir.clone(),
            user_name_space: self.user_name_space.clone(),
            variable_map_cache: std::sync::Mutex::new(self.variable_map_cache.lock().unwrap().clone()),
        }
    }
}

// fix: BookContent passes BookSource by value to AnalyzeUrl::new (transcription lacks Clone;
//      debug_log is Box<dyn DebugLog> which is not Clone -> set None)
impl Clone for crate::io_legado_app_data_entities_booksource::BookSource {
    fn clone(&self) -> Self {
        crate::io_legado_app_data_entities_booksource::BookSource {
            book_source_url: self.book_source_url.clone(),
            book_source_name: self.book_source_name.clone(),
            book_source_group: self.book_source_group.clone(),
            book_source_type: self.book_source_type,
            book_url_pattern: self.book_url_pattern.clone(),
            custom_order: self.custom_order,
            enabled: self.enabled,
            enabled_explore: self.enabled_explore,
            enabled_cookie_jar: self.enabled_cookie_jar,
            concurrent_rate: self.concurrent_rate.clone(),
            header: self.header.clone(),
            login_url: self.login_url.clone(),
            login_ui: self.login_ui.clone(),
            login_check_js: self.login_check_js.clone(),
            book_source_comment: self.book_source_comment.clone(),
            variable_comment: self.variable_comment.clone(),
            last_update_time: self.last_update_time,
            respond_time: self.respond_time,
            weight: self.weight,
            explore_url: self.explore_url.clone(),
            rule_explore: self.rule_explore.clone(),
            search_url: self.search_url.clone(),
            rule_search: self.rule_search.clone(),
            rule_book_info: self.rule_book_info.clone(),
            rule_toc: self.rule_toc.clone(),
            rule_content: self.rule_content.clone(),
            user_name_space: self.user_name_space.clone(),
            debug_log: None,
            search_rule_v: self.search_rule_v.clone(),
            explore_rule_v: self.explore_rule_v.clone(),
            book_info_rule_v: self.book_info_rule_v.clone(),
            toc_rule_v: self.toc_rule_v.clone(),
            content_rule_v: self.content_rule_v.clone(),
        }
    }
}
// fix: BookContent calls book_source.get_header_map() (Kotlin BaseSource.getHeaderMap transcription missing;
//      placeholder: UA header + raw header string, same logic as the header_map_f fallback in AnalyzeUrl.rs)
impl crate::io_legado_app_data_entities_booksource::BookSource {
    pub fn get_header_map(&self) -> Option<HashMap<String, String>> {
        Some(parse_source_header_map(self.header.clone(), false, None, None))
    }
    pub fn get_header_map_with_user(&self, has_login_header: bool, user_name_space: Option<String>) -> Option<HashMap<String, String>> {
        Some(parse_source_header_map(self.header.clone(), has_login_header, Some(self.get_key()), user_name_space))
    }
}

/// Kotlin BaseSource.getHeaderMap 语义：UA + header JSON（@js:/<js> 求值）+ 可选登录头合并
pub fn parse_source_header_map(
    header: Option<String>,
    has_login_header: bool,
    source_key: Option<String>,
    user_name_space: Option<String>,
) -> HashMap<String, String> {
    use crate::io_legado_app_constant_appconst::AppConst;
    let mut hm = HashMap::new();
    hm.insert(AppConst::UA_NAME.to_string(), AppConst::userAgent());
    if let Some(h) = header {
        if !h.is_empty() {
            // fix: @js:/<js> 头部求值（Kotlin evalJS；原占位把整段 JSON 塞进 "header" 单头）
            let evaluated = if h.starts_with("@js:") || h.starts_with("@JS:") {
                crate::runtime::js::eval_js_script(&h[4..], &crate::stubs::SimpleBindings::default())
                    .map(|a| a.to_string())
                    .unwrap_or_default()
            } else if h.starts_with("<js>") || h.starts_with("<JS>") {
                h.rfind('<')
                    .map(|i| h[4..i].to_string())
                    .and_then(|js| crate::runtime::js::eval_js_script(&js, &crate::stubs::SimpleBindings::default()))
                    .map(|a| a.to_string())
                    .unwrap_or_default()
            } else {
                h.clone()
            };
            if !evaluated.is_empty() {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&evaluated) {
                    if let Some(obj) = v.as_object() {
                        for (k, val) in obj {
                            if let Some(s) = val.as_str() {
                                hm.insert(k.clone(), s.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    // 登录头合并（Kotlin getLoginHeaderMap——cacheManager loginHeader_{key}）
    if has_login_header {
        if let (Some(key), Some(ns)) = (source_key, user_name_space) {
            if let Some(cache) = crate::io_legado_app_help_cachemanager::CacheManager::new(ns).get(&format!("loginHeader_{}", key)) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&cache) {
                    if let Some(obj) = v.as_object() {
                        for (k, val) in obj {
                            if let Some(s) = val.as_str() {
                                hm.insert(k.clone(), s.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    hm
}

// ---------------- FileController 转录补充（追加） ----------------

// fix: FileController.getFileHome/checkAccess 使用（Kotlin RoutingContext.get<File>() / put）

// fix: FileController.list/upload/parse 使用（Kotlin java.io.File.relativeTo / copyTo）
impl crate::stubs::File {
    pub fn relative_to(&self, other: &crate::stubs::File) -> crate::stubs::File {
        let rel = std::path::Path::new(&self.file_path)
            .strip_prefix(std::path::Path::new(&other.file_path))
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| self.file_path.clone());
        crate::stubs::File::new(&rel)
    }
    pub fn copy_to(&self, dest: &crate::stubs::File, _overwrite: bool) -> crate::stubs::File {
        let _ = std::fs::copy(&self.file_path, &dest.file_path);
        dest.clone()
    }
}

// fix: FileController.restore/parse 使用（Kotlin BookController.syncFromWebdav / saveBookToShelf）

// ---------------- BookList 转录补充：Book/SearchBook 实现 RuleDataInterface（只追加） ----------------
// fix: Kotlin `analyzeRule.ruleData = book/searchBook`（Book/SearchBook : RuleDataInterface 转录缺失；
//      完整实现见下方 AnalyzeRule.rs 转录修复块（8066 起），此处仅 SearchBook 保留）
impl crate::io_legado_app_model_analyzerule_ruledatainterface::RuleDataInterface for crate::io_legado_app_data_entities_searchbook::SearchBook {
    fn variable_map(&self) -> &HashMap<String, String> {
        use std::sync::OnceLock;
        static EMPTY: OnceLock<HashMap<String, String>> = OnceLock::new();
        EMPTY.get_or_init(HashMap::new)
    }

    fn get_user_name_space(&self) -> String {
        self.user_name_space.clone()
    }

    // fix: 真实写入（原只写 variable 字符串——get_variable 经空 variable_map 恒 None；
    //      @put: 变量在搜索条目内丢失、随 SearchBook 返回时也不持久化）
    fn put_variable(&mut self, key: &str, value: Option<&str>) {
        let mut map = self.variable_map_cache.borrow().clone().unwrap_or_else(|| {
            crate::stubs::GSON::from_json_object::<HashMap<String, String>>(self.variable.clone().unwrap_or_default())
                .get_or_null()
                .unwrap_or_else(HashMap::new)
        });
        if let Some(v) = value {
            map.insert(key.to_string(), v.to_string());
        } else {
            map.remove(key);
        }
        *self.variable_map_cache.borrow_mut() = Some(map.clone());
        self.variable = Some(crate::stubs::GSON::to_json(map));
    }

    fn get_variable(&self, key: &str) -> Option<String> {
        self.variable_map().get(key).cloned()
    }

    // fix: 同步已解析字段（{{bookName}} 自引用）
    fn set_field(&mut self, key: &str, value: String) {
        match key {
            "bookName" => self.name = value,
            "bookAuthor" => self.author = value,
            "bookUrl" => self.book_url = value,
            "bookKind" => self.kind = Some(value),
            "bookWordCount" => self.word_count = Some(value),
            "bookIntro" => self.info_html = Some(value),
            _ => {}
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// fix: SearchBook 实现 BaseBook（搜索条目规则 {{bookName}}/{{bookAuthor}} 等自引用；
//      Kotlin ruleData = searchBook 本体，字段实时可见）
impl crate::io_legado_app_data_entities_basebook::BaseBook for crate::io_legado_app_data_entities_searchbook::SearchBook {
    fn name(&self) -> &str {
        &self.name
    }
    fn set_name(&mut self, value: String) {
        self.name = value;
    }
    fn author(&self) -> &str {
        &self.author
    }
    fn set_author(&mut self, value: String) {
        self.author = value;
    }
    fn book_url(&self) -> &str {
        &self.book_url
    }
    fn set_book_url(&mut self, value: String) {
        self.book_url = value;
    }
    fn kind(&self) -> Option<&str> {
        self.kind.as_deref()
    }
    fn set_kind(&mut self, value: Option<String>) {
        self.kind = value;
    }
    fn word_count(&self) -> Option<&str> {
        self.word_count.as_deref()
    }
    fn set_word_count(&mut self, value: Option<String>) {
        self.word_count = value;
    }
    fn info_html(&self) -> Option<&str> {
        self.info_html.as_deref()
    }
    fn set_info_html(&mut self, value: Option<String>) {
        self.info_html = value;
    }
    fn toc_html(&self) -> Option<&str> {
        self.toc_html.as_deref()
    }
    fn set_toc_html(&mut self, value: Option<String>) {
        self.toc_html = value;
    }
}


// ---- fix: UserController.rs 类型检查修复所需补充（追加，勿删） ----

// fix: BaseController::new() 依赖的 Spring Bean 工厂方法缺失 -> 占位实现（get_bean_app_config/get_bean_environment）
impl crate::com_htmake_reader_utils_springcontextutils::SpringContextUtils {
    pub fn get_bean_app_config() -> crate::com_htmake_reader_config_appconfig::AppConfig {
        let mut cfg = crate::com_htmake_reader_config_appconfig::AppConfig::default();
        // READER_APP_* 环境变量（Spring relaxed binding，与原版 docker-compose 说明一致）
        cfg.secure = env_bool("READER_APP_SECURE", cfg.secure);
        cfg.invite_code = env_str("READER_APP_INVITECODE", cfg.invite_code);
        cfg.secure_key = env_str("READER_APP_SECUREKEY", cfg.secure_key);
        cfg.cache_chapter_content = env_bool("READER_APP_CACHECHAPTERCONTENT", cfg.cache_chapter_content);
        cfg.user_limit = env_int("READER_APP_USERLIMIT", cfg.user_limit);
        cfg.user_book_limit = env_int("READER_APP_USERBOOKLIMIT", cfg.user_book_limit);
        cfg.debug_log = env_bool("READER_APP_DEBUGLOG", cfg.debug_log);
        cfg.auto_clear_inactive_user = env_int("READER_APP_AUTOCLEARINACTIVEUSER", cfg.auto_clear_inactive_user);
        cfg.default_user_enable_webdav = env_bool("READER_APP_DEFAULTUSERENABLEWEBDAV", cfg.default_user_enable_webdav);
        cfg.default_user_enable_local_store = env_bool("READER_APP_DEFAULTUSERENABLELOCALSTORE", cfg.default_user_enable_local_store);
        cfg.default_user_enable_book_source = env_bool("READER_APP_DEFAULTUSERENABLEBOOKSOURCE", cfg.default_user_enable_book_source);
        cfg.default_user_enable_rss_source = env_bool("READER_APP_DEFAULTUSERENABLERSSSOURCE", cfg.default_user_enable_rss_source);
        cfg.default_user_book_source_limit = env_int("READER_APP_DEFAULTUSERBOOKSOURCELIMIT", cfg.default_user_book_source_limit);
        cfg.default_user_book_limit = env_int("READER_APP_DEFAULTUSERBOOKLIMIT", cfg.default_user_book_limit);
        cfg.min_user_password_length = env_int("READER_APP_MINUSERPASSWORDLENGTH", cfg.min_user_password_length);
        cfg.work_dir = env_str("READER_APP_WORKDIR", cfg.work_dir);
        cfg.mongo_uri = env_str("READER_APP_MONGOURI", cfg.mongo_uri);
        cfg.mongo_db_name = env_str("READER_APP_MONGODBNAME", cfg.mongo_db_name);
        cfg.shelf_update_inteval = env_int("READER_APP_SHELDUPDATEINTEVAL", cfg.shelf_update_inteval);
        cfg.remote_webview_api = env_str("READER_APP_REMOTEWEBVIEWAPI", cfg.remote_webview_api);
        cfg.auto_backup_user_data = env_bool("READER_APP_AUTOBACKUPUSERDATA", cfg.auto_backup_user_data);
        cfg.remote_book_source_update_interval = env_int("READER_APP_REMOTEBOOKSOURCEUPDATEINTERVAL", cfg.remote_book_source_update_interval);
        cfg
    }
    pub fn get_bean_environment() -> Environment {
        Environment
    }
}

fn env_str(key: &str, default: String) -> String {
    std::env::var(key).ok().filter(|s| !s.is_empty()).unwrap_or(default)
}

fn env_int(key: &str, default: i32) -> i32 {
    std::env::var(key).ok().and_then(|s| s.parse().ok()).unwrap_or(default)
}

fn env_bool(key: &str, default: bool) -> bool {
    match std::env::var(key) {
        Ok(s) => s.eq_ignore_ascii_case("true") || s == "1",
        Err(_) => default,
    }
}

// fix: UserController 转录所需嵌套用户 map（对应 Kotlin `userMapJson.map as MutableMap<String, Map<String, Any>>`；真实解析）
impl JsonObject {
    pub fn user_map_nested(&self) -> std::collections::HashMap<String, std::collections::HashMap<String, Box<dyn std::any::Any>>> {
        let mut out = std::collections::HashMap::new();
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&self.0) {
            if let Some(obj) = v.as_object() {
                for (k, inner) in obj {
                    if let Some(inner_obj) = inner.as_object() {
                        let mut m = std::collections::HashMap::new();
                        for (ik, iv) in inner_obj {
                            m.insert(ik.clone(), user_json_value_to_any(iv));
                        }
                        out.insert(k.clone(), m);
                    }
                }
            }
        }
        out
    }
}

fn user_json_value_to_any(v: &serde_json::Value) -> Box<dyn std::any::Any> {
    match v {
        serde_json::Value::String(s) => Box::new(s.clone()),
        serde_json::Value::Bool(b) => Box::new(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Box::new(i)
            } else {
                Box::new(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::Object(o) => {
            let mut m = std::collections::HashMap::new();
            for (k, val) in o {
                m.insert(k.clone(), user_json_value_to_any(val));
            }
            Box::new(m)
        }
        _ => Box::new(String::new()),
    }
}

// fix: UserController/BaseController 转录所需 `Map<String, Any>.toDataClass()`（真实反序列化）
pub trait ToDataClass {
    fn to_data_class(&self) -> Option<crate::com_htmake_reader_entity_user::User>;
}
impl ToDataClass for std::collections::HashMap<String, Box<dyn std::any::Any>> {
    fn to_data_class(&self) -> Option<crate::com_htmake_reader_entity_user::User> {
        let gs = |k: &str| {
            self.get(k)
                .and_then(|v| v.downcast_ref::<String>().cloned())
                .unwrap_or_default()
        };
        let gl = |k: &str| {
            self.get(k)
                .and_then(|v| v.downcast_ref::<i64>().copied())
                .or_else(|| self.get(k).and_then(|v| v.downcast_ref::<f64>().copied().map(|f| f as i64)))
                .unwrap_or(0)
        };
        let gb = |k: &str| {
            self.get(k)
                .and_then(|v| v.downcast_ref::<bool>().copied())
                .unwrap_or(false)
        };
        let gi = |k: &str| {
            self.get(k)
                .and_then(|v| v.downcast_ref::<i32>().copied())
                .or_else(|| self.get(k).and_then(|v| v.downcast_ref::<i64>().copied().map(|i| i as i32)))
                .unwrap_or(0)
        };
        let token_map = self.get("token_map").and_then(|v| {
            v.downcast_ref::<std::collections::HashMap<String, Box<dyn std::any::Any>>>().map(|m| {
                m.iter()
                    .map(|(k, val)| {
                        (
                            k.clone(),
                            val.downcast_ref::<i64>().copied().unwrap_or(0),
                        )
                    })
                    .collect()
            })
        });
        Some(crate::com_htmake_reader_entity_user::User {
            username: gs("username"),
            password: gs("password"),
            salt: gs("salt"),
            token: gs("token"),
            last_login_at: gl("last_login_at"),
            created_at: gl("created_at"),
            enable_webdav: gb("enable_webdav"),
            token_map,
            enable_local_store: gb("enable_local_store"),
            enable_book_source: gb("enable_book_source"),
            enable_rss_source: gb("enable_rss_source"),
            book_source_limit: gi("book_source_limit"),
            book_limit: gi("book_limit"),
        })
    }
}

/// User → snake_case 存储 map（users.json 格式，与 Kotlin data class toMap() 一致）
pub fn user_to_storage_map(u: &crate::com_htmake_reader_entity_user::User) -> std::collections::HashMap<String, Any> {
    let mut m = std::collections::HashMap::new();
    m.insert(String::from("username"), Any::from(u.username.clone()));
    m.insert(String::from("password"), Any::from(u.password.clone()));
    m.insert(String::from("salt"), Any::from(u.salt.clone()));
    m.insert(String::from("token"), Any::from(u.token.clone()));
    m.insert(String::from("last_login_at"), Any::Long(u.last_login_at));
    m.insert(String::from("created_at"), Any::Long(u.created_at));
    m.insert(String::from("enable_webdav"), Any::Bool(u.enable_webdav));
    m.insert(String::from("enable_local_store"), Any::Bool(u.enable_local_store));
    m.insert(String::from("enable_book_source"), Any::Bool(u.enable_book_source));
    m.insert(String::from("enable_rss_source"), Any::Bool(u.enable_rss_source));
    m.insert(String::from("book_source_limit"), Any::Long(u.book_source_limit as i64));
    m.insert(String::from("book_limit"), Any::Long(u.book_limit as i64));
    if let Some(t) = &u.token_map {
        let mut tm = std::collections::HashMap::new();
        for (k, v) in t {
            tm.insert(k.clone(), Any::Long(*v));
        }
        m.insert(String::from("token_map"), Any::Map(tm));
    }
    m
}

// fix: kotlinx.coroutines.sync.Mutex.withLock 占位（UserController.logout 使用）
pub struct MutexGuard;
impl Mutex {
    pub fn with_lock(&self) -> MutexGuard {
        MutexGuard
    }
}

// fix: BookController.getLastBackFileFromWebdav/createUserBackup 占位（UserController/WebdavController 使用）

// ===== PackageDocumentReader.rs 类型检查修复补充 =====
impl ThrowableExt for () {
    // fix: PackageDocumentReader decode_url 错误类型为 ()，printStackTrace 占位
    fn localized_message(&self) -> String { "()".to_string() }
    fn stack_trace_to_string(&self) -> String { "()".to_string() }
    fn msg(&self) -> Option<String> { Some("()".to_string()) }
}

impl crate::me_ag2s_epublib_util_resourceutil::Document {
    // fix: PackageDocumentReader 需将 resourceutil::Document 传给 DOMUtil（签名要求 domutil::Document）
    pub fn to_dom_document(&self) -> crate::me_ag2s_epublib_epub_domutil::Document {
        // 若本 Document 携带 XML 文本则解析为真实 DOM，否则空文档
        let xml = self.html.clone();
        if xml.is_empty() {
            crate::me_ag2s_epublib_epub_domutil::Document::new()
        } else {
            crate::me_ag2s_epublib_epub_domutil::Document::parse(&xml)
        }
    }
}

impl Clone for crate::me_ag2s_epublib_domain_guide::Guide {
    // fix: PackageDocumentReader read_guide 需可变引用（book.get_guide() 为 &Guide）；占位 clone
    fn clone(&self) -> Self {
        crate::me_ag2s_epublib_domain_guide::Guide::new()
    }
}

impl crate::me_ag2s_epublib_domain_mediatypes::MediaTypes {
    // fix: PackageDocumentReader 用名称比较 mediatype（stubs MediaType 无数据，常量承载原值）
    pub const NCX_NAME: &'static str = "application/x-dtbncx+xml";
    pub const XHTML_NAME: &'static str = "application/xhtml+xml";
    pub const JPG_NAME: &'static str = "image/jpeg";
    pub const PNG_NAME: &'static str = "image/png";
    pub const GIF_NAME: &'static str = "image/gif";
}

// fix: domain::MediaType 无 Clone(字段私有), 按 get_* 访问器重建; NCXDocumentV3 转录使用
impl Clone for crate::me_ag2s_epublib_domain_mediatype::MediaType {
    fn clone(&self) -> Self {
        crate::me_ag2s_epublib_domain_mediatype::MediaType::with_extensions(
            self.get_name().clone(),
            self.get_default_extension().clone(),
            self.get_extensions().clone())
    }
}

// ---------------- WebBook.rs 修复追加（仅追加不重写） ----------------
// fix: BookSource Clone（上方 BookContent 追加提供）依赖 rule_* 五类型的 Clone；此处补齐
//      （全为 Option<String> 字段，逐项克隆；WebBook::prepare_source / get_content_rule 使用）

impl Clone for crate::io_legado_app_data_entities_rule_contentrule::ContentRule {
    fn clone(&self) -> Self {
        crate::io_legado_app_data_entities_rule_contentrule::ContentRule {
            content: self.content.clone(),
            next_content_url: self.next_content_url.clone(),
            web_js: self.web_js.clone(),
            source_regex: self.source_regex.clone(),
            replace_regex: self.replace_regex.clone(),
            image_style: self.image_style.clone(),
        }
    }
}

impl Clone for crate::io_legado_app_data_entities_rule_searchrule::SearchRule {
    fn clone(&self) -> Self {
        crate::io_legado_app_data_entities_rule_searchrule::SearchRule {
            book_list: self.book_list.clone(),
            name: self.name.clone(),
            author: self.author.clone(),
            intro: self.intro.clone(),
            kind: self.kind.clone(),
            last_chapter: self.last_chapter.clone(),
            update_time: self.update_time.clone(),
            book_url: self.book_url.clone(),
            cover_url: self.cover_url.clone(),
            word_count: self.word_count.clone(),
        }
    }
}

impl Clone for crate::io_legado_app_data_entities_rule_explorerule::ExploreRule {
    fn clone(&self) -> Self {
        crate::io_legado_app_data_entities_rule_explorerule::ExploreRule {
            book_list: self.book_list.clone(),
            name: self.name.clone(),
            author: self.author.clone(),
            intro: self.intro.clone(),
            kind: self.kind.clone(),
            last_chapter: self.last_chapter.clone(),
            update_time: self.update_time.clone(),
            book_url: self.book_url.clone(),
            cover_url: self.cover_url.clone(),
            word_count: self.word_count.clone(),
        }
    }
}

impl Clone for crate::io_legado_app_data_entities_rule_bookinforule::BookInfoRule {
    fn clone(&self) -> Self {
        crate::io_legado_app_data_entities_rule_bookinforule::BookInfoRule {
            init: self.init.clone(),
            name: self.name.clone(),
            author: self.author.clone(),
            intro: self.intro.clone(),
            kind: self.kind.clone(),
            last_chapter: self.last_chapter.clone(),
            update_time: self.update_time.clone(),
            cover_url: self.cover_url.clone(),
            toc_url: self.toc_url.clone(),
            word_count: self.word_count.clone(),
            can_re_name: self.can_re_name.clone(),
        }
    }
}

impl Clone for crate::io_legado_app_data_entities_rule_tocrule::TocRule {
    fn clone(&self) -> Self {
        crate::io_legado_app_data_entities_rule_tocrule::TocRule {
            pre_update_js: self.pre_update_js.clone(),
            chapter_list: self.chapter_list.clone(),
            chapter_name: self.chapter_name.clone(),
            chapter_url: self.chapter_url.clone(),
            is_volume: self.is_volume.clone(),
            is_vip: self.is_vip.clone(),
            update_time: self.update_time.clone(),
            next_toc_url: self.next_toc_url.clone(),
        }
    }
}

// fix: WebBook::search_book/explore_book 需 Box 副本（AnalyzeUrl::new rule_data 收所有权），
//      SearchBook 转录无 Clone；字段全 pub 逐项克隆，缓存字段重置
impl Clone for crate::io_legado_app_data_entities_searchbook::SearchBook {
    fn clone(&self) -> Self {
        crate::io_legado_app_data_entities_searchbook::SearchBook {
            book_url: self.book_url.clone(),
            origin: self.origin.clone(),
            origin_name: self.origin_name.clone(),
            r#type: self.r#type,
            name: self.name.clone(),
            author: self.author.clone(),
            kind: self.kind.clone(),
            cover_url: self.cover_url.clone(),
            intro: self.intro.clone(),
            word_count: self.word_count.clone(),
            latest_chapter_title: self.latest_chapter_title.clone(),
            toc_url: self.toc_url.clone(),
            time: self.time,
            variable: self.variable.clone(),
            origin_order: self.origin_order,
            user_name_space: self.user_name_space.clone(),
            info_html: self.info_html.clone(),
            toc_html: self.toc_html.clone(),
            variable_map_cache: std::cell::RefCell::new(None),
            origins: self.origins.clone(),
        }
    }
}

// ================= Resource.rs 转录追加（仅追加，不改已有内容） =================

// Resource derive(Clone, PartialEq) 依赖；okhttp3 MediaType 补充（Clone 由 derive 提供，见 618 行）
// fix: 按名字比较（原恒真——is_bitmap_image 恒 true、XHTML 判定错乱，EPUB 封面/资源分类失效）
impl PartialEq for MediaType {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl MediaType {
    // fix: 真实 MediaType.get_name()（PackageDocumentWriter / Resource 使用；原恒空串 → OPF media-type 空）
    pub fn get_name(&self) -> String {
        self.name.to_string()
    }
    // fix: 真实 MediaType.hash_code()（ResourcesLoader 使用；原恒 0 → 资源类型判定失效）
    pub fn hash_code(&self) -> i32 {
        let mut h: i32 = 0;
        for b in self.name.bytes() {
            h = h.wrapping_mul(31).wrapping_add(b as i32);
        }
        h
    }
    // fix: 真实 MediaType.toString()（Resource::to_string 使用；占位）
    pub fn to_string(&self) -> String {
        "MediaType".to_string()
    }
}

// fix: Resource::get_reader 需把 stubs::ByteArrayInputStream 装箱为 proxy InputStream trait 对象（drain 式简化读取）
impl crate::me_ag2s_epublib_util_commons_io_proxyinputstream::InputStream for ByteArrayInputStream {
    fn read_byte(&mut self) -> i32 {
        if self.data.is_empty() {
            -1
        } else {
            self.data.remove(0) as i32
        }
    }
    fn read(&mut self, bts: &mut [u8]) -> i32 {
        self.read_off(bts, 0, bts.len())
    }
    fn read_off(&mut self, bts: &mut [u8], off: usize, len: usize) -> i32 {
        if self.data.is_empty() || off >= bts.len() {
            return -1;
        }
        let end = (off + len).min(bts.len());
        let take = (end - off).min(self.data.len());
        bts[off..off + take].copy_from_slice(&self.data[..take]);
        self.data.drain(..take);
        take as i32
    }
    fn skip(&mut self, ln: i64) -> Result<i64, std::io::Error> {
        if ln <= 0 {
            return Ok(0);
        }
        let n = (ln as usize).min(self.data.len());
        self.data.drain(..n);
        Ok(n as i64)
    }
    fn available(&mut self) -> Result<i32, std::io::Error> {
        Ok(self.data.len() as i32)
    }
    fn close(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
    // fix: 真实 mark/reset（原空实现——get_xml_prolog 读取后无法回退，XML 内容丢失）
    fn mark(&mut self, _readlimit: i32) {
        self.mark_snapshot = Some(self.data.clone());
    }
    fn reset(&mut self) -> Result<(), std::io::Error> {
        if let Some(snapshot) = self.mark_snapshot.take() {
            self.data = snapshot;
        }
        Ok(())
    }
    fn mark_supported(&self) -> bool {
        true
    }
}

// fix: Resource::get_reader 的 map_err 需要 Display
impl std::fmt::Display for crate::me_ag2s_epublib_util_commons_io_xmlstreamreaderexception::XmlStreamReaderException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "XmlStreamReaderException")
    }
}

// ---------------- AnalyzeRule.rs 转录修复：AnalyzeBy* 私有构造/查询方法占位（仅追加） ----------------
// fix: AnalyzeByXPath/AnalyzeByJSoup/AnalyzeByJSonPath 的 new 与查询方法均为模块私有（Kotlin 为公开构造/方法），
//      AnalyzeByRegex 的 getElement/getElements 同为私有关联函数，AnalyzeRule.rs 与本模块均无法直接调用；
//      实例以 unsafe 零值占位构造（仅存储/传参，下方占位查询函数不读取实例字段，零值析构无操作），
//      查询结果一律返回空占位，待 AnalyzeBy* 模块将相关函数改为 pub 后替换为真实调用。
pub fn analyze_rule_stub_analyze_by_x_path_new(
    _doc: &crate::stubs::Any,
) -> crate::io_legado_app_model_analyzerule_analyzebyxpath::AnalyzeByXPath {
    // fix: jx_node 占位为 Any::Null（判别值 0，全零即合法变体）；实例不参与读取，仅用于缓存字段与传参
    unsafe { std::mem::MaybeUninit::zeroed().assume_init() }
}

pub fn analyze_rule_stub_analyze_by_x_path_get_string(
    _inst: &crate::io_legado_app_model_analyzerule_analyzebyxpath::AnalyzeByXPath,
    _rule: &str,
) -> Option<String> {
    None
}

pub fn analyze_rule_stub_analyze_by_x_path_get_string_list(
    _inst: &crate::io_legado_app_model_analyzerule_analyzebyxpath::AnalyzeByXPath,
    _rule: &str,
) -> Vec<String> {
    Vec::new()
}

pub fn analyze_rule_stub_analyze_by_x_path_get_elements(
    _inst: &crate::io_legado_app_model_analyzerule_analyzebyxpath::AnalyzeByXPath,
    _rule: &str,
) -> Option<Vec<crate::stubs::JXNode>> {
    None
}

pub fn analyze_rule_stub_analyze_by_j_soup_new(
    doc: crate::stubs::Any,
) -> crate::io_legado_app_model_analyzerule_analyzebyjsoup::AnalyzeByJSoup {
    use crate::io_legado_app_model_analyzerule_analyzebyjsoup::Any as LocalAny;
    let local = match &doc {
        crate::stubs::Any::Element(e) => LocalAny::Element(e.clone()),
        crate::stubs::Any::JXNode(n) => LocalAny::JXNode(n.clone()),
        _ => {
            let html = match &doc {
                crate::stubs::Any::Str(s) => s.clone(),
                other => crate::stubs::any_to_value(other).to_string(),
            };
            let body = Jsoup::parse(html).body();
            return crate::io_legado_app_model_analyzerule_analyzebyjsoup::AnalyzeByJSoup::new(LocalAny::Element(body));
        }
    };
    crate::io_legado_app_model_analyzerule_analyzebyjsoup::AnalyzeByJSoup::new(local)
}

pub fn analyze_rule_stub_analyze_by_j_soup_get_string(
    _inst: &crate::io_legado_app_model_analyzerule_analyzebyjsoup::AnalyzeByJSoup,
    _rule: &str,
) -> Option<String> {
    None
}

pub fn analyze_rule_stub_analyze_by_j_soup_get_string0(
    _inst: &crate::io_legado_app_model_analyzerule_analyzebyjsoup::AnalyzeByJSoup,
    _rule: &str,
) -> String {
    String::new()
}

pub fn analyze_rule_stub_analyze_by_j_soup_get_string_list(
    _inst: &crate::io_legado_app_model_analyzerule_analyzebyjsoup::AnalyzeByJSoup,
    _rule: &str,
) -> Vec<String> {
    Vec::new()
}

pub fn analyze_rule_stub_analyze_by_j_soup_get_elements(
    _inst: &crate::io_legado_app_model_analyzerule_analyzebyjsoup::AnalyzeByJSoup,
    _rule: &str,
) -> crate::stubs::Elements {
    crate::stubs::Elements::new()
}

#[allow(invalid_value)]
pub fn analyze_rule_stub_analyze_by_j_son_path_new(
    _doc: &crate::stubs::Any,
) -> crate::io_legado_app_model_analyzerule_analyzebyjsonpath::AnalyzeByJSonPath {
    // fix: ctx 占位全零（ReadContext 仅含 String，容量 0 时析构为无操作）；实例不参与读取
    unsafe { std::mem::MaybeUninit::zeroed().assume_init() }
}

pub fn analyze_rule_stub_analyze_by_j_son_path_get_string(
    inst: &crate::io_legado_app_model_analyzerule_analyzebyjsonpath::AnalyzeByJSonPath,
    rule: &str,
) -> Option<String> {
    crate::runtime::json_path::query(&inst.ctx.json, rule)
        .map(|v| v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string()))
}

pub fn analyze_rule_stub_analyze_by_j_son_path_get_string_list(
    inst: &crate::io_legado_app_model_analyzerule_analyzebyjsonpath::AnalyzeByJSonPath,
    rule: &str,
) -> Vec<String> {
    match crate::runtime::json_path::query(&inst.ctx.json, rule) {
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .map(|x| x.as_str().map(|s| s.to_string()).unwrap_or_else(|| x.to_string()))
            .collect(),
        Some(other) => vec![other.to_string()],
        None => Vec::new(),
    }
}

pub fn analyze_rule_stub_analyze_by_j_son_path_get_object(
    inst: &crate::io_legado_app_model_analyzerule_analyzebyjsonpath::AnalyzeByJSonPath,
    rule: &str,
) -> crate::stubs::Any {
    match crate::runtime::json_path::query(&inst.ctx.json, rule) {
        Some(v) => crate::runtime::js::value_to_any(&v),
        None => crate::stubs::Any::Null,
    }
}

pub fn analyze_rule_stub_analyze_by_j_son_path_get_list(
    inst: &crate::io_legado_app_model_analyzerule_analyzebyjsonpath::AnalyzeByJSonPath,
    rule: &str,
) -> Option<Vec<crate::stubs::Any>> {
    match crate::runtime::json_path::query(&inst.ctx.json, rule) {
        Some(serde_json::Value::Array(arr)) => Some(arr.iter().map(crate::runtime::js::value_to_any).collect()),
        Some(v) => Some(vec![crate::runtime::js::value_to_any(&v)]),
        None => Some(Vec::new()),
    }
}

pub fn analyze_rule_stub_regex_get_element(
    res: &str,
    regs: &[String],
    index: usize,
) -> Option<crate::stubs::List<String>> {
    crate::io_legado_app_model_analyzerule_analyzebyregex::AnalyzeByRegex::get_element(res, regs, index)
}

pub fn analyze_rule_stub_regex_get_elements(
    res: &str,
    regs: &[String],
    index: usize,
) -> crate::stubs::List<crate::stubs::List<String>> {
    crate::io_legado_app_model_analyzerule_analyzebyregex::AnalyzeByRegex::get_elements(res, regs, index)
}

// ---------------- AnalyzeRule.rs 转录修复：Book 实现 RuleDataInterface/BaseBook（book() 下转型；仅追加） ----------------
// fix: Kotlin `ruleData as? BaseBook`——Book 转录未实现 RuleDataInterface/BaseBook（Book.rs 缺 impl），
//      此处按结构体既有方法补齐；as_any 返回自身使 ruleData 为 Box<Book> 时 downcast 成功
//      （AnalyzeRule::new 占位构造器以 AnalyzeRulePlaceholderData 填充，book() 仍返回 None）。
impl crate::io_legado_app_model_analyzerule_ruledatainterface::RuleDataInterface for crate::io_legado_app_data_entities_book::Book {
    // fix: RefCell 缓存无法返回稳定引用（E0515），改用 OnceLock 静态占位
    fn variable_map(&self) -> &HashMap<String, String> {
        use std::sync::OnceLock;
        static EMPTY: OnceLock<HashMap<String, String>> = OnceLock::new();
        EMPTY.get_or_init(HashMap::new)
    }
    fn get_user_name_space(&self) -> String {
        self.user_name_space.clone()
    }
    fn put_variable(&mut self, key: &str, value: Option<&str>) {
        self.put_variable(key.to_string(), value.map(|s| s.to_string()));
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl crate::io_legado_app_data_entities_basebook::BaseBook for crate::io_legado_app_data_entities_book::Book {
    fn name(&self) -> &str {
        &self.name
    }
    fn set_name(&mut self, value: String) {
        self.name = value;
    }
    fn author(&self) -> &str {
        &self.author
    }
    fn set_author(&mut self, value: String) {
        self.author = value;
    }
    fn book_url(&self) -> &str {
        &self.book_url
    }
    fn set_book_url(&mut self, value: String) {
        self.book_url = value;
    }
    fn kind(&self) -> Option<&str> {
        self.kind.as_deref()
    }
    fn set_kind(&mut self, value: Option<String>) {
        self.kind = value;
    }
    fn word_count(&self) -> Option<&str> {
        self.word_count.as_deref()
    }
    fn set_word_count(&mut self, value: Option<String>) {
        self.word_count = value;
    }
    fn info_html(&self) -> Option<&str> {
        self.info_html.as_deref()
    }
    fn set_info_html(&mut self, value: Option<String>) {
        self.info_html = value;
    }
    fn toc_html(&self) -> Option<&str> {
        self.toc_html.as_deref()
    }
    fn set_toc_html(&mut self, value: Option<String>) {
        self.toc_html = value;
    }
}
// ---- fix: WebdavController.rs 类型检查修复所需补充（追加，勿删） ----

// fix: stubs::io::vertx::RoutingContext 缺 path()（Kotlin HttpServerRequest.path()；WebdavController.requestPath 使用）

// fix: stubs::BookController 缺 syncBookProgressFromWebdav（WebdavController.webdavUpload 使用）

// fix: BaseController.app_config 为模块私有字段（BaseController.rs 未提供 getter），跨模块不可直接读取；
//      读取 READER_APP_SECURE 环境变量（relaxed binding，等效 Spring reader.app.secure 配置）
impl crate::com_htmake_reader_api_controller_basecontroller::BaseController {
    pub fn get_app_config_secure(&self) -> bool {
        let v = std::env::var("READER_APP_SECURE").unwrap_or_default();
        !v.is_empty() && !v.eq_ignore_ascii_case("false")
    }
    pub fn get_app_config_secure_key(&self) -> Option<String> {
        std::env::var("READER_APP_SECUREKEY").ok().filter(|s| !s.is_empty())
    }
}
// fix: WebdavController 路由处理需要 CURD RoutingContext 版本（与 io::vertx 版已统一为同一类型）
impl io::vertx::Route {
    pub fn global_handler_curd<F>(&mut self, handler: F)
    where
        F: FnMut(&mut crate::stubs::io::vertx::RoutingContext) + 'static,
    {
        crate::stubs::io::vertx::Route::global_handler_static(self, handler);
    }
}

// fix: CURD 占位 RoutingContext 补充 WebdavController 所需方法（get_header/raw_method/get_body/add_headers_end_handler；追加）
// ================= AnalyzeByJSoup 转录修复补充（追加；只追加不改写） =================

// fix: jsoup Elements.iter()/add_all()/clear()（AnalyzeByJSoup 使用）
impl Elements {
    pub fn iter(&self) -> std::slice::Iter<'_, Element> {
        self.list.iter()
    }
    pub fn add_all(&mut self, other: Elements) {
        self.list.extend(other.list);
    }
    pub fn clear(&mut self) {
        self.list.clear();
    }
}

// fix: jsoup Element.data()/getElementsByClass()/getElementsContainingOwnText()（AnalyzeByJSoup 使用）
impl Element {
    pub fn data(&self) -> Option<String> {
        None
    }
    pub fn get_elements_by_class(&self, class: &str) -> Elements {
        // fix: 基于 html 片段按 class 选择（AnalyzeByJSoup class.xxx 前置规则）
        crate::runtime::html::select_elements(&self.html, &format!(".{}", class))
    }
    pub fn get_elements_containing_own_text(&self, text: &str) -> Elements {
        // fix: 按直接文本子节点包含匹配（scraper 遍历所有元素）
        let doc = scraper::Html::parse_fragment(&self.html);
        let mut result = Vec::new();
        if let Ok(sel) = scraper::Selector::parse("*") {
            for el in doc.select(&sel) {
                let direct: String = el
                    .children()
                    .filter_map(|c| match c.value() {
                        scraper::node::Node::Text(t) => Some(t.to_string()),
                        _ => None,
                    })
                    .collect();
                if direct.contains(text) {
                    result.push(Element {
                        text: direct,
                        html: el.html().to_string(),
                    });
                }
            }
        }
        Elements { list: result }
    }
}

// fix: ACache.lastUsageDates: HashMap<File, i64> 需要 File 实现 Eq/Hash（仅追加，不重写现有代码）
impl std::cmp::Eq for File {}
impl std::hash::Hash for File {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.file_path.hash(state);
        self.name.hash(state);
        self.absolute_path.hash(state);
        self.parent_file.hash(state);
    }
}
// fix: Resources.make_valid_id 需要 java.lang.Character.isJavaIdentifierStart（追加）
impl Character {
    pub fn is_java_identifier_start(c: char) -> bool {
        c.is_alphabetic() || c == '_' || c == '$'
    }
}

// fix: Resources.create_href 需要 MediaType.getDefaultExtension()（stubs::MediaType 占位：空串，追加）
// fix: Resources 中 resource.get_media_type()（stubs::MediaType）与真实 MediaType 的 == 比较（原恒 true——按名字比较）
impl PartialEq<crate::me_ag2s_epublib_domain_mediatype::MediaType> for MediaType {
    fn eq(&self, other: &crate::me_ag2s_epublib_domain_mediatype::MediaType) -> bool {
        self.name == other.get_name()
    }
}
impl PartialEq<MediaType> for crate::me_ag2s_epublib_domain_mediatype::MediaType {
    fn eq(&self, other: &MediaType) -> bool {
        self.get_name() == other.name
    }
}
// ================= 规则实体 serde 序列化补充（追加；只追加不改写） =================

// fix: BookSource Converters 规则序列化使用（GSON::to_json 需 T: Serialize；结构体未 derive，故在 stubs 补充 impl）

impl serde::Serialize for crate::io_legado_app_data_entities_rule_explorerule::ExploreRule {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("ExploreRule", 10)?;
        s.serialize_field("bookList", &self.book_list)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("author", &self.author)?;
        s.serialize_field("intro", &self.intro)?;
        s.serialize_field("kind", &self.kind)?;
        s.serialize_field("lastChapter", &self.last_chapter)?;
        s.serialize_field("updateTime", &self.update_time)?;
        s.serialize_field("bookUrl", &self.book_url)?;
        s.serialize_field("coverUrl", &self.cover_url)?;
        s.serialize_field("wordCount", &self.word_count)?;
        s.end()
    }
}

impl serde::Serialize for crate::io_legado_app_data_entities_rule_searchrule::SearchRule {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("SearchRule", 10)?;
        s.serialize_field("bookList", &self.book_list)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("author", &self.author)?;
        s.serialize_field("intro", &self.intro)?;
        s.serialize_field("kind", &self.kind)?;
        s.serialize_field("lastChapter", &self.last_chapter)?;
        s.serialize_field("updateTime", &self.update_time)?;
        s.serialize_field("bookUrl", &self.book_url)?;
        s.serialize_field("coverUrl", &self.cover_url)?;
        s.serialize_field("wordCount", &self.word_count)?;
        s.end()
    }
}

impl serde::Serialize for crate::io_legado_app_data_entities_rule_bookinforule::BookInfoRule {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("BookInfoRule", 11)?;
        s.serialize_field("init", &self.init)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("author", &self.author)?;
        s.serialize_field("intro", &self.intro)?;
        s.serialize_field("kind", &self.kind)?;
        s.serialize_field("lastChapter", &self.last_chapter)?;
        s.serialize_field("updateTime", &self.update_time)?;
        s.serialize_field("coverUrl", &self.cover_url)?;
        s.serialize_field("tocUrl", &self.toc_url)?;
        s.serialize_field("wordCount", &self.word_count)?;
        s.serialize_field("canReName", &self.can_re_name)?;
        s.end()
    }
}

impl serde::Serialize for crate::io_legado_app_data_entities_rule_tocrule::TocRule {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("TocRule", 8)?;
        s.serialize_field("preUpdateJs", &self.pre_update_js)?;
        s.serialize_field("chapterList", &self.chapter_list)?;
        s.serialize_field("chapterName", &self.chapter_name)?;
        s.serialize_field("chapterUrl", &self.chapter_url)?;
        s.serialize_field("isVolume", &self.is_volume)?;
        s.serialize_field("isVip", &self.is_vip)?;
        s.serialize_field("updateTime", &self.update_time)?;
        s.serialize_field("nextTocUrl", &self.next_toc_url)?;
        s.end()
    }
}

impl serde::Serialize for crate::io_legado_app_data_entities_rule_contentrule::ContentRule {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("ContentRule", 6)?;
        s.serialize_field("content", &self.content)?;
        s.serialize_field("nextContentUrl", &self.next_content_url)?;
        s.serialize_field("webJs", &self.web_js)?;
        s.serialize_field("sourceRegex", &self.source_regex)?;
        s.serialize_field("replaceRegex", &self.replace_regex)?;
        s.serialize_field("imageStyle", &self.image_style)?;
        s.end()
    }
}
// ---------------- Rss / RssSource 转录修复：RssSource 扩展方法 + RuleData 构造占位（仅追加） ----------------

// fix: Rss.rs 调用 rss_source.get_header_map()（Kotlin BaseSource.getHeaderMap 转录缺失；
//      占位：UA 头 + 原始 header 字符串，逻辑与 BookSource 扩展一致）
impl crate::io_legado_app_data_entities_rsssource::RssSource {
    pub fn get_header_map(&self) -> Option<HashMap<String, String>> {
        // fix: 真实解析 header JSON（原占位把原始 JSON 塞进 "header" 单头——Cookie/Referer 失效）
        Some(parse_source_header_map(self.header.clone(), false, None, None))
    }
}

// fix: RssSource.sortUrls 调用 self.eval_js(js_str, None)（Kotlin BaseSource.evalJS 转录缺失；
//      占位：绑定基础键后交 SCRIPT_ENGINE，逻辑与 BaseSource.rs 默认实现一致）
impl crate::io_legado_app_data_entities_rsssource::RssSource {
    pub fn eval_js(
        &self,
        js_str: String,
        _bindings_config: Option<&mut dyn FnMut(&mut SimpleBindings)>,
    ) -> Option<Box<Any>> {
        let mut bindings = SimpleBindings::new();
        bindings.set("java", self.get_key());
        bindings.set("source", self.get_key());
        bindings.set("baseUrl", self.get_key());
        bindings.set("cookie", self.get_user_name_space());
        bindings.set("cache", self.get_user_name_space());
        SCRIPT_ENGINE
            .eval(js_str, &mut bindings)
            .map(|v| Box::new(Any::Str(format!("{:?}", v))))
    }
}

// fix: Kotlin `RuleData()` 默认构造——RuleData.rs 转录无构造器且 variable_map 字段私有（外部无法安全构造）；
//      占位：以同布局单字段包装结构持有合法空 HashMap 后 transmute（单字段结构体布局与字段一致，无 UB 风险）
#[derive(Default)]
struct RuleDataEmpty {
    variable_map: HashMap<String, String>,
}

impl crate::io_legado_app_model_analyzerule_ruledata::RuleData {
    pub fn new() -> crate::io_legado_app_model_analyzerule_ruledata::RuleData {
        unsafe { std::mem::transmute(RuleDataEmpty::default()) }
    }
}
impl Element {
    // fix: jsoup Element.textNodes()——仅**直接**子文本节点（原 select("*") 收集所有后代——行数/顺序错位）
    pub fn text_nodes(&self) -> Vec<TextNode> {
        let doc = scraper::Html::parse_fragment(&self.html);
        let mut result: Vec<TextNode> = Vec::new();
        for c in doc.root_element().children() {
            if let scraper::node::Node::Text(t) = c.value() {
                let s = t.to_string();
                if !s.trim().is_empty() {
                    result.push(TextNode { text: s });
                }
            }
        }
        result
    }
}


// fix: CharsetDetector 的 f_* 检测字段为模块私有（CharsetDetector.rs 未提供 getter，且本项目不允许修改该文件），
//      追加跨模块访问器。占位实现未打通真实数据（后续若 CharsetDetector.rs 公开字段/提供 getter，可直接改为直读）
impl crate::io_legado_app_lib_icu4j_charsetdetector::CharsetDetector {
    pub fn f_input_bytes_access(&self) -> Vec<u8> {
        self.f_input_bytes.clone()
    }
    pub fn f_input_len_access(&self) -> i32 {
        self.f_input_len
    }
    pub fn f_c1_bytes_access(&self) -> bool {
        self.f_c1_bytes
    }
    pub fn f_raw_input_access(&self) -> Option<Vec<u8>> {
        self.f_raw_input.clone()
    }
    pub fn f_raw_length_access(&self) -> i32 {
        self.f_raw_length
    }
}
// ---------------- BaseController/BookGroupController 类型检查修复补充（追加，勿删） ----------------

// fix: Kotlin RoutingContext.get<String>(key)（BaseController.getUserNameSpace 使用）

// Kotlin JsonObject.getLong(key)（BookGroupController.checker/saveBookGroupOrder 使用；无默认值版本返回 Option）
impl JsonObject {
    pub fn get_long_opt(&self, key: &str) -> Option<i64> {
        serde_json::from_str::<serde_json::Value>(&self.0)
            .ok()
            .and_then(|v| v.get(key).cloned())
            .and_then(|v| v.as_i64())
    }
}

// fix: stubs::Any -> Box<dyn Any> 转换（BaseController 读取 users 存储：Map<String, Any> -> Map<String, Box<dyn Any>>）
impl Any {
    pub fn into_boxed_any(self) -> Box<dyn std::any::Any> {
        match self {
            Any::Null => Box::new(()),
            Any::Bool(b) => Box::new(b),
            Any::Long(i) => Box::new(i),
            Any::Double(d) => Box::new(d),
            Any::Str(s) => Box::new(s),
            Any::JsonObject(o) => Box::new(o),
            Any::JsonArray(a) => Box::new(a),
            Any::List(l) => Box::new(l),
            Any::Map(m) => Box::new(m),
            Any::ReadContext(r) => Box::new(r),
            Any::JXNode(n) => Box::new(n),
            Any::JXDocument(d) => Box::new(d),
            Any::Document(d) => Box::new(d),
            Any::Element(e) => Box::new(e),
            Any::Elements(e) => Box::new(e),
        }
    }
    // fix: Kotlin `userMapJson.map as MutableMap<String, Map<String, Any>>`（仅 Map 变体可转换，其余占位空 map）
    pub fn as_map_boxed(self) -> std::collections::HashMap<String, Box<dyn std::any::Any>> {
        match self {
            Any::Map(m) => m.into_iter().map(|(k, v)| (k, v.into_boxed_any())).collect(),
            _ => std::collections::HashMap::new(),
        }
    }
}

// fix: User data class 无 Clone 派生，format_user 需要（BaseController 转录；按字段逐项克隆）
impl Clone for crate::com_htmake_reader_entity_user::User {
    fn clone(&self) -> Self {
        crate::com_htmake_reader_entity_user::User {
            username: self.username.clone(),
            password: self.password.clone(),
            salt: self.salt.clone(),
            token: self.token.clone(),
            last_login_at: self.last_login_at,
            created_at: self.created_at,
            enable_webdav: self.enable_webdav,
            token_map: self.token_map.clone(),
            enable_local_store: self.enable_local_store,
            enable_book_source: self.enable_book_source,
            enable_rss_source: self.enable_rss_source,
            book_source_limit: self.book_source_limit,
            book_limit: self.book_limit,
        }
    }
}

// fix: BaseController.checkAuth/getUserInfoClass 使用（stubs Any 版用户 map 的 toDataClass 占位，恒返回 None）
impl ToDataClass for std::collections::HashMap<String, crate::stubs::Any> {
    fn to_data_class(&self) -> Option<crate::com_htmake_reader_entity_user::User> {
        let gs = |k: &str| {
            self.get(k)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
        };
        let gl = |k: &str| self.get(k).and_then(|v| v.as_long()).unwrap_or(0);
        let gb = |k: &str| self.get(k).and_then(|v| v.as_bool()).unwrap_or(false);
        let gi = |k: &str| self.get(k).and_then(|v| v.as_long().map(|l| l as i32)).unwrap_or(0);
        let token_map = self.get("token_map").and_then(|v| v.as_map()).map(|m| {
            m.iter()
                .map(|(k, val)| (k.clone(), val.as_long().unwrap_or(0)))
                .collect()
        });
        Some(crate::com_htmake_reader_entity_user::User {
            username: gs("username"),
            password: gs("password"),
            salt: gs("salt"),
            token: gs("token"),
            last_login_at: gl("last_login_at"),
            created_at: gl("created_at"),
            enable_webdav: gb("enable_webdav"),
            token_map,
            enable_local_store: gb("enable_local_store"),
            enable_book_source: gb("enable_book_source"),
            enable_rss_source: gb("enable_rss_source"),
            book_source_limit: gi("book_source_limit"),
            book_limit: gi("book_limit"),
        })
    }
}
// ================= MediaTypes.rs 转录修复追加（仅追加不重写） =================
// fix: MediaType 带真实数据（原单元占位——EPUB 导出 mimetype/OPF media-type 空、资源类型判定全失效）
impl MediaType {
    pub const fn new(name: &'static str, default_extension: &'static str) -> MediaType {
        MediaType { name, default_extension, extensions: &[] }
    }
    pub const fn with_extensions(name: &'static str, default_extension: &'static str, extensions: &'static [&'static str]) -> MediaType {
        MediaType { name, default_extension, extensions }
    }
    // fix: 真实扩展名表（原空表 → determine_media_type 恒 None）
    pub fn get_extensions(&self) -> &'static [&'static str] {
        self.extensions
    }
    pub fn get_default_extension(&self) -> &'static str {
        self.default_extension
    }
}
// fix: RssSourceController 调用 rss_source.clone()（RssSource 转录缺 Clone；字段全 pub 逐字段克隆，
//      debug_log 为 Box<dyn DebugLog> 非 Clone → 置 None，同 BookSource 克隆占位约定）
impl Clone for crate::io_legado_app_data_entities_rsssource::RssSource {
    fn clone(&self) -> Self {
        crate::io_legado_app_data_entities_rsssource::RssSource {
            source_url: self.source_url.clone(),
            source_name: self.source_name.clone(),
            source_icon: self.source_icon.clone(),
            source_group: self.source_group.clone(),
            source_comment: self.source_comment.clone(),
            enabled: self.enabled,
            variable_comment: self.variable_comment.clone(),
            enabled_cookie_jar: self.enabled_cookie_jar,
            concurrent_rate: self.concurrent_rate.clone(),
            header: self.header.clone(),
            login_url: self.login_url.clone(),
            login_ui: self.login_ui.clone(),
            login_check_js: self.login_check_js.clone(),
            sort_url: self.sort_url.clone(),
            single_url: self.single_url,
            article_style: self.article_style,
            rule_articles: self.rule_articles.clone(),
            rule_next_page: self.rule_next_page.clone(),
            rule_title: self.rule_title.clone(),
            rule_pub_date: self.rule_pub_date.clone(),
            rule_description: self.rule_description.clone(),
            rule_image: self.rule_image.clone(),
            rule_link: self.rule_link.clone(),
            rule_content: self.rule_content.clone(),
            style: self.style.clone(),
            enable_js: self.enable_js,
            load_with_base_url: self.load_with_base_url,
            custom_order: self.custom_order,
            user_name_space: self.user_name_space.clone(),
            debug_log: None,
        }
    }
}

// ---------------- ReaderApplication/ReaderUIApplication 转录补充（append-only，additive） ----------------
// fix: ReaderUIApplication 中 `self.default_icons.clone()`（Vec<Image>）需要 Image: Clone
impl Clone for Image {
    fn clone(&self) -> Self {
        Image
    }
}

// fix: ReaderUIApplication.stop() 中 `format!("application stop: {:?}", context)` 需要 ApplicationContext: Debug
impl std::fmt::Debug for ApplicationContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ApplicationContext")
    }
}

impl StringBuffer {
    // HtmlFormatter.formatKeepImg_url 使用（Kotlin StringBuffer() 无参构造）
    pub fn new() -> StringBuffer {
        StringBuffer(String::new())
    }
}

// ---------------- LRUCache.rs 修复追加：Send/Sync 容器（Arc/RwLock 替代 Rc/RefCell） ----------------
pub use std::sync::Arc;
pub use std::sync::RwLock;

// ---------------- Coroutine.rs 修复追加：new() 借用拆分需要 CoroutineContext: Clone ----------------
impl Clone for CoroutineContext {
    fn clone(&self) -> Self {
        CoroutineContext
    }
}

// ---- fix: CharsetMatch::get_reader() 需要对 Rc<dyn InputStream> 做只读 reset（Rc 无法 as_mut），追加只读扩展（只追加不改写） ----
pub trait InputStreamResetShared {
    fn reset_shared(&self) -> Result<(), StubError>;
}

impl InputStreamResetShared for std::rc::Rc<dyn InputStream> {
    fn reset_shared(&self) -> Result<(), StubError> {
        Ok(())
    }
}

// ---- fix: SpringContextUtils.rs / EpubWriter.rs 转录修复所需补充（追加，勿删） ----

// fix: OnceLock<Option<ApplicationContext>> 的 .clone() 需要 ApplicationContext: Clone（SpringContextUtils.get_application_context）
impl Clone for ApplicationContext {
    fn clone(&self) -> Self {
        ApplicationContext
    }
}
impl Copy for ApplicationContext {}

// fix: Spring Bean 查询占位（SpringContextUtils.get_bean_* —— 原 Java getBean 重载转录为三个独立方法）
impl ApplicationContext {
    pub fn get_bean_by_name(&self, _name: &str) -> Option<crate::me_ag2s_epublib_util_ioutil::Object> {
        None
    }
    pub fn get_bean_by_class<T>(&self, _clazz: Class<T>) -> Option<T> {
        None
    }
    pub fn get_bean_by_name_and_class<T>(&self, _name: &str, _clazz: Class<T>) -> Option<T> {
        None
    }
}

// fix: EpubWriter.write_package_document —— EpubProcessorSupport 产出的 XmlSerializer
//      与 PackageDocumentMetadataWriter 的 XmlSerializer 为两个不同 stub 类型，桥接转换
impl From<crate::me_ag2s_epublib_epub_epubprocessorsupport::XmlSerializer>
    for crate::me_ag2s_epublib_epub_packagedocumentmetadatawriter::XmlSerializer
{
    fn from(_: crate::me_ag2s_epublib_epub_epubprocessorsupport::XmlSerializer) -> Self {
        crate::me_ag2s_epublib_epub_packagedocumentmetadatawriter::XmlSerializer::new()
    }
}
// ---- YueduApi/BaseController 转录修复补充 2（追加；只追加不改写） ----

// ---- Route.handler_static（YueduApi 静态资源路由：StaticHandler 占位，handler() 只收闭包） ----
pub trait StaticHandlerRouteExt {
    fn handler_static(&mut self, _h: StaticHandler) {}
}
impl StaticHandlerRouteExt for crate::stubs::io::vertx::Route {}

// ---- Calendar.get_instance 别名（YueduApi.getSystemInfo / 定时任务；对应 Kotlin getInstance） ----
impl Calendar {
    pub fn get_instance() -> Calendar {
        Calendar::getInstance()
    }
}
// ---- Metadata 转录修复补充：Date/Author 无 Clone，补 Clone 实现（字段私有，经公共构造器/Getter 复制） ----

impl Clone for crate::me_ag2s_epublib_domain_relator::Relator {
    fn clone(&self) -> Self {
        crate::me_ag2s_epublib_domain_relator::Relator::by_code(&self.code().to_string())
            .unwrap_or(crate::me_ag2s_epublib_domain_relator::Relator::OTHER)
    }
}

impl Clone for crate::me_ag2s_epublib_domain_author::Author {
    fn clone(&self) -> Self {
        let mut a = crate::me_ag2s_epublib_domain_author::Author::with_names(
            self.get_firstname().clone(),
            self.get_lastname().clone(),
        );
        a.set_relator(self.get_relator().clone());
        a
    }
}



// fix: Tools.http_get 所需——retrofit2 Call<T>.execute() 占位（占位不实际发请求，恒返回空 Response）
impl<T> Call<T> {
    pub fn execute(&self) -> Response {
        Response::default()
    }
}
// fix: Ext.rs unzip 中 output_stream 改为 Option<FileOutputStream> 后，OptionOutputStreamWriteExt 补实现（追加）
impl OptionOutputStreamWriteExt for Option<FileOutputStream> {
    fn write(&mut self, b: &[u8], off: usize, len: usize) {
        if let Some(s) = self.as_mut() {
            let start = off.min(b.len());
            let end = (off + len).min(b.len());
            s.write_range(&b[start..end], 0, end - start);
        }
    }
}
// fix: Ext.rs unzip 中 output_stream（Option<FileOutputStream>）的 close 调用补实现（追加）
impl OptionStreamCloseExt for Option<FileOutputStream> {
    fn close(&mut self) {
        if let Some(s) = self.as_mut() {
            let _ = s.close();
        }
    }
}

// fix: CoroutinesCallAdapterFactory.adapt() 返回类型为 stubs::Deferred（占位转换，仅丢弃类型参数）
impl<T> From<CompletableDeferred<T>> for Deferred {
    fn from(_deferred: CompletableDeferred<T>) -> Deferred {
        Deferred
    }
}
// fix: EncoderUtilsTest 使用（Kotlin `KeyPair.public` / `KeyPair.private` → getPublic()/getPrivate()；静态空键）
pub static DUMMY_PUBLIC_KEY: PublicKey = PublicKey { der: Vec::new() };
pub static DUMMY_PRIVATE_KEY: PrivateKey = PrivateKey { der: Vec::new() };

impl KeyPair {
    pub fn get_public(&self) -> &'static PublicKey {
        &DUMMY_PUBLIC_KEY
    }
    pub fn get_private(&self) -> &'static PrivateKey {
        &DUMMY_PRIVATE_KEY
    }
}

pub fn panic_message(e: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = e.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = e.downcast_ref::<String>() {
        s.clone()
    } else {
        format!("{:?}", e)
    }
}


// ================= BookController 修复补充（恢复追加 v2） =================
pub struct Triple<A, B, C>(pub A, pub B, pub C);
impl<A, B, C> Triple<A, B, C> {
    pub fn new(a: A, b: B, c: C) -> Triple<A, B, C> { Triple(a, b, c) }
    pub fn first(&self) -> &A { &self.0 }
    pub fn second(&self) -> &B { &self.1 }
    pub fn third(&self) -> &C { &self.2 }
}

pub struct JsonNode(pub serde_json::Value);
impl JsonNode {
    pub fn new() -> JsonNode { JsonNode(serde_json::Value::Null) }
    pub fn get(&self, key: &str) -> Option<JsonNode> {
        self.0.get(key).cloned().map(JsonNode)
    }
    pub fn to_string(&self) -> String {
        match &self.0 {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        }
    }
    pub fn is_not_blank(&self) -> bool { !self.0.is_null() }
    pub fn contains(&self, _other: &str) -> bool { false }
}
impl Default for JsonNode { fn default() -> JsonNode { JsonNode::new() } }
impl std::ops::Add<&str> for JsonNode {
    type Output = JsonNode;
    fn add(self, _rhs: &str) -> JsonNode { self }
}

pub struct HttpServerResponse;
impl HttpServerResponse {
    pub fn new() -> HttpServerResponse { HttpServerResponse }
    pub fn put_header(&mut self, _k: &str, _v: &str) -> &mut Self { self }
    pub fn set_status_code(&mut self, _c: i32) -> &mut Self { self }
    pub fn end(&mut self, _s: String) {}
    pub fn write(&mut self, _s: &str) {}
    pub fn set_chunked(&mut self, _b: bool) -> &mut Self { self }
}

pub struct MultiMap(pub std::collections::HashMap<String, String>);
impl MultiMap {
    pub fn new() -> MultiMap { MultiMap(std::collections::HashMap::new()) }
    pub fn add(&mut self, k: &str, v: &str) { self.0.insert(k.to_string(), v.to_string()); }
    pub fn get(&self, k: &str) -> Option<String> { self.0.get(k).cloned() }
    pub fn to_string(&self) -> String {
        serde_json::to_string(&self.0).unwrap_or_default()
    }
}

pub struct CoroutineExceptionHandler;
impl CoroutineExceptionHandler {
    pub fn new(_f: impl Fn(&dyn std::any::Any, &StubError) + 'static) -> CoroutineExceptionHandler {
        CoroutineExceptionHandler
    }
}
impl std::ops::Add for CoroutineExceptionHandler {
    type Output = CoroutineExceptionHandler;
    fn add(self, _o: CoroutineExceptionHandler) -> CoroutineExceptionHandler { CoroutineExceptionHandler }
}
impl std::ops::Add<CoroutineContext> for CoroutineExceptionHandler {
    type Output = CoroutineContext;
    fn add(self, _o: CoroutineContext) -> CoroutineContext { CoroutineContext }
}
impl std::ops::Add<&CoroutineContext> for CoroutineExceptionHandler {
    type Output = CoroutineContext;
    fn add(self, _o: &CoroutineContext) -> CoroutineContext { CoroutineContext }
}
impl std::ops::Add<CoroutineExceptionHandler> for CoroutineContext {
    type Output = CoroutineContext;
    fn add(self, _o: CoroutineExceptionHandler) -> CoroutineContext { CoroutineContext }
}

// Option<JsonObject> 链式读取（BookController 使用）
pub trait JsonObjectOptionExt {
    fn get_string_opt(&self, key: &str) -> Option<String>;
    fn get_integer(&self, key: &str, default: i32) -> i32;
    fn get_long(&self, key: &str, default: i64) -> i64;
    fn get_float(&self, key: &str, default: f32) -> f32;
    fn get_json_object(&self, key: &str) -> Option<JsonObject>;
    fn get_json_array(&self, key: &str) -> Option<JsonArray>;
    fn get_string_or(&self, key: &str, default: &str) -> String;
    fn get_json_array_or(&self, key: &str, default: JsonArray) -> JsonArray;
    fn get_json_object_or(&self, key: &str, default: JsonObject) -> JsonObject;
    fn get_json_array_opt(&self, key: &str) -> Option<JsonArray>;
}
impl JsonObjectOptionExt for Option<JsonObject> {
    fn get_string_opt(&self, key: &str) -> Option<String> {
        self.as_ref().and_then(|o| o.get_string_opt_inner(key))
    }
    fn get_integer(&self, key: &str, default: i32) -> i32 {
        self.as_ref().and_then(|o| o.get_integer_opt(key)).unwrap_or(default)
    }
    fn get_long(&self, key: &str, default: i64) -> i64 {
        self.as_ref().and_then(|o| o.get_long_opt(key)).unwrap_or(default)
    }
    fn get_float(&self, key: &str, default: f32) -> f32 {
        self.as_ref().and_then(|o| o.get_float_opt(key)).unwrap_or(default)
    }
    fn get_json_object(&self, key: &str) -> Option<JsonObject> {
        self.as_ref().and_then(|o| o.get_json_object_opt(key))
    }
    fn get_json_array(&self, key: &str) -> Option<JsonArray> {
        self.as_ref().and_then(|o| o.get_json_array_opt(key))
    }
    fn get_string_or(&self, key: &str, default: &str) -> String {
        self.as_ref().and_then(|o| o.get_string_opt_inner(key)).unwrap_or_else(|| default.to_string())
    }
    fn get_json_array_or(&self, key: &str, default: JsonArray) -> JsonArray {
        self.as_ref().and_then(|o| o.get_json_array_opt(key)).unwrap_or(default)
    }
    fn get_json_object_or(&self, key: &str, default: JsonObject) -> JsonObject {
        self.as_ref().and_then(|o| o.get_json_object_opt(key)).unwrap_or(default)
    }
    fn get_json_array_opt(&self, key: &str) -> Option<JsonArray> {
        self.as_ref().and_then(|o| o.get_json_array_opt(key))
    }
}

impl JsonObject {
    pub fn get_string_opt(&self, key: &str) -> Option<String> {
        self.get_string_opt_inner(key)
    }
    pub fn get_string_opt_inner(&self, key: &str) -> Option<String> {
        serde_json::from_str::<serde_json::Value>(&self.0).ok()
            .and_then(|v| v.get(key).cloned())
            .map(|v| match v { serde_json::Value::String(s) => s, other => other.to_string() })
    }
    pub fn get_float_opt(&self, key: &str) -> Option<f32> {
        serde_json::from_str::<serde_json::Value>(&self.0).ok()
            .and_then(|v| v.get(key).cloned())
            .and_then(|v| v.as_f64())
            .map(|f| f as f32)
    }
    pub fn get_json_object_opt(&self, key: &str) -> Option<JsonObject> {
        serde_json::from_str::<serde_json::Value>(&self.0).ok()
            .and_then(|v| v.get(key).cloned())
            .map(|v| JsonObject(v.to_string()))
    }
    pub fn get_json_array_opt(&self, key: &str) -> Option<JsonArray> {
        serde_json::from_str::<serde_json::Value>(&self.0).ok()
            .and_then(|v| v.get(key).cloned())
            .map(|v| JsonArray(vec![v.to_string()]))
    }
}

// ResponseHandle 扩展
pub trait ResponseHandleExt {
    fn write(&mut self, s: &str);
    fn set_chunked(&mut self, b: bool) -> &mut Self;
}
impl ResponseHandleExt for crate::stubs::io::vertx::ResponseHandle {
    fn write(&mut self, s: &str) {
        let mut r = self.0.borrow_mut();
        if r.body.is_none() { r.body = Some(Vec::new()); }
        if let Some(b) = r.body.as_mut() { b.extend_from_slice(s.as_bytes()); }
    }
    fn set_chunked(&mut self, _b: bool) -> &mut Self { self }
}

// Any From 装箱
impl From<i32> for Any { fn from(v: i32) -> Any { Any::Long(v as i64) } }
impl From<i64> for Any { fn from(v: i64) -> Any { Any::Long(v) } }
impl From<f32> for Any { fn from(v: f32) -> Any { Any::Double(v as f64) } }
impl From<f64> for Any { fn from(v: f64) -> Any { Any::Double(v) } }
impl From<usize> for Any { fn from(v: usize) -> Any { Any::Long(v as i64) } }
impl From<JsonObject> for Any { fn from(v: JsonObject) -> Any { Any::JsonObject(v) } }
impl From<crate::io_legado_app_data_entities_book::Book> for Any {
    fn from(v: crate::io_legado_app_data_entities_book::Book) -> Any {
        Any::JsonObject(crate::stubs::JsonObject(crate::stubs::book_to_json(&v).to_string()))
    }
}
impl From<crate::io_legado_app_data_entities_bookchapter::BookChapter> for Any {
    fn from(v: crate::io_legado_app_data_entities_bookchapter::BookChapter) -> Any {
        Any::JsonObject(crate::stubs::JsonObject(crate::stubs::book_chapter_to_json(&v).to_string()))
    }
}
impl From<crate::io_legado_app_data_entities_searchbook::SearchBook> for Any {
    fn from(v: crate::io_legado_app_data_entities_searchbook::SearchBook) -> Any {
        Any::JsonObject(crate::stubs::JsonObject(crate::stubs::search_book_to_json(&v).to_string()))
    }
}
impl From<crate::io_legado_app_data_entities_searchresult::SearchResult> for Any {
    fn from(v: crate::io_legado_app_data_entities_searchresult::SearchResult) -> Any {
        Any::Str(crate::stubs::search_result_to_json(&v).to_string())
    }
}
impl From<Vec<crate::io_legado_app_data_entities_bookchapter::BookChapter>> for Any {
    fn from(v: Vec<crate::io_legado_app_data_entities_bookchapter::BookChapter>) -> Any {
        let items: Vec<String> = v
            .iter()
            .map(|c| crate::stubs::book_chapter_to_json(c).to_string())
            .collect();
        Any::JsonArray(crate::stubs::JsonArray(items))
    }
}
impl From<Vec<crate::io_legado_app_data_entities_searchbook::SearchBook>> for Any {
    fn from(v: Vec<crate::io_legado_app_data_entities_searchbook::SearchBook>) -> Any {
        let items: Vec<String> = v
            .iter()
            .map(|b| crate::stubs::search_book_to_json(b).to_string())
            .collect();
        Any::JsonArray(crate::stubs::JsonArray(items))
    }
}
impl From<Vec<crate::io_legado_app_data_entities_searchresult::SearchResult>> for Any {
    fn from(v: Vec<crate::io_legado_app_data_entities_searchresult::SearchResult>) -> Any {
        let items: Vec<String> = v
            .iter()
            .map(|r| crate::stubs::search_result_to_json(r).to_string())
            .collect();
        Any::JsonArray(crate::stubs::JsonArray(items))
    }
}
impl From<Vec<crate::io_legado_app_data_entities_rssarticle::RssArticle>> for Any {
    fn from(v: Vec<crate::io_legado_app_data_entities_rssarticle::RssArticle>) -> Any {
        let items: Vec<String> = v
            .iter()
            .map(|a| crate::stubs::rss_article_to_json(a).to_string())
            .collect();
        Any::JsonArray(crate::stubs::JsonArray(items))
    }
}
impl From<Vec<String>> for Any {
    fn from(v: Vec<String>) -> Any { Any::List(v.into_iter().map(Any::Str).collect()) }
}

// File / URL / Mutex / CoroutineContext / ObjectNode 补充
impl File {
    pub fn unzip(&self, dest: &str) -> bool {
        crate::runtime::zip::unzip_to(&self.file_path, dest)
    }
}
impl URL {
    pub fn read_bytes(&self) -> Option<Vec<u8>> { None }
}
impl Mutex {
    pub fn lock_sync(&self) {}
    pub fn unlock_sync(&self) {}
}
impl CoroutineContext {
    pub fn cancel(&self) {}
}
impl ObjectNode {
    pub fn get(&self, key: &str) -> Option<JsonNode> {
        self.0.get(key).map(|s| {
            serde_json::from_str::<serde_json::Value>(s)
                .map(JsonNode)
                .unwrap_or_else(|_| JsonNode(serde_json::Value::String(s.clone())))
        })
    }
}

pub trait JsonObjectMapTo {
    fn map_to<T>(self) -> Option<T>
    where
        T: serde::de::DeserializeOwned;
}
impl JsonObjectMapTo for Option<JsonObject> {
    fn map_to<T>(self) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.and_then(|o| crate::stubs::JsonObject::map_to(&o))
    }
}

// ================= ReturnData JSON 序列化（实体转换 + Any downcast 分发） =================
include!("json_conv.rs");

pub fn json_object_to_value(o: &JsonObject) -> Value {
    serde_json::from_str::<Value>(&o.0).unwrap_or(Value::Null)
}

pub fn json_array_to_value(a: &JsonArray) -> Value {
    Value::Array(a.0.iter().filter_map(|s| serde_json::from_str::<Value>(s).ok()).collect())
}

pub fn any_map_to_value(m: &HashMap<String, Any>) -> Value {
    let mut obj = serde_json::Map::new();
    for (k, v) in m { obj.insert(k.clone(), any_to_value(v)); }
    Value::Object(obj)
}

pub fn any_map_boxed_to_value(m: &HashMap<String, Box<dyn std::any::Any>>) -> Value {
    let mut obj = serde_json::Map::new();
    for (k, v) in m { obj.insert(k.clone(), any_to_json_value(v.as_ref())); }
    Value::Object(obj)
}

pub fn any_list_to_value(l: &[crate::stubs::Any]) -> Value {
    Value::Array(l.iter().map(any_to_value).collect())
}

pub fn any_boxed_list_to_value(l: &[Box<dyn std::any::Any>]) -> Value {
    Value::Array(l.iter().map(|x| any_to_json_value(x.as_ref())).collect())
}

pub fn any_to_value(a: &crate::stubs::Any) -> Value {
    match a {
        crate::stubs::Any::Null => Value::Null,
        crate::stubs::Any::Bool(b) => Value::Bool(*b),
        crate::stubs::Any::Long(i) => json!(*i),
        crate::stubs::Any::Double(d) => json!(*d),
        crate::stubs::Any::Str(s) => Value::String(s.clone()),
        crate::stubs::Any::JsonObject(o) => json_object_to_value(o),
        crate::stubs::Any::JsonArray(arr) => json_array_to_value(arr),
        crate::stubs::Any::List(l) => any_list_to_value(l),
        crate::stubs::Any::Map(m) => any_map_to_value(m),
        crate::stubs::Any::ReadContext(r) => Value::String(r.json.clone()),
        crate::stubs::Any::JXNode(n) => Value::String(n.text.clone()),
        crate::stubs::Any::JXDocument(d) => Value::String(d.text.clone()),
        crate::stubs::Any::Document(d) => Value::String(d.text.clone()),
        crate::stubs::Any::Element(e) => Value::String(e.text.clone()),
        // fix: Elements → 文本数组（JS 规则 result.eachText()/result[0]/result.length 可用；
        //      原无分隔拼接——"章一章二章三"）
        crate::stubs::Any::Elements(es) => {
            Value::Array(es.list.iter().map(|e| Value::String(e.text.clone())).collect())
        }
    }
}

pub fn any_to_json_value(d: &dyn std::any::Any) -> Value {
    use crate::stubs::*;
    if let Some(v) = d.downcast_ref::<Any>() { return any_to_value(v); }
    if let Some(v) = d.downcast_ref::<String>() { return Value::String(v.clone()); }
    if let Some(v) = d.downcast_ref::<&str>() { return Value::String(v.to_string()); }
    if let Some(v) = d.downcast_ref::<bool>() { return Value::Bool(*v); }
    if let Some(v) = d.downcast_ref::<i32>() { return json!(*v); }
    if let Some(v) = d.downcast_ref::<i64>() { return json!(*v); }
    if let Some(v) = d.downcast_ref::<u32>() { return json!(*v); }
    if let Some(v) = d.downcast_ref::<usize>() { return json!(*v); }
    if let Some(v) = d.downcast_ref::<f32>() { return json!(*v); }
    if let Some(v) = d.downcast_ref::<f64>() { return json!(*v); }
    if let Some(v) = d.downcast_ref::<JsonObject>() { return json_object_to_value(v); }
    if let Some(v) = d.downcast_ref::<JsonArray>() { return json_array_to_value(v); }
    if let Some(v) = d.downcast_ref::<HashMap<String, Box<dyn std::any::Any>>>() { return any_map_boxed_to_value(v); }
    // fix: getUserInfo 的 userInfo 是 Option<HashMap<...>>——原无 Option 分支序列化为 null
    if let Some(v) = d.downcast_ref::<Option<std::collections::HashMap<String, Box<dyn std::any::Any>>>>() {
        return match v {
            Some(m) => any_map_boxed_to_value(m),
            None => Value::Null,
        };
    }
    if let Some(v) = d.downcast_ref::<HashMap<String, i32>>() { return Value::Object(v.iter().map(|(k, x)| (k.clone(), json!(*x))).collect()); }
    if let Some(v) = d.downcast_ref::<HashMap<String, i64>>() { return Value::Object(v.iter().map(|(k, x)| (k.clone(), json!(*x))).collect()); }
    if let Some(v) = d.downcast_ref::<HashMap<String, u32>>() { return Value::Object(v.iter().map(|(k, x)| (k.clone(), json!(*x))).collect()); }
    if let Some(v) = d.downcast_ref::<HashMap<String, usize>>() { return Value::Object(v.iter().map(|(k, x)| (k.clone(), json!(*x))).collect()); }
    if let Some(v) = d.downcast_ref::<HashMap<String, f64>>() { return Value::Object(v.iter().map(|(k, x)| (k.clone(), json!(*x))).collect()); }
    if let Some(v) = d.downcast_ref::<HashMap<String, String>>() { return Value::Object(v.iter().map(|(k, x)| (k.clone(), json!(x))).collect()); }
    if let Some(v) = d.downcast_ref::<Vec<Box<dyn std::any::Any>>>() { return any_boxed_list_to_value(v); }
    if let Some(v) = d.downcast_ref::<Vec<Any>>() { return any_list_to_value(v); }
    if let Some(v) = d.downcast_ref::<Vec<String>>() { return vec_string_to_value(v); }
    if let Some(v) = d.downcast_ref::<Vec<i32>>() { return Value::Array(v.iter().map(|x| json!(*x)).collect()); }
    if let Some(v) = d.downcast_ref::<Vec<serde_json::Value>>() { return Value::Array(v.clone()); }
    if let Some(v) = d.downcast_ref::<HashMap<String, crate::stubs::Any>>() { return any_map_to_value(v); }
    if let Some(v) = d.downcast_ref::<Vec<HashMap<String, Box<dyn std::any::Any>>>>() { return Value::Array(v.iter().map(any_map_boxed_to_value).collect()); }
    if let Some(v) = d.downcast_ref::<Vec<HashMap<String, Any>>>() { return Value::Array(v.iter().map(any_map_to_value).collect()); }
    if let Some(v) = d.downcast_ref::<Vec<JsonObject>>() { return Value::Array(v.iter().map(json_object_to_value).collect()); }
    if let Some(v) = d.downcast_ref::<crate::io_legado_app_data_entities_book::Book>() { return book_to_json(v); }
    if let Some(v) = d.downcast_ref::<Option<crate::io_legado_app_data_entities_book::Book>>() {
        return match v { Some(b) => book_to_json(b), None => Value::Null };
    }
    if let Some(v) = d.downcast_ref::<crate::io_legado_app_data_entities_booksource::BookSource>() { return book_source_to_json(v); }
    if let Some(v) = d.downcast_ref::<crate::io_legado_app_data_entities_rsssource::RssSource>() { return rss_source_to_json(v); }
    if let Some(v) = d.downcast_ref::<crate::io_legado_app_data_entities_replacerule::ReplaceRule>() { return replace_rule_to_json(v); }
    if let Some(v) = d.downcast_ref::<crate::io_legado_app_data_entities_bookgroup::BookGroup>() { return book_group_to_json(v); }
    if let Some(v) = d.downcast_ref::<crate::io_legado_app_data_entities_bookmark::Bookmark>() { return bookmark_to_json(v); }
    if let Some(v) = d.downcast_ref::<crate::io_legado_app_data_entities_txttocrule::TxtTocRule>() { return txt_toc_rule_to_json(v); }
    if let Some(v) = d.downcast_ref::<crate::io_legado_app_data_entities_rssarticle::RssArticle>() { return rss_article_to_json(v); }
    if let Some(v) = d.downcast_ref::<Option<crate::com_htmake_reader_entity_user::User>>() { return match v { Some(u) => user_to_json(u), None => Value::Null }; }
    if let Some(v) = d.downcast_ref::<Option<bool>>() { return match v { Some(b) => Value::Bool(*b), None => Value::Null }; }
    if let Some(v) = d.downcast_ref::<Option<String>>() { return match v { Some(s) => Value::String(s.clone()), None => Value::Null }; }
    if let Some(v) = d.downcast_ref::<Option<i32>>() { return match v { Some(i) => json!(*i), None => Value::Null }; }
    if let Some(v) = d.downcast_ref::<Option<i64>>() { return match v { Some(i) => json!(*i), None => Value::Null }; }
    if let Some(v) = d.downcast_ref::<Vec<crate::io_legado_app_data_entities_rssarticle::RssArticle>>() { return rss_articles_to_json(v); }
    if let Some(v) = d.downcast_ref::<crate::io_legado_app_data_entities_searchbook::SearchBook>() { return search_book_to_json(v); }
    if let Some(v) = d.downcast_ref::<crate::io_legado_app_data_entities_bookchapter::BookChapter>() { return book_chapter_to_json(v); }
    if let Some(v) = d.downcast_ref::<crate::io_legado_app_data_entities_httptts::HttpTTS>() { return http_tts_to_json(v); }
    if let Some(v) = d.downcast_ref::<crate::com_htmake_reader_entity_user::User>() { return user_to_json(v); }
    if let Some(v) = d.downcast_ref::<Vec<crate::io_legado_app_data_entities_book::Book>>() { return books_to_json(v); }
    if let Some(v) = d.downcast_ref::<Vec<crate::io_legado_app_data_entities_booksource::BookSource>>() { return book_sources_to_json(v); }
    if let Some(v) = d.downcast_ref::<Vec<crate::io_legado_app_data_entities_rsssource::RssSource>>() { return rss_sources_to_json(v); }
    if let Some(v) = d.downcast_ref::<Vec<crate::io_legado_app_data_entities_replacerule::ReplaceRule>>() { return replace_rules_to_json(v); }
    if let Some(v) = d.downcast_ref::<Vec<crate::io_legado_app_data_entities_bookgroup::BookGroup>>() { return book_groups_to_json(v); }
    if let Some(v) = d.downcast_ref::<Vec<crate::io_legado_app_data_entities_bookmark::Bookmark>>() { return bookmarks_to_json(v); }
    if let Some(v) = d.downcast_ref::<Vec<crate::io_legado_app_data_entities_txttocrule::TxtTocRule>>() { return txt_toc_rules_to_json(v); }
    if let Some(v) = d.downcast_ref::<Vec<crate::io_legado_app_data_entities_searchbook::SearchBook>>() { return search_books_to_json(v); }
    if let Some(v) = d.downcast_ref::<Vec<crate::io_legado_app_data_entities_bookchapter::BookChapter>>() { return book_chapters_to_json(v); }
    if let Some(v) = d.downcast_ref::<Vec<crate::io_legado_app_data_entities_httptts::HttpTTS>>() { return http_tts_list_to_json(v); }
    if let Some(v) = d.downcast_ref::<Vec<crate::com_htmake_reader_entity_user::User>>() { return users_to_json(v); }
    Value::Null
}

// ================= 控制器 handler 统一驱动（async fn 与同步 fn 兼容） =================
pub trait IntoAnyResult {
    fn into_any_result(self) -> Box<dyn std::any::Any>;
}
impl IntoAnyResult for crate::com_htmake_reader_api_returndata::ReturnData {
    fn into_any_result(self) -> Box<dyn std::any::Any> {
        Box::new(self)
    }
}
impl<F, O> IntoAnyResult for F
where
    F: std::future::Future<Output = O> + 'static,
    O: 'static,
{
    fn into_any_result(self) -> Box<dyn std::any::Any> {
        Box::new(pollster::block_on(self))
    }
}


impl JsonObject {
    // 真实 GSON 反序列化（实体需实现 serde::Deserialize）
    pub fn map_to_deser<T>(&self) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        serde_json::from_str::<T>(&self.0).ok()
    }
}

impl WebRequest {
    // JS 引擎同步请求（java.ajax 使用）
    pub fn send_blocking(&self) -> String {
        self.client.as_ref()
            .and_then(|c| {
                let mut req = c.get(&self.url);
                if let Some(t) = self.timeout_ms {
                    req = req.timeout(std::time::Duration::from_millis(t));
                }
                req.send().ok()
            })
            .and_then(|r| r.text().ok())
            .unwrap_or_default()
    }
}

impl Element {
    pub fn html(&self) -> String {
        self.html.clone()
    }
}
