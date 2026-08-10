// package io.legado.app.model.rss
//
// import io.legado.app.data.entities.RssArticle
// import io.legado.app.data.entities.RssSource
// import io.legado.app.exception.NoStackTraceException
// import io.legado.app.model.DebugLog
// import io.legado.app.model.analyzeRule.AnalyzeRule
// import io.legado.app.model.analyzeRule.RuleData
// import io.legado.app.utils.NetworkUtils
// import java.util.*

pub struct RssParserByRule;

impl RssParserByRule {

    // @Throws(Exception::class)
    pub fn parse_xml(
        sort_name: &str,
        sort_url: &str,
        body: Option<&str>,
        rss_source: &RssSource,
        rule_data: &RuleData,
        debug_log: Option<&DebugLog>
    ) -> (Vec<RssArticle>, Option<String>) {
        let source_url = rss_source.source_url.clone();
        let mut next_url: Option<String> = None;
        if body.is_none() || body.unwrap().is_blank() {
            panic!("error_get_web_content: {}", rss_source.source_url);
        }
        // debugLog?.log(sourceUrl, "≡获取成功:$sourceUrl")
        // debugLog?.log(sourceUrl, body)
        let mut rule_articles = rss_source.rule_articles.clone();
        if rule_articles.is_blank() {
            if let Some(dl) = debug_log {
                dl.log(&source_url, "⇒列表规则为空, 使用默认规则解析");
            }
            return RssParserDefault::parse_xml(sort_name, body.unwrap(), &source_url, debug_log);
        } else {
            let mut article_list = Vec::<RssArticle>::new();
            let mut analyze_rule = AnalyzeRule::new(rule_data, rss_source, debug_log);
            analyze_rule.set_content(body.unwrap()).set_base_url(sort_url);
            analyze_rule.set_redirect_url(sort_url);
            let mut reverse = false;
            if rule_articles.starts_with("-") {
                reverse = true;
                rule_articles = rule_articles[1..].to_string();
            }
            if let Some(dl) = debug_log {
                dl.log(&source_url, "┌获取列表");
            }
            let collections = analyze_rule.get_elements(&rule_articles);
            if let Some(dl) = debug_log {
                dl.log(&source_url, &format!("└列表大小:{}", collections.len()));
            }
            if !rss_source.rule_next_page.is_empty() {
                if let Some(dl) = debug_log {
                    dl.log(&source_url, "┌获取下一页链接");
                }
                if rss_source.rule_next_page.to_uppercase() == "PAGE" {
                    next_url = Some(sort_url.to_string());
                } else {
                    next_url = Some(analyze_rule.get_string(&rss_source.rule_next_page));
                    if !next_url.clone().unwrap().is_empty() {
                        next_url = Some(get_absolute_url(sort_url, &next_url.unwrap()));
                    }
                }
                if let Some(dl) = debug_log {
                    dl.log(&source_url, &format!("└{}", next_url.clone().unwrap_or_default()));
                }
            }
            let rule_title = analyze_rule.split_source_rule(&rss_source.rule_title);
            let rule_pub_date = analyze_rule.split_source_rule(&rss_source.rule_pub_date);
            let rule_description = analyze_rule.split_source_rule(&rss_source.rule_description);
            let rule_image = analyze_rule.split_source_rule(&rss_source.rule_image);
            let rule_link = analyze_rule.split_source_rule(&rss_source.rule_link);
            let variable = rule_data.get_variable();
            for (index, item) in collections.iter().enumerate() {
                if let Some(mut it) = Self::get_item(
                    &source_url, item, &mut analyze_rule, &variable, index == 0,
                    &rule_title, &rule_pub_date, &rule_description, &rule_image, &rule_link, debug_log
                ) {
                    it.sort = sort_name.to_string();
                    it.origin = source_url.clone();
                    article_list.push(it);
                }
            }
            if reverse {
                article_list.reverse();
            }
            return (article_list, next_url);
        }
    }

    fn get_item(
        source_url: &str,
        item: &Box<dyn Any>,
        analyze_rule: &mut AnalyzeRule,
        variable: &str,
        log: bool,
        rule_title: &Vec<AnalyzeRule::SourceRule>,
        rule_pub_date: &Vec<AnalyzeRule::SourceRule>,
        rule_description: &Vec<AnalyzeRule::SourceRule>,
        rule_image: &Vec<AnalyzeRule::SourceRule>,
        rule_link: &Vec<AnalyzeRule::SourceRule>,
        debug_log: Option<&DebugLog>
    ) -> Option<RssArticle> {
        let mut rss_article = RssArticle::new(variable);
        analyze_rule.rule_data = rss_article;
        analyze_rule.set_content(item);
        if let Some(dl) = debug_log {
            dl.log(source_url, "┌获取标题", log);
        }
        rss_article.title = analyze_rule.get_string(rule_title);
        if let Some(dl) = debug_log {
            dl.log(source_url, &format!("└{}", rss_article.title), log);
            dl.log(source_url, "┌获取时间", log);
        }
        rss_article.pub_date = analyze_rule.get_string(rule_pub_date);
        if let Some(dl) = debug_log {
            dl.log(source_url, &format!("└{}", rss_article.pub_date), log);
            dl.log(source_url, "┌获取描述", log);
        }
        if rule_description.is_empty() {
            rss_article.description = None;
            if let Some(dl) = debug_log {
                dl.log(source_url, "└描述规则为空，将会解析内容页", log);
            }
        } else {
            rss_article.description = Some(analyze_rule.get_string(rule_description));
            if let Some(dl) = debug_log {
                dl.log(source_url, &format!("└{}", rss_article.description.clone().unwrap_or_default()), log);
            }
        }
        if let Some(dl) = debug_log {
            dl.log(source_url, "┌获取图片url", log);
        }
        rss_article.image = Some(analyze_rule.get_string(rule_image, true));
        if let Some(dl) = debug_log {
            dl.log(source_url, &format!("└{}", rss_article.image.clone().unwrap_or_default()), log);
            dl.log(source_url, "┌获取文章链接", log);
        }
        rss_article.link = get_absolute_url(source_url, &analyze_rule.get_string(rule_link));
        if let Some(dl) = debug_log {
            dl.log(source_url, &format!("└{}", rss_article.link), log);
        }
        if rss_article.title.is_blank() {
            return None;
        }
        return Some(rss_article);
    }
}
