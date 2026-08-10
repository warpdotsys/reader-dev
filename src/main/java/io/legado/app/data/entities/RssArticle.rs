// package io.legado.app.data.entities

// import io.legado.app.utils.GSON
// import io.legado.app.utils.fromJsonObject
// import io.legado.app.model.analyzeRule.RuleDataInterface
// import com.fasterxml.jackson.annotation.JsonIgnoreProperties

// @JsonIgnoreProperties("variableMap", "_userNameSpace", "userNameSpace")
pub struct RssArticle {
    pub origin: String,
    pub sort: String,
    pub title: String,
    pub order: i64,
    pub link: String,
    pub pub_date: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub image: Option<String>,
    pub read: bool,
    pub variable: Option<String>,

    // private var _userNameSpace = ""
    pub user_name_space: String,

    // @delegate:Transient
    // override val variableMap: HashMap<String, String> by lazy {
    //     GSON.fromJsonObject<HashMap<String, String>>(variable).getOrNull() ?: hashMapOf()
    // }
    pub variable_map_cache: RefCell<Option<HashMap<String, String>>>,
}

impl RssArticle {
    pub fn variable_map(&self) -> HashMap<String, String> {
        if let Some(cached) = self.variable_map_cache.borrow().as_ref() {
            return cached.clone();
        }
        let map = GSON::from_json_object::<HashMap<String, String>>(self.variable.as_ref())
            .get_or_null()
            .unwrap_or_else(HashMap::new);
        *self.variable_map_cache.borrow_mut() = Some(map.clone());
        map
    }

    pub fn put_variable(&mut self, key: String, value: Option<String>) {
        let mut map = self.variable_map();
        if let Some(v) = value {
            map.insert(key, v);
        } else {
            map.remove(&key);
        }
        *self.variable_map_cache.borrow_mut() = Some(map.clone());
        self.variable = Some(GSON::to_json(map));
    }

    pub fn set_user_name_space(&mut self, name_space: String) {
        self.user_name_space = name_space;
    }

    pub fn get_user_name_space(&self) -> String {
        self.user_name_space.clone()
    }

    // fun toStar() = RssStar(
    //     origin = origin,
    //     sort = sort,
    //     title = title,
    //     starTime = System.currentTimeMillis(),
    //     link = link,
    //     pubDate = pubDate,
    //     description = description,
    //     content = content,
    //     image = image
    // )
}

impl Default for RssArticle {
    fn default() -> Self {
        RssArticle {
            origin: String::new(),
            sort: String::new(),
            title: String::new(),
            order: 0,
            link: String::new(),
            pub_date: None,
            description: None,
            content: None,
            image: None,
            read: false,
            variable: None,
            user_name_space: String::new(),
            variable_map_cache: RefCell::new(None),
        }
    }
}

impl PartialEq for RssArticle {
    // override fun equals(other: Any?): Boolean {
    //     other ?: return false
    //     return if (other is RssArticle) origin == other.origin && link == other.link else false
    // }
    fn eq(&self, other: &Self) -> bool {
        self.origin == other.origin && self.link == other.link
    }
}

impl Eq for RssArticle {}

impl std::hash::Hash for RssArticle {
    // override fun hashCode() = link.hashCode()
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.link.hash(state);
    }
}
