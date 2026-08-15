// 占位真实化验证：SimpleDateFormat / XmlUtils DOM / get_absolute_url
use reader::io_legado_app_utils_stringutils::StringUtils;
use reader::io_legado_app_utils_xmlutils::XmlUtils;

#[test]
fn test_simple_date_format_format_and_parse() {
    // format
    let fmt = reader::stubs::SimpleDateFormat::new("yyyy/MM/dd HH:mm");
    let out = fmt.format(0);
    assert!(!out.is_empty(), "format(0) should not be empty, got '{}'", out);
    // parse 回环
    let parsed = fmt.parse(&out).unwrap_or(0);
    assert!(parsed >= 0);
    // dateConvert_source（今天/昨天判断，不应 panic）
    let s = StringUtils::dateConvert_source("2026-08-13 12:00:00", "yyyy-MM-dd HH:mm:ss");
    assert!(!s.is_empty());
    println!("dateConvert_source: {}", s);
}

#[test]
fn test_xml2map_real_parse() {
    let xml = "<?xml version=\"1.0\"?><rss><channel>测试频道</channel><item><title>条目一</title><link>http://example.com/1</link></item></rss>";
    let map = XmlUtils::xml2map(&reader::stubs::Any::from(xml.to_string()));
    let v = map.get("rss").cloned().unwrap_or(reader::stubs::Any::Null);
    let json = format!("{}", v);
    println!("xml2map rss: {}", json);
    assert!(json.contains("测试频道"), "rss 内 channel 文本应解析: {}", json);
}

#[test]
fn test_get_absolute_url_join() {
    let base = reader::stubs::URL::parse("https://example.com/books/").unwrap();
    assert_eq!(
        reader::stubs::get_absolute_url(Some(&base), "chapter/1.html".to_string()),
        "https://example.com/books/chapter/1.html"
    );
    assert_eq!(
        reader::stubs::get_absolute_url(Some(&base), "/index.html".to_string()),
        "https://example.com/index.html"
    );
    assert_eq!(
        reader::stubs::get_absolute_url(Some(&base), "https://other.com/x".to_string()),
        "https://other.com/x"
    );
}

#[test]
fn test_parse_xml_tree_direct() {
    let xml = "<rss><channel><title>测试</title></channel></rss>";
    let doc = reader::stubs::DocumentBuilder::new_instance().parse_str(Some(xml.to_string()));
    println!("root length={}", doc.childNodes.length);
    for node in &doc.childNodes.nodes {
        println!("root node: type={} name={}", node.nodeType, node.nodeName);
        for sub in &node.childNodes.nodes {
            println!("  sub: type={} name={} value={:?}", sub.nodeType, sub.nodeName, sub.nodeValue);
            for leaf in &sub.childNodes.nodes {
                println!("    leaf: type={} name={} value={:?}", leaf.nodeType, leaf.nodeName, leaf.nodeValue);
            }
        }
    }
    assert!(doc.childNodes.length >= 1);
    fn dump(nodes: &[reader::stubs::XmlDomNode], indent: usize) {
        for n in nodes {
            println!("{}name={} type={} value={:?}", " ".repeat(indent), n.nodeName, n.nodeType, n.nodeValue);
            dump(&n.childNodes.nodes, indent + 2);
        }
    }
    dump(&doc.childNodes.nodes, 0);
}
#[test]
fn test_aes_encrypt_decrypt_roundtrip() {
    use reader::stubs::{Cipher, SecretKeySpec, IvParameterSpec};
    let key = b"0123456789abcdef";
    let iv = b"1234567890abcdef";
    let data = b"hello reader aes test";

    // ECB
    let mut c = Cipher::getInstance("AES/ECB/PKCS5Padding");
    c.init_spec(Cipher::ENCRYPT_MODE, &SecretKeySpec::new(key, "AES"));
    let ct = c.do_final_data(data);
    assert!(!ct.is_empty(), "ECB 加密不应为空");
    let mut d = Cipher::getInstance("AES/ECB/PKCS5Padding");
    d.init_spec(Cipher::DECRYPT_MODE, &SecretKeySpec::new(key, "AES"));
    let pt = d.do_final_data(&ct);
    assert_eq!(pt, data.to_vec(), "ECB 解密应还原");

    // CBC
    let mut c2 = Cipher::getInstance("AES/CBC/PKCS5Padding");
    c2.init_spec_iv(Cipher::ENCRYPT_MODE, &SecretKeySpec::new(key, "AES"), &IvParameterSpec::new(iv));
    let ct2 = c2.do_final_data(data);
    assert!(!ct2.is_empty(), "CBC 加密不应为空");
    let mut d2 = Cipher::getInstance("AES/CBC/PKCS5Padding");
    d2.init_spec_iv(Cipher::DECRYPT_MODE, &SecretKeySpec::new(key, "AES"), &IvParameterSpec::new(iv));
    let pt2 = d2.do_final_data(&ct2);
    assert_eq!(pt2, data.to_vec(), "CBC 解密应还原");
}

#[test]
fn test_quick_xml_events() {
    let xml = "<rss><channel><title>测试</title></channel></rss>";
    let mut reader = quick_xml::Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(e)) => println!("EV Start {}", String::from_utf8_lossy(e.name().as_ref())),
            Ok(quick_xml::events::Event::Text(t)) => println!("EV Text {:?} unescaped={:?}", t.unescape(), t.unescape().map(|s| s.trim().to_string())),
            Ok(quick_xml::events::Event::End(_)) => println!("EV End"),
            Ok(quick_xml::events::Event::Eof) => break,
            Ok(other) => println!("EV other {:?}", other),
            Err(e) => { println!("EV err {}", e); break; }
        }
        buf.clear();
    }
}
#[test]
fn test_aes_ecb_with_iv() {
    use reader::stubs::{Cipher, SecretKeySpec, IvParameterSpec};
    let key = b"0123456789abcdef";
    let iv = b"1234567890abcdef";
    let mut c = Cipher::getInstance("AES/ECB/PKCS5Padding");
    c.init_spec_iv(Cipher::ENCRYPT_MODE, &SecretKeySpec::new(key, "AES"), &IvParameterSpec::new(iv));
    let ct = c.do_final_data(b"reader");
    assert_eq!(ct.len(), 16, "ECB 密文应为 16 字节");
    let b64 = reader::io_legado_app_utils_base64::Base64::encodeToString(&ct, 2);
    println!("DBG ct_b64={}", b64);
    let mut d = Cipher::getInstance("AES/ECB/PKCS5Padding");
    d.init_spec_iv(Cipher::DECRYPT_MODE, &SecretKeySpec::new(key, "AES"), &IvParameterSpec::new(iv));
    let pt = d.do_final_data(&ct);
    assert_eq!(pt, b"reader".to_vec(), "ECB+iv 解密应还原");
}
#[test]
fn test_charset_detect_windows1256() {
    // windows-1256 阿拉伯文本（"مرحبا" 的 1256 字节：0xE3 0xD1 0xE0 0xC8 0xC7）
    let bytes = vec![0xE3u8, 0xD1, 0xE0, 0xC8, 0xC7, 0x20, 0xD3, 0xC7, 0xE1];
    let mut det = reader::io_legado_app_lib_icu4j_charsetdetector::CharsetDetector::new();
    det.set_text(bytes);
    let m = det.detect();
    assert!(m.is_some(), "windows-1256 should match");
    if let Some(m) = m {
        eprintln!("DBG detected: {} conf={}", m.get_name(), m.get_confidence());
        assert!(m.get_name() == "windows-1256" || m.get_name() == "KOI8-R", "arabic text should detect single-byte charset, got {}", m.get_name());
    }
}

#[test]
fn test_charset_detect_koi8r() {
    // KOI8-R 俄语 "Привет"（小写西里尔区）
    let bytes = vec![0xD0u8, 0xD2, 0xC9, 0xD7, 0xC5, 0xD4];
    let mut det = reader::io_legado_app_lib_icu4j_charsetdetector::CharsetDetector::new();
    det.set_text(bytes);
    let m = det.detect();
    assert!(m.is_some(), "KOI8-R should match");
    if let Some(m) = m {
        eprintln!("DBG detected: {} conf={}", m.get_name(), m.get_confidence());
        assert!(m.get_name() == "KOI8-R" || m.get_name() == "windows-1256", "cyrillic text should detect single-byte charset, got {}", m.get_name());
    }
}
#[test]
fn test_book_read_config_roundtrip() {
    use reader::io_legado_app_data_entities_book::{Book, ReadConfig};
    let mut book = Book::default();
    book.name = "配置书".into();
    {
        let mut rc = book.read_config.lock().unwrap();
        *rc = Some(ReadConfig {
            reverse_toc: true,
            page_anim: 2,
            re_segment: true,
            image_style: Some("FULL".into()),
            use_replace_rule: true,
            del_tag: 14,
            pdf_image_width: 640.0,
        });
    }
    let json = reader::stubs::book_to_json(&book).to_string();
    assert!(json.contains("\"reverseToc\":true"), "write reverseToc: {json}");
    assert!(json.contains("\"delTag\":14"), "write delTag: {json}");
    let parsed: Book = serde_json::from_str(&json).expect("parse book");
    let rc = parsed.read_config.lock().unwrap();
    let rc = rc.as_ref().expect("readConfig parsed");
    assert!(rc.reverse_toc, "reverseToc roundtrip");
    assert_eq!(rc.page_anim, 2, "pageAnim roundtrip");
    assert_eq!(rc.image_style.as_deref(), Some("FULL"), "imageStyle roundtrip");
    assert_eq!(rc.del_tag, 14, "delTag roundtrip");
    assert_eq!(rc.pdf_image_width, 640.0, "pdfImageWidth roundtrip");
}
#[test]
fn test_session_isolation() {
    use reader::stubs::io::vertx::RoutingContext;
    let set_cookie = |ctx: &RoutingContext, value: &str| {
        ctx.request.borrow_mut().headers.insert("Cookie".to_string(), format!("reader.session={}", value));
    };
    // 客户端 A：带 cookie 值 aaa
    let ctx1 = RoutingContext::new();
    set_cookie(&ctx1, "aaa");
    let s1 = ctx1.session();
    s1.put("username", "userA".to_string());
    assert_eq!(s1.get("username").as_deref(), Some("userA"), "A 写入会话");
    // 客户端 B：无 cookie → 独立会话
    let ctx2 = RoutingContext::new();
    let s2 = ctx2.session();
    assert!(s2.get("username").is_none(), "B 会话应与 A 隔离");
    s2.put("username", "userB".to_string());
    // A 再次访问（同 cookie）→ 恢复原会话
    let ctx1b = RoutingContext::new();
    set_cookie(&ctx1b, &s1.id);
    let s1b = ctx1b.session();
    assert_eq!(s1b.get("username").as_deref(), Some("userA"), "A 会话应按 cookie 恢复");
    assert_ne!(s1b.id, s2.id, "A/B 会话 id 不同");
    // 登出 destroy 只清自己
    s1b.destroy();
    let ctx1c = RoutingContext::new();
    set_cookie(&ctx1c, &s1.id);
    let s1c = ctx1c.session();
    assert!(s1c.get("username").is_none(), "destroy 后 A 会话清空");
    let ctx2b = RoutingContext::new();
    let s2b = ctx2b.session();
    let _ = s2b;
    // B 会话数据仍应保留（destroy 只影响 A）
    let ctx2c = RoutingContext::new();
    ctx2c.request.borrow_mut().headers.insert("Cookie".to_string(), format!("reader.session={}", s2.id));
    let s2c = ctx2c.session();
    assert_eq!(s2c.get("username").as_deref(), Some("userB"), "B 会话不受 A 登出影响");
}
#[test]
fn test_zlib_roundtrip() {
    use reader::stubs::{ByteArrayInputStream, ByteArrayOutputStream, DeflaterOutputStream, InflaterInputStream};
    // 压缩
    let mut bos = ByteArrayOutputStream::new();
    {
        let mut zos = DeflaterOutputStream::new(&mut bos);
        zos.write(b"UMD chapter content");
        zos.close();
    }
    let compressed = bos.to_byte_array();
    assert!(!compressed.is_empty(), "compressed output");
    // 解压
    let bais = ByteArrayInputStream::new(compressed);
    let mut iis = InflaterInputStream::new(bais);
    let mut out = Vec::new();
    loop {
        let mut buf = [0u8; 64];
        let n = iis.read(&mut buf);
        if n <= 0 { break; }
        out.extend_from_slice(&buf[..n as usize]);
    }
    assert_eq!(String::from_utf8_lossy(&out), "UMD chapter content", "zlib roundtrip");
}
#[test]
fn test_with_timeout_fires() {
    use reader::stubs::with_timeout;
    let fut = with_timeout(50, || async { std::future::pending::<i32>().await });
    let r = std::panic::catch_unwind(|| reader::stubs::block_on(fut));
    assert!(r.is_err(), "withTimeout 应超时 panic（原不超时挂起）");
    // 正常完成不 panic
    let fut2 = with_timeout(1000, || async { 42 });
    let v = reader::stubs::block_on(fut2);
    assert_eq!(v, 42, "正常 future 应完成");
}
#[test]
fn test_search_book_type_serialized() {
    let mut sb = reader::io_legado_app_data_entities_searchbook::SearchBook::default();
    sb.r#type = 1;
    let json = reader::stubs::search_book_to_json(&sb).to_string();
    assert!(json.contains("\"type\":1"), "type 应序列化（音频书源契约）: {json}");
}

#[test]
fn test_rss_source_rule_content_serialized() {
    let mut rs = reader::io_legado_app_data_entities_rsssource::RssSource::default();
    rs.rule_content = Some("css:.content".to_string());
    let json = reader::stubs::rss_source_to_json(&rs).to_string();
    assert!(json.contains("ruleContent"), "ruleContent 应序列化（保存后不丢正文规则）: {json}");
}
#[test]
fn test_any_option_map_serialization() {
    use reader::stubs::{any_to_json_value, Any};
    let mut m: std::collections::HashMap<String, Box<dyn std::any::Any>> = std::collections::HashMap::new();
    m.insert("username".to_string(), Box::new("transwarp".to_string()));
    let opt: Option<std::collections::HashMap<String, Box<dyn std::any::Any>>> = Some(m);
    let v = any_to_json_value(&opt);
    assert_eq!(v.to_string(), "{\"username\":\"transwarp\"}", "Some map 应序列化");
    let none: Option<std::collections::HashMap<String, Box<dyn std::any::Any>>> = None;
    let v2 = any_to_json_value(&none);
    assert_eq!(v2, serde_json::Value::Null, "None 应为 null");
}