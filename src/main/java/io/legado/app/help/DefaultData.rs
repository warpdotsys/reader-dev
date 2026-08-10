// package io.legado.app.help
//
// import io.legado.app.data.entities.RssSource
// import io.legado.app.data.entities.TxtTocRule
// import io.legado.app.utils.GSON
// import io.legado.app.utils.fromJsonArray
// import java.io.File

pub struct DefaultData;

impl DefaultData {
    pub const txt_toc_rule_file_name: &'static str = "txtTocRule.json";

    // val txtTocRules: List<TxtTocRule> by lazy {
    //     val json = String(DefaultData::class.java.getResource("/defaultData/${txtTocRuleFileName}").readBytes())
    //     GSON.fromJsonArray<TxtTocRule>(json).getOrNull() ?: emptyList()
    // }
    pub fn txt_toc_rules() -> &'static Vec<TxtTocRule> {
        use std::sync::OnceLock;
        static TXT_TOC_RULES: OnceLock<Vec<TxtTocRule>> = OnceLock::new();
        TXT_TOC_RULES.get_or_init(|| {
            let json = String::from_utf8_lossy(
                &DefaultData::class_resource(format!("/defaultData/{}", Self::txt_toc_rule_file_name))
                    .read_bytes()
                    .unwrap_or_default(),
            ).into_owned();
            GSON::from_json_array::<TxtTocRule>(&json).get_or_none().unwrap_or_else(Vec::new)
        })
    }

    // val rssSources by lazy {
    //     val json = String(
    //         File("defaultData${File.separator}rssSources.json")
    //             .readBytes()
    //     )
    //     GSON.fromJsonArray<RssSource>(json)!!
    // }
}
