// package com.htmake.reader.api.controller

// private val logger = KotlinLogging.logger {}

// class ReplaceRuleController(coroutineContext: CoroutineContext): BaseController(coroutineContext), CURD<ReplaceRule> {
pub struct ReplaceRuleController {
    base: BaseController,
}

impl ReplaceRuleController {
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
    fn checker(&self, json: &JsonObject, entity: &ReplaceRule) -> bool {
        return entity.name == json.get_string("name");
    }

    // override fun beforeSave(entity: ReplaceRule, db: DB<ReplaceRule>): ReturnData? {
    //     if (entity.name.isEmpty()) {
    //         return ReturnData().setErrorMsg("名称不能为空")
    //     }
    //     if (entity.pattern.isEmpty()) {
    //         return ReturnData().setErrorMsg("规则不能为空")
    //     }
    //     return null
    // }
    fn before_save(&self, entity: &ReplaceRule, db: &DB<ReplaceRule>) -> Option<ReturnData> {
        if entity.name.is_empty() {
            return Some(ReturnData::new().set_error_msg(String::from("名称不能为空")).clone());
        }
        if entity.pattern.is_empty() {
            return Some(ReturnData::new().set_error_msg(String::from("规则不能为空")).clone());
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
