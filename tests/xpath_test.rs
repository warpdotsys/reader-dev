use reader::runtime::xpath;

const HTML: &str = r#"<html><body><div class="list">
  <div class="item" id="a1"><a href="https://x.com/1">书名一</a><span class="author">作者甲</span></div>
  <div class="item" id="a2"><a href="https://x.com/2">书名二</a><span class="author">作者乙</span></div>
</div></body></html>"#;

#[test]
fn test_xpath_elements() {
    let nodes = xpath::select_nodes(HTML, "//div[@class='item']").unwrap_or_default();
    assert_eq!(nodes.len(), 2);
    println!("OK elements: {}", nodes.len());
}

#[test]
fn test_xpath_child() {
    let nodes = xpath::select_nodes(HTML, "//div[@class='item']/a").unwrap_or_default();
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0].text, "书名一");
    println!("OK child: {}", nodes[0].text);
}

#[test]
fn test_xpath_attr() {
    let out = xpath::select_strings(HTML, "//a/@href").unwrap_or_default();
    assert!(out.contains(&"https://x.com/1".to_string()));
    println!("OK attr: {:?}", out);
}

#[test]
fn test_xpath_text() {
    let out = xpath::select_strings(HTML, "//div[@class='item']/a/text()").unwrap_or_default();
    assert_eq!(out, vec!["书名一".to_string(), "书名二".to_string()]);
    println!("OK text: {:?}", out);
}

#[test]
fn test_xpath_contains() {
    let nodes = xpath::select_nodes(HTML, "//div[contains(@class,'item')]").unwrap_or_default();
    assert_eq!(nodes.len(), 2);
    println!("OK contains: {}", nodes.len());
}

#[test]
fn test_xpath_descendant() {
    let nodes = xpath::select_nodes(HTML, "//div[@class='list']//a").unwrap_or_default();
    assert_eq!(nodes.len(), 2);
    println!("OK descendant: {}", nodes.len());
}
