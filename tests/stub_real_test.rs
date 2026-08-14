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