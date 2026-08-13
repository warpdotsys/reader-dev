use crate::prelude::*;
// package io.legado.app.data.entities

//@Parcelize
//@Entity(tableName = "bookmarks", indices = [(Index(value = ["bookUrl"], unique = true))])
// data class Bookmark(
//    @PrimaryKey
//    val time: Long = System.currentTimeMillis(),
//    val bookName: String = "",
//    val bookAuthor: String = "",
//    var chapterIndex: Int = 0,
//    var chapterPos: Int = 0,
//    var chapterName: String = "",
//    var bookText: String = "",
//    var content: String = ""
// )
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Bookmark {
    pub time: i64,
    pub book_name: String,
    pub book_author: String,
    pub chapter_index: i32,
    pub chapter_pos: i32,
    pub chapter_name: String,
    pub book_text: String,
    pub content: String,
}

impl Default for Bookmark {
    fn default() -> Self {
        Bookmark {
            time: System::current_time_millis(),
            book_name: String::new(),
            book_author: String::new(),
            chapter_index: 0,
            chapter_pos: 0,
            chapter_name: String::new(),
            book_text: String::new(),
            content: String::new(),
        }
    }
}

