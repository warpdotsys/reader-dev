package com.htmake.reader.api

import io.legado.app.data.entities.Book
import io.legado.app.data.entities.BookChapter
import io.legado.app.data.entities.SearchBook
import io.legado.app.data.entities.BookGroup
import io.legado.app.data.entities.BookSource
import io.legado.app.data.entities.RssSource
import io.legado.app.data.entities.RssArticle
import io.legado.app.model.webBook.WebBook
import io.vertx.ext.web.Router
import io.vertx.ext.web.RoutingContext
import io.vertx.ext.web.handler.StaticHandler;
import io.vertx.core.net.impl.URIDecoder
import mu.KotlinLogging
import com.htmake.reader.config.AppConfig
import com.htmake.reader.config.BookConfig
import io.legado.app.constant.DeepinkBookSource
import com.htmake.reader.api.controller.BookController
import com.htmake.reader.api.controller.BookSourceController
import com.htmake.reader.api.controller.RssSourceController
import com.htmake.reader.api.controller.UserController
import com.htmake.reader.api.controller.WebdavController
import com.htmake.reader.api.controller.ReplaceRuleController
import com.htmake.reader.api.controller.BookmarkController
import com.htmake.reader.api.controller.BookGroupController
import com.htmake.reader.api.controller.FileController
import com.htmake.reader.api.controller.HttpTTSController
import com.htmake.reader.utils.error
import com.htmake.reader.utils.success
import com.htmake.reader.utils.getStorage
import com.htmake.reader.utils.saveStorage
import com.htmake.reader.utils.asJsonArray
import com.htmake.reader.utils.asJsonObject
import com.htmake.reader.utils.toDataClass
import com.htmake.reader.utils.toMap
import com.htmake.reader.utils.fillData
import com.htmake.reader.utils.getWorkDir
import com.htmake.reader.utils.getRandomString
import com.htmake.reader.utils.genEncryptedPassword
import com.htmake.reader.entity.User
import com.htmake.reader.utils.SpringContextUtils
import com.htmake.reader.utils.deleteRecursively
import com.htmake.reader.utils.unzip
import com.htmake.reader.utils.zip
import com.htmake.reader.utils.jsonEncode
import com.htmake.reader.utils.getRelativePath
import com.htmake.reader.utils.RemoteWebview
import com.htmake.reader.utils.getTraceId
import com.htmake.reader.init.ReaderAdapter
import io.legado.app.adapters.ReaderAdapterHelper
import com.htmake.reader.verticle.RestVerticle
import com.htmake.reader.SpringEvent
import org.springframework.stereotype.Component
import io.vertx.core.json.JsonObject
import io.vertx.core.json.JsonArray
import io.vertx.core.http.HttpMethod
import com.htmake.reader.api.ReturnData
import io.legado.app.utils.MD5Utils
import java.net.URLDecoder;
import java.net.URLEncoder;
import java.net.URL;
import java.util.UUID;
import io.vertx.ext.web.client.WebClient
import org.springframework.beans.factory.annotation.Autowired
import org.springframework.core.env.Environment
import java.io.File
import java.lang.Runtime
import kotlin.collections.mutableMapOf
import kotlin.system.measureTimeMillis
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlin.random.Random
import java.text.SimpleDateFormat;
import io.legado.app.utils.EncoderUtils
import io.legado.app.model.rss.Rss
import org.springframework.scheduling.annotation.Scheduled
import io.legado.app.model.localBook.LocalBook
import java.nio.file.Paths
import kotlinx.coroutines.withContext
import kotlinx.coroutines.async
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.slf4j.MDCContext
import org.slf4j.MDC
import java.util.Calendar

private val logger = KotlinLogging.logger {}

@Component
class YueduApi : RestVerticle() {
    @Autowired
    private lateinit var appConfig: AppConfig

    @Autowired
    private lateinit var env: Environment

    override fun getContextPath(): String {
        return env.getProperty("reader.server.contextPath", "") ?: ""
    }

    override suspend fun initRouter(router: Router) {
        setupPort()

        if (appConfig.mongoUri.isNotEmpty()) {
            com.htmake.reader.utils.MongoManager.connect(appConfig.mongoUri)
        }

        if (appConfig.remoteWebviewApi.isNotEmpty()) {
            RemoteWebview.setRemoteApi(appConfig.remoteWebviewApi)
        }
        ReaderAdapterHelper.setAdapter(ReaderAdapter)

        // 旧版数据迁移
        migration()

        // web界面
        router.route("/*").handler(StaticHandler.create("web").setDefaultContentEncoding("UTF-8"));

        // assets
        var assetsDir = getWorkDir("storage", "assets");
        var assetsDirFile = File(assetsDir);
        if (!assetsDirFile.exists()) {
            assetsDirFile.mkdirs();
        }
        var assetsCss = getWorkDir("storage", "assets", "reader.css");
        var assetsCssFile = File(assetsCss);
        if (!assetsCssFile.exists()) {
            assetsCssFile.writeText("/* 在此处可以编写CSS样式来自定义页面 */");
        }
        router.route("/assets/*").handler(StaticHandler.create().setAllowRootFileSystemAccess(true).setWebRoot(assetsDir).setDefaultContentEncoding("UTF-8"));

        // 书籍资源
        var dataDir = getWorkDir("storage", "data");
        router.route("/book-assets/*").handler {
            var path = it.request().path().replace("/book-assets/", "/", true)
            path = URIDecoder.decodeURIComponent(path, false)
            if ((path.endsWith("html", true) || path.endsWith("htm", true))) {
                val filePath = File(dataDir + path)
                if (filePath.exists()) {
                    BookConfig.injectJavascriptToEpubChapter(filePath.toString())
                }
            }
            it.next()
        }
        router.route("/book-assets/*").handler(StaticHandler.create().setAllowRootFileSystemAccess(true).setWebRoot(dataDir).setDefaultContentEncoding("UTF-8"))

        // epub资源
        router.route("/epub/*").handler {
            var path = it.request().path().replace("/epub/", "/", true)
            path = URLDecoder.decode(path, "UTF-8")
            if (path.endsWith("html", true)) {
                var filePath = File(dataDir + path)
                if (filePath.exists()) {
                    // 处理 js 注入脚本
                    BookConfig.injectJavascriptToEpubChapter(filePath.toString())
                }
            }
            it.next()
        }
        router.route("/epub/*").handler(StaticHandler.create().setAllowRootFileSystemAccess(true).setWebRoot(dataDir).setDefaultContentEncoding("UTF-8"));

        // simple-web界面
        router.route("/simple-web").handler {
            if (it.request().path().endsWith("/simple-web")) {
                val location = URLDecoder.decode(it.request().absoluteURI(), "UTF-8")
                    .replace("/simple-web", "/simple-web/", false)
                it.response().putHeader("Location", location).setStatusCode(302).end()
            } else {
                it.next()
            }
        }
        router.route("/simple-web/*").handler {
            it.next()
        }
        router.route("/simple-web/*").handler(StaticHandler.create("simple-web").setDefaultContentEncoding("UTF-8"))

        // 获取系统信息
        router.get("/reader3/getSystemInfo").coroutineHandler { getSystemInfo(it) }

        ////////// 接口部分
        val bookController = BookController(coroutineContext)
        val bookSourceController = BookSourceController(coroutineContext)
        val rssSourceController = RssSourceController(coroutineContext)
        val userController = UserController(coroutineContext)
        val webdavController = WebdavController(coroutineContext, router) { ctx, error ->
            onHandlerError(ctx, error)
        }
        val replaceRuleController = ReplaceRuleController(coroutineContext)
        val bookmarkController = BookmarkController(coroutineContext)
        val bookGroupController = BookGroupController(coroutineContext)
        val fileController = FileController(coroutineContext)
        val httpTTSController = HttpTTSController(coroutineContext)

        /** 书源模块 */
        router.post("/reader3/saveBookSource").coroutineHandler { bookSourceController.saveBookSource(it) }
        router.post("/reader3/saveBookSources").coroutineHandler { bookSourceController.saveBookSources(it) }

        router.get("/reader3/getBookSource").coroutineHandler { bookSourceController.getBookSource(it) }
        router.post("/reader3/getBookSource").coroutineHandler { bookSourceController.getBookSource(it) }
        router.get("/reader3/getBookSources").coroutineHandler { bookSourceController.getBookSources(it) }
        router.post("/reader3/getBookSources").coroutineHandler { bookSourceController.getBookSources(it) }

        router.post("/reader3/deleteAllBookSources").coroutineHandler { bookSourceController.deleteAllBookSources(it) }
        router.post("/reader3/deleteBookSource").coroutineHandler { bookSourceController.deleteBookSource(it) }
        router.post("/reader3/deleteBookSources").coroutineHandler { bookSourceController.deleteBookSources(it) }

        // 上传书源文件
        router.post("/reader3/readSourceFile").coroutineHandler { bookSourceController.readSourceFile(it) }

        router.post("/reader3/saveFromRemoteSource").coroutineHandlerWithoutRes { bookSourceController.saveFromRemoteSource(it) }

        // 设置默认书源
        router.post("/reader3/setAsDefaultBookSources").coroutineHandler { bookSourceController.setAsDefaultBookSources(it) }
        router.post("/reader3/deleteUserBookSource").coroutineHandler { bookSourceController.deleteUserBookSource(it) }
        router.post("/reader3/deleteBookSourcesFile").coroutineHandler { bookSourceController.deleteBookSourcesFile(it) }

        /** 书籍模块 */
        // 书架
        router.get("/reader3/getBookshelf").coroutineHandler { bookController.getBookshelf(it) }
        router.get("/reader3/getShelfBook").coroutineHandler { bookController.getShelfBook(it) }
        router.post("/reader3/saveBook").coroutineHandler { bookController.saveBook(it) }
        router.post("/reader3/deleteBook").coroutineHandler { bookController.deleteBook(it) }
        router.post("/reader3/deleteBooks").coroutineHandler { bookController.deleteBooks(it) }

        // 失效书源
        router.post("/reader3/getInvalidBookSources").coroutineHandler { bookController.getInvalidBookSources(it) }

        // 探索
        router.post("/reader3/exploreBook").coroutineHandler { bookController.exploreBook(it) }
        router.get("/reader3/exploreBook").coroutineHandler { bookController.exploreBook(it) }

        // 搜索
        router.get("/reader3/searchBook").coroutineHandler { bookController.searchBook(it) }
        router.post("/reader3/searchBook").coroutineHandler { bookController.searchBook(it) }
        router.get("/reader3/searchBookMulti").coroutineHandler { bookController.searchBookMulti(it) }
        router.post("/reader3/searchBookMulti").coroutineHandler { bookController.searchBookMulti(it) }
        router.get("/reader3/searchBookMultiSSE").coroutineHandlerWithoutRes { bookController.searchBookMultiSSE(it) }

        // 书籍详情
        router.get("/reader3/getBookInfo").coroutineHandler { bookController.getBookInfo(it) }
        router.post("/reader3/getBookInfo").coroutineHandler { bookController.getBookInfo(it) }

        // 章节列表
        router.get("/reader3/getChapterList").coroutineHandler { bookController.getChapterList(it) }
        router.post("/reader3/getChapterList").coroutineHandler { bookController.getChapterList(it) }

        // 内容
        router.get("/reader3/getBookContent").coroutineHandler { bookController.getBookContent(it) }
        router.post("/reader3/getBookContent").coroutineHandler { bookController.getBookContent(it) }

        // 保存阅读进度
        router.post("/reader3/saveBookProgress").coroutineHandler { bookController.saveBookProgress(it) }

        // 封面
        router.get("/reader3/cover").coroutineHandlerWithoutRes { bookController.getBookCover(it) }

        // 搜索其它来源
        router.get("/reader3/searchBookSource").coroutineHandler { bookController.searchBookSource(it) }
        router.post("/reader3/searchBookSource").coroutineHandler { bookController.searchBookSource(it) }
        router.get("/reader3/getAvailableBookSource").coroutineHandler { bookController.getAvailableBookSource(it) }
        router.post("/reader3/getAvailableBookSource").coroutineHandler { bookController.getAvailableBookSource(it) }
        router.get("/reader3/searchBookSourceSSE").coroutineHandlerWithoutRes { bookController.searchBookSourceSSE(it) }

        // 换源
        router.get("/reader3/setBookSource").coroutineHandler { bookController.setBookSource(it) }
        router.post("/reader3/setBookSource").coroutineHandler { bookController.setBookSource(it) }

        // 修改分组
        router.post("/reader3/saveBookGroupId").coroutineHandler { bookController.saveBookGroupId(it) }
        router.post("/reader3/addBookGroupMulti").coroutineHandler { bookController.addBookGroupMulti(it) }
        router.post("/reader3/removeBookGroupMulti").coroutineHandler { bookController.removeBookGroupMulti(it) }

        // 导入本地文件
        router.post("/reader3/importBookPreview").coroutineHandler { bookController.importBookPreview(it) }
        router.post("/reader3/refreshLocalBook").coroutineHandler { bookController.refreshLocalBook(it) }

        // 获取txt章节规则
        router.get("/reader3/getTxtTocRules").coroutineHandler { bookController.getTxtTocRules(it) }
        router.post("/reader3/getChapterListByRule").coroutineHandler { bookController.getChapterListByRule(it) }

        // 书籍分组
        router.get("/reader3/getBookGroups").coroutineHandler { bookGroupController.getBookGroups(it) }
        router.post("/reader3/saveBookGroup").coroutineHandler { bookGroupController.saveBookGroup(it) }
        router.post("/reader3/deleteBookGroup").coroutineHandler { bookGroupController.deleteBookGroup(it) }
        router.post("/reader3/saveBookGroupOrder").coroutineHandler { bookGroupController.saveBookGroupOrder(it) }

        // 调试书源
        router.get("/reader3/bookSourceDebugSSE").coroutineHandlerWithoutRes { bookController.bookSourceDebugSSE(it) }

        // 缓存书籍章节
        router.get("/reader3/cacheBookSSE").coroutineHandlerWithoutRes { bookController.cacheBookSSE(it) }
        // 获取书籍缓存信息
        router.get("/reader3/getShelfBookWithCacheInfo").coroutineHandler { bookController.getShelfBookWithCacheInfo(it) }
        // 删除书籍章节缓存
        router.post("/reader3/deleteBookCache").coroutineHandler { bookController.deleteBookCache(it) }

        // 导出书籍
        router.post("/reader3/exportBook").coroutineHandlerWithoutRes { bookController.exportBook(it) }
        router.get("/reader3/exportBook").coroutineHandlerWithoutRes { bookController.exportBook(it) }

        // 全文搜索
        router.get("/reader3/searchBookContent").coroutineHandler { bookController.searchBookContent(it) }
        router.post("/reader3/searchBookContent").coroutineHandler { bookController.searchBookContent(it) }

        /** 用户模块 */
        // 上传文件
        router.post("/reader3/uploadFile").coroutineHandler { userController.uploadFile(it) }

        // 删除文件
        router.post("/reader3/deleteFile").coroutineHandler { userController.deleteFile(it) }

        // 登录
        router.post("/reader3/login").coroutineHandler { userController.login(it) }
        // 注销登录
        router.post("/reader3/logout").coroutineHandler { userController.logout(it) }

        // 获取用户信息
        router.get("/reader3/getUserInfo").coroutineHandler { userController.getUserInfo(it) }

        // 用户备份本地配置
        router.post("/reader3/saveUserConfig").coroutineHandler { userController.saveUserConfig(it) }

        // 用户恢复本地配置
        router.get("/reader3/getUserConfig").coroutineHandler { userController.getUserConfig(it) }

        // 获取用户列表
        router.get("/reader3/getUserList").coroutineHandler { userController.getUserList(it) }

        // 删除用户
        router.post("/reader3/deleteUsers").coroutineHandler { userController.deleteUsers(it) }

        // 添加用户
        router.post("/reader3/addUser").coroutineHandler { userController.addUser(it) }

        // 重置用户密码
        router.post("/reader3/resetPassword").coroutineHandler { userController.resetPassword(it) }

        // 更新用户
        router.post("/reader3/updateUser").coroutineHandler { userController.updateUser(it) }


        /** rss模块 */
        // rss
        router.get("/reader3/getRssSources").coroutineHandler { rssSourceController.getRssSources(it) }
        router.post("/reader3/saveRssSource").coroutineHandler { rssSourceController.saveRssSource(it) }
        router.post("/reader3/saveRssSources").coroutineHandler { rssSourceController.saveRssSources(it) }
        router.post("/reader3/deleteRssSource").coroutineHandler { rssSourceController.deleteRssSource(it) }
        // rss 列表
        router.get("/reader3/getRssArticles").coroutineHandler { rssSourceController.getRssArticles(it) }
        router.post("/reader3/getRssArticles").coroutineHandler { rssSourceController.getRssArticles(it) }
        // rss 内容
        router.get("/reader3/getRssContent").coroutineHandler { rssSourceController.getRssContent(it) }
        router.post("/reader3/getRssContent").coroutineHandler { rssSourceController.getRssContent(it) }

        /** 替换规则模块 */
        router.get("/reader3/getReplaceRules").coroutineHandler { replaceRuleController.getReplaceRules(it) }
        router.post("/reader3/saveReplaceRule").coroutineHandler { replaceRuleController.saveReplaceRule(it) }
        router.post("/reader3/saveReplaceRules").coroutineHandler { replaceRuleController.saveReplaceRules(it) }
        router.post("/reader3/deleteReplaceRule").coroutineHandler { replaceRuleController.deleteReplaceRule(it) }
        router.post("/reader3/deleteReplaceRules").coroutineHandler { replaceRuleController.deleteReplaceRules(it) }

        /** 书签模块 */
        router.get("/reader3/getBookmarks").coroutineHandler { bookmarkController.getBookmarks(it) }
        router.post("/reader3/saveBookmark").coroutineHandler { bookmarkController.saveBookmark(it) }
        router.post("/reader3/saveBookmarks").coroutineHandler { bookmarkController.saveBookmarks(it) }
        router.post("/reader3/deleteBookmark").coroutineHandler { bookmarkController.deleteBookmark(it) }
        router.post("/reader3/deleteBookmarks").coroutineHandler { bookmarkController.deleteBookmarks(it) }

        router.post("/reader3/book/saveBookConfig").coroutineHandler { bookController.saveBookConfig(it) }
        router.get("/reader3/user/downloadBackupFile").coroutineHandlerWithoutRes { userController.downloadBackupFile(it) }

        router.get("/reader3/book/tts").coroutineHandlerWithoutRes { bookController.textToSpeech(it) }
        router.post("/reader3/book/tts").coroutineHandlerWithoutRes { bookController.textToSpeech(it) }
        // 保存书籍章节内容到缓存
        router.post("/reader3/saveBookContent").coroutineHandler { bookController.saveBookContent(it) }

        /** MongoDB备份恢复 */
        router.post("/reader3/backupToMongodb").coroutineHandler { bookController.backupToMongodb(it) }
        router.post("/reader3/restoreFromMongodb").coroutineHandler { bookController.restoreFromMongodb(it) }

        /** 缓存书籍到服务器 */
        router.post("/reader3/cacheBookOnServer").coroutineHandler { bookController.cacheBookOnServer(it) }

        /** 清理不活跃用户 */
        router.post("/reader3/clearInactiveUsers").coroutineHandler { userController.clearInactiveUsers(it) }

        /** webdav备份 */
        router.post("/reader3/backupToWebdav").coroutineHandler { webdavController.backupToWebdav(it) }

        /** 文件管理模块 */
        router.get("/reader3/file/list").coroutineHandler { fileController.list(it) }
        router.get("/reader3/file/get").coroutineHandler { fileController.get(it) }
        router.post("/reader3/file/save").coroutineHandler { fileController.save(it) }
        router.post("/reader3/file/mkdir").coroutineHandler { fileController.mkdir(it) }
        router.get("/reader3/file/download").coroutineHandlerWithoutRes { fileController.download(it) }
        router.post("/reader3/file/upload").coroutineHandler { fileController.upload(it) }
        router.post("/reader3/file/delete").coroutineHandler { fileController.delete(it) }
        router.post("/reader3/file/deleteMulti").coroutineHandler { fileController.deleteMulti(it) }
        router.post("/reader3/file/importPreview").coroutineHandler { fileController.importPreview(it) }
        router.post("/reader3/file/restore").coroutineHandler { fileController.restore(it) }
        router.get("/reader3/file/parse").coroutineHandler { fileController.parse(it) }
        router.post("/reader3/file/parse").coroutineHandler { fileController.parse(it) }

        /** HttpTTS模块 */
        router.get("/reader3/httpTTS/list").coroutineHandler { httpTTSController.getHttpTTSList(it) }
        router.post("/reader3/httpTTS/save").coroutineHandler { httpTTSController.saveHttpTTS(it) }
        router.post("/reader3/httpTTS/saveMulti").coroutineHandler { httpTTSController.saveHttpTTSList(it) }
        router.post("/reader3/httpTTS/delete").coroutineHandler { httpTTSController.deleteHttpTTS(it) }
        router.post("/reader3/httpTTS/deleteMulti").coroutineHandler { httpTTSController.deleteHttpTTS(it) }
    }

    suspend fun setupPort() {
        logger.info("port: {}", port)
        var serverPort = env.getProperty("reader.server.port", Int::class.java)
        logger.info("serverPort: {}", serverPort)
        if (serverPort != null && serverPort > 0) {
            port = serverPort;
        }
    }

    suspend fun migration() {
        try {
            var storageDir = File(getWorkDir("storage"))
            var dataDir = File(getWorkDir("storage", "data", "default"))
            if (!storageDir.exists()) {
                // 直接使用新版本，则创建 default 目录，防止重启之后被迁移
                dataDir.mkdirs()
            } else if (!dataDir.exists()) {
                // 旧版本不管了
                dataDir.mkdirs()
            }
        } catch(e: Exception) {
            e.printStackTrace()
        }
    }

    override fun started() {
        SpringContextUtils.getApplicationContext().publishEvent(SpringEvent(this as java.lang.Object, "READY", ""));
    }

    override fun onStartError() {
        logger.error("应用启动失败，请检查" + port + "端口是否被占用")
        SpringContextUtils.getApplicationContext().publishEvent(SpringEvent(this as java.lang.Object, "START_ERROR", "应用启动失败，请检查" + port + "端口是否被占用"));
    }

    override fun onHandlerError(ctx: RoutingContext, error: Exception) {
        val returnData = ReturnData()
        logger.error("onHandlerError: ", error)
        if (!ctx.response().headWritten()) {
            ctx.success(returnData.setErrorMsg(error.toString()))
        } else {
            ctx.response().end(error.toString())
        }
    }

    private suspend fun getSystemInfo(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        var systemFont = System.getProperty("reader.system.fonts")
        var freeMemory = "" + (Runtime.getRuntime().freeMemory() / 1024 / 1024) + "M"
        var totalMemory = "" + (Runtime.getRuntime().totalMemory() / 1024 / 1024) + "M"
        var maxMemory = "" + (Runtime.getRuntime().maxMemory() / 1024 / 1024) + "M"
        val userController = UserController(coroutineContext)
        var dayLoginUser = 0
        var sevenDayLoginUser = 0
        var monthLoginUser = 0
        var keepUser = 0
        var dayRegisterUser = 0
        var sevenDayRegisterUser = 0
        var monthRegisterUser = 0
        val calendar = Calendar.getInstance().apply {
            set(Calendar.DAY_OF_MONTH, 1)
            set(Calendar.HOUR_OF_DAY, 0)
            set(Calendar.MINUTE, 0)
            set(Calendar.SECOND, 0)
            set(Calendar.MILLISECOND, 0)
        }
        userController.forEachUser { user ->
            if (user.last_login_at >= System.currentTimeMillis() - 86400000L) dayLoginUser++
            if (user.last_login_at >= System.currentTimeMillis() - 604800000L) sevenDayLoginUser++
            if (user.last_login_at >= calendar.timeInMillis) monthLoginUser++
            if (user.created_at >= System.currentTimeMillis() - 86400000L) dayRegisterUser++
            if (user.created_at >= System.currentTimeMillis() - 604800000L) sevenDayRegisterUser++
            if (user.created_at >= calendar.timeInMillis) monthRegisterUser++
            if (user.last_login_at >= user.created_at + 604800000L &&
                user.last_login_at >= System.currentTimeMillis() - 604800000L) keepUser++
            false
        }
        return returnData.setData(mapOf(
            "fonts" to systemFont,
            "freeMemory" to freeMemory,
            "totalMemory" to totalMemory,
            "maxMemory" to maxMemory,
            "dayRegisterUser" to dayRegisterUser,
            "dayLoginUser" to dayLoginUser,
            "sevenDayRegisterUser" to sevenDayRegisterUser,
            "sevenDayLoginUser" to sevenDayLoginUser,
            "monthRegisterUser" to monthRegisterUser,
            "monthLoginUser" to monthLoginUser,
            "keepUser" to keepUser
        ))
    }

    /**
     * 定时任务
     */

    /**
     * 定期检查书架书籍更新 (使用配置的间隔时间)
     */
    @Scheduled(cron = "0 0/10 * * * ?")
    fun shelfUpdateJob()
    {
        if (appConfig.shelfUpdateInteval <= 0) {
            return
        }
        val now = Calendar.getInstance()
        val minuteFromToday = now.get(Calendar.HOUR_OF_DAY) * 60 + now.get(Calendar.MINUTE)
        if (minuteFromToday % appConfig.shelfUpdateInteval != 0) {
            return
        }
        MDC.put("traceId", getTraceId())
        launch(MDCContext() + Dispatchers.IO) {
            try {
                val bookController = BookController(coroutineContext)

                logger.info("开始检查书架书籍更新")
                // 刷新系统默认书架
                bookController.getBookShelfBooks(true, "default")

                // 刷新用户书架
                val userController = UserController(coroutineContext)
                userController.forEachUser { user ->
                    if (user.last_login_at >= System.currentTimeMillis() - 259200000L) {
                        bookController.getBookShelfBooks(true, user.username)
                    }
                    false
                }
                logger.info("书架书籍更新检查结束")
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }

    /**
     * 每天清理不活跃用户
     */
    @Scheduled(cron = "0 59 23 * * ?")
    fun clearUser()
    {
        if (appConfig.autoClearInactiveUser <= 0 || !appConfig.secure) {
            return
        }
        MDC.put("traceId", getTraceId())
        launch(MDCContext() + Dispatchers.IO) {
            try {
                logger.info("开始清理 {} 天未登录用户", appConfig.autoClearInactiveUser)
                UserController(coroutineContext).clearInactiveUsers(appConfig.autoClearInactiveUser)
                logger.info("不活跃用户自动清理结束")
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }

    /**
     * 自动备份用户数据 (每天凌晨2点)
     */
    @Scheduled(cron = "0 50 23 * * ?")
    fun autoBackup()
    {
        if (!appConfig.autoBackupUserData) {
            return
        }
        MDC.put("traceId", getTraceId())
        launch(MDCContext() + Dispatchers.IO) {
            try {
                logger.info("开始备份用户数据")
                val bookController = BookController(coroutineContext)

                // 备份默认用户
                bookController.saveToWebdav("default")

                // 备份其他用户
                val userController = UserController(coroutineContext)
                userController.forEachUser { user ->
                    if (user.last_login_at >= System.currentTimeMillis() - 259200000L) {
                        bookController.saveToWebdav(user.username)
                    }
                    false
                }
                logger.info("备份用户数据结束")
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }

    /**
     * 定期执行垃圾回收
     */
    @Scheduled(cron = "0 0 2 * * ?")
    fun autoGC()
    {
        System.gc()
    }

    /**
     * 远程书源订阅更新
     */
    @Scheduled(cron = "0 0/10 * * * ?")
    fun remoteBookSourceSubUpdateJob()
    {
        if (appConfig.remoteBookSourceUpdateInterval <= 0) {
            return
        }
        val now = Calendar.getInstance()
        val minuteFromToday = now.get(Calendar.HOUR_OF_DAY) * 60 + now.get(Calendar.MINUTE)
        if (minuteFromToday % appConfig.remoteBookSourceUpdateInterval != 0) {
            return
        }
        MDC.put("traceId", getTraceId())
        launch(MDCContext() + Dispatchers.IO) {
            try {
                logger.info("开始检查远程书源更新")
                val bookSourceController = BookSourceController(coroutineContext)
                // Update for default namespace
                bookSourceController.updateRemoteSourceSub("default")

                // Update for all users
                val userController = UserController(coroutineContext)
                userController.forEachUser { user ->
                    if (user.last_login_at >= System.currentTimeMillis() - 259200000L) {
                        bookSourceController.updateRemoteSourceSub(user.username)
                    }
                    false
                }
                logger.info("远程书源更新检查结束")
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }
}
