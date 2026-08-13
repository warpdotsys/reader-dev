use crate::prelude::*;
// package io.legado.app.data.entities.rule

// data class BookInfoRule(
//     var init: String? = None,
//     var name: String? = None,
//     var author: String? = None,
//     var intro: String? = None,
//     var kind: String? = None,
//     var lastChapter: String? = None,
//     var updateTime: String? = None,
//     var coverUrl: String? = None,
//     var tocUrl: String? = None,
//     var wordCount: String? = None,
//     var canReName: String? = None
// )
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookInfoRule {
    pub init: Option<String>,
    pub name: Option<String>,
    pub author: Option<String>,
    pub intro: Option<String>,
    pub kind: Option<String>,
    pub last_chapter: Option<String>,
    pub update_time: Option<String>,
    pub cover_url: Option<String>,
    pub toc_url: Option<String>,
    pub word_count: Option<String>,
    pub can_re_name: Option<String>,
}

impl Default for BookInfoRule {
    fn default() -> Self {
        BookInfoRule {
            init: None,
            name: None,
            author: None,
            intro: None,
            kind: None,
            last_chapter: None,
            update_time: None,
            cover_url: None,
            toc_url: None,
            word_count: None,
            can_re_name: None,
        }
    }
}

impl PartialEq for BookInfoRule {
    fn eq(&self, other: &Self) -> bool {
        self.init == other.init
            && self.name == other.name
            && self.author == other.author
            && self.intro == other.intro
            && self.kind == other.kind
            && self.last_chapter == other.last_chapter
            && self.update_time == other.update_time
            && self.cover_url == other.cover_url
            && self.toc_url == other.toc_url
            && self.word_count == other.word_count
            && self.can_re_name == other.can_re_name
    }
}

impl Eq for BookInfoRule {}
