pub struct TextUtils;

impl TextUtils {
    pub fn isEmpty(str: Option<&dyn CharSequence>) -> bool {
        match str {
            Some(s) => s.length() == 0,
            None => true,
        }
    }

    /**
     * Returns a string containing the tokens joined by delimiters.
     *
     * @param delimiter a CharSequence that will be inserted between the tokens. If null, the string
     *     "null" will be used as the delimiter.
     * @param tokens an array objects to be joined. Strings will be formed from the objects by
     *     calling object.toString(). If tokens is null, a NullPointerException will be thrown. If
     *     tokens is an empty array, an empty string will be returned.
     */
    pub fn join(delimiter: &dyn CharSequence, tokens: &[&dyn Any]) -> String {
        let length = tokens.len();
        if length == 0 {
            return "".to_string();
        }
        let mut sb = String::new();
        sb.push_str(&tokens[0].to_string());
        for i in 1..length {
            sb.push_str(&delimiter.to_string());
            sb.push_str(&tokens[i].to_string());
        }
        sb
    }

    /**
     * Returns a string containing the tokens joined by delimiters.
     *
     * @param delimiter a CharSequence that will be inserted between the tokens. If null, the string
     *     "null" will be used as the delimiter.
     * @param tokens an array objects to be joined. Strings will be formed from the objects by
     *     calling object.toString(). If tokens is null, a NullPointerException will be thrown. If
     *     tokens is empty, an empty string will be returned.
     */
    pub fn join_iter(delimiter: &dyn CharSequence, tokens: &mut dyn Iterator<Item = &dyn Any>) -> String {
        if !tokens.has_next() {
            return "".to_string();
        }
        let mut sb = String::new();
        sb.push_str(&tokens.next().unwrap().to_string());
        while tokens.has_next() {
            sb.push_str(&delimiter.to_string());
            sb.push_str(&tokens.next().unwrap().to_string());
        }
        sb
    }
}
