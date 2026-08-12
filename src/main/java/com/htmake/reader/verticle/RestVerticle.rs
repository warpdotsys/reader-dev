use crate::prelude::*;
// package com.htmake.reader.verticle

// fix: 显式导入，消除 prelude glob 重名歧义（Any 取 stubs 枚举；vertx 嵌套模块类型不在 glob 范围内）
use crate::stubs::Any;
use crate::stubs::MDCContext;
use crate::stubs::io::vertx::{
    AsyncResult, BodyHandler, Cookie, HttpServer, LocalSessionStore, LoggerFormat, LoggerHandler,
    Route, Router, RoutingContext, SessionHandler, URLDecoder, Vertx,
};

#[allow(non_upper_case_globals)]
static logger: Log = Log;

// fix: Kotlin `vertx`（ReaderApplication.vertx 伴生对象）→ 模块级常量
const vertx: Vertx = Vertx;

// import io.vertx.core.http.HttpMethod
// import io.vertx.ext.web.Route
// import io.vertx.ext.web.Router
// import io.vertx.ext.web.RoutingContext
// import io.vertx.ext.web.handler.BodyHandler
// import io.vertx.ext.web.handler.CorsHandler
// import io.vertx.ext.web.handler.LoggerFormat
// import io.vertx.ext.web.handler.LoggerHandler
// import io.vertx.ext.web.handler.SessionHandler
// import io.vertx.ext.web.sstore.LocalSessionStore
// import io.vertx.kotlin.coroutines.CoroutineVerticle
// import kotlinx.coroutines.Dispatchers
// import kotlinx.coroutines.Job
// import kotlinx.coroutines.launch
// import kotlinx.coroutines.slf4j.MDCContext
// import mu.KotlinLogging
// import com.htmake.reader.utils.error
// import com.htmake.reader.utils.globalHandler
// import com.htmake.reader.utils.success
// import com.htmake.reader.utils.toDir
// import java.net.URLDecoder

// private val logger = KotlinLogging.logger {}

// abstract class RestVerticle : CoroutineVerticle() {
pub struct RestVerticle {
    // protected lateinit var router: Router
    pub router: Option<Router>,
    // open var port: Int = 8080
    pub port: i32,
}

impl RestVerticle {
    pub fn new() -> RestVerticle {
        RestVerticle {
            router: None,
            port: 8080,
        }
    }

    // override suspend fun start() {
    pub async fn start(&mut self) {
        // fix: CoroutineVerticle.super.start() 无对应基类，移除 super 调用
        self.router = Some(Router::router(vertx));
        let cookie_name = "reader.session".to_string();
        let cookie_name2 = cookie_name.clone();
        self.router.as_mut().unwrap().route().global_handler(move |it| {
            SessionHandler::create(LocalSessionStore::create(vertx))
                .set_session_cookie_name(&cookie_name)
                .set_session_timeout(7 * 86400 * 1000)
                .set_session_cookie_path("/");
        });
        self.router.as_mut().unwrap().route().global_handler(move |it| {
            it.add_headers_end_handler(|_| {
                let cookie = it.get_cookie(&cookie_name2);
                if let Some(mut cookie) = cookie {
                    // 每次访问都延长cookie有效期
                    cookie.set_max_age(2 * 86400 * 1000);
                    cookie.set_path("/");
                }
            });
            it.next();
        });

        // CORS support
        self.router.as_mut().unwrap().route().global_handler(|it| {
            it.add_headers_end_handler(|_| {
                let origin = it.request().get_header("Origin");
                if let Some(origin) = origin {
                    if !origin.is_empty() {
                        let mut res = it.response();
                        res.put_header("Access-Control-Allow-Origin", &origin);
                        res.put_header("Access-Control-Allow-Credentials", "true");
                        res.put_header("Access-Control-Allow-Methods", "GET, POST, PATCH, PUT, DELETE");
                        res.put_header("Access-Control-Allow-Headers", "Authorization, Content-Type, If-Match, If-Modified-Since, If-None-Match, If-Unmodified-Since, X-Requested-With");
                    }
                }
            });
            let origin = it.request().get_header("Origin");
            if origin.is_some() && !origin.clone().unwrap().is_empty() && it.request().method() == HttpMethod::OPTIONS {
                it.remove_cookie("reader.session");
                success(it, Some(Any::Str(String::new())));
            } else {
                it.next();
            }
        });

        self.router.as_mut().unwrap().route().global_handler(|it| {
            BodyHandler::create();
        });

        self.router.as_mut().unwrap().route().global_handler(|it| {
            LoggerHandler::create(LoggerFormat::DEFAULT);
        });
        self.router.as_mut().unwrap().route_with_path("/reader3/*").global_handler(|it| {
            logger.info(format!("{} {}", it.request().raw_method(), URLDecoder::decode(it.request().absolute_uri(), "UTF-8")));
            if !it.request().raw_method().equals("PUT") && (it.file_uploads().is_empty()) && !it.body_as_string().is_empty() && it.body_as_string().len() < 1000 {
                logger.info(format!("Request body: {}", it.body_as_string()));
            }
            it.next();
        });

        self.router.as_mut().unwrap().get("/health").global_handler(|it| { success(it, Some(Any::Str("ok!".to_string()))); });

        // fix: 先取出 Router 再 await，避免 &mut self 与 &self 借用冲突
        let mut router = self.router.take().unwrap();
        self.init_router(&mut router).await;
        self.router = Some(router);

        //        router.errorHandler(500) { routerContext ->
        //            logger.error { routerContext.failure().message }
        //            routerContext.error(routerContext.failure())
        //        }

        self.router.as_mut().unwrap().route().last().failure_handler(|ctx| {
            if let Some(f) = ctx.failure() {
                error(ctx, f);
            }
        });

        let context_path = self.get_context_path();
        let main_router = if !context_path.is_empty() {
            // fix: .let { } 尾随 lambda 改为块
            let mut it = Router::router(vertx);
            it.mount_sub_router(to_dir(&context_path, true), self.router.clone().unwrap());
            it
        } else {
            self.router.clone().unwrap()
        };
        logger.info(format!("port: {}", self.port));
        vertx.create_http_server().request_handler(main_router).exception_handler(|error| {
            eprintln!("vertx exception: {}", error);
        }).listen(self.port, |res| {
            if res.succeeded() {
                println!("ReaderApplication Started");
            } else {
                eprintln!("Reader server start error");
            }
        });
    }

    // abstract fun getContextPath(): String
    pub fn get_context_path(&self) -> String {
        // fix: 抽象方法占位（子类未转录继承）
        String::new()
    }

    // abstract suspend fun initRouter(router: Router);
    pub async fn init_router(&self, _router: &mut Router) {
        // fix: 抽象方法占位（子类未转录继承）
    }

    // open fun onException(error: Throwable) {
    pub fn on_exception(&self, error: Throwable) {
        logger.error(format!("vertx exception: {}", error));
    }

    // open fun onStartError() {
    pub fn on_start_error(&self) {
    }

    // open fun started() {
    pub fn started(&self) {
    }

    // open fun onHandlerError(ctx: RoutingContext, error: Exception) {
    pub fn on_handler_error(&self, ctx: &mut RoutingContext, error: Exception) {
        logger.error(format!("Error: {}", error));
        // fix: 参数 `error` 遮蔽同名函数，使用全限定路径调用
        crate::com_htmake_reader_utils_vertroute::error(ctx, error);
    }

    /**
     * An extension method for simplifying coroutines usage with Vert.x Web routers
     */
    // fun Route.coroutineHandler(fn: suspend (RoutingContext) -> Any?) {
    pub fn coroutine_handler(&self, this: Route, fn_: &dyn Fn(RoutingContext) -> Option<Any>) {
        // fix: fn_ 为借用引用，transmute 到 'static 后转裸指针使闭包可 'static（fn_ 生命周期覆盖程序运行期）
        let fn_static: &'static dyn Fn(RoutingContext) -> Option<Any> =
            unsafe { std::mem::transmute(fn_) };
        let fn_ptr: *const dyn Fn(RoutingContext) -> Option<Any> = fn_static;
        global_handler(this, move |ctx| {
            let f = unsafe { &*fn_ptr };
            let result = f(ctx.clone());
            success(ctx, result);
        });
    }

    // fun Route.coroutineHandlerWithoutRes(fn: suspend (RoutingContext) -> Any?) {
    pub fn coroutine_handler_without_res(&self, this: Route, fn_: &dyn Fn(RoutingContext) -> Option<Any>) {
        let fn_static: &'static dyn Fn(RoutingContext) -> Option<Any> =
            unsafe { std::mem::transmute(fn_) };
        let fn_ptr: *const dyn Fn(RoutingContext) -> Option<Any> = fn_static;
        global_handler(this, move |ctx| {
            let f = unsafe { &*fn_ptr };
            let _ = f(ctx.clone());
        });
    }
}
