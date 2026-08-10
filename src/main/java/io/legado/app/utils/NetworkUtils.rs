pub struct NetworkUtils;

impl NetworkUtils {
    pub fn getUrl(response: &Response) -> String {
        let networkResponse = response.raw().networkResponse;
        match networkResponse {
            Some(nr) => nr.request.url.to_string(),
            None => response.raw().request.url.to_string(),
        }
    }

    fn notNeedEncoding() -> BitSet {
        let mut bitSet = BitSet::new(256);
        for i in 'a' as u32..='z' as u32 {
            bitSet.set(i);
        }
        for i in 'A' as u32..='Z' as u32 {
            bitSet.set(i);
        }
        for i in '0' as u32..='9' as u32 {
            bitSet.set(i);
        }
        for char in "+-_.$:()!*@&#,[]".chars() {
            bitSet.set(char as u32);
        }
        bitSet
    }

    /**
     * 支持JAVA的URLEncoder.encode出来的string做判断。 即: 将' '转成'+'
     * 0-9a-zA-Z保留 <br></br>
     * ! * ' ( ) ; : @ & = + $ , / ? # [ ] 保留
     * 其他字符转成%XX的格式，X是16进制的大写字符，范围是[0-9A-F]
     */
    pub fn hasUrlEncoded(str: &str) -> bool {
        let notNeedEncoding = Self::notNeedEncoding();
        let mut needEncode = false;
        let mut i = 0;
        while i < str.len() {
            let c = str.as_bytes()[i] as char;
            if notNeedEncoding.get(c as u32) {
                i += 1;
                continue;
            }
            if c == '%' && i + 2 < str.len() {
                // 判断是否符合urlEncode规范
                i += 1;
                let c1 = str.as_bytes()[i] as char;
                i += 1;
                let c2 = str.as_bytes()[i] as char;
                if Self::isDigit16Char(c1) && Self::isDigit16Char(c2) {
                    i += 1;
                    continue;
                }
            }
            // 其他字符，肯定需要urlEncode
            needEncode = true;
            break;
        }

        !needEncode
    }

    /**
     * 判断c是否是16进制的字符
     */
    fn isDigit16Char(c: char) -> bool {
        c >= '0' && c <= '9' || c >= 'A' && c <= 'F' || c >= 'a' && c <= 'f'
    }

    /**
     * 获取绝对地址
     */
    pub fn getAbsoluteURL(baseURL: Option<&str>, relativePath: &str) -> String {
        if baseURL == None || baseURL.unwrap().is_empty() {
            return relativePath.to_string();
        }
        if relativePath.is_empty() {
            return baseURL.unwrap().to_string();
        }
        let mut relativeUrl = relativePath.to_string();
        let result: Result<(), ()> = (|| {
            let absoluteUrl = URL::new(baseURL.unwrap().splitn(2, ",").next().unwrap());
            let parseUrl = URL::new_relative(&absoluteUrl, relativePath);
            relativeUrl = parseUrl.to_string();
            Ok(())
        })();
        if let Err(e) = result {
            e.printStackTrace();
        }
        relativeUrl
    }

    /**
     * 获取绝对地址
     */
    pub fn getAbsoluteURL_url(baseURL: Option<&URL>, relativePath: &str) -> String {
        if baseURL == None {
            return relativePath.to_string();
        }
        let mut relativeUrl = relativePath.to_string();
        let result: Result<(), ()> = (|| {
            let parseUrl = URL::new_relative(baseURL.unwrap(), relativePath);
            relativeUrl = parseUrl.to_string();
            Ok(())
        })();
        if let Err(e) = result {
            e.printStackTrace();
        }
        relativeUrl
    }

    pub fn getBaseUrl(url: Option<&str>) -> Option<String> {
        if url == None || !url.unwrap().starts_with("http") {
            return None;
        }
        let index = url.unwrap().find_index_of("/", 9);
        if index == -1 {
            Some(url.unwrap().to_string())
        } else {
            Some(url.unwrap()[0..index].to_string())
        }
    }

    pub fn getSubDomain(url: Option<&str>) -> String {
        let baseUrl = match Self::getBaseUrl(url) {
            Some(b) => b,
            None => return "".to_string(),
        };
        if baseUrl.find('.').unwrap_or(0usize.wrapping_sub(1)) == baseUrl.rfind('.').unwrap_or(0usize.wrapping_sub(1)) {
            baseUrl[baseUrl.rfind('/').unwrap() + 1..].to_string()
        } else {
            baseUrl[baseUrl.find('.').unwrap() + 1..].to_string()
        }
    }

    /**
     * Get local Ip address.
     */
    pub fn getLocalIPAddress() -> Option<InetAddress> {
        let enumeration: Option<Enumeration<NetworkInterface>> = (|| {
            Ok(NetworkInterface::getNetworkInterfaces())
        })()
        .ok();
        if let Some(enumeration) = enumeration {
            for nif in enumeration {
                let addresses = nif.inetAddresses;
                if let Some(addresses) = addresses {
                    for address in addresses {
                        if !address.isLoopbackAddress && Self::isIPv4Address(&address.hostAddress) {
                            return Some(address);
                        }
                    }
                }
            }
        }
        None
    }

    /**
     * Check if valid IPV4 address.
     *
     * @param input the address string to check for validity.
     * @return True if the input parameter is a valid IPv4 address.
     */
    pub fn isIPv4Address(input: &str) -> bool {
        Self::IPV4_PATTERN().matcher(input).matches()
    }

    /**
     * Ipv4 address check.
     */
    pub fn IPV4_PATTERN() -> Pattern {
        Pattern::compile(
            "^(" + "([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])\\.){3}"
                + "([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])$"
        )
    }
}
