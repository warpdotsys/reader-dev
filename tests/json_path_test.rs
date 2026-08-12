use reader::stubs::{Any, ReadContext, JsonPath};

fn ctx(json: &str) -> ReadContext {
    JsonPath::parse(json.to_string())
}

const JSON: &str = r#"{"code":200,"data":{"list":[{"name":"书A","author":"甲","tags":["奇幻","热血"]},{"name":"书B","author":"乙","tags":["都市"]}],"total":2},"msg":"ok"}"#;

#[test]
fn test_json_path_root() {
    let c = ctx(JSON);
    let v: i64 = c.read("$.code").unwrap();
    assert_eq!(v, 200);
    println!("OK root: {}", v);
}

#[test]
fn test_json_path_field() {
    let c = ctx(JSON);
    let v: String = c.read("$.data.list[0].name").unwrap();
    assert_eq!(v, "书A");
    println!("OK field: {}", v);
}

#[test]
fn test_json_path_negative_index() {
    let c = ctx(JSON);
    let v: String = c.read("$.data.list[-1].name").unwrap();
    assert_eq!(v, "书B");
    println!("OK negative index: {}", v);
}

#[test]
fn test_json_path_array_field() {
    let c = ctx(JSON);
    let v: String = c.read("$.data.list[1].tags[0]").unwrap();
    assert_eq!(v, "都市");
    println!("OK array: {}", v);
}

#[test]
fn test_json_path_list_to_any() {
    let c = ctx(JSON);
    let v: Any = c.read("$.data.list[*].name").unwrap();
    match v {
        Any::List(l) => {
            assert_eq!(l.len(), 2);
            println!("OK list: {:?}", l);
        }
        other => panic!("expected List, got {:?}", other),
    }
}

#[test]
fn test_json_path_filter() {
    let c = ctx(JSON);
    let v: Any = c.read(r#"$.data.list[?(@.name=="书B")]"#).unwrap();
    match v {
        Any::List(l) => assert_eq!(l.len(), 1),
        other => panic!("expected filtered List, got {:?}", other),
    }
    println!("OK filter");
}

#[test]
fn test_json_path_bracket_field() {
    let c = ctx(JSON);
    let v: String = c.read("$['data']['list'][0]['name']").unwrap();
    assert_eq!(v, "书A");
    println!("OK bracket field: {}", v);
}

#[test]
fn test_json_path_not_found() {
    let c = ctx(JSON);
    assert!(c.read::<String>("$.data.missing").is_err());
    println!("OK not found -> Err");
}
