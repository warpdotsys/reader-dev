/**
 * 自动获取文件的编码
 * */
#[allow(dead_code, unused)]
pub struct EncodingDetect;

impl EncodingDetect {
    pub fn getHtmlEncode(bytes: &[u8]) -> String {
        let result: Option<String> = (|| {
            let htmlStr = String::from_utf8_lossy(bytes).to_string();
            let doc = Jsoup::parse(&htmlStr);
            let metaTags = doc.getElementsByTag("meta");
            let mut charsetStr: String;
            for metaTag in metaTags {
                charsetStr = metaTag.attr("charset");
                if !charsetStr.is_empty() {
                    return Some(charsetStr);
                }
                let httpEquiv = metaTag.attr("http-equiv");
                if httpEquiv.eq_ignore_ascii_case("content-type") {
                    let content = metaTag.attr("content");
                    if content.to_lowercase().contains("charset") {
                        charsetStr = content[
                            content.to_lowercase().find("charset").unwrap() + "charset=".len()..
                        ].to_string();
                    } else {
                        charsetStr = content[content.to_lowercase().find(";").unwrap() + 1..].to_string();
                    }
                    if !charsetStr.is_empty() {
                        return Some(charsetStr);
                    }
                }
            }
            None
        })();
        match result {
            Some(s) => s,
            None => Self::getEncode(bytes),
        }
    }

    pub fn getEncode(bytes: &[u8]) -> String {
        let detect = CharsetDetector::new().setText(bytes).detect();
        match detect {
            Some(m) => m.name,
            None => "UTF-8".to_string(),
        }
    }

    /**
     * 得到文件的编码
     */
    pub fn getEncode_filePath(filePath: &str) -> String {
        Self::getEncode_file(File::new(filePath))
    }

    /**
     * 得到文件的编码
     */
    pub fn getEncode_file(file: &File) -> String {
        let tempByte = Self::getFileBytes(Some(file));
        Self::getEncode(&tempByte)
    }

    fn getFileBytes(file: Option<&File>) -> Vec<u8> {
        let mut byteArray = vec![0u8; 8000];
        if let Some(file) = file {
            let read_result: Result<(), std::io::Error> = (|| {
                let mut input = FileInputStream::new(file);
                input.read(&mut byteArray);
                Ok(())
            })();
            if let Err(e) = read_result {
                System.err.println(format!("Error: {e}"));
            }
        }
        byteArray
    }
}
