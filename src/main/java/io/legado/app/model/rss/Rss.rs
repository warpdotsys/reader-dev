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
        // fix: Kotlin AnalyzeUrl(mUrl=sortUrl, page=page, source=rssSource, ruleData=ruleData,
        //      headerMapF=rssSource.getHeaderMap(), debugLog=debugLog)——ruleData 真实传入
        //      （原 None：sortUrl 的 {{put:}} 变量丢失）；source 因类型限制（AnalyzeUrl 仅收 BookSource）置空
        let mut analyze_url = AnalyzeUrl::new(
            sort_url.to_string(),
            None,
            Some(page),
            None,
            None,
            String::new(),
            None,
            Some(Box::new(rule_data.clone())),
            None,
            rss_source.get_header_map(),
            debug_log.map(|dl| dl.clone_box())
        );
        // fix: 启用 RSS 源 cookie jar（原 source 占位恒 false——需登录的 RSS 源抓取失败）
        analyze_url.set_cookie_enabled(rss_source.enabled_cookie_jar.unwrap_or(false));
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
        // fix: Kotlin AnalyzeUrl(mUrl=rssArticle.link, baseUrl=rssArticle.origin, source=rssSource,
        //      ruleData=rssArticle, headerMapF=rssSource.getHeaderMap(), debugLog=debugLog)——
        //      原参数错位（origin 塞进 key、base_url 空串→相对 link 抓取失败）；
        //      ruleData 真实传入（{{put:}} 变量）；source 因类型限制置空
        let mut analyze_url = AnalyzeUrl::new(
            rss_article.link.clone(),
            None,
            None,
            None,
            None,
            rss_article.origin.clone(),
            None,
            Some(Box::new(rss_article.clone())),
            None,
            rss_source.get_header_map(),
            debug_log.map(|dl| dl.clone_box())
        );
        // fix: 启用 RSS 源 cookie jar（原 source 占位恒 false——需登录的 RSS 源抓取失败）
        analyze_url.set_cookie_enabled(rss_source.enabled_cookie_jar.unwrap_or(false));
        let body = analyze_url.get_str_response_await(None, None, false).await.body().cloned();
        // debugLog?.log(rssSource.sourceUrl, "┌获取链接内容:${rssArticle.link}")
        // debugLog?.log(rssSource.sourceUrl, "└\n${body}")
        // fix: E0596 set_content/get_string 需要 &mut self
        let mut analyze_rule = AnalyzeRule::new(&*rss_article, None, debug_log);
        analyze_rule.set_content(body.map(|s| Box::new(Any::Str(s))), None)
            // fix: 基准 = origin 解析 link（Kotlin NetworkUtils.getAbsoluteURL(rssArticle.origin, rssArticle.link)；
            //      原 get_absolute_url(None, link)——相对 link 基准错误）
            .set_base_url(Some(get_absolute_url(crate::stubs::URL::parse(&rss_article.origin).ok().as_ref(), rss_article.link.clone())));
        return analyze_rule.get_string(Some(rule_content.to_string()), None, false);
    }
}
