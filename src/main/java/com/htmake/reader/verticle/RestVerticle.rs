// package com.htmake.reader.verticle

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
        super::start().await;
        self.router = Some(Router::router(vertx));
        let cookie_name = "reader.session";
        self.router.as_mut().unwrap().route().global_handler(&|it| {
            SessionHandler::create(LocalSessionStore::create(vertx))
                .set_session_cookie_name(cookie_name)
                .set_session_timeout(7 * 86400 * 1000)
                .set_session_cookie_path("/");
        });
        self.router.as_mut().unwrap().route().global_handler(&|it| {
            it.add_headers_end_handler(|_| {
                let cookie = it.get_cookie(cookie_name);
                if cookie.is_some() {
                    // 每次访问都延长cookie有效期
                    cookie.set_max_age(2 * 86400 * 1000);
                    cookie.set_path("/");
                }
            });
            it.next();
        });

        // CORS support
        self.router.as_mut().unwrap().route().global_handler(&|it| {
            it.add_headers_end_handler(|_| {
                let origin = it.request().get_header("Origin");
                if origin.is_some() && !origin.is_empty() {
                    let res = it.response();
                    res.put_header("Access-Control-Allow-Origin", origin);
                    res.put_header("Access-Control-Allow-Credentials", "true");
                    res.put_header("Access-Control-Allow-Methods", "GET, POST, PATCH, PUT, DELETE");
                    res.put_header("Access-Control-Allow-Headers", "Authorization, Content-Type, If-Match, If-Modified-Since, If-None-Match, If-Unmodified-Since, X-Requested-With");
                }
            });
            let origin = it.request().get_header("Origin");
            if origin.is_some() && !origin.is_empty() && it.request().method() == HttpMethod::OPTIONS {
                it.remove_cookie(cookie_name);
                success(it, Some(""));
            } else {
                it.next();
            }
        });

        self.router.as_mut().unwrap().route().global_handler(&|it| {
            BodyHandler::create();
        });

        self.router.as_mut().unwrap().route().global_handler(&|it| {
            LoggerHandler::create(LoggerFormat::DEFAULT);
        });
        self.router.as_mut().unwrap().route("/reader3/*").global_handler(&|it| {
            logger.info(format!("{} {}", it.request().raw_method(), URLDecoder::decode(it.request().absolute_uri(), "UTF-8")));
            if !it.request().raw_method().equals("PUT") && (it.file_uploads().is_empty()) && !it.body_as_string().is_empty() && it.body_as_string().len() < 1000 {
                logger.info(format!("Request body: {}", it.body_as_string()));
            }
            it.next();
        });

        self.router.as_mut().unwrap().get("/health").global_handler(&|it| { success(it, Some("ok!")); });

        self.init_router(self.router.as_mut().unwrap());

        //        router.errorHandler(500) { routerContext ->
        //            logger.error { routerContext.failure().message }
        //            routerContext.error(routerContext.failure())
        //        }

        self.router.as_mut().unwrap().route().last().failure_handler(&|ctx| {
            if let Some(f) = ctx.failure() {
                error(ctx, f);
            }
        });

        let context_path = self.get_context_path();
        let main_router = if !context_path.is_empty() {
            Router::router(vertx).let(|it| { it.mount_sub_router(to_dir(&context_path, true), self.router.clone().unwrap()); it })
        } else {
            self.router.clone().unwrap()
        };
        logger.info(format!("port: {}", self.port));
        vertx.create_http_server().request_handler(main_router).exception_handler(|error| {
            self.on_exception(error);
        }).listen(self.port, |res| {
            if res.succeeded() {
                logger.info(format!("Server running at: http://localhost:{}", self.port));
                logger.info(format!("Web reader running at: http://localhost:{}", self.port));
                println!("ReaderApplication Started");
                self.started();
            } else {
                self.on_start_error();
            }
        });
    }

    // abstract fun getContextPath(): String
    pub fn get_context_path(&self) -> String {
        todo!()
    }

    // abstract suspend fun initRouter(router: Router);
    pub async fn init_router(&self, router: &mut Router) {
        todo!()
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
        error(ctx, error);
    }

    /**
     * An extension method for simplifying coroutines usage with Vert.x Web routers
     */
    // fun Route.coroutineHandler(fn: suspend (RoutingContext) -> Any?) {
    pub fn coroutine_handler(&self, this: Route, fn_: &dyn Fn(RoutingContext) -> Option<Any>) {
        global_handler(this, &|ctx| {
            let mut job: Option<Job> = None;
            ctx.request().connection().close_handler(|| {
                logger.info("客户端已断开链接，终止运行");
                if let Some(j) = job.take() {
                    j.cancel();
                }
            });
            job = Some(launch(MDCContext::new() + Dispatchers::IO, async {
                try {
                    success(ctx, fn_(ctx.clone()));
                } catch (e: Exception) {
                    self.on_handler_error(ctx, e);
                }
            }));
        });
    }

    // fun Route.coroutineHandlerWithoutRes(fn: suspend (RoutingContext) -> Any?) {
    pub fn coroutine_handler_without_res(&self, this: Route, fn_: &dyn Fn(RoutingContext) -> Option<Any>) {
        global_handler(this, &|ctx| {
            let mut job: Option<Job> = None;
            ctx.request().connection().close_handler(|| {
                logger.info("客户端已断开链接，终止运行");
                if let Some(j) = job.take() {
                    j.cancel();
                }
            });
            job = Some(launch(MDCContext::new() + Dispatchers::IO, async {
                try {
                    fn_(ctx.clone());
                } catch (e: Exception) {
                    self.on_handler_error(ctx, e);
                }
            }));
        });
    }
}
