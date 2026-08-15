// 真实 XPath 子集解析（→ CSS 选择器 + scraper）
// 支持：//tag  /tag  //tag[@attr='v']  //tag[contains(@class,'x')]
//       //tag[contains(text(),'x')]  //tag/text()  //tag/@attr  //tag[n]  // 与 / 轴
//       多谓词 [a and b] / [a or b] / [a][b]；@attr 存在；!= / 数值比较；
//       position() / last()

use crate::stubs::{Element, JXNode};
use scraper::ElementRef;

struct Step {
    tag: String,
    predicates: Vec<Predicate>,
    is_attribute: bool,
    is_text: bool,
    child_axis: bool, // true = //, false = /
}

#[derive(Clone, Debug)]
enum Predicate {
    AttrEq(String, String),
    AttrNe(String, String),
    AttrContains(String, String),
    AttrExists(String),
    AttrStartsWith(String, String),
    TextContains(String),
    TextStartsWith(String),
    Index(usize),
    PositionEq(usize),
    PositionGt(usize),
    PositionLt(usize),
    Last,
    Cmp(String, String, f64), // op: ">" "<" ">=" "<="
    Or(Vec<Predicate>),
    And(Vec<Predicate>),
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

/// 解析单个谓词表达式（如 `@id='x'`、`contains(@class,'a')`、`@width>100`、`position()>1`）
fn parse_single_predicate(expr: &str) -> Option<Predicate> {
    let e = expr.trim();
    if e.is_empty() {
        return None;
    }
    // 纯数字索引
    if let Ok(idx) = e.parse::<usize>() {
        return Some(Predicate::Index(idx));
    }
    // position() / last()
    if e.starts_with("position()") {
        let rest = &e["position()".len()..];
        return parse_position(rest, false);
    }
    if e == "last()" {
        return Some(Predicate::Last);
    }
    // @attr 存在
    if e.starts_with('@') && !e.contains('=') && !e.contains('>') && !e.contains('<') {
        return Some(Predicate::AttrExists(e[1..].to_string()));
    }
    // 比较运算
    for op in [">=", "<=", "!=", ">", "<", "="] {
        if let Some(pos) = e.find(op) {
            let lhs = e[..pos].trim();
            let rhs_raw = e[pos + op.len()..].trim();
            if op == "=" {
                let rhs = extract_quoted(&rhs_raw);
                if let Some(attr) = lhs.strip_prefix('@') {
                    return Some(Predicate::AttrEq(attr.to_string(), rhs));
                }
                if lhs.contains("contains(@") {
                    let attr = lhs
                        .split('@')
                        .nth(1)
                        .unwrap_or("")
                        .split(')')
                        .next()
                        .unwrap_or("")
                        .to_string();
                    return Some(Predicate::AttrContains(attr, rhs));
                }
                if lhs.contains("contains(text()") {
                    return Some(Predicate::TextContains(rhs));
                }
                return None;
            }
            let attr = lhs.strip_prefix('@').unwrap_or("").to_string();
            if attr.is_empty() {
                return None;
            }
            let num: f64 = if rhs_raw.contains('\'') || rhs_raw.contains('"') {
                extract_quoted(rhs_raw).parse().ok()?
            } else {
                rhs_raw.parse().ok()?
            };
            let sym = op.to_string();
            return Some(Predicate::Cmp(attr, sym, num));
        }
    }
    // contains(@attr,'x') 无等号形式
    if e.contains("contains(@") {
        let attr = e
            .split('@')
            .nth(1)
            .unwrap_or("")
            .split([')', ','])
            .next()
            .unwrap_or("")
            .to_string();
        let rhs = extract_quoted(e);
        return Some(Predicate::AttrContains(attr, rhs));
    }
    if e.contains("contains(text()") {
        let rhs = extract_quoted(e);
        return Some(Predicate::TextContains(rhs));
    }
    // fix: starts-with(@attr,'x') / starts-with(text(),'x')（原不支持——规则解析失败）
    if e.contains("starts-with(@") {
        let attr = e
            .split('@')
            .nth(1)
            .unwrap_or("")
            .split([')', ','])
            .next()
            .unwrap_or("")
            .to_string();
        let rhs = extract_quoted(e);
        return Some(Predicate::AttrStartsWith(attr, rhs));
    }
    if e.contains("starts-with(text()") {
        let rhs = extract_quoted(e);
        return Some(Predicate::TextStartsWith(rhs));
    }
    None
}

fn parse_position(rest: &str, _is_last: bool) -> Option<Predicate> {
    // rest 形如 "=2" / ">1" / "<3" / ">=2" / "<=3" 或 ")"
    let r = rest.trim_start_matches(')').trim();
    for op in [">=", "<=", ">", "<", "="] {
        if let Some(pos) = r.find(op) {
            let num: usize = r[pos + op.len()..].trim().parse().ok()?;
            return match op {
                "=" => Some(Predicate::PositionEq(num)),
                ">" => Some(Predicate::PositionGt(num)),
                "<" => Some(Predicate::PositionLt(num)),
                ">=" => Some(Predicate::PositionGt(num.saturating_sub(1))),
                "<=" => Some(Predicate::PositionLt(num + 1)),
                _ => None,
            };
        }
    }
    Some(Predicate::PositionEq(1))
}

/// 解析 `[...]` 谓词串（支持 and / or / 嵌套列表）
fn parse_predicate_group(inner: &str) -> Option<Vec<Predicate>> {
    // 顶层按括号深度切分 and / or
    let mut parts: Vec<(String, String)> = Vec::new(); // (expr, joiner)
    let mut depth = 0i32;
    let mut current = String::new();
    let mut last_joiner = String::from("and");
    for ch in inner.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            _ if depth == 0 => {
                let lower: String = current.to_lowercase();
                if lower.ends_with(" and") {
                    current.truncate(current.len() - 4);
                    parts.push((current.trim().to_string(), last_joiner.clone()));
                    current = String::new();
                    last_joiner = String::from("and");
                } else if lower.ends_with(" or") {
                    current.truncate(current.len() - 3);
                    parts.push((current.trim().to_string(), last_joiner.clone()));
                    current = String::new();
                    last_joiner = String::from("or");
                } else {
                    current.push(ch);
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        parts.push((current.trim().to_string(), last_joiner.clone()));
    }
    if parts.is_empty() {
        return None;
    }
    let mut preds: Vec<Predicate> = Vec::new();
    let mut group: Vec<Predicate> = Vec::new();
    for (expr, joiner) in parts {
        if let Some(p) = parse_single_predicate(&expr) {
            group.push(p);
        }
        if joiner == "or" {
            if !group.is_empty() {
                preds.push(Predicate::Or(std::mem::take(&mut group)));
            }
        }
    }
    if !group.is_empty() {
        if group.len() == 1 {
            preds.push(group.pop().unwrap());
        } else {
            preds.push(Predicate::And(group));
        }
    }
    Some(preds)
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
    while let Some(p) = rest.find('[') {
        if tag.is_empty() {
            tag = rest[..p].to_string();
        }
        // 找到匹配的 ]
        let close = rest[p..].find(']')? + p;
        let inner = &rest[p + 1..close];
        if let Some(ps) = parse_predicate_group(inner) {
            predicates.extend(ps);
        }
        rest = &rest[close + 1..];
        if rest.is_empty() {
            break;
        }
        if !tag.is_empty() && rest.starts_with('[') {
            continue;
        }
        break;
    }
    if tag.is_empty() {
        tag = rest.to_string();
    }
    Some(Step {
        tag,
        predicates,
        is_attribute: false,
        is_text: false,
        child_axis: false,
    })
}

fn to_css(html: &str, xpath: &str) -> Option<(String, Vec<Predicate>, bool, bool, Option<String>)> {
    // 返回 (css, 后处理谓词, has_index, need_text, need_attr)
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
    let mut need_attr: Option<String> = None;
    let mut need_text = false;
    let mut has_index = false;
    let mut post_predicates: Vec<Predicate> = Vec::new();
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
        let mut text_filters = false;
        for p in &step.predicates {
            match p {
                Predicate::AttrEq(k, v) => css.push_str(&format!("[{}=\"{}\"]", k, v)),
                Predicate::AttrContains(k, v) => css.push_str(&format!("[{}*=\"{}\"]", k, v)),
                Predicate::AttrExists(k) => css.push_str(&format!("[{}]", k)),
                Predicate::Index(_) => {
                    has_index = true;
                    post_predicates.push(p.clone());
                }
                Predicate::PositionEq(n) if *n == 1 => {
                    has_index = true;
                    post_predicates.push(p.clone());
                }
                _ => {
                    // 无法用 CSS 表达的谓词 → 后处理
                    post_predicates.push(p.clone());
                    text_filters = true;
                }
            }
        }
        if text_filters {
            need_text = true;
        }
    }
    Some((css, post_predicates, has_index, need_text, need_attr))
}

/// 执行 XPath 选择
pub fn select_nodes(html: &str, xpath: &str) -> Option<Vec<JXNode>> {
    let parsed = to_css(html, xpath)?;
    let (css, post_predicates, has_index, _need_text, _need_attr) = parsed;

    let doc = scraper::Html::parse_document(html);
    let Ok(sel) = scraper::Selector::parse(&css) else {
        return None;
    };
    let mut nodes: Vec<JXNode> = Vec::new();
    for e in doc.select(&sel) {
        if !predicates_match(&e, &post_predicates) {
            continue;
        }
        nodes.push(JXNode {
            text: e.text().collect::<String>(),
            html: e.html().to_string(),
        });
    }
    // Index/position 谓词：过滤后取第 n 个匹配（XPath 语义）
    let target: Option<usize> = {
        let mut t = None;
        for p in &post_predicates {
            match p {
                Predicate::Index(n) => t = Some(*n),
                Predicate::PositionEq(n) => t = Some(*n),
                _ => {}
            }
        }
        t
    };
    let last_only = post_predicates.iter().any(|p| matches!(p, Predicate::Last));
    if let Some(n) = target {
        if n >= 1 && n <= nodes.len() {
            nodes = vec![nodes[n - 1].clone()];
        } else {
            return None;
        }
    }
    if last_only {
        if let Some(last) = nodes.last() {
            nodes = vec![last.clone()];
        } else {
            return None;
        }
    }
    if nodes.is_empty() {
        return None;
    }
    Some(nodes)
}

fn attr_of(el: &ElementRef, name: &str) -> String {
    el.value()
        .attr(name)
        .map(|s| s.to_string())
        .unwrap_or_default()
}

fn predicates_match(el: &ElementRef, preds: &[Predicate]) -> bool {
    for p in preds {
        if !predicate_match(el, p) {
            return false;
        }
    }
    true
}

fn predicate_match(el: &ElementRef, p: &Predicate) -> bool {
    match p {
        Predicate::AttrEq(k, v) => attr_of(el, k) == *v,
        Predicate::AttrNe(k, v) => attr_of(el, k) != *v,
        Predicate::AttrContains(k, v) => attr_of(el, k).contains(v.as_str()),
        Predicate::AttrStartsWith(k, v) => attr_of(el, k).starts_with(v.as_str()),
        Predicate::AttrExists(k) => !attr_of(el, k).is_empty(),
        Predicate::TextContains(v) => {
            // XPath text() 语义：直接文本子节点（不含后代元素文本）
            let direct_text: String = el
                .children()
                .filter_map(|c| match c.value() {
                    scraper::node::Node::Text(t) => Some(t.to_string()),
                    _ => None,
                })
                .collect();
            direct_text.contains(v.as_str())
        }
        Predicate::TextStartsWith(v) => {
            let direct_text: String = el
                .children()
                .filter_map(|c| match c.value() {
                    scraper::node::Node::Text(t) => Some(t.to_string()),
                    _ => None,
                })
                .collect();
            direct_text.trim_start().starts_with(v.as_str())
        }
        Predicate::Cmp(k, op, num) => {
            let val: f64 = attr_of(el, k).parse().unwrap_or(f64::NAN);
            if val.is_nan() {
                return false;
            }
            match op.as_str() {
                ">" => val > *num,
                "<" => val < *num,
                ">=" => val >= *num,
                "<=" => val <= *num,
                "!=" => val != *num,
                _ => false,
            }
        }
        Predicate::Or(ps) => ps.iter().any(|x| predicate_match(el, x)),
        Predicate::And(ps) => ps.iter().all(|x| predicate_match(el, x)),
        Predicate::Index(_) | Predicate::PositionEq(_) | Predicate::PositionGt(_) | Predicate::PositionLt(_) | Predicate::Last => true,
    }
}

/// XPath 字符串值（含 @attr / text() 提取）
pub fn select_strings(html: &str, xpath: &str) -> Option<Vec<String>> {
    let mut rest = xpath.trim();
    let mut attr: Option<String> = None;
    let mut is_text = false;
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
