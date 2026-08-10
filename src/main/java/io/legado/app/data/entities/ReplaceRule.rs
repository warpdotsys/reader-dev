// package io.legado.app.data.entities

// import com.fasterxml.jackson.annotation.JsonProperty;

//@Parcelize
//@Entity(
//    tableName = "replace_rules",
//    indices = [(Index(value = ["id"]))]
//)
// data class ReplaceRule(
//    @PrimaryKey(autoGenerate = true)
//    var id: Long = System.currentTimeMillis(),
//    var name: String = "",
//    var group: String? = null,
//    var pattern: String = "",
//    var replacement: String = "",
//    var scope: String? = null,
//    var scopeTitle: Boolean = false,
//    var scopeContent: Boolean = true,
//    @get:JsonProperty("isEnabled") var isEnabled: Boolean = true,
//    @get:JsonProperty("isRegex") var isRegex: Boolean = false,
//    var timeoutMillisecond: Long = 3000L,
//    @ColumnInfo(name = "sortOrder")
//    var order: Int = 0
// )
pub struct ReplaceRule {
    pub id: i64,
    pub name: String,
    pub group: Option<String>,
    pub pattern: String,
    pub replacement: String,
    pub scope: Option<String>,
    pub scope_title: bool,
    pub scope_content: bool,
    // @get:JsonProperty("isEnabled")
    pub is_enabled: bool,
    // @get:JsonProperty("isRegex")
    pub is_regex: bool,
    pub timeout_millisecond: i64,
    //    @ColumnInfo(name = "sortOrder")
    pub order: i32,
}

impl Default for ReplaceRule {
    fn default() -> Self {
        ReplaceRule {
            id: System::current_time_millis(),
            name: String::new(),
            group: None,
            pattern: String::new(),
            replacement: String::new(),
            scope: None,
            scope_title: false,
            scope_content: true,
            is_enabled: true,
            is_regex: false,
            timeout_millisecond: 3000,
            order: 0,
        }
    }
}
