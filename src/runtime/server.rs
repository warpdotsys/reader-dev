// 服务器层：把 vertx 风格的 Router 规则桥接到 axum，提供 main 启动入口。

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::stubs::io::vertx::{HttpMethod, HttpRequest, HttpResponse, RouteRule, RouteStep, Router, StaticHandler};

// ---------------- 路径转换：vertx 风格 → axum ----------------

fn convert_path(path: &str) -> String {
    if path.is_empty() {
        return "/{*rest}".to_string();
    }
    // axum 通配：{*name} 必须独占一段
    if path == "/*" {
        return "/{*rest}".to_string();
    }
    if path.ends_with("/*") {
        return format!("{}/{{*rest}}", &path[..path.len() - 2]);
    }
    if path == "*" {
        return "/{*rest}".to_string();
    }
    path.to_string()
}

/// 段内通配（如 /reader3/webdav*）拆成 axum 可表达的多个路径：
/// 子路径 /reader3/webdav/{*rest} + 前缀自身 /reader3/webdav
fn convert_paths(path: &str) -> Vec<String> {
    let base = convert_path(path);
    if base == path && path.contains('*') {
        let idx = path.find('*').unwrap();
        let prefix = &path[..idx];
        // fix: 段内通配（/reader3/webdav*）——补尾斜杠路径（axum {*rest} 不匹配尾斜杠根路径，
        //      /reader3/webdav/ 请求会落到 /* → 404）
        return vec![format!("{}/{{*rest}}", prefix), prefix.to_string(), format!("{}/", prefix)];
    }
    vec![base]
}

fn url_decode(s: &str) -> String {
    percent_encoding::percent_decode_str(s).decode_utf8_lossy().to_string()
}

fn match_method(rule_method: Option<HttpMethod>, actual: &str) -> bool {
    match rule_method {
        None => true,
        Some(m) => {
            let name = format!("{:?}", m);
            name.eq_ignore_ascii_case(actual)
        }
    }
}

fn parse_query(raw: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in raw.split('&') {
        if pair.is_empty() {
            continue;
        }
        if let Some((k, v)) = pair.split_once('=') {
            map.insert(url_decode(k), url_decode(v));
        } else {
            map.insert(url_decode(pair), String::new());
        }
    }
    map
}

// ---------------- 静态文件服务 ----------------

fn serve_static(handler: &StaticHandler, req_path: &str, response: &mut HttpResponse) {
    let root = if let Some(w) = handler.web_root.clone() {
        w
    } else if let Some(c) = handler.classpath_root.clone() {
        // classpath 资源：已含 src/ 前缀（项目路径）直接使用；否则视为 resources 下相对目录
        if c.starts_with("src/") || c.starts_with("./") || c.starts_with("C:") || c.starts_with("D:") {
            c
        } else {
            ["src/main/resources".to_string(), c.trim_start_matches('/').to_string()].join("/")
        }
    } else {
        String::new()
    };
    let mut rel = req_path.trim_start_matches('/').to_string();
    if rel.is_empty() {
        rel = "index.html".to_string();
    }
    // fix: 剥离挂载前缀（/assets/、/book-assets/、/epub/——web_root 下不应重复该前缀）
    if let Some(prefix) = &handler.mount_prefix {
        if let Some(stripped) = rel.strip_prefix(prefix.trim_start_matches('/')) {
            rel = stripped.trim_start_matches('/').to_string();
        }
    }
    // 非默认根（simple-web 等）：剥离与挂载目录同名的路径前缀
    if let Some(c) = &handler.classpath_root {
        let root_name = c.trim_matches('/').rsplit(['/', '\\']).next().unwrap_or("");
        if !root_name.is_empty() && root_name != "web" {
            if let Some(stripped) = rel.strip_prefix(root_name) {
                rel = stripped.trim_start_matches('/').to_string();
            }
        }
    }
    // 防目录穿越
    let clean: Vec<&str> = rel.split('/').filter(|s| *s != ".." && !s.is_empty()).collect();
    let clean = clean.join("/");
    let full = if root.is_empty() {
        clean
    } else {
        format!("{}/{}", root.trim_end_matches('/'), clean)
    };
    eprintln!("  static: root={} rel={} full={}", root, rel, full);
    // 直接尝试读取（避免 metadata 检查的权限/时序问题）
    match std::fs::read(&full) {
        Ok(bytes) => {
            let mime = mime_guess::from_path(&full).first_or_octet_stream();
            response.status = 200;
            response.headers.insert("content-type".to_string(), mime.to_string());
            response.body = Some(bytes);
            response.ended = true;
            return;
        }
        Err(e) => {
            eprintln!("  static read err: {} cwd={}", e, std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_default());
        }
    }
    let index = format!("{}/index.html", full.trim_end_matches('/'));
    if let Ok(bytes) = std::fs::read(&index) {
        response.status = 200;
        response.headers.insert("content-type".to_string(), "text/html; charset=utf-8".to_string());
        response.body = Some(bytes);
        response.ended = true;
        return;
    }
    response.status = 404;
    response.body = Some(b"Not Found".to_vec());
    response.ended = true;
}

// ---------------- 规则执行 ----------------

// 路径匹配：返回特异性分数（越高越优先），None 表示不匹配
// 精确路径 > 带通配前缀 > 全通配（/*）> 空路径（route()）
/// 解析请求体：multipart/form-data 提取文件并保存到 storage/file-uploads；否则按 UTF-8 文本
fn parse_http_body(bytes: &[u8], content_type: &str) -> (String, Vec<String>) {
    let ct_lower = content_type.to_lowercase();
    if ct_lower.contains("multipart/form-data") {
        let boundary = extract_boundary(content_type);
        if let Some(boundary) = boundary {
            let files = parse_multipart(bytes, &boundary);
            let mut uploads = Vec::new();
            let dir = std::path::Path::new("storage").join("file-uploads");
            let _ = std::fs::create_dir_all(&dir);
            for (name, data, filename) in files {
                if data.is_empty() {
                    continue;
                }
                let safe_name: String = filename
                    .chars()
                    .map(|c| if c.is_alphanumeric() || c == '.' || c == '_' || c == '-' { c } else { '_' })
                    .collect();
                let path = dir.join(format!("{}_{}", name, safe_name));
                let _ = std::fs::write(&path, &data);
                uploads.push(path.to_string_lossy().to_string());
            }
            return (String::new(), uploads);
        }
        (String::new(), Vec::new())
    } else {
        (String::from_utf8_lossy(bytes).to_string(), Vec::new())
    }
}

fn extract_boundary(content_type: &str) -> Option<String> {
    for part in content_type.split(';') {
        let part = part.trim();
        if let Some(v) = part.strip_prefix("boundary=") {
            return Some(v.trim_matches('"').to_string());
        }
    }
    None
}

/// 解析 multipart body：返回 (字段名, 数据, 文件名)
fn parse_multipart(bytes: &[u8], boundary: &str) -> Vec<(String, Vec<u8>, String)> {
    let mut result = Vec::new();
    let boundary_bytes = format!("--{}", boundary).into_bytes();
    let body = bytes;
    let mut pos = 0usize;
    // 跳过开头的 --boundary
    while pos + boundary_bytes.len() <= body.len() {
        if &body[pos..pos + boundary_bytes.len()] == boundary_bytes.as_slice() {
            break;
        }
        pos += 1;
    }
    pos += boundary_bytes.len();
    while pos + 2 <= body.len() && body[pos] == b'\r' && body[pos + 1] == b'\n' {
        pos += 2;
    }
    loop {
        // 找下一个 --boundary
        let next = find_bytes(body, &boundary_bytes, pos);
        if next.is_none() {
            break;
        }
        let part_end = next.unwrap();
        let part = &body[pos..part_end];
        // 解析 part：headers\r\n\r\n data
        if let Some(sep) = find_bytes(part, b"\r\n\r\n", 0) {
            let head = String::from_utf8_lossy(&part[..sep]).to_string();
            let data = part[sep + 4..].to_vec();
            let mut name = String::new();
            let mut filename = String::new();
            for line in head.split("\r\n") {
                if line.to_lowercase().contains("content-disposition") {
                    if let Some(n) = extract_attr(&line, "name") {
                        name = n;
                    }
                    if let Some(f) = extract_attr(&line, "filename") {
                        filename = f;
                    }
                }
            }
            if !name.is_empty() {
                result.push((name, data, filename));
            }
        }
        // 跳过 --boundary（可能是结尾 --boundary--）
        pos = part_end + boundary_bytes.len();
        // 结尾判断：--boundary-- 后面是 -- 
        if pos + 2 <= body.len() && body[pos] == b'-' && body[pos + 1] == b'-' {
            break;
        }
        while pos + 2 <= body.len() && body[pos] == b'\r' && body[pos + 1] == b'\n' {
            pos += 2;
        }
    }
    result
}

fn find_bytes(haystack: &[u8], needle: &[u8], start: usize) -> Option<usize> {
    if needle.is_empty() || haystack.len() < start + needle.len() {
        return None;
    }
    let mut i = start;
    while i + needle.len() <= haystack.len() {
        if &haystack[i..i + needle.len()] == needle {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn extract_attr(line: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr);
    if let Some(start) = line.find(&pattern) {
        let rest = &line[start + pattern.len()..];
        let end = rest.find('"').unwrap_or(rest.len());
        return Some(rest[..end].to_string());
    }
    let pattern = format!("{}=", attr);
    if let Some(start) = line.find(&pattern) {
        let rest = &line[start + pattern.len()..];
        let end = rest.find(';').unwrap_or(rest.len());
        return Some(rest[..end].trim_matches('"').to_string());
    }
    None
}

fn rule_match_score(rule: &RouteRule, method: &str, path: &str) -> Option<i32> {    if !match_method(rule.method, method) {
        return None;
    }
    let pattern = &rule.path;
    if pattern.is_empty() {
        return Some(0);
    }
    let path_trim = path.trim_matches('/');
    if pattern == "/*" || pattern == "*" {
        return Some(1);
    }
    if let Some(star) = pattern.find('*') {
        // 通配前缀：/a/b/* 或 /reader3/webdav*
        let prefix = &pattern[..star];
        if path.starts_with(prefix) {
            let segments = prefix.trim_matches('/').split('/').filter(|s| !s.is_empty()).count();
            return Some((segments as i32) * 10 + 2);
        }
        return None;
    }
    if pattern.trim_matches('/') == path_trim {
        let segments = pattern.trim_matches('/').split('/').filter(|s| !s.is_empty()).count();
        return Some((segments as i32) * 10 + 5);
    }
    None
}

fn execute_rules(
    rules: &[RouteRule],
    method: &str,
    uri: &str,
    query_map: &HashMap<String, String>,
    headers: &HashMap<String, String>,
    body: String,
    file_uploads: Vec<String>,
    path_params: &HashMap<String, String>,
) -> (i32, HashMap<String, String>, Option<Vec<u8>>) {
    let path = uri.split('?').next().unwrap_or(uri).to_string();
    // 选择特异性最高的匹配规则（vertx 语义：最长匹配优先）
    let mut best: Option<(i32, &RouteRule)> = None;
    eprintln!("  match: path={} method={} rules={}", path, method, rules.len());
    for rule in rules {
        if let Some(score) = rule_match_score(rule, method, &path) {
            eprintln!("    rule {} {:?} score={}", rule.path, rule.method, score);
            if best.as_ref().map(|(s, _)| score > *s).unwrap_or(true) {
                best = Some((score, rule));
            }
        }
    }
    let Some((_, rule)) = best else {
        return (404, HashMap::new(), Some(b"Not Found".to_vec()));
    };
    let mut ctx = crate::stubs::io::vertx::RoutingContext::new();
    ctx.file_uploads = file_uploads;
    {
        let mut req = ctx.request.borrow_mut();
        req.raw_method_str = method.to_uppercase();
        req.method = match method.to_uppercase().as_str() {
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "DELETE" => HttpMethod::DELETE,
            "HEAD" => HttpMethod::HEAD,
            "OPTIONS" => HttpMethod::OPTIONS,
            "PATCH" => HttpMethod::PATCH,
            _ => HttpMethod::GET,
        };
        req.path = path.clone();
        req.absolute_uri = uri.to_string();
        req.query = query_map.clone();
        req.path_params = path_params.clone();
        req.headers = headers.clone();
        req.body = Some(body.clone());
        req.raw_body = body.clone().into_bytes();
    }
    let mut steps = rule.steps.borrow_mut();
    for step in steps.iter_mut() {
        match step {
            RouteStep::Handler(f) => {
                f(&mut ctx);
                if ctx.response.borrow().ended {
                    break;
                }
            }
            RouteStep::Static(sh) => {
                let mut resp = ctx.response.borrow_mut();
                serve_static(sh, &path, &mut resp);
                break;
            }
        }
    }
    if ctx.response.borrow().ended {
        let r = ctx.response.borrow();
        let status = if r.status == 0 { 200 } else { r.status };
        if let Some(path) = &r.send_file {
            if let Ok(content) = std::fs::read(path) {
                return (status, r.headers.clone(), Some(content));
            }
        }
        return (status, r.headers.clone(), r.body.clone());
    }
    (404, HashMap::new(), Some(b"Not Found".to_vec()))
}

// ---------------- axum 桥接 ----------------

fn axum_response(status: i32, headers: &HashMap<String, String>, body: Option<Vec<u8>>) -> axum::response::Response {
    use axum::body::Body;
    use axum::http::header;
    use axum::http::StatusCode;
    let mut builder = axum::response::Response::builder().status(
        StatusCode::from_u16(status.clamp(100, 599) as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
    );
    for (k, v) in headers {
        if let Ok(name) = header::HeaderName::from_bytes(k.as_bytes()) {
            builder = builder.header(name, v);
        }
    }
    builder
        .body(Body::from(body.unwrap_or_default()))
        .unwrap_or_else(|_| axum::response::Response::new(Body::empty()))
}

async fn dispatch(
    rules: &[RouteRule],
    method: axum::http::Method,
    uri: axum::http::Uri,
    params: HashMap<String, String>,
    query: axum::extract::Query<HashMap<String, String>>,
    headers: axum::http::HeaderMap,
    body: String,
    file_uploads: Vec<String>,
) -> axum::response::Response {
    eprintln!("dispatch: {} {}", method, uri);
    let mut header_map = HashMap::new();
    for (k, v) in headers.iter() {
        if let Ok(v) = v.to_str() {
            header_map.insert(k.as_str().to_string(), v.to_string());
        }
    }

    // CORS 预检
    let origin = header_map.get("origin").cloned().unwrap_or_default();
    if method == axum::http::Method::OPTIONS && !origin.is_empty() {
        let mut h = HashMap::new();
        h.insert("Access-Control-Allow-Origin".to_string(), origin.clone());
        h.insert("Access-Control-Allow-Credentials".to_string(), "true".to_string());
        h.insert("Access-Control-Allow-Methods".to_string(), "GET, POST, PATCH, PUT, DELETE, OPTIONS".to_string());
        h.insert("Access-Control-Allow-Headers".to_string(), "Authorization, Content-Type, If-Match, If-Modified-Since, If-None-Match, If-Unmodified-Since, X-Requested-With".to_string());
        return axum_response(200, &h, Some(b"".to_vec()));
    }

    let (status, mut resp_headers, resp_body) = execute_rules(
        &rules,
        method.as_str(),
        &uri.to_string(),
        &query.0,
        &header_map,
        body,
        file_uploads,
        &params,
    );

    if !origin.is_empty() && !resp_headers.contains_key("Access-Control-Allow-Origin") {
        resp_headers.insert("Access-Control-Allow-Origin".to_string(), origin);
        resp_headers.insert("Access-Control-Allow-Credentials".to_string(), "true".to_string());
    }

    // 静态文件（send_file）
    if resp_headers.is_empty() && resp_body.is_none() && status == 200 {
        // 无 body 的 200 → 空
    }

    axum_response(status, &resp_headers, resp_body)
}

pub fn build_axum_app(router: Router) -> axum::Router {
    let rules: Vec<RouteRule> = router.rules.borrow().clone();

    // 按 axum 路径分组（axum 不允许重复注册同一路径）
    let mut grouped: HashMap<String, Vec<RouteRule>> = HashMap::new();
    for rule in rules {
        for axpath in convert_paths(&rule.path) {
            grouped.entry(axpath).or_default().push(rule.clone());
        }
    }

    let mut app = axum::Router::new();
    for (axpath, group) in grouped {
        // fix: axum handler 的 Future 需 Send；规则以 &'static 切片传递（unsafe Sync 保证单线程执行安全）
        let rules_static: &'static [RouteRule] = Box::leak(group.into_boxed_slice());
        let handler = move |method: axum::http::Method,
                             uri: axum::http::Uri,
                             params: axum::extract::Path<HashMap<String, String>>,
                             query: axum::extract::Query<HashMap<String, String>>,
                             headers: axum::http::HeaderMap,
                             body: axum::body::Bytes| {
            async move {
                // multipart/form-data 解析（上传接口依赖）
                let content_type = headers
                    .get("content-type")
                    .map(|v| v.to_str().unwrap_or("").to_string())
                    .unwrap_or_default();
                let (body_str, file_uploads) = parse_http_body(&body, &content_type);
                dispatch(rules_static, method, uri, params.0, query, headers, body_str, file_uploads).await
            }
        };
        app = app.route(&axpath, axum::routing::any(handler));
        // fix: axum 的 catch-all /{*rest} 不匹配根路径 "/"，单独补注册
        if axpath == "/{*rest}" {
            app = app.route("/", axum::routing::any(handler));
        }
    }

    async fn not_found() -> axum::response::Response {
        axum::response::Response::builder()
            .status(axum::http::StatusCode::NOT_FOUND)
            .body(axum::body::Body::from("Not Found"))
            .unwrap()
    }

    app.fallback(not_found)
}

// ---------------- 启动 ----------------

pub fn start_server(router: Router, port: i32, on_listen: impl FnMut(bool) + Send + 'static) {
    let mut cb = on_listen;
    std::thread::spawn(move || {
        // fix: 路由/处理器基于 Rc/RefCell（非 Send），使用单线程 runtime 保证全部 handler 在同一线程执行
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("tokio runtime error: {}", e);
                cb(false);
                return;
            }
        };
        rt.block_on(async move {
            let app = build_axum_app(router);
            let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port as u16));
            let listener = match tokio::net::TcpListener::bind(addr).await {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("listen error: {}", e);
                    cb(false);
                    return;
                }
            };
            cb(true);
            if let Err(e) = axum::serve(listener, app).await {
                eprintln!("server error: {}", e);
            }
        });
    });
}

// ---------------- 应用启动（main 入口） ----------------

/// 定时任务调度（对应 Kotlin @Scheduled 注解）
fn spawn_scheduled_jobs() {
    std::thread::spawn(|| {
        use chrono::Timelike;
        let mut last_run_min: Option<i64> = None;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(30));
            let now = chrono::Local::now();
            let total_min = now.hour() as i64 * 60 + now.minute() as i64;
            let api = crate::com_htmake_reader_api_yueduapi::YueduApi::new();
            // 每 10 分钟：书架刷新 + 远程书源订阅（方法内部按配置间隔自判）
            if total_min % 10 == 0 && last_run_min != Some(total_min) {
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    api.shelf_update_job();
                    api.remote_book_source_sub_update_job();
                }));
                last_run_min = Some(total_min);
            }
            // 23:50 每日自动 WebDAV 备份
            if now.hour() == 23 && now.minute() == 50 {
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| api.auto_backup()));
            }
            // 23:59 每日清理不活跃用户
            if now.hour() == 23 && now.minute() == 59 {
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| api.clear_user()));
            }
            // 2:00 每日自动 GC
            if now.hour() == 2 && now.minute() == 0 {
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| api.auto_gc()));
            }
        }
    });
}

pub fn run_application(port: i32, context_path: &str) {
    let mut yuedu_api = crate::com_htmake_reader_api_yueduapi::YueduApi::new();
    let mut router = Router::router(crate::stubs::io::vertx::Vertx::vertx());

    // 静态资源：项目内 web 构建产物
    let web_root = "src/main/resources/web";
    router
        .route_with_path("/*")
        .handler_static(crate::stubs::io::vertx::StaticHandler::create_root(web_root));

    // API 路由
    yuedu_api.init_router(&mut router);

    // 定时任务调度（对应 Kotlin @Scheduled：书架刷新/远程书源订阅每 10 分钟，
    // 23:50 自动备份、23:59 清理不活跃用户、2:00 自动 GC）
    spawn_scheduled_jobs();

    let full_router = if !context_path.is_empty() {
        let mut main = Router::router(crate::stubs::io::vertx::Vertx::vertx());
        main.mount_sub_router(context_path.to_string(), router);
        main
    } else {
        router
    };

    {
        let rules = full_router.rules.borrow();
        println!("registered routes: {}", rules.len());
        for r in rules.iter().take(20) {
            println!("  route: {} {:?}", r.path, r.method);
        }
    }

    let log_port = port;
    start_server(full_router, port, move |ok| {
        if ok {
            println!("reader server started on port {}", log_port);
        }
    });
}

pub fn block_forever() {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
}
