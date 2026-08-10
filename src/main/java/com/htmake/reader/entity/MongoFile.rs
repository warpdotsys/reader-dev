// package com.htmake.reader.entity

// data class MongoFile(
//     var path: String = "",
//     var content: String = "",
//     var created_at: Long = System.currentTimeMillis(),
//     var updated_at: Long = System.currentTimeMillis()
// )
pub struct MongoFile {
    pub path: String,
    pub content: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Default for MongoFile {
    fn default() -> Self {
        MongoFile {
            path: String::new(),
            content: String::new(),
            created_at: System::current_time_millis(),
            updated_at: System::current_time_millis(),
        }
    }
}
