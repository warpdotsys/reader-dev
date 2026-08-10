// package com.htmake.reader.lib.tts.util;

// import okhttp3.OkHttpClient;
// import okhttp3.Request;
// import okhttp3.Response;
// import org.slf4j.Logger;
// import org.slf4j.LoggerFactory;

// import java.text.SimpleDateFormat;
// import java.time.LocalDateTime;
// import java.time.format.DateTimeFormatter;
// import java.util.Date;
// import java.util.Locale;
// import java.util.UUID;
// import java.util.regex.Pattern;

// public class Tools {
// private static OkHttpClient client = new OkHttpClient();
static CLIENT: std::sync::OnceLock<OkHttpClient> = std::sync::OnceLock::new();

pub struct Tools;

impl Tools {
    // public static final Pattern NO_VOICE_PATTERN = Pattern.compile("[\\s\\p{C}\\p{P}\\p{Z}\\p{S}]");
    pub const NO_VOICE_PATTERN: &'static str = r"[\\s\\p{C}\\p{P}\\p{Z}\\p{S}]";

    // public static final String SDF = "EEE MMM dd yyyy HH:mm:ss 'GMT'Z";
    pub const SDF: &'static str = "EEE MMM dd yyyy HH:mm:ss 'GMT'Z";

    // public static final DateTimeFormatter DTF = DateTimeFormatter.ofPattern("yyyyMMdd_HHmmss");
    pub const DTF: &'static str = "yyyyMMdd_HHmmss";

    // public static Logger log = LoggerFactory.getLogger(Tools.class);

    // private static OkHttpClient client = new OkHttpClient();
    static CLIENT: std::sync::OnceLock<OkHttpClient> = std::sync::OnceLock::new();

    fn client() -> &'static OkHttpClient {
        CLIENT.get_or_init(|| OkHttpClient::new())
    }

    // public Tools() {
    // }
    pub fn new() -> Tools {
        Tools
    }

    // public static String httpGet(String url) {
    pub fn http_get(url: &str) -> String {
        try {
            let request = Request::builder().url(url).build();
            let response = Self::client().new_call(request).execute();
            log::info("response.toString():{:?}", response.to_string());
            log::info("response.isSuccessful():{:?}", response.is_successful());
            if response.is_successful() {
                let body = response.body().string();
                return body;
            }
            panic!(format!("request：{} fail, message:{}", url, response.code()));
        } catch (e) {
            panic!(e);
        }
    }

    // public static boolean isNoVoice(CharSequence text) {
    pub fn is_no_voice(text: &str) -> bool {
        return regex_replace_all(Self::NO_VOICE_PATTERN, text, "").is_empty();
    }

    // public static void sleep(int seconds) {
    pub fn sleep(seconds: i32) {
        try {
            thread::sleep(Duration::from_secs(seconds as u64));
        } catch (e: InterruptedException) {
            // ignored
        }
    }

    // public static String date() {
    pub fn date() -> String {
        return SimpleDateFormat(Self::SDF).format(Date::now());
    }

    // public static String localDateTime() {
    pub fn local_date_time() -> String {
        return LocalDateTime::now().format(Self::DTF);
    }

    // public static String localeToEmoji(Locale locale) {
    pub fn locale_to_emoji(locale: &Locale) -> String {
        let country = locale.get_country();
        if "TW" == country && "CN" == Locale::get_default().get_country() {
            return String::new();
        }
        let first_code_point = country.chars().next().unwrap() as i32 - 65 + 127462;
        let second_code_point = country.chars().nth(1).unwrap() as i32 - 65 + 127462;
        return char::from_u32(first_code_point as u32).unwrap().to_string() + &char::from_u32(second_code_point as u32).unwrap().to_string();
    }

    // public static String getRandomId() {
    pub fn get_random_id() -> String {
        return Uuid::new_v4().to_string().replace("-", "");
    }
}
