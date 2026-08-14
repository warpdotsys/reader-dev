use crate::prelude::*;
// package io.legado.app.model.analyzeRule
// import java.util.*
// import java.util.regex.Pattern

// object AnalyzeByRegex
pub struct AnalyzeByRegex;

impl AnalyzeByRegex {
    pub fn get_element(res: &str, regs: &[String], index: usize) -> Option<List<String>> {
        let mut v_index = index;
        let haystack = res.to_string();
        let pattern = Pattern::compile(&regs[v_index]);
        let mut res_m = pattern.matcher(haystack);
        if !res_m.find() {
            return None;
        }
        // 判断索引的规则是最后一个规则
        return if v_index + 1 == regs.len() {
            // 新建容器
            let mut info = array_list_of::<String>();
            for group_index in 0..=res_m.group_count() {
                info.add(res_m.group_idx(group_index).unwrap());
            }
            Some(info)
        } else {
            let mut result = StringBuilder::new();
            loop {
                result.append(res_m.group());
                if !res_m.find() { break; }
            }
            Self::get_element(&result.to_string(), regs, { v_index += 1; v_index })
        }
    }

    pub fn get_elements(res: &str, regs: &[String], index: usize) -> List<List<String>> {
        let mut v_index = index;
        let haystack = res.to_string();
        let pattern = Pattern::compile(&regs[v_index]);
        let mut res_m = pattern.matcher(haystack);
        if !res_m.find() {
            return array_list_of();
        }
        // 判断索引的规则是最后一个规则
        if v_index + 1 == regs.len() {
            // 创建书息缓存数组
            let mut books = ArrayList::<List<String>>::new();
            // 提取列表
            loop {
                // 新建容器
                let mut info = array_list_of::<String>();
                for group_index in 0..=res_m.group_count() {
                    info.add(res_m.group_idx(group_index).unwrap_or_default());
                }
                books.add(info);
                if !res_m.find() { break; }
            }
            return books;
        } else {
            let mut result = StringBuilder::new();
            loop {
                result.append(res_m.group());
                if !res_m.find() { break; }
            }
            return Self::get_elements(&result.to_string(), regs, { v_index += 1; v_index });
        }
    }
}
