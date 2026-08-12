use crate::prelude::*;
use crate::stubs::{Any, GSON, JsonPath, ReadContext};
// package io.legado.app.data.entities

// import com.fasterxml.jackson.annotation.JsonIgnoreProperties
// import com.jayway.jsonpath.DocumentContext
// import io.legado.app.model.DebugLog
// import io.legado.app.utils.GSON
// import io.legado.app.utils.jsonPath
// import io.legado.app.utils.readLong
// import io.legado.app.utils.readString

// @JsonIgnoreProperties("headerMap", "source", "userNameSpace")
pub struct HttpTTS {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub content_type: Option<String>,
    pub concurrent_rate: Option<String>,
    pub login_url: Option<String>,
    pub login_ui: Option<String>,
    pub header: Option<String>,
    pub js_lib: Option<String>,
    pub enabled_cookie_jar: Option<bool>,
    pub login_check_js: Option<String>,
    pub last_update_time: i64,

    // @Transient
    // private var _userNameSpace: String = ""
    pub user_name_space: String,

    // @Transient
    // private var debugLog: DebugLog? = None
    pub debug_log: Option<Box<dyn DebugLog>>,
}

impl HttpTTS {
    pub fn get_tag(&self) -> String {
        self.name.clone()
    }

    pub fn get_key(&self) -> String {
        format!("httpTts:{}", self.id)
    }

    pub fn set_user_name_space(&mut self, value: String) {
        self.user_name_space = value;
    }

    pub fn get_user_name_space(&self) -> String {
        self.user_name_space.clone()
    }

    pub fn set_logger(&mut self, value: Option<Box<dyn DebugLog>>) {
        self.debug_log = value;
    }

    pub fn get_logger(&self) -> Option<&dyn DebugLog> {
        self.debug_log.as_deref()
    }
}

impl Default for HttpTTS {
    fn default() -> Self {
        HttpTTS {
            id: System::current_time_millis(),
            name: String::new(),
            url: String::new(),
            content_type: None,
            concurrent_rate: Some("0".to_string()),
            login_url: None,
            login_ui: None,
            header: None,
            js_lib: None,
            enabled_cookie_jar: Some(false),
            login_check_js: None,
            last_update_time: System::current_time_millis(),
            user_name_space: String::new(),
            debug_log: None,
        }
    }
}

// companion object {
//     fun fromJsonDoc(doc: DocumentContext): Result<HttpTTS> = runCatching {
//         val loginUi = doc.read<Any>("$.loginUi")
//         HttpTTS(
//             id = doc.readLong("$.id") ?: System.currentTimeMillis(),
//             name = doc.readString("$.name")!!,
//             url = doc.readString("$.url")!!,
//             contentType = doc.readString("$.contentType"),
//             concurrentRate = doc.readString("$.concurrentRate"),
//             loginUrl = doc.readString("$.loginUrl"),
//             loginUi = if (loginUi is List<*>) GSON.toJson(loginUi) else loginUi?.toString(),
//             header = doc.readString("$.header"),
//             loginCheckJs = doc.readString("$.loginCheckJs")
//         )
//     }
//     fun fromJson(json: String): Result<HttpTTS> = runCatching {
//         fromJsonDoc(jsonPath.parse(json)).getOrThrow()
//     }
//     fun fromJsonArray(jsonArray: String): Result<List<HttpTTS>> = runCatching {
//         val list = jsonPath.parse(jsonArray).read<Any>("$") as List<*>
//         list.map { jsonItem ->
//             val doc = jsonPath.parse(jsonItem)
//             fromJsonDoc(doc).getOrThrow()
//         }
//     }
// }
impl HttpTTS {
    pub fn from_json_doc(doc: ReadContext) -> Result<HttpTTS, StubError> {
        // runCatching { ... }
        (|| -> Result<HttpTTS, StubError> {
            let login_ui = doc.read::<Any>("$.loginUi").unwrap_or(Any::Null);
            let mut tts = HttpTTS {
                id: doc.read_long("$.id").unwrap_or_else(System::current_time_millis),
                name: doc.read_string("$.name").expect("name"),
                url: doc.read_string("$.url").expect("url"),
                content_type: doc.read_string("$.contentType"),
                concurrent_rate: doc.read_string("$.concurrentRate"),
                login_url: doc.read_string("$.loginUrl"),
                login_ui: if login_ui.is_list() { Some(GSON::to_json(&login_ui)) } else { login_ui.to_string_opt() },
                header: doc.read_string("$.header"),
                login_check_js: doc.read_string("$.loginCheckJs"),
                ..HttpTTS::default()
            };
            Ok(tts)
        })()
    }

    pub fn from_json(json: String) -> Result<HttpTTS, StubError> {
        // runCatching { fromJsonDoc(jsonPath.parse(json)).getOrThrow() }
        (|| -> Result<HttpTTS, StubError> {
            Self::from_json_doc(JsonPath::parse(json))
        })()
    }

    pub fn from_json_array(json_array: String) -> Result<Vec<HttpTTS>, StubError> {
        // runCatching {
        //     val list = jsonPath.parse(jsonArray).read<Any>("$") as List<*>
        //     list.map { jsonItem ->
        //         val doc = jsonPath.parse(jsonItem)
        //         fromJsonDoc(doc).getOrThrow()
        //     }
        // }
        (|| -> Result<Vec<HttpTTS>, StubError> {
            let list = JsonPath::parse(json_array).read::<Vec<Any>>("$").unwrap_or_default();
            let mut result = Vec::new();
            for json_item in list {
                let doc = JsonPath::parse(json_item);
                result.push(Self::from_json_doc(doc)?);
            }
            Ok(result)
        })()
    }
}
