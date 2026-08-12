use reader::stubs::{SCRIPT_ENGINE, SimpleBindings, Any};

#[test]
fn test_exact_eval_js_bindings() {
    let mut b = SimpleBindings::new();
    b.set("java", "default".to_string());
    b.set("baseUrl", "http://localhost:18999".to_string());
    b.set("cookie", "".to_string());
    b.set("cache", Some("".to_string()));
    b.set("page", Some(1i32));
    b.set("key", Some("测试".to_string()));
    b.set("speakText", None::<String>);
    b.set("speakSpeed", None::<i32>);
    b.set("book", Some("default".to_string()));
    b.set("source", Some("http://localhost:18999".to_string()));
    b.set("result", None::<String>);
    let r = SCRIPT_ENGINE.eval("key".to_string(), &mut b);
    let out = r.map(|v| {
        v.as_any()
            .downcast_ref::<Any>()
            .map(|a| reader::stubs::any_to_value(a).to_string())
            .unwrap_or_default()
    });
    println!("eval key -> {:?}", out);
}

#[test]
fn test_type_name_via_script_engine() {
    let mut b = SimpleBindings::new();
    b.set("key", Some("测试".to_string()));
    let r = SCRIPT_ENGINE.eval("key".to_string(), &mut b);
    match &r {
        Some(v) => {
            println!("type: {}", std::any::type_name_of_val(&**v));
            println!("as_any downcast: {}", v.as_any().downcast_ref::<Any>().is_some());
            let boxed_any: Box<dyn std::any::Any> = v.as_any().downcast_ref::<Any>().map(|a| Box::new(a.clone())).unwrap();
            println!("boxed any type: {}", std::any::type_name_of_val(&*boxed_any));
        }
        None => println!("none"),
    }
}
