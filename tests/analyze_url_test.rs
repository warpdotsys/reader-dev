use reader::io_legado_app_model_analyzerule_analyzeurl::AnalyzeUrl;
use reader::stubs::{SCRIPT_ENGINE, SimpleBindings, Any};

#[test]
fn test_script_engine_in_lib_context() {
    let mut b = SimpleBindings::new();
    b.set("key", Some("测试".to_string()));
    let r = SCRIPT_ENGINE.eval("key".to_string(), &mut b);
    println!("downcast: {:?}", r.as_ref().map(|v| v.as_any().downcast_ref::<Any>().is_some()));
}

#[test]
fn test_analyze_url_eval_js() {
    let au = AnalyzeUrl::new(
        "http://localhost:18999/search?key={{key}}&page={{page}}".to_string(),
        Some("测试".to_string()),
        Some(1),
        None,
        None,
        "http://localhost:18999".to_string(),
        None,
        None,
        None,
        None,
        None,
    );
    let r = au.eval_js("key".to_string(), None);
    println!("eval key -> {:?}", r.as_ref().map(|v| v.to_string()));
}

#[test]
fn test_param_pattern() {
    let p = reader::stubs::Pattern::compile(r"\s*,\s*(?=\{)");
    let s = "http://localhost:18999/search?key=测试&page=1";
    println!("match: {}", p.find(s).is_some());
    let s2 = "http://x.com,{body}";
    println!("match2: {}", p.find(s2).is_some());
}

#[test]
fn test_regex_direct() {
    let re = regex::Regex::new(r"\s*,\s*(?=\{)").unwrap();
    println!("direct1: {}", re.is_match("http://localhost:18999/search?key=测试&page=1"));
    println!("direct2: {}", re.is_match("http://x.com,{body}"));
}
