use crate::prelude::*;
// package io.legado.app.data.entities

//@Parcelize
//@Entity(tableName = "search_keywords", indices = [(Index(value = ["word"], unique = true))])
// data class SearchKeyword(
//    @PrimaryKey
//    var word: String = "",                      // 搜索关键词
//    var usage: Int = 1,                         // 使用次数
//    var lastUseTime: Long = System.currentTimeMillis()      // 最后一次使用时间
// )
pub struct SearchKeyword {
    pub word: String,                      // 搜索关键词
    pub usage: i32,                        // 使用次数
    pub last_use_time: i64,                // 最后一次使用时间
}

impl Default for SearchKeyword {
    fn default() -> Self {
        SearchKeyword {
            word: String::new(),
            usage: 1,
            last_use_time: System::current_time_millis(),
        }
    }
}
