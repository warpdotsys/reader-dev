@file:JvmName("VertExtKt")

package com.htmake.reader.utils

import com.htmake.reader.entity.BasicError
import com.htmake.reader.verticle.sanitizeRequestTargetForLog
import io.vertx.core.Handler
import io.vertx.core.json.JsonObject
import io.vertx.ext.web.Route
import io.vertx.ext.web.RoutingContext
import java.net.URLDecoder
import org.slf4j.MDC

fun RoutingContext.success(any: Any?) {
    val toJson = if (any is JsonObject) any.toString() else gson.toJson(any)
    response()
        .putHeader("content-type", "application/json; charset=utf-8")
        .end(toJson)
}

fun RoutingContext.error(throwable: Throwable) {
    val error = BasicError(
        "Internal Server Error",
        throwable.toString(),
        throwable.message.toString(),
        URLDecoder.decode(request().absoluteURI(), "UTF-8"),
        500,
        System.currentTimeMillis()
    )
    val errorJson = gson.toJson(error)
    // Keep the legacy HTTP response, but never send its URI/message or the
    // original Throwable (including causes/suppressed exceptions) to a log sink.
    // Exception type + route + timestamp provide a bounded diagnostic record.
    val errorLog = gson.toJson(linkedMapOf(
        "error" to error.error,
        "exceptionType" to throwable.javaClass.name,
        "method" to request().method().name,
        "path" to sanitizeRequestTargetForLog(request().path()),
        "status" to error.status,
        "timestamp" to error.timestamp
    ))
    logger.error { errorLog }
    response()
        .putHeader("content-type", "application/json; charset=utf-8")
        .setStatusCode(500)
        .end(errorJson)
}

fun Route.globalHandler(handler: Handler<RoutingContext>) {
    handler { context ->
        val traceId = context.get<String>("traceId").takeUnless { it.isNullOrEmpty() } ?: getTraceId()
        MDC.put("traceId", traceId)
        context.put("traceId", traceId)
        handler.handle(context)
    }
}
