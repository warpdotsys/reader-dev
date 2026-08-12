use reader::stubs::{Element, Jsoup};

const HTML: &str = r#"<html><head><title>测试页面</title></head><body>
<div id="list">
  <a class="book" href="https://a.com/1" title="书一">书名一</a>
  <a class="book" href="https://a.com/2" title="书二">书名二</a>
  <p class="desc">简介内容</p>
</div>
</body></html>"#;

#[test]
fn test_jsoup_select() {
    let doc = Jsoup::parse(HTML.to_string());
    let books = doc.select(".book");
    assert_eq!(books.size(), 2, "should find 2 .book elements");
    let first = books.first().unwrap();
    assert_eq!(first.attr("href"), "https://a.com/1");
    assert_eq!(first.attr("title"), "书一");
    assert_eq!(first.text(), "书名一");
    println!("OK select: {}", books.size());
}

#[test]
fn test_jsoup_text() {
    let doc = Jsoup::parse(HTML.to_string());
    let desc = doc.select(".desc").first().unwrap();
    assert_eq!(desc.text(), "简介内容");
    println!("OK text: {}", desc.text());
}

#[test]
fn test_jsoup_tag_and_outer() {
    let doc = Jsoup::parse(HTML.to_string());
    let a = doc.select("a.book").first().unwrap();
    assert_eq!(a.tag_name(), "a");
    assert!(a.outer_html().contains("href=\"https://a.com/1\""));
    println!("OK tag/outer: {}", a.tag_name());
}

#[test]
fn test_jsoup_nested_select() {
    let doc = Jsoup::parse(HTML.to_string());
    let div = doc.select("#list").first().unwrap();
    let nested = div.select("a.book");
    assert_eq!(nested.size(), 2);
    println!("OK nested: {}", nested.size());
}

#[test]
fn test_jsoup_body() {
    let doc = Jsoup::parse(HTML.to_string());
    let body = doc.body();
    assert!(body.html().contains("list"));
    println!("OK body");
}

#[test]
fn test_jsoup_title() {
    let doc = Jsoup::parse(HTML.to_string());
    assert_eq!(doc.title(), "测试页面");
    println!("OK title: {}", doc.title());
}
