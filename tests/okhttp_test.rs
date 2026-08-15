use reader::stubs::Request;

#[test]
fn test_okhttp_execute() {
    let req = Request {
        url: "http://127.0.0.1:18999/search?key=test".to_string(),
        method: "GET".to_string(),
        ..Default::default()
    };
    let r = reader::runtime::okhttp::execute(&req, None, None);
    println!("result: {:?}", r.as_ref().map(|resp| (resp.status, resp.body_text.len())));
}

#[test]
fn test_multipart_build() {
    use reader::stubs::{MultipartBody, RequestBody};
    let mb = MultipartBody::builder();
    mb.add_form_data_part("field1", "", RequestBody::from_text("value1"));
    mb.add_form_data_part("file", "a.txt", RequestBody::from_text("hello"));
    let body = mb.build();
    assert!(body.text.contains("name=\"field1\""), "field1 name part");
    assert!(body.text.contains("value1"), "field1 value");
    assert!(body.text.contains("filename=\"a.txt\""), "file filename");
    assert!(body.text.contains("hello"), "file content");
    assert!(body.text.ends_with("--\r\n"), "boundary close");
}
#[test]
fn test_socks4_proxy_request() {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        for stream in listener.incoming().take(1) {
            if let Ok(mut s) = stream {
                // SOCKS4a 握手（本测试 host=127.0.0.1、userid 空——总长 19 字节）
                let mut handshake = [0u8; 19];
                let _ = s.read_exact(&mut handshake);
                assert_eq!(handshake[0], 0x04, "VN=4");
                assert_eq!(handshake[1], 0x01, "CD=CONNECT");
                assert_eq!(&handshake[4..8], &[0, 0, 0, 1], "SOCKS4a 域名标记");
                // 成功响应
                let _ = s.write_all(&[0, 0x5A, 0, 0, 0, 0, 0, 0]);
                // 读 HTTP 请求头
                let mut req = Vec::new();
                let mut tmp = [0u8; 2048];
                let mut header_end = false;
                while !header_end {
                    let n = s.read(&mut tmp).unwrap_or(0);
                    if n == 0 { break; }
                    req.extend_from_slice(&tmp[..n]);
                    header_end = req.windows(4).any(|w| w == b"\r\n\r\n");
                }
                let req_str = String::from_utf8_lossy(&req);
                assert!(req_str.contains("GET /hello HTTP/1.1"), "socks4 HTTP request line: {req_str}");
                assert!(req_str.contains("Host: 127.0.0.1"), "Host header: {req_str}");
                let resp = "HTTP/1.1 200 OK\r\nContent-Length: 11\r\n\r\nhello world";
                let _ = s.write_all(resp.as_bytes());
            }
        }
    });
    let req = reader::stubs::Request {
        url: "http://127.0.0.1:9999/hello".to_string(),
        method: "GET".to_string(),
        headers: std::collections::HashMap::new(),
        ..Default::default()
    };
    let proxy = format!("socks4://127.0.0.1:{}", addr.port());
    let r = reader::runtime::okhttp::execute(&req, Some(&proxy), None).expect("socks4 request");
    assert_eq!(r.status, 200, "status");
    assert_eq!(r.body_text, "hello world", "body");
}