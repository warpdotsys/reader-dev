use crate::prelude::*;
// 显式导入消解跨模块 glob 导入歧义（stubs Any 与 AnalyzeByJSoup.Any 同名）
use crate::stubs::Any;
// package io.legado.app.model.rss
//
// import io.legado.app.data.entities.RssArticle
// import io.legado.app.data.entities.RssSource
// import io.legado.app.model.DebugLog
// import io.legado.app.model.analyzeRule.AnalyzeRule
// import io.legado.app.model.analyzeRule.AnalyzeUrl
// import io.legado.app.model.analyzeRule.RuleData
// import io.legado.app.utils.NetworkUtils

pub struct Rss;

impl Rss {

    pub async fn get_articles(
        sort_name: &str,
        sort_url: &str,
        rss_source: &RssSource,
        page: i32,
        debug_log: Option<&dyn DebugLog>
    ) -> (Vec<RssArticle>, Option<String>) {
        let rule_data = RuleData::new();
        // fix: 原 Kotlin AnalyzeUrl(mUrl=sortUrl, page=page, source=rssSource, ruleData=ruleData,
        //      headerMapF=rssSource.getHeaderMap(), debugLog=debugLog)；转录 new 收所有权，
        //      source/ruleData 类型不匹配或需所有权 → 置空占位（同 analyze_url_new_placeholder 约定），
        //      debug_log 借用型无法转 Box，以 Debug 占位保持 is_some 语义
        let mut analyze_url = AnalyzeUrl::new(
            sort_url.to_string(),
            None,
            Some(page),
            None,
            None,
            String::new(),
            None,
            None,
            None,
            rss_source.get_header_map(),
            debug_log.map(|_| Box::new(Debug) as Box<dyn DebugLog>)
        );
        let body = analyze_url.get_str_response_await(None, None, false).await.body().cloned();
        // debugLog?.log(rssSource.sourceUrl, "┌获取链接内容:${sortUrl}")
        // debugLog?.log(rssSource.sourceUrl, "└\n${body}")
        return RssParserByRule::parse_xml(sort_name, sort_url, body.as_deref(), rss_source, &rule_data, debug_log);
    }

    pub async fn get_content(
        rss_article: &RssArticle,
        rule_content: &str,
        rss_source: &RssSource,
        debug_log: Option<&dyn DebugLog>
    ) -> String {
        // fix: 原 Kotlin AnalyzeUrl(mUrl=rssArticle.link, key=rssArticle.origin, source=rssSource,
        //      ruleData=rssArticle, headerMapF=rssSource.getHeaderMap(), debugLog=debugLog)；
        //      source/ruleData 需所有权置空占位，debug_log 以 Debug 占位
        let mut analyze_url = AnalyzeUrl::new(
            rss_article.link.clone(),
            Some(rss_article.origin.clone()),
            None,
            None,
            None,
            String::new(),
            None,
            None,
            None,
            rss_source.get_header_map(),
            debug_log.map(|_| Box::new(Debug) as Box<dyn DebugLog>)
        );
        let body = analyze_url.get_str_response_await(None, None, false).await.body().cloned();
        // debugLog?.log(rssSource.sourceUrl, "┌获取链接内容:${rssArticle.link}")
        // debugLog?.log(rssSource.sourceUrl, "└\n${body}")
        // fix: E0596 set_content/get_string 需要 &mut self
        let mut analyze_rule = AnalyzeRule::new(rss_article, rss_source, debug_log);
        analyze_rule.set_content(body.map(|s| Box::new(Any::Str(s))), None)
            // fix: get_absolute_url 占位签名 (Option<&URL>, String)——原 Kotlin 传 baseUrl(rssArticle.origin)
            .set_base_url(Some(get_absolute_url(None, rss_article.link.clone())));
        return analyze_rule.get_string(Some(rule_content.to_string()), None, false);
    }
}
