// 真实 okhttp3 语义请求执行（reqwest blocking），供 Call::enqueue 使用

use crate::stubs::{Request, Response};

pub fn execute(
    req: &Request,
    proxy: Option<&str>,
    proxy_auth: Option<(&str, &str)>,
) -> Result<Response, crate::stubs::Throwable> {
    // reqwest::blocking 在 async 上下文创建/丢弃 runtime 会 panic → 独立线程执行
    let req = req.clone();
    let proxy = proxy.map(|s| s.to_string());
    let proxy_auth = proxy_auth.map(|(u, p)| (u.to_string(), p.to_string()));
    std::thread::spawn(move || execute_inner(&req, proxy.as_deref(), proxy_auth.as_ref().map(|(u, p)| (u.as_str(), p.as_str()))))
        .join()
        .map_err(|_| crate::stubs::StubError::new("request thread panicked"))?
}

fn execute_inner(
    req: &Request,
    proxy: Option<&str>,
    proxy_auth: Option<(&str, &str)>,
) -> Result<Response, crate::stubs::Throwable> {
    // fix: socks4 代理手写握手（reqwest 仅支持 socks5——书源配置 socks4:// 时代理不生效）
    if let Some(p) = proxy {
        let p_trim = p.trim();
        if p_trim.starts_with("socks4://") {
            return socks4_http_request(req, p_trim, proxy_auth);
        }
    }
    let mut client_builder = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .connect_timeout(std::time::Duration::from_secs(15))
        .timeout(std::time::Duration::from_secs(45))
        // fix: 与原版 OkHttp unsafeTrustManager 一致——全信任证书（自签/过期站点可用）
        .danger_accept_invalid_certs(true);
    if let Some(p) = proxy {
        if !p.trim().is_empty() {
            if let Ok(mut rp) = reqwest::Proxy::all(p) {
                // fix: 代理账号密码（Proxy-Authorization Basic）
                if let Some((u, pw)) = proxy_auth {
                    rp = rp.basic_auth(u, pw);
                }
                client_builder = client_builder.proxy(rp);
            }
        }
    }
    let client = client_builder
        .build()
        .map_err(|e| crate::stubs::StubError::new(e.to_string()))?;

    // fix: POST form 的 body 作为请求体发送（Kotlin FormBody 语义），不拼 URL
    let mut builder = if req.method == "POST" {
        let mut b = client.post(&req.url);
        // fix: form body 必须带 Content-Type: application/x-www-form-urlencoded
        let has_ct = req.headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("content-type"));
        if !has_ct {
            if let Some(ct) = &req.content_type {
                b = b.header("Content-Type", ct);
            }
        }
        b
    } else {
        client.get(&req.url)
    };

    for (k, v) in &req.headers {
        builder = builder.header(k, v);
    }
    // fix: 二进制 body 用原始字节（lossy 文本会损坏）
    if let Some(body_bytes) = &req.body_bytes {
        builder = builder.body(body_bytes.clone());
    } else if let Some(body) = &req.body {
        builder = builder.body(body.clone());
    }

    let resp = builder
        .send()
        .map_err(|e| crate::stubs::StubError::new(e.to_string()))?;

    let status = resp.status().as_u16() as i32;
    let mut headers = std::collections::HashMap::new();
    let mut headers_multi = std::collections::HashMap::new();
    for (k, v) in resp.headers() {
        let key = k.as_str().to_lowercase();
        if let Ok(v) = v.to_str() {
            headers.insert(key.clone(), v.to_string());
            headers_multi.entry(key).or_insert_with(Vec::new).push(v.to_string());
        }
    }
    let final_url = resp.url().to_string();
    // fix: 保留原始字节（lossy 解码会损坏二进制内容——封面/图片/EPUB 下载）
    let body_bytes = resp.bytes().unwrap_or_default().to_vec();
    let body_text = String::from_utf8_lossy(&body_bytes).into_owned();
    Ok(Response {
        status,
        headers,
        headers_multi,
        body_text,
        body_bytes,
        url: final_url,
    })
}

/// socks4/socks4a 手写代理（HTTP 场景；HTTPS 隧道需 TLS 手动协商——返回错误提示）
fn socks4_http_request(
    req: &Request,
    proxy: &str,
    proxy_auth: Option<(&str, &str)>,
) -> Result<Response, crate::stubs::Throwable> {
    use std::io::{Read, Write};
    let rest = proxy.trim_start_matches("socks4://");
    let (proxy_host, proxy_port) = match rest.rsplit_once(':') {
        Some((h, p)) => (h.to_string(), p.parse::<u16>().unwrap_or(1080)),
        None => (rest.to_string(), 1080),
    };
    let (target_host, target_port, https) = parse_target(&req.url);
    if https {
        return Err(crate::stubs::StubError::new("socks4 代理暂不支持 HTTPS 隧道".to_string()));
    }
    let mut stream = std::net::TcpStream::connect((proxy_host.as_str(), proxy_port))
        .map_err(|e| crate::stubs::StubError::new(format!("socks4 connect: {}", e)))?;
    stream.set_read_timeout(Some(std::time::Duration::from_secs(30))).ok();
    // SOCKS4a CONNECT 握手
    let userid = proxy_auth.map(|(u, _)| u).unwrap_or("");
    let mut handshake = Vec::new();
    handshake.push(0x04); // VN = 4
    handshake.push(0x01); // CD = CONNECT
    handshake.extend_from_slice(&target_port.to_be_bytes());
    handshake.extend_from_slice(&[0, 0, 0, 1]); // SOCKS4a 域名标记
    handshake.extend_from_slice(userid.as_bytes());
    handshake.push(0x00);
    handshake.extend_from_slice(target_host.as_bytes());
    handshake.push(0x00);
    stream.write_all(&handshake).map_err(|e| crate::stubs::StubError::new(format!("socks4 handshake write: {}", e)))?;
    let mut reply = [0u8; 8];
    stream.read_exact(&mut reply).map_err(|e| crate::stubs::StubError::new(format!("socks4 handshake read: {}", e)))?;
    if reply[1] != 0x5A {
        return Err(crate::stubs::StubError::new(format!("socks4 connect 被拒绝 (code {})", reply[1])));
    }
    // 构造并发送 HTTP 请求
    let (_, _, path) = split_url_path(&req.url);
    let mut http = String::new();
    http.push_str(&format!("{} {} HTTP/1.1\r\n", req.method, path));
    let mut headers = req.headers.clone();
    headers.insert(String::from("Host"), target_host.clone());
    for (k, v) in &headers {
        http.push_str(&format!("{}: {}\r\n", k, v));
    }
    http.push_str("\r\n");
    if let Some(b) = &req.body {
        http.push_str(b);
    }
    stream.write_all(http.as_bytes()).map_err(|e| crate::stubs::StubError::new(format!("socks4 http write: {}", e)))?;
    parse_http_response(&mut stream)
}

/// 从 URL 解析目标 host/port/https
fn parse_target(url: &str) -> (String, u16, bool) {
    let https = url.starts_with("https://");
    let rest = url
        .trim_start_matches("http://")
        .trim_start_matches("https://");
    let hostport = match rest.find('/') {
        Some(i) => &rest[..i],
        None => rest,
    };
    match hostport.rsplit_once(':') {
        Some((h, p)) => (
            h.to_string(),
            p.parse::<u16>().unwrap_or(if https { 443 } else { 80 }),
            https,
        ),
        None => (hostport.to_string(), if https { 443 } else { 80 }, https),
    }
}

/// 从 URL 提取请求路径（默认 /）
fn split_url_path(url: &str) -> (String, u16, String) {
    let (host, port, _) = parse_target(url);
    let rest = url
        .trim_start_matches("http://")
        .trim_start_matches("https://");
    let path = match rest.find('/') {
        Some(i) => rest[i..].to_string(),
        None => String::from("/"),
    };
    (host, port, path)
}

/// 从原始 TCP 流解析 HTTP 响应（状态行 + headers + Content-Length/chunked body）
fn parse_http_response(stream: &mut std::net::TcpStream) -> Result<Response, crate::stubs::Throwable> {
    use std::io::Read;
    let mut buf = Vec::new();
    let mut tmp = [0u8; 8192];
    let header_end = loop {
        let n = stream.read(&mut tmp).map_err(|e| crate::stubs::StubError::new(format!("socks4 read: {}", e)))?;
        if n == 0 {
            return Err(crate::stubs::StubError::new("socks4 连接提前关闭".to_string()));
        }
        buf.extend_from_slice(&tmp[..n]);
        if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break pos + 4;
        }
    };
    let header_str = String::from_utf8_lossy(&buf[..header_end]).into_owned();
    let status = header_str
        .lines()
        .next()
        .unwrap_or("")
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);
    let mut headers = std::collections::HashMap::new();
    for line in header_str.lines().skip(1) {
        if let Some(colon) = line.find(':') {
            headers.insert(
                line[..colon].trim().to_lowercase(),
                line[colon + 1..].trim().to_string(),
            );
        }
    }
    let mut body = buf[header_end..].to_vec();
    if let Some(cl) = headers.get("content-length").and_then(|v| v.parse::<usize>().ok()) {
        while body.len() < cl {
            let n = stream.read(&mut tmp).map_err(|e| crate::stubs::StubError::new(format!("socks4 body read: {}", e)))?;
            if n == 0 {
                break;
            }
            body.extend_from_slice(&tmp[..n]);
        }
        body.truncate(cl);
    } else if headers.get("transfer-encoding").map(|v| v.contains("chunked")).unwrap_or(false) {
        // 简化 chunked 解析
        let mut decoded = Vec::new();
        let mut rest = body;
        loop {
            // 读行（chunk size）
            let mut line_end = None;
            for (i, w) in rest.windows(2).enumerate() {
                if w == b"\r\n" {
                    line_end = Some(i);
                    break;
                }
            }
            let Some(le) = line_end else { break };
            let size_str = String::from_utf8_lossy(&rest[..le]).to_string();
            let size = usize::from_str_radix(size_str.split(';').next().unwrap_or("0").trim(), 16).unwrap_or(0);
            rest.drain(..le + 2);
            if size == 0 {
                break;
            }
            while rest.len() < size {
                let n = stream.read(&mut tmp).map_err(|e| crate::stubs::StubError::new(format!("socks4 chunk read: {}", e)))?;
                if n == 0 {
                    break;
                }
                rest.extend_from_slice(&tmp[..n]);
            }
            decoded.extend_from_slice(&rest[..size.min(rest.len())]);
            rest.drain(..size);
            // 跳过 chunk 尾部 \r\n
            if rest.len() >= 2 && &rest[..2] == b"\r\n" {
                rest.drain(..2);
            }
        }
        body = decoded;
    }
    Ok(Response {
        status,
        headers: headers.clone(),
        headers_multi: headers.iter().map(|(k, v)| (k.clone(), vec![v.clone()])).collect(),
        body_text: String::from_utf8_lossy(&body).into_owned(),
        body_bytes: body,
        url: String::new(),
    })
}
