

#[test]
fn test_book_source_from_json() {
    let json = r#"{"bookSourceUrl":"http://localhost:18999","bookSourceName":"本地测试书源","bookSourceType":0,"searchUrl":"http://localhost:18999/search?key={{key}}&page={{page}}","ruleSearch":{"bookList":"$.books","name":"$.name","author":"$.author","bookUrl":"$.bookUrl","intro":"$.intro","coverUrl":"$.coverUrl"},"ruleBookInfo":{"name":"$.name","author":"$.author","intro":"$.intro","tocUrl":"$.tocUrl"},"ruleToc":{"chapterList":"$.chapters","chapterName":"$.title","chapterUrl":"$.url"},"ruleContent":{"content":"div.content"}}"#;
    match reader::io_legado_app_data_entities_booksource::BookSource::from_json(json.to_string()) {
        Ok(b) => {
            println!("searchUrl: {:?}", b.search_url);
            println!("ruleSearch.bookList: {:?}", b.rule_search.as_ref().map(|r| r.book_list.clone()));
            println!("ruleToc.chapterList: {:?}", b.rule_toc.as_ref().map(|r| r.chapter_list.clone()));
        }
        Err(e) => println!("FAILED: {}", reader::stubs::panic_message(e)),
    }
}
