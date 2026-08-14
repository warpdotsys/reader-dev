use reader::stubs::JsonPath;

#[test]
fn test_query_books() {
    let json = r#"{"books":[{"name":"测试之书","author":"作者甲","bookUrl":"http://localhost:18999/book/1","intro":"一本用于测试的书","coverUrl":"http://localhost:18999/cover.jpg"},{"name":"测试第二本","author":"作者乙","bookUrl":"http://localhost:18999/book/2","intro":"第二本测试书"}]}"#;
    let v = reader::runtime::json_path::query(json, "$.books");
    println!("query: {:?}", v.as_ref().map(|v| v.as_array().map(|a| a.len())));
    let ctx = JsonPath::parse(json.to_string());
    let list: Result<Vec<reader::stubs::Any>, _> = ctx.read("$.books");
    println!("read vec: {:?}", list.as_ref().map(|l| l.len()));
}

#[test]
fn test_recursive_and_numeric_filter() {
    let json = r#"{"store":{"book":[{"title":"A","price":10},{"title":"B","price":20}],"extra":{"title":"C"}}}"#;
    // $..title 递归收集所有 title
    let r = reader::runtime::json_path::query(json, "$..title");
    let s = r.map(|v| v.to_string()).unwrap_or_default();
    assert!(s.contains("\"A\"") && s.contains("\"B\"") && s.contains("\"C\""), "recursive: {s}");
    // 数值过滤 [?(@.price==20)]
    let r = reader::runtime::json_path::query(json, "$.store.book[?(@.price==20)]");
    let s = r.map(|v| v.to_string()).unwrap_or_default();
    assert!(s.contains("\"B\"") && !s.contains("\"A\""), "numeric filter: {s}");
    // 字符串过滤保持
    let r = reader::runtime::json_path::query(json, "$.store.book[?(@.title=='A')]");
    let s = r.map(|v| v.to_string()).unwrap_or_default();
    assert!(s.contains("\"A\""), "string filter: {s}");
}