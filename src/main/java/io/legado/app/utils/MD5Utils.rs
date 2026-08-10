/**
 * 将字符串转化为MD5
 */
pub struct MD5Utils;

impl MD5Utils {
    pub fn md5Encode(str: Option<&str>) -> String {
        if str == None {
            return "".to_string();
        }
        let mut reStr = "".to_string();
        let result: Result<(), String> = (|| {
            let md5 = MessageDigest::getInstance("MD5");
            let bytes = md5.digest(str.unwrap().as_bytes());
            let mut stringBuffer = String::new();
            for b in bytes {
                let bt = b as i32 & 0xff;
                if bt < 16 {
                    stringBuffer.push('0');
                }
                stringBuffer.push_str(&Integer::toHexString(bt));
            }
            reStr = stringBuffer;
            Ok(())
        })();
        if let Err(e) = result {
            e.printStackTrace();
        }
        reStr
    }

    pub fn md5Encode16(str: &str) -> String {
        let mut reStr = Self::md5Encode(Some(str));
        reStr = reStr[8..24].to_string();
        reStr
    }
}
