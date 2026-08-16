use crate::prelude::*;

// fix: 显式导入消除 stubs / CURD / DB glob 重导出歧义（显式导入优先于 glob）
use crate::com_htmake_reader_db_db::DB;
use crate::stubs::JsonObject;
// package com.htmake.reader.api.controller

// private val logger = KotlinLogging.logger {}

// class ReplaceRuleController(coroutineContext: CoroutineContext): BaseController(coroutineContext), CURD<ReplaceRule> {
pub struct ReplaceRuleController {
    base: BaseController,
}

impl CURD<ReplaceRule> for ReplaceRuleController {
    // override fun getTableName(): String {
    //     return "replaceRule"
    // }
    fn get_table_name(&self) -> String {
        return String::from("replaceRule");
    }

    // override fun getEntityClass(): Class<ReplaceRule> {
    //     return ReplaceRule::class.java
    // }
    fn get_entity_class(&self) -> std::any::TypeId {
        return std::any::TypeId::of::<ReplaceRule>();
    }

    // override fun checker(json: JsonObject, entity: ReplaceRule): Boolean {
    //     return entity.name == json.getString("name")
    // }
    // fix: Kotlin getString("name") 缺 key → null 恒不匹配（原缺 key 返回 ""——空 name 实体误匹配）
    fn checker(&self, json: &JsonObject, entity: &ReplaceRule) -> bool {
        match json.get_string_opt("name") {
            Some(v) => entity.name == v,
            None => false,
        }
    }

    // override fun beforeSave(entity: ReplaceRule, db: DB<ReplaceRule>): ReturnData? {
    //     if (entity.name.isEmpty()) {
    //         return ReturnData().setErrorMsg("名称不能为空")
    //     }
    //     if (entity.pattern.isEmpty()) {
    //         return ReturnData().setErrorMsg("规则不能为空")
    //     }
    //     return None
    // }
    fn before_save(&self, entity: &ReplaceRule, db: &DB<ReplaceRule>) -> Option<ReturnData> {
        if entity.name.is_empty() {
            // fix: ReturnData 未实现 Clone，set_error_msg 返回 &mut，先构造再返回
            let mut data = ReturnData::new();
            data.set_error_msg(String::from("名称不能为空"));
            return Some(data);
        }
        if entity.pattern.is_empty() {
            // fix: ReturnData 未实现 Clone，set_error_msg 返回 &mut，先构造再返回
            let mut data = ReturnData::new();
            data.set_error_msg(String::from("规则不能为空"));
            return Some(data);
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
}

impl ReplaceRuleController {
    pub fn new() -> ReplaceRuleController {
        ReplaceRuleController {
            base: BaseController::new(),
        }
    }

    // suspend fun getReplaceRules(context: RoutingContext): ReturnData {
    //     return list(context)
    // }
    pub fn get_replace_rules(&self, context: &RoutingContext) -> ReturnData {
        return self.list(context);
    }

    // suspend fun saveReplaceRule(context: RoutingContext): ReturnData {
    //     return save(context)
    // }
    pub fn save_replace_rule(&self, context: &RoutingContext) -> ReturnData {
        return self.save(context);
    }

    // suspend fun saveReplaceRules(context: RoutingContext): ReturnData {
    //     return saveMulti(context)
    // }
    pub fn save_replace_rules(&self, context: &RoutingContext) -> ReturnData {
        return self.save_multi(context);
    }

    // suspend fun deleteReplaceRule(context: RoutingContext): ReturnData {
    //     return delete(context)
    // }
    pub fn delete_replace_rule(&self, context: &RoutingContext) -> ReturnData {
        return self.delete(context);
    }

    // suspend fun deleteReplaceRules(context: RoutingContext): ReturnData {
    //     return deleteMulti(context)
    // }
    pub fn delete_replace_rules(&self, context: &RoutingContext) -> ReturnData {
        return self.delete_multi(context);
    }
}
