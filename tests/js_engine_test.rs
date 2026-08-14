use reader::stubs::{Any, SimpleBindings, SCRIPT_ENGINE};

fn eval(js: &str, bindings: &mut SimpleBindings) -> Option<Any> {
    SCRIPT_ENGINE
        .eval(js.to_string(), bindings)
        .and_then(|v| v.as_any().downcast_ref::<Any>().cloned())
}

#[test]
fn test_js_arithmetic() {
    let mut b = SimpleBindings::new();
    let r = eval("1 + 2 * 3", &mut b).unwrap();
    match r {
        Any::Long(7) => println!("OK arithmetic: 7"),
        other => panic!("expected 7, got {:?}", other),
    }
}

#[test]
fn test_js_bindings() {
    let mut b = SimpleBindings::new();
    b.put("result", Any::Str("hello".to_string()));
    b.put("baseUrl", "https://example.com".to_string());
    let r = eval("result + \" world\"", &mut b).unwrap();
    match r {
        Any::Str(s) => assert_eq!(s, "hello world"),
        other => panic!("expected string, got {:?}", other),
    }
}

#[test]
fn test_js_object_result() {
    let mut b = SimpleBindings::new();
    let r = eval("({a: 1, b: 'x', c: [1, 2]})", &mut b).unwrap();
    match &r {
        Any::Map(m) => {
            assert!(m.contains_key("a"));
            assert!(m.contains_key("b"));
            println!("OK object: {:?}", m.keys());
        }
        other => panic!("expected map, got {:?}", other),
    }
}

#[test]
fn test_js_json_parse() {
    let mut b = SimpleBindings::new();
    let r = eval("JSON.parse('{\"a\":5}').a", &mut b).unwrap();
    match r {
        Any::Long(5) => println!("OK json parse"),
        other => panic!("expected 5, got {:?}", other),
    }
}

#[test]
fn test_js_string_functions() {
    let mut b = SimpleBindings::new();
    let r = eval("'a,b,c'.split(',').length", &mut b).unwrap();
    match r {
        Any::Long(3) => println!("OK split"),
        other => panic!("expected 3, got {:?}", other),
    }
}

#[test]
fn test_js_ajax_function_exists() {
    let mut b = SimpleBindings::new();
    let r = eval("typeof java.ajax", &mut b).unwrap();
    match r {
        Any::Str(s) => {
            assert_eq!(s, "function");
            println!("OK java.ajax exists");
        }
        other => panic!("expected 'function', got {:?}", other),
    }
}

#[test]
fn test_js_syntax_error() {
    let mut b = SimpleBindings::new();
    let r = eval("this is not valid js {{{", &mut b);
    assert!(r.is_none());
    println!("OK syntax error -> None");
}

#[test]
fn test_java_obj_methods() {
    use reader::stubs::SimpleBindings;
    let b = SimpleBindings::new();
    // base64 + md5 + aes
    let r = reader::runtime::js::eval_js_script("java.base64Encode('hello');", &b);
    assert!(r.is_some());
    let out = format!("{}", r.unwrap());
    assert!(out.contains("aGVsbG8="), "base64Encode: {}", out);
    let r = reader::runtime::js::eval_js_script("java.md5Encode('abc');", &b);
    let out = format!("{}", r.unwrap());
    assert_eq!(out, "900150983cd24fb0d6963f7d28e17f72", "md5Encode: {}", out);
    let r = reader::runtime::js::eval_js_script("java.md5Encode16('abc');", &b);
    let out = format!("{}", r.unwrap());
    assert_eq!(out, "3cd24fb0d6963f7d", "md5Encode16: {}", out);
    // aes 回环
    let r = reader::runtime::js::eval_js_script("var e = java.aesEncodeToBase64String('reader', '0123456789abcdef', 'AES/ECB/PKCS5Padding', '1234567890abcdef'); e + '|' + java.aesBase64DecodeToString(e, '0123456789abcdef', 'AES/ECB/PKCS5Padding', '1234567890abcdef');", &b);
    let out = format!("{}", r.unwrap());
    assert!(out.contains("reader"), "aes roundtrip: {}", out);
    println!("OK java obj methods");
}