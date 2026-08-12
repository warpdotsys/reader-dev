use crate::prelude::*;
use crate::stubs::URL;
// fix: AnalyzeUrl::paramPattern 已在 AnalyzeUrl.rs 转为私有模块函数，此处使用等价的本地 Pattern
fn PARAM_PATTERN() -> Pattern {
    Pattern::compile(r"\s*,\s*(?=\{)")
}
pub struct HtmlFormatter {
    wrapHtmlRegex: Regex,
    commentRegex: Regex, //注释
    notImgHtmlRegex: Regex,
    otherHtmlRegex: Regex,
    formatImagePattern: Pattern,
}

impl HtmlFormatter {
    pub fn new() -> HtmlFormatter {
        HtmlFormatter {
            wrapHtmlRegex: Regex::new("</?(?:div|p|br|hr|h\\d|article|dd|dl)[^>]*>").unwrap(),
            commentRegex: Regex::new("<!--[^>]*-->").unwrap(), //注释
            notImgHtmlRegex: Regex::new("</?(?!img)[a-zA-Z]+(?=[ >])[^<>]*>").unwrap(),
            otherHtmlRegex: Regex::new("</?[a-zA-Z]+(?=[ >])[^<>]*>").unwrap(),
            formatImagePattern: Pattern::compile_with(
                "<img[^>]*src *= *\"([^\"{]*\\{(?:[^{}]|\\{[^}]+\\})+\\})\"[^>]*>|<img[^>]*data-[^=]*= *\"([^\"]*)\"[^>]*>|<img[^>]*src *= *\"([^\"]*)\"[^>]*>",
                Pattern::CASE_INSENSITIVE
            ),
        }
    }

    pub fn format(&self, html: Option<&str>) -> String {
        self.format_other(html, &self.otherHtmlRegex)
    }

    pub fn format_other(&self, html: Option<&str>, otherRegex: &Regex) -> String {
        let html = match html {
            Some(h) => h,
            None => return "".to_string(),
        };
        html.replace_with_regex(self.wrapHtmlRegex.as_str(), "\n")
            .replace_with_regex(self.commentRegex.as_str(), "")
            .replace_with_regex(otherRegex.as_str(), "")
            .replace_with_regex("\\s*\\n+\\s*", "\n　　")
            .replace_with_regex("^[\\n\\s]+", "　　")
            .replace_with_regex("[\\n\\s]+$", "")
    }

    pub fn formatKeepImg(&self, html: Option<&str>) -> String {
        self.formatKeepImg_url(html, None)
    }

    pub fn formatKeepImg_url(&self, html: Option<&str>, redirectUrl: Option<&URL>) -> String {
        let html = match html {
            Some(h) => h,
            None => return "".to_string(),
        };
        let keepImgHtml = self.format_other(Some(html), &self.notImgHtmlRegex);

        //正则的“|”处于顶端而不处于（）中时，具有类似||的熔断效果，故以此机制简化原来的代码
        let mut matcher = self.formatImagePattern.matcher(keepImgHtml.clone());
        let mut appendPos = 0;
        let mut sb = StringBuffer::new();
        while matcher.find() {
            let mut param = String::new();
            sb.append_str(&keepImgHtml[appendPos..matcher.start()]);
            // fix: E0308——getAbsoluteURL_url 第二参需要 &str，先把 Option 链求值结果绑定为 String
            let src = matcher.group_idx(1).map(|it| {
                let paramPattern = PARAM_PATTERN();
                let mut urlMatcher = paramPattern.matcher(it.clone());
                if urlMatcher.find() {
                    param = format!(",{}", &it[urlMatcher.end()..]);
                    it[0..urlMatcher.start()].to_string()
                } else {
                    it.to_string()
                }
            }).or_else(|| matcher.group_idx(2)).unwrap_or(matcher.group_idx(3).unwrap());
            sb.append_str(&format!(
                "<img src=\"{}\"",
                NetworkUtils::getAbsoluteURL_url(
                    redirectUrl,
                    &src,
                )
            ));
            sb.append_str(&param);
            sb.append_str("\">");
            appendPos = matcher.end();
        }
        if appendPos < keepImgHtml.len() {
            sb.append_str(&keepImgHtml[appendPos..keepImgHtml.len()]);
        }
        sb.to_string()
    }
}
