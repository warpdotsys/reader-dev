pub struct AppPattern;

impl AppPattern {
    pub fn JS_PATTERN() -> Pattern {
        Pattern::compile("<js>([\\w\\W]*?)</js>|@js:([\\w\\W]*)", Pattern::CASE_INSENSITIVE)
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
        Regex::new("data:.*?;base64,(.*)")
    }

    pub fn nameRegex() -> Regex {
        Regex::new("\\s+作\\s*者.*|\\s+\\S+\\s+著")
    }

    pub fn authorRegex() -> Regex {
        Regex::new("^\\s*作\\s*者[:：\\s]+|\\s+著")
    }

    pub fn fileNameRegex() -> Regex {
        Regex::new("[\\\\/:*?\"<>|.]")
    }

    pub fn splitGroupRegex() -> Regex {
        Regex::new("[,;，；]")
    }

    //书源调试信息中的各种符号
    pub fn debugMessageSymbolRegex() -> Regex {
        Regex::new("[⇒◇┌└≡]")
    }

    //本地书籍支持类型
    pub fn bookFileRegex() -> Regex {
        Regex::new(".*\\.(txt|epub|umd)", RegexOption::IGNORE_CASE)
    }

    /**
     * 所有标点
     */
    pub fn bdRegex() -> Regex {
        Regex::new("(\\p{P})+")
    }

    /**
     * 换行
     */
    pub fn rnRegex() -> Regex {
        Regex::new("[\\r\\n]")
    }

    /**
     * 不发音段落判断
     */
    pub fn notReadAloudRegex() -> Regex {
        Regex::new("^(\\s|\\p{C}|\\p{P}|\\p{Z}|\\p{S})+$")
    }
}
