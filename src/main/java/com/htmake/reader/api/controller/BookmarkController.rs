// package com.htmake.reader.api.controller

// private val logger = KotlinLogging.logger {}

// class BookmarkController(coroutineContext: CoroutineContext): BaseController(coroutineContext), CURD<Bookmark> {
pub struct BookmarkController {
    base: BaseController,
}

impl BookmarkController {
    // override fun getTableName(): String {
    //     return "bookmark"
    // }
    fn get_table_name(&self) -> String {
        return String::from("bookmark");
    }

    // override fun getEntityClass(): Class<Bookmark> {
    //     return Bookmark::class.java
    // }
    fn get_entity_class(&self) -> std::any::TypeId {
        return std::any::TypeId::of::<Bookmark>();
    }

    // override fun checker(json: JsonObject, entity: Bookmark): Boolean {
    //     return entity.time == json.getLong("time")
    // }
    fn checker(&self, json: &JsonObject, entity: &Bookmark) -> bool {
        return entity.time == json.get_long("time");
    }

    // override fun beforeSave(entity: Bookmark, db: DB<Bookmark>): ReturnData? {
    //     if (entity.bookName.isEmpty() && entity.bookAuthor.isEmpty()) {
    //         return ReturnData().setErrorMsg("书签信息错误")
    //     }
    //     return null
    // }
    fn before_save(&self, entity: &Bookmark, db: &DB<Bookmark>) -> Option<ReturnData> {
        if entity.book_name.is_empty() && entity.book_author.is_empty() {
            return Some(ReturnData::new().set_error_msg(String::from("书签信息错误")).clone());
        }
        return None;
    }

    // override suspend fun checkUserAuth(context: RoutingContext): Boolean {
    //     return checkAuth(context)
    // }
    fn check_user_auth(&self, context: &RoutingContext) -> bool {
        return self.base.check_auth(context);
    }

    // override fun getUserNS(context: RoutingContext): String {
    //     return getUserNameSpace(context)
    // }
    fn get_user_ns(&self, context: &RoutingContext) -> String {
        return self.base.get_user_name_space(context);
    }

    // suspend fun getBookmarks(context: RoutingContext): ReturnData {
    //     return list(context)
    // }
    pub fn get_bookmarks(&self, context: &RoutingContext) -> ReturnData {
        return self.list(context);
    }

    // suspend fun saveBookmark(context: RoutingContext): ReturnData {
    //     return save(context)
    // }
    pub fn save_bookmark(&self, context: &RoutingContext) -> ReturnData {
        return self.save(context);
    }

    // suspend fun saveBookmarks(context: RoutingContext): ReturnData {
    //     return saveMulti(context)
    // }
    pub fn save_bookmarks(&self, context: &RoutingContext) -> ReturnData {
        return self.save_multi(context);
    }

    // suspend fun deleteBookmark(context: RoutingContext): ReturnData {
    //     return delete(context)
    // }
    pub fn delete_bookmark(&self, context: &RoutingContext) -> ReturnData {
        return self.delete(context);
    }

    // suspend fun deleteBookmarks(context: RoutingContext): ReturnData {
    //     return deleteMulti(context)
    // }
    pub fn delete_bookmarks(&self, context: &RoutingContext) -> ReturnData {
        return self.delete_multi(context);
    }
}
