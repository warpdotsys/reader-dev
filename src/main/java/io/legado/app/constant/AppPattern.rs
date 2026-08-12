use crate::prelude::*;
pub struct AppPattern;

impl AppPattern {
    pub fn JS_PATTERN() -> Pattern {
        Pattern::compile_with("<js>([\\w\\W]*?)</js>|@js:([\\w\\W]*)", Pattern::CASE_INSENSITIVE)
    }

    pub fn EXP_PATTERN() -> Pattern {
        Pattern::compile("\\{\\{([\\w\\W]*?)\\}\\}")
    }

    //匹配格式化后的图片格式
    pub fn imgPattern() -> Pattern {
        Pattern::compile("<img[^>]*src=\"([^\"]*(?:\"[^>]+\\})?)\"[^>]*>")
    }

    //dataURL图片类型
    pub fn dataUriRegex() -> Regex {
        regex_new_with_option("data:.*?;base64,(.*)", 0)
    }

    pub fn nameRegex() -> Regex {
        regex_new_with_option("\\s+作\\s*者.*|\\s+\\S+\\s+著", 0)
    }

    pub fn authorRegex() -> Regex {
        regex_new_with_option("^\\s*作\\s*者[:：\\s]+|\\s+著", 0)
    }

    pub fn fileNameRegex() -> Regex {
        regex_new_with_option("[\\\\/:*?\"<>|.]", 0)
    }

    pub fn splitGroupRegex() -> Regex {
        regex_new_with_option("[,;，；]", 0)
    }

    //书源调试信息中的各种符号
    pub fn debugMessageSymbolRegex() -> Regex {
        regex_new_with_option("[⇒◇┌└≡]", 0)
    }

    //本地书籍支持类型
    pub fn bookFileRegex() -> Regex {
        regex_new_with_option(".*\\.(txt|epub|umd)", RegexOption::IGNORE_CASE)
    }

    /**
     * 所有标点
     */
    pub fn bdRegex() -> Regex {
        regex_new_with_option("(\\p{P})+", 0)
    }

    /**
     * 换行
     */
    pub fn rnRegex() -> Regex {
        regex_new_with_option("[\\r\\n]", 0)
    }

    /**
     * 不发音段落判断
     */
    pub fn notReadAloudRegex() -> Regex {
        regex_new_with_option("^(\\s|\\p{C}|\\p{P}|\\p{Z}|\\p{S})+$", 0)
    }
}
