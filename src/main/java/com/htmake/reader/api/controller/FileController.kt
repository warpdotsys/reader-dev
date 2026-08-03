package com.htmake.reader.api.controller

import com.htmake.reader.api.ReturnData
import com.htmake.reader.entity.User
import com.htmake.reader.utils.getWorkDir
import com.htmake.reader.utils.listFilesRecursively
import com.htmake.reader.utils.success
import io.legado.app.data.entities.Book
import io.legado.app.exception.TocEmptyException
import io.legado.app.model.localBook.LocalBook
import io.vertx.core.http.HttpMethod
import io.vertx.ext.web.RoutingContext
import mu.KotlinLogging
import java.io.File
import java.net.URLEncoder
import kotlin.coroutines.CoroutineContext

private val logger = KotlinLogging.logger {}

class FileController(coroutineContext: CoroutineContext) : BaseController(coroutineContext) {

    private fun resolveSecurePath(baseDir: File, relativePath: String): File? {
        val basePath = baseDir.toPath().toAbsolutePath().normalize()
        val resolved = basePath.resolve(relativePath.removePrefix("/").removePrefix("\\")).normalize()
        return resolved.takeIf { it.startsWith(basePath) }?.toFile()
    }

    private fun requestedHome(context: RoutingContext): String = when {
        context.request().method() == HttpMethod.POST && context.fileUploads().isNotEmpty() ->
            context.request().getParam("home") ?: ""
        context.request().method() == HttpMethod.POST -> context.bodyAsJson?.getString("home", "") ?: ""
        else -> context.queryParam("home").firstOrNull() ?: ""
    }

    private fun requestPath(context: RoutingContext, key: String = "path"): String = if (context.request().method() == HttpMethod.POST) {
        context.bodyAsJson?.getString(key) ?: ""
    } else {
        context.queryParam(key).firstOrNull() ?: ""
    }

    private fun getFileHome(context: RoutingContext): File? = context.get<File>("__FILE_HOME__")

    suspend fun checkAccess(context: RoutingContext, isSave: Boolean = false, isDelete: Boolean = false): ReturnData? {
        val returnData = ReturnData()
        if (!checkAuth(context)) return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        context.put("__FILE_HOME__", null)
        val directory = when (requestedHome(context)) {
            "__WEBDAV__" -> {
                if (appConfig.secure) {
                    val userInfo = context.get<User>("userInfo")
                        ?: return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
                    if (!userInfo.enable_webdav) return returnData.setErrorMsg("未开启webdav功能")
                }
                File(getUserWebdavHome(context))
            }
            "__LOCAL_STORE__" -> {
                if (appConfig.secure) {
                    val userInfo = context.get<User>("userInfo")
                        ?: return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
                    if (!userInfo.enable_local_store) return returnData.setErrorMsg("未开启本地书仓功能")
                }
                if ((isSave || isDelete) && !checkManagerAuth(context)) {
                    return returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码")
                }
                File(getWorkDir("storage", "localStore"))
            }
            "__HOME__" -> File(getWorkDir("storage", "data", getUserNameSpace(context)))
            "__STORAGE__" -> {
                if (!checkManagerAuth(context)) return returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码")
                File(getWorkDir("storage"))
            }
            else -> {
                // 空 home 回退用户数据目录（JAR 继承 bug：home= 空值误报"非法访问"，
                // 兼容旧客户端/手动构造 URL 的 file/list 等请求）
                if (requestedHome(context).isEmpty()) {
                    File(getWorkDir("storage", "data", getUserNameSpace(context)))
                } else {
                    return returnData.setErrorMsg("非法访问")
                }
            }
        }
        directory.mkdirs()
        context.put("__FILE_HOME__", directory)
        logger.info { "context.__FILE_HOME__ $directory" }
        return null
    }

    suspend fun list(context: RoutingContext): ReturnData {
        checkAccess(context)?.let { return it }
        val returnData = ReturnData()
        val baseDir = getFileHome(context) ?: return returnData.setErrorMsg("参数错误")
        val path = requestPath(context).ifEmpty { "/" }
        val file = resolveSecurePath(baseDir, path) ?: return returnData.setErrorMsg("路径不存在")
        logger.info { "file: $path $file" }
        if (!file.exists()) {
            if (path != "/") return returnData.setErrorMsg("路径不存在")
            file.mkdirs()
        }
        if (!file.isDirectory) return returnData.setErrorMsg("路径不是目录")
        val files = file.listFiles() ?: emptyArray()
        val fileList = files.filterNot { it.name.startsWith(".") }.map { item ->
            mapOf(
                "name" to item.name,
                "size" to item.length(),
                "path" to "/" + item.relativeTo(baseDir).path.replace(File.separatorChar, '/'),
                "lastModified" to item.lastModified(),
                "isDirectory" to item.isDirectory
            )
        }
        return returnData.setData(fileList)
    }

    suspend fun upload(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (context.fileUploads().isEmpty()) return returnData.setErrorMsg("请上传文件")
        checkAccess(context, isSave = true)?.let { return it }
        val baseDir = getFileHome(context) ?: return returnData.setErrorMsg("参数错误")
        val path = (context.request().getParam("path") ?: "").ifEmpty { "/" }
        val targetDir = resolveSecurePath(baseDir, path) ?: return returnData.setErrorMsg("路径不存在")
        val fileList = ArrayList<Map<String, Any>>()
        context.fileUploads().forEach { upload ->
            val source = File(upload.uploadedFileName())
            if (!source.exists()) return@forEach
            val destination = resolveSecurePath(targetDir, File(upload.fileName()).name) ?: return@forEach
            destination.parentFile.mkdirs()
            if (destination.exists()) destination.delete()
            if (source.copyTo(destination, overwrite = false).exists()) {
                fileList += mapOf(
                    "name" to destination.name,
                    "size" to destination.length(),
                    "path" to "/" + destination.relativeTo(baseDir).path.replace(File.separatorChar, '/'),
                    "lastModified" to destination.lastModified(),
                    "isDirectory" to destination.isDirectory
                )
            }
            source.deleteRecursively()
        }
        return returnData.setData(fileList)
    }

    suspend fun download(context: RoutingContext) {
        val accessResult = checkAccess(context)
        if (accessResult != null) {
            context.success(accessResult)
            return
        }
        val returnData = ReturnData()
        val path = requestPath(context)
        val stream = if (context.request().method() == HttpMethod.POST) {
            context.bodyAsJson?.getInteger("stream", 0) ?: 0
        } else {
            context.queryParam("stream").firstOrNull()?.toIntOrNull() ?: 0
        }
        if (path.isEmpty()) {
            context.success(returnData.setErrorMsg("参数错误"))
            return
        }
        val baseDir = getFileHome(context)
        val file = baseDir?.let { resolveSecurePath(it, path) }
        if (file == null) {
            context.success(returnData.setErrorMsg("参数错误"))
            return
        }
        logger.info { "file: $path $file" }
        if (!file.exists()) {
            context.success(returnData.setErrorMsg("路径不存在"))
            return
        }
        val response = context.response().putHeader("Cache-Control", "86400")
        if (stream <= 0) response.putHeader("Content-Disposition", "attachment; filename=${URLEncoder.encode(file.name, "UTF-8")}")
        response.sendFile(file.toString())
    }

    suspend fun get(context: RoutingContext): ReturnData {
        checkAccess(context)?.let { return it }
        val returnData = ReturnData()
        val path = requestPath(context)
        if (path.isEmpty()) return returnData.setErrorMsg("参数错误")
        val file = getFileHome(context)?.let { resolveSecurePath(it, path) } ?: return returnData.setErrorMsg("参数错误")
        logger.info { "file: $path $file" }
        if (!file.exists()) return returnData.setErrorMsg("路径不存在")
        return returnData.setData(file.readText())
    }

    suspend fun save(context: RoutingContext): ReturnData {
        checkAccess(context, isSave = true)?.let { return it }
        val returnData = ReturnData()
        val path = context.bodyAsJson?.getString("path", "") ?: ""
        val content = context.bodyAsJson?.getString("content", "") ?: ""
        if (path.isEmpty()) return returnData.setErrorMsg("参数错误")
        val file = getFileHome(context)?.let { resolveSecurePath(it, path) } ?: return returnData.setErrorMsg("参数错误")
        logger.info { "file: $path $file" }
        file.parentFile.mkdirs()
        file.writeText(content)
        return returnData.setData("")
    }

    suspend fun mkdir(context: RoutingContext): ReturnData {
        checkAccess(context, isSave = true)?.let { return it }
        val returnData = ReturnData()
        val path = context.bodyAsJson?.getString("path", "") ?: ""
        val name = context.bodyAsJson?.getString("name", "") ?: ""
        if (path.isEmpty() || name.isEmpty() || name.startsWith(".")) return returnData.setErrorMsg("参数错误")
        val parent = getFileHome(context)?.let { resolveSecurePath(it, path) } ?: return returnData.setErrorMsg("参数错误")
        val directory = resolveSecurePath(parent, name) ?: return returnData.setErrorMsg("参数错误")
        logger.info { "file: $path $directory" }
        if (directory.exists()) return returnData.setErrorMsg("路径已存在")
        directory.mkdirs()
        return returnData.setData("")
    }

    suspend fun delete(context: RoutingContext): ReturnData {
        checkAccess(context, isDelete = true)?.let { return it }
        val returnData = ReturnData()
        val path = requestPath(context)
        if (path.isEmpty()) return returnData.setErrorMsg("参数错误")
        val file = getFileHome(context)?.let { resolveSecurePath(it, path) } ?: return returnData.setErrorMsg("参数错误")
        logger.info { "file: $path $file" }
        if (!file.exists()) return returnData.setErrorMsg("路径不存在")
        file.deleteRecursively()
        return returnData.setData("")
    }

    suspend fun deleteMulti(context: RoutingContext): ReturnData {
        checkAccess(context, isDelete = true)?.let { return it }
        val returnData = ReturnData()
        val paths = context.bodyAsJson?.getJsonArray("path") ?: return returnData.setErrorMsg("参数错误")
        val baseDir = getFileHome(context) ?: return returnData.setErrorMsg("参数错误")
        paths.forEach { value ->
            val path = value as? String ?: return@forEach
            if (path.isNotEmpty()) resolveSecurePath(baseDir, path)?.deleteRecursively()
        }
        return returnData.setData("")
    }

    suspend fun importPreview(context: RoutingContext): ReturnData {
        checkAccess(context)?.let { return it }
        val returnData = ReturnData()
        val paths = context.bodyAsJson?.getJsonArray("path") ?: return returnData.setErrorMsg("参数错误")
        val baseDir = getFileHome(context) ?: return returnData.setErrorMsg("参数错误")
        val userNameSpace = getUserNameSpace(context)
        val rootDir = getWorkDir().let { if (it.endsWith(File.separator)) it else it + File.separator }
        val fileList = ArrayList<Map<String, Any>>()
        paths.forEach { value ->
            val path = value as? String ?: return@forEach
            if (path.isEmpty()) return@forEach
            val file = resolveSecurePath(baseDir, path) ?: return@forEach
            logger.info { "localFile: $path $file" }
            logger.debug("rootDir: {} path: {}", rootDir, file.path)
            if (!file.exists() || file.isDirectory) return@forEach
            val ext = getFileExt(file.name)
            if (ext !in setOf("txt", "epub", "umd", "cbz", "pdf")) {
                return returnData.setErrorMsg("不支持导入${ext}格式的书籍文件")
            }
            var relativePath = file.path
            if (relativePath.startsWith(rootDir)) relativePath = relativePath.removePrefix(rootDir)
            logger.debug("relative path: {}", relativePath)
            val book = Book.initLocalBook(relativePath.replace("\\", "/"), relativePath, rootDir)
            book.setUserNameSpace(userNameSpace)
            try {
                fileList += mapOf("book" to book, "chapters" to LocalBook.getChapterList(book))
            } catch (_: TocEmptyException) {
                fileList += mapOf("book" to book, "chapters" to arrayListOf<Any>())
            }
        }
        return returnData.setData(fileList)
    }

    suspend fun restore(context: RoutingContext): ReturnData {
        checkAccess(context)?.let { return it }
        val returnData = ReturnData()
        val path = requestPath(context).ifEmpty { "/" }
        if (getFileExt(path) != "zip") return returnData.setErrorMsg("路径不是zip备份文件")
        val file = getFileHome(context)?.let { resolveSecurePath(it, path) } ?: return returnData.setErrorMsg("参数错误")
        logger.info { "file: $path $file" }
        if (!file.exists()) return returnData.setErrorMsg("路径不存在")
        if (!BookController(coroutineContext).syncFromWebdav(file.toString(), getUserNameSpace(context))) {
            return returnData.setErrorMsg("恢复失败")
        }
        return returnData.setData("")
    }

    suspend fun parse(context: RoutingContext): ReturnData {
        checkAccess(context)?.let { return it }
        val returnData = ReturnData()
        val path = requestPath(context).ifEmpty { "/" }
        val import = if (context.request().method() == HttpMethod.POST) {
            context.bodyAsJson?.getInteger("import", 0) ?: 0
        } else {
            context.queryParam("import").firstOrNull()?.toIntOrNull() ?: 0
        }
        val baseDir = getFileHome(context) ?: return returnData.setErrorMsg("参数错误")
        val directory = resolveSecurePath(baseDir, path) ?: return returnData.setErrorMsg("路径不存在")
        logger.info { "file: $path $directory" }
        if (!directory.exists()) return returnData.setErrorMsg("路径不存在")
        if (!directory.isDirectory) return returnData.setErrorMsg("路径不是目录")
        val userNameSpace = getUserNameSpace(context)
        val rootDir = getWorkDir().let { if (it.endsWith(File.separator)) it else it + File.separator }
        val bookController = BookController(coroutineContext)
        val fileList = ArrayList<Map<String, Any>>()
        listFilesRecursively(directory).forEach { file ->
            if (file.name.startsWith(".") || !file.isFile || getFileExt(file.name) !in setOf("txt", "epub", "umd", "cbz", "pdf")) return@forEach
            logger.debug("rootDir: {} path: {}", rootDir, file.path)
            var relativePath = file.path
            if (relativePath.startsWith(rootDir)) relativePath = relativePath.removePrefix(rootDir)
            logger.debug("relative path: {}", relativePath)
            val book = Book.initLocalBook(relativePath.replace("\\", "/"), relativePath, rootDir)
            book.setUserNameSpace(userNameSpace)
            logger.debug("book {}", book)
            if (import > 0) {
                val result = bookController.saveBookToShelf(book, userNameSpace, context)
                if (result.second == null && result.first.isInShelf) fileList += mapOf("name" to file.name)
            } else {
                fileList += mapOf(
                    "name" to file.name,
                    "size" to file.length(),
                    "path" to "/" + file.relativeTo(baseDir).path.replace(File.separatorChar, '/'),
                    "lastModified" to file.lastModified(),
                    "book" to book
                )
            }
        }
        return returnData.setData(fileList)
    }
}
