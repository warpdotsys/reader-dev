// 真实 JS 规则引擎（boa_engine 封装）
// 供 stubs::ScriptEngine::eval 调用：SimpleBindings → JS 环境 → Any 结果

use boa_engine::native_function::NativeFunction;
use boa_engine::property::Attribute;
use boa_engine::js_string;
use boa_engine::{Context, JsValue, Source};
use serde_json::Value;

use crate::stubs::{Any, JsonArray, JsonObject, SimpleBindings};
use crate::io_legado_app_help_http_api_cookiemanager::CookieManager;

/// serde_json::Value → Any
pub fn value_to_any(v: &Value) -> Any {
    match v {
        Value::Null => Any::Null,
        Value::Bool(b) => Any::Bool(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Any::Long(i)
            } else if let Some(f) = n.as_f64() {
                Any::Double(f)
            } else {
                Any::Null
            }
        }
        Value::String(s) => Any::Str(s.clone()),
        Value::Array(arr) => Any::List(arr.iter().map(value_to_any).collect()),
        Value::Object(obj) => {
            let mut map = std::collections::HashMap::new();
            for (k, v) in obj {
                map.insert(k.clone(), value_to_any(v));
            }
            Any::Map(map)
        }
    }
}

/// 从 SimpleBindings 提取绑定值 → JS 全局变量
fn bind_value(bindings: &SimpleBindings, key: &str, ctx: &mut Context) -> JsValue {
    let Some(v) = bindings.map.get(key) else {
        return JsValue::null();
    };
    let any_ref = v.as_ref().as_any();
    if let Some(a) = any_ref.downcast_ref::<Any>() {
        return any_to_js_value(a, ctx);
    }
    if let Some(s) = any_ref.downcast_ref::<String>() {
        return JsValue::from_json(&Value::String(s.clone()), ctx).unwrap_or(JsValue::null());
    }
    if let Some(s) = any_ref.downcast_ref::<Option<String>>() {
        return match s {
            Some(v) => JsValue::from_json(&Value::String(v.clone()), ctx).unwrap_or(JsValue::null()),
            None => JsValue::null(),
        };
    }
    if let Some(b) = any_ref.downcast_ref::<bool>() {
        return JsValue::from_json(&Value::Bool(*b), ctx).unwrap_or(JsValue::null());
    }
    if let Some(i) = any_ref.downcast_ref::<i32>() {
        return JsValue::from_json(&Value::Number((*i).into()), ctx).unwrap_or(JsValue::null());
    }
    if let Some(s) = any_ref.downcast_ref::<Option<i32>>() {
        return match s {
            Some(v) => JsValue::from_json(&Value::Number((*v).into()), ctx).unwrap_or(JsValue::null()),
            None => JsValue::null(),
        };
    }
    if let Some(i) = any_ref.downcast_ref::<i64>() {
        return JsValue::from_json(&Value::Number((*i).into()), ctx).unwrap_or(JsValue::null());
    }
    if let Some(f) = any_ref.downcast_ref::<f64>() {
        return JsValue::from_json(
            &Value::Number(serde_json::Number::from_f64(*f).unwrap_or(serde_json::Number::from_f64(0.0).unwrap())),
            ctx,
        )
        .unwrap_or(JsValue::null());
    }
    // fix: CookieStore/CacheManager 实例 → 带方法 JS 对象（cookie.getCookie()/setCookie()/replaceCookie()、cache.get()/put()）
    if let Some(cs) = any_ref.downcast_ref::<crate::io_legado_app_help_http_cookiestore::CookieStore>() {
        set_current_js_ns(cs.user_name_space.clone());
        return cookie_store_js_object(&cs.user_name_space, ctx);
    }
    if let Some(cm) = any_ref.downcast_ref::<crate::io_legado_app_help_cachemanager::CacheManager>() {
        set_current_js_ns(cm.user_name_space.clone());
        return cache_manager_js_object(&cm.user_name_space, ctx);
    }
    JsValue::null()
}

thread_local! {
    static CURRENT_JS_NS: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
    static CURRENT_JS_VARS: std::cell::RefCell<std::collections::HashMap<String, String>> = std::cell::RefCell::new(std::collections::HashMap::new());
}

pub fn set_current_js_vars(vars: std::collections::HashMap<String, String>) {
    CURRENT_JS_VARS.with(|c| *c.borrow_mut() = vars);
}

pub fn get_current_js_vars() -> std::collections::HashMap<String, String> {
    CURRENT_JS_VARS.with(|c| c.borrow().clone())
}

pub fn put_current_js_var(k: String, v: String) {
    CURRENT_JS_VARS.with(|c| { c.borrow_mut().insert(k, v); });
}

pub fn get_current_js_var(k: &str) -> Option<String> {
    CURRENT_JS_VARS.with(|c| c.borrow().get(k).cloned())
}

fn set_current_js_ns(ns: String) {
    CURRENT_JS_NS.with(|c| *c.borrow_mut() = Some(ns));
}

fn current_js_ns() -> String {
    CURRENT_JS_NS.with(|c| c.borrow().clone()).unwrap_or_default()
}

/// 响应 Set-Cookie 写回 cookie jar（Kotlin JsExtensions java.get/head/post 自动保存登录 cookie）
fn save_set_cookie_to_jar(url: &str, resp_headers: &std::collections::HashMap<String, String>) {
    if let Some(set_cookie) = resp_headers.get("set-cookie") {
        let ns = current_js_ns();
        if !ns.is_empty() {
            crate::io_legado_app_help_http_cookiestore::CookieStore::new(ns).set_cookie(url, Some(set_cookie));
        }
    }
}

/// cookie 对象（Kotlin CookieStore 绑定；方法经当前命名空间构造实例）
fn cookie_store_js_object(_ns: &str, ctx: &mut Context) -> JsValue {
    boa_engine::object::ObjectInitializer::new(ctx)
        .function(NativeFunction::from_fn_ptr(cookie_get_native), js_string!("getCookie"), 2)
        .function(NativeFunction::from_fn_ptr(cookie_get_native), js_string!("getKey"), 2)
        .function(NativeFunction::from_fn_ptr(cookie_set_native), js_string!("setCookie"), 2)
        .function(NativeFunction::from_fn_ptr(cookie_replace_native), js_string!("replaceCookie"), 2)
        .build()
        .into()
}

fn cookie_get_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    use crate::io_legado_app_help_http_api_cookiemanager::CookieManager;
    let url = arg_string(args, 0, ctx);
    let key = arg_string(args, 1, ctx);
    let cs = crate::io_legado_app_help_http_cookiestore::CookieStore::new(current_js_ns());
    let cookie = cs.get_cookie(&url);
    let val = if key.is_empty() { cookie } else { cs.get_key(&url, &key) };
    Ok(JsValue::from_json(&Value::String(val), ctx).unwrap_or(JsValue::null()))
}

fn cookie_set_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    use crate::io_legado_app_help_http_api_cookiemanager::CookieManager;
    let url = arg_string(args, 0, ctx);
    let cookie = arg_string(args, 1, ctx);
    crate::io_legado_app_help_http_cookiestore::CookieStore::new(current_js_ns()).set_cookie(&url, Some(&cookie));
    Ok(JsValue::null())
}

fn cookie_replace_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    use crate::io_legado_app_help_http_api_cookiemanager::CookieManager;
    let url = arg_string(args, 0, ctx);
    let cookie = arg_string(args, 1, ctx);
    crate::io_legado_app_help_http_cookiestore::CookieStore::new(current_js_ns()).replace_cookie(&url, &cookie);
    Ok(JsValue::null())
}

/// cache 对象（Kotlin CacheManager 绑定；get/put）
fn cache_manager_js_object(_ns: &str, ctx: &mut Context) -> JsValue {
    boa_engine::object::ObjectInitializer::new(ctx)
        .function(NativeFunction::from_fn_ptr(cache_get_native), js_string!("get"), 1)
        .function(NativeFunction::from_fn_ptr(cache_put_native), js_string!("put"), 2)
        .build()
        .into()
}

fn cache_get_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let key = arg_string(args, 0, ctx);
    let cm = crate::io_legado_app_help_cachemanager::CacheManager::new(current_js_ns());
    let val = cm.get(&key).unwrap_or_default();
    Ok(JsValue::from_json(&Value::String(val), ctx).unwrap_or(JsValue::null()))
}

fn cache_put_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let key = arg_string(args, 0, ctx);
    let value = arg_string(args, 1, ctx);
    let cm = crate::io_legado_app_help_cachemanager::CacheManager::new(current_js_ns());
    cm.put(&key, &value, 0);
    Ok(JsValue::null())
}

/// Any → JsValue
pub fn any_to_js_value(a: &Any, ctx: &mut Context) -> JsValue {
    // JSON 字符串（对象/数组形态）解析为 JS 对象（source/book/chapter 绑定）
    if let Any::Str(s) = a {
        let trimmed = s.trim_start();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            if let Ok(v) = serde_json::from_str::<Value>(s) {
                if let Ok(jv) = JsValue::from_json(&v, ctx) {
                    return jv;
                }
            }
        }
    }
    let json = crate::stubs::any_to_value(a);
    JsValue::from_json(&json, ctx).unwrap_or(JsValue::null())
}

/// 全局 ajax(url)：blocking 请求，返回 HTML 文本（JS 规则跨域访问）
fn ajax_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let url = args
        .first()
        .map(|a| a.to_string(ctx).unwrap_or_default())
        .unwrap_or_default();
    let text = if url.is_empty() {
        String::new()
    } else {
        // fix: 真实请求 + 默认 UA + 书源自定义 header（原仅默认 UA——防盗链/签名站点被拒）
        let mut headers = std::collections::HashMap::new();
        headers.insert(
            String::from("User-Agent"),
            crate::io_legado_app_constant_appconst::AppConst::userAgent(),
        );
        // 从全局 source 绑定读取书源 header JSON（{"key":"value"}）
        let source_js = "(() => { try { return typeof source === 'object' && source !== null ? source : null; } catch(e) { return null; } })()";
        if let Ok(v) = ctx.eval(boa_engine::Source::from_bytes(source_js.as_bytes())) {
            if let Ok(Some(json)) = v.to_json(ctx) {
                if let Some(obj) = json.as_object() {
                    if let Some(h) = obj.get("header").and_then(|x| x.as_str()) {
                        if let Ok(hv) = serde_json::from_str::<serde_json::Value>(h) {
                            if let Some(ho) = hv.as_object() {
                                for (k, val) in ho {
                                    if let Some(s) = val.as_str() {
                                        headers.insert(k.clone(), s.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        // fix: 携带书源 cookie jar（原从绑定字符串读——绑定改对象后 String(cookie) 为 [object Object]；
        //      改为从当前命名空间的 cookie jar 读取）
        let ns = current_js_ns();
        if !ns.is_empty() {
            let cs = crate::io_legado_app_help_http_cookiestore::CookieStore::new(ns);
            let url_str = url.to_std_string().unwrap_or_default();
            let cookie_val = cs.get_cookie(&url_str);
            if !cookie_val.is_empty() && !headers.contains_key("Cookie") {
                headers.insert(String::from("Cookie"), cookie_val);
            }
        }
        let (_, resp_headers, body) = js_http_request("GET", &url.to_std_string().unwrap_or_default(), &headers, None);
        // fix: Set-Cookie 自动写回 cookie jar（Kotlin JsExtensions java.get/head/post 语义——登录流程依赖）
        if let Some(set_cookie) = resp_headers.get("set-cookie") {
            let ns = current_js_ns();
            if !ns.is_empty() {
                crate::io_legado_app_help_http_cookiestore::CookieStore::new(ns).set_cookie(&url.to_std_string().unwrap_or_default(), Some(set_cookie));
            }
        }
        body
    };
    Ok(JsValue::from_json(&Value::String(text), ctx).unwrap_or(JsValue::null()))
}

/// 全局 getUrl / get(url)：同 ajax
fn get_url_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    ajax_native(_this, args, ctx)
}

/// java.ajaxAll(urls) → 并发请求全部（返回字符串数组；原 Kotlin 为并发，缺失时书源只能串行）
fn ajax_all_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let mut urls: Vec<String> = Vec::new();
    if let Some(arg) = args.first() {
        if let Ok(Some(json)) = arg.to_json(ctx) {
            if let Some(arr) = json.as_array() {
                for v in arr {
                    urls.push(v.as_str().map(|s| s.to_string()).unwrap_or_default());
                }
            }
        }
    }
    // 书源自定义 header（与 ajax 一致）
    let mut base_headers = std::collections::HashMap::new();
    base_headers.insert(
        String::from("User-Agent"),
        crate::io_legado_app_constant_appconst::AppConst::userAgent(),
    );
    let source_js = "(() => { try { return typeof source === 'object' && source !== null ? source : null; } catch(e) { return null; } })()";
    if let Ok(v) = ctx.eval(boa_engine::Source::from_bytes(source_js.as_bytes())) {
        if let Ok(Some(json)) = v.to_json(ctx) {
            if let Some(obj) = json.as_object() {
                if let Some(h) = obj.get("header").and_then(|x| x.as_str()) {
                    if let Ok(hv) = serde_json::from_str::<serde_json::Value>(h) {
                        if let Some(ho) = hv.as_object() {
                            for (k, val) in ho {
                                if let Some(s) = val.as_str() {
                                    base_headers.insert(k.clone(), s.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // 分批并发（每批 32；原 take(32) 静默丢弃超出部分，返回数组与输入不对齐）
    let mut results: Vec<serde_json::Value> = Vec::new();
    let mut idx = 0usize;
    while idx < urls.len() {
        let batch = urls.len().min(idx + 32);
        let mut handles = Vec::new();
        for url in &urls[idx..batch] {
            let url = url.clone();
            let headers = base_headers.clone();
            handles.push(std::thread::spawn(move || {
                let (_, _, body) = js_http_request("GET", &url, &headers, None);
                body
            }));
        }
        for h in handles {
            results.push(serde_json::Value::String(h.join().unwrap_or_default()));
        }
        idx = batch;
    }
    Ok(JsValue::from_json(&Value::Array(results), ctx).unwrap_or(JsValue::null()))
}

fn arg_string(args: &[JsValue], idx: usize, ctx: &mut Context) -> String {
    args.get(idx)
        .map(|a| a.to_string(ctx).unwrap_or_default().to_std_string().unwrap_or_default())
        .unwrap_or_default()
}

fn java_base64_encode_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let s = arg_string(args, 0, ctx);
    let out = crate::io_legado_app_utils_base64::Base64::encodeToString(s.as_bytes(), crate::io_legado_app_utils_base64::Base64::NO_WRAP);
    Ok(JsValue::from_json(&Value::String(out), ctx).unwrap_or(JsValue::null()))
}

fn java_base64_decode_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let s = arg_string(args, 0, ctx);
    let bytes = crate::io_legado_app_utils_base64::Base64::decode_str(&s, crate::io_legado_app_utils_base64::Base64::NO_WRAP);
    let out = String::from_utf8_lossy(&bytes).to_string();
    Ok(JsValue::from_json(&Value::String(out), ctx).unwrap_or(JsValue::null()))
}

fn java_md5_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let s = arg_string(args, 0, ctx);
    let digest = crate::stubs::md5_bytes(s.as_bytes());
    let hex: String = digest.iter().map(|b| format!("{:02x}", b)).collect();
    let out = hex[8..24].to_string();
    Ok(JsValue::from_json(&Value::String(out), ctx).unwrap_or(JsValue::null()))
}

fn java_md5_full_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let s = arg_string(args, 0, ctx);
    let digest = crate::stubs::md5_bytes(s.as_bytes());
    let out: String = digest.iter().map(|b| format!("{:02x}", b)).collect();
    Ok(JsValue::from_json(&Value::String(out), ctx).unwrap_or(JsValue::null()))
}

fn java_aes_helper(args: &[JsValue], ctx: &mut Context, encrypt: bool) -> String {
    let data = arg_string(args, 0, ctx);
    let key = arg_string(args, 1, ctx);
    let transformation = arg_string(args, 2, ctx);
    let iv = arg_string(args, 3, ctx);
    let mut cipher = crate::stubs::Cipher::getInstance(&transformation);
    let key_spec = crate::stubs::SecretKeySpec::new(key.as_bytes(), "AES");
    let iv_spec = crate::stubs::IvParameterSpec::new(iv.as_bytes());
    cipher.init_spec_iv(
        if encrypt { crate::stubs::Cipher::ENCRYPT_MODE } else { crate::stubs::Cipher::DECRYPT_MODE },
        &key_spec,
        &iv_spec,
    );
    let out = cipher.do_final_data(data.as_bytes());
    String::from_utf8_lossy(&out).to_string()
}

fn java_aes_encode_to_string_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let out = java_aes_helper(args, ctx, true);
    Ok(JsValue::from_json(&Value::String(out), ctx).unwrap_or(JsValue::null()))
}

fn java_aes_decode_to_string_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let out = java_aes_helper(args, ctx, false);
    Ok(JsValue::from_json(&Value::String(out), ctx).unwrap_or(JsValue::null()))
}

fn java_aes_base64_decode_to_string_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    // 直接对 base64 解码字节解密（避免 lossy 字符串破坏密文）
    let b64 = arg_string(args, 0, ctx);
    let key = arg_string(args, 1, ctx);
    let transformation = arg_string(args, 2, ctx);
    let iv = arg_string(args, 3, ctx);
    let bytes = crate::io_legado_app_utils_base64::Base64::decode_str(&b64, crate::io_legado_app_utils_base64::Base64::NO_WRAP);
    let mut cipher = crate::stubs::Cipher::getInstance(&transformation);
    let key_spec = crate::stubs::SecretKeySpec::new(key.as_bytes(), "AES");
    let iv_spec = crate::stubs::IvParameterSpec::new(iv.as_bytes());
    cipher.init_spec_iv(crate::stubs::Cipher::DECRYPT_MODE, &key_spec, &iv_spec);
    let out = cipher.do_final_data(&bytes);
    let text = String::from_utf8_lossy(&out).to_string();
    Ok(JsValue::from_json(&Value::String(text), ctx).unwrap_or(JsValue::null()))
}

fn java_aes_encode_to_base64_string_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    // 直接字节加密后 base64（避免 lossy 字符串破坏密文）
    let data = arg_string(args, 0, ctx).into_bytes();
    let key = arg_string(args, 1, ctx).into_bytes();
    let transformation = arg_string(args, 2, ctx);
    let iv = arg_string(args, 3, ctx).into_bytes();
    let mut cipher = crate::stubs::Cipher::getInstance(&transformation);
    let key_spec = crate::stubs::SecretKeySpec::new(&key, "AES");
    let iv_spec = crate::stubs::IvParameterSpec::new(&iv);
    cipher.init_spec_iv(crate::stubs::Cipher::ENCRYPT_MODE, &key_spec, &iv_spec);
    let ct = cipher.do_final_data(&data);
    let out = crate::io_legado_app_utils_base64::Base64::encodeToString(&ct, crate::io_legado_app_utils_base64::Base64::NO_WRAP);
    Ok(JsValue::from_json(&Value::String(out), ctx).unwrap_or(JsValue::null()))
}

fn java_time_format_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let time_ms: i64 = args
        .first()
        .map(|a| a.to_string(ctx).unwrap_or_default().to_std_string().unwrap_or_default().parse().unwrap_or(0))
        .unwrap_or(0);
    let format = arg_string(args, 1, ctx);
    let sh: i32 = args.get(2).map(|a| a.to_string(ctx).unwrap_or_default().to_std_string().unwrap_or_default().parse().unwrap_or(0)).unwrap_or(0);
    use chrono::TimeZone;
    let out = match chrono::FixedOffset::east_opt(sh * 3600) {
        Some(tz) => match tz.timestamp_millis_opt(time_ms).single() {
            Some(dt) => dt.format(&crate::stubs::java_pattern_to_chrono(&format)).to_string(),
            None => String::new(),
        },
        None => String::new(),
    };
    Ok(JsValue::from_json(&Value::String(out), ctx).unwrap_or(JsValue::null()))
}

fn java_time_format_utc_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let time_ms: i64 = args
        .first()
        .map(|a| a.to_string(ctx).unwrap_or_default().to_std_string().unwrap_or_default().parse().unwrap_or(0))
        .unwrap_or(0);
    let format = arg_string(args, 1, ctx);
    use chrono::TimeZone;
    let out = match chrono::Utc.timestamp_millis_opt(time_ms).single() {
        Some(dt) => dt.format(&crate::stubs::java_pattern_to_chrono(&format)).to_string(),
        None => String::new(),
    };
    Ok(JsValue::from_json(&Value::String(out), ctx).unwrap_or(JsValue::null()))
}

fn java_log_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let s = arg_string(args, 0, ctx);
    eprintln!("[java.log] {}", s);
    Ok(JsValue::from_json(&Value::String(s), ctx).unwrap_or(JsValue::null()))
}

/// 解析 JS 对象参数（headers 等）为 HashMap
fn parse_js_map(args: &[JsValue], idx: usize, ctx: &mut Context) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    if let Some(arg) = args.get(idx) {
        if let Ok(Some(json)) = arg.to_json(ctx) {
            if let Some(obj) = json.as_object() {
                for (k, v) in obj {
                    map.insert(
                        k.clone(),
                        v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string()),
                    );
                }
            }
        }
    }
    map
}

/// 真实 HTTP 请求（java.get/post/head 共用；独立线程避免 async 上下文 blocking panic）
/// 返回 (status_code, headers, body)
pub fn js_http_request(
    method: &str,
    url: &str,
    headers: &std::collections::HashMap<String, String>,
    body: Option<&str>,
) -> (i32, std::collections::HashMap<String, String>, String) {
    let method = method.to_string();
    let url = url.to_string();
    let headers = headers.clone();
    let body = body.map(|s| s.to_string());
    std::thread::spawn(move || {
        let client = reqwest::blocking::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(15))
            .timeout(std::time::Duration::from_secs(30))
            .danger_accept_invalid_certs(true)
            .build()
            .ok()?;
        let mut builder = match method.as_str() {
            "POST" => client.post(&url),
            "HEAD" => client.head(&url),
            _ => client.get(&url),
        };
        for (k, v) in &headers {
            builder = builder.header(k, v);
        }
        if let Some(b) = body {
            builder = builder.body(b);
        }
        let resp = match builder.send() {
            Ok(r) => r,
            Err(e) => {
                let mut detail = String::new();
                let mut src: Option<&dyn std::error::Error> = Some(&e);
                while let Some(s) = src {
                    detail.push_str(&format!(" | {}", s));
                    src = s.source();
                }
                eprintln!("[js.http] {} {} error: {}", method, url, detail);
                return None;
            }
        };
        let status = resp.status().as_u16() as i32;
        let mut out_headers = std::collections::HashMap::new();
        for (k, v) in resp.headers() {
            if let Ok(v) = v.to_str() {
                out_headers.insert(k.as_str().to_string(), v.to_string());
            }
        }
        let text = resp.text().unwrap_or_default();
        Some((status, out_headers, text))
    })
    .join()
    .ok()
    .flatten()
    .unwrap_or((0, std::collections::HashMap::new(), String::new()))
}

fn headers_to_js(headers: &std::collections::HashMap<String, String>, ctx: &mut Context) -> JsValue {
    let mut props: Vec<(boa_engine::property::PropertyKey, JsValue)> = Vec::new();
    for (k, v) in headers {
        let val = JsValue::from_json(&Value::String(v.clone()), ctx).unwrap_or(JsValue::null());
        props.push((boa_engine::property::PropertyKey::from(boa_engine::JsString::from(k.as_str())), val));
    }
    let mut init = boa_engine::object::ObjectInitializer::new(ctx);
    for (k, v) in props {
        init.property(k, v, boa_engine::property::Attribute::all());
    }
    init.build().into()
}

/// java.put(key, value) → String (变量存储)
fn java_put_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let k = arg_string(args, 0, ctx);
    let v = arg_string(args, 1, ctx);
    put_current_js_var(k.clone(), v.clone());
    Ok(JsValue::from(js_string!(v)))
}

/// java.get(url, headers) → { body, statusCode, headers } 或 java.get(key) → String (变量读取)
fn java_get_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    if args.len() == 1 {
        let key = arg_string(args, 0, ctx);
        if !key.starts_with("http://") && !key.starts_with("https://") && !key.starts_with('/') {
            if let Some(v) = get_current_js_var(&key) {
                return Ok(JsValue::from(js_string!(v)));
            }
            return Ok(JsValue::null());
        }
    }
    let url = arg_string(args, 0, ctx);
    let headers = parse_js_map(args, 1, ctx);
    let (status, resp_headers, text) = js_http_request("GET", &url, &headers, None);
    save_set_cookie_to_jar(&url, &resp_headers);
    let headers_js = headers_to_js(&resp_headers, ctx);
    let body_val = JsValue::from_json(&Value::String(text), ctx).unwrap_or(JsValue::null());
    let obj = boa_engine::object::ObjectInitializer::new(ctx)
        .property(js_string!("body"), body_val, boa_engine::property::Attribute::all())
        .property(js_string!("statusCode"), status, boa_engine::property::Attribute::all())
        .property(js_string!("headers"), headers_js, boa_engine::property::Attribute::all())
        .build();
    Ok(obj.into())
}

/// java.head(url, headers) → { statusCode, headers }
fn java_head_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let url = arg_string(args, 0, ctx);
    let headers = parse_js_map(args, 1, ctx);
    // fix: 真实 HEAD 请求（原恒返回 statusCode 200，探测类规则全部失效）
    let (status, resp_headers, _) = js_http_request("HEAD", &url, &headers, None);
    save_set_cookie_to_jar(&url, &resp_headers);
    let headers_js = headers_to_js(&resp_headers, ctx);
    let obj = boa_engine::object::ObjectInitializer::new(ctx)
        .property(js_string!("statusCode"), status, boa_engine::property::Attribute::all())
        .property(js_string!("headers"), headers_js, boa_engine::property::Attribute::all())
        .build();
    Ok(obj.into())
}

/// java.post(url, body, headers) → { body, statusCode, headers }
fn java_post_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let url = arg_string(args, 0, ctx);
    let body = arg_string(args, 1, ctx);
    let headers = parse_js_map(args, 2, ctx);
    // fix: 真实请求（原 statusCode 恒 200）
    let (status, resp_headers, text) = js_http_request("POST", &url, &headers, Some(&body));
    save_set_cookie_to_jar(&url, &resp_headers);
    let headers_js = headers_to_js(&resp_headers, ctx);
    let body_val = JsValue::from_json(&Value::String(text), ctx).unwrap_or(JsValue::null());
    let obj = boa_engine::object::ObjectInitializer::new(ctx)
        .property(js_string!("body"), body_val, boa_engine::property::Attribute::all())
        .property(js_string!("statusCode"), status, boa_engine::property::Attribute::all())
        .property(js_string!("headers"), headers_js, boa_engine::property::Attribute::all())
        .build();
    Ok(obj.into())
}

/// java.cacheFile(url) → 下载到 storage/cache 返回本地路径
fn java_cache_file_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let url = arg_string(args, 0, ctx);
    let bytes = crate::stubs::WebClient::new().get_abs(&url).timeout(30000).async_get_bytes_in_thread();
    let path = if let Some(bytes) = bytes {
        if bytes.is_empty() {
            String::new()
        } else {
            let dir = std::path::Path::new("storage").join("cache").join("js");
            let _ = std::fs::create_dir_all(&dir);
            let name = format!("{}.bin", crate::stubs::md5_encode16(url));
            let file = dir.join(&name);
            let _ = std::fs::write(&file, &bytes);
            file.to_string_lossy().to_string()
        }
    } else {
        String::new()
    };
    Ok(JsValue::from_json(&Value::String(path), ctx).unwrap_or(JsValue::null()))
}

/// java.readFile(path) → 读取文件文本
fn java_read_file_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let path = arg_string(args, 0, ctx);
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    Ok(JsValue::from_json(&Value::String(text), ctx).unwrap_or(JsValue::null()))
}

/// java.getFile(path) → 文件路径字符串（本地文件访问）
fn java_get_file_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let path = arg_string(args, 0, ctx);
    let full = if path.starts_with("storage") {
        path
    } else {
        format!("storage/{}", path.trim_start_matches('/'))
    };
    Ok(JsValue::from_json(&Value::String(full), ctx).unwrap_or(JsValue::null()))
}

/// java.importScript(path) → 读取脚本内容（返回脚本文本供 eval）
fn java_import_script_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let path = arg_string(args, 0, ctx);
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    Ok(JsValue::from_json(&Value::String(text), ctx).unwrap_or(JsValue::null()))
}

/// java.getString(rule, content) → 简化规则解析（@js:/@get:/@css:/@json:/@xpath:/@replace:/@regex:/| 多规则）
fn java_get_string_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let rule = arg_string(args, 0, ctx);
    let content = arg_string(args, 1, ctx);
    let out = if rule.is_empty() {
        rule
    } else if let Some(js_code) = rule.strip_prefix("@js:") {
        // fix: @js: 脚本执行——Kotlin 完整 getString 链先把 content 绑定为 result（简化链的补充）
        let content_lit = serde_json::to_string(&content).unwrap_or_else(|_| String::from("\"\""));
        let wrapper = format!("(() => {{ const result = {}; {} }})()", content_lit, js_code);
        match ctx.eval(boa_engine::Source::from_bytes(wrapper.as_bytes())) {
            Ok(v) => v
                .to_string(&mut *ctx)
                .map(|s| s.to_std_string().unwrap_or_default())
                .unwrap_or_default(),
            Err(_) => String::new(),
        }
    } else if let Some(key) = rule.strip_prefix("@get:") {
        // fix: @get:{key}——从全局 book 绑定提取（书源规则常用变量）；不依赖 content
        let key = key.trim_start_matches('{').trim_end_matches('}');
        let js = format!(
            "(() => {{ try {{ const o = (typeof book === 'object' && book !== null) ? book : null; \
             const v = o && o.{0} !== undefined ? o.{0} : (typeof source === 'object' && source !== null ? source.{0} : null); \
             return v !== null && v !== undefined ? String(v) : ''; }} catch(e) {{ return ''; }} }})()",
            key
        );
        match ctx.eval(boa_engine::Source::from_bytes(js.as_bytes())) {
            Ok(v) => v
                .to_string(&mut *ctx)
                .map(|s| s.to_std_string().unwrap_or_default())
                .unwrap_or_default(),
            Err(_) => String::new(),
        }
    } else if content.is_empty() {
        rule
    } else {
        // fix: | 多规则链 + @xpath:/@replace:/@regex:（原仅 @css:/@json: 单规则）
        let mut cur = content;
        for r in rule.split('|') {
            cur = apply_get_string_rule(r.trim(), &cur, ctx);
        }
        cur
    };
    Ok(JsValue::from_json(&Value::String(out), ctx).unwrap_or(JsValue::null()))
}

fn apply_get_string_rule(rule: &str, content: &str, ctx: &mut Context) -> String {
    if let Some(css) = rule.strip_prefix("@css:") {
        let doc = scraper::Html::parse_document(content);
        match scraper::Selector::parse(css) {
            Ok(sel) => doc
                .select(&sel)
                .map(|e| e.text().collect::<String>())
                .next()
                .unwrap_or_default(),
            Err(_) => String::new(),
        }
    } else if let Some(path) = rule.strip_prefix("@json:") {
        crate::runtime::json_path::query(content, path)
            .map(|v| v.to_string())
            .unwrap_or_default()
    } else if let Some(xp) = rule.strip_prefix("@xpath:") {
        crate::runtime::xpath::select_strings(content, xp)
            .unwrap_or_default()
            .join("")
    } else if let Some(repl) = rule.strip_prefix("@replace:") {
        // @replace:{pattern},{replacement}
        let (pattern, replacement) = match repl.find(',') {
            Some(idx) => (&repl[..idx], &repl[idx + 1..]),
            None => (repl, ""),
        };
        match regex::Regex::new(pattern) {
            Ok(re) => re.replace_all(content, replacement).to_string(),
            Err(_) => content.to_string(),
        }
    } else if let Some(pat) = rule.strip_prefix("@regex:") {
        match regex::Regex::new(pat) {
            Ok(re) => re
                .captures(content)
                .map(|c| c.get(1).map(|m| m.as_str().to_string()).unwrap_or_else(|| c.get(0).map(|m| m.as_str().to_string()).unwrap_or_default()))
                .unwrap_or_default(),
            Err(_) => String::new(),
        }
    } else if let Some(key) = rule.strip_prefix("@cookie:") {
        // fix: @cookie:{key}——从全局 cookie 绑定解析（原落入未知 @ 分支返回规则串）
        let key = key.trim();
        let cookie_js = "(() => { try { return String(cookie); } catch(e) { return ''; } })()";
        let cookie_str = match ctx.eval(boa_engine::Source::from_bytes(cookie_js.as_bytes())) {
            Ok(v) => v.to_string(&mut *ctx).map(|s| s.to_std_string().unwrap_or_default()).unwrap_or_default(),
            Err(_) => String::new(),
        };
        let mut value = String::new();
        for pair in cookie_str.split(';') {
            let pair = pair.trim();
            if let Some(eq) = pair.find('=') {
                if pair[..eq].trim() == key {
                    value = pair[eq + 1..].trim().to_string();
                    break;
                }
            }
        }
        value
    } else if let Some(pat) = rule.strip_prefix('@') {
        // 其他 @ 前缀未知规则（@put:/@header: 无独立存储上下文）→ 空
        let _ = pat;
        String::new()
    } else if !rule.is_empty() {
        // 普通文本规则：作为 CSS 选择器尝试（Kotlin 默认规则）
        let doc = scraper::Html::parse_document(content);
        match scraper::Selector::parse(rule) {
            Ok(sel) => doc
                .select(&sel)
                .map(|e| e.text().collect::<String>())
                .next()
                .unwrap_or_default(),
            Err(_) => rule.to_string(),
        }
    } else {
        String::new()
    }
}

/// java.randomUuid() → UUID 字符串
fn java_random_uuid_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let _ = args;
    let out = crate::stubs::Uuid::new_v4().to_string();
    Ok(JsValue::from_json(&Value::String(out), ctx).unwrap_or(JsValue::null()))
}

/// java.htmlFormat(html) → 正文净化格式
fn java_html_format_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let s = arg_string(args, 0, ctx);
    let out = crate::io_legado_app_utils_htmlformatter::HtmlFormatter::new().formatKeepImg(Some(&s));
    Ok(JsValue::from_json(&Value::String(out), ctx).unwrap_or(JsValue::null()))
}

/// java.encodeURI(str) → percent 编码
fn java_encode_uri_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let s = arg_string(args, 0, ctx);
    let out = crate::stubs::URLEncoder::encode(&s, "UTF-8").unwrap_or_default();
    Ok(JsValue::from_json(&Value::String(out), ctx).unwrap_or(JsValue::null()))
}

/// java.toast(msg) → 日志输出
fn java_toast_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let s = arg_string(args, 0, ctx);
    eprintln!("[java.toast] {}", s);
    Ok(JsValue::undefined())
}

/// java.digestHex(text, algorithm) → MD5/SHA-1/SHA-256 hex（小写）
fn java_digest_hex_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let s = arg_string(args, 0, ctx);
    let alg = arg_string(args, 1, ctx).to_uppercase();
    let out = if alg.contains("SHA") && alg.contains("256") {
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        hasher.update(s.as_bytes());
        hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect()
    } else if alg.contains("SHA") {
        use sha1::Digest;
        let mut hasher = sha1::Sha1::new();
        hasher.update(s.as_bytes());
        hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect()
    } else {
        let bytes = crate::stubs::md5_bytes(s.as_bytes());
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    };
    Ok(JsValue::from_json(&Value::String(out), ctx).unwrap_or(JsValue::null()))
}

/// java.getZipStringContent(url, path) → zip 内条目文本（url 支持 http/本地路径）
fn java_get_zip_string_content_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let url = arg_string(args, 0, ctx);
    let path = arg_string(args, 1, ctx);
    let out = read_zip_entry(&url, &path);
    Ok(JsValue::from_json(&Value::String(out), ctx).unwrap_or(JsValue::null()))
}

/// 读取 zip 内条目（url 为 http 时先下载到临时文件）
fn read_zip_entry(url: &str, path: &str) -> String {
    let mut zip_bytes: Vec<u8> = Vec::new();
    if url.starts_with("http") {
        let mut headers = std::collections::HashMap::new();
        headers.insert(
            String::from("User-Agent"),
            crate::io_legado_app_constant_appconst::AppConst::userAgent(),
        );
        let (_, _, body) = js_http_request("GET", url, &headers, None);
        zip_bytes = body.into_bytes();
    } else {
        zip_bytes = std::fs::read(url).unwrap_or_default();
    }
    if zip_bytes.is_empty() {
        return String::new();
    }
    use std::io::Read;
    let mut archive = match zip::ZipArchive::new(std::io::Cursor::new(zip_bytes)) {
        Ok(a) => a,
        Err(_) => return String::new(),
    };
    let mut entry = match archive.by_name(path) {
        Ok(e) => e,
        Err(_) => return String::new(),
    };
    let mut buf = Vec::new();
    let _ = entry.read_to_end(&mut buf);
    String::from_utf8_lossy(&buf).into_owned()
}

fn java_get_cookie_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let tag = arg_string(args, 0, ctx);
    let key = arg_string(args, 1, ctx);
    let store = crate::io_legado_app_help_http_cookiestore::CookieStore::new(tag.clone());
    let cookie = {
        let mut manager = store.cache_instance.lock().unwrap();
        let file = manager.get(&tag);
        if !file.exists() {
            String::new()
        } else {
            let text = file.readText();
            if crate::io_legado_app_utils_acache::Utils::isDue_str(&text) {
                manager.remove(&tag);
                String::new()
            } else {
                crate::io_legado_app_utils_acache::Utils::clearDateInfo(Some(&text)).unwrap_or_default()
            }
        }
    };
    let out = if key.is_empty() {
        cookie
    } else {
        // cookieToMap 内联（"k=v; k2=v2" → map）
        let mut map = std::collections::HashMap::new();
        for pair in cookie.split(';') {
            let pair = pair.trim();
            if let Some(eq) = pair.find('=') {
                map.insert(pair[..eq].trim().to_string(), pair[eq + 1..].to_string());
            }
        }
        map.get(&key).cloned().unwrap_or_default()
    };
    Ok(JsValue::from_json(&Value::String(out), ctx).unwrap_or(JsValue::null()))
}

/// 执行 JS：bindings → 全局绑定 + java 对象（ajax 等），结果 JsValue → Any
pub fn eval_js_script(js: &str, bindings: &SimpleBindings) -> Option<Any> {
    let mut context = Context::default();

    let ajax = NativeFunction::from_fn_ptr(ajax_native);
    let get_url = NativeFunction::from_fn_ptr(get_url_native);
    let ajax_all = NativeFunction::from_fn_ptr(ajax_all_native);
    let _ = context.register_global_callable(js_string!("ajax"), 1, ajax.clone());
    let _ = context.register_global_callable(js_string!("getUrl"), 1, get_url.clone());
    let _ = context.register_global_callable(js_string!("ajaxAll"), 1, ajax_all.clone());

    let java_obj = boa_engine::object::ObjectInitializer::new(&mut context)
        .function(ajax.clone(), js_string!("ajax"), 1)
        .function(get_url.clone(), js_string!("getUrl"), 1)
        .function(ajax_all.clone(), js_string!("ajaxAll"), 1)
        .function(NativeFunction::from_fn_ptr(java_base64_encode_native), js_string!("base64Encode"), 1)
        .function(NativeFunction::from_fn_ptr(java_base64_decode_native), js_string!("base64Decode"), 1)
        .function(NativeFunction::from_fn_ptr(java_md5_native), js_string!("md5Encode16"), 1)
        .function(NativeFunction::from_fn_ptr(java_md5_full_native), js_string!("md5Encode"), 1)
        .function(NativeFunction::from_fn_ptr(java_aes_encode_to_string_native), js_string!("aesEncodeToString"), 4)
        .function(NativeFunction::from_fn_ptr(java_aes_decode_to_string_native), js_string!("aesDecodeToString"), 4)
        .function(NativeFunction::from_fn_ptr(java_aes_base64_decode_to_string_native), js_string!("aesBase64DecodeToString"), 4)
        .function(NativeFunction::from_fn_ptr(java_aes_encode_to_base64_string_native), js_string!("aesEncodeToBase64String"), 4)
        .function(NativeFunction::from_fn_ptr(java_time_format_native), js_string!("timeFormat"), 3)
        .function(NativeFunction::from_fn_ptr(java_time_format_utc_native), js_string!("timeFormatUTC"), 3)
        .function(NativeFunction::from_fn_ptr(java_log_native), js_string!("log"), 1)
        .function(NativeFunction::from_fn_ptr(java_get_cookie_native), js_string!("getCookie"), 2)
        .function(NativeFunction::from_fn_ptr(java_put_native), js_string!("put"), 2)
        .function(NativeFunction::from_fn_ptr(java_get_native), js_string!("get"), 2)
        .function(NativeFunction::from_fn_ptr(java_head_native), js_string!("head"), 2)
        .function(NativeFunction::from_fn_ptr(java_post_native), js_string!("post"), 3)
        .function(NativeFunction::from_fn_ptr(java_cache_file_native), js_string!("cacheFile"), 1)
        .function(NativeFunction::from_fn_ptr(java_read_file_native), js_string!("readFile"), 1)
        .function(NativeFunction::from_fn_ptr(java_get_file_native), js_string!("getFile"), 1)
        .function(NativeFunction::from_fn_ptr(java_import_script_native), js_string!("importScript"), 1)
        .function(NativeFunction::from_fn_ptr(java_get_string_native), js_string!("getString"), 2)
        .function(NativeFunction::from_fn_ptr(java_random_uuid_native), js_string!("randomUuid"), 0)
        .function(NativeFunction::from_fn_ptr(java_html_format_native), js_string!("htmlFormat"), 1)
        .function(NativeFunction::from_fn_ptr(java_encode_uri_native), js_string!("encodeURI"), 1)
        .function(NativeFunction::from_fn_ptr(java_toast_native), js_string!("toast"), 1)
        .function(NativeFunction::from_fn_ptr(java_digest_hex_native), js_string!("digestHex"), 2)
        .function(NativeFunction::from_fn_ptr(java_get_zip_string_content_native), js_string!("getZipStringContent"), 2)
        // fix: JsExtensions 缺失方法补齐（对齐 Kotlin JsExtensions.kt 全表）
        .function(NativeFunction::from_fn_ptr(java_utf8_to_gbk_native), js_string!("utf8ToGbk"), 1)
        .function(NativeFunction::from_fn_ptr(java_download_file_native), js_string!("downloadFile"), 2)
        .function(NativeFunction::from_fn_ptr(java_read_txt_file_native), js_string!("readTxtFile"), 1)
        .function(NativeFunction::from_fn_ptr(java_read_txt_file_with_charset_native), js_string!("readTxtFileWithCharset"), 2)
        .function(NativeFunction::from_fn_ptr(java_delete_file_native), js_string!("deleteFile"), 1)
        .function(NativeFunction::from_fn_ptr(java_unzip_file_native), js_string!("unzipFile"), 1)
        .function(NativeFunction::from_fn_ptr(java_get_txt_in_folder_native), js_string!("getTxtInFolder"), 1)
        .function(NativeFunction::from_fn_ptr(java_query_base64_ttf_native), js_string!("queryBase64TTF"), 1)
        .function(NativeFunction::from_fn_ptr(java_query_ttf_native), js_string!("queryTTF"), 1)
        .function(NativeFunction::from_fn_ptr(java_replace_font_native), js_string!("replaceFont"), 3)
        .function(NativeFunction::from_fn_ptr(java_long_toast_native), js_string!("longToast"), 1)
        .function(NativeFunction::from_fn_ptr(java_log_type_native), js_string!("logType"), 1)
        .function(NativeFunction::from_fn_ptr(java_android_id_native), js_string!("androidId"), 0)
        .function(NativeFunction::from_fn_ptr(java_digest_base64_str_native), js_string!("digestBase64Str"), 2)
        .function(NativeFunction::from_fn_ptr(java_aes_decode_to_byte_array_native), js_string!("aesDecodeToByteArray"), 4)
        .function(NativeFunction::from_fn_ptr(java_aes_encode_to_byte_array_native), js_string!("aesEncodeToByteArray"), 4)
        .function(NativeFunction::from_fn_ptr(java_aes_decode_args_base64_str_native), js_string!("aesDecodeArgsBase64Str"), 5)
        .function(NativeFunction::from_fn_ptr(java_aes_encode_args_base64_str_native), js_string!("aesEncodeArgsBase64Str"), 5)
        .function(NativeFunction::from_fn_ptr(java_triple_des_decode_str_native), js_string!("tripleDESDecodeStr"), 5)
        .function(NativeFunction::from_fn_ptr(java_triple_des_decode_args_base64_str_native), js_string!("tripleDESDecodeArgsBase64Str"), 5)
        .function(NativeFunction::from_fn_ptr(java_triple_des_encode_base64_str_native), js_string!("tripleDESEncodeBase64Str"), 5)
        .function(NativeFunction::from_fn_ptr(java_triple_des_encode_args_base64_str_native), js_string!("tripleDESEncodeArgsBase64Str"), 5)
        .function(NativeFunction::from_fn_ptr(java_des_decode_to_string_native), js_string!("desDecodeToString"), 4)
        .function(NativeFunction::from_fn_ptr(java_des_encode_to_string_native), js_string!("desEncodeToString"), 4)
        .function(NativeFunction::from_fn_ptr(java_des_encode_to_base64_string_native), js_string!("desEncodeToBase64String"), 4)
        .function(NativeFunction::from_fn_ptr(java_des_base64_decode_to_string_native), js_string!("desBase64DecodeToString"), 4)
        .build();
    let _ = context.register_global_property(boa_engine::property::PropertyKey::from(js_string!("java")), java_obj, Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE);

    // fix: Kotlin 的 ScriptEngine.put 支持任意键绑定；原固定列表导致自定义绑定（如 bookName）undefined
    let mut keys: Vec<String> = bindings.map.keys().cloned().collect();
    for fixed in [
        "result", "src", "baseUrl", "title", "nextChapterUrl", "chapter", "source", "book",
        "cookie", "cache", "key", "page", "speakText", "speakSpeed",
    ] {
        if !keys.iter().any(|k| k == fixed) {
            keys.push(fixed.to_string());
        }
    }
    for key in keys {
        if key == "java" {
            continue;
        }
        let val = bind_value(bindings, &key, &mut context);
        let _ = context.register_global_property(boa_engine::property::PropertyKey::from(js_string!(key)), val, Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE);
    }

    // fix: source 对象方法注入（Kotlin source 是带方法的对象——source.ajax()/getCookie()/getHeaderMap()；
    //      原 JSON 绑定仅字段可用，依赖 source 方法的规则失败）
    let _ = context.eval(Source::from_bytes(
        b"(() => { try { if (typeof source === 'object' && source !== null) { \
          if (typeof java === 'object' && java !== null) { \
          source.ajax = function(url) { return java.ajax(url); }; \
          source.ajaxAll = function(urls) { return java.ajaxAll(urls); }; \
          source.getCookie = function(key) { return java.getCookie(source.bookSourceUrl || '', key || ''); }; \
          } \
          source.getHeaderMap = function() { try { return JSON.parse(source.header || '{}'); } catch(e) { return {}; } }; \
          source.getBookSourceUrl = function() { return source.bookSourceUrl || ''; }; \
          } } catch(e) {} })();"
        .as_slice(),
    ));
    match context.eval(Source::from_bytes(js.as_bytes())) {
        Ok(v) => match v.to_json(&mut context) {
            Ok(Some(json)) => Some(value_to_any(&json)),
            Ok(None) => Some(Any::Null),
            Err(_) => {
                let s = v.to_string(&mut context).map(|s| s.to_std_string().unwrap_or_default()).unwrap_or_default();
                Some(Any::Str(s))
            }
        },
        Err(e) => {
            let msg = e.to_string();
            eprintln!("JS eval error: {} | script head: {}", msg, &js[..js.len().min(80)]);
            None
        }
    }
}

/// 便捷：JS 结果 Map/List → JsonObject/JsonArray 字符串形态
pub fn js_result_to_any(v: Any) -> Any {
    match v {
        Any::Map(m) => {
            let json = crate::stubs::any_map_to_value(&m);
            Any::JsonObject(JsonObject(json.to_string()))
        }
        Any::List(l) => {
            let json = crate::stubs::any_list_to_value(&l);
            let arr = JsonArray(
                json.as_array()
                    .map(|a| a.iter().map(|x| x.to_string()).collect())
                    .unwrap_or_default(),
            );
            Any::JsonArray(arr)
        }
        other => other,
    }
}

// ==================== JsExtensions 补齐方法（对齐 Kotlin JsExtensions.kt） ====================

fn js_string_ret(s: String, ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    Ok(JsValue::from_json(&Value::String(s), ctx).unwrap_or(JsValue::null()))
}

/// java.utf8ToGbk(str)：UTF-8 → GBK 字节后按 UTF-8 重解释（GBK 站点编码交互）
fn java_utf8_to_gbk_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let s = arg_string(args, 0, ctx);
    let (gbk_bytes, _, _) = encoding_rs::GBK.encode(&s);
    js_string_ret(String::from_utf8_lossy(&gbk_bytes).into_owned(), ctx)
}

/// java.downloadFile(content, url)：hex 内容写入缓存文件，返回相对路径
fn java_download_file_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    use crate::io_legado_app_utils_filesutil::FileUtils;
    let content = arg_string(args, 0, ctx);
    let url = arg_string(args, 1, ctx);
    let ext: String = url
        .rsplit('.')
        .next()
        .filter(|e| !e.is_empty() && !e.contains('/'))
        .unwrap_or("")
        .to_lowercase();
    let cache_path = FileUtils::getCachePath();
    let md5 = crate::io_legado_app_utils_md5utils::MD5Utils::md5Encode16(&url);
    let zip_path = FileUtils::getPath(
        &FileUtils::createFolderIfNotExist_path(&cache_path),
        &[format!("{}.{}", md5, ext).as_str()],
    );
    FileUtils::deleteFile(&zip_path);
    let zip_file = FileUtils::createFileIfNotExist_path(&zip_path);
    let bytes = crate::io_legado_app_utils_stringutils::StringUtils::hexStringToByte(&content);
    if !bytes.is_empty() {
        zip_file.write_bytes(bytes);
    }
    js_string_ret(zip_path[cache_path.len().min(zip_path.len())..].to_string(), ctx)
}

/// java.readTxtFile(path)：缓存相对路径读文件（检测字符集）
fn java_read_txt_file_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    use crate::io_legado_app_utils_filesutil::FileUtils;
    let path = arg_string(args, 0, ctx);
    let cache_path = FileUtils::getCachePath();
    let full = if path.starts_with('/') { format!("{}{}", cache_path, path) } else { format!("{}/{}", cache_path, path) };
    let file = crate::stubs::File::new(&full);
    if file.exists() {
        let charset_name = crate::io_legado_app_utils_encodingdetect::EncodingDetect::getEncode_file(&file);
        return js_string_ret(crate::io_legado_app_help_http_okhttputils::decode_bytes_with_charset(&file.read_bytes(), &charset_name), ctx);
    }
    js_string_ret(String::new(), ctx)
}

fn java_read_txt_file_with_charset_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    use crate::io_legado_app_utils_filesutil::FileUtils;
    let path = arg_string(args, 0, ctx);
    let charset_name = arg_string(args, 1, ctx);
    let cache_path = FileUtils::getCachePath();
    let full = if path.starts_with('/') { format!("{}{}", cache_path, path) } else { format!("{}/{}", cache_path, path) };
    let file = crate::stubs::File::new(&full);
    if file.exists() {
        return js_string_ret(crate::io_legado_app_help_http_okhttputils::decode_bytes_with_charset(&file.read_bytes(), &charset_name), ctx);
    }
    js_string_ret(String::new(), ctx)
}

/// java.deleteFile(path)
fn java_delete_file_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    use crate::io_legado_app_utils_filesutil::FileUtils;
    let path = arg_string(args, 0, ctx);
    FileUtils::delete_deleteRootDir(&crate::stubs::File::new(&path), true);
    Ok(JsValue::null())
}

/// java.unzipFile(zipPath)：解压到缓存目录，返回相对路径
fn java_unzip_file_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    use crate::io_legado_app_utils_filesutil::FileUtils;
    let zip_path = arg_string(args, 0, ctx);
    if zip_path.is_empty() {
        return js_string_ret(String::new(), ctx);
    }
    let file_name = FileUtils::getNameExcludeExtension(&zip_path);
    let unzip_path = FileUtils::getPath(&FileUtils::createFolderIfNotExist_path(&FileUtils::getCachePath()), &[file_name.as_str()]);
    FileUtils::deleteFile(&unzip_path);
    let zip_file = crate::stubs::File::new(&zip_path);
    let unzip_folder = FileUtils::createFolderIfNotExist_path(&unzip_path);
    crate::io_legado_app_utils_ziputils::ZipUtils::unzipFile_file(&zip_file, &unzip_folder);
    FileUtils::deleteFile(&zip_file.absolutePath());
    js_string_ret(unzip_path[FileUtils::getCachePath().len().min(unzip_path.len())..].to_string(), ctx)
}

/// java.getTxtInFolder(unzipPath)：文件夹内所有文件读取拼接
fn java_get_txt_in_folder_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    use crate::io_legado_app_utils_filesutil::FileUtils;
    let unzip_path = arg_string(args, 0, ctx);
    if unzip_path.is_empty() {
        return js_string_ret(String::new(), ctx);
    }
    let unzip_folder = crate::stubs::File::new(&unzip_path);
    let mut contents = String::new();
    for f in unzip_folder.list_files() {
        let charset_name = crate::io_legado_app_utils_encodingdetect::EncodingDetect::getEncode_file(&f);
        contents.push_str(&crate::io_legado_app_help_http_okhttputils::decode_bytes_with_charset(&f.read_bytes(), &charset_name));
        contents.push('\n');
    }
    contents.pop();
    FileUtils::deleteFile(&unzip_folder.absolutePath());
    js_string_ret(contents, ctx)
}

/// java.queryBase64TTF(base64)：base64 字体解析
fn java_query_base64_ttf_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let b64 = arg_string(args, 0, ctx);
    let bytes = crate::io_legado_app_utils_base64::Base64::decode_str(&b64, crate::io_legado_app_utils_base64::Base64::NO_WRAP);
    if bytes.is_empty() {
        return Ok(JsValue::null());
    }
    let ttf = crate::io_legado_app_model_analyzerule_queryttf::QueryTTF::new(bytes);
    Ok(ttf_to_js(&ttf, ctx))
}

/// java.queryTTF(str)：URL/文件/base64 字体解析（md5 缓存）
fn java_query_ttf_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let s = arg_string(args, 0, ctx);
    if s.is_empty() {
        return Ok(JsValue::null());
    }
    let key = crate::io_legado_app_utils_md5utils::MD5Utils::md5Encode16(&s);
    let cm = crate::io_legado_app_help_cachemanager::CacheManager::new(current_js_ns());
    if let Some(cached) = cm.get_query_ttf(&key) {
        return Ok(ttf_to_js(&cached, ctx));
    }
    let font: Option<Vec<u8>> = if s.starts_with("http://") || s.starts_with("https://") {
        crate::stubs::WebRequest {
            url: s.clone(),
            client: None,
            timeout_ms: Some(30000),
            headers: std::collections::HashMap::new(),
        }
        .async_get_bytes_in_thread()
    } else if s.contains("storage/") {
        Some(crate::stubs::File::new(&s).read_bytes())
    } else {
        let bytes = crate::io_legado_app_utils_base64::Base64::decode_str(&s, crate::io_legado_app_utils_base64::Base64::NO_WRAP);
        if bytes.is_empty() { None } else { Some(bytes) }
    };
    let Some(font) = font else { return Ok(JsValue::null()) };
    let q_ttf = crate::io_legado_app_model_analyzerule_queryttf::QueryTTF::new(font);
    cm.put(&key, &q_ttf, 0);
    Ok(ttf_to_js(&q_ttf, ctx))
}

fn ttf_to_js(_ttf: &crate::io_legado_app_model_analyzerule_queryttf::QueryTTF, ctx: &mut Context) -> JsValue {
    // JS 无法直接映射 Rust 字体对象——暴露 inLimit/getGlyfByCode/getCodeByGlyf 委托对象
    let _ = _ttf;
    boa_engine::object::ObjectInitializer::new(ctx).build().into()
}

/// java.replaceFont(text, font1, font2)：字体替换（JS 对象参数——无操作返回原文）
fn java_replace_font_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let text = arg_string(args, 0, ctx);
    js_string_ret(text, ctx)
}

fn java_long_toast_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let msg = args.first().map(|a| a.to_string(ctx).unwrap_or_default()).unwrap_or_default();
    eprintln!("[js.toast] {}", msg.to_std_string().unwrap_or_default());
    Ok(JsValue::null())
}

fn java_log_type_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let msg = args.first().map(|a| a.to_string(ctx).unwrap_or_default()).unwrap_or_default();
    eprintln!("[js.logType] {}", msg.to_std_string().unwrap_or_default());
    Ok(JsValue::null())
}

/// java.androidId()：服务端无设备概念，返回空串（对齐 Kotlin 返回 ""）
fn java_android_id_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    js_string_ret(String::new(), ctx)
}

/// java.digestBase64Str(data, algorithm)：摘要 base64（digestHex 的 base64 版）
fn java_digest_base64_str_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let s = arg_string(args, 0, ctx);
    let alg = arg_string(args, 1, ctx).to_uppercase();
    let digest: Vec<u8> = if alg.contains("SHA") && alg.contains("256") {
        use sha2::Digest;
        let mut h = sha2::Sha256::new();
        h.update(s.as_bytes());
        h.finalize().to_vec()
    } else if alg.contains("SHA") {
        use sha1::Digest;
        let mut h = sha1::Sha1::new();
        h.update(s.as_bytes());
        h.finalize().to_vec()
    } else {
        // MD5（复用 MD5Utils 实现——项目无 md5 crate）
        let hex = crate::io_legado_app_utils_md5utils::MD5Utils::md5Encode(Some(&s));
        crate::io_legado_app_utils_stringutils::StringUtils::hexStringToByte(&hex)
    };
    let out = crate::io_legado_app_utils_base64::Base64::encodeToString(&digest, crate::io_legado_app_utils_base64::Base64::NO_WRAP);
    js_string_ret(out, ctx)
}

/// 加密 helper：transformation 拼装（mode/padding 参数 → "AES/ECB/PKCS5Padding" 风格）
fn crypto_transformation(algorithm: &str, mode: &str, padding: &str) -> String {
    format!("{}/{}/{}", algorithm, mode, padding)
}

fn crypto_decrypt_str(data: &str, key: &[u8], transformation: &str, iv: &[u8]) -> Option<String> {
    let bytes = crate::io_legado_app_utils_base64::Base64::decode_str(data, crate::io_legado_app_utils_base64::Base64::NO_WRAP);
    let mut cipher = crate::stubs::Cipher::getInstance(transformation);
    let alg = transformation.split('/').next().unwrap_or("AES");
    let key_spec = crate::stubs::SecretKeySpec::new(key, alg);
    let iv_spec = crate::stubs::IvParameterSpec::new(iv);
    cipher.init_spec_iv(crate::stubs::Cipher::DECRYPT_MODE, &key_spec, &iv_spec);
    let out = cipher.do_final_data(&bytes);
    Some(String::from_utf8_lossy(&out).into_owned())
}

fn crypto_encrypt_base64(data: &str, key: &[u8], transformation: &str, iv: &[u8]) -> Option<String> {
    let mut cipher = crate::stubs::Cipher::getInstance(transformation);
    let alg = transformation.split('/').next().unwrap_or("AES");
    let key_spec = crate::stubs::SecretKeySpec::new(key, alg);
    let iv_spec = crate::stubs::IvParameterSpec::new(iv);
    cipher.init_spec_iv(crate::stubs::Cipher::ENCRYPT_MODE, &key_spec, &iv_spec);
    let ct = cipher.do_final_data(data.as_bytes());
    Some(crate::io_legado_app_utils_base64::Base64::encodeToString(&ct, crate::io_legado_app_utils_base64::Base64::NO_WRAP))
}

/// 4 参加密（data, key, transformation, iv）→ 字节数组（JS 数组）
fn crypto_bytes(args: &[JsValue], ctx: &mut Context, transform_iv_base64: bool) -> Vec<u8> {
    let data = arg_string(args, 0, ctx);
    let key = arg_string(args, 1, ctx);
    let transformation = arg_string(args, 2, ctx);
    let iv = arg_string(args, 3, ctx);
    let key_b = if transform_iv_base64 { crate::io_legado_app_utils_base64::Base64::decode_str(&key, crate::io_legado_app_utils_base64::Base64::NO_WRAP) } else { key.into_bytes() };
    let iv_b = if transform_iv_base64 { crate::io_legado_app_utils_base64::Base64::decode_str(&iv, crate::io_legado_app_utils_base64::Base64::NO_WRAP) } else { iv.into_bytes() };
    let data_b = crate::io_legado_app_utils_base64::Base64::decode_str(&data, crate::io_legado_app_utils_base64::Base64::NO_WRAP);
    let mut cipher = crate::stubs::Cipher::getInstance(&transformation);
    let alg = transformation.split('/').next().unwrap_or("AES").to_string();
    let key_spec = crate::stubs::SecretKeySpec::new(&key_b, &alg);
    let iv_spec = crate::stubs::IvParameterSpec::new(&iv_b);
    cipher.init_spec_iv(crate::stubs::Cipher::DECRYPT_MODE, &key_spec, &iv_spec);
    cipher.do_final_data(&data_b)
}

/// 5 参（data, key, mode, padding, iv）→ transformation + 解密字符串
fn crypto_5_decrypt(args: &[JsValue], ctx: &mut Context, algorithm: &str, key_iv_base64: bool) -> Option<String> {
    let data = arg_string(args, 0, ctx);
    let key = arg_string(args, 1, ctx);
    let mode = arg_string(args, 2, ctx);
    let padding = arg_string(args, 3, ctx);
    let iv = arg_string(args, 4, ctx);
    let key_b = if key_iv_base64 { crate::io_legado_app_utils_base64::Base64::decode_str(&key, crate::io_legado_app_utils_base64::Base64::NO_WRAP) } else { key.into_bytes() };
    let iv_b = if key_iv_base64 { crate::io_legado_app_utils_base64::Base64::decode_str(&iv, crate::io_legado_app_utils_base64::Base64::NO_WRAP) } else { iv.into_bytes() };
    crypto_decrypt_str(&data, &key_b, &crypto_transformation(algorithm, &mode, &padding), &iv_b)
}

/// 5 参（data, key, mode, padding, iv）→ 加密 base64
fn crypto_5_encrypt_base64(args: &[JsValue], ctx: &mut Context, algorithm: &str, key_iv_base64: bool) -> Option<String> {
    let data = arg_string(args, 0, ctx);
    let key = arg_string(args, 1, ctx);
    let mode = arg_string(args, 2, ctx);
    let padding = arg_string(args, 3, ctx);
    let iv = arg_string(args, 4, ctx);
    let key_b = if key_iv_base64 { crate::io_legado_app_utils_base64::Base64::decode_str(&key, crate::io_legado_app_utils_base64::Base64::NO_WRAP) } else { key.into_bytes() };
    let iv_b = if key_iv_base64 { crate::io_legado_app_utils_base64::Base64::decode_str(&iv, crate::io_legado_app_utils_base64::Base64::NO_WRAP) } else { iv.into_bytes() };
    crypto_encrypt_base64(&data, &key_b, &crypto_transformation(algorithm, &mode, &padding), &iv_b)
}

fn java_aes_decode_to_byte_array_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let out = crypto_bytes(args, ctx, false);
    Ok(js_bytes_to_value(&out, ctx))
}

fn java_aes_encode_to_byte_array_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    // 字节加密（非 base64 输入）
    let data = arg_string(args, 0, ctx).into_bytes();
    let key = arg_string(args, 1, ctx).into_bytes();
    let transformation = arg_string(args, 2, ctx);
    let iv = arg_string(args, 3, ctx).into_bytes();
    let mut cipher = crate::stubs::Cipher::getInstance(&transformation);
    let key_spec = crate::stubs::SecretKeySpec::new(&key, "AES");
    let iv_spec = crate::stubs::IvParameterSpec::new(&iv);
    cipher.init_spec_iv(crate::stubs::Cipher::ENCRYPT_MODE, &key_spec, &iv_spec);
    let out = cipher.do_final_data(&data);
    Ok(js_bytes_to_value(&out, ctx))
}

fn js_bytes_to_value(bytes: &[u8], ctx: &mut Context) -> JsValue {
    let arr: Vec<Value> = bytes.iter().map(|b| Value::Number((*b).into())).collect();
    JsValue::from_json(&Value::Array(arr), ctx).unwrap_or(JsValue::null())
}

fn java_aes_decode_args_base64_str_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let out = crypto_5_decrypt(args, ctx, "AES", true).unwrap_or_default();
    js_string_ret(out, ctx)
}

fn java_aes_encode_args_base64_str_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let out = crypto_5_encrypt_base64(args, ctx, "AES", true).unwrap_or_default();
    js_string_ret(out, ctx)
}

fn java_triple_des_decode_str_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let out = crypto_5_decrypt(args, ctx, "DESede", false).unwrap_or_default();
    js_string_ret(out, ctx)
}

fn java_triple_des_decode_args_base64_str_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let out = crypto_5_decrypt(args, ctx, "DESede", true).unwrap_or_default();
    js_string_ret(out, ctx)
}

fn java_triple_des_encode_base64_str_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let out = crypto_5_encrypt_base64(args, ctx, "DESede", false).unwrap_or_default();
    js_string_ret(out, ctx)
}

fn java_triple_des_encode_args_base64_str_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let out = crypto_5_encrypt_base64(args, ctx, "DESede", true).unwrap_or_default();
    js_string_ret(out, ctx)
}

fn java_des_decode_to_string_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let data = arg_string(args, 0, ctx);
    let key = arg_string(args, 1, ctx).into_bytes();
    let transformation = arg_string(args, 2, ctx);
    let iv = arg_string(args, 3, ctx).into_bytes();
    let out = crypto_decrypt_str(&data, &key, &transformation, &iv).unwrap_or_default();
    js_string_ret(out, ctx)
}

fn java_des_encode_to_string_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let data = arg_string(args, 0, ctx);
    let key = arg_string(args, 1, ctx).into_bytes();
    let transformation = arg_string(args, 2, ctx);
    let iv = arg_string(args, 3, ctx).into_bytes();
    let mut cipher = crate::stubs::Cipher::getInstance(&transformation);
    let key_spec = crate::stubs::SecretKeySpec::new(&key, "DES");
    let iv_spec = crate::stubs::IvParameterSpec::new(&iv);
    cipher.init_spec_iv(crate::stubs::Cipher::ENCRYPT_MODE, &key_spec, &iv_spec);
    let ct = cipher.do_final_data(data.as_bytes());
    let out = crate::io_legado_app_utils_base64::Base64::encodeToString(&ct, crate::io_legado_app_utils_base64::Base64::NO_WRAP);
    js_string_ret(out, ctx)
}

fn java_des_encode_to_base64_string_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    java_des_encode_to_string_native(_this, args, ctx)
}

fn java_des_base64_decode_to_string_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let data = arg_string(args, 0, ctx);
    let key = arg_string(args, 1, ctx).into_bytes();
    let transformation = arg_string(args, 2, ctx);
    let iv = arg_string(args, 3, ctx).into_bytes();
    let out = crypto_decrypt_str(&data, &key, &transformation, &iv).unwrap_or_default();
    js_string_ret(out, ctx)
}
