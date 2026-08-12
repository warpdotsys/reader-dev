use crate::prelude::*;
// package io.legado.app.data.entities

//@Parcelize
//@Entity(tableName = "book_groups")
// data class BookGroup(
//        @PrimaryKey
//        var groupId: Long = 0L,
//        var groupName: String = "",
//        var cover: String? = None,
//        var order: Int = 0,
//        var show: Boolean = true
// )
pub struct BookGroup {
    pub group_id: i64,
    pub group_name: String,
    pub cover: Option<String>,
    pub order: i32,
    pub show: bool,
}

impl Default for BookGroup {
    fn default() -> Self {
        BookGroup {
            group_id: 0,
            group_name: String::new(),
            cover: None,
            order: 0,
            show: true,
        }
    }
}
