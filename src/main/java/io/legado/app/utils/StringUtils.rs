pub struct StringUtils {
    TAG: &'static str,
}

impl StringUtils {
    pub const HOUR_OF_DAY: i64 = 24;
    pub const DAY_OF_YESTERDAY: i64 = 2;
    pub const TIME_UNIT: i64 = 60;

    fn new() -> StringUtils {
        StringUtils { TAG: "StringUtils" }
    }

    pub fn chnMap() -> HashMap<char, i32> {
        let mut map = HashMap::<char, i32>::new();
        let mut cnStr = "零一二三四五六七八九十";
        let mut c: Vec<char> = cnStr.chars().collect();
        for i in 0..=10 {
            map.insert(c[i], i as i32);
        }
        cnStr = "〇壹贰叁肆伍陆柒捌玖拾";
        c = cnStr.chars().collect();
        for i in 0..=10 {
            map.insert(c[i], i as i32);
        }
        map.insert('两', 2);
        map.insert('百', 100);
        map.insert('佰', 100);
        map.insert('千', 1000);
        map.insert('仟', 1000);
        map.insert('万', 10000);
        map.insert('亿', 100000000);
        map
    }

    //将时间转换成日期
    pub fn dateConvert(time: i64, pattern: &str) -> String {
        let date = Date::new(time);
        let format = SimpleDateFormat::new(pattern);
        format.format(&date)
    }

    //将日期转换成昨天、今天、明天
    pub fn dateConvert_source(source: &str, pattern: &str) -> String {
        let format = SimpleDateFormat::new(pattern);
        let calendar = Calendar::getInstance();
        let result: Result<String, ParseException> = (|| {
            let date = format.parse(source)?;
            let curTime = calendar.timeInMillis;
            calendar.time = date;
            //将MISC 转换成 sec
            let difSec = (curTime - date.time).abs() / 1000;
            let difMin = difSec / 60;
            let difHour = difMin / 60;
            let difDate = difHour / 60;
            let oldHour = calendar.get(Calendar::HOUR);
            //如果没有时间
            if oldHour == 0 {
                //比日期:昨天今天和明天
                if difDate == 0 {
                    return Ok("今天".to_string());
                } else if difDate < Self::DAY_OF_YESTERDAY {
                    return Ok("昨天".to_string());
                } else {
                    let convertFormat = SimpleDateFormat::new("yyyy-MM-dd");
                    return Ok(convertFormat.format(&date));
                }
            }

            if difSec < Self::TIME_UNIT {
                Ok(difSec.to_string() + "秒前")
            } else if difMin < Self::TIME_UNIT {
                Ok(difMin.to_string() + "分钟前")
            } else if difHour < Self::HOUR_OF_DAY {
                Ok(difHour.to_string() + "小时前")
            } else if difDate < Self::DAY_OF_YESTERDAY {
                Ok("昨天".to_string())
            } else {
                let convertFormat = SimpleDateFormat::new("yyyy-MM-dd");
                Ok(convertFormat.format(&date))
            }
        })();
        match result {
            Ok(s) => s,
            Err(e) => {
                e.printStackTrace();
                "".to_string()
            }
        }
    }

    /**
     * 单位转换
     */
    pub fn toSize(length: i64) -> String {
        if length <= 0 {
            return "0".to_string();
        }
        let units = ["b", "kb", "M", "G", "T"];
        //计算单位的，原理是利用lg,公式是 lg(1024^n) = nlg(1024)，最后 nlg(1024)/lg(1024) = n。
        let digitGroups = (log10(length as f64) / log10(1024.0)) as i32;
        //计算原理是，size/单位值。单位值指的是:比如说b = 1024,KB = 1024^2
        DecimalFormat::new("#,##0.##")
            .format(length as f64 / 1024.0f64.powf(digitGroups as f64)) + " " + units[digitGroups as usize]
    }

    pub fn toFirstCapital(str: &str) -> String {
        str[0..1].to_uppercase() + &str[1..]
    }

    /**
     * 将文本中的半角字符，转换成全角字符
     */
    pub fn halfToFull(input: &str) -> String {
        let mut c: Vec<char> = input.chars().collect();
        for i in 0..c.len() {
            if c[i] as u32 == 32
            //半角空格
            {
                c[i] = char::from_u32(12288).unwrap();
                continue;
            }
            //根据实际情况，过滤不需要转换的符号
            //if (c[i] == 46) //半角点号，不转换
            // continue;

            if (c[i] as u32) >= 33 && (c[i] as u32) <= 126
            //其他符号都转换为全角
            {
                c[i] = char::from_u32(c[i] as u32 + 65248).unwrap();
            }
        }
        c.into_iter().collect()
    }

    /**
     * 字符串全角转换为半角
     */
    pub fn fullToHalf(input: &str) -> String {
        let mut c: Vec<char> = input.chars().collect();
        for i in 0..c.len() {
            if c[i] as u32 == 12288
            //全角空格
            {
                c[i] = char::from_u32(32).unwrap();
                continue;
            }

            if (c[i] as u32) >= 65281 && (c[i] as u32) <= 65374 {
                c[i] = char::from_u32(c[i] as u32 - 65248).unwrap();
            }
        }
        c.into_iter().collect()
    }

    /**
     * 中文大写数字转数字
     */
    pub fn chineseNumToInt(chNum: &str) -> i32 {
        let map = Self::chnMap();
        let mut result = 0;
        let mut tmp = 0;
        let mut billion = 0;
        let mut cn: Vec<char> = chNum.chars().collect();

        // "一零二五" 形式
        if cn.len() > 1 && Regex::new("^[〇零一二三四五六七八九壹贰叁肆伍陆柒捌玖]$").matches(chNum) {
            for i in 0..cn.len() {
                cn[i] = char::from_u32(48 + map[&cn[i]] as u32).unwrap();
            }
            return Integer::parseInt(&cn.into_iter().collect::<String>());
        }

        // "一千零二十五", "一千二" 形式
        let result2: Result<i32, ()> = (|| {
            for i in 0..cn.len() {
                let tmpNum = map[&cn[i]];
                if tmpNum == 100000000 {
                    result += tmp;
                    result *= tmpNum;
                    billion = billion * 100000000 + result;
                    result = 0;
                    tmp = 0;
                } else if tmpNum == 10000 {
                    result += tmp;
                    result *= tmpNum;
                    tmp = 0;
                } else if tmpNum >= 10 {
                    if tmp == 0 {
                        tmp = 1;
                    }
                    result += tmpNum * tmp;
                    tmp = 0;
                } else {
                    tmp = if i >= 2 && i == cn.len() - 1 && map[&cn[i - 1]] > 10 {
                        tmpNum * map[&cn[i - 1]] / 10
                    } else {
                        tmp * 10 + tmpNum
                    };
                }
            }
            Ok(result + tmp + billion)
        })();
        result2.unwrap_or(-1)
    }

    /**
     * 字符串转数字
     */
    pub fn stringToInt(str: Option<&str>) -> i32 {
        if let Some(str) = str {
            let num = Self::fullToHalf(str).replace(&Regex::new("\\s+"), "");
            let parsed: Result<i32, ()> = (|| Ok(Integer::parseInt(&num)))();
            parsed.unwrap_or_else(|_| Self::chineseNumToInt(&num))
        } else {
            -1
        }
    }

    /**
     * 是否包含数字
     */
    pub fn isContainNumber(company: &str) -> bool {
        let p = Pattern::compile("[0-9]+");
        let m = p.matcher(company);
        m.find()
    }

    /**
     * 是否数字
     */
    pub fn isNumeric(str: &str) -> bool {
        let pattern = Pattern::compile("-?[0-9]+");
        let isNum = pattern.matcher(str);
        isNum.matches()
    }

    pub fn wordCountFormat(wc: Option<&str>) -> String {
        if wc == None {
            return "".to_string();
        }
        let mut wordsS = "".to_string();
        if Self::isNumeric(wc.unwrap()) {
            let words: i32 = wc.unwrap().parse().unwrap();
            if words > 0 {
                wordsS = words.to_string() + "字";
                if words > 10000 {
                    let df = DecimalFormat::new("#.#");
                    wordsS = df.format(words as f64 * 1.0 / 10000.0) + "万字";
                }
            }
        } else {
            wordsS = wc.unwrap().to_string();
        }
        wordsS
    }

    /**
     * 移除字符串首尾空字符的高效方法(利用ASCII值判断,包括全角空格)
     */
    pub fn trim(s: &str) -> String {
        if s.is_empty() {
            return "".to_string();
        }
        let chars: Vec<char> = s.chars().collect();
        let mut start = 0;
        let len = chars.len();
        let mut end = len - 1;
        while start < end && (chars[start] as u32 <= 0x20 || chars[start] == '　') {
            start += 1;
        }
        while start < end && (chars[end] as u32 <= 0x20 || chars[end] == '　') {
            end -= 1;
        }
        if end < len {
            end += 1;
        }
        if start > 0 || end < len {
            chars[start..end].iter().collect()
        } else {
            s.to_string()
        }
    }

    /**
     * 重复字符串
     */
    pub fn repeat(str: &str, n: i32) -> String {
        let mut stringBuilder = String::new();
        for _ in 0..n {
            stringBuilder.push_str(str);
        }
        stringBuilder
    }

    /**
     * 移除UTF头
     */
    pub fn removeUTFCharacters(data: Option<&str>) -> Option<String> {
        let data = data?;
        let p = Pattern::compile("\\\\u(\\p{XDigit}{4})");
        let m = p.matcher(data);
        let mut buf = StringBuffer::with_capacity(data.len());
        while m.find() {
            let ch = char::from_u32(Integer::parseInt(m.group(1).unwrap(), 16) as u32).unwrap();
            m.appendReplacement(&mut buf, Matcher::quoteReplacement(&ch.to_string()));
        }
        m.appendTail(&mut buf);
        Some(buf.to_string())
    }

    pub fn formatHtml(html: &str) -> String {
        if TextUtils::isEmpty(html) {
            "".to_string()
        } else {
            html.replace(&Regex::new("(?i)<(br[\\s/]*|/*p.*?|/*div.*?)>"), "\n")// 替换特定标签为换行符
                .replace(&Regex::new("<[script>]*.*?>|&nbsp;"), "")// 删除script标签对和空格转义符
                .replace(&Regex::new("\\s*\\n+\\s*"), "\n　　")// 移除空行,并增加段前缩进2个汉字
                .replace(&Regex::new("^[\\n\\s]+"), "　　")//移除开头空行,并增加段前缩进2个汉字
                .replace(&Regex::new("[\\n\\s]+$"), "") //移除尾部空行
        }
    }

    pub fn byteToHexString(bytes: Option<&[u8]>) -> String {
        if bytes == None {
            return "".to_string();
        }
        let mut sb = String::with_capacity(bytes.unwrap().len() * 2);
        for b in bytes.unwrap() {
            let hex = 0xff & *b as i32;
            if hex < 16 {
                sb.push('0');
            }
            sb.push_str(&Integer::toHexString(hex));
        }
        sb
    }

    pub fn hexStringToByte(hexString: &str) -> Vec<u8> {
        let hexStr = hexString.replace(" ", "");
        let len = hexStr.len();
        let mut bytes = vec![0u8; len / 2];
        let mut i = 0;
        while i < len {
            // 两位一组，表示一个字节,把这样表示的16进制字符串，还原成一个字节
            bytes[i / 2] = ((Character::digit(hexString.as_bytes()[i] as char, 16) << 4)
                + Character::digit(hexString.as_bytes()[i + 1] as char, 16)) as u8;
            i += 2;
        }
        bytes
    }
}
