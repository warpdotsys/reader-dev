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

#[test]
fn test_xpath_index() {
    let nodes = xpath::select_nodes(HTML, "//div[@class='item'][1]").unwrap_or_default();
    assert_eq!(nodes.len(), 1);
    assert!(nodes[0].text.contains("书名一"), "索引[1]应取第一项: {:?}", nodes[0].text);
    println!("OK index: {}", nodes[0].text);
}

#[test]
fn test_xpath_and_or_predicates() {
    let nodes = xpath::select_nodes(HTML, "//div[@class='item' and @id='a2']").unwrap_or_default();
    assert_eq!(nodes.len(), 1);
    assert!(nodes[0].text.contains("书名二"));
    // or
    let nodes2 = xpath::select_nodes(HTML, "//span[@class='author' or @class='none']").unwrap_or_default();
    assert_eq!(nodes2.len(), 2);
    println!("OK and/or predicates");
}

#[test]
fn test_xpath_attr_exists_and_text_contains() {
    let nodes = xpath::select_nodes(HTML, "//a[@href]").unwrap_or_default();
    assert_eq!(nodes.len(), 2);
    // text() 为直接文本子节点（XPath 语义）
    let html_dt = "<div id='outer'><div id='a'>书名二直接文本</div><div id='b'><span>书名二嵌套</span></div></div>";
    let nodes2 = xpath::select_nodes(html_dt, "//div[contains(text(),'书名二')]").unwrap_or_default();
    assert_eq!(nodes2.len(), 1, "仅直接文本匹配: {:?}", nodes2.iter().map(|n| n.text.clone()).collect::<Vec<_>>());
    println!("OK attr exists + text contains");
}

#[test]
fn test_xpath_position_and_last() {
    let nodes = xpath::select_nodes(HTML, "//div[@class='item'][position()=2]").unwrap_or_default();
    assert_eq!(nodes.len(), 1);
    assert!(nodes[0].text.contains("书名二"));
    let nodes2 = xpath::select_nodes(HTML, "//div[@class='item'][last()]").unwrap_or_default();
    assert_eq!(nodes2.len(), 1);
    assert!(nodes2[0].text.contains("书名二"));
    println!("OK position/last");
}

#[test]
fn test_xpath_numeric_compare() {
    let html2 = "<ul><li data-n='1'>一</li><li data-n='3'>三</li></ul>";
    let nodes = xpath::select_nodes(html2, "//li[@data-n>1]").unwrap_or_default();
    assert_eq!(nodes.len(), 1);
    assert!(nodes[0].text.contains("三"));
    println!("OK numeric compare");
}

#[test]
fn test_xpath_attr_not_eq() {
    let nodes = xpath::select_nodes(HTML, "//div[@class!='none']").unwrap_or_default();
    assert!(nodes.len() >= 1);
    println!("OK != predicate");
}