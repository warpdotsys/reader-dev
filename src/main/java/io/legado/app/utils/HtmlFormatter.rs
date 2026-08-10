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
            wrapHtmlRegex: Regex::new("</?(?:div|p|br|hr|h\\d|article|dd|dl)[^>]*>"),
            commentRegex: Regex::new("<!--[^>]*-->"), //注释
            notImgHtmlRegex: Regex::new("</?(?!img)[a-zA-Z]+(?=[ >])[^<>]*>"),
            otherHtmlRegex: Regex::new("</?[a-zA-Z]+(?=[ >])[^<>]*>"),
            formatImagePattern: Pattern::compile(
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
        html.replace(&self.wrapHtmlRegex, "\n")
            .replace(&self.commentRegex, "")
            .replace(otherRegex, "")
            .replace(&Regex::new("\\s*\\n+\\s*"), "\n　　")
            .replace(&Regex::new("^[\\n\\s]+"), "　　")
            .replace(&Regex::new("[\\n\\s]+$"), "")
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
        let matcher = self.formatImagePattern.matcher(&keepImgHtml);
        let mut appendPos = 0;
        let mut sb = StringBuffer::new();
        while matcher.find() {
            let mut param = "";
            sb.append_str(&keepImgHtml[appendPos..matcher.start()]);
            sb.append_str(&format!(
                "<img src=\"{}",
                NetworkUtils::getAbsoluteURL(
                    redirectUrl,
                    matcher.group(1).map(|it| {
                        let urlMatcher = AnalyzeUrl::paramPattern.matcher(it);
                        if urlMatcher.find() {
                            param = &format!(",{}", &it[urlMatcher.end()..]);
                            it[0..urlMatcher.start()].to_string()
                        } else {
                            it.to_string()
                        }
                    }).or_else(|| matcher.group(2)).unwrap_or(matcher.group(3).unwrap()),
                )
            ));
            sb.append_str(param);
            sb.append_str("\">");
            appendPos = matcher.end();
        }
        if appendPos < keepImgHtml.len() {
            sb.append_str(&keepImgHtml[appendPos..keepImgHtml.len()]);
        }
        sb.to_string()
    }
}
