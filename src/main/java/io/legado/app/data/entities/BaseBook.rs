use crate::prelude::*;
// package io.legado.app.data.entities

// import io.legado.app.model.analyzeRule.RuleDataInterface
// import io.legado.app.utils.splitNotBlank

pub trait BaseBook : RuleDataInterface {
    fn name(&self) -> &str;
    fn set_name(&mut self, value: String);
    fn author(&self) -> &str;
    fn set_author(&mut self, value: String);
    fn book_url(&self) -> &str;
    fn set_book_url(&mut self, value: String);
    fn kind(&self) -> Option<&str>;
    fn set_kind(&mut self, value: Option<String>);
    fn word_count(&self) -> Option<&str>;
    fn set_word_count(&mut self, value: Option<String>);

    fn info_html(&self) -> Option<&str>;
    fn set_info_html(&mut self, value: Option<String>);
    fn toc_html(&self) -> Option<&str>;
    fn set_toc_html(&mut self, value: Option<String>);

    fn get_kind_list(&self) -> Vec<String> {
        let mut kind_list: Vec<String> = Vec::new();
        if let Some(it) = self.word_count() {
            if !it.trim().is_empty() { kind_list.push(it.to_string()) }
        }
        if let Some(it) = self.kind() {
            // fix: splitNotBlank(",", "\n") → 按两个分隔符拆分并过滤空白（stubs split_not_blank 仅单分隔符）
            let kinds: Vec<String> = it.split(&[',', '\n'][..])
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.to_string())
                .collect();
            kind_list.extend(kinds);
        }
        kind_list
    }
}
