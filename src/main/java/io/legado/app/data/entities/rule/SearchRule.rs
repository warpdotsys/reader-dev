// package io.legado.app.data.entities.rule

// data class SearchRule(
//         override var bookList: String? = null,
//         override var name: String? = null,
//         override var author: String? = null,
//         override var intro: String? = null,
//         override var kind: String? = null,
//         override var lastChapter: String? = null,
//         override var updateTime: String? = null,
//         override var bookUrl: String? = null,
//         override var coverUrl: String? = null,
//         override var wordCount: String? = null
// ) : BookListRule
pub struct SearchRule {
    pub book_list: Option<String>,
    pub name: Option<String>,
    pub author: Option<String>,
    pub intro: Option<String>,
    pub kind: Option<String>,
    pub last_chapter: Option<String>,
    pub update_time: Option<String>,
    pub book_url: Option<String>,
    pub cover_url: Option<String>,
    pub word_count: Option<String>,
}

impl SearchRule {
    fn book_list(&self) -> Option<&str> {
        self.book_list.as_deref()
    }

    fn set_book_list(&mut self, value: Option<String>) {
        self.book_list = value;
    }

    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn set_name(&mut self, value: Option<String>) {
        self.name = value;
    }

    fn author(&self) -> Option<&str> {
        self.author.as_deref()
    }

    fn set_author(&mut self, value: Option<String>) {
        self.author = value;
    }

    fn intro(&self) -> Option<&str> {
        self.intro.as_deref()
    }

    fn set_intro(&mut self, value: Option<String>) {
        self.intro = value;
    }

    fn kind(&self) -> Option<&str> {
        self.kind.as_deref()
    }

    fn set_kind(&mut self, value: Option<String>) {
        self.kind = value;
    }

    fn last_chapter(&self) -> Option<&str> {
        self.last_chapter.as_deref()
    }

    fn set_last_chapter(&mut self, value: Option<String>) {
        self.last_chapter = value;
    }

    fn update_time(&self) -> Option<&str> {
        self.update_time.as_deref()
    }

    fn set_update_time(&mut self, value: Option<String>) {
        self.update_time = value;
    }

    fn book_url(&self) -> Option<&str> {
        self.book_url.as_deref()
    }

    fn set_book_url(&mut self, value: Option<String>) {
        self.book_url = value;
    }

    fn cover_url(&self) -> Option<&str> {
        self.cover_url.as_deref()
    }

    fn set_cover_url(&mut self, value: Option<String>) {
        self.cover_url = value;
    }

    fn word_count(&self) -> Option<&str> {
        self.word_count.as_deref()
    }

    fn set_word_count(&mut self, value: Option<String>) {
        self.word_count = value;
    }
}

// impl BookListRule for SearchRule {}

impl Default for SearchRule {
    fn default() -> Self {
        SearchRule {
            book_list: None,
            name: None,
            author: None,
            intro: None,
            kind: None,
            last_chapter: None,
            update_time: None,
            book_url: None,
            cover_url: None,
            word_count: None,
        }
    }
}

impl PartialEq for SearchRule {
    fn eq(&self, other: &Self) -> bool {
        self.book_list == other.book_list
            && self.name == other.name
            && self.author == other.author
            && self.intro == other.intro
            && self.kind == other.kind
            && self.last_chapter == other.last_chapter
            && self.update_time == other.update_time
            && self.book_url == other.book_url
            && self.cover_url == other.cover_url
            && self.word_count == other.word_count
    }
}

impl Eq for SearchRule {}
