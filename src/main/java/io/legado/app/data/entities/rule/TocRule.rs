use crate::prelude::*;
// package io.legado.app.data.entities.rule

// data class TocRule(
//     var preUpdateJs: String? = None,
//     var chapterList: String? = None,
//     var chapterName: String? = None,
//     var chapterUrl: String? = None,
//     var isVolume: String? = None,
//     var isVip: String? = None,
//     var updateTime: String? = None,
//     var nextTocUrl: String? = None
// )
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TocRule {
    pub pre_update_js: Option<String>,
    pub chapter_list: Option<String>,
    pub chapter_name: Option<String>,
    pub chapter_url: Option<String>,
    pub is_volume: Option<String>,
    pub is_vip: Option<String>,
    pub update_time: Option<String>,
    pub next_toc_url: Option<String>,
}

impl Default for TocRule {
    fn default() -> Self {
        TocRule {
            pre_update_js: None,
            chapter_list: None,
            chapter_name: None,
            chapter_url: None,
            is_volume: None,
            is_vip: None,
            update_time: None,
            next_toc_url: None,
        }
    }
}

impl PartialEq for TocRule {
    fn eq(&self, other: &Self) -> bool {
        self.pre_update_js == other.pre_update_js
            && self.chapter_list == other.chapter_list
            && self.chapter_name == other.chapter_name
            && self.chapter_url == other.chapter_url
            && self.is_volume == other.is_volume
            && self.is_vip == other.is_vip
            && self.update_time == other.update_time
            && self.next_toc_url == other.next_toc_url
    }
}

impl Eq for TocRule {}
