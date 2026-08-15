use reader::io_legado_app_data_entities_book::Book;
use reader::io_legado_app_data_entities_booksource::BookSource;
use reader::io_legado_app_model_analyzerule_analyzerule::AnalyzeRule;
use reader::stubs::{SCRIPT_ENGINE, SimpleBindings};

#[test]
fn test_analyze_rule_book_variables() {
    let mut book = Book::default();
    book.name = "测试书名".into();
    book.author = "测试作者".into();
    book.book_url = "http://x.com/book/1".into();
    let bs = BookSource::default();
    let mut ar = AnalyzeRule::new(&book, Some(&bs), None::<&dyn reader::io_legado_app_model_debuglog::DebugLog>);
    let v = ar.get("bookName".to_string());
    assert_eq!(v, "测试书名", "bookName 变量应提取自 ruleData");
    let v = ar.get("bookAuthor".to_string());
    assert_eq!(v, "测试作者");
    let v = ar.get("bookUrl".to_string());
    assert_eq!(v, "http://x.com/book/1");
}

#[test]
fn test_analyze_rule_get_var_in_js() {
    let mut book = Book::default();
    book.name = "书名甲".into();
    book.author = "作者乙".into();
    book.book_url = "http://x.com/b/2".into();
    let bs = BookSource::default();
    let mut ar = AnalyzeRule::new(&book, Some(&bs), None::<&dyn reader::io_legado_app_model_debuglog::DebugLog>);
    let mut bindings = SimpleBindings::new();
    bindings.put("bookName", "书名甲".to_string());
    let r = ar
        .eval_js("java.getString('@get:name')".to_string(), None)
        .map(|v| v.to_string())
        .unwrap_or_default();
    assert_eq!(r, "书名甲", "@get:{{name}} 应经 get_variable 提取");
}

#[test]
fn test_analyze_rule_js_book_binding() {
    let mut book = Book::default();
    book.name = "绑定书名".into();
    book.book_url = "http://x.com/b/3".into();
    let bs = BookSource::default();
    let mut ar = AnalyzeRule::new(&book, Some(&bs), None::<&dyn reader::io_legado_app_model_debuglog::DebugLog>);
    let r = ar
        .eval_js("book.name + '|' + source.bookSourceUrl".to_string(), None)
        .map(|v| v.to_string())
        .unwrap_or_default();
    assert_eq!(r, "绑定书名|", "book 绑定应包含 name（source 绑定真实书源 JSON 含 bookSourceUrl）");
}

#[test]
fn test_script_engine_in_lib_context() {
    let mut b = SimpleBindings::new();
    b.put("x", "42".to_string());
    let r = SCRIPT_ENGINE.eval_downcast_any("x + 1".to_string(), &mut b);
    assert!(r.is_some());
}
