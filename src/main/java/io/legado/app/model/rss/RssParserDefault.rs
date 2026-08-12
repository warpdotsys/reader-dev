use crate::prelude::*;
// 显式导入消解跨模块 glob 导入歧义（XmlPullParserFactory 与 EpubProcessorSupport 同名；
// XmlPullParser 为 kdom trait，事件常量改指 KXmlParser 关联常量）
use crate::stubs::{StringReader, XmlPullParserFactory};
use crate::org_kxml2_io_kxmlparser::KXmlParser;
// package io.legado.app.model.rss
//
// import io.legado.app.data.entities.RssArticle
// import io.legado.app.model.DebugLog
// import org.xmlpull.v1.XmlPullParser
// import org.xmlpull.v1.XmlPullParserException
// import org.xmlpull.v1.XmlPullParserFactory
// import java.io.IOException
// import java.io.StringReader

// @Suppress("unused")
pub struct RssParserDefault;

impl RssParserDefault {

    // @Throws(XmlPullParserException::class, IOException::class)
    pub fn parse_xml(
        sort_name: &str,
        xml: &str,
        source_url: &str,
        debug_log: Option<&dyn DebugLog>
    ) -> (Vec<RssArticle>, Option<String>) {

        let mut article_list = Vec::<RssArticle>::new();
        let mut current_article = RssArticle::default();

        // val factory = XmlPullParserFactory.newInstance()
        // val factory = XmlPullParserFactory.newInstance("""
        // org.kxml2.io.KXmlParser
        // org.kxml2.io.KXmlSerializer
        //        """, Thread.currentThread().getContextClassLoader().javaClass)
        let mut factory = XmlPullParserFactory::new_instance();
        factory.is_namespace_aware = false;

        let mut xml_pull_parser = factory.new_pull_parser();
        xml_pull_parser.set_input(Some(Box::new(StringReader::new(xml))));

        // A flag just to be sure of the correct parsing
        let mut inside_item = false;

        let mut event_type = xml_pull_parser.type_;

        // Start parsing the xml
        'loop_label: while event_type != KXmlParser::END_DOCUMENT {

            // Start parsing the item
            if event_type == KXmlParser::START_TAG {
                if xml_pull_parser.name.as_deref().unwrap_or_default().eq_ignore_ascii_case(Self::RSS_ITEM) {
                    inside_item = true;
                } else if xml_pull_parser.name.as_deref().unwrap_or_default().eq_ignore_ascii_case(Self::RSS_ITEM_TITLE) {
                    if inside_item {
                        current_article.title = xml_pull_parser.next_text().unwrap_or_default().trim().to_string();
                    }
                } else if xml_pull_parser.name.as_deref().unwrap_or_default().eq_ignore_ascii_case(Self::RSS_ITEM_LINK) {
                    if inside_item {
                        current_article.link = xml_pull_parser.next_text().unwrap_or_default().trim().to_string();
                    }
                } else if xml_pull_parser.name.as_deref().unwrap_or_default().eq_ignore_ascii_case(Self::RSS_ITEM_THUMBNAIL) {
                    if inside_item {
                        current_article.image = xml_pull_parser.get_attribute_value_named(None, Self::RSS_ITEM_URL.to_string());
                    }
                } else if xml_pull_parser.name.as_deref().unwrap_or_default().eq_ignore_ascii_case(Self::RSS_ITEM_ENCLOSURE) {
                    if inside_item {
                        let type_attr = xml_pull_parser.get_attribute_value_named(None, Self::RSS_ITEM_TYPE.to_string());
                        if type_attr.is_some() && type_attr.unwrap().contains("image/") {
                            current_article.image = xml_pull_parser.get_attribute_value_named(None, Self::RSS_ITEM_URL.to_string());
                        }
                    }
                } else if xml_pull_parser.name.as_deref().unwrap_or_default().eq_ignore_ascii_case(Self::RSS_ITEM_DESCRIPTION) {
                    if inside_item {
                        let description = xml_pull_parser.next_text().unwrap_or_default();
                        current_article.description = Some(description.trim().to_string());
                        if current_article.image.is_none() {
                            current_article.image = Self::get_image_url(&description);
                        }
                    }
                } else if xml_pull_parser.name.as_deref().unwrap_or_default().eq_ignore_ascii_case(Self::RSS_ITEM_CONTENT) {
                    if inside_item {
                        let content = xml_pull_parser.next_text().unwrap_or_default().trim().to_string();
                        current_article.content = Some(content.clone());
                        if current_article.image.is_none() {
                            current_article.image = Self::get_image_url(&content);
                        }
                    }
                } else if xml_pull_parser.name.as_deref().unwrap_or_default().eq_ignore_ascii_case(Self::RSS_ITEM_PUB_DATE) {
                    if inside_item {
                        let next_token_type = xml_pull_parser.next().unwrap_or_default();
                        if next_token_type == KXmlParser::TEXT {
                            current_article.pub_date = Some(xml_pull_parser.get_text().unwrap_or_default().trim().to_string());
                        }
                        // Skip to be able to find date inside 'tag' tag
                        continue 'loop_label;
                    }
                } else if xml_pull_parser.name.as_deref().unwrap_or_default().eq_ignore_ascii_case(Self::RSS_ITEM_TIME) {
                    if inside_item {
                        current_article.pub_date = Some(xml_pull_parser.next_text().unwrap_or_default());
                    }
                }
            } else if event_type == KXmlParser::END_TAG
                && xml_pull_parser.name.as_deref().unwrap_or_default().eq_ignore_ascii_case("item")
            {
                // The item is correctly parsed
                inside_item = false;
                current_article.origin = source_url.to_string();
                current_article.sort = sort_name.to_string();
                article_list.push(current_article);
                current_article = RssArticle::default();
            }
            event_type = xml_pull_parser.next().unwrap_or_default();
        }
        if let Some(it) = article_list.first() {
            if let Some(dl) = debug_log {
                dl.log(Some(source_url), Some("┌获取标题"), false);
                dl.log(Some(source_url), Some(&format!("└{}", it.title)), false);
                dl.log(Some(source_url), Some("┌获取时间"), false);
                dl.log(Some(source_url), Some(&format!("└{}", it.pub_date.clone().unwrap_or_default())), false);
                dl.log(Some(source_url), Some("┌获取描述"), false);
                dl.log(Some(source_url), Some(&format!("└{}", it.description.clone().unwrap_or_default())), false);
                dl.log(Some(source_url), Some("┌获取图片url"), false);
                dl.log(Some(source_url), Some(&format!("└{}", it.image.clone().unwrap_or_default())), false);
                dl.log(Some(source_url), Some("┌获取文章链接"), false);
                dl.log(Some(source_url), Some(&format!("└{}", it.link)), false);
            }
        }
        return (article_list, None);
    }

    /**
     * Finds the first img tag and get the src as featured image
     *
     * @param input The content in which to search for the tag
     * @return The url, if there is one
     */
    fn get_image_url(input: &str) -> Option<String> {

        let mut url: Option<String> = None;
        let pattern_img = Regex::new("(<img [^>]*>)").unwrap();
        let matcher_img = pattern_img.captures(input);
        if let Some(caps_img) = matcher_img {
            let img_tag = caps_img.get(1).map(|m| m.as_str());
            let pattern_link = Regex::new("src\\s*=\\s*\"([^\"]+)\"").unwrap();
            if let Some(link_caps) = pattern_link.captures(img_tag.unwrap()) {
                url = Some(link_caps.get(1).unwrap().as_str().trim().to_string());
            }
        }
        return url;
    }

    const RSS_ITEM: &str = "item";
    const RSS_ITEM_TITLE: &str = "title";
    const RSS_ITEM_LINK: &str = "link";
    const RSS_ITEM_CATEGORY: &str = "category";
    const RSS_ITEM_THUMBNAIL: &str = "media:thumbnail";
    const RSS_ITEM_ENCLOSURE: &str = "enclosure";
    const RSS_ITEM_DESCRIPTION: &str = "description";
    const RSS_ITEM_CONTENT: &str = "content:encoded";
    const RSS_ITEM_PUB_DATE: &str = "pubDate";
    const RSS_ITEM_TIME: &str = "time";
    const RSS_ITEM_URL: &str = "url";
    const RSS_ITEM_TYPE: &str = "type";
}
