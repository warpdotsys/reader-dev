use crate::prelude::*;
use crate::io_legado_app_model_analyzerule_analyzerule::SourceRule;
use crate::stubs::{Any, GSON};
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
        debug_log: Option<&dyn DebugLog>
    ) -> (Vec<RssArticle>, Option<String>) {
        let source_url = rss_source.source_url.clone();
        let mut next_url: Option<String> = None;
        if body.is_none() || body.unwrap().is_blank() {
            panic!("error_get_web_content: {}", rss_source.source_url);
        }
        // debugLog?.log(sourceUrl, "≡获取成功:$sourceUrl")
        // debugLog?.log(sourceUrl, body)
        let mut rule_articles = rss_source.rule_articles.clone();
        if rule_articles.as_deref().map_or(true, |s| s.is_blank()) {
            if let Some(dl) = debug_log {
                dl.log(Some(source_url.as_str()), Some("⇒列表规则为空, 使用默认规则解析"), false);
            }
            return RssParserDefault::parse_xml(sort_name, body.unwrap(), &source_url, debug_log);
        } else {
            let mut article_list = Vec::<RssArticle>::new();
            let mut analyze_rule = AnalyzeRule::new(rule_data, None, debug_log);
            analyze_rule.set_content(Some(Box::new(Any::Str(body.unwrap().to_string()))), None).set_base_url(Some(sort_url.to_string()));
            analyze_rule.set_redirect_url(sort_url.to_string());
            let mut reverse = false;
            if rule_articles.as_deref().map_or(false, |s| s.starts_with("-")) {
                reverse = true;
                rule_articles = Some(rule_articles.as_deref().unwrap()[1..].to_string());
            }
            if let Some(dl) = debug_log {
                dl.log(Some(source_url.as_str()), Some("┌获取列表"), false);
            }
            let collections = analyze_rule.get_elements(rule_articles.clone().unwrap_or_default());
            if let Some(dl) = debug_log {
                dl.log(Some(source_url.as_str()), Some(format!("└列表大小:{}", collections.len()).as_str()), false);
            }
            if rss_source.rule_next_page.as_ref().map_or(false, |s| !s.is_empty()) {
                if let Some(dl) = debug_log {
                    dl.log(Some(source_url.as_str()), Some("┌获取下一页链接"), false);
                }
                if rss_source.rule_next_page.clone().unwrap_or_default().to_uppercase() == "PAGE" {
                    next_url = Some(sort_url.to_string());
                } else {
                    next_url = Some(analyze_rule.get_string(rss_source.rule_next_page.clone(), None, false));
                    if !next_url.clone().unwrap().is_empty() {
                        // fix: get_absolute_url 占位签名 (Option<&URL>, String)——原 Kotlin 传 baseUrl(sortUrl)
                        next_url = Some(get_absolute_url(crate::stubs::URL::parse(sort_url).ok().as_ref(), next_url.unwrap()));
                    }
                }
                if let Some(dl) = debug_log {
                    dl.log(Some(source_url.as_str()), Some(format!("└{}", next_url.clone().unwrap_or_default()).as_str()), false);
                }
            }
            let rule_title = analyze_rule.split_source_rule(rss_source.rule_title.clone(), false);
            let rule_pub_date = analyze_rule.split_source_rule(rss_source.rule_pub_date.clone(), false);
            let rule_description = analyze_rule.split_source_rule(rss_source.rule_description.clone(), false);
            let rule_image = analyze_rule.split_source_rule(rss_source.rule_image.clone(), false);
            let rule_link = analyze_rule.split_source_rule(rss_source.rule_link.clone(), false);
            let variable = if rule_data.variable_map().is_empty() {
                None
            } else {
                Some(GSON::to_json(rule_data.variable_map()))
            };
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
        item: &Box<Any>,
        analyze_rule: &mut AnalyzeRule,
        variable: &Option<String>,
        log: bool,
        rule_title: &Vec<SourceRule>,
        rule_pub_date: &Vec<SourceRule>,
        rule_description: &Vec<SourceRule>,
        rule_image: &Vec<SourceRule>,
        rule_link: &Vec<SourceRule>,
        debug_log: Option<&dyn DebugLog>
    ) -> Option<RssArticle> {
        let mut rss_article = RssArticle {
            variable: variable.clone(),
            ..Default::default()
        };
        analyze_rule.rule_data = Box::new(rss_article.clone());
        analyze_rule.set_content(Some(item.clone()), None);
        if let Some(dl) = debug_log {
            dl.log(Some(source_url), Some("┌获取标题"), log);
        }
        rss_article.title = analyze_rule.get_string_inner(rule_title.clone(), None, false);
        if let Some(dl) = debug_log {
            dl.log(Some(source_url), Some(format!("└{}", rss_article.title).as_str()), log);
            dl.log(Some(source_url), Some("┌获取时间"), log);
        }
        rss_article.pub_date = Some(analyze_rule.get_string_inner(rule_pub_date.clone(), None, false));
        if let Some(dl) = debug_log {
            dl.log(Some(source_url), Some(format!("└{}", rss_article.pub_date.clone().unwrap_or_default()).as_str()), log);
            dl.log(Some(source_url), Some("┌获取描述"), log);
        }
        if rule_description.is_empty() {
            rss_article.description = None;
            if let Some(dl) = debug_log {
                dl.log(Some(source_url), Some("└描述规则为空，将会解析内容页"), log);
            }
        } else {
            rss_article.description = Some(analyze_rule.get_string_inner(rule_description.clone(), None, false));
            if let Some(dl) = debug_log {
                dl.log(Some(source_url), Some(format!("└{}", rss_article.description.clone().unwrap_or_default()).as_str()), log);
            }
        }
        if let Some(dl) = debug_log {
            dl.log(Some(source_url), Some("┌获取图片url"), log);
        }
        rss_article.image = Some(analyze_rule.get_string_inner(rule_image.clone(), None, true));
        if let Some(dl) = debug_log {
            dl.log(Some(source_url), Some(format!("└{}", rss_article.image.clone().unwrap_or_default()).as_str()), log);
            dl.log(Some(source_url), Some("┌获取文章链接"), log);
        }
        // fix: get_absolute_url 占位签名 (Option<&URL>, String)——原 Kotlin 传 baseUrl(sourceUrl)
        rss_article.link = get_absolute_url(crate::stubs::URL::parse(source_url).ok().as_ref(), analyze_rule.get_string_inner(rule_link.clone(), None, false));
        if let Some(dl) = debug_log {
            dl.log(Some(source_url), Some(format!("└{}", rss_article.link).as_str()), log);
        }
        if rss_article.title.is_blank() {
            return None;
        }
        return Some(rss_article);
    }
}
