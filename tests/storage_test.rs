use reader::com_htmake_reader_utils_vertext;

#[test]
fn test_get_storage_file() {
    let f = reader::com_htmake_reader_utils_vertext::get_storage_file(
        &vec![String::from("data"), String::from("default"), String::from("bookSource")],
        ".json",
    );
    println!("file: {} exists={}", f.to_string(), f.exists());
    if f.exists() {
        let text = f.read_text();
        println!("content: {}", &text[..text.len().min(200)]);
        let arr = reader::com_htmake_reader_utils_vertext::parse_json_string_list(
            &f, None, None, 0, i32::MAX, None, None,
        );
        println!("parsed: {:?}", arr.map(|a| a.to_string()));
    }
}

#[test]
fn test_json_object_map() {
    let o = reader::stubs::JsonObject(r#"{"bookSourceName":"测试","bookSourceUrl":"http://x"}"#.to_string());
    let m = o.map();
    println!("map len: {} keys: {:?}", m.len(), m.keys());
}

#[test]
fn test_book_source_lookup() {
    let f = reader::com_htmake_reader_utils_vertext::get_storage_file(
        &vec![String::from("data"), String::from("default"), String::from("bookSource")],
        ".json",
    );
    if !f.exists() { println!("file missing"); return; }
    let parser = reader::stubs::ObjectMapper::new().factory().create_parser(&f);
    let mut result: Option<String> = None;
    if parser.next_token() == reader::stubs::JsonToken::START_ARRAY {
        while parser.next_token() != reader::stubs::JsonToken::END_ARRAY {
            if parser.current_token() != reader::stubs::JsonToken::START_OBJECT { continue; }
            let node: reader::stubs::JsonNode = parser.read_value_as_json_node();
            let url = node.get("bookSourceUrl").map(|n| n.to_string()).unwrap_or_default();
            println!("node url: <{}>", url);
            if url == "http://localhost:18999" { result = Some(node.to_string()); break; }
        }
    }
    println!("lookup result: {:?}", result.map(|s| s.len()));
}
