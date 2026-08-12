// 真实 HTML 解析（scraper 封装），供 stubs Document/Element/Jsoup 使用

use crate::stubs::{Element, Elements};

/// HTML 纯文本
pub fn text_of(html: &str) -> String {
    let doc = scraper::Html::parse_fragment(html);
    doc.root_element().text().collect::<String>()
}

/// 第一个元素的 HTML 与纯文本
pub fn first_element(html: &str) -> (String, String) {
    let doc = scraper::Html::parse_fragment(html);
    let mut it = doc.root_element().children().filter_map(|c| scraper::ElementRef::wrap(c));
    if let Some(el) = it.next() {
        return (el.html().to_string(), el.text().collect::<String>());
    }
    (String::new(), String::new())
}

/// 属性
pub fn attr_of(html: &str, name: &str) -> String {
    let doc = scraper::Html::parse_fragment(html);
    let mut it = doc.root_element().children().filter_map(|c| scraper::ElementRef::wrap(c));
    if let Some(el) = it.next() {
        return el.attr(name).unwrap_or("").to_string();
    }
    String::new()
}

pub fn has_attr(html: &str, name: &str) -> bool {
    let doc = scraper::Html::parse_fragment(html);
    let mut it = doc.root_element().children().filter_map(|c| scraper::ElementRef::wrap(c));
    if let Some(el) = it.next() {
        return el.attr(name).is_some();
    }
    false
}

pub fn tag_name_of(html: &str) -> String {
    let doc = scraper::Html::parse_fragment(html);
    let mut it = doc.root_element().children().filter_map(|c| scraper::ElementRef::wrap(c));
    if let Some(el) = it.next() {
        return el.value().name().to_string();
    }
    String::new()
}

/// CSS 选择器
pub fn select_elements(html: &str, css: &str) -> Elements {
    let doc = scraper::Html::parse_fragment(html);
    let Ok(sel) = scraper::Selector::parse(css) else {
        return Elements::default();
    };
    let mut list = Vec::new();
    for e in doc.select(&sel) {
        list.push(Element {
            text: e.text().collect::<String>(),
            html: e.html().to_string(),
        });
    }
    Elements { list }
}

/// 直接子元素（第一层）
pub fn children_of(html: &str) -> Elements {
    let doc = scraper::Html::parse_fragment(html);
    let mut list = Vec::new();
    for child in doc.root_element().children() {
        if let Some(el) = scraper::ElementRef::wrap(child) {
            let h = el.html().to_string();
            list.push(Element {
                text: text_of(&h),
                html: h,
            });
        }
    }
    Elements { list }
}

/// 内层 HTML（去掉最外层标签）
pub fn inner_html_of(html: &str) -> String {
    let doc = scraper::Html::parse_fragment(html);
    let mut it = doc.root_element().children().filter_map(|c| scraper::ElementRef::wrap(c));
    if let Some(el) = it.next() {
        return el.inner_html().to_string();
    }
    html.to_string()
}

/// document title
pub fn title_of(html: &str) -> String {
    let doc = scraper::Html::parse_document(html);
    let sel = match scraper::Selector::parse("title") {
        Ok(s) => s,
        Err(_) => return String::new(),
    };
    doc.select(&sel)
        .next()
        .map(|e| e.text().collect::<String>())
        .unwrap_or_default()
}

/// body 元素
pub fn body_of(html: &str) -> (String, String) {
    let doc = scraper::Html::parse_document(html);
    let sel = match scraper::Selector::parse("body") {
        Ok(s) => s,
        Err(_) => return (String::new(), String::new()),
    };
    match doc.select(&sel).next() {
        Some(e) => (e.html().to_string(), e.text().collect::<String>()),
        None => (html.to_string(), text_of(html)),
    }
}
