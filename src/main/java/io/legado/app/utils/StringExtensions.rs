// import org.apache.commons.text.StringEscapeUtils

pub fn safeTrim(s: Option<&str>) -> Option<String> {
    match s {
        Some(s) if !s.is_blank() => Some(s.trim().to_string()),
        _ => None,
    }
}

pub fn isAbsUrl(s: Option<&str>) -> bool {
    match s {
        Some(s) if !s.is_blank() => s.starts_with_ignore_case("http://")
            || s.starts_with_ignore_case("https://"),
        _ => false,
    }
}

pub fn isDataUrl(s: Option<&str>) -> bool {
    match s {
        Some(s) => AppPattern::dataUriRegex().matches(s),
        None => false,
    }
}

pub fn isJson(s: Option<&str>) -> bool {
    match s {
        Some(s) => {
            let str = s.trim();
            if str.starts_with("{") && str.ends_with("}") {
                true
            } else if str.starts_with("[") && str.ends_with("]") {
                true
            } else {
                false
            }
        }
        None => false,
    }
}

pub fn isJsonObject(s: Option<&str>) -> bool {
    match s {
        Some(s) => {
            let str = s.trim();
            str.starts_with("{") && str.ends_with("}")
        }
        None => false,
    }
}

pub fn isJsonArray(s: Option<&str>) -> bool {
    match s {
        Some(s) => {
            let str = s.trim();
            str.starts_with("[") && str.ends_with("]")
        }
        None => false,
    }
}

pub fn isXml(s: Option<&str>) -> bool {
    match s {
        Some(s) => {
            let str = s.trim();
            str.starts_with("<") && str.ends_with(">")
        }
        None => false,
    }
}

pub fn isTrue(s: Option<&str>) -> bool {
    isTrue_default(s, false)
}

pub fn isTrue_default(s: Option<&str>, nullIsTrue: bool) -> bool {
    if s == None || s.unwrap().is_blank() || s.unwrap() == "null" {
        return nullIsTrue;
    }
    !Regex::new("\\s*(?i)(false|no|not|0)\\s*").matches(s.unwrap())
}

pub fn htmlFormat(s: Option<&str>) -> String {
    if s == None || s.unwrap().is_blank() {
        return "".to_string();
    }
    s.unwrap()
        .replace(&Regex::new("(?i)<(br[\\s/]*|/*p\\b.*?|/*div\\b.*?)>"), "\n")// 替换特定标签为换行符
        .replace(&Regex::new("<[script>]*.*?>|&nbsp;"), "")// 删除script标签对和空格转义符
        .replace(&Regex::new("\\s*\\n+\\s*"), "\n　　")// 移除空行,并增加段前缩进2个汉字
        .replace(&Regex::new("^[\\n\\s]+"), "　　")//移除开头空行,并增加段前缩进2个汉字
        .replace(&Regex::new("[\\n\\s]+$"), "") //移除尾部空行
}

pub fn splitNotBlank(s: &str, delimiter: &[&str]) -> Vec<String> {
    split_impl(s, delimiter)
}

fn split_impl(s: &str, delimiter: &[&str]) -> Vec<String> {
    let mut parts = vec![s.to_string()];
    for d in delimiter {
        let mut new_parts = Vec::new();
        for p in parts {
            new_parts.extend(p.split(d).map(|x| x.to_string()));
        }
        parts = new_parts;
    }
    parts.iter().map(|it| it.trim().to_string()).filter(|it| !it.is_blank()).collect()
}

pub fn splitNotBlank_regex(s: &str, regex: &Regex) -> Vec<String> {
    splitNotBlank_regex_limit(s, regex, 0)
}

pub fn splitNotBlank_regex_limit(s: &str, regex: &Regex, limit: i32) -> Vec<String> {
    s.split_limit(regex, limit).iter().map(|it| it.trim().to_string()).filter(|it| !it.is_blank()).collect()
}

pub fn startWithIgnoreCase(s: &str, start: &str) -> bool {
    if s.is_blank() {
        false
    } else {
        s.starts_with_ignore_case(start)
    }
}

pub fn cnCompare(s: &str, other: &str) -> i32 {
    // return java.text.Collator.getInstance(Locale.CHINA).compare(this, other)
    s.compare_to(other)
}

/**
 * 将字符串拆分为单个字符,包含emoji
 */
pub fn toStringArray(s: &str) -> Vec<String> {
    let mut codePointIndex = 0;
    let result: Result<Vec<String>, ()> = (|| {
        let mut arr = Vec::new();
        let count = codePointCount(s, 0, s.len());
        for _ in 0..count {
            let start = codePointIndex;
            codePointIndex = offsetByCodePoints(s, start, 1);
            arr.push(s[start..codePointIndex].to_string());
        }
        Ok(arr)
    })();
    match result {
        Ok(arr) => arr,
        Err(_) => s.split("").filter(|x| !x.is_empty()).collect(),
    }
}
