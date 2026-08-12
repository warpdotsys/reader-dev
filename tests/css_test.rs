#[test]
fn test_css_content() {
    let html = r#"<html><body><div class="content">这是正文内容段落，来自 mock 书源。</div></body></html>"#;
    let els = reader::runtime::html::select_elements(html, "div.content");
    println!("count: {}", els.size());
    if let Some(e) = els.first() {
        println!("text: {}", e.text());
    }
}
