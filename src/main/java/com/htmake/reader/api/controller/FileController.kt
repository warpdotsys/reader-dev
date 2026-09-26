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
import java.io.IOException
import java.net.URLEncoder
import java.nio.file.Files
import java.nio.file.InvalidPathException
import java.nio.file.LinkOption
import kotlin.coroutines.CoroutineContext

private val logger = KotlinLogging.logger {}

class FileController(coroutineContext: CoroutineContext) : BaseController(coroutineContext) {

    private fun resolveSecurePath(baseDir: File, relativePath: String): File? {
        return try {
            val basePath = baseDir.toPath().toAbsolutePath().normalize()
            val requested = relativePath.removePrefix("/").removePrefix("\\")
            val lexicalTarget = basePath.resolve(requested).normalize()
            if (!lexicalTarget.startsWith(basePath)) return null

            // Windows 的目录联接和 POSIX 符号链接都会让逻辑路径与物理路径不同。
            // 必须从真实根目录逐段前进：每个已存在的节点都检查其真实位置，随后
            // 的不存在节点才附加到已验证的真实祖先。这样既不会把正常文件误判为
            // 越界，也不会把 I/O 再落回未经验证的逻辑路径。
            val baseReal = basePath.toRealPath()
            var target = baseReal
            basePath.relativize(lexicalTarget).forEach { segment ->
                val candidate = target.resolve(segment)
                target = if (Files.exists(candidate, LinkOption.NOFOLLOW_LINKS)) {
                    candidate.toRealPath().also {
                        if (!it.startsWith(baseReal)) return null
                    }
                } else {
                    candidate
                }
            }
            target.toFile()
        } catch (_: IOException) {
            null
        } catch (_: InvalidPathException) {
            null
        }
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
                "path" to File.separator + item.relativeTo(baseDir).path,
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
                    "path" to File.separator + destination.relativeTo(baseDir).path,
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

    suspend fun rename(context: RoutingContext): ReturnData {
        checkAccess(context, isSave = true, isDelete = true)?.let { return it }
        val returnData = ReturnData()
        val path = context.bodyAsJson?.getString("path", "") ?: ""
        val name = context.bodyAsJson?.getString("name", "") ?: ""
        if (path.isEmpty() || name.isEmpty() || name == "." || name == ".." ||
            name.startsWith(".") || name.contains('/') || name.contains('\\') || name.contains('\u0000')) {
            return returnData.setErrorMsg("参数错误")
        }
        val baseDir = getFileHome(context) ?: return returnData.setErrorMsg("参数错误")
        val source = resolveSecurePath(baseDir, path) ?: return returnData.setErrorMsg("参数错误")
        if (!source.exists()) return returnData.setErrorMsg("路径不存在")
        return try {
            // Resolve real paths here as well: this route must remain safe even if
            // the shared resolver changes or a directory is a symlink/junction.
            val baseReal = baseDir.toPath().toRealPath()
            val sourceReal = source.toPath().toRealPath()
            if (!sourceReal.startsWith(baseReal)) returnData.setErrorMsg("参数错误")
            else if (sourceReal == baseReal) returnData.setErrorMsg("不能重命名根目录")
            else {
                val parentReal = source.parentFile.toPath().toRealPath()
                if (!parentReal.startsWith(baseReal)) returnData.setErrorMsg("参数错误")
                else {
                    val target = parentReal.resolve(name).normalize()
                    if (!target.startsWith(baseReal)) returnData.setErrorMsg("参数错误")
                    else if (java.nio.file.Files.exists(target, java.nio.file.LinkOption.NOFOLLOW_LINKS)) {
                        returnData.setErrorMsg("路径已存在")
                    } else {
                        java.nio.file.Files.move(source.toPath(), target)
                        returnData.setData("")
                    }
                }
            }
        } catch (_: java.io.IOException) {
            returnData.setErrorMsg("重命名失败")
        } catch (_: java.nio.file.InvalidPathException) {
            returnData.setErrorMsg("参数错误")
        }
    }

    suspend fun move(context: RoutingContext): ReturnData {
        checkAccess(context, isSave = true, isDelete = true)?.let { return it }
        val returnData = ReturnData()
        val path = context.bodyAsJson?.getString("path", "") ?: ""
        val targetDirInput = context.bodyAsJson?.getString("targetDir", "") ?: ""
        if (path.isEmpty() || path.contains('\u0000') || targetDirInput.isEmpty() || targetDirInput.contains('\u0000')) {
            return returnData.setErrorMsg("参数错误")
        }
        val baseDir = getFileHome(context) ?: return returnData.setErrorMsg("参数错误")
        val source = resolveSecurePath(baseDir, path) ?: return returnData.setErrorMsg("参数错误")
        if (!source.exists()) return returnData.setErrorMsg("路径不存在")
        return try {
            val baseReal = baseDir.toPath().toRealPath()
            val sourceReal = source.toPath().toRealPath()
            if (!sourceReal.startsWith(baseReal) || sourceReal == baseReal) {
                returnData.setErrorMsg("参数错误")
            } else {
                val requested = baseReal.resolve(targetDirInput.trimStart('/', '\\')).normalize()
                if (!requested.startsWith(baseReal)) returnData.setErrorMsg("参数错误")
                else {
                    // Resolve the nearest existing ancestor before creating directories;
                    // a symlink/junction must not redirect the target outside this home.
                    var ancestor = requested
                    while (!java.nio.file.Files.exists(ancestor, java.nio.file.LinkOption.NOFOLLOW_LINKS)) {
                        ancestor = ancestor.parent ?: return returnData.setErrorMsg("参数错误")
                    }
                    val ancestorReal = ancestor.toRealPath()
                    if (!ancestorReal.startsWith(baseReal) || !java.nio.file.Files.isDirectory(ancestorReal)) {
                        returnData.setErrorMsg("参数错误")
                    } else {
                        val destinationDir = ancestorReal.resolve(ancestor.relativize(requested)).normalize()
                        if (!destinationDir.startsWith(baseReal) ||
                            (source.isDirectory && destinationDir.startsWith(sourceReal))) {
                            returnData.setErrorMsg("不能移动到自身或子目录")
                        } else {
                            java.nio.file.Files.createDirectories(destinationDir)
                            val destinationReal = destinationDir.toRealPath()
                            if (!destinationReal.startsWith(baseReal)) returnData.setErrorMsg("参数错误")
                            else {
                                val target = destinationReal.resolve(source.name)
                                if (java.nio.file.Files.exists(target, java.nio.file.LinkOption.NOFOLLOW_LINKS)) {
                                    returnData.setErrorMsg("目标路径已存在")
                                } else {
                                    java.nio.file.Files.move(source.toPath(), target)
                                    returnData.setData("/" + baseReal.relativize(target).toString().replace('\\', '/'))
                                }
                            }
                        }
                    }
                }
            }
        } catch (_: java.io.IOException) {
            returnData.setErrorMsg("移动失败")
        } catch (_: java.nio.file.InvalidPathException) {
            returnData.setErrorMsg("参数错误")
        }
    }

    suspend fun scanLocalBookDir(context: RoutingContext): ReturnData {
        checkAccess(context)?.let { return it }
        val returnData = ReturnData()
        val baseDir = getFileHome(context) ?: return returnData.setErrorMsg("参数错误")
        val path = context.bodyAsJson?.getString("path", "/") ?: "/"
        val recursive = context.bodyAsJson?.getBoolean("recursive", true) ?: true
        val selected = resolveSecurePath(baseDir, path) ?: return returnData.setErrorMsg("参数错误")
        if (!selected.exists()) return returnData.setErrorMsg("路径不存在")
        val baseReal: java.nio.file.Path
        val selectedReal: java.nio.file.Path
        try {
            baseReal = baseDir.toPath().toRealPath()
            selectedReal = selected.toPath().toRealPath()
        } catch (_: java.io.IOException) {
            return returnData.setErrorMsg("路径不存在")
        }
        if (!selectedReal.startsWith(baseReal)) return returnData.setErrorMsg("参数错误")
        if (selected.isFile && getFileExt(selected.name) !in setOf("txt", "epub", "umd", "cbz", "pdf")) {
            return returnData.setErrorMsg("不支持导入${getFileExt(selected.name)}格式的书籍文件")
        }

        val userNameSpace = getUserNameSpace(context)
        val rootDir = getWorkDir().let { if (it.endsWith(File.separator)) it else it + File.separator }
        val bookController = BookController(coroutineContext)
        val errors = ArrayList<Map<String, String>>()
        var imported = 0
        var total = 0
        val stream = try {
            when {
                selected.isFile -> java.util.stream.Stream.of(selectedReal)
                recursive -> java.nio.file.Files.walk(selectedReal)
                else -> java.nio.file.Files.list(selectedReal)
            }
        } catch (_: java.io.IOException) {
            return returnData.setErrorMsg("目录扫描失败")
        }
        try {
            stream.use { paths ->
                paths.forEach { candidate ->
                    if (!java.nio.file.Files.isRegularFile(candidate, java.nio.file.LinkOption.NOFOLLOW_LINKS)) return@forEach
                    val file = candidate.toFile()
                    if (file.name.startsWith(".") || getFileExt(file.name) !in setOf("txt", "epub", "umd", "cbz", "pdf")) {
                        return@forEach
                    }
                    total++
                    try {
                        val relativePath = file.path.removePrefix(rootDir)
                        val book = Book.initLocalBook(relativePath.replace("\\", "/"), relativePath, rootDir)
                        book.setUserNameSpace(userNameSpace)
                        val result = bookController.saveBookToShelf(book, userNameSpace, context)
                        if (result.second == null && result.first.isInShelf) imported++
                        else errors += mapOf("name" to file.name, "error" to (result.second ?: "导入失败"))
                    } catch (e: Exception) {
                        logger.warn(e) { "Local book import failed: ${file.name}" }
                        errors += mapOf("name" to file.name, "error" to "解析或导入失败")
                    }
                }
            }
        } catch (_: java.io.UncheckedIOException) {
            return returnData.setErrorMsg("目录扫描中断；部分书籍可能已导入")
        }
        return returnData.setData(mapOf(
            "imported" to imported,
            "failed" to (total - imported),
            "total" to total,
            "errors" to errors
        ))
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
            // Keep the JAR's mixed-separator path representation for local-book
            // chapter IDs, while using the separately validated file for I/O.
            val requestedPath = baseDir.path.trimEnd(File.separatorChar) + "/" +
                path.removePrefix("/").removePrefix("\\")
            var relativePath = if (
                File(requestedPath).toPath().toAbsolutePath().normalize() == file.toPath().toAbsolutePath().normalize()
            ) requestedPath else file.path
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
                    "path" to File.separator + file.relativeTo(baseDir).path,
                    "lastModified" to file.lastModified(),
                    "book" to book
                )
            }
        }
        return returnData.setData(fileList)
    }
}
