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
        debug_log: Option<&DebugLog>
    ) -> (Vec<RssArticle>, Option<String>) {
        let rule_data = RuleData::new();
        let analyze_url = AnalyzeUrl::new(
            sort_url,
            page,
            rss_source,
            &rule_data,
            rss_source.get_header_map(),
            debug_log
        );
        let body = analyze_url.get_str_response_await().body;
        // debugLog?.log(rssSource.sourceUrl, "┌获取链接内容:${sortUrl}")
        // debugLog?.log(rssSource.sourceUrl, "└\n${body}")
        return RssParserByRule::parse_xml(sort_name, sort_url, &body, rss_source, &rule_data, debug_log);
    }

    pub async fn get_content(
        rss_article: &RssArticle,
        rule_content: &str,
        rss_source: &RssSource,
        debug_log: Option<&DebugLog>
    ) -> String {
        let analyze_url = AnalyzeUrl::new(
            &rss_article.link,
            &rss_article.origin,
            rss_source,
            rss_article,
            rss_source.get_header_map(),
            debug_log
        );
        let body = analyze_url.get_str_response_await().body;
        // debugLog?.log(rssSource.sourceUrl, "┌获取链接内容:${rssArticle.link}")
        // debugLog?.log(rssSource.sourceUrl, "└\n${body}")
        let analyze_rule = AnalyzeRule::new(rss_article, rss_source, debug_log);
        analyze_rule.set_content(&body)
            .set_base_url(get_absolute_url(&rss_article.origin, &rss_article.link));
        return analyze_rule.get_string(rule_content);
    }
}
