// package com.htmake.reader.entity

// data class BasicError(
//         val error: String,
//         val exception: String,
//         val message: String,
//         val path: String,
//         val status: Int,
//         val timestamp: Long
// )
pub struct BasicError {
    pub error: String,
    pub exception: String,
    pub message: String,
    pub path: String,
    pub status: i32,
    pub timestamp: i64,
}
