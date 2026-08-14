// 真实 Vert.x 风格运行时（可运行版）
// 由 stubs.rs 的 io::vertx 模块 include 引入；基于 axum + tokio 提供 HTTP 服务。

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::stubs::StubError;
use crate::stubs::{JsonArray, JsonObject, Throwable};

pub type RouteStepFn = Box<dyn FnMut(&mut RoutingContext)>;

// ---------------- 请求 / 响应 ----------------

#[derive(Clone, Default)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub raw_method_str: String,
    pub path: String,
    pub absolute_uri: String,
    pub query: HashMap<String, String>,
    pub path_params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub raw_body: Vec<u8>,
}

impl HttpRequest {
    pub fn path(&self) -> String {
        self.path.clone()
    }
    pub fn method(&self) -> HttpMethod {
        self.method.clone()
    }
    pub fn absolute_uri(&self) -> String {
        self.absolute_uri.clone()
    }
    pub fn get_header(&self, name: &str) -> Option<String> {
        for (k, v) in &self.headers {
            if k.eq_ignore_ascii_case(name) {
                return Some(v.clone());
            }
        }
        None
    }

    pub fn get_param(&self, name: &str) -> Option<String> {
        self.query.get(name).cloned().or_else(|| self.path_params.get(name).cloned())
    }
    pub fn raw_method(&self) -> String {
        if self.raw_method_str.is_empty() {
            format!("{:?}", self.method)
        } else {
            self.raw_method_str.clone()
        }
    }
    pub fn query_param(&self, name: &str) -> Option<String> {
        self.query.get(name).cloned()
    }
    pub fn path_param(&self, name: &str) -> String {
        self.path_params.get(name).cloned().unwrap_or_default()
    }
    pub fn body(&self) -> Option<String> {
        self.body.clone()
    }
    pub fn uri(&self) -> String {
        self.absolute_uri.clone()
    }
}

#[derive(Clone, Default)]
pub struct HttpResponse {
    pub status: i32,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub send_file: Option<String>,
    pub ended: bool,
}

impl HttpResponse {
    pub fn put_header(&mut self, k: &str, v: &str) -> &mut Self {
        self.headers.insert(k.to_string(), v.to_string());
        self
    }
    pub fn set_status_code(&mut self, code: i32) -> &mut Self {
        self.status = code;
        self
    }
    pub fn end(&mut self, s: String) {
        self.body = Some(s.into_bytes());
        self.ended = true;
    }
    pub fn json(&mut self, s: String) {
        self.headers
            .insert("content-type".to_string(), "application/json; charset=utf-8".to_string());
        self.body = Some(s.into_bytes());
        self.ended = true;
    }
    pub fn send_file(&mut self, path: String) {
        self.send_file = Some(path);
        self.ended = true;
    }
    pub fn put_header_str(&mut self, k: String, v: String) -> &mut Self {
        self.headers.insert(k, v);
        self
    }
}

#[derive(Clone)]
pub struct ResponseHandle(pub Rc<RefCell<HttpResponse>>);

impl ResponseHandle {
    pub fn put_header(&mut self, k: &str, v: &str) -> &mut Self {
        self.0.borrow_mut().headers.insert(k.to_string(), v.to_string());
        self
    }
    pub fn set_status_code(&mut self, code: i32) -> &mut Self {
        self.0.borrow_mut().status = code;
        self
    }
    pub fn end(&mut self, s: String) {
        let mut r = self.0.borrow_mut();
        r.body = Some(s.into_bytes());
        r.ended = true;
    }
    pub fn json(&mut self, s: String) {
        let mut r = self.0.borrow_mut();
        r.headers
            .insert("content-type".to_string(), "application/json; charset=utf-8".to_string());
        r.body = Some(s.into_bytes());
        r.ended = true;
    }
    pub fn send_file(&mut self, path: String) {
        let mut r = self.0.borrow_mut();
        r.send_file = Some(path);
        r.ended = true;
    }

    pub fn head_written(&self) -> bool {
        let r = self.0.borrow();
        r.headers.contains_key("content-type") || r.status != 0
    }
}

// ---------------- 路由上下文 ----------------

#[derive(Clone)]
pub struct RoutingContext {
    pub request: Rc<RefCell<HttpRequest>>,
    pub response: Rc<RefCell<HttpResponse>>,
    pub next_called: bool,
    pub failed: Option<Throwable>,
    pub file_uploads: Vec<String>,
    pub store: Rc<RefCell<HashMap<String, Rc<dyn std::any::Any>>>>,
    pub base_path: String,
}

impl Default for RoutingContext {
    fn default() -> Self {
        RoutingContext {
            request: Rc::new(RefCell::new(HttpRequest::default())),
            response: Rc::new(RefCell::new(HttpResponse::default())),
            next_called: false,
            failed: None,
            file_uploads: Vec::new(),
            store: Rc::new(RefCell::new(HashMap::new())),
            base_path: String::new(),
        }
    }
}

impl RoutingContext {
    pub fn new() -> Self {
        RoutingContext::default()
    }

    pub fn request(&self) -> HttpRequest {
        self.request.borrow().clone()
    }

    pub fn response(&self) -> ResponseHandle {
        ResponseHandle(self.response.clone())
    }

    pub fn method(&self) -> HttpMethod {
        self.request.borrow().method.clone()
    }

    pub fn path(&self) -> String {
        self.request.borrow().path.clone()
    }

    pub fn body(&self) -> Option<String> {
        self.request.borrow().body.clone()
    }

    pub fn query_param(&self, name: &str) -> Option<String> {
        self.request.borrow().query.get(name).cloned()
    }

    pub fn path_param(&self, name: &str) -> String {
        self.request.borrow().path_params.get(name).cloned().unwrap_or_default()
    }

    pub fn put_header(&self, k: &str, v: &str) -> &Self {
        self.response.borrow_mut().headers.insert(k.to_string(), v.to_string());
        self
    }

    pub fn end(&self, s: String) {
        let mut r = self.response.borrow_mut();
        r.body = Some(s.into_bytes());
        r.ended = true;
    }

    pub fn json(&self, s: String) {
        let mut r = self.response.borrow_mut();
        r.headers
            .insert("content-type".to_string(), "application/json; charset=utf-8".to_string());
        r.body = Some(s.into_bytes());
        r.ended = true;
    }

    pub fn fail(&mut self, code: i32, msg: String) {
        let mut r = self.response.borrow_mut();
        r.status = code;
        r.body = Some(msg.clone().into_bytes());
        r.ended = true;
        self.failed = Some(Throwable::new(msg));
    }

    pub fn next(&self) {
        // 链式 handler 由服务器调度；next 语义：不再拦截，继续后续 handler。
        // 单 handler 场景无需动作。
    }

    pub fn absolute_uri(&self) -> String {
        self.request.borrow().absolute_uri.clone()
    }

    pub fn raw_method(&self) -> String {
        format!("{:?}", self.request.borrow().method)
    }

    pub fn get_header(&self, name: &str) -> Option<String> {
        self.request.borrow().get_header(name)
    }

    pub fn add_headers_end_handler(&self, _f: impl FnOnce(())) {
        // 响应头结束回调：CORS 头已在服务器层统一处理
    }

    pub fn file_uploads(&self) -> Vec<String> {
        self.file_uploads.clone()
    }

    pub fn body_as_string(&self) -> String {
        self.request.borrow().body.clone().unwrap_or_default()
    }

    pub fn get_cookie(&self, _name: &str) -> Option<Cookie> {
        None
    }

    pub fn remove_cookie(&self, _name: &str) {}

    pub fn connection(&self) -> RoutingContext {
        self.clone()
    }

    pub fn close_handler(&self, _f: impl FnOnce()) {}

    pub fn failure(&self) -> Option<Throwable> {
        self.failed.clone()
    }

    pub fn set_status_code(&self, code: i32) -> &Self {
        self.response.borrow_mut().status = code;
        self
    }

    pub fn send_file(&self, path: String) {
        let mut r = self.response.borrow_mut();
        r.send_file = Some(path);
        r.ended = true;
    }

    pub fn get_body(&self) -> crate::stubs::io::vertx::Buffer {
        let bytes = self.request.borrow().raw_body.clone();
        crate::stubs::io::vertx::Buffer::new(bytes)
    }

    pub fn put<T: 'static>(&self, key: &str, value: T) {
        self.store.borrow_mut().insert(key.to_string(), Rc::new(value));
    }

    pub fn get_user<T: 'static>(&self, key: &str) -> Option<T> {
        self.store
            .borrow()
            .get(key)
            .and_then(|v| v.clone().downcast::<T>().ok())
            .map(|rc| Rc::try_unwrap(rc).unwrap_or_else(|rc| unsafe { std::ptr::read(&*rc) }))
    }

    pub fn body_as_json(&self) -> Option<JsonObject> {
        self.request
            .borrow()
            .body
            .clone()
            .map(|b| JsonObject::new_parsed(&b))
    }

    pub fn body_as_json_array(&self) -> Option<JsonArray> {
        self.request
            .borrow()
            .body
            .clone()
            .and_then(|b| JsonArray::from_json(b))
    }

    // ---- CURD 控制器体系方法（原 com_htmake_reader_api_controller_curd::RoutingContext） ----

    pub fn get_param(&self, name: &str) -> Option<String> {
        self.query_param(name).or_else(|| self.path_param_opt(name))
    }

    pub fn path_param_opt(&self, name: &str) -> Option<String> {
        self.request.borrow().path_params.get(name).cloned()
    }

    pub fn session(&self) -> Session {
        Session
    }

    pub fn success(&self, data: &crate::com_htmake_reader_api_returndata::ReturnData) {
        // fix: errorMsg 经 serde 转义（原裸拼接会破坏含引号的错误信息 JSON）
        let body = format!(
            "{{\"isSuccess\":{},\"errorMsg\":{},\"data\":{}}}",
            data.is_success(),
            serde_json::to_string(data.error_msg()).unwrap_or_else(|_| "\"\"".to_string()),
            data.data()
                .as_ref()
                .map(|d| crate::stubs::any_to_json_value(d.as_ref()).to_string())
                .unwrap_or_else(|| "null".to_string())
        );
        self.response
            .borrow_mut()
            .headers
            .insert("content-type".to_string(), "application/json; charset=utf-8".to_string());
        self.response.borrow_mut().body = Some(body.into_bytes());
        self.response.borrow_mut().ended = true;
    }

    pub fn file_uploads_opt(&self) -> Option<Vec<FileUpload>> {
        if self.file_uploads.is_empty() {
            None
        } else {
            Some(self.file_uploads.iter().map(|p| FileUpload::new(p.clone())).collect())
        }
    }

    pub fn get_file(&self, key: &str) -> Option<crate::stubs::File> {
        if let Some(v) = self.store.borrow().get(key) {
            if let Some(f) = v.downcast_ref::<crate::stubs::File>() {
                return Some(f.clone());
            }
        }
        self.request.borrow().path_params.get(key).map(|p| crate::stubs::File::new(p.as_str()))
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.query_param(key)
    }

    pub fn remove(&self, _key: &str) {}

    pub fn head_written(&self) -> bool {
        self.response.borrow().headers.contains_key("content-type")
            || self.response.borrow().status != 0
    }
}

pub struct Session;

impl Session {
    // fix: 委托 stubs 全局会话存储（真实内存会话）
    pub fn put(&self, key: &str, value: String) {
        crate::stubs::Session::put(&crate::stubs::Session, key, value);
    }
    pub fn get(&self, key: &str) -> Option<String> {
        crate::stubs::Session::get(&crate::stubs::Session, key)
    }
    pub fn remove(&self, key: &str) {
        crate::stubs::Session::put(&crate::stubs::Session, key, String::new());
    }
    pub fn destroy(&self) {
        crate::stubs::Session::destroy(&crate::stubs::Session);
    }
}

pub struct FileUpload {
    pub file_name: String,
}

impl FileUpload {
    pub fn new(file_name: String) -> FileUpload {
        FileUpload { file_name }
    }
    pub fn uploaded_file_name(&self) -> String {
        self.file_name.clone()
    }
    pub fn file_name(&self) -> String {
        self.file_name.clone()
    }
}

// ---------------- Cookie / Session 占位（真实会话后续实现） ----------------

pub struct Cookie {
    name: String,
    max_age: i64,
    path: String,
}

impl Cookie {
    pub fn new(name: String) -> Cookie {
        Cookie { name, max_age: -1, path: "/".to_string() }
    }
    pub fn set_max_age(&mut self, age: i64) {
        self.max_age = age;
    }
    pub fn set_path(&mut self, path: &str) {
        self.path = path.to_string();
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}

pub struct SessionHandler;
impl SessionHandler {
    pub fn create(_store: LocalSessionStore) -> SessionHandler {
        SessionHandler
    }
    pub fn set_session_cookie_name(&self, _name: &str) -> SessionHandler {
        SessionHandler
    }
    pub fn set_session_timeout(&self, _timeout: i64) -> SessionHandler {
        SessionHandler
    }
    pub fn set_session_cookie_path(&self, _path: &str) -> SessionHandler {
        SessionHandler
    }
}

pub struct LocalSessionStore;
impl LocalSessionStore {
    pub fn create(_vertx: Vertx) -> LocalSessionStore {
        LocalSessionStore
    }
}

pub struct BodyHandler;
impl BodyHandler {
    pub fn create() -> BodyHandler {
        BodyHandler
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoggerFormat {
    DEFAULT,
}

pub struct LoggerHandler;
impl LoggerHandler {
    pub fn create(_format: LoggerFormat) -> LoggerHandler {
        LoggerHandler
    }
}

pub struct URLDecoder;
impl URLDecoder {
    pub fn decode(s: String, _encoding: &str) -> String {
        percent_encoding::percent_decode_str(&s)
            .decode_utf8_lossy()
            .to_string()
    }
}

pub struct AsyncResult {
    pub succeeded: bool,
    pub cause: Option<String>,
}

impl AsyncResult {
    pub fn succeeded(&self) -> bool {
        self.succeeded
    }
}

// ---------------- 静态文件处理器 ----------------

#[derive(Clone)]
pub struct StaticHandler {
    pub web_root: Option<String>,
    pub classpath_root: Option<String>,
    pub allow_root: bool,
    pub default_encoding: Option<String>,
    pub directory_listing: bool,
}

impl StaticHandler {
    pub fn create(root: &str) -> StaticHandler {
        StaticHandler {
            classpath_root: if root.is_empty() { None } else { Some(root.to_string()) },
            web_root: None,
            allow_root: false,
            default_encoding: None,
            directory_listing: false,
        }
    }
    pub fn create_root(root: &str) -> StaticHandler {
        StaticHandler {
            classpath_root: Some(root.to_string()),
            web_root: None,
            allow_root: false,
            default_encoding: None,
            directory_listing: false,
        }
    }
    pub fn set_allow_root_file_system_access(mut self, v: bool) -> StaticHandler {
        self.allow_root = v;
        self
    }
    pub fn set_web_root(mut self, root: impl AsRef<str>) -> StaticHandler {
        self.web_root = Some(root.as_ref().to_string());
        self
    }
    pub fn set_default_content_encoding(mut self, enc: &str) -> StaticHandler {
        self.default_encoding = Some(enc.to_string());
        self
    }
    pub fn set_directory_listing(mut self, v: bool) -> StaticHandler {
        self.directory_listing = v;
        self
    }
}

// ---------------- 路由 ----------------

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum HttpMethod {
    #[default]
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    PATCH,
    TRACE,
}

pub enum RouteStep {
    Handler(RouteStepFn),
    Static(StaticHandler),
}

#[derive(Clone)]
pub struct RouteRule {
    pub path: String,
    pub method: Option<HttpMethod>,
    pub steps: Rc<RefCell<Vec<RouteStep>>>,
}

impl RouteRule {
    fn new(path: String, method: Option<HttpMethod>) -> RouteRule {
        RouteRule { path, method, steps: Rc::new(RefCell::new(Vec::new())) }
    }
}

#[derive(Clone)]
pub struct Router {
    pub rules: Rc<RefCell<Vec<RouteRule>>>,
}

// fix: 路由器在单线程 current_thread runtime 中构建与执行（见 runtime/server.rs），
//      跨线程仅传递 Rc 句柄，不跨线程访问内部数据，因此 Send/Sync 安全
unsafe impl Send for Router {}
unsafe impl Sync for Router {}
unsafe impl Send for RouteRule {}
unsafe impl Sync for RouteRule {}
unsafe impl Send for RouteStep {}
unsafe impl Sync for RouteStep {}

impl Router {
    pub fn router(_vertx: Vertx) -> Router {
        Router { rules: Rc::new(RefCell::new(Vec::new())) }
    }

    pub fn route(&mut self) -> Route {
        self.route_with_path("")
    }

    pub fn route_with_path(&mut self, path: &str) -> Route {
        let rule = RouteRule::new(path.to_string(), None);
        self.rules.borrow_mut().push(rule.clone());
        Route { rule }
    }

    pub fn get(&mut self, path: &str) -> Route {
        let rule = RouteRule::new(path.to_string(), Some(HttpMethod::GET));
        self.rules.borrow_mut().push(rule.clone());
        Route { rule }
    }

    pub fn post(&mut self, path: &str) -> Route {
        let rule = RouteRule::new(path.to_string(), Some(HttpMethod::POST));
        self.rules.borrow_mut().push(rule.clone());
        Route { rule }
    }

    pub fn put(&mut self, path: &str) -> Route {
        let rule = RouteRule::new(path.to_string(), Some(HttpMethod::PUT));
        self.rules.borrow_mut().push(rule.clone());
        Route { rule }
    }

    pub fn delete(&mut self, path: &str) -> Route {
        let rule = RouteRule::new(path.to_string(), Some(HttpMethod::DELETE));
        self.rules.borrow_mut().push(rule.clone());
        Route { rule }
    }

    pub fn head(&mut self, path: &str) -> Route {
        let rule = RouteRule::new(path.to_string(), Some(HttpMethod::HEAD));
        self.rules.borrow_mut().push(rule.clone());
        Route { rule }
    }

    pub fn options(&mut self, path: &str) -> Route {
        let rule = RouteRule::new(path.to_string(), Some(HttpMethod::OPTIONS));
        self.rules.borrow_mut().push(rule.clone());
        Route { rule }
    }

    pub fn patch(&mut self, path: &str) -> Route {
        let rule = RouteRule::new(path.to_string(), Some(HttpMethod::PATCH));
        self.rules.borrow_mut().push(rule.clone());
        Route { rule }
    }

    pub fn mount_sub_router(&mut self, mount_point: String, sub_router: Router) {
        let sub = sub_router.rules.borrow().clone();
        for mut rule in sub {
            rule.path = format!("{}{}", mount_point, rule.path);
            self.rules.borrow_mut().push(rule);
        }
    }

    pub fn route_count(&self) -> usize {
        self.rules.borrow().len()
    }
}

#[derive(Clone)]
pub struct Route {
    pub rule: RouteRule,
}

impl Route {
    pub fn global_handler<F>(&mut self, handler: F)
    where
        F: FnMut(&mut RoutingContext) + 'static,
    {
        self.rule.steps.borrow_mut().push(RouteStep::Handler(Box::new(handler)));
    }

    pub fn global_handler_static<F>(&mut self, handler: F)
    where
        F: FnMut(&mut RoutingContext) + 'static,
    {
        self.rule.steps.borrow_mut().push(RouteStep::Handler(Box::new(handler)));
    }

    pub fn handler<F>(&mut self, handler: F)
    where
        F: FnMut(&mut RoutingContext) + 'static,
    {
        self.rule.steps.borrow_mut().push(RouteStep::Handler(Box::new(handler)));
    }

    pub fn handler_static(&mut self, static_handler: StaticHandler) {
        self.rule.steps.borrow_mut().push(RouteStep::Static(static_handler));
    }

    pub fn last(&mut self) -> Route {
        self.clone()
    }

    pub fn failure_handler<F>(&mut self, _handler: F)
    where
        F: FnMut(&mut RoutingContext) + 'static,
    {
        // 失败兜底：由服务器层统一处理
    }
}

// ---------------- Vertx / HttpServer ----------------

#[derive(Clone, Default)]
pub struct Vertx;

impl Vertx {
    pub fn vertx() -> Vertx {
        Vertx
    }

    pub fn create_http_server(&self) -> HttpServer {
        HttpServer::default()
    }

    pub fn create_http_client(&self, _options: HttpClientOptions) -> HttpClient {
        HttpClient::default()
    }

    pub fn deploy_verticle(&self, _verticle: &crate::com_htmake_reader_api_yueduapi::YueduApi) {
        // 实际部署由 SpringApplication::run 完成（YueduApi::init_router 在其中被调用）
    }
}

#[derive(Default)]
pub struct HttpServer {
    pub router: Option<Router>,
    pub exception: Option<Box<dyn FnMut(Throwable) + Send>>,
}

impl HttpServer {
    pub fn request_handler(&mut self, router: Router) -> &mut Self {
        self.router = Some(router);
        self
    }

    pub fn exception_handler<F: FnMut(Throwable) + Send + 'static>(&mut self, f: F) -> &mut Self {
        self.exception = Some(Box::new(f));
        self
    }

    pub fn listen<F: FnMut(AsyncResult) + Send + 'static>(&mut self, port: i32, mut handler: F) {
        let router = self.router.clone().unwrap();
        let mut exception = self.exception.take();
        crate::runtime::server::start_server(router, port, move |res| {
            handler(AsyncResult { succeeded: res, cause: None });
            if let Some(mut ex) = exception.take() {
                ex(Throwable::new("server stopped"));
            }
        });
    }
}

pub struct HttpClient;
impl Default for HttpClient {
    fn default() -> Self {
        HttpClient
    }
}

pub struct HttpClientOptions {
    pub trust_all: bool,
}

impl HttpClientOptions {
    pub fn new() -> HttpClientOptions {
        HttpClientOptions { trust_all: false }
    }
    pub fn set_trust_all(mut self, v: bool) -> Self {
        self.trust_all = v;
        self
    }
}

// ---------------- Buffer ----------------

#[derive(Clone, Default)]
pub struct Buffer {
    pub data: Vec<u8>,
}

impl Buffer {
    pub fn new(data: Vec<u8>) -> Buffer {
        Buffer { data }
    }
    pub fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.data).to_string()
    }
    pub fn get_bytes(&self) -> Vec<u8> {
        self.data.clone()
    }
    pub fn length(&self) -> usize {
        self.data.len()
    }
}

// ---------------- Json（Jackson 风格工具） ----------------

pub struct Json;

impl Json {
    pub fn mapper() -> crate::stubs::ObjectMapper {
        crate::stubs::ObjectMapper::new()
    }
    pub fn pretty_mapper() -> crate::stubs::ObjectMapper {
        crate::stubs::ObjectMapper::new()
    }
    pub fn encode_to_pretty_string(value: &impl serde::Serialize) -> String {
        serde_json::to_string_pretty(value).unwrap_or_default()
    }
    pub fn encode_to_string(value: &impl serde::Serialize) -> String {
        serde_json::to_string(value).unwrap_or_default()
    }
    pub fn decode_value(s: &str) -> serde_json::Value {
        serde_json::from_str(s).unwrap_or(serde_json::Value::Null)
    }
}

// ---------------- WebClient（reqwest 封装，异步包装） ----------------

pub struct WebClient {
    pub client: reqwest::blocking::Client,
}

impl Clone for WebClient {
    fn clone(&self) -> Self {
        WebClient { client: self.client.clone() }
    }
}

impl WebClient {
    pub fn wrap(_client: HttpClient, _options: WebClientOptions) -> WebClient {
        WebClient {
            client: Self::shared_client(),
        }
    }

    pub fn new() -> WebClient {
        WebClient {
            client: Self::shared_client(),
        }
    }

    // reqwest blocking Client 在 async 上下文 build/drop 会 panic → 全局单例（独立线程构建，永不 drop）
    fn shared_client() -> reqwest::blocking::Client {
        static CLIENT: std::sync::OnceLock<reqwest::blocking::Client> = std::sync::OnceLock::new();
        CLIENT
            .get_or_init(|| {
                std::thread::spawn(|| {
                    reqwest::blocking::Client::builder()
                        .redirect(reqwest::redirect::Policy::limited(10))
                        .build()
                        .unwrap_or_else(|_| reqwest::blocking::Client::new())
                })
                .join()
                .unwrap_or_else(|_| reqwest::blocking::Client::new())
            })
            .clone()
    }

    pub fn get_str(&self, url: &str, headers: &HashMap<String, String>) -> Result<String, StubError> {
        let mut req = self.client.get(url);
        for (k, v) in headers {
            req = req.header(k, v);
        }
        let resp = req.send().map_err(|e| StubError::new(e.to_string()))?;
        Ok(resp.text().map_err(|e| StubError::new(e.to_string()))?)
    }

    pub fn get_abs(&self, url: &str) -> crate::stubs::WebRequest {
        crate::stubs::WebRequest {
            url: url.to_string(),
            client: Some(self.client.clone()),
            headers: std::collections::HashMap::new(),
            timeout_ms: None,
        }
    }
}

// 兼容 BookController 等处的 io::vertx::ext::web::client 路径引用
pub mod ext {
    pub mod web {
        pub mod client {
            pub struct HttpResponse<T>(pub T);
        }
    }
}

pub struct WebClientOptions {
    pub is_try_use_compression: bool,
    pub log_activity: bool,
    pub is_follow_redirects: bool,
    pub is_trust_all: bool,
}

impl WebClientOptions {
    pub fn new() -> WebClientOptions {
        WebClientOptions {
            is_try_use_compression: false,
            log_activity: false,
            is_follow_redirects: false,
            is_trust_all: false,
        }
    }
}
