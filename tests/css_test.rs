#[test]
fn test_css_content() {
    let html = r#"<html><body><div class="content">这是正文内容段落，来自 mock 书源。</div></body></html>"#;
    let els = reader::runtime::html::select_elements(html, "div.content");
    println!("count: {}", els.size());
    if let Some(e) = els.first() {
        println!("text: {}", e.text());
    }
}

#[test]
fn test_jsoup_pseudo_classes() {
    use reader::stubs::Element;
    let html = "<ul><li class='item'><a>苹果</a></li><li class='item'><a>香蕉</a></li><li class='item'><a>橘子</a></li></ul>";
    // :contains
    let els = reader::runtime::html::select_elements(html, "li:contains('香蕉')");
    assert_eq!(els.list.len(), 1, "contains 应匹配 1 个");
    assert!(els.list[0].text.contains("香蕉"), "contains 文本: {}", els.list[0].text);
    // :first / :last
    let els = reader::runtime::html::select_elements(html, "li:first");
    assert_eq!(els.list.len(), 1);
    assert!(els.list[0].text.contains("苹果"));
    let els = reader::runtime::html::select_elements(html, "li:last");
    assert!(els.list[0].text.contains("橘子"));
    // :eq
    let els = reader::runtime::html::select_elements(html, "li:eq(1)");
    assert_eq!(els.list.len(), 1);
    assert!(els.list[0].text.contains("香蕉"));
    // :gt/:lt
    let els = reader::runtime::html::select_elements(html, "li:gt(0)");
    assert_eq!(els.list.len(), 2);
    let els = reader::runtime::html::select_elements(html, "li:lt(2)");
    assert_eq!(els.list.len(), 2);
    println!("OK jsoup pseudo classes");
}

#[test]
fn test_gbk_decode() {
    // GBK 编码 "中文"（gb2312 字节）
    let gbk = [0xd6, 0xd0, 0xce, 0xc4];
    let text = reader::io_legado_app_help_http_okhttputils::decode_bytes_with_charset(&gbk, "GBK");
    assert_eq!(text, "中文", "GBK 解码: {:?}", text);
    println!("OK gbk decode");
}