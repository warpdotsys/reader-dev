package com.htmake.reader.verticle;

import ch.qos.logback.classic.Logger;
import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.core.read.ListAppender;
import com.htmake.reader.api.YueduApi;
import com.htmake.reader.api.ReturnData;
import com.htmake.reader.utils.ExtKt;
import io.vertx.core.http.HttpMethod;
import io.vertx.core.http.HttpServerRequest;
import io.vertx.core.http.HttpServerResponse;
import io.vertx.core.Vertx;
import io.vertx.core.json.JsonObject;
import io.vertx.ext.web.Router;
import io.vertx.ext.web.RoutingContext;
import java.net.HttpURLConnection;
import java.net.Proxy;
import java.net.URL;
import java.nio.charset.StandardCharsets;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import org.junit.Test;
import org.mockito.ArgumentCaptor;
import org.slf4j.LoggerFactory;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertTrue;
import static org.mockito.ArgumentMatchers.anyInt;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.mockito.Mockito.CALLS_REAL_METHODS;
import static org.mockito.Mockito.never;

public class RestVerticleLoggingTest {

    @Test
    public void queryParametersAreNeverIncludedInRequestLogs() {
        String target = "/reader3/getBookshelf?accessToken=user:secret&secureKey=secret";
        String sanitized = RestVerticleKt.sanitizeRequestTargetForLog(target);

        assertEquals("/reader3/getBookshelf", sanitized);
        assertFalse(sanitized.contains("accessToken"));
        assertFalse(sanitized.contains("secret"));
    }

    @Test
    public void controlCharactersCannotInjectExtraLogLines() {
        String sanitized = RestVerticleKt.sanitizeRequestTargetForLog(
                "/reader3/getUserInfo\r\nforged-log-entry\tvalue");

        assertEquals("/reader3/getUserInfo__forged-log-entry_value", sanitized);
        assertFalse(sanitized.contains("\r"));
        assertFalse(sanitized.contains("\n"));
        assertFalse(sanitized.contains("\t"));
    }

    @Test
    public void errorSinkOmitsQueriesMessagesCausesSuppressedAndStackDetails() {
        IllegalStateException failure = new IllegalStateException(
                "synthetic-message-secret", new IllegalArgumentException("synthetic-cause-secret"));
        failure.addSuppressed(new RuntimeException("synthetic-suppressed-secret"));
        failure.setStackTrace(new StackTraceElement[] {
                new StackTraceElement("Synthetic", "test", "synthetic-stack-secret", 7)
        });
        checkErrorSink(failure, "/reader3/generated?accessToken=synthetic-query-secret&secureKey=synthetic-key-secret",
                "/reader3/generated", "synthetic-message-secret");
    }

    @Test
    public void errorSinkBoundsPathAndHandlesNullExceptionMessage() {
        String path = "/reader3/" + new String(new char[2100]).replace('\0', 'x') + "\r\n\t";
        checkErrorSink(new IllegalStateException(), path + "?accessToken=synthetic-query-secret",
                path.substring(0, 2048), "null");
    }

    @Test
    public void errorSinkEscapesRouteControlsAndDoesNotTraverseCyclicCauses() {
        IllegalStateException failure = new IllegalStateException("synthetic-message-secret");
        IllegalArgumentException cause = new IllegalArgumentException("synthetic-cause-secret", failure);
        failure.initCause(cause);
        checkErrorSink(failure, "/reader3/generated\r\n\t?accessToken=synthetic-query-secret",
                "/reader3/generated___", "synthetic-message-secret");
    }

    @Test(timeout = 20000)
    public void actualLoopbackHttpFailureKeepsResponseButDoesNotLogItsToken() throws Exception {
        checkActualHttpFailure(false);
    }

    @Test(timeout = 20000)
    public void actualLoopbackBusinessFailureKeepsReturnDataAndDoesNotLogItsToken() throws Exception {
        checkActualHttpFailure(true);
    }

    private void checkActualHttpFailure(boolean businessHandler) throws Exception {
        Vertx vertx = Vertx.vertx();
        HttpURLConnection connection = null;
        Logger root = (Logger) LoggerFactory.getLogger(Logger.ROOT_LOGGER_NAME);
        ListAppender<ILoggingEvent> appender = new ListAppender<>();
        appender.start();
        root.addAppender(appender);
        try {
            Router router = Router.router(vertx);
            RestVerticle baseHandler = businessHandler ? new YueduApi() : mock(RestVerticle.class, CALLS_REAL_METHODS);
            router.post("/reader3/generated-error").handler(context -> baseHandler.onHandlerError(context,
                    new IllegalStateException("synthetic-http-message-secret")));
            CompletableFuture<Integer> listening = new CompletableFuture<>();
            vertx.createHttpServer().requestHandler(router).listen(0, "127.0.0.1", result -> {
                if (result.succeeded()) listening.complete(result.result().actualPort());
                else listening.completeExceptionally(result.cause());
            });
            String uri = "http://127.0.0.1:" + listening.get(5, TimeUnit.SECONDS)
                    + "/reader3/generated-error?accessToken=synthetic-http-query-secret";
            connection = (HttpURLConnection) new URL(uri).openConnection(Proxy.NO_PROXY);
            connection.setConnectTimeout(5000);
            connection.setReadTimeout(5000);
            connection.setRequestMethod("POST");
            assertEquals(businessHandler ? 200 : 500, connection.getResponseCode());
            assertEquals("application/json; charset=utf-8", connection.getContentType());
            JsonObject response = new JsonObject(new String((businessHandler ? connection.getInputStream() :
                    connection.getErrorStream()).readAllBytes(),
                    StandardCharsets.UTF_8));
            if (businessHandler) {
                assertEquals(new JsonObject(ExtKt.getGson().toJson(new ReturnData().setErrorMsg(
                        "java.lang.IllegalStateException: synthetic-http-message-secret"))), response);
            } else {
                assertEquals(uri, response.getString("path"));
                assertEquals("synthetic-http-message-secret", response.getString("message"));
                assertEquals(6, response.size());
            }
            int safeErrors = 0;
            // Close waits for the event loop, so the captured events cannot race
            // the assertions below. No existing Reader process is involved.
            CompletableFuture<Void> closed = new CompletableFuture<>();
            vertx.close(result -> {
                if (result.succeeded()) closed.complete(null);
                else closed.completeExceptionally(result.cause());
            });
            closed.get(5, TimeUnit.SECONDS);
            for (ILoggingEvent event : appender.list) {
                String record = event.getFormattedMessage();
                assertFalse(record.contains("synthetic-http-query-secret"));
                assertFalse(record.contains("synthetic-http-message-secret"));
                assertNull("the caller must not log a raw Throwable either", event.getThrowableProxy());
                if (record.startsWith("{\"error\":")) {
                    JsonObject logged = new JsonObject(record);
                    assertEquals("/reader3/generated-error", logged.getString("path"));
                    assertEquals("POST", logged.getString("method"));
                    assertEquals(Integer.valueOf(businessHandler ? 200 : 500), logged.getInteger("status"));
                    if (businessHandler) assertTrue(logged.getLong("timestamp") > 0);
                    else assertEquals(response.getLong("timestamp"), logged.getLong("timestamp"));
                    safeErrors++;
                }
            }
            assertEquals(1, safeErrors);
        } finally {
            if (connection != null) connection.disconnect();
            root.detachAppender(appender);
            appender.stop();
            vertx.close();
        }
    }

    @Test
    public void businessFailureKeepsLegacyReturnDataAndOmitsSensitiveExceptionDetails() {
        checkBusinessFailure(false);
    }

    @Test
    public void alreadyCommittedBusinessFailureKeepsOldBodyWithoutChangingHeadersOrStatus() {
        checkBusinessFailure(true);
    }

    private void checkBusinessFailure(boolean committed) {
        RoutingContext context = mock(RoutingContext.class);
        HttpServerRequest request = mock(HttpServerRequest.class);
        HttpServerResponse response = mock(HttpServerResponse.class);
        when(context.request()).thenReturn(request);
        when(context.response()).thenReturn(response);
        when(request.path()).thenReturn("/reader3/generated?accessToken=synthetic-query-secret");
        when(request.method()).thenReturn(HttpMethod.POST);
        when(response.headWritten()).thenReturn(committed);
        when(response.getStatusCode()).thenReturn(committed ? 206 : 200);
        when(response.putHeader(anyString(), anyString())).thenReturn(response);
        IllegalStateException failure = new IllegalStateException("synthetic-message-secret",
                new IllegalArgumentException("synthetic-cause-secret"));
        failure.addSuppressed(new RuntimeException("synthetic-suppressed-secret"));
        Logger root = (Logger) LoggerFactory.getLogger(Logger.ROOT_LOGGER_NAME);
        ListAppender<ILoggingEvent> appender = new ListAppender<>();
        appender.start();
        root.addAppender(appender);
        try {
            new YueduApi().onHandlerError(context, failure);
            assertEquals(1, appender.list.size());
            ILoggingEvent event = appender.list.get(0);
            assertNull(event.getThrowableProxy());
            String record = event.getFormattedMessage();
            for (String secret : new String[] {"synthetic-query-secret", "synthetic-message-secret",
                    "synthetic-cause-secret", "synthetic-suppressed-secret", "accessToken"}) {
                assertFalse(secret, record.contains(secret));
            }
            JsonObject logged = new JsonObject(record);
            assertEquals(6, logged.size());
            assertEquals("Request Handler Error", logged.getString("error"));
            assertEquals("/reader3/generated", logged.getString("path"));
            assertEquals(Integer.valueOf(committed ? 206 : 200), logged.getInteger("status"));
            assertTrue(logged.getLong("timestamp") > 0);
            ArgumentCaptor<String> body = ArgumentCaptor.forClass(String.class);
            verify(response).end(body.capture());
            if (committed) {
                assertEquals(failure.toString(), body.getValue());
                verify(response, never()).putHeader(anyString(), anyString());
            } else {
                assertEquals(ExtKt.getGson().toJson(new ReturnData().setErrorMsg(failure.toString())), body.getValue());
                verify(response).putHeader("content-type", "application/json; charset=utf-8");
            }
            verify(response, never()).setStatusCode(anyInt());
        } finally {
            root.detachAppender(appender);
            appender.stop();
        }
    }

    private void checkErrorSink(Throwable failure, String target, String safePath, String oldMessage) {
        RoutingContext context = mock(RoutingContext.class);
        HttpServerRequest request = mock(HttpServerRequest.class);
        HttpServerResponse response = mock(HttpServerResponse.class);
        String uri = "http://127.0.0.1" + target;
        when(context.request()).thenReturn(request);
        when(context.response()).thenReturn(response);
        when(request.absoluteURI()).thenReturn(uri);
        when(request.path()).thenReturn(target.substring(0, target.indexOf('?')));
        when(request.method()).thenReturn(HttpMethod.POST);
        when(response.putHeader(anyString(), anyString())).thenReturn(response);
        when(response.setStatusCode(anyInt())).thenReturn(response);

        Logger root = (Logger) LoggerFactory.getLogger(Logger.ROOT_LOGGER_NAME);
        ListAppender<ILoggingEvent> appender = new ListAppender<>();
        appender.start();
        root.addAppender(appender);
        try {
            mock(RestVerticle.class, CALLS_REAL_METHODS).onHandlerError(context, (Exception) failure);
            assertEquals("one safe error record, not a second raw Throwable record", 1, appender.list.size());
            ILoggingEvent event = appender.list.get(0);
            assertNull("causes and suppressed messages must not reach the sink", event.getThrowableProxy());
            String record = event.getFormattedMessage();
            for (String secret : new String[] {"synthetic-query-secret", "synthetic-key-secret",
                    "synthetic-message-secret", "synthetic-cause-secret", "synthetic-suppressed-secret",
                    "synthetic-stack-secret", "accessToken", "secureKey"}) {
                assertFalse(secret, record.contains(secret));
            }
            assertFalse(record.contains("\r"));
            assertFalse(record.contains("\n"));
            assertFalse(record.contains("\t"));
            JsonObject log = new JsonObject(record);
            assertEquals(6, log.size());
            assertEquals("Internal Server Error", log.getString("error"));
            assertEquals(failure.getClass().getName(), log.getString("exceptionType"));
            assertEquals("POST", log.getString("method"));
            assertEquals(safePath, log.getString("path"));
            assertEquals(Integer.valueOf(500), log.getInteger("status"));
            assertTrue(log.getLong("timestamp") > 0);

            // Compatibility assertion: only logging changes. The old HTTP body
            // intentionally still has its six fields, exception/message and URI.
            ArgumentCaptor<String> body = ArgumentCaptor.forClass(String.class);
            verify(response).end(body.capture());
            verify(response).setStatusCode(500);
            verify(response).putHeader("content-type", "application/json; charset=utf-8");
            JsonObject error = new JsonObject(body.getValue());
            assertEquals(6, error.size());
            assertEquals("Internal Server Error", error.getString("error"));
            assertEquals(failure.toString(), error.getString("exception"));
            assertEquals(oldMessage, error.getString("message"));
            assertEquals(uri, error.getString("path"));
            assertEquals(Integer.valueOf(500), error.getInteger("status"));
            assertEquals(log.getLong("timestamp"), error.getLong("timestamp"));
        } finally {
            root.detachAppender(appender);
            appender.stop();
        }
    }
}
