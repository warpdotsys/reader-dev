use crate::prelude::*;
// package io.legado.app.model.analyzeRule

pub trait RuleDataInterface {
    fn variable_map(&self) -> &HashMap<String, String>;

    fn get_user_name_space(&self) -> String;

    fn put_variable(&mut self, key: &str, value: Option<&str>);

    fn get_variable(&self, key: &str) -> Option<String> {
        self.variable_map().get(key).cloned()
    }

    // fix: 同步已解析字段到规则数据本体（搜索条目 {{bookName}} 等自引用；默认空实现，SearchBook 覆盖）
    fn set_field(&mut self, _key: &str, _value: String) {}

    // fix: 转录需要 `ruleData as? Book/BaseBook` 下转型; 默认实现返回非目标类型(恒 None 结果)
    fn as_any(&self) -> &dyn std::any::Any {
        static FALLBACK: i32 = 0;
        &FALLBACK
    }
}
