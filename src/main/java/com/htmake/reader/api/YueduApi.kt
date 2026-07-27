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
import com.htmake.reader.api.controller.LicenseController
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
import kotlinx.coroutines.launch
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

        // 旧版数据迁移
        migration()

        // simple-web界面
        router.route("/simple-web/*").handler(StaticHandler.create("simple-web").setDefaultContentEncoding("UTF-8"))

        // bookSourceDebug界面
        router.route("/bookSourceDebug/*").handler(StaticHandler.create("bookSourceDebug").setDefaultContentEncoding("UTF-8"))

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

        // epub资源
        var dataDir = getWorkDir("storage", "data");
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

        // 获取系统信息
        router.get("/reader3/getSystemInfo").coroutineHandler { getSystemInfo(it) }

        // contextPath support - if contextPath is configured, add redirect/rewrite
        val contextPath = env.getProperty("reader.server.contextPath", "") ?: ""
        if (contextPath.isNotEmpty()) {
            val prefix = if (contextPath.startsWith("/")) contextPath else "/$contextPath"
            // Mount routes with contextPath prefix that strip the prefix before passing to handlers
            router.route("${prefix}/*").handler { ctx ->
                val originalPath = ctx.request().path()
                val newPath = originalPath.removePrefix(prefix)
                ctx.reroute(if (newPath.isEmpty()) "/" else newPath)
            }
            // Also mount static resources under contextPath
            router.route("${prefix}").handler { ctx ->
                ctx.reroute("/")
            }
        }


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
        val licenseController = LicenseController(coroutineContext)

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

        // 读取远程书源文件
        router.post("/reader3/readRemoteSourceFile").coroutineHandlerWithoutRes { bookSourceController.readRemoteSourceFile(it) }

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

        // 保存书籍配置
        router.post("/reader3/saveBookConfig").coroutineHandler { bookController.saveBookConfig(it) }

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

        // 书仓功能
        // 获取书仓文件列表
        router.get("/reader3/getLocalStoreFileList").coroutineHandler { bookController.getLocalStoreFileList(it) }
        // 下载书仓文件
        router.get("/reader3/getLocalStoreFile").coroutineHandlerWithoutRes { bookController.getLocalStoreFile(it) }
        // 删除书仓文件
        router.post("/reader3/deleteLocalStoreFile").coroutineHandler { bookController.deleteLocalStoreFile(it) }
        router.post("/reader3/deleteLocalStoreFileList").coroutineHandler { bookController.deleteLocalStoreFileList(it) }
        // 从本地书仓/webdav导入
        router.post("/reader3/importFromLocalPathPreview").coroutineHandler { bookController.importFromLocalPathPreview(it) }
        // 上传文件到书仓
        router.post("/reader3/uploadFileToLocalStore").coroutineHandler { bookController.uploadFileToLocalStore(it) }

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


        /** webdav模块 */
        // 获取webdav备份列表
        router.get("/reader3/getWebdavFileList").coroutineHandler { webdavController.getWebdavFileList(it) }

        // 下载webdav文件
        router.get("/reader3/getWebdavFile").coroutineHandlerWithoutRes { webdavController.getWebdavFile(it) }

        // 上传webdav文件
        router.post("/reader3/uploadFileToWebdav").coroutineHandler { webdavController.uploadFileToWebdav(it) }

        // 删除webdav文件
        router.get("/reader3/deleteWebdavFile").coroutineHandler { webdavController.deleteWebdavFile(it) }
        router.post("/reader3/deleteWebdavFile").coroutineHandler { webdavController.deleteWebdavFile(it) }
        router.post("/reader3/deleteWebdavFileList").coroutineHandler { webdavController.deleteWebdavFileList(it) }

        // 从webdav备份恢复
        router.post("/reader3/restoreFromWebdav").coroutineHandler { webdavController.restoreFromWebdav(it) }

        // 备份到webdav
        router.post("/reader3/backupToWebdav").coroutineHandler { webdavController.backupToWebdav(it) }


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

        /** 文件管理模块 */
        router.get("/reader3/getFileList").coroutineHandler { fileController.list(it) }
        router.post("/reader3/getFileList").coroutineHandler { fileController.list(it) }
        router.post("/reader3/uploadFileToDir").coroutineHandler { fileController.upload(it) }
        router.get("/reader3/downloadFile").coroutineHandlerWithoutRes { fileController.download(it) }
        router.get("/reader3/getFileContent").coroutineHandler { fileController.get(it) }
        router.post("/reader3/getFileContent").coroutineHandler { fileController.get(it) }
        router.post("/reader3/saveFileContent").coroutineHandler { fileController.save(it) }
        router.post("/reader3/createDir").coroutineHandler { fileController.mkdir(it) }
        router.post("/reader3/deleteFileOrDir").coroutineHandler { fileController.delete(it) }
        router.post("/reader3/deleteFileOrDirMulti").coroutineHandler { fileController.deleteMulti(it) }
        router.post("/reader3/importFilePreview").coroutineHandler { fileController.importPreview(it) }
        router.post("/reader3/restoreFromFile").coroutineHandler { fileController.restore(it) }
        router.post("/reader3/parseFile").coroutineHandler { fileController.parse(it) }

        /** HttpTTS模块 */
        router.get("/reader3/getHttpTTSList").coroutineHandler { httpTTSController.getHttpTTSList(it) }
        router.post("/reader3/saveHttpTTS").coroutineHandler { httpTTSController.saveHttpTTS(it) }
        router.post("/reader3/saveHttpTTSList").coroutineHandler { httpTTSController.saveHttpTTSList(it) }
        router.post("/reader3/deleteHttpTTS").coroutineHandler { httpTTSController.deleteHttpTTS(it) }

        /** 路径风格路由别名 (Pro前端兼容) */
        // 文件管理 path-based aliases
        router.get("/reader3/file/list").coroutineHandler { fileController.list(it) }
        router.post("/reader3/file/list").coroutineHandler { fileController.list(it) }
        router.post("/reader3/file/upload").coroutineHandler { fileController.upload(it) }
        router.get("/reader3/file/download").coroutineHandlerWithoutRes { fileController.download(it) }
        router.get("/reader3/file/get").coroutineHandler { fileController.get(it) }
        router.post("/reader3/file/get").coroutineHandler { fileController.get(it) }
        router.post("/reader3/file/save").coroutineHandler { fileController.save(it) }
        router.post("/reader3/file/mkdir").coroutineHandler { fileController.mkdir(it) }
        router.post("/reader3/file/delete").coroutineHandler { fileController.delete(it) }
        router.post("/reader3/file/deleteMulti").coroutineHandler { fileController.deleteMulti(it) }
        router.post("/reader3/file/importPreview").coroutineHandler { fileController.importPreview(it) }
        router.post("/reader3/file/restore").coroutineHandler { fileController.restore(it) }
        router.post("/reader3/file/parse").coroutineHandler { fileController.parse(it) }

        // HttpTTS path-based aliases
        router.get("/reader3/httpTTS/list").coroutineHandler { httpTTSController.getHttpTTSList(it) }
        router.post("/reader3/httpTTS/save").coroutineHandler { httpTTSController.saveHttpTTS(it) }
        router.post("/reader3/httpTTS/saveMulti").coroutineHandler { httpTTSController.saveHttpTTSList(it) }
        router.post("/reader3/httpTTS/deleteMulti").coroutineHandler { httpTTSController.deleteHttpTTS(it) }

        // Book path-based aliases
        router.post("/reader3/book/saveBookConfig").coroutineHandler { bookController.saveBookConfig(it) }

        // User path-based aliases
        router.get("/reader3/user/downloadBackupFile").coroutineHandlerWithoutRes { userController.downloadBackupFile(it) }

        /** PDF渲染模块 */
        router.get("/reader3/convertPdfToImage").coroutineHandlerWithoutRes { bookController.convertPdfToImage(it) }
        router.post("/reader3/convertPdfToImage").coroutineHandlerWithoutRes { bookController.convertPdfToImage(it) }
        router.get("/reader3/savePdfPageToImage").coroutineHandler { bookController.savePdfPageToImage(it) }
        router.post("/reader3/savePdfPageToImage").coroutineHandler { bookController.savePdfPageToImage(it) }

        /** TTS语音合成 */
        router.get("/reader3/textToSpeech").coroutineHandlerWithoutRes { bookController.textToSpeech(it) }
        router.post("/reader3/textToSpeech").coroutineHandlerWithoutRes { bookController.textToSpeech(it) }
        router.get("/reader3/book/tts").coroutineHandlerWithoutRes { bookController.textToSpeech(it) }
        router.post("/reader3/book/tts").coroutineHandlerWithoutRes { bookController.textToSpeech(it) }
        router.get("/reader3/getSpeakStream").coroutineHandlerWithoutRes { bookController.getSpeakStream(it) }
        router.post("/reader3/getSpeakStream").coroutineHandlerWithoutRes { bookController.getSpeakStream(it) }

        // 保存书籍章节内容到缓存
        router.post("/reader3/saveBookContent").coroutineHandler { bookController.saveBookContent(it) }

        /** MongoDB备份恢复 */
        router.post("/reader3/backupToMongodb").coroutineHandler { bookController.backupToMongodb(it) }
        router.post("/reader3/restoreFromMongodb").coroutineHandler { bookController.restoreFromMongodb(it) }

        /** 缓存书籍到服务器 */
        router.post("/reader3/cacheBookOnServer").coroutineHandler { bookController.cacheBookOnServer(it) }
        router.get("/reader3/cacheBookOnServer").coroutineHandler { bookController.cacheBookOnServer(it) }

        /** 用户备份下载 */
        router.get("/reader3/downloadBackupFile").coroutineHandlerWithoutRes { userController.downloadBackupFile(it) }

        /** 清理不活跃用户 */
        router.post("/reader3/clearInactiveUsers").coroutineHandler { userController.clearInactiveUsers(it) }

        /** 许可证模块 */
        router.get("/reader3/isLicenseValid").coroutineHandler { licenseController.isLicenseValid(it) }
        router.get("/reader3/getLicense").coroutineHandler { licenseController.getLicense(it) }
        router.post("/reader3/importLicense").coroutineHandler { licenseController.importLicense(it) }
        router.post("/reader3/activateLicense").coroutineHandler { licenseController.activateLicense(it) }
        router.get("/reader3/checkLicense").coroutineHandler { licenseController.checkLicense(it) }
        router.post("/reader3/supplyLicense").coroutineHandler { licenseController.supplyLicense(it) }
        router.post("/reader3/generateKeys").coroutineHandler { licenseController.generateKeys(it) }
        router.post("/reader3/generateLicense").coroutineHandler { licenseController.generateLicense(it) }
        router.get("/reader3/isHostValid").coroutineHandler { licenseController.isHostValid(it) }
        router.post("/reader3/decryptLicense").coroutineHandler { licenseController.decryptLicense(it) }
        router.post("/reader3/sendCodeToEmail").coroutineHandler { licenseController.sendCodeToEmail(it) }
    }

    suspend fun setupPort() {
        logger.info("port: {}", port)
        var serverPort = env.getProperty("reader.server.port", Int::class.java)
        logger.info("serverPort: {}", serverPort)
        if (serverPort != null && serverPort > 0) {
            port = serverPort;
        }
        // Initialize MongoDB connection if configured
        val mongoUri = appConfig.mongoUri
        if (mongoUri.isNotEmpty()) {
            try {
                com.htmake.reader.utils.MongoManager.connect(mongoUri)
                logger.info("MongoDB connected")
            } catch (e: Exception) {
                logger.error("MongoDB connection failed: {}", e.message)
            }
        }
    }

    suspend fun migration() {
        try {
            // Support workDir configuration
            val workDir = appConfig.workDir
            if (workDir.isNotEmpty() && workDir != appConfig.storagePath) {
                com.htmake.reader.utils.workDirPath = workDir
                com.htmake.reader.utils.workDirInit = true
            }

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
        return returnData.setData(mapOf(
            "fonts" to systemFont,
            "freeMemory" to freeMemory,
            "totalMemory" to totalMemory,
            "maxMemory" to maxMemory
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
        launch(Dispatchers.IO) {
            try {
                val bookController = BookController(coroutineContext)

                logger.info("开始检查书架书籍更新")
                // 刷新系统默认书架
                bookController.getBookShelfBooks(true, "default")

                // 刷新用户书架
                if (appConfig.secure) {
                    var userMap = mutableMapOf<String, Map<String, Any>>()
                    var userMapJson: JsonObject? = asJsonObject(getStorage("data", "users"))
                    if (userMapJson != null) {
                        userMap = userMapJson.map as MutableMap<String, Map<String, Any>>
                    }
                    userMap.forEach{
                        try {
                            var ns = it.value.getOrDefault("username", "") as String? ?: ""
                            if (ns.isNotEmpty()) {
                                bookController.getBookShelfBooks(true, ns)
                            }
                        } catch (e: Exception) {
                            e.printStackTrace()
                        }
                    }
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
        launch(Dispatchers.IO) {
            try {
                logger.info("开始清理 {} 天未登录用户", appConfig.autoClearInactiveUser)

                var userMap = mutableMapOf<String, Map<String, Any>>()
                var userMapJson: JsonObject? = asJsonObject(getStorage("data", "users"))
                if (userMapJson != null) {
                    userMap = userMapJson.map as MutableMap<String, Map<String, Any>>
                }
                val expireTime = System.currentTimeMillis() - appConfig.autoClearInactiveUser * 86400L * 1000L
                userMap.keys.forEach{
                    try {
                        var user = userMap.get(it)
                        if (user != null) {
                            var username = user.getOrDefault("username", "") as String? ?: ""
                            var last_login_at = user.getOrDefault("last_login_at", 0) as Long? ?: 0L
                            if (username.isNotEmpty() && last_login_at < expireTime) {
                                logger.info("delete user: {}", user)
                                // 删除用户信息
                                userMap.remove(username)
                                // 移除用户目录
                                var userHome = File(getWorkDir("storage", "data", username))
                                logger.info("delete userHome: {}", userHome)
                                if (userHome.exists()) {
                                    userHome.deleteRecursively()
                                }
                            }
                        }
                    } catch (e: Exception) {
                        e.printStackTrace()
                    }
                }
                logger.info("不活跃用户自动清理结束")
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }

    /**
     * 自动备份用户数据 (每天凌晨2点)
     */
    @Scheduled(cron = "0 0 2 * * ?")
    fun autoBackup()
    {
        if (!appConfig.autoBackupUserData) {
            return
        }
        launch(Dispatchers.IO) {
            try {
                logger.info("开始自动备份用户数据")
                val bookController = BookController(coroutineContext)

                // 备份默认用户
                bookController.createUserBackup("default")

                // 备份其他用户
                if (appConfig.secure) {
                    var userMap = mutableMapOf<String, Map<String, Any>>()
                    var userMapJson: JsonObject? = asJsonObject(getStorage("data", "users"))
                    if (userMapJson != null) {
                        userMap = userMapJson.map as MutableMap<String, Map<String, Any>>
                    }
                    userMap.forEach{
                        try {
                            var ns = it.value.getOrDefault("username", "") as String? ?: ""
                            if (ns.isNotEmpty()) {
                                bookController.createUserBackup(ns)
                            }
                        } catch (e: Exception) {
                            e.printStackTrace()
                        }
                    }
                }
                logger.info("自动备份用户数据结束")
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }

    /**
     * 定期执行垃圾回收 (每6小时)
     */
    @Scheduled(cron = "0 0 0/6 * * ?")
    fun autoGC()
    {
        launch(Dispatchers.IO) {
            try {
                logger.info("执行垃圾回收")
                System.gc()
                logger.info("垃圾回收完成")
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }

    /**
     * 定期检查许可证有效性 (每6小时)
     */
    @Scheduled(cron = "0 0 0/6 * * ?")
    fun checkLicense()
    {
        launch(Dispatchers.IO) {
            try {
                val license = com.htmake.reader.utils.getInstalledLicense()
                if (license != null) {
                    logger.info("License check: found installed license")
                    val licenseController = LicenseController(coroutineContext)
                    licenseController.checkLicense(license)
                }
                com.htmake.reader.utils.setLicenseValid(true)
            } catch (e: Exception) {
                logger.error("checkLicense error: {}", e.message)
            }
        }
    }

    /**
     * 远程书源订阅更新 (默认每12小时)
     */
    @Scheduled(cron = "0 0 0/12 * * ?")
    fun remoteBookSourceSubUpdateJob()
    {
        if (appConfig.remoteBookSourceUpdateInterval <= 0) {
            return
        }
        launch(Dispatchers.IO) {
            try {
                logger.info("开始更新远程书源订阅")
                val bookSourceController = BookSourceController(coroutineContext)
                // Update for default namespace
                bookSourceController.updateRemoteSourceSub("default")

                // Update for all users
                if (appConfig.secure) {
                    var userMap = mutableMapOf<String, Map<String, Any>>()
                    var userMapJson: JsonObject? = asJsonObject(getStorage("data", "users"))
                    if (userMapJson != null) {
                        userMap = userMapJson.map as MutableMap<String, Map<String, Any>>
                    }
                    userMap.forEach{
                        try {
                            var ns = it.value.getOrDefault("username", "") as String? ?: ""
                            if (ns.isNotEmpty()) {
                                bookSourceController.updateRemoteSourceSub(ns)
                            }
                        } catch (e: Exception) {
                            e.printStackTrace()
                        }
                    }
                }
                logger.info("远程书源订阅更新结束")
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }
}