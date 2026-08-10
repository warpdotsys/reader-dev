// package io.legado.app.help.http.api

/**
 * interface CookieManager {
 */
pub trait CookieManager {
    /**
     * 保存cookie
     */
    fn set_cookie(&self, url: &str, cookie: Option<&str>);

    /**
     * 替换cookie
     */
    fn replace_cookie(&self, url: &str, cookie: &str);

    /**
     * 获取cookie
     */
    fn get_cookie(&self, url: &str) -> String;

    /**
     * 移除cookie
     */
    fn remove_cookie(&self, url: &str);

    fn cookie_to_map(&self, cookie: &str) -> std::collections::HashMap<String, String>;

    fn map_to_cookie(&self, cookie_map: Option<&std::collections::HashMap<String, String>>) -> Option<String>;
}
