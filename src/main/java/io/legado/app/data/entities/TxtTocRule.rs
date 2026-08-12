use crate::prelude::*;
// package io.legado.app.data.entities

// import androidx.room.Entity
// import androidx.room.PrimaryKey

// @Entity(tableName = "txtTocRules")
// data class TxtTocRule(
//     // @PrimaryKey
//     var id: Long = System.currentTimeMillis(),
//     var name: String = "",
//     var rule: String = "",
//     var serialNumber: Int = -1,
//     var enable: Boolean = true
// )
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct TxtTocRule {
    pub id: i64,
    pub name: String,
    pub rule: String,
    pub serial_number: i32,
    pub enable: bool,
}

impl Default for TxtTocRule {
    fn default() -> Self {
        TxtTocRule {
            id: System::current_time_millis(),
            name: String::new(),
            rule: String::new(),
            serial_number: -1,
            enable: true,
        }
    }
}
