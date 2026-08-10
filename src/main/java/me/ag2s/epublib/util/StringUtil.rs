/**
 * Various String utility functions.
 * <p>
 * Most of the functions herein are re-implementations of the ones in apache
 * commons StringUtils. The reason for re-implementing this is that the
 * functions are fairly simple and using my own implementation saves the
 * inclusion of a 200Kb jar file.
 *
 * @author paul.siegmann
 */
pub struct StringUtil;

impl StringUtil {

    /**
     * Changes a path containing '..', '.' and empty dirs into a path that
     * doesn't. X/foo/../Y is changed into 'X/Y', etc. Does not handle invalid
     * paths like "../".
     *
     * @param path path
     * @return the normalized path
     */
    pub fn collapse_path_dots(path: &str) -> String {
        let string_parts: Vec<&str> = path.split("/").collect();
        let mut parts: Vec<String> = string_parts.iter().map(|s| s.to_string()).collect();
        let mut i = 0;
        while i < parts.len() - 1 {
            let current_dir = parts.get(i).cloned().unwrap();
            if current_dir.len() == 0 || current_dir == "." {
                parts.remove(i);
                if i > 0 { i -= 1; }
            } else if current_dir == ".." {
                if i >= 1 { parts.remove(i - 1); }
                if i >= 1 { parts.remove(i - 1); }
                if i >= 2 { i -= 2; }
            } else {
                i += 1;
            }
        }
        let mut result = String::new();
        if path.starts_with("/") {
            result.push('/');
        }
        for i in 0..parts.len() {
            result.push_str(&parts[i]);
            if i < (parts.len() - 1) {
                result.push('/');
            }
        }
        result
    }

    /**
     * Whether the String is not null, not zero-length and does not contain of
     * only whitespace.
     *
     * @param text text
     * @return Whether the String is not null, not zero-length and does not contain of
     */
    pub fn is_not_blank(text: &str) -> bool {
        !is_blank(text)
    }

    /**
     * Whether the String is null, zero-length and does contain only whitespace.
     *
     * @return Whether the String is null, zero-length and does contain only whitespace.
     */
    pub fn is_blank(text: &str) -> bool {
        if is_empty(text) {
            return true;
        }
        for c in text.chars() {
            if !c.is_whitespace() {
                return false;
            }
        }
        true
    }

    /**
     * Whether the given string is null or zero-length.
     *
     * @param text the input for this method
     * @return Whether the given string is null or zero-length.
     */
    pub fn is_empty(text: &str) -> bool {
        text.is_empty()
    }

    /**
     * Whether the given source string ends with the given suffix, ignoring
     * case.
     *
     * @param source source
     * @param suffix suffix
     * @return Whether the given source string ends with the given suffix, ignoring case.
     */
    pub fn ends_with_ignore_case(source: &str, suffix: &str) -> bool {
        if is_empty(suffix) {
            return true;
        }
        if is_empty(source) {
            return false;
        }
        if suffix.len() > source.len() {
            return false;
        }
        source[source.len() - suffix.len()..].to_lowercase().ends_with(&suffix.to_lowercase())
    }

    /**
     * If the given text is null return "", the original text otherwise.
     *
     * @param text text
     * @return If the given text is null "", the original text otherwise.
     */
    pub fn default_if_null(text: &str) -> String {
        default_if_null_default(text, "")
    }

    /**
     * If the given text is null return "", the given defaultValue otherwise.
     *
     * @param text         d
     * @param defaultValue d
     * @return If the given text is null "", the given defaultValue otherwise.
     */
    pub fn default_if_null_default(text: &str, default_value: &str) -> String {
        if text == null {
            return default_value.to_string();
        }
        text.to_string()
    }

    /**
     * Null-safe string comparator
     *
     * @param text1 d
     * @param text2 d
     * @return whether the two strings are equal
     */
    pub fn equals(text1: &str, text2: &str) -> bool {
        text1 == text2
    }

    /**
     * Pretty toString printer.
     *
     * @param keyValues d
     * @return a string representation of the input values
     */
    pub fn to_string(key_values: Vec<Option<String>>) -> String {
        let mut result = String::new();
        result.push('[');
        let mut i = 0;
        while i < key_values.len() {
            if i > 0 {
                result.push_str(", ");
            }
            result.push_str(key_values[i].as_deref().unwrap_or(""));
            result.push_str(": ");
            let mut value = None;
            if (i + 1) < key_values.len() {
                value = key_values[i + 1].clone();
            }
            if value == null {
                result.push_str("<null>");
            } else {
                result.push('\'');
                result.push_str(value.as_deref().unwrap_or(""));
                result.push('\'');
            }
            i += 2;
        }
        result.push(']');
        result
    }

    pub fn hash_code(values: Vec<&str>) -> i32 {
        let mut result = 31;
        for value in values {
            let mut hash = 0;
            for b in value.as_bytes() {
                hash = 31 * hash + *b as i32;
            }
            result ^= hash;
        }
        result
    }

    /**
     * Gives the substring of the given text before the given separator.
     * <p>
     * If the text does not contain the given separator then the given text is
     * returned.
     *
     * @param text      d
     * @param separator d
     * @return the substring of the given text before the given separator.
     */
    pub fn substring_before(text: &str, separator: char) -> String {
        if is_empty(text) {
            return text.to_string();
        }
        let sep_pos = text.find(separator);
        if sep_pos < 0 {
            return text.to_string();
        }
        text[0..sep_pos].to_string()
    }

    /**
     * Gives the substring of the given text before the last occurrence of the
     * given separator.
     * <p>
     * If the text does not contain the given separator then the given text is
     * returned.
     *
     * @param text      d
     * @param separator d
     * @return the substring of the given text before the last occurrence of the given separator.
     */
    pub fn substring_before_last(text: &str, separator: char) -> String {
        if is_empty(text) {
            return text.to_string();
        }
        let c_pos = text.rfind(separator);
        if c_pos < 0 {
            return text.to_string();
        }
        text[0..c_pos].to_string()
    }

    /**
     * Gives the substring of the given text after the last occurrence of the
     * given separator.
     * <p>
     * If the text does not contain the given separator then "" is returned.
     *
     * @param text      d
     * @param separator d
     * @return the substring of the given text after the last occurrence of the given separator.
     */
    pub fn substring_after_last(text: &str, separator: char) -> String {
        if is_empty(text) {
            return text.to_string();
        }
        let c_pos = text.rfind(separator);
        if c_pos < 0 {
            return "".to_string();
        }
        text[c_pos + 1..].to_string()
    }

    /**
     * Gives the substring of the given text after the given separator.
     * <p>
     * If the text does not contain the given separator then "" is returned.
     *
     * @param text the input text
     * @param c    the separator char
     * @return the substring of the given text after the given separator.
     */
    pub fn substring_after(text: &str, c: char) -> String {
        if is_empty(text) {
            return text.to_string();
        }
        let c_pos = text.find(c);
        if c_pos < 0 {
            return "".to_string();
        }
        text[c_pos + 1..].to_string()
    }

    pub fn format_html(text: &str) -> String {
        let mut body = String::new();
        for line in text.split("\n") {
            let mut s = line.trim().to_string();
            if s.len() > 0 {
                //段落为一张图片才认定为图片章节/漫画并启用多看单图优化，否则认定为普通文字夹杂着的图片文字。
                if let Some(captured) = match_img_tag(&s) {
                    body.push_str(&format!("<div class=\"duokan-image-single\"><img class=\"picture-80\" {}/></div>", captured));
                } else {
                    body.push_str("<p>");
                    body.push_str(&s);
                    body.push_str("</p>");
                }
            }
        }
        body
    }
}

fn match_img_tag(s: &str) -> Option<String> {
    //(?i)^<img\s([^>]+)/?>$
    let lower = s.to_lowercase();
    let lower = lower.trim();
    if lower.starts_with("<img") {
        let rest = lower.trim_start_matches("<img");
        if rest.chars().next().map(|c| c.is_whitespace()).unwrap_or(false) {
            let inner = rest.trim_start();
            if let Some(end) = inner.find('/') {
                let attrs = &inner[..end];
                if !attrs.is_empty() && !attrs.contains('>') {
                    let after = &inner[end..];
                    let after = after.trim_start_matches('/');
                    let after = after.trim_start_matches('>');
                    if after.is_empty() {
                        return Some(attrs.to_string());
                    }
                }
            } else if let Some(end) = inner.find('>') {
                let attrs = &inner[..end];
                if !attrs.is_empty() && !attrs.contains('>') {
                    return Some(attrs.to_string());
                }
            }
        }
    }
    None
}
