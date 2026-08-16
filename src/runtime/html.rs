// 真实 HTML 解析（scraper 封装），供 stubs Document/Element/Jsoup 使用

use crate::stubs::{Element, Elements};

/// jsoup 块级元素（text() 在其后追加空格）——jsoup 的 appendWhitespaceIfBr 算法
fn is_jsoup_block_tag(tag: &str) -> bool {
    matches!(
        tag,
        "address" | "article" | "aside" | "blockquote" | "br" | "dd" | "div" | "dl" | "dt" | "fieldset"
            | "figcaption" | "figure" | "footer" | "form" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
            | "header" | "hr" | "li" | "main" | "nav" | "ol" | "p" | "pre" | "section" | "table" | "ul"
            | "tr" | "td" | "th"
    )
}

/// 递归收集文本（跳过 script/style；块级元素与 <br> 前后追加空格）
fn collect_jsoup_text(el: &scraper::ElementRef, out: &mut String) {
    for child in el.children() {
        match child.value() {
            scraper::node::Node::Text(t) => out.push_str(t),
            scraper::node::Node::Element(_) => {
                if let Some(ce) = scraper::ElementRef::wrap(child) {
                    let tag = ce.value().name().to_ascii_lowercase();
                    // jsoup text() 不含 script/style（DataNode）
                    if tag == "script" || tag == "style" {
                        continue;
                    }
                    if is_jsoup_block_tag(&tag) {
                        out.push(' ');
                    }
                    collect_jsoup_text(&ce, out);
                    if is_jsoup_block_tag(&tag) {
                        out.push(' ');
                    }
                }
            }
            _ => {}
        }
    }
}

/// jsoup 文本规范化：空白折叠（换行/制表→空格、连续空白合并）+ trim + 块级/<br> 分隔
/// fix: 原 scraper e.text() 原始拼接（<p>段一</p><p>段二</p>→"段一段二"、br 不换行、script 混入）
pub fn jsoup_normalise_text(el: &scraper::ElementRef) -> String {
    let mut raw = String::new();
    collect_jsoup_text(el, &mut raw);
    let mut res = String::new();
    let mut last_space = false;
    for c in raw.chars() {
        if c.is_whitespace() {
            if !last_space {
                res.push(' ');
                last_space = true;
            }
        } else {
            res.push(c);
            last_space = false;
        }
    }
    res.trim().to_string()
}

/// jsoup ownText()：仅直接文本子节点（规范化 + trim）
pub fn own_text_of(html: &str) -> String {
    let doc = scraper::Html::parse_fragment(html);
    let mut raw = String::new();
    for child in doc.root_element().children() {
        if let scraper::node::Node::Text(t) = child.value() {
            raw.push_str(t);
        }
    }
    let mut res = String::new();
    let mut last_space = false;
    for c in raw.chars() {
        if c.is_whitespace() {
            if !last_space {
                res.push(' ');
                last_space = true;
            }
        } else {
            res.push(c);
            last_space = false;
        }
    }
    res.trim().to_string()
}

/// 剔除 <script>/<style> 块（jsoup Elements.select(...).remove() 语义；Element 仅有 html 字符串）
pub fn strip_script_style_html(html: &str) -> String {
    let re_script = fancy_regex::Regex::new(r"(?is)<script[^>]*>.*?</script>").unwrap();
    let re_style = fancy_regex::Regex::new(r"(?is)<style[^>]*>.*?</style>").unwrap();
    let s = re_script.replace_all(html, "");
    re_style.replace_all(&s, "").to_string()
}

/// HTML 纯文本（jsoup 规范化）
pub fn text_of(html: &str) -> String {
    let doc = scraper::Html::parse_fragment(html);
    jsoup_normalise_text(&doc.root_element())
}

/// 第一个元素的 HTML 与纯文本
pub fn first_element(html: &str) -> (String, String) {
    let doc = scraper::Html::parse_fragment(html);
    let mut it = doc.root_element().children().filter_map(|c| scraper::ElementRef::wrap(c));
    if let Some(el) = it.next() {
        return (el.html().to_string(), jsoup_normalise_text(&el));
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
    // fix: 中间伪类（div:contains(x) span）——先整体尝试 scraper（标准伪类如 :first-child 直接可用）；
    //      失败再降级剥离 jsoup 伪类
    if let Ok(sel) = scraper::Selector::parse(css) {
        let mut list: Vec<Element> = Vec::new();
        for e in doc.select(&sel) {
            list.push(Element {
                text: jsoup_normalise_text(&e),
                html: e.html().to_string(),
            });
        }
        return Elements { list };
    }
    // jsoup 伪类降级（:contains/:containsOwn/:eq/:gt/:lt/:first/:last——scraper 不支持）
    let (base_css, pseudos, tail) = parse_jsoup_pseudo(css);
    let Ok(sel) = scraper::Selector::parse(&base_css) else {
        // fix: 无效选择器打日志（原静默空——拼错选择器难以排查；Kotlin 抛 SelectorParseException）
        eprintln!("[jsoup] 无效选择器: {:?} (原规则: {})", base_css, css);
        return Elements::default();
    };
    let mut list: Vec<Element> = Vec::new();
    for e in doc.select(&sel) {
        let mut pass = true;
        for p in &pseudos {
            match p {
                JsoupPseudo::Contains(t) => {
                    // fix: jsoup :contains 大小写不敏感、基于规范化文本
                    if !jsoup_normalise_text(&e).to_lowercase().contains(&t.to_lowercase()) {
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
                    if !direct.to_lowercase().contains(&t.to_lowercase()) {
                        pass = false;
                        break;
                    }
                }
                // fix: :has(selector)——元素包含匹配后代（jsoup 语义）
                JsoupPseudo::Has(sel_str) => {
                    if let Ok(sel) = scraper::Selector::parse(sel_str) {
                        if e.select(&sel).next().is_none() {
                            pass = false;
                            break;
                        }
                    }
                }
                _ => {}
            }
        }
        if pass {
            list.push(Element {
                text: jsoup_normalise_text(&e),
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
    // fix: 中间伪类后的后缀选择器（div:contains(x) span → 过滤后对每个元素继续选择 span）
    if let Some(tail) = tail {
        let mut final_list = Vec::new();
        for el in list {
            final_list.extend(el.select(&tail).list);
        }
        list = final_list;
    }
    Elements { list }
}

enum JsoupPseudo {
    Contains(String),
    ContainsOwn(String),
    Has(String),
    Eq(usize),
    Gt(usize),
    Lt(usize),
    First,
    Last,
}

/// 从 CSS 尾部提取 jsoup 伪类（scraper 不支持），返回基础选择器 + 伪类列表 + 伪类后的后缀选择器
/// fix: 原实现 rfind 尾部剥离——中间伪类（div:contains(x) span）把伪类后的内容一并丢弃
fn parse_jsoup_pseudo(css: &str) -> (String, Vec<JsoupPseudo>, Option<String>) {
    let mut base = css.trim().to_string();
    let mut pseudos = Vec::new();
    let mut tail: Option<String> = None;
    loop {
        let trimmed = base.trim_end();
        if let Some(idx) = trimmed.rfind(":contains(") {
            let close = trimmed[idx..].find(')').map(|c| idx + c + 1);
            if let Some(close) = close {
                let inner = &trimmed[idx + ":contains(".len()..close - 1];
                let text = inner.trim().trim_matches(['\'', '"']);
                pseudos.push(JsoupPseudo::Contains(text.to_string()));
                let after = trimmed[close..].trim();
                if tail.is_none() && !after.is_empty() {
                    tail = Some(after.to_string());
                }
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
                let after = trimmed[close..].trim();
                if tail.is_none() && !after.is_empty() {
                    tail = Some(after.to_string());
                }
                base = trimmed[..idx].to_string();
                continue;
            }
        }
        // fix: :has(selector) 实现为过滤（jsoup 语义：元素包含匹配 selector 的后代；
        //      原条件剥离——div:has(a) 返回所有 div 过度匹配，索引错位）
        if let Some(idx) = trimmed.rfind(":has(") {
            let close = trimmed[idx..].find(')').map(|c| idx + c + 1);
            if let Some(close) = close {
                let inner = &trimmed[idx + ":has(".len()..close - 1];
                pseudos.push(JsoupPseudo::Has(inner.trim().to_string()));
                let after = trimmed[close..].trim();
                if tail.is_none() && !after.is_empty() {
                    tail = Some(after.to_string());
                }
                base = trimmed[..idx].to_string();
                continue;
            }
        }
        // fix: 通用剥离括号类伪类（:matches/:containsWholeText/:matchesOwn——scraper 不支持；
        //      条件剥离保留伪类后内容为 tail）
        let mut strip_pos: Option<usize> = None;
        let mut strip_end: Option<usize> = None;
        for pseudo_name in [":matches(", ":containsWholeText(", ":matchesOwn("] {
            if let Some(idx) = trimmed.rfind(pseudo_name) {
                if let Some(c) = trimmed[idx..].find(')') {
                    strip_pos = Some(idx);
                    strip_end = Some(idx + c + 1);
                    break;
                }
            }
        }
        if let (Some(idx), Some(close)) = (strip_pos, strip_end) {
            let after = trimmed[close..].trim();
            if tail.is_none() && !after.is_empty() {
                tail = Some(after.to_string());
            }
            base = trimmed[..idx].to_string();
            continue;
        }
        if let Some(idx) = trimmed.rfind(":eq(") {
            let close = trimmed[idx..].find(')').map(|c| idx + c + 1);
            if let Some(close) = close {
                let inner = &trimmed[idx + ":eq(".len()..close - 1];
                if let Ok(n) = inner.trim().parse::<usize>() {
                    pseudos.push(JsoupPseudo::Eq(n));
                    let after = trimmed[close..].trim();
                    if tail.is_none() && !after.is_empty() {
                        tail = Some(after.to_string());
                    }
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
                    let after = trimmed[close..].trim();
                    if tail.is_none() && !after.is_empty() {
                        tail = Some(after.to_string());
                    }
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
                    let after = trimmed[close..].trim();
                    if tail.is_none() && !after.is_empty() {
                        tail = Some(after.to_string());
                    }
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
    (base, pseudos, tail)
}

/// 直接子元素（第一层）
pub fn children_of(html: &str) -> Elements {
    let doc = scraper::Html::parse_fragment(html);
    let mut list = Vec::new();
    for child in doc.root_element().children() {
        if let Some(el) = scraper::ElementRef::wrap(child) {
            let h = el.html().to_string();
            list.push(Element {
                text: jsoup_normalise_text(&el),
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
        .map(|e| jsoup_normalise_text(&e))
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
        Some(e) => (e.html().to_string(), jsoup_normalise_text(&e)),
        None => (html.to_string(), text_of(html)),
    }
}
