// package io.legado.app.data.entities

// @Entity(tableName = "cookies", indices = [(Index(value = ["url"], unique = true))])
// data class Cookie(
//     // @PrimaryKey
//     var url: String = "",
//     var cookie: String = ""
// )
pub struct Cookie {
    pub url: String,
    pub cookie: String,
}

impl Default for Cookie {
    fn default() -> Self {
        Cookie {
            url: String::new(),
            cookie: String::new(),
        }
    }
}
