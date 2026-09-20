package com.htmake.reader.verticle

import io.vertx.core.http.HttpMethod
import io.vertx.ext.web.Route
import io.vertx.ext.web.Router
import io.vertx.ext.web.RoutingContext
import io.vertx.ext.web.handler.BodyHandler
import io.vertx.ext.web.handler.CorsHandler
import io.vertx.ext.web.handler.SessionHandler
import io.vertx.ext.web.sstore.LocalSessionStore
import io.vertx.kotlin.coroutines.CoroutineVerticle
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlinx.coroutines.slf4j.MDCContext
import mu.KotlinLogging
import com.htmake.reader.utils.error
import com.htmake.reader.utils.globalHandler
import com.htmake.reader.utils.success
import com.htmake.reader.utils.toDir
import java.util.concurrent.TimeUnit


private val logger = KotlinLogging.logger {}

fun sanitizeRequestTargetForLog(requestTarget: String): String {
    val path = requestTarget.substringBefore('?').substringBefore('#')
    return path.asSequence()
        .map { if (it == '\r' || it == '\n' || it == '\t') '_' else it }
        .take(2048)
        .joinToString("")
}

abstract class RestVerticle : CoroutineVerticle() {

    protected lateinit var router: Router

    open var port: Int = 8080

    override suspend fun start() {
        super.start()
        router = Router.router(vertx)
        val cookieName = "reader.session"
	    router.route().globalHandler(
            SessionHandler.create(LocalSessionStore.create(vertx))
                .setSessionCookieName(cookieName)
                .setSessionTimeout(7L * 86400 * 1000)
                .setSessionCookiePath("/")
        )
        router.route().globalHandler {
            it.addHeadersEndHandler { _ ->
                val cookie = it.getCookie(cookieName)
                if (cookie != null) {
                    // 每次访问都延长cookie有效期
                    cookie.setMaxAge(2L * 86400 * 1000)
                    cookie.setPath("/")
                }
            }
            it.next()
        }

        // CORS support
        router.route().globalHandler {
            it.addHeadersEndHandler { _ ->
                val origin = it.request().getHeader("Origin")
                if (origin != null && origin.isNotEmpty()) {
                    var res = it.response()
                    res.putHeader("Access-Control-Allow-Origin", origin)
                    res.putHeader("Access-Control-Allow-Credentials", "true")
                    res.putHeader("Access-Control-Allow-Methods", "GET, POST, PATCH, PUT, DELETE")
                    res.putHeader("Access-Control-Allow-Headers", "Authorization, Content-Type, If-Match, If-Modified-Since, If-None-Match, If-Unmodified-Since, X-Requested-With")
                }
            }
            val origin = it.request().getHeader("Origin")
            if (origin != null && origin.isNotEmpty() && it.request().method() == HttpMethod.OPTIONS) {
                it.removeCookie(cookieName)
                it.success("")
            } else {
                it.next()
            }
        }

        router.route().globalHandler(BodyHandler.create())

        router.route("/reader3/*").globalHandler {
            val startedAt = System.nanoTime()
            val method = it.request().rawMethod()
            val path = sanitizeRequestTargetForLog(it.request().path())
            it.addBodyEndHandler { _ ->
                val elapsedMillis = TimeUnit.NANOSECONDS.toMillis(System.nanoTime() - startedAt)
                logger.info("{} {} -> {} ({} ms)", method, path, it.response().statusCode, elapsedMillis)
            }
            it.next()
        }

        router.get("/health").globalHandler { it.success("ok!") }

        initRouter(router)

//        router.errorHandler(500) { routerContext ->
//            logger.error { routerContext.failure().message }
//            routerContext.error(routerContext.failure())
//        }

        router.route().last().failureHandler { ctx ->
            ctx.failure()?.let { ctx.error(it) }
        }

        val contextPath = getContextPath()
        val mainRouter = if (contextPath.isNotEmpty()) {
            Router.router(vertx).also { it.mountSubRouter(contextPath.toDir(true), router) }
        } else {
            router
        }
        logger.info("port: {}", port)
        vertx.createHttpServer().requestHandler(mainRouter).exceptionHandler { error ->
            onException(error)
        }.listen(port) { res ->
            if (res.succeeded()) {
                logger.info("Server running at: http://localhost:{}", port);
                logger.info("Web reader running at: http://localhost:{}", port);
                println("ReaderApplication Started")
                started();
            } else {
                onStartError();
            }
        }
    }

    abstract fun getContextPath(): String

    abstract suspend fun initRouter(router: Router);

    open fun onException(error: Throwable) {
        logger.error("vertx exception: {}", error)
    }

    open fun onStartError() {
    }

    open fun started() {

    }

    open fun onHandlerError(ctx: RoutingContext, error: Exception) {
        logger.error("Error: {}", error)
        ctx.error(error)
    }

    /**
     * An extension method for simplifying coroutines usage with Vert.x Web routers
     */
    fun Route.coroutineHandler(fn: suspend (RoutingContext) -> Any?) {
        globalHandler { ctx ->
            var job: Job? = null
            ctx.request().connection().closeHandler {
                logger.info("客户端已断开链接，终止运行")
                job?.cancel()
            }
            job = launch(MDCContext() + Dispatchers.IO) {
                try {
                    ctx.success(fn(ctx))
                } catch (e: Exception) {
                    onHandlerError(ctx, e)
                }
            }
        }
    }

    fun Route.coroutineHandlerWithoutRes(fn: suspend (RoutingContext) -> Any?) {
        globalHandler { ctx ->
            var job: Job? = null
            ctx.request().connection().closeHandler {
                logger.info("客户端已断开链接，终止运行")
                job?.cancel()
            }
            job = launch(MDCContext() + Dispatchers.IO) {
                try {
                    fn(ctx)
                } catch (e: Exception) {
                    onHandlerError(ctx, e)
                }
            }
        }
    }
}
