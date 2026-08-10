pub struct AppConst;

impl AppConst {
    pub const UA_NAME: &'static str = "User-Agent";

    pub fn userAgent() -> String {
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/75.0.3770.142 Safari/537.36".to_string()
    }

    pub fn SCRIPT_ENGINE() -> RhinoScriptEngine {
        RhinoScriptEngine::new()
    }

    pub fn TIME_FORMAT() -> SimpleDateFormat {
        SimpleDateFormat::new("HH:mm")
    }

    pub fn timeFormat() -> SimpleDateFormat {
        SimpleDateFormat::new("HH:mm")
    }

    pub fn dateFormat() -> SimpleDateFormat {
        SimpleDateFormat::new("yyyy/MM/dd HH:mm")
    }

    pub fn fileNameFormat() -> SimpleDateFormat {
        SimpleDateFormat::new("yy-MM-dd-HH-mm-ss")
    }

    pub fn keyboardToolChars() -> Vec<String> {
        vec![
            "@".to_string(), "&".to_string(), "|".to_string(), "%".to_string(), "/".to_string(), ":".to_string(), "[".to_string(), "]".to_string(), "{".to_string(), "}".to_string(), "<".to_string(), ">".to_string(), "\\".to_string(), "$".to_string(), "#".to_string(), "!".to_string(), ".".to_string(),
            "href".to_string(), "src".to_string(), "textNodes".to_string(), "xpath".to_string(), "json".to_string(), "css".to_string(), "id".to_string(), "class".to_string(), "tag".to_string()
        ]
    }
}
