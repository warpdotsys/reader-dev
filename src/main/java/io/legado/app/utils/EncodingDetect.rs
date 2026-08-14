use crate::prelude::*;
// fix: E0659 歧义——prelude glob 同时导出 stubs 与 ResourceUtil 模块的 File/FileInputStream，显式导入覆盖
use crate::stubs::{File, FileInputStream};
/**
 * 自动获取文件的编码
 * */
#[allow(dead_code, unused)]
pub struct EncodingDetect;

impl EncodingDetect {
    pub fn getHtmlEncode(bytes: &[u8]) -> String {
        let result: Option<String> = (|| {
            let htmlStr = String::from_utf8_lossy(bytes).to_string();
            let doc = Jsoup::parse(htmlStr);
            let metaTags = doc.getElementsByTag("meta");
            let mut charsetStr: String;
            for metaTag in &metaTags.list {
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
        // fix: 先验证 UTF-8 有效性（原直走 CharsetDetector——单字节识别器会把 UTF-8 中文文本误判为
        //      KOI8-R/windows-1256 → 乱码；UTF-8 合法时无需检测）
        if std::str::from_utf8(bytes).is_ok() {
            return "UTF-8".to_string();
        }
        let detect = CharsetDetector::new().set_text(bytes.to_vec()).detect();
        match detect {
            Some(m) => m.get_name(),
            None => "UTF-8".to_string(),
        }
    }

    /**
     * 得到文件的编码
     */
    pub fn getEncode_filePath(filePath: &str) -> String {
        Self::getEncode_file(&File::new(filePath))
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
                let read_len = byteArray.len();
                input.read(&mut byteArray, 0, read_len);
                Ok(())
            })();
            if let Err(e) = read_result {
                System::err.println(format!("Error: {e}"));
            }
        }
        byteArray
    }

    // fix: LocalBook 转录调用别名（Kotlin EncodingDetect.getEncode(File)）
    pub fn get_encode(file: File) -> String {
        Self::getEncode_file(&file)
    }

    // fix: TextFile 转录调用别名（Kotlin EncodingDetect.getEncodeFromBytes(ByteArray)）
    pub fn get_encode_from_bytes(bytes: Vec<u8>) -> String {
        Self::getEncode(&bytes)
    }
}
