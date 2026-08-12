use crate::prelude::*;
// package io.legado.app.data.entities

// data class SearchResult(
//     val resultCount: Int = 0,
//     val resultCountWithinChapter: Int = 0,
//     val resultText: String = "",
//     val chapterTitle: String = "",
//     val query: String = "",
//     val pageSize: Int = 0,
//     val chapterIndex: Int = 0,
//     val pageIndex: Int = 0,
//     val queryIndexInResult: Int = 0,
//     val queryIndexInChapter: Int = 0
// ) {
// }
pub struct SearchResult {
    pub result_count: i32,
    pub result_count_within_chapter: i32,
    pub result_text: String,
    pub chapter_title: String,
    pub query: String,
    pub page_size: i32,
    pub chapter_index: i32,
    pub page_index: i32,
    pub query_index_in_result: i32,
    pub query_index_in_chapter: i32,
}

impl Default for SearchResult {
    fn default() -> Self {
        SearchResult {
            result_count: 0,
            result_count_within_chapter: 0,
            result_text: String::new(),
            chapter_title: String::new(),
            query: String::new(),
            page_size: 0,
            chapter_index: 0,
            page_index: 0,
            query_index_in_result: 0,
            query_index_in_chapter: 0,
        }
    }
}
