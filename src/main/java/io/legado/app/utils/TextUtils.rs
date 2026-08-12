use crate::prelude::*;
pub struct TextUtils;

impl TextUtils {
    pub fn is_empty(s: Option<&str>) -> bool {
        match s {
            Some(s) => s.is_empty(),
            None => true,
        }
    }

    /**
     * Returns a string containing the tokens joined by delimiters.
     *
     * @param delimiter a CharSequence that will be inserted between the tokens. If None, the string
     *     "null" will be used as the delimiter.
     * @param tokens an array objects to be joined. Strings will be formed from the objects by
     *     calling object.toString(). If tokens is None, a NullPointerException will be thrown. If
     *     tokens is an empty array, an empty string will be returned.
     */
    pub fn join<T: std::fmt::Display>(delimiter: &str, tokens: Vec<T>) -> String {
        let length = tokens.len();
        if length == 0 {
            return "".to_string();
        }
        let mut sb = String::new();
        sb.push_str(&tokens[0].to_string());
        for i in 1..length {
            sb.push_str(delimiter);
            sb.push_str(&tokens[i].to_string());
        }
        sb
    }

    /**
     * Returns a string containing the tokens joined by delimiters.
     *
     * @param delimiter a CharSequence that will be inserted between the tokens. If None, the string
     *     "null" will be used as the delimiter.
     * @param tokens an array objects to be joined. Strings will be formed from the objects by
     *     calling object.toString(). If tokens is None, a NullPointerException will be thrown. If
     *     tokens is empty, an empty string will be returned.
     */
    pub fn join_iter<T: std::fmt::Display>(delimiter: &str, tokens: impl Iterator<Item = T>) -> String {
        let mut iter = tokens;
        match iter.next() {
            None => "".to_string(),
            Some(first) => {
                let mut sb = String::new();
                sb.push_str(&first.to_string());
                for t in iter {
                    sb.push_str(delimiter);
                    sb.push_str(&t.to_string());
                }
                sb
            }
        }
    }
}
