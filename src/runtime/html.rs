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
    // jsoup 伪类降级（:contains/:containsOwn/:eq/:gt/:lt/:first/:last——scraper 不支持）
    let (base_css, pseudos) = parse_jsoup_pseudo(css);
    let Ok(sel) = scraper::Selector::parse(&base_css) else {
        return Elements::default();
    };
    let mut list: Vec<Element> = Vec::new();
    for e in doc.select(&sel) {
        let mut pass = true;
        for p in &pseudos {
            match p {
                JsoupPseudo::Contains(t) => {
                    if !e.text().collect::<String>().contains(t.as_str()) {
                        pass = false;
                        break;
                    }
                }
                JsoupPseudo::ContainsOwn(t) => {
                    let direct: String = e
                        .children()
                        .filter_map(|c| match c.value() {
                            scraper::node::Node::Text(tx) => Some(tx.to_string()),
                            _ => None,
                        })
                        .collect();
                    if !direct.contains(t.as_str()) {
                        pass = false;
                        break;
                    }
                }
                _ => {}
            }
        }
        if pass {
            list.push(Element {
                text: e.text().collect::<String>(),
                html: e.html().to_string(),
            });
        }
    }
    // 索引类伪类（集合级）
    for p in &pseudos {
        match p {
            JsoupPseudo::Eq(n) => {
                list = list.into_iter().nth(*n).map(|x| vec![x]).unwrap_or_default();
            }
            JsoupPseudo::Gt(n) => {
                list = list.into_iter().skip(n + 1).collect();
            }
            JsoupPseudo::Lt(n) => {
                list.truncate(*n);
            }
            JsoupPseudo::First => {
                list = list.into_iter().take(1).collect();
            }
            JsoupPseudo::Last => {
                list = list.into_iter().rev().take(1).collect();
            }
            _ => {}
        }
    }
    Elements { list }
}

enum JsoupPseudo {
    Contains(String),
    ContainsOwn(String),
    Eq(usize),
    Gt(usize),
    Lt(usize),
    First,
    Last,
}

/// 从 CSS 尾部提取 jsoup 伪类（scraper 不支持），返回基础选择器 + 伪类列表
fn parse_jsoup_pseudo(css: &str) -> (String, Vec<JsoupPseudo>) {
    let mut base = css.trim().to_string();
    let mut pseudos = Vec::new();
    loop {
        let trimmed = base.trim_end();
        if let Some(idx) = trimmed.rfind(":contains(") {
            let close = trimmed[idx..].find(')').map(|c| idx + c + 1);
            if let Some(close) = close {
                let inner = &trimmed[idx + ":contains(".len()..close - 1];
                let text = inner.trim().trim_matches(['\'', '"']);
                pseudos.push(JsoupPseudo::Contains(text.to_string()));
                base = trimmed[..idx].to_string();
                continue;
            }
        }
        if let Some(idx) = trimmed.rfind(":containsOwn(") {
            let close = trimmed[idx..].find(')').map(|c| idx + c + 1);
            if let Some(close) = close {
                let inner = &trimmed[idx + ":containsOwn(".len()..close - 1];
                let text = inner.trim().trim_matches(['\'', '"']);
                pseudos.push(JsoupPseudo::ContainsOwn(text.to_string()));
                base = trimmed[..idx].to_string();
                continue;
            }
        }
        if let Some(idx) = trimmed.rfind(":eq(") {
            let close = trimmed[idx..].find(')').map(|c| idx + c + 1);
            if let Some(close) = close {
                let inner = &trimmed[idx + ":eq(".len()..close - 1];
                if let Ok(n) = inner.trim().parse::<usize>() {
                    pseudos.push(JsoupPseudo::Eq(n));
                    base = trimmed[..idx].to_string();
                    continue;
                }
            }
        }
        if let Some(idx) = trimmed.rfind(":gt(") {
            let close = trimmed[idx..].find(')').map(|c| idx + c + 1);
            if let Some(close) = close {
                let inner = &trimmed[idx + ":gt(".len()..close - 1];
                if let Ok(n) = inner.trim().parse::<usize>() {
                    pseudos.push(JsoupPseudo::Gt(n));
                    base = trimmed[..idx].to_string();
                    continue;
                }
            }
        }
        if let Some(idx) = trimmed.rfind(":lt(") {
            let close = trimmed[idx..].find(')').map(|c| idx + c + 1);
            if let Some(close) = close {
                let inner = &trimmed[idx + ":lt(".len()..close - 1];
                if let Ok(n) = inner.trim().parse::<usize>() {
                    pseudos.push(JsoupPseudo::Lt(n));
                    base = trimmed[..idx].to_string();
                    continue;
                }
            }
        }
        if trimmed.ends_with(":first") {
            pseudos.push(JsoupPseudo::First);
            base = trimmed[..trimmed.len() - ":first".len()].to_string();
            continue;
        }
        if trimmed.ends_with(":last") {
            pseudos.push(JsoupPseudo::Last);
            base = trimmed[..trimmed.len() - ":last".len()].to_string();
            continue;
        }
        break;
    }
    (base, pseudos)
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
