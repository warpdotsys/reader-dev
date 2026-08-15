use crate::prelude::*;
use crate::stubs::GSON;
// package io.legado.app.data.entities

// import io.legado.app.utils.GSON
// import io.legado.app.utils.fromJsonObject
// import io.legado.app.utils.MD5Utils
// import io.legado.app.model.analyzeRule.AnalyzeUrl
// import io.legado.app.model.analyzeRule.RuleDataInterface
// import io.legado.app.utils.NetworkUtils
// import com.fasterxml.jackson.annotation.JsonIgnoreProperties

// @JsonIgnoreProperties("variableMap", "_userNameSpace", "userNameSpace")
// fix: AnalyzeUrl companion 的 paramPattern 转录为其私有模块级函数, 此处按同样式提供等价正则
fn PARAM_PATTERN() -> Pattern {
    Pattern::compile(r"\s*,\s*(?=\{)")
}

pub struct BookChapter {
    pub url: String,               // 章节地址
    pub title: String,             // 章节标题
    pub is_volume: bool,           // 是否是卷名
    pub base_url: String,          //用来拼接相对url
    pub book_url: String,          // 书籍地址
    pub index: i32,                // 章节序号
    pub resource_url: Option<String>, // 音频真实URL
    pub tag: Option<String>,       //
    pub start: Option<i64>,        // 章节起始位置
    pub end: Option<i64>,          // 章节终止位置
    pub start_fragment_id: Option<String>, //EPUB书籍当前章节的fragmentId
    pub end_fragment_id: Option<String>,   //EPUB书籍下一章节的fragmentId
    pub variable: Option<String>,  //变量

    // private var _userNameSpace = ""
    pub user_name_space: String,

    // @delegate:Transient
    // override val variableMap: HashMap<String, String> by lazy {
    //     GSON.fromJsonObject<HashMap<String, String>>(variable).getOrNull() ?: hashMapOf()
    // }
    pub variable_map_cache: std::sync::Mutex<Option<HashMap<String, String>>>,
}

impl BookChapter {
    pub fn variable_map(&self) -> HashMap<String, String> {
        if let Some(cached) = self.variable_map_cache.lock().unwrap().as_ref() {
            return cached.clone();
        }
        let map = GSON::from_json_object::<HashMap<String, String>>(
            self.variable.clone().unwrap_or_default()
        )
            .get_or_null()
            .unwrap_or_else(HashMap::new);
        *self.variable_map_cache.lock().unwrap() = Some(map.clone());
        map
    }

    pub fn put_variable(&mut self, key: String, value: Option<String>) {
        let mut map = self.variable_map();
        if let Some(v) = value {
            map.insert(key, v);
        } else {
            map.remove(&key);
        }
        *self.variable_map_cache.lock().unwrap() = Some(map.clone());
        self.variable = Some(GSON::to_json(map));
    }

    pub fn set_user_name_space(&mut self, name_space: String) {
        self.user_name_space = name_space;
    }

    pub fn get_user_name_space(&self) -> String {
        self.user_name_space.clone()
    }

    pub fn get_absolute_url(&self) -> String {
        let pattern = PARAM_PATTERN();
        let mut url_matcher = pattern.matcher(self.url.clone());
        let url_before = if url_matcher.find() {
            self.url[0..url_matcher.start()].to_string()
        } else {
            self.url.clone()
        };
        let url_absolute_before = NetworkUtils::getAbsoluteURL(Some(&self.base_url), &url_before);
        if url_before.len() == self.url.len() {
            url_absolute_before
        } else {
            url_absolute_before + "," + &self.url[url_matcher.end()..]
        }
    }

    pub fn get_file_name(&self) -> String {
        format!("{:05}-{}.nb", self.index, MD5Utils::md5Encode16(&self.title))
    }
}

impl Default for BookChapter {
    fn default() -> Self {
        BookChapter {
            url: String::new(),
            title: String::new(),
            is_volume: false,
            base_url: String::new(),
            book_url: String::new(),
            index: 0,
            resource_url: None,
            tag: None,
            start: None,
            end: None,
            start_fragment_id: None,
            end_fragment_id: None,
            variable: None,
            user_name_space: String::new(),
            variable_map_cache: std::sync::Mutex::new(None),
        }
    }
}

impl PartialEq for BookChapter {
    // override fun equals(other: Any?): Boolean {
    //     if (other is BookChapter) {
    //         return other.url == url
    //     }
    //     return false
    // }
    fn eq(&self, other: &Self) -> bool {
        other.url == self.url
    }
}

impl Eq for BookChapter {}

impl std::hash::Hash for BookChapter {
    // override fun hashCode() = url.hashCode()
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.url.hash(state);
    }
}

impl<'de> serde::Deserialize<'de> for BookChapter {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = serde_json::Value::deserialize(deserializer)?;
        let gs = |k: &str| v.get(k).and_then(|x| x.as_str()).map(|s| s.to_string());
        let gi = |k: &str| v.get(k).and_then(|x| x.as_i64()).map(|i| i as i32).unwrap_or(0);
        let go = |k: &str| v.get(k).and_then(|x| x.as_i64());
        Ok(BookChapter {
            url: gs("url").unwrap_or_default(),
            title: gs("title").unwrap_or_default(),
            is_volume: v.get("isVolume").and_then(|x| x.as_bool()).unwrap_or(false),
            base_url: gs("baseUrl").unwrap_or_default(),
            book_url: gs("bookUrl").unwrap_or_default(),
            index: gi("index"),
            resource_url: gs("resourceUrl"),
            tag: gs("tag"),
            start: go("start"),
            end: go("end"),
            start_fragment_id: gs("startFragmentId"),
            end_fragment_id: gs("endFragmentId"),
            variable: gs("variable"),
            user_name_space: gs("userNameSpace").unwrap_or_default(),
            variable_map_cache: std::sync::Mutex::new(None),
        })
    }
}