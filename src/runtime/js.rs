// 真实 JS 规则引擎（boa_engine 封装）
// 供 stubs::ScriptEngine::eval 调用：SimpleBindings → JS 环境 → Any 结果

use boa_engine::native_function::NativeFunction;
use boa_engine::property::Attribute;
use boa_engine::{Context, JsValue, Source};
use serde_json::Value;

use crate::stubs::{Any, JsonArray, JsonObject, SimpleBindings};

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
    JsValue::null()
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
        crate::stubs::WebClient::new()
            .get_abs(&url.to_std_string().unwrap_or_default())
            .timeout(30000)
            .send_blocking()
    };
    Ok(JsValue::from_json(&Value::String(text), ctx).unwrap_or(JsValue::null()))
}

/// 全局 getUrl / get(url)：同 ajax
fn get_url_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    ajax_native(_this, args, ctx)
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

/// java.get(url, headers) → { body, statusCode, headers }（Connection.Response 简化）
fn java_get_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let url = arg_string(args, 0, ctx);
    let headers = parse_js_map(args, 1, ctx);
    let mut req = crate::stubs::WebClient::new().get_abs(&url).timeout(30000);
    for (k, v) in &headers {
        req = req.header(k, v);
    }
    let text = req.async_get_text_in_thread().unwrap_or_default();
    let body_val = JsValue::from_json(&Value::String(text), ctx).unwrap_or(JsValue::null());
    let obj = boa_engine::object::ObjectInitializer::new(ctx)
        .property(boa_engine::js_string!("body"), body_val, boa_engine::property::Attribute::all())
        .property(boa_engine::js_string!("statusCode"), 200, boa_engine::property::Attribute::all())
        .property(boa_engine::js_string!("headers"), JsValue::null(), boa_engine::property::Attribute::all())
        .build();
    Ok(obj.into())
}

/// java.head(url, headers) → { statusCode, headers }
fn java_head_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let _url = arg_string(args, 0, ctx);
    let obj = boa_engine::object::ObjectInitializer::new(ctx)
        .property(boa_engine::js_string!("statusCode"), 200, boa_engine::property::Attribute::all())
        .property(boa_engine::js_string!("headers"), JsValue::null(), boa_engine::property::Attribute::all())
        .build();
    Ok(obj.into())
}

/// java.post(url, body, headers) → { body, statusCode, headers }
fn java_post_native(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> boa_engine::JsResult<JsValue> {
    let url = arg_string(args, 0, ctx);
    let body = arg_string(args, 1, ctx);
    let headers = parse_js_map(args, 2, ctx);
    let mut req = crate::stubs::WebClient::new().get_abs(&url).timeout(30000);
    for (k, v) in &headers {
        req = req.header(k, v);
    }
    let text = req.async_post_in_thread(&body).unwrap_or_default();
    let body_val = JsValue::from_json(&Value::String(text), ctx).unwrap_or(JsValue::null());
    let obj = boa_engine::object::ObjectInitializer::new(ctx)
        .property(boa_engine::js_string!("body"), body_val, boa_engine::property::Attribute::all())
        .property(boa_engine::js_string!("statusCode"), 200, boa_engine::property::Attribute::all())
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
    let _ = context.register_global_callable(boa_engine::js_string!("ajax"), 1, ajax.clone());
    let _ = context.register_global_callable(boa_engine::js_string!("getUrl"), 1, get_url.clone());

    let java_obj = boa_engine::object::ObjectInitializer::new(&mut context)
        .function(ajax.clone(), boa_engine::js_string!("ajax"), 1)
        .function(get_url.clone(), boa_engine::js_string!("getUrl"), 1)
        .function(NativeFunction::from_fn_ptr(java_base64_encode_native), boa_engine::js_string!("base64Encode"), 1)
        .function(NativeFunction::from_fn_ptr(java_base64_decode_native), boa_engine::js_string!("base64Decode"), 1)
        .function(NativeFunction::from_fn_ptr(java_md5_native), boa_engine::js_string!("md5Encode16"), 1)
        .function(NativeFunction::from_fn_ptr(java_md5_full_native), boa_engine::js_string!("md5Encode"), 1)
        .function(NativeFunction::from_fn_ptr(java_aes_encode_to_string_native), boa_engine::js_string!("aesEncodeToString"), 4)
        .function(NativeFunction::from_fn_ptr(java_aes_decode_to_string_native), boa_engine::js_string!("aesDecodeToString"), 4)
        .function(NativeFunction::from_fn_ptr(java_aes_base64_decode_to_string_native), boa_engine::js_string!("aesBase64DecodeToString"), 4)
        .function(NativeFunction::from_fn_ptr(java_aes_encode_to_base64_string_native), boa_engine::js_string!("aesEncodeToBase64String"), 4)
        .function(NativeFunction::from_fn_ptr(java_time_format_native), boa_engine::js_string!("timeFormat"), 3)
        .function(NativeFunction::from_fn_ptr(java_time_format_utc_native), boa_engine::js_string!("timeFormatUTC"), 3)
        .function(NativeFunction::from_fn_ptr(java_log_native), boa_engine::js_string!("log"), 1)
        .function(NativeFunction::from_fn_ptr(java_get_cookie_native), boa_engine::js_string!("getCookie"), 2)
        .function(NativeFunction::from_fn_ptr(java_get_native), boa_engine::js_string!("get"), 2)
        .function(NativeFunction::from_fn_ptr(java_head_native), boa_engine::js_string!("head"), 2)
        .function(NativeFunction::from_fn_ptr(java_post_native), boa_engine::js_string!("post"), 3)
        .function(NativeFunction::from_fn_ptr(java_cache_file_native), boa_engine::js_string!("cacheFile"), 1)
        .function(NativeFunction::from_fn_ptr(java_read_file_native), boa_engine::js_string!("readFile"), 1)
        .function(NativeFunction::from_fn_ptr(java_get_file_native), boa_engine::js_string!("getFile"), 1)
        .function(NativeFunction::from_fn_ptr(java_import_script_native), boa_engine::js_string!("importScript"), 1)
        .build();
    let _ = context.register_global_property(boa_engine::property::PropertyKey::from(boa_engine::js_string!("java")), java_obj, Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE);

    for key in [
        "result", "src", "baseUrl", "title", "nextChapterUrl", "chapter", "source", "book",
        "cookie", "cache", "key", "page", "speakText", "speakSpeed",
    ] {
        if key == "java" {
            continue;
        }
        let val = bind_value(bindings, key, &mut context);
        let _ = context.register_global_property(boa_engine::property::PropertyKey::from(boa_engine::js_string!(key)), val, Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE);
    }

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
