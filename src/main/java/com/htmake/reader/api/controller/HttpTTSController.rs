use crate::prelude::*;
// fix: 显式导入消解 prelude 多 glob 重导出歧义（JsonObject ← curd/stubs；DB ← curd/db）
use crate::stubs::JsonObject;
use crate::com_htmake_reader_db_db::DB;
// package com.htmake.reader.api.controller

// private val logger = KotlinLogging.logger {}

// class HttpTTSController(coroutineContext: CoroutineContext): BaseController(coroutineContext), CURD<HttpTTS> {
pub struct HttpTTSController {
    base: BaseController,
}

impl CURD<HttpTTS> for HttpTTSController {
    // override fun getTableName(): String {
    //     return "httpTTS"
    // }
    fn get_table_name(&self) -> String {
        return String::from("httpTTS");
    }

    // override fun getEntityClass(): Class<HttpTTS> {
    //     return HttpTTS::class.java
    // }
    fn get_entity_class(&self) -> std::any::TypeId {
        return std::any::TypeId::of::<HttpTTS>();
    }

    // override fun checker(json: JsonObject, entity: HttpTTS): Boolean {
    //     return entity.name == json.getString("name")
    // }
    // fix: Kotlin getString("name") 缺 key → null 恒不匹配（同 ReplaceRuleController）
    fn checker(&self, json: &JsonObject, entity: &HttpTTS) -> bool {
        match json.get_string_opt("name") {
            Some(v) => entity.name == v,
            None => false,
        }
    }

    // override fun beforeSave(entity: HttpTTS, db: DB<HttpTTS>): ReturnData? {
    //     val returnData = ReturnData()
    //     if (entity.name.isEmpty()) return returnData.setErrorMsg("名称不能为空")
    //     if (entity.url.isEmpty()) return returnData.setErrorMsg("链接不能为空")
    //     return None
    // }
    fn before_save(&self, entity: &HttpTTS, db: &DB<HttpTTS>) -> Option<ReturnData> {
        let mut return_data = ReturnData::new();
        if entity.name.is_empty() {
            return_data.set_error_msg(String::from("名称不能为空"));
            return Some(return_data);
        }
        if entity.url.is_empty() {
            return_data.set_error_msg(String::from("链接不能为空"));
            return Some(return_data);
        }
        return None;
    }

    // override fun convertToEntity(json: JsonObject): HttpTTS {
    //     return HttpTTS.fromJson(json.toString()).getOrNull()!!
    // }
    fn convert_to_entity(&self, json: &JsonObject) -> HttpTTS {
        return HttpTTS::from_json(json.to_string()).get_or_none().unwrap();
    }

    // override fun convertToEntityList(json: String): Array<HttpTTS> {
    //     return asJsonArray(json)!!.map { HttpTTS.fromJson(it.toString()).getOrNull()!! }.toTypedArray()
    // }
    fn convert_to_entity_list(&self, json: &String) -> Vec<HttpTTS> {
        return as_json_array(Some(crate::stubs::Any::from_string(json.clone()))).unwrap().get_list().into_iter().map(|it| HttpTTS::from_json(it.to_string()).get_or_none().unwrap()).collect();
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
}

impl HttpTTSController {
    pub fn new() -> HttpTTSController {
        HttpTTSController {
            base: BaseController::new(),
        }
    }

    //     return list(context)
    // }
    pub fn get_http_tts_list(&self, context: &RoutingContext) -> ReturnData {
        return self.list(context);
    }

    // suspend fun saveHttpTTS(context: RoutingContext): ReturnData {
    //     return save(context)
    // }
    pub fn save_http_tts(&self, context: &RoutingContext) -> ReturnData {
        return self.save(context);
    }

    // suspend fun saveHttpTTSList(context: RoutingContext): ReturnData {
    //     return saveMulti(context)
    // }
    pub fn save_http_tts_list(&self, context: &RoutingContext) -> ReturnData {
        return self.save_multi(context);
    }

    // suspend fun deleteHttpTTS(context: RoutingContext): ReturnData {
    //     return delete(context)
    // }
    pub fn delete_http_tts(&self, context: &RoutingContext) -> ReturnData {
        return self.delete(context);
    }

    // fix: deleteMulti 路由原指向单删（前端发数组→单删解析失败；Kotlin 继承缺陷）——真批量删除
    pub fn delete_http_tts_list(&self, context: &RoutingContext) -> ReturnData {
        return self.delete_multi(context);
    }
}
