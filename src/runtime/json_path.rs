// 真实 JSONPath 解析（serde_json 实现，供 ReadContext.read 使用）
// 支持：$ .field ['field'] [n] [*] 以及简单 [?(@.k=="v")] 过滤

use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Root,
    Field(String),
    Index(i64),
    Wildcard,
    Filter(String, String),
}

fn tokenize(path: &str) -> Vec<Token> {
    let p = path.trim();
    if p.is_empty() {
        return vec![Token::Root];
    }
    let mut tokens = Vec::new();
    let mut rest = p;
    if let Some(r) = rest.strip_prefix('$') {
        rest = r;
        tokens.push(Token::Root);
    } else {
        tokens.push(Token::Root);
    }
    while !rest.is_empty() {
        if let Some(r) = rest.strip_prefix('.') {
            rest = r;
            // 字段名
            let (name, rem) = take_field(rest);
            tokens.push(Token::Field(name));
            rest = rem;
        } else if let Some(r) = rest.strip_prefix("['") {
            rest = r;
            if let Some(end) = rest.find("']") {
                let name = rest[..end].to_string();
                tokens.push(Token::Field(name));
                rest = &rest[end + 2..];
            } else {
                rest = "";
            }
        } else if let Some(r) = rest.strip_prefix('[') {
            rest = r;
            if rest.starts_with('*') {
                tokens.push(Token::Wildcard);
                rest = &rest[1..];
                if let Some(r) = rest.strip_prefix(']') {
                    rest = r;
                }
            } else if let Some(end) = rest.find(']') {
                let inner = rest[..end].trim();
                rest = &rest[end + 1..];
                if inner.starts_with("?(") {
                    // 简单过滤 [?(@.k=="v")] 或 [?(@.k='v')]
                    let inner = inner[2..inner.len() - 1].trim();
                    if let Some(eq) = find_operator(inner) {
                        let op = &inner[..eq];
                        let rhs = &inner[eq..];
                        let rhs = rhs.trim_matches(|c| c == '"' || c == '\'' || c == '=' || c == ' ');
                        let key = op.trim().trim_start_matches("@.").to_string();
                        tokens.push(Token::Filter(key, rhs.to_string()));
                    }
                } else if let Ok(i) = inner.parse::<i64>() {
                    tokens.push(Token::Index(i));
                }
            } else {
                rest = "";
            }
        } else {
            // 无前缀字段（容错：直接当字段）
            let (name, rem) = take_field(rest);
            tokens.push(Token::Field(name));
            rest = rem;
        }
    }
    tokens
}

fn find_operator(s: &str) -> Option<usize> {
    for (i, c) in s.char_indices() {
        if c == '=' {
            return Some(i);
        }
    }
    None
}

fn take_field(rest: &str) -> (String, &str) {
    let end = rest
        .char_indices()
        .find(|(i, c)| *i > 0 && (*c == '.' || *c == '['))
        .map(|(i, _)| i)
        .unwrap_or(rest.len());
    let name = rest[..end].to_string();
    (name, &rest[end..])
}

pub fn query(json_str: &str, path: &str) -> Option<Value> {
    let root: Value = serde_json::from_str(json_str).ok()?;
    let tokens = tokenize(path);
    resolve(&root, &tokens)
}

fn resolve(node: &Value, tokens: &[Token]) -> Option<Value> {
    if tokens.is_empty() {
        return Some(node.clone());
    }
    match &tokens[0] {
        Token::Root => resolve(node, &tokens[1..]),
        Token::Field(name) => {
            let obj = node.as_object()?;
            match obj.get(name) {
                Some(v) => resolve(v, &tokens[1..]),
                None => None,
            }
        }
        Token::Index(i) => {
            let arr = node.as_array()?;
            let idx = if *i < 0 {
                (arr.len() as i64 + *i) as usize
            } else {
                *i as usize
            };
            match arr.get(idx) {
                Some(v) => resolve(v, &tokens[1..]),
                None => None,
            }
        }
        Token::Wildcard => {
            let arr = node.as_array()?;
            let mut out = Vec::new();
            for v in arr {
                if let Some(r) = resolve(v, &tokens[1..]) {
                    out.push(r);
                }
            }
            Some(Value::Array(out))
        }
        Token::Filter(key, expected) => {
            let arr = node.as_array()?;
            let mut out = Vec::new();
            for v in arr {
                let ok = v
                    .as_object()
                    .and_then(|o| o.get(key))
                    .and_then(|x| x.as_str())
                    .map(|s| s == expected)
                    .unwrap_or(false);
                if ok {
                    out.push(v.clone());
                }
            }
            if out.is_empty() {
                return None;
            }
            let result = Value::Array(out);
            resolve(&result, &tokens[1..])
        }
    }
}
