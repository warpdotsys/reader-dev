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
        .build();
    let _ = context.register_global_property(boa_engine::property::PropertyKey::from(boa_engine::js_string!("java")), java_obj, Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE);

    for key in [
        "result", "src", "baseUrl", "title", "nextChapterUrl", "chapter", "source", "book",
        "cookie", "cache", "key", "page", "speakText", "speakSpeed",
    ] {
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
