use crate::prelude::*;
// 显式导入消解跨模块 glob 导入歧义（stubs GSON 提供 to_json）
use crate::stubs::GSON;
// package io.legado.app.model.analyzeRule
// import io.legado.app.utils.GSON

pub struct RuleData {
    variable_map: HashMap<String, String>,
}

// fix: Rss.get_articles 以副本充当 AnalyzeUrl 规则数据（{{put:}} 变量在解析后读回）
impl Clone for RuleData {
    fn clone(&self) -> Self {
        RuleData {
            variable_map: self.variable_map.clone(),
        }
    }
}

impl RuleDataInterface for RuleData {
    fn variable_map(&self) -> &HashMap<String, String> {
        &self.variable_map
    }

    fn put_variable(&mut self, key: &str, value: Option<&str>) {
        if value.is_none() {
            self.variable_map.remove(key);
        } else {
            self.variable_map.insert(key.to_string(), value.unwrap().to_string());
        }
    }

    fn get_user_name_space(&self) -> String {
        "unknow".to_string()
    }
}

impl RuleData {
    fn get_variable(&self) -> Option<String> {
        if self.variable_map.is_empty() {
            return None;
        }
        Some(GSON::to_json(&self.variable_map))
    }
}
