use crate::prelude::*;
// package io.legado.app.data.entities.rule

// data class ContentRule(
//     var content: String? = None,
//     var nextContentUrl: String? = None,
//     var webJs: String? = None,
//     var sourceRegex: String? = None,
//     var replaceRegex: String? = None, //替换规则
//     var imageStyle: String? = None,  //默认大小居中,FULL最大宽度
// )
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentRule {
    pub content: Option<String>,
    pub next_content_url: Option<String>,
    pub web_js: Option<String>,
    pub source_regex: Option<String>,
    pub replace_regex: Option<String>, //替换规则
    pub image_style: Option<String>,  //默认大小居中,FULL最大宽度
}

impl Default for ContentRule {
    fn default() -> Self {
        ContentRule {
            content: None,
            next_content_url: None,
            web_js: None,
            source_regex: None,
            replace_regex: None,
            image_style: None,
        }
    }
}

impl PartialEq for ContentRule {
    fn eq(&self, other: &Self) -> bool {
        self.content == other.content
            && self.next_content_url == other.next_content_url
            && self.web_js == other.web_js
            && self.source_regex == other.source_regex
            && self.replace_regex == other.replace_regex
            && self.image_style == other.image_style
    }
}

impl Eq for ContentRule {}
