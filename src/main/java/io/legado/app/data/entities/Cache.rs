use crate::prelude::*;
// package io.legado.app.data.entities

// @Entity(tableName = "caches", indices = [(Index(value = ["key"], unique = true))])
// data class Cache(
//     // @PrimaryKey
//     val key: String = "",
//     var value: String? = None,
//     var deadline: Long = 0L
// )
pub struct Cache {
    pub key: String,
    pub value: Option<String>,
    pub deadline: i64,
}

impl Default for Cache {
    fn default() -> Self {
        Cache {
            key: String::new(),
            value: None,
            deadline: 0,
        }
    }
}
