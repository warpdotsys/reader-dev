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

    let url = if req.form_fields.is_empty() {
        req.url.clone()
    } else {
        let mut u = req.url.clone();
        let sep = if u.contains('?') { '&' } else { '?' };
        let mut parts = Vec::new();
        for (k, v) in &req.form_fields {
            parts.push(format!("{}={}", k, v));
        }
        u.push(sep);
        u.push_str(&parts.join("&"));
        u
    };

    let mut builder = if req.method == "POST" {
        client.post(&url)
    } else {
        client.get(&url)
    };

    for (k, v) in &req.headers {
        builder = builder.header(k, v);
    }
    if let Some(body) = &req.body {
        builder = builder.body(body.clone());
    }

    let resp = builder
        .send()
        .map_err(|e| crate::stubs::StubError::new(e.to_string()))?;

    let status = resp.status().as_u16() as i32;
    let mut headers = std::collections::HashMap::new();
    for (k, v) in resp.headers() {
        if let Ok(v) = v.to_str() {
            headers.insert(k.as_str().to_string(), v.to_string());
        }
    }
    let final_url = resp.url().to_string();
    let body_text = resp.text().unwrap_or_default();
    Ok(Response {
        status,
        headers,
        body_text,
        url: final_url,
    })
}
