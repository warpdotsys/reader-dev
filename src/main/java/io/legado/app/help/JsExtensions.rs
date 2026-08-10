// package io.legado.app.help
//
// import cn.hutool.crypto.digest.DigestUtil
// import cn.hutool.crypto.symmetric.AES
// import cn.hutool.crypto.symmetric.DESede
// import io.legado.app.adapters.ReaderAdapterHelper
// import io.legado.app.utils.Base64
// import io.legado.app.constant.AppConst.dateFormat
// import io.legado.app.help.http.*
// import io.legado.app.model.Debug
// import io.legado.app.model.DebugLog
// import io.legado.app.model.analyzeRule.AnalyzeUrl
// import io.legado.app.model.analyzeRule.QueryTTF
// import io.legado.app.utils.*
// import io.legado.app.data.entities.BaseSource
// import io.legado.app.exception.NoStackTraceException
// import kotlinx.coroutines.Dispatchers.IO
// import kotlinx.coroutines.async
// import kotlinx.coroutines.runBlocking
// import org.jsoup.Connection
// import org.jsoup.Jsoup
// import java.io.ByteArrayInputStream
// import java.io.ByteArrayOutputStream
// import java.io.File
// import java.net.URLEncoder
// import java.nio.charset.Charset
// import java.util.*
// import java.util.zip.ZipEntry
// import java.util.zip.ZipInputStream
// import java.text.SimpleDateFormat

/**
 * js扩展类, 在js中通过java变量调用
 * 所有对于文件的读写删操作都是相对路径,只能操作阅读缓存内的文件
 * /android/data/{package}/cache/...
 */
// @Suppress("unused")
pub trait JsExtensions {

    fn get_source(&self) -> Option<BaseSource>;

    fn get_user_name_space(&self) -> String;

    fn get_logger(&self) -> Option<DebugLog>;

    /**
     * 访问网络,返回String
     */
    fn ajax(&self, url_str: &str) -> Option<String> {
        // return runBlocking {
        //     kotlin.runCatching {
        //         val analyzeUrl = AnalyzeUrl(urlStr, source = getSource())
        //         analyzeUrl.getStrResponse(urlStr).body
        //     }.onFailure {
        //         it.printOnDebug()
        //     }.getOrElse {
        //         it.msg
        //     }
        // }
        run_blocking(|| async {
            let analyze_url = AnalyzeUrl::new(url_str, source = self.get_source());
            match analyze_url.get_str_response(url_str) {
                Ok(response) => response.body,
                Err(e) => {
                    e.print_on_debug();
                    e.msg
                }
            }
        })
    }

    /**
     * 并发访问网络
     */
    fn ajax_all(&self, url_list: &[&str]) -> Vec<Option<StrResponse>> {
        // return runBlocking {
        //     val asyncArray = Array(urlList.size) {
        //         async(IO) {
        //             val url = urlList[it]
        //             val analyzeUrl = AnalyzeUrl(url, source = getSource())
        //             analyzeUrl.getStrResponse(url)
        //         }
        //     }
        //     val resArray = Array<StrResponse?>(urlList.size) {
        //         asyncArray[it].await()
        //     }
        //     resArray
        // }
        run_blocking(|| async {
            let mut async_array: Vec<Deferred<StrResponse>> = Vec::new();
            for it in 0..url_list.len() {
                let url = url_list[it];
                async_array.push(scope_async(IO, || async move {
                    let analyze_url = AnalyzeUrl::new(url, source = self.get_source());
                    analyze_url.get_str_response(url)
                }));
            }
            let mut res_array: Vec<Option<StrResponse>> = Vec::new();
            for it in 0..url_list.len() {
                res_array.push(async_array[it].await());
            }
            res_array
        })
    }

    /**
     * 访问网络,返回Response<String>
     */
    fn connect(&self, url_str: &str) -> StrResponse {
        // return runBlocking {
        //     val analyzeUrl = AnalyzeUrl(urlStr, source = getSource())
        //     kotlin.runCatching {
        //         analyzeUrl.getStrResponseAwait()
        //     }.onFailure {
        //         it.printOnDebug()
        //     }.getOrElse {
        //         StrResponse(analyzeUrl.url, it.localizedMessage)
        //     }
        // }
        run_blocking(|| async {
            let analyze_url = AnalyzeUrl::new(url_str, source = self.get_source());
            match analyze_url.get_str_response_await().await {
                Ok(response) => response,
                Err(e) => {
                    e.print_on_debug();
                    StrResponse::new_url(analyze_url.url, e.localized_message)
                }
            }
        })
    }

    fn connect_with_header(&self, url_str: &str, header: Option<&str>) -> StrResponse {
        // return runBlocking {
        //     val headerMap = GSON.fromJsonObject<Map<String, String>>(header).getOrNull()
        //     val analyzeUrl = AnalyzeUrl(urlStr, headerMapF = headerMap, source = getSource())
        //     kotlin.runCatching {
        //         analyzeUrl.getStrResponseAwait()
        //     }.onFailure {
        //         it.printOnDebug()
        //     }.getOrElse {
        //         StrResponse(analyzeUrl.url, it.localizedMessage)
        //     }
        // }
        run_blocking(|| async {
            let header_map = GSON::from_json_object::<std::collections::HashMap<String, String>>(header).get_or_none();
            let analyze_url = AnalyzeUrl::new(url_str, header_map_f = header_map, source = self.get_source());
            match analyze_url.get_str_response_await().await {
                Ok(response) => response,
                Err(e) => {
                    e.print_on_debug();
                    StrResponse::new_url(analyze_url.url, e.localized_message)
                }
            }
        })
    }

    /**
     * 使用webView访问网络
     * @param html 直接用webView载入的html, 如果html为空直接访问url
     * @param url html内如果有相对路径的资源不传入url访问不了
     * @param js 用来取返回值的js语句, 没有就返回整个源代码
     * @return 返回js获取的内容
     */
    fn web_view(&self, html: Option<&str>, url: Option<&str>, js: Option<&str>) -> Option<String> {
        // return null
        None
    }

    /**
     * 可从网络，本地文件(阅读私有缓存目录和书籍保存位置支持相对路径)导入JavaScript脚本
     */
    fn import_script(&self, path: &str) -> String {
        // val result = when {
        //     path.startsWith("http") -> cacheFile(path) ?: ""
        //     path.startsWith("/storage") -> FileUtils.readText(path)
        //     else -> readTxtFile(path)
        // }
        let result: String = if path.starts_with("http") {
            self.cache_file(path).unwrap_or("".to_string())
        } else if path.starts_with("/storage") {
            FileUtils::read_text(path)
        } else {
            self.read_txt_file(path)
        };
        // if (result.isBlank()) throw NoStackTraceException("$path 内容获取失败或者为空")
        if result.trim().is_empty() {
            panic!("{} 内容获取失败或者为空", path)
        }
        result
    }

    /**
     * 缓存以文本方式保存的文件 如.js .txt等
     */
    fn cache_file(&self, url_str: &str) -> Option<String> {
        // return cacheFile(urlStr, 0)
        self.cache_file_save_time(url_str, 0)
    }

    /**
     * 缓存以文本方式保存的文件 如.js .txt等
     * @param urlStr 网络文件的链接
     * @param saveTime 缓存时间，单位：秒
     * @return 返回缓存后的文件内容
     */
    fn cache_file_save_time(&self, url_str: &str, save_time: i32) -> Option<String> {
        // val key = md5Encode16(urlStr)
        let key = self.md5_encode16(url_str);
        // val cacheManager = CacheManager(getUserNameSpace())
        let cache_manager = CacheManager::new(self.get_user_name_space());
        // val cache = cacheManager.getFile(key)
        let cache = cache_manager.get_file(&key);
        // if (cache.isNullOrBlank()) {
        //     log("首次下载 $urlStr")
        //     val value = ajax(urlStr) ?: return null
        //     cacheManager.putFile(key, value, saveTime)
        //     return value
        // }
        // return cache
        match cache {
            Some(cache) if !cache.trim().is_empty() => Some(cache),
            _ => {
                self.log(format!("首次下载 {}", url_str));
                let value = match self.ajax(url_str) {
                    Some(value) => value,
                    None => return None,
                };
                cache_manager.put_file(&key, &value, save_time);
                Some(value)
            }
        }
    }

    /**
     *js实现读取cookie
     */
    fn get_cookie(&self, tag: &str, key: Option<&str>) -> String {
        // val cookieStore = CookieStore(getUserNameSpace())
        let cookie_store = CookieStore::new(self.get_user_name_space());
        // val cookie = cookieStore.getCookie(tag)
        let cookie = cookie_store.get_cookie(tag);
        // val cookieMap = cookieStore.cookieToMap(cookie)
        let cookie_map = cookie_store.cookie_to_map(&cookie);
        // return if (key != null) {
        //     cookieMap[key] ?: ""
        // } else {
        //     cookie
        // }
        match key {
            Some(key) => cookie_map.get(key).cloned().unwrap_or("".to_string()),
            None => cookie,
        }
    }

    /**
     * 实现16进制字符串转文件
     * @param content 需要转成文件的16进制字符串
     * @param url 通过url里的参数来判断文件类型
     * @return 相对路径
     */
    fn download_file(&self, content: &str, url: &str) -> String {
        // val type = AnalyzeUrl(url).type ?: return ""
        let type_ = match AnalyzeUrl::new(url, source = None).type_ {
            Some(type_) => type_,
            None => return "".to_string(),
        };
        // val zipPath = FileUtils.getPath(
        //     FileUtils.createFolderIfNotExist(FileUtils.getCachePath()),
        //     "${MD5Utils.md5Encode16(url)}.${type}"
        // )
        let zip_path = FileUtils::get_path(
            FileUtils::create_folder_if_not_exist(FileUtils::get_cache_path()),
            format!("{}.{}", self.md5_encode16(url), type_),
        );
        FileUtils::delete_file(&zip_path);
        let zip_file = FileUtils::create_file_if_not_exist(&zip_path);
        // StringUtils.hexStringToByte(content).let {
        //     if (it.isNotEmpty()) {
        //         zipFile.writeBytes(it)
        //     }
        // }
        let bytes = StringUtils::hex_string_to_byte(content);
        if !bytes.is_empty() {
            zip_file.write_bytes(bytes);
        }
        // return zipPath.substring(FileUtils.getCachePath().length)
        zip_path[FileUtils::get_cache_path().len()..].to_string()
    }

    /**
     * js实现重定向拦截,网络访问get
     */
    fn get(&self, url_str: &str, headers: &std::collections::HashMap<String, String>) -> ConnectionResponse {
        // val response = Jsoup.connect(urlStr)
        //     .sslSocketFactory(SSLHelper.unsafeSSLSocketFactory)
        //     .ignoreContentType(true)
        //     .followRedirects(false)
        //     .headers(headers)
        //     .method(Connection.Method.GET)
        //     .execute()
        let response = Jsoup::connect(url_str)
            .ssl_socket_factory(SSLHelper::unsafe_ssl_socket_factory())
            .ignore_content_type(true)
            .follow_redirects(false)
            .headers(headers)
            .method(ConnectionMethod::GET)
            .execute();
        // val cookieStore = CookieStore(getUserNameSpace())
        let cookie_store = CookieStore::new(self.get_user_name_space());
        // cookieStore.mapToCookie(response.cookies())?.let {
        //     val domain = NetworkUtils.getSubDomain(urlStr)
        //     cookieStore.replaceCookie("${domain}_cookieJar", it)
        // }
        if let Some(it) = cookie_store.map_to_cookie(&response.cookies()) {
            let domain = NetworkUtils::get_sub_domain(url_str);
            cookie_store.replace_cookie(&format!("{}_cookieJar", domain), &it);
        }
        response
    }

    /**
     * 网络访问head
     */
    fn head(&self, url_str: &str, headers: &std::collections::HashMap<String, String>) -> ConnectionResponse {
        // val response = Jsoup.connect(urlStr)
        //     .sslSocketFactory(SSLHelper.unsafeSSLSocketFactory)
        //     .ignoreContentType(true)
        //     .followRedirects(false)
        //     .headers(headers)
        //     .method(Connection.Method.HEAD)
        //     .execute()
        let response = Jsoup::connect(url_str)
            .ssl_socket_factory(SSLHelper::unsafe_ssl_socket_factory())
            .ignore_content_type(true)
            .follow_redirects(false)
            .headers(headers)
            .method(ConnectionMethod::HEAD)
            .execute();
        // val cookieStore = CookieStore(getUserNameSpace())
        let cookie_store = CookieStore::new(self.get_user_name_space());
        // cookieStore.mapToCookie(response.cookies())?.let {
        //     val domain = NetworkUtils.getSubDomain(urlStr)
        //     cookieStore.replaceCookie("${domain}_cookieJar", it)
        // }
        if let Some(it) = cookie_store.map_to_cookie(&response.cookies()) {
            let domain = NetworkUtils::get_sub_domain(url_str);
            cookie_store.replace_cookie(&format!("{}_cookieJar", domain), &it);
        }
        response
    }

    /**
     * 网络访问post
     */
    fn post(&self, url_str: &str, body: &str, headers: &std::collections::HashMap<String, String>) -> ConnectionResponse {
        // val response = Jsoup.connect(urlStr)
        //     .sslSocketFactory(SSLHelper.unsafeSSLSocketFactory)
        //     .ignoreContentType(true)
        //     .followRedirects(false)
        //     .requestBody(body)
        //     .headers(headers)
        //     .method(Connection.Method.POST)
        //     .execute()
        let response = Jsoup::connect(url_str)
            .ssl_socket_factory(SSLHelper::unsafe_ssl_socket_factory())
            .ignore_content_type(true)
            .follow_redirects(false)
            .request_body(body)
            .headers(headers)
            .method(ConnectionMethod::POST)
            .execute();
        // val cookieStore = CookieStore(getUserNameSpace())
        let cookie_store = CookieStore::new(self.get_user_name_space());
        // cookieStore.mapToCookie(response.cookies())?.let {
        //     val domain = NetworkUtils.getSubDomain(urlStr)
        //     cookieStore.replaceCookie("${domain}_cookieJar", it)
        // }
        if let Some(it) = cookie_store.map_to_cookie(&response.cookies()) {
            let domain = NetworkUtils::get_sub_domain(url_str);
            cookie_store.replace_cookie(&format!("{}_cookieJar", domain), &it);
        }
        response
    }

    /**
     * js实现解码,不能删
     */
    fn base64_decode(&self, str: &str) -> String {
        // return EncoderUtils.base64Decode(str, Base64.NO_WRAP)
        EncoderUtils::base64_decode(str, Base64::NO_WRAP)
    }

    fn base64_decode_with_flags(&self, str: &str, flags: i32) -> String {
        // return EncoderUtils.base64Decode(str, flags)
        EncoderUtils::base64_decode(str, flags)
    }

    fn base64_decode_to_byte_array(&self, str: Option<&str>) -> Option<Vec<u8>> {
        // if (str.isNullOrBlank()) {
        //     return null
        // }
        // return Base64.decode(str, Base64.DEFAULT)
        match str {
            Some(str) if !str.trim().is_empty() => Some(Base64::decode(str, Base64::DEFAULT)),
            _ => None,
        }
    }

    fn base64_decode_to_byte_array_with_flags(&self, str: Option<&str>, flags: i32) -> Option<Vec<u8>> {
        // if (str.isNullOrBlank()) {
        //     return null
        // }
        // return Base64.decode(str, flags)
        match str {
            Some(str) if !str.trim().is_empty() => Some(Base64::decode(str, flags)),
            _ => None,
        }
    }

    fn base64_encode(&self, str: &str) -> Option<String> {
        // return EncoderUtils.base64Encode(str, Base64.NO_WRAP)
        EncoderUtils::base64_encode(str, Base64::NO_WRAP)
    }

    fn base64_encode_with_flags(&self, str: &str, flags: i32) -> Option<String> {
        // return EncoderUtils.base64Encode(str, flags)
        EncoderUtils::base64_encode(str, flags)
    }

    fn md5_encode(&self, str: &str) -> String {
        // return MD5Utils.md5Encode(str)
        MD5Utils::md5_encode(str)
    }

    fn md5_encode16(&self, str: &str) -> String {
        // return MD5Utils.md5Encode16(str)
        MD5Utils::md5_encode16(str)
    }

    /**
     * 格式化时间
     */
    fn time_format_utc(&self, time: i64, format: &str, sh: i32) -> Option<String> {
        // val utc = SimpleTimeZone(sh, "UTC")
        let utc = SimpleTimeZone::new(sh, "UTC");
        // return SimpleDateFormat(format, Locale.getDefault()).run {
        //     timeZone = utc
        //     format(Date(time))
        // }
        Some(SimpleDateFormat::new(format, Locale::get_default()).apply(|it| {
            it.time_zone = utc;
            it.format(Date::new(time))
        }))
    }

    /**
     * 时间格式化
     */
    fn time_format(&self, time: i64) -> String {
        // return dateFormat.format(Date(time))
        AppConst::date_format().format(Date::new(time))
    }

    /**
     * utf8编码转gbk编码
     */
    fn utf8_to_gbk(&self, str: &str) -> String {
        // val utf8 = String(str.toByteArray(charset("UTF-8")))
        let utf8 = String::from_utf8_lossy(&str.to_byte_array("UTF-8")).into_owned();
        // val unicode = String(utf8.toByteArray(), charset("UTF-8"))
        let unicode = String::from_utf8_lossy(&utf8.to_byte_array()).into_owned();
        // return String(unicode.toByteArray(charset("GBK")), Charsets.UTF_8)
        String::from_utf8_lossy(&unicode.to_byte_array("GBK")).into_owned()
    }

    fn encode_uri(&self, str: &str) -> String {
        // return try {
        //     URLEncoder.encode(str, "UTF-8")
        // } catch (e: Exception) {
        //     ""
        // }
        match URLEncoder::encode(str, "UTF-8") {
            Ok(value) => value,
            Err(_e) => "".to_string(),
        }
    }

    fn encode_uri_with_enc(&self, str: &str, enc: &str) -> String {
        // return try {
        //     URLEncoder.encode(str, enc)
        // } catch (e: Exception) {
        //     ""
        // }
        match URLEncoder::encode(str, enc) {
            Ok(value) => value,
            Err(_e) => "".to_string(),
        }
    }

    fn html_format(&self, str: &str) -> String {
        // return HtmlFormatter.formatKeepImg(str)
        HtmlFormatter::format_keep_img(str)
    }

    //****************文件操作******************//

    /**
     * 获取本地文件
     * @param path 相对路径
     * @return File
     */
    fn get_file(&self, path: &str) -> File {
        // val cachePath = ReaderAdapterHelper.getAdapter().getCacheDir()
        let cache_path = ReaderAdapterHelper::get_adapter().get_cache_dir();
        // val aPath: String = if (path.startsWith(File.separator)) {
        //     cachePath + path
        // } else {
        //     cachePath + File.separator + path
        // }
        let a_path: String = if path.starts_with(File::separator()) {
            format!("{}{}", cache_path, path)
        } else {
            format!("{}{}{}", cache_path, File::separator(), path)
        };
        // return File(aPath)
        File::new(&a_path)
    }

    fn read_file(&self, path: &str) -> Option<Vec<u8>> {
        let file = self.get_file(path);
        if file.exists() {
            return Some(file.read_bytes());
        }
        None
    }

    fn read_txt_file(&self, path: &str) -> String {
        let file = self.get_file(path);
        if file.exists() {
            // val charsetName = EncodingDetect.getEncode(file)
            let charset_name = EncodingDetect::get_encode(&file);
            // return String(file.readBytes(), charset(charsetName))
            return String::from_utf8_lossy(&file.read_bytes()).into_owned();
        }
        "".to_string()
    }

    fn read_txt_file_with_charset(&self, path: &str, charset_name: &str) -> String {
        let file = self.get_file(path);
        if file.exists() {
            // return String(file.readBytes(), charset(charsetName))
            return String::from_utf8_lossy(&file.read_bytes()).into_owned();
        }
        "".to_string()
    }

    /**
     * 删除本地文件
     */
    fn delete_file(&self, path: &str) {
        let file = self.get_file(path);
        FileUtils::delete(&file, true);
    }

    /**
     * js实现压缩文件解压
     * @param zipPath 相对路径
     * @return 相对路径
     */
    fn unzip_file(&self, zip_path: &str) -> String {
        // if (zipPath.isEmpty()) return ""
        if zip_path.is_empty() {
            return "".to_string();
        }
        // val unzipPath = FileUtils.getPath(
        //     FileUtils.createFolderIfNotExist(FileUtils.getCachePath()),
        //     FileUtils.getNameExcludeExtension(zipPath)
        // )
        let unzip_path = FileUtils::get_path(
            FileUtils::create_folder_if_not_exist(FileUtils::get_cache_path()),
            FileUtils::get_name_exclude_extension(zip_path),
        );
        FileUtils::delete_file(&unzip_path);
        let zip_file = self.get_file(zip_path);
        let unzip_folder = FileUtils::create_folder_if_not_exist(&unzip_path);
        ZipUtils::unzip_file(&zip_file, &unzip_folder);
        FileUtils::delete_file(&zip_file.absolute_path());
        // return unzipPath.substring(FileUtils.getCachePath().length)
        unzip_path[FileUtils::get_cache_path().len()..].to_string()
    }

    /**
     * js实现文件夹内所有文件读取
     */
    fn get_txt_in_folder(&self, unzip_path: &str) -> String {
        // if (unzipPath.isEmpty()) return ""
        if unzip_path.is_empty() {
            return "".to_string();
        }
        let unzip_folder = self.get_file(unzip_path);
        // val contents = StringBuilder()
        let mut contents = String::new();
        // unzipFolder.listFiles().let {
        //     if (it != null) {
        //         for (f in it) {
        //             val charsetName = EncodingDetect.getEncode(f)
        //             contents.append(String(f.readBytes(), charset(charsetName)))
        //                 .append("\n")
        //         }
        //         contents.deleteCharAt(contents.length - 1)
        //     }
        // }
        if let Some(files) = unzip_folder.list_files() {
            for f in files {
                let charset_name = EncodingDetect::get_encode(&f);
                contents.push_str(&String::from_utf8_lossy(&f.read_bytes()));
                contents.push_str("\n");
            }
            // contents.deleteCharAt(contents.length - 1)
            contents.pop();
        }
        FileUtils::delete_file(&unzip_folder.absolute_path());
        contents
    }

    /**
     * 获取网络zip文件里面的数据
     * @param url zip文件的链接或十六进制字符串
     * @param path 所需获取文件在zip内的路径
     * @return zip指定文件的数据
     */
    fn get_zip_string_content(&self, url: &str, path: &str) -> String {
        // val byteArray = getZipByteArrayContent(url, path) ?: return ""
        let byte_array = match self.get_zip_byte_array_content(url, path) {
            Some(byte_array) => byte_array,
            None => return "".to_string(),
        };
        // val charsetName = EncodingDetect.getEncode(byteArray)
        let charset_name = EncodingDetect::get_encode_bytes(&byte_array);
        // return String(byteArray, Charset.forName(charsetName))
        String::from_utf8_lossy(&byte_array).into_owned()
    }

    fn get_zip_string_content_with_charset(&self, url: &str, path: &str, charset_name: &str) -> String {
        // val byteArray = getZipByteArrayContent(url, path) ?: return ""
        let byte_array = match self.get_zip_byte_array_content(url, path) {
            Some(byte_array) => byte_array,
            None => return "".to_string(),
        };
        // return String(byteArray, Charset.forName(charsetName))
        String::from_utf8_lossy(&byte_array).into_owned()
    }

    /**
     * 获取网络zip文件里面的数据
     * @param url zip文件的链接或十六进制字符串
     * @param path 所需获取文件在zip内的路径
     * @return zip指定文件的数据
     */
    fn get_zip_byte_array_content(&self, url: &str, path: &str) -> Option<Vec<u8>> {
        // val bytes = if (url.startsWith("http://") || url.startsWith("https://")) {
        //     runBlocking {
        //         return@runBlocking okHttpClient.newCall { url(url) }.bytes()
        //     }
        // } else {
        //     StringUtils.hexStringToByte(url)
        // }
        let bytes: Vec<u8> = if url.starts_with("http://") || url.starts_with("https://") {
            run_blocking(|| async {
                ok_http_client().new_call(0, |builder| url(builder, url)).await.bytes()
            })
        } else {
            StringUtils::hex_string_to_byte(url)
        };
        // val bos = ByteArrayOutputStream()
        let mut bos = ByteArrayOutputStream::new();
        // val zis = ZipInputStream(ByteArrayInputStream(bytes))
        let mut zis = ZipInputStream::new(ByteArrayInputStream::new(&bytes));
        // var entry: ZipEntry? = zis.nextEntry
        let mut entry: Option<ZipEntry> = zis.next_entry();
        // while (entry != null) {
        //     if (entry.name.equals(path)) {
        //         zis.use { it.copyTo(bos) }
        //         return bos.toByteArray()
        //     }
        //     entry = zis.nextEntry
        // }
        while let Some(entry) = entry {
            if entry.name == path {
                // zis.use { it.copyTo(bos) }
                zis.copy_to(&mut bos);
                return Some(bos.to_byte_array());
            }
            entry = zis.next_entry();
        }
        Debug::log("getZipContent 未发现内容");

        None
    }
}
    //******************文件操作************************//

    /**
     * 解析字体,返回字体解析类
     */
    fn query_base64_ttf(&self, base64: Option<&str>) -> Option<QueryTTF> {
        // base64DecodeToByteArray(base64)?.let {
        //     return QueryTTF(it)
        // }
        // return null
        if let Some(it) = self.base64_decode_to_byte_array(base64) {
            return Some(QueryTTF::new(it));
        }
        None
    }

    /**
     * 返回字体解析类
     * @param str 支持url,本地文件,base64,自动判断,自动缓存
     */
    fn query_ttf(&self, str: Option<&str>) -> Option<QueryTTF> {
        // str ?: return null
        let str = match str {
            Some(str) => str,
            None => return None,
        };
        // val key = md5Encode16(str)
        let key = self.md5_encode16(str);
        // val cacheManager = CacheManager(getUserNameSpace())
        let cache_manager = CacheManager::new(self.get_user_name_space());
        // var qTTF = cacheManager.getQueryTTF(key)
        let mut q_ttf = cache_manager.get_query_ttf(&key);
        // if (qTTF != null) return qTTF
        if q_ttf.is_some() {
            return q_ttf;
        }
        // val font: ByteArray? = when {
        //     str.isAbsUrl() -> runBlocking {
        //         return@runBlocking okHttpClient.newCall { url(str) }.bytes()
        //     }
        //     str.indexOf("storage/") > 0 -> File(str).readBytes()
        //     else -> base64DecodeToByteArray(str)
        // }
        let font: Option<Vec<u8>> = if str.is_abs_url() {
            Some(run_blocking(|| async {
                ok_http_client().new_call(0, |builder| url(builder, str)).await.bytes()
            }))
        } else if str.find("storage/").map(|i| i > 0).unwrap_or(false) {
            Some(File::new(str).read_bytes())
        } else {
            self.base64_decode_to_byte_array(Some(str))
        };
        // font ?: return null
        let font = match font {
            Some(font) => font,
            None => return None,
        };
        // qTTF = QueryTTF(font)
        q_ttf = Some(QueryTTF::new(font));
        // cacheManager.put(key, qTTF)
        cache_manager.put(&key, q_ttf.as_ref().unwrap(), 0);
        q_ttf
    }

    /**
     * @param text 包含错误字体的内容
     * @param font1 错误的字体
     * @param font2 正确的字体
     */
    fn replace_font(
        &self,
        text: &str,
        font1: Option<&QueryTTF>,
        font2: Option<&QueryTTF>,
    ) -> String {
        // if (font1 == null || font2 == null) return text
        let font1 = match font1 {
            Some(font1) => font1,
            None => return text.to_string(),
        };
        let font2 = match font2 {
            Some(font2) => font2,
            None => return text.to_string(),
        };
        // val contentArray = text.toCharArray()
        let mut content_array: Vec<char> = text.chars().collect();
        // contentArray.forEachIndexed { index, s ->
        //     val oldCode = s.code
        //     if (font1.inLimit(s)) {
        //         val code = font2.getCodeByGlyf(font1.getGlyfByCode(oldCode))
        //         if (code != 0) contentArray[index] = code.toChar()
        //     }
        // }
        for (index, s) in content_array.iter_mut().enumerate() {
            let old_code = *s as i32;
            if font1.in_limit(*s) {
                let code = font2.get_code_by_glyf(font1.get_glyf_by_code(old_code));
                if code != 0 {
                    content_array[index] = char::from_u32(code as u32).unwrap_or('\u{0}');
                }
            }
        }
        // return contentArray.joinToString("")
        content_array.into_iter().collect()
    }

    /**
     * 弹窗提示
     */
    fn toast(&self, msg: Option<&dyn std::any::Any>) {
        self.get_logger().map(|logger| logger.log(format!("toast: {:?}", msg.map(|it| it.type_id()))));
        Debug::log(format!("toast: {:?}", msg));
    }

    /**
     * 弹窗提示 停留时间较长
     */
    fn long_toast(&self, msg: Option<&dyn std::any::Any>) {
        self.get_logger().map(|logger| logger.log(format!("longToast: {:?}", msg)));
        Debug::log(format!("longToast: {:?}", msg));
    }

    /**
     * 输出调试日志
     */
    fn log(&self, msg: String) -> String {
        self.get_logger().map(|logger| logger.log(msg.clone()));
        Debug::log(&msg);
        msg
    }

    /**
     * 输出对象类型
     */
    fn log_type(&self, any: Option<&dyn std::any::Any>) {
        // if (any == null) {
        //     log("null")
        // } else {
        //     log(any.javaClass.name)
        // }
        match any {
            None => {
                self.log("null".to_string());
            }
            Some(any) => {
                self.log(any.type_name().to_string());
            }
        }
    }

    /**
     * 生成UUID
     */
    fn random_uuid(&self) -> String {
        // return UUID.randomUUID().toString()
        Uuid::random_uuid().to_string()
    }

    /**
     * AES 解码为 ByteArray
     * @param str 传入的AES加密的数据
     * @param key AES 解密的key
     * @param transformation AES加密的方式
     * @param iv ECB模式的偏移向量
     */
    fn aes_decode_to_byte_array(
        &self,
        str: &str, key: &str, transformation: &str, iv: &str,
    ) -> Option<Vec<u8>> {
        // return try {
        //     EncoderUtils.decryptAES(
        //         data = str.encodeToByteArray(),
        //         key = key.encodeToByteArray(),
        //         transformation,
        //         iv.encodeToByteArray()
        //     )
        // } catch (e: Exception) {
        //     e.printOnDebug()
        //     log(e.localizedMessage ?: "aesDecodeToByteArrayERROR")
        //     null
        // }
        match EncoderUtils::decrypt_aes(
            data = str.to_byte_array(),
            key = key.to_byte_array(),
            transformation,
            iv.to_byte_array(),
        ) {
            Ok(value) => Some(value),
            Err(e) => {
                e.print_on_debug();
                self.log(e.localized_message.unwrap_or("aesDecodeToByteArrayERROR".to_string()));
                None
            }
        }
    }

    /**
     * AES 解码为 String
     * @param str 传入的AES加密的数据
     * @param key AES 解密的key
     * @param transformation AES加密的方式
     * @param iv ECB模式的偏移向量
     */

    fn aes_decode_to_string(
        &self,
        str: &str, key: &str, transformation: &str, iv: &str,
    ) -> Option<String> {
        // return aesDecodeToByteArray(str, key, transformation, iv)?.let { String(it, Charsets.UTF_8) }
        self.aes_decode_to_byte_array(str, key, transformation, iv)
            .map(|it| String::from_utf8_lossy(&it).into_owned())
    }

    /**
     * 已经base64的AES 解码为 ByteArray
     * @param str 传入的AES Base64加密的数据
     * @param key AES 解密的key
     * @param transformation AES加密的方式
     * @param iv ECB模式的偏移向量
     */

    fn aes_base64_decode_to_byte_array(
        &self,
        str: &str, key: &str, transformation: &str, iv: &str,
    ) -> Option<Vec<u8>> {
        // return try {
        //     EncoderUtils.decryptBase64AES(
        //         str.encodeToByteArray(),
        //         key.encodeToByteArray(),
        //         transformation,
        //         iv.encodeToByteArray()
        //     )
        // } catch (e: Exception) {
        //     e.printOnDebug()
        //     log(e.localizedMessage ?: "aesDecodeToByteArrayERROR")
        //     null
        // }
        match EncoderUtils::decrypt_base64_aes(
            str.to_byte_array(),
            key.to_byte_array(),
            transformation,
            iv.to_byte_array(),
        ) {
            Ok(value) => Some(value),
            Err(e) => {
                e.print_on_debug();
                self.log(e.localized_message.unwrap_or("aesDecodeToByteArrayERROR".to_string()));
                None
            }
        }
    }

    /**
     * 已经base64的AES 解码为 String
     * @param str 传入的AES Base64加密的数据
     * @param key AES 解密的key
     * @param transformation AES加密的方式
     * @param iv ECB模式的偏移向量
     */

    fn aes_base64_decode_to_string(
        &self,
        str: &str, key: &str, transformation: &str, iv: &str,
    ) -> Option<String> {
        // return aesBase64DecodeToByteArray(str, key, transformation, iv)?.let { String(it, Charsets.UTF_8) }
        self.aes_base64_decode_to_byte_array(str, key, transformation, iv)
            .map(|it| String::from_utf8_lossy(&it).into_owned())
    }

    /**
     * 加密aes为ByteArray
     * @param data 传入的原始数据
     * @param key AES加密的key
     * @param transformation AES加密的方式
     * @param iv ECB模式的偏移向量
     */
    fn aes_encode_to_byte_array(
        &self,
        data: &str, key: &str, transformation: &str, iv: &str,
    ) -> Option<Vec<u8>> {
        // return try {
        //     EncoderUtils.encryptAES(
        //         data.encodeToByteArray(),
        //         key = key.encodeToByteArray(),
        //         transformation,
        //         iv.encodeToByteArray()
        //     )
        // } catch (e: Exception) {
        //     e.printOnDebug()
        //     log(e.localizedMessage ?: "aesEncodeToByteArrayERROR")
        //     null
        // }
        match EncoderUtils::encrypt_aes(
            data.to_byte_array(),
            key = key.to_byte_array(),
            transformation,
            iv.to_byte_array(),
        ) {
            Ok(value) => Some(value),
            Err(e) => {
                e.print_on_debug();
                self.log(e.localized_message.unwrap_or("aesEncodeToByteArrayERROR".to_string()));
                None
            }
        }
    }

    /**
     * 加密aes为String
     * @param data 传入的原始数据
     * @param key AES加密的key
     * @param transformation AES加密的方式
     * @param iv ECB模式的偏移向量
     */
    fn aes_encode_to_string(
        &self,
        data: &str, key: &str, transformation: &str, iv: &str,
    ) -> Option<String> {
        // return aesEncodeToByteArray(data, key, transformation, iv)?.let { String(it, Charsets.UTF_8) }
        self.aes_encode_to_byte_array(data, key, transformation, iv)
            .map(|it| String::from_utf8_lossy(&it).into_owned())
    }

    /**
     * 加密aes后Base64化的ByteArray
     * @param data 传入的原始数据
     * @param key AES加密的key
     * @param transformation AES加密的方式
     * @param iv ECB模式的偏移向量
     */
    fn aes_encode_to_base64_byte_array(
        &self,
        data: &str, key: &str, transformation: &str, iv: &str,
    ) -> Option<Vec<u8>> {
        // return try {
        //     EncoderUtils.encryptAES2Base64(
        //         data.encodeToByteArray(),
        //         key.encodeToByteArray(),
        //         transformation,
        //         iv.encodeToByteArray()
        //     )
        // } catch (e: Exception) {
        //     e.printOnDebug()
        //     log(e.localizedMessage ?: "aesEncodeToBase64ByteArrayERROR")
        //     null
        // }
        match EncoderUtils::encrypt_aes2_base64(
            data.to_byte_array(),
            key.to_byte_array(),
            transformation,
            iv.to_byte_array(),
        ) {
            Ok(value) => Some(value),
            Err(e) => {
                e.print_on_debug();
                self.log(e.localized_message.unwrap_or("aesEncodeToBase64ByteArrayERROR".to_string()));
                None
            }
        }
    }

    /**
     * 加密aes后Base64化的String
     * @param data 传入的原始数据
     * @param key AES加密的key
     * @param transformation AES加密的方式
     * @param iv ECB模式的偏移向量
     */
    fn aes_encode_to_base64_string(
        &self,
        data: &str, key: &str, transformation: &str, iv: &str,
    ) -> Option<String> {
        // return aesEncodeToBase64ByteArray(data, key, transformation, iv)?.let { String(it, Charsets.UTF_8) }
        self.aes_encode_to_base64_byte_array(data, key, transformation, iv)
            .map(|it| String::from_utf8_lossy(&it).into_owned())
    }

    fn android_id(&self) -> String {
        // return ""
        "".to_string()
    }

    /**
     * AES解密，算法参数经过Base64加密
     *
     * @param data 加密的字符串
     * @param key Base64后的密钥
     * @param mode 模式
     * @param padding 补码方式
     * @param iv Base64后的加盐
     * @return 解密后的字符串
     */
    fn aes_decode_args_base64_str(
        &self,
        data: &str,
        key: &str,
        mode: &str,
        padding: &str,
        iv: &str,
    ) -> Option<String> {
        // return AES(
        //     mode,
        //     padding,
        //     Base64.decode(key, Base64.NO_WRAP),
        //     Base64.decode(iv, Base64.NO_WRAP)
        // ).decryptStr(data)
        AES::new(
            mode,
            padding,
            Base64::decode(key, Base64::NO_WRAP),
            Base64::decode(iv, Base64::NO_WRAP),
        ).decrypt_str(data)
    }

    /**
     * 3DES解密
     *
     * @param data 加密的字符串
     * @param key 密钥
     * @param mode 模式
     * @param padding 补码方式
     * @param iv 加盐
     * @return 解密后的字符串
     */
    fn triple_des_decode_str(
        &self,
        data: &str,
        key: &str,
        mode: &str,
        padding: &str,
        iv: &str,
    ) -> Option<String> {
        // return DESede(mode, padding, key.toByteArray(Charsets.UTF_8), iv.toByteArray(Charsets.UTF_8)).decryptStr(data)
        DESede::new(mode, padding, key.to_byte_array(), iv.to_byte_array()).decrypt_str(data)
    }

    /**
     * 3DES解密，算法参数经过Base64加密
     *
     * @param data 加密的字符串
     * @param key Base64后的密钥
     * @param mode 模式
     * @param padding 补码方式
     * @param iv Base64后的加盐
     * @return 解密后的字符串
     */
    fn triple_des_decode_args_base64_str(
        &self,
        data: &str,
        key: &str,
        mode: &str,
        padding: &str,
        iv: &str,
    ) -> Option<String> {
        // return DESede(
        //     mode,
        //     padding,
        //     Base64.decode(key, Base64.NO_WRAP),
        //     Base64.decode(iv, Base64.NO_WRAP)
        // ).decryptStr(data)
        DESede::new(
            mode,
            padding,
            Base64::decode(key, Base64::NO_WRAP),
            Base64::decode(iv, Base64::NO_WRAP),
        ).decrypt_str(data)
    }

    /**
     * AES加密并转为Base64，算法参数经过Base64加密
     *
     * @param data 被加密的字符串
     * @param key Base64后的密钥
     * @param mode 模式
     * @param padding 补码方式
     * @param iv Base64后的加盐
     * @return 加密后的Base64
     */
    fn aes_encode_args_base64_str(
        &self,
        data: &str,
        key: &str,
        mode: &str,
        padding: &str,
        iv: &str,
    ) -> Option<String> {
        // return AES(
        //     mode,
        //     padding,
        //     Base64.decode(key, Base64.NO_WRAP),
        //     Base64.decode(iv, Base64.NO_WRAP)
        // ).encryptBase64(data)
        AES::new(
            mode,
            padding,
            Base64::decode(key, Base64::NO_WRAP),
            Base64::decode(iv, Base64::NO_WRAP),
        ).encrypt_base64(data)
    }
    /////DES
    fn des_decode_to_string(
        &self,
        data: &str, key: &str, transformation: &str, iv: &str,
    ) -> Option<String> {
        // return EncoderUtils.decryptDES(
        //     data.encodeToByteArray(),
        //     key.encodeToByteArray(),
        //     transformation,
        //     iv.encodeToByteArray()
        // )?.let { String(it, Charsets.UTF_8) }
        EncoderUtils::decrypt_des(
            data.to_byte_array(),
            key.to_byte_array(),
            transformation,
            iv.to_byte_array(),
        ).map(|it| String::from_utf8_lossy(&it).into_owned())
    }

    fn des_base64_decode_to_string(
        &self,
        data: &str, key: &str, transformation: &str, iv: &str,
    ) -> Option<String> {
        // return EncoderUtils.decryptBase64DES(
        //     data.encodeToByteArray(),
        //     key.encodeToByteArray(),
        //     transformation,
        //     iv.encodeToByteArray()
        // )?.let { String(it, Charsets.UTF_8) }
        EncoderUtils::decrypt_base64_des(
            data.to_byte_array(),
            key.to_byte_array(),
            transformation,
            iv.to_byte_array(),
        ).map(|it| String::from_utf8_lossy(&it).into_owned())
    }

    fn des_encode_to_string(
        &self,
        data: &str, key: &str, transformation: &str, iv: &str,
    ) -> Option<String> {
        // return EncoderUtils.encryptDES(
        //     data.encodeToByteArray(),
        //     key.encodeToByteArray(),
        //     transformation,
        //     iv.encodeToByteArray()
        // )?.let { String(it, Charsets.UTF_8) }
        EncoderUtils::encrypt_des(
            data.to_byte_array(),
            key.to_byte_array(),
            transformation,
            iv.to_byte_array(),
        ).map(|it| String::from_utf8_lossy(&it).into_owned())
    }

    fn des_encode_to_base64_string(
        &self,
        data: &str, key: &str, transformation: &str, iv: &str,
    ) -> Option<String> {
        // return EncoderUtils.encryptDES2Base64(
        //     data.encodeToByteArray(),
        //     key.encodeToByteArray(),
        //     transformation,
        //     iv.encodeToByteArray()
        // )?.let { String(it, Charsets.UTF_8) }
        EncoderUtils::encrypt_des2_base64(
            data.to_byte_array(),
            key.to_byte_array(),
            transformation,
            iv.to_byte_array(),
        ).map(|it| String::from_utf8_lossy(&it).into_owned())
    }
    /**
     * 3DES加密并转为Base64
     *
     * @param data 被加密的字符串
     * @param key 密钥
     * @param mode 模式
     * @param padding 补码方式
     * @param iv 加盐
     * @return 加密后的Base64
     */
    fn triple_des_encode_base64_str(
        &self,
        data: &str,
        key: &str,
        mode: &str,
        padding: &str,
        iv: &str,
    ) -> Option<String> {
        // return DESede(mode, padding, key.toByteArray(Charsets.UTF_8), iv.toByteArray(Charsets.UTF_8)).encryptBase64(data)
        DESede::new(mode, padding, key.to_byte_array(), iv.to_byte_array()).encrypt_base64(data)
    }

    /**
     * 3DES加密并转为Base64，算法参数经过Base64加密
     *
     * @param data 被加密的字符串
     * @param key Base64后的密钥
     * @param mode 模式
     * @param padding 补码方式
     * @param iv Base64后的加盐
     * @return 加密后的Base64
     */
    fn triple_des_encode_args_base64_str(
        &self,
        data: &str,
        key: &str,
        mode: &str,
        padding: &str,
        iv: &str,
    ) -> Option<String> {
        // return DESede(
        //     mode,
        //     padding,
        //     Base64.decode(key, Base64.NO_WRAP),
        //     Base64.decode(iv, Base64.NO_WRAP)
        // ).encryptBase64(data)
        DESede::new(
            mode,
            padding,
            Base64::decode(key, Base64::NO_WRAP),
            Base64::decode(iv, Base64::NO_WRAP),
        ).encrypt_base64(data)
    }

    /**
     * 生成摘要，并转为16进制字符串
     *
     * @param data 被摘要数据
     * @param algorithm 签名算法
     * @return 16进制字符串
     */
    fn digest_hex(
        &self,
        data: &str,
        algorithm: &str,
    ) -> Option<String> {
        // return DigestUtil.digester(algorithm).digestHex(data)
        DigestUtil::digester(algorithm).digest_hex(data)
    }

    /**
     * 生成摘要，并转为Base64字符串
     *
     * @param data 被摘要数据
     * @param algorithm 签名算法
     * @return Base64字符串
     */
    fn digest_base64_str(
        &self,
        data: &str,
        algorithm: &str,
    ) -> Option<String> {
        // return Base64.encodeToString(DigestUtil.digester(algorithm).digest(data), Base64.NO_WRAP)
        Some(Base64::encode_to_string(DigestUtil::digester(algorithm).digest(data), Base64::NO_WRAP))
    }

}
