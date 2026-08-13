use crate::prelude::*;
use crate::stubs::GSON;
use std::cmp::Ordering;
// package io.legado.app.data.entities

//import android.os.Parcelable
//import androidx.room.*
// import io.legado.app.utils.GSON
// import io.legado.app.utils.fromJsonObject
// import com.fasterxml.jackson.annotation.JsonIgnoreProperties;

//@Parcelize
//@Entity(
//    tableName = "searchBooks",
//    indices = [(Index(value = ["bookUrl"], unique = true))],
//    foreignKeys = [(ForeignKey(
//        entity = BookSource::class,
//        parentColumns = ["bookSourceUrl"],
//        childColumns = ["origin"],
//        onDelete = ForeignKey.CASCADE
//    ))]
//)
// @JsonIgnoreProperties("variableMap", "infoHtml", "tocHtml", "origins", "kindList")
pub struct SearchBook {
    //    @PrimaryKey
    pub book_url: String,
    pub origin: String,                     // 书源规则
    pub origin_name: String,
    pub r#type: i32,                        // @BookType
    pub name: String,
    pub author: String,
    pub kind: Option<String>,
    pub cover_url: Option<String>,
    pub intro: Option<String>,
    pub word_count: Option<String>,
    pub latest_chapter_title: Option<String>,
    pub toc_url: String,                    // 目录页Url (toc=table of Contents)
    pub time: i64,
    pub variable: Option<String>,
    pub origin_order: i32,

    // private var _userNameSpace = ""
    pub user_name_space: String,

    //    @Ignore
    //    @IgnoredOnParcel
    pub info_html: Option<String>,

    //    @Ignore
    //    @IgnoredOnParcel
    pub toc_html: Option<String>,

    // @delegate:Transient
    // override val variableMap: HashMap<String, String> by lazy {
    //     GSON.fromJsonObject<HashMap<String, String>>(variable).getOrNull() ?: hashMapOf()
    // }
    pub variable_map_cache: RefCell<Option<HashMap<String, String>>>,

    //    @Ignore
    //    @IgnoredOnParcel
    // var origins: LinkedHashSet<String>? = None
    //     private set
    pub origins: Option<LinkedHashSet<String>>,
}

impl SearchBook {
    pub fn variable_map(&self) -> HashMap<String, String> {
        if let Some(cached) = self.variable_map_cache.borrow().as_ref() {
            return cached.clone();
        }
        let map = GSON::from_json_object::<HashMap<String, String>>(self.variable.clone().unwrap_or_default())
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

    pub fn add_origin(&mut self, origin: String) {
        if self.origins.is_none() {
            let mut set = LinkedHashSet::new();
            set.insert(self.origin.clone());
            self.origins = Some(set);
        }
        if let Some(origins) = self.origins.as_mut() {
            origins.insert(origin);
        }
    }

    pub fn to_book(&mut self) -> Book {
        let mut book = Book {
            name: self.name.clone(),
            author: self.author.clone(),
            kind: self.kind.clone(),
            book_url: self.book_url.clone(),
            origin: self.origin.clone(),
            origin_name: self.origin_name.clone(),
            r#type: self.r#type,
            word_count: self.word_count.clone(),
            latest_chapter_title: self.latest_chapter_title.clone(),
            cover_url: self.cover_url.clone(),
            intro: self.intro.clone(),
            toc_url: self.toc_url.clone(),
            //            originOrder = originOrder,
            variable: self.variable.clone(),
            ..Book::default()
        };
        book.info_html = self.info_html.clone();
        book.toc_url = self.toc_url.clone();
        book.set_user_name_space(self.get_user_name_space());
        book
    }
}

impl Default for SearchBook {
    fn default() -> Self {
        SearchBook {
            book_url: String::new(),
            origin: String::new(),
            origin_name: String::new(),
            r#type: 0,
            name: String::new(),
            author: String::new(),
            kind: None,
            cover_url: None,
            intro: None,
            word_count: None,
            latest_chapter_title: None,
            toc_url: String::new(),
            time: 0,
            variable: None,
            origin_order: 0,
            user_name_space: String::new(),
            info_html: None,
            toc_html: None,
            variable_map_cache: RefCell::new(None),
            origins: None,
        }
    }
}

impl PartialEq for SearchBook {
    // override fun equals(other: Any?): Boolean {
    //     if (other is SearchBook) {
    //         if (other.bookUrl == bookUrl) {
    //             return true
    //         }
    //     }
    //     return false
    // }
    fn eq(&self, other: &Self) -> bool {
        other.book_url == self.book_url
    }
}

impl Eq for SearchBook {}

impl std::hash::Hash for SearchBook {
    // override fun hashCode(): Int {
    //     return bookUrl.hashCode()
    // }
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.book_url.hash(state);
    }
}

// override fun compareTo(other: SearchBook): Int {
//     return other.originOrder - this.originOrder
// }
impl Ord for SearchBook {
    fn cmp(&self, other: &Self) -> Ordering {
        (other.origin_order - self.origin_order).cmp(&0)
    }
}

impl PartialOrd for SearchBook {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'de> serde::Deserialize<'de> for SearchBook {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = serde_json::Value::deserialize(deserializer)?;
        let gs = |k: &str| v.get(k).and_then(|x| x.as_str()).map(|s| s.to_string());
        let gi = |k: &str| v.get(k).and_then(|x| x.as_i64()).map(|i| i as i32).unwrap_or(0);
        let gl = |k: &str| v.get(k).and_then(|x| x.as_i64()).unwrap_or(0);
        Ok(SearchBook {
            book_url: gs("bookUrl").unwrap_or_default(),
            origin: gs("origin").unwrap_or_default(),
            origin_name: gs("originName").unwrap_or_default(),
            r#type: gi("type"),
            name: gs("name").unwrap_or_default(),
            author: gs("author").unwrap_or_default(),
            kind: gs("kind"),
            cover_url: gs("coverUrl"),
            intro: gs("intro"),
            word_count: gs("wordCount"),
            latest_chapter_title: gs("latestChapterTitle"),
            toc_url: gs("tocUrl").unwrap_or_default(),
            time: gl("time"),
            variable: gs("variable"),
            origin_order: gi("originOrder"),
            user_name_space: gs("userNameSpace").unwrap_or_default(),
            info_html: gs("infoHtml"),
            toc_html: gs("tocHtml"),
            origins: None,
            variable_map_cache: std::cell::RefCell::new(None),
        })
    }
}