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
fn rule_match_score(rule: &RouteRule, method: &str, path: &str) -> Option<i32> {
    if !match_method(rule.method, method) {
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
    {
        let mut req = ctx.request.borrow_mut();
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
        let axpath = convert_path(&rule.path);
        grouped.entry(axpath).or_default().push(rule);
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
                             body: String| {
            async move {
                dispatch(rules_static, method, uri, params.0, query, headers, body).await
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
