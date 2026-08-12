// 真实 okhttp3 语义请求执行（reqwest blocking），供 Call::enqueue 使用

use crate::stubs::{Request, Response};

pub fn execute(req: &Request) -> Result<Response, crate::stubs::Throwable> {
    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
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
    let body_text = resp.text().unwrap_or_default();
    Ok(Response {
        status,
        headers,
        body_text,
        url: req.url.clone(),
    })
}
