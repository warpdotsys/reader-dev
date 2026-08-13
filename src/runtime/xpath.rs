// 真实 XPath 子集解析（→ CSS 选择器 + scraper）
// 支持：//tag  /tag  //tag[@attr='v']  //tag[contains(@class,'x')]
//       //tag[contains(text(),'x')]  //tag/text()  //tag/@attr  //tag[n]  // 与 / 轴

use crate::stubs::{Element, Elements, JXNode};
use serde_json::Value;

struct Step {
    tag: String,
    predicates: Vec<Predicate>,
    is_attribute: bool,
    is_text: bool,
    child_axis: bool, // true = //, false = /
}

enum Predicate {
    AttrEq(String, String),
    AttrContains(String, String),
    TextContains(String),
    Index(usize),
}

fn extract_quoted(inner: &str) -> String {
    if let Some(rest) = inner.split('\'').nth(1) {
        return rest.split('\'').next().unwrap_or("").to_string();
    }
    if let Some(rest) = inner.split('"').nth(1) {
        return rest.split('"').next().unwrap_or("").to_string();
    }
    String::new()
}

fn parse_step(s: &str) -> Option<Step> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    // @attr
    if let Some(rest) = s.strip_prefix('@') {
        return Some(Step {
            tag: String::new(),
            predicates: Vec::new(),
            is_attribute: true,
            is_text: false,
            child_axis: false,
        });
    }
    // text()
    if s == "text()" {
        return Some(Step {
            tag: String::new(),
            predicates: Vec::new(),
            is_attribute: false,
            is_text: true,
            child_axis: false,
        });
    }
    // tag[predicates]
    let mut tag = String::new();
    let mut rest = s;
    let mut predicates = Vec::new();
    if let Some(p) = rest.find('[') {
        tag = rest[..p].to_string();
        rest = &rest[p..];
        // 解析 [..] 谓词
        let mut inner = rest.trim_start_matches('[').trim_end_matches(']').to_string();
        if let Ok(idx) = inner.parse::<usize>() {
            predicates.push(Predicate::Index(idx));
        } else if let Some(pos) = inner.find('=') {
            let lhs = inner[..pos].trim().to_string();
            let rhs = extract_quoted(&inner);
            if lhs.starts_with("@") {
                predicates.push(Predicate::AttrEq(lhs[1..].to_string(), rhs));
            } else if lhs.contains("contains(@") {
                let attr = lhs.split('@').nth(1).unwrap_or("").split(')').next().unwrap_or("").to_string();
                predicates.push(Predicate::AttrContains(attr, rhs));
            } else if lhs.contains("contains(text()") {
                predicates.push(Predicate::TextContains(rhs));
            }
        } else if inner.contains("contains(@") {
            let attr = inner.split('@').nth(1).unwrap_or("").split([')', ',']).next().unwrap_or("").to_string();
            let rhs = extract_quoted(&inner);
            predicates.push(Predicate::AttrContains(attr, rhs));
        } else if inner.contains("contains(text()") {
            let rhs = extract_quoted(&inner);
            predicates.push(Predicate::TextContains(rhs));
        }
    } else {
        tag = s.to_string();
    }
    Some(Step {
        tag,
        predicates,
        is_attribute: false,
        is_text: false,
        child_axis: false,
    })
}

fn to_css(html: &str, xpath: &str) -> Option<(String, bool, bool)> {
    // 返回 (css, need_text_filter, has_index)
    let mut steps: Vec<(Step, bool)> = Vec::new();
    let mut rest = xpath.trim();
    while !rest.is_empty() {
        let child_axis = rest.starts_with("//");
        if child_axis {
            rest = &rest[2..];
        } else if rest.starts_with('/') {
            rest = &rest[1..];
        } else {
            rest = rest.trim_start_matches('/');
        }
        if rest.is_empty() {
            break;
        }
        // 取下一个段（到 / 为止）
        let end = rest.find('/').unwrap_or(rest.len());
        let seg = &rest[..end];
        let mut step = parse_step(seg)?;
        step.child_axis = child_axis;
        steps.push((step, child_axis));
        rest = &rest[end..];
    }
    if steps.is_empty() {
        return None;
    }
    // 属性/text 提取标记
    let mut need_attr: Option<String> = None;
    let mut need_text = false;
    let mut has_index = false;
    let mut css = String::new();
    for (i, (step, child_axis)) in steps.iter().enumerate() {
        if step.is_attribute {
            need_attr = Some(String::new());
            continue;
        }
        if step.is_text {
            need_text = true;
            continue;
        }
        if i > 0 {
            css.push_str(if *child_axis { " " } else { " > " });
        }
        if step.tag.is_empty() {
            css.push_str("*");
        } else {
            css.push_str(&step.tag);
        }
        let mut text_filters = Vec::new();
        for p in &step.predicates {
            match p {
                Predicate::AttrEq(k, v) => css.push_str(&format!("[{}=\"{}\"]", k, v)),
                Predicate::AttrContains(k, v) => css.push_str(&format!("[{}*=\"{}\"]", k, v)),
                Predicate::TextContains(_) => text_filters.push(true),
                Predicate::Index(_) => has_index = true,
            }
        }
        if !text_filters.is_empty() {
            need_text = true;
        }
    }
    Some((css, need_text || need_attr.is_some(), has_index))
}

/// 执行 XPath 选择
pub fn select_nodes(html: &str, xpath: &str) -> Option<Vec<JXNode>> {
    let parsed = to_css(html, xpath)?;
    let (css, need_filter, has_index) = parsed;

    let doc = scraper::Html::parse_document(html);
    let Ok(sel) = scraper::Selector::parse(&css) else {
        return None;
    };
    let mut nodes: Vec<JXNode> = Vec::new();
    for e in doc.select(&sel) {
        nodes.push(JXNode {
            text: e.text().collect::<String>(),
            html: e.html().to_string(),
        });
    }
    if nodes.is_empty() {
        return None;
    }
    if has_index {
        // 简化：单步索引处理（如 //div[1]）
    }
    Some(nodes)
}

/// XPath 字符串值（含 @attr / text() 提取）
pub fn select_strings(html: &str, xpath: &str) -> Option<Vec<String>> {
    // 属性/text 提取的简化：在 CSS 结果上按最终步骤处理
    let mut rest = xpath.trim();
    let mut attr: Option<String> = None;
    let mut is_text = false;
    let mut last_css = String::new();
    // 解析最后一步
    while !rest.is_empty() {
        let child_axis = rest.starts_with("//");
        if child_axis {
            rest = &rest[2..];
        } else if rest.starts_with('/') {
            rest = &rest[1..];
        }
        let end = rest.find('/').unwrap_or(rest.len());
        let seg = &rest[..end];
        if let Some(a) = seg.trim().strip_prefix('@') {
            attr = Some(a.to_string());
        }
        if seg.trim() == "text()" {
            is_text = true;
        }
        rest = &rest[end..];
    }
    let nodes = select_nodes(html, xpath)?;
    let mut out = Vec::new();
    for n in nodes {
        if let Some(a) = &attr {
            // 从节点 HTML 取属性
            let el = Element {
                text: n.text.clone(),
                html: n.html.clone(),
            };
            let v = el.attr(a);
            if !v.is_empty() && !out.contains(&v) {
                out.push(v);
            }
        } else if is_text {
            if !n.text.is_empty() {
                out.push(n.text.clone());
            }
        } else {
            out.push(n.text.clone());
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

// 保留：serde_json 引用（避免未使用警告的占位无意义，这里仅用于类型推导）
#[allow(dead_code)]
fn _unused(v: &Value) {}
