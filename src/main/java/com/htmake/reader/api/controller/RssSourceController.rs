use crate::prelude::*;
use crate::com_htmake_reader_api_controller_curd::RoutingContext;
use crate::io_legado_app_model_debug::Debug;
use crate::stubs::{Any, JsonArray, JsonObject};
// package com.htmake.reader.api.controller

// private val logger = KotlinLogging.logger {}

// class RssSourceController(coroutineContext: CoroutineContext): BaseController(coroutineContext) {
pub struct RssSourceController {
    base: BaseController,
}

impl RssSourceController {
    pub fn new() -> RssSourceController {
        RssSourceController {
            base: BaseController::new(),
        }
    }

    // suspend fun canEditRssSource(context: RoutingContext): Boolean {
    //     if (!appConfig.secure) {
    //         return true
    //     }
    //     val userInfo = context.get("userInfo") as User? ?: return false
    //     return userInfo.enable_book_source
    // }
    pub fn can_edit_rss_source(&self, context: &RoutingContext) -> bool {
        // fix: 原 Kotlin `if (!appConfig.secure) return true`（真实读取配置）
        let secure = crate::com_htmake_reader_utils_springcontextutils::SpringContextUtils::get_bean_app_config().secure;
        if !secure {
            return true;
        }
        let user_info = context.get_user::<User>("userInfo");
        let user_info = match user_info {
            Some(u) => u,
            None => return false,
        };
        return user_info.enable_book_source;
    }

    // suspend fun getRssSources(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     var userNameSpace = getUserNameSpace(context)
    //     var list: JsonArray? = asJsonArray(getUserStorage(userNameSpace, "rssSources"))
    //     if (list != None) {
    //         return returnData.setData(list.getList())
    //     }
    //     return returnData.setData(arrayListOf<Int>())
    // }
    pub fn get_rss_sources(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
            return return_data;
        }
        let user_name_space = self.base.get_user_name_space(context);
        let list: Option<JsonArray> = as_json_array(self.base.get_user_storage(&user_name_space, vec![String::from("rssSources")]).map(Any::from_string));
        if let Some(list) = list {
            return_data.set_data(Box::new(list.get_list()), String::from(""));
            return return_data;
        }
        return_data.set_data(Box::new(Vec::<i32>::new()), String::from(""));
        return return_data;
    }

    // suspend fun saveRssSource(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     if (!canEditRssSource(context)) {
    //         return returnData.setErrorMsg("权限不足")
    //     }
    //     val rssSource = RssSource.fromJson(context.bodyAsString).getOrNull()
    //         ?: return returnData.setErrorMsg("参数错误")
    //     if (rssSource.sourceUrl.isEmpty()) {
    //         return returnData.setErrorMsg("RSS链接不能为空")
    //     }
    //     if (rssSource.sourceName.isEmpty()) {
    //         return returnData.setErrorMsg("RSS名称不能为空")
    //     }
    //
    //     var userNameSpace = getUserNameSpace(context)
    //     var rssSourceList: JsonArray? = asJsonArray(getUserStorage(userNameSpace, "rssSources"))
    //     if (rssSourceList == None) {
    //         rssSourceList = JsonArray()
    //     }
    //     // 遍历判断是否存在
    //     var existIndex: Int = -1
    //     for (i in 0 until rssSourceList.size()) {
    //         val _rssSource = RssSource.fromJson(rssSourceList.getJsonObject(i).toString()).getOrNull()
    //             ?: continue
    //         if (_rssSource.sourceUrl.equals(rssSource.sourceUrl)) {
    //             existIndex = i
    //             break;
    //         }
    //     }
    //     if (existIndex >= 0) {
    //         var list = rssSourceList.getList()
    //         list.set(existIndex, JsonObject.mapFrom(rssSource))
    //         rssSourceList = JsonArray(list)
    //     } else {
    //         // 新增rss源
    //         rssSourceList.add(JsonObject.mapFrom(rssSource))
    //     }
    //
    //     // logger.info("rssSourceList: {}", rssSourceList)
    //     saveUserStorage(userNameSpace, "rssSources", rssSourceList)
    //     return returnData.setData("")
    // }
    pub fn save_rss_source(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
            return return_data;
        }
        if !self.can_edit_rss_source(context) {
            return_data.set_error_msg(String::from("权限不足"));
            return return_data;
        }
        let rss_source = match RssSource::from_json(context.body_as_string()).get_or_none() {
            Some(v) => v,
            None => {
                return_data.set_error_msg(String::from("参数错误"));
                return return_data;
            }
        };
        if rss_source.source_url.is_empty() {
            return_data.set_error_msg(String::from("RSS链接不能为空"));
            return return_data;
        }
        if rss_source.source_name.is_empty() {
            return_data.set_error_msg(String::from("RSS名称不能为空"));
            return return_data;
        }

        let user_name_space = self.base.get_user_name_space(context);
        let mut rss_source_list: Option<JsonArray> = as_json_array(self.base.get_user_storage(&user_name_space, vec![String::from("rssSources")]).map(Any::from_string));
        if rss_source_list.is_none() {
            rss_source_list = Some(JsonArray::new());
        }
        let mut rss_source_list = rss_source_list.unwrap();
        // 遍历判断是否存在
        let mut exist_index: i32 = -1;
        for i in 0..rss_source_list.size() {
            let _rss_source = match RssSource::from_json(rss_source_list.get_json_object(i).map(|o| o.to_string()).unwrap_or_default()).get_or_none() {
                Some(v) => v,
                None => continue,
            };
            if _rss_source.source_url == rss_source.source_url {
                exist_index = i;
                break;
            }
        }
        if exist_index >= 0 {
            rss_source_list.set(exist_index as usize, JsonObject::map_from(rss_source.clone()));
        } else {
            // 新增rss源
            rss_source_list.add(JsonObject::map_from(rss_source.clone()));
        }

        // logger.info("rssSourceList: {}", rssSourceList)
        self.base.save_user_storage(&user_name_space, String::from("rssSources"), Box::new(rss_source_list));
        return_data.set_data(Box::new(String::from("")), String::from(""));
        return return_data;
    }

    // suspend fun saveRssSources(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     if (!canEditRssSource(context)) {
    //         return returnData.setErrorMsg("权限不足")
    //     }
    //     val rssSourceJsonArray = context.bodyAsJsonArray
    //     if (rssSourceJsonArray == None) {
    //         return returnData.setErrorMsg("参数错误")
    //     }
    //     var userNameSpace = getUserNameSpace(context)
    //     var rssSourceList: JsonArray? = asJsonArray(getUserStorage(userNameSpace, "rssSources"))
    //     if (rssSourceList == None) {
    //         rssSourceList = JsonArray()
    //     }
    //     for (k in 0 until rssSourceJsonArray.size()) {
    //         val rssSource = RssSource.fromJson(rssSourceJsonArray.getJsonObject(k).toString()).getOrNull()
    //             ?: continue
    //         if (rssSource.sourceUrl.isEmpty()) {
    //             continue
    //         }
    //         if (rssSource.sourceName.isEmpty()) {
    //             continue
    //         }
    //         // 遍历判断是否存在
    //         var existIndex: Int = -1
    //         for (i in 0 until rssSourceList!!.size()) {
    //             val _rssSource = RssSource.fromJson(rssSourceList.getJsonObject(i).toString()).getOrNull()
    //                 ?: continue
    //             if (_rssSource.sourceUrl.equals(rssSource.sourceUrl)) {
    //                 existIndex = i
    //                 break;
    //             }
    //         }
    //         if (existIndex >= 0) {
    //             var list = rssSourceList.getList()
    //             list.set(existIndex, JsonObject.mapFrom(rssSource))
    //             rssSourceList = JsonArray(list)
    //         } else {
    //             // 新增rss源
    //             rssSourceList.add(JsonObject.mapFrom(rssSource))
    //         }
    //     }
    //
    //     // logger.info("rssSourceList: {}", rssSourceList)
    //     saveUserStorage(userNameSpace, "rssSources", rssSourceList!!)
    //     return returnData.setData("")
    // }
    pub fn save_rss_sources(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
            return return_data;
        }
        if !self.can_edit_rss_source(context) {
            return_data.set_error_msg(String::from("权限不足"));
            return return_data;
        }
        let rss_source_json_array = context.body_as_json_array();
        if rss_source_json_array.is_none() {
            return_data.set_error_msg(String::from("参数错误"));
            return return_data;
        }
        let rss_source_json_array = rss_source_json_array.unwrap();
        let user_name_space = self.base.get_user_name_space(context);
        let mut rss_source_list: Option<JsonArray> = as_json_array(self.base.get_user_storage(&user_name_space, vec![String::from("rssSources")]).map(Any::from_string));
        if rss_source_list.is_none() {
            rss_source_list = Some(JsonArray::new());
        }
        let mut rss_source_list = rss_source_list.unwrap();
        for k in 0..rss_source_json_array.size() {
            let rss_source = match RssSource::from_json(rss_source_json_array.get_json_object(k).map(|o| o.to_string()).unwrap_or_default()).get_or_none() {
                Some(v) => v,
                None => continue,
            };
            if rss_source.source_url.is_empty() {
                continue;
            }
            if rss_source.source_name.is_empty() {
                continue;
            }
            // 遍历判断是否存在
            let mut exist_index: i32 = -1;
            for i in 0..rss_source_list.size() {
                let _rss_source = match RssSource::from_json(rss_source_list.get_json_object(i).map(|o| o.to_string()).unwrap_or_default()).get_or_none() {
                    Some(v) => v,
                    None => continue,
                };
                if _rss_source.source_url == rss_source.source_url {
                    exist_index = i;
                    break;
                }
            }
            if exist_index >= 0 {
                rss_source_list.set(exist_index as usize, JsonObject::map_from(rss_source.clone()));
            } else {
                // 新增rss源
                rss_source_list.add(JsonObject::map_from(rss_source.clone()));
            }
        }

        // logger.info("rssSourceList: {}", rssSourceList)
        self.base.save_user_storage(&user_name_space, String::from("rssSources"), Box::new(rss_source_list));
        return_data.set_data(Box::new(String::from("")), String::from(""));
        return return_data;
    }

    // suspend fun deleteRssSource(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     if (!canEditRssSource(context)) {
    //         return returnData.setErrorMsg("权限不足")
    //     }
    //     val rssSource = RssSource.fromJson(context.bodyAsString).getOrNull()
    //         ?: return returnData.setErrorMsg("参数错误")
    //     var userNameSpace = getUserNameSpace(context)
    //     var rssSourceList: JsonArray? = asJsonArray(getUserStorage(userNameSpace, "rssSources"))
    //     if (rssSourceList == None) {
    //         rssSourceList = JsonArray()
    //     }
    //     // 遍历判断是否存在
    //     var existIndex: Int = -1
    //     for (i in 0 until rssSourceList.size()) {
    //         val _rssSource = RssSource.fromJson(rssSourceList.getJsonObject(i).toString()).getOrNull()
    //             ?: continue
    //         if (_rssSource.sourceUrl.equals(rssSource.sourceUrl)) {
    //             existIndex = i
    //             break;
    //         }
    //     }
    //     if (existIndex >= 0) {
    //         rssSourceList.remove(existIndex)
    //     }
    //     // logger.info("rssSource: {}", rssSource)
    //     saveUserStorage(userNameSpace, "rssSources", rssSourceList)
    //     return returnData.setData("")
    // }
    pub fn delete_rss_source(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
            return return_data;
        }
        if !self.can_edit_rss_source(context) {
            return_data.set_error_msg(String::from("权限不足"));
            return return_data;
        }
        let rss_source = match RssSource::from_json(context.body_as_string()).get_or_none() {
            Some(v) => v,
            None => {
                return_data.set_error_msg(String::from("参数错误"));
                return return_data;
            }
        };
        let user_name_space = self.base.get_user_name_space(context);
        let mut rss_source_list: Option<JsonArray> = as_json_array(self.base.get_user_storage(&user_name_space, vec![String::from("rssSources")]).map(Any::from_string));
        if rss_source_list.is_none() {
            rss_source_list = Some(JsonArray::new());
        }
        let mut rss_source_list = rss_source_list.unwrap();
        // 遍历判断是否存在
        let mut exist_index: i32 = -1;
        for i in 0..rss_source_list.size() {
            let _rss_source = match RssSource::from_json(rss_source_list.get_json_object(i).map(|o| o.to_string()).unwrap_or_default()).get_or_none() {
                Some(v) => v,
                None => continue,
            };
            if _rss_source.source_url == rss_source.source_url {
                exist_index = i;
                break;
            }
        }
        if exist_index >= 0 {
            rss_source_list.remove(exist_index as usize);
        }
        // logger.info("rssSource: {}", rssSource)
        self.base.save_user_storage(&user_name_space, String::from("rssSources"), Box::new(rss_source_list));
        return_data.set_data(Box::new(String::from("")), String::from(""));
        return return_data;
    }

    // fun getRssSourceByURL(url: String, userNameSpace: String): RssSource? {
    //     if (url.isEmpty()) {
    //         return None
    //     }
    //     var list: JsonArray? = asJsonArray(getUserStorage(userNameSpace, "rssSources"))
    //     if (list == None) {
    //         return None
    //     }
    //     for (i in 0 until list.size()) {
    //         val _rssSource = RssSource.fromJson(list.getJsonObject(i).toString()).getOrNull()
    //             ?: continue
    //         if (_rssSource.sourceUrl.equals(url)) {
    //             return _rssSource
    //         }
    //     }
    //     return None
    // }
    pub fn get_rss_source_by_url(&self, url: &String, user_name_space: String) -> Option<RssSource> {
        if url.is_empty() {
            return None;
        }
        let list: Option<JsonArray> = as_json_array(self.base.get_user_storage(&user_name_space, vec![String::from("rssSources")]).map(Any::from_string));
        if list.is_none() {
            return None;
        }
        let list = list.unwrap();
        for i in 0..list.size() {
            let _rss_source = match RssSource::from_json(list.get_json_object(i).map(|o| o.to_string()).unwrap_or_default()).get_or_none() {
                Some(v) => v,
                None => continue,
            };
            if _rss_source.source_url == *url {
                return Some(_rss_source);
            }
        }
        return None;
    }

    // suspend fun getRssArticles(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     var sourceUrl: String
    //     var sortName: String
    //     var sortUrl: String
    //     var page: Int
    //     if (context.request().method() == HttpMethod.POST) {
    //         // post 请求
    //         sourceUrl = context.bodyAsJson.getString("sourceUrl")
    //         sortName = context.bodyAsJson.getString("sortName", "")
    //         sortUrl = context.bodyAsJson.getString("sortUrl", "")
    //         page = context.bodyAsJson.getInteger("page", 1)
    //     } else {
    //         // get 请求
    //         sourceUrl = context.queryParam("sourceUrl").firstOrNull() ?: ""
    //         sortName = context.queryParam("sortName").firstOrNull() ?: ""
    //         sortUrl = context.queryParam("sortUrl").firstOrNull() ?: ""
    //         page = context.queryParam("page").firstOrNull()?.toInt() ?: 1
    //     }
    //     if (sourceUrl.isEmpty()) {
    //         return returnData.setErrorMsg("RSS源链接不能为空")
    //     }
    //     if (sortUrl.isEmpty()) {
    //         sortUrl = sourceUrl
    //     }
    //
    //     var userNameSpace = getUserNameSpace(context)
    //     var rssSource = getRssSourceByURL(sourceUrl, userNameSpace)
    //     if (rssSource == None) {
    //         return returnData.setErrorMsg("RSS源不存在")
    //     }
    //
    //     val rssArtcles = Rss.getArticles(sortName, sortUrl, rssSource, page, Debug)
    //
    //     return returnData.setData(rssArtcles)
    // }
    pub fn get_rss_articles(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
            return return_data;
        }
        let source_url: String;
        let sort_name: String;
        let mut sort_url: String;
        let page: i32;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            let body = match context.body_as_json() {
                Some(v) => v,
                None => {
                    return_data.set_error_msg(String::from("参数错误"));
                    return return_data;
                }
            };
            source_url = body.get_string("sourceUrl");
            sort_name = body.get_string("sortName");
            sort_url = body.get_string("sortUrl");
            page = body.get_integer("page", 1);
        } else {
            // get 请求
            source_url = context.query_param("sourceUrl").unwrap_or(String::from(""));
            sort_name = context.query_param("sortName").unwrap_or(String::from(""));
            sort_url = context.query_param("sortUrl").unwrap_or(String::from(""));
            page = context.query_param("page").and_then(|s| s.parse::<i32>().ok()).unwrap_or(1);
        }
        if source_url.is_empty() {
            return_data.set_error_msg(String::from("RSS源链接不能为空"));
            return return_data;
        }
        if sort_url.is_empty() {
            sort_url = source_url.clone();
        }

        let user_name_space = self.base.get_user_name_space(context);
        let rss_source = self.get_rss_source_by_url(&source_url, user_name_space);
        if rss_source.is_none() {
            return_data.set_error_msg(String::from("RSS源不存在"));
            return return_data;
        }
        let rss_source = rss_source.unwrap();

        let rss_articles = block_on(Rss::get_articles(&sort_name, &sort_url, &rss_source, page, Some(&Debug)));
        // fix: Kotlin 返回 Pair(文章列表, 下一页URL)——Gson 序列化为 {"first":[...],"second":...}，前端读 data.first
        let next_page = rss_articles.1.unwrap_or_default();
        let mut pair_map: std::collections::HashMap<String, crate::stubs::Any> = std::collections::HashMap::new();
        pair_map.insert(String::from("first"), crate::stubs::Any::from(rss_articles.0));
        pair_map.insert(String::from("second"), crate::stubs::Any::from(next_page));
        return_data.set_data(Box::new(pair_map), String::from(""));
        return return_data;
    }

    // suspend fun getRssContent(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     var sourceUrl: String
    //     var link: String
    //     var origin: String
    //     if (context.request().method() == HttpMethod.POST) {
    //         // post 请求
    //         sourceUrl = context.bodyAsJson.getString("sourceUrl")
    //         link = context.bodyAsJson.getString("link")
    //         origin = context.bodyAsJson.getString("origin")
    //     } else {
    //         // get 请求
    //         sourceUrl = context.queryParam("sourceUrl").firstOrNull() ?: ""
    //         link = context.queryParam("link").firstOrNull() ?: ""
    //         origin = context.queryParam("origin").firstOrNull() ?: ""
    //     }
    //     if (sourceUrl.isEmpty()) {
    //         return returnData.setErrorMsg("RSS链接不能为空")
    //     }
    //     if (link.isEmpty()) {
    //         return returnData.setErrorMsg("RSS文章链接不能为空")
    //     }
    //     if (origin.isEmpty()) {
    //         return returnData.setErrorMsg("RSS文章来源不能为空")
    //     }
    //
    //     var userNameSpace = getUserNameSpace(context)
    //     var rssSource = getRssSourceByURL(sourceUrl, userNameSpace)
    //     if (rssSource == None) {
    //         return returnData.setErrorMsg("RSS源不存在")
    //     }
    //     val rssArticle = RssArticle(origin = origin, link = link)
    //     var content = ""
    //     if (rssSource.ruleContent != None) {
    //         content = Rss.getContent(rssArticle, rssSource.ruleContent as String, rssSource, Debug)
    //     }
    //
    //     return returnData.setData(content)
    // }
    pub fn get_rss_content(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
            return return_data;
        }
        let source_url: String;
        let link: String;
        let origin: String;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            let body = match context.body_as_json() {
                Some(v) => v,
                None => {
                    return_data.set_error_msg(String::from("参数错误"));
                    return return_data;
                }
            };
            source_url = body.get_string("sourceUrl");
            link = body.get_string("link");
            origin = body.get_string("origin");
        } else {
            // get 请求
            source_url = context.query_param("sourceUrl").unwrap_or(String::from(""));
            link = context.query_param("link").unwrap_or(String::from(""));
            origin = context.query_param("origin").unwrap_or(String::from(""));
        }
        if source_url.is_empty() {
            return_data.set_error_msg(String::from("RSS链接不能为空"));
            return return_data;
        }
        if link.is_empty() {
            return_data.set_error_msg(String::from("RSS文章链接不能为空"));
            return return_data;
        }
        if origin.is_empty() {
            return_data.set_error_msg(String::from("RSS文章来源不能为空"));
            return return_data;
        }

        let user_name_space = self.base.get_user_name_space(context);
        let rss_source = self.get_rss_source_by_url(&source_url, user_name_space);
        if rss_source.is_none() {
            return_data.set_error_msg(String::from("RSS源不存在"));
            return return_data;
        }
        let rss_source = rss_source.unwrap();
        let rss_article = RssArticle {
            origin,
            link,
            ..RssArticle::default()
        };
        let mut content = String::from("");
        if rss_source.rule_content.is_some() {
            content = block_on(Rss::get_content(&rss_article, rss_source.rule_content.as_deref().unwrap_or(""), &rss_source, Some(&Debug)));
        }

        return_data.set_data(Box::new(content), String::from(""));
        return return_data;
    }
}
