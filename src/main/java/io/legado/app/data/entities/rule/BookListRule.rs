use crate::prelude::*;
// package io.legado.app.data.entities.rule

// interface BookListRule {
//     var bookList: String?
//     var name: String?
//     var author: String?
//     var intro: String?
//     var kind: String?
//     var lastChapter: String?
//     var updateTime: String?
//     var bookUrl: String?
//     var coverUrl: String?
//     var wordCount: String?
// }
pub trait BookListRule {
    fn book_list(&self) -> Option<&str>;
    fn set_book_list(&mut self, value: Option<String>);
    fn name(&self) -> Option<&str>;
    fn set_name(&mut self, value: Option<String>);
    fn author(&self) -> Option<&str>;
    fn set_author(&mut self, value: Option<String>);
    fn intro(&self) -> Option<&str>;
    fn set_intro(&mut self, value: Option<String>);
    fn kind(&self) -> Option<&str>;
    fn set_kind(&mut self, value: Option<String>);
    fn last_chapter(&self) -> Option<&str>;
    fn set_last_chapter(&mut self, value: Option<String>);
    fn update_time(&self) -> Option<&str>;
    fn set_update_time(&mut self, value: Option<String>);
    fn book_url(&self) -> Option<&str>;
    fn set_book_url(&mut self, value: Option<String>);
    fn cover_url(&self) -> Option<&str>;
    fn set_cover_url(&mut self, value: Option<String>);
    fn word_count(&self) -> Option<&str>;
    fn set_word_count(&mut self, value: Option<String>);
}
