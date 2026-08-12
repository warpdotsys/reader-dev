use crate::prelude::*;
use crate::stubs::Any;
use crate::stubs::io::vertx::{Route, RoutingContext, URLDecoder};
// @file:JvmName("VertExtKt")

// package com.htmake.reader.utils

// import com.htmake.reader.entity.BasicError
// import io.vertx.core.Handler
// import io.vertx.core.json.JsonObject
// import io.vertx.ext.web.Route
// import io.vertx.ext.web.RoutingContext
// import java.net.URLDecoder
// import org.slf4j.MDC

static logger: Log = Log;

// fun RoutingContext.success(any: Any?) {
pub fn success(this: &mut RoutingContext, any: Option<Any>) {
    // fix: Kotlin `any is JsonObject` 类型检查 → Rust 占位判断（JsonObject 为占位类型）
    let to_json = if any.as_ref().is_some_and(|a| matches!(a, Any::JsonObject(_))) {
        any.unwrap().to_string()
    } else {
        // fix: gson().to_json(any) 占位（Any 无 serde::Serialize），以 Display 输出近似 JSON
        any.map_or_else(|| "null".to_string(), |a| a.to_string())
    };
    this.response()
        .put_header("content-type", "application/json; charset=utf-8")
        .end(to_json);
}

// fun RoutingContext.error(throwable: Throwable) {
pub fn error(this: &mut RoutingContext, throwable: Throwable) {
    let error = BasicError {
        error: "Internal Server Error".to_string(),
        exception: throwable.to_string(),
        message: throwable.msg.to_string(),
        path: URLDecoder::decode(this.request().absolute_uri(), "UTF-8"),
        status: 500,
        timestamp: System::current_time_millis(),
    };
    // fix: BasicError 未 derive serde，gson().to_json 不可用 → 占位序列化
    let error_json = gson_to_json_placeholder(&error);
    logger.error(format!("Internal Server Error: {}", throwable));
    logger.error(error_json.clone());
    this.response()
        .put_header("content-type", "application/json; charset=utf-8")
        .set_status_code(500)
        .end(error_json);
}

// fun Route.globalHandler(handler: Handler<RoutingContext>) {
pub fn global_handler<F>(mut this: Route, handler: F)
where
    F: Fn(&mut RoutingContext) + 'static,
{
    let wrapped = move |context: &mut RoutingContext| {
        let trace_id = context.get_user::<String>("traceId").filter(|it| !it.is_empty()).unwrap_or_else(|| get_trace_id());
        MDC::put("traceId", trace_id.clone());
        context.put("traceId", trace_id);
        handler(context);
    };
    this.global_handler(wrapped);
}
