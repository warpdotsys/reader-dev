// @file:JvmName("VertExtKt")

// package com.htmake.reader.utils

// import com.htmake.reader.entity.BasicError
// import io.vertx.core.Handler
// import io.vertx.core.json.JsonObject
// import io.vertx.ext.web.Route
// import io.vertx.ext.web.RoutingContext
// import java.net.URLDecoder
// import org.slf4j.MDC

// fun RoutingContext.success(any: Any?) {
pub fn success(this: &mut RoutingContext, any: Option<Any>) {
    let to_json = if any is JsonObject { any.to_string() } else { gson().to_json(any) };
    this.response()
        .put_header("content-type", "application/json; charset=utf-8")
        .end(to_json);
}

// fun RoutingContext.error(throwable: Throwable) {
pub fn error(this: &mut RoutingContext, throwable: Throwable) {
    let error = BasicError {
        error: "Internal Server Error".to_string(),
        exception: throwable.to_string(),
        message: throwable.message.to_string(),
        path: URLDecoder::decode(this.request().absolute_uri(), "UTF-8"),
        status: 500,
        timestamp: System::current_time_millis(),
    };
    let error_json = gson().to_json(&error);
    logger.error("Internal Server Error", throwable);
    logger.error(error_json.clone());
    this.response()
        .put_header("content-type", "application/json; charset=utf-8")
        .set_status_code(500)
        .end(error_json);
}

// fun Route.globalHandler(handler: Handler<RoutingContext>) {
pub fn global_handler(this: Route, handler: &dyn Fn(&mut RoutingContext)) {
    handler(|context: &mut RoutingContext| {
        let trace_id = context.get::<String>("traceId").take_if(|it| !it.is_empty()).unwrap_or_else(|| get_trace_id());
        MDC::put("traceId", &trace_id);
        context.put("traceId", trace_id);
        handler(context);
    });
}
