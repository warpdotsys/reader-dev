package com.htmake.reader.api.controller

import io.vertx.ext.web.RoutingContext
import io.vertx.core.json.JsonObject
import io.vertx.core.json.JsonArray
import io.vertx.core.http.HttpMethod
import mu.KotlinLogging
import com.htmake.reader.api.ReturnData
import com.htmake.reader.entity.User
import com.htmake.reader.utils.getWorkDir
import com.htmake.reader.utils.jsonEncode
import com.htmake.reader.utils.asJsonArray
import com.htmake.reader.utils.asJsonObject
import com.htmake.reader.utils.toDataClass
import com.htmake.reader.utils.toMap
import com.htmake.reader.utils.getStorage
import com.htmake.reader.utils.saveStorage
import com.htmake.reader.utils.deleteRecursively
import com.htmake.reader.utils.unzip
import java.io.File
import java.nio.file.Paths
import kotlin.coroutines.CoroutineContext

private val logger = KotlinLogging.logger {}

class FileController(coroutineContext: CoroutineContext): BaseController(coroutineContext) {

    private fun resolveSecurePath(baseDir: File, relativePath: String): File? {
        val basePath = baseDir.toPath().toAbsolutePath().normalize()
        val resolved = basePath.resolve(relativePath).normalize()
        if (!resolved.startsWith(basePath)) {
            return null
        }
        return resolved.toFile()
    }

    private fun requestedHome(context: RoutingContext): String {
        return when {
            context.request().method() == HttpMethod.POST && context.fileUploads().isNotEmpty() -> context.request().getParam("home") ?: ""
            context.request().method() == HttpMethod.POST -> context.bodyAsJson?.getString("home", "") ?: ""
            else -> context.queryParam("home").firstOrNull() ?: ""
        }
    }

    private fun getFileHome(context: RoutingContext, isSave: Boolean = false, isDelete: Boolean = false): File? {
        val home = requestedHome(context)
        val userNameSpace = getUserNameSpace(context)
        val directory = when (home) {
            "__WEBDAV__" -> File(getUserWebdavHome(context))
            "__LOCAL_STORE__" -> {
                if ((isSave || isDelete) && !checkManagerAuth(context)) return null
                File(getWorkDir("storage", "localStore"))
            }
            "__HOME__" -> File(getWorkDir("storage", "data", userNameSpace))
            "__STORAGE__" -> {
                if (!checkManagerAuth(context)) return null
                File(getWorkDir("storage"))
            }
            else -> return null
        }
        if (!directory.exists()) directory.mkdirs()
        return directory
    }

    suspend fun list(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        val path = if (context.request().method() == HttpMethod.POST) {
            context.bodyAsJson?.getString("path", "") ?: ""
        } else {
            context.queryParam("path").firstOrNull() ?: ""
        }

        val baseDir = getFileHome(context) ?: return returnData.setErrorMsg("非法访问")
        val targetDir = if (path.isEmpty()) {
            baseDir
        } else {
            resolveSecurePath(baseDir, path) ?: return returnData.setErrorMsg("路径不合法")
        }

        if (!targetDir.exists() || !targetDir.isDirectory) {
            return returnData.setData(arrayListOf<Any>())
        }

        val files = targetDir.listFiles() ?: return returnData.setData(arrayListOf<Any>())
        val fileList = files.map { file ->
            mapOf(
                "name" to file.name,
                "path" to file.relativeTo(baseDir).path,
                "isDirectory" to file.isDirectory,
                "size" to file.length(),
                "lastModified" to file.lastModified()
            )
        }.sortedWith(compareByDescending<Map<String, Any>> { it["isDirectory"] as Boolean }.thenBy { (it["name"] as String).lowercase() })

        return returnData.setData(fileList)
    }

    suspend fun upload(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        if (context.fileUploads() == null || context.fileUploads().isEmpty()) {
            return returnData.setErrorMsg("请上传文件")
        }
        val path = context.request().getParam("path") ?: ""
        val baseDir = getFileHome(context, isSave = true) ?: return returnData.setErrorMsg("非法访问")
        val targetDir = if (path.isEmpty()) {
            baseDir
        } else {
            resolveSecurePath(baseDir, path) ?: return returnData.setErrorMsg("路径不合法")
        }

        if (!targetDir.exists()) {
            targetDir.mkdirs()
        }

        val uploadedFiles = arrayListOf<String>()
        context.fileUploads().forEach {
            val file = File(it.uploadedFileName())
            if (file.exists()) {
                val fileName = File(it.fileName()).name
                val newFile = resolveSecurePath(targetDir, fileName) ?: return@forEach
                if (newFile.exists()) {
                    newFile.delete()
                }
                file.copyTo(newFile, overwrite = true)
                file.delete()
                uploadedFiles.add(newFile.relativeTo(baseDir).path)
            }
        }
        return returnData.setData(uploadedFiles)
    }

    suspend fun download(context: RoutingContext) {
        if (!checkAuth(context)) {
            context.response().setStatusCode(401).end("NEED_LOGIN")
            return
        }
        val path = context.queryParam("path").firstOrNull() ?: ""
        if (path.isEmpty()) {
            context.response().setStatusCode(400).end("请输入文件路径")
            return
        }

        val baseDir = getFileHome(context) ?: run {
            context.response().setStatusCode(403).end("非法访问")
            return
        }
        val file = resolveSecurePath(baseDir, path)
        if (file == null || !file.exists() || file.isDirectory) {
            context.response().setStatusCode(404).end("文件不存在")
            return
        }

        context.response()
            .putHeader("Content-Disposition", "attachment; filename=\"${file.name}\"")
            .putHeader("Content-Type", "application/octet-stream")
            .sendFile(file.absolutePath)
    }

    suspend fun get(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        val path = if (context.request().method() == HttpMethod.POST) {
            context.bodyAsJson?.getString("path", "") ?: ""
        } else {
            context.queryParam("path").firstOrNull() ?: ""
        }
        if (path.isEmpty()) {
            return returnData.setErrorMsg("请输入文件路径")
        }

        val baseDir = getFileHome(context) ?: return returnData.setErrorMsg("非法访问")
        val file = resolveSecurePath(baseDir, path)
        if (file == null || !file.exists() || file.isDirectory) {
            return returnData.setErrorMsg("文件不存在")
        }

        val content = file.readText(Charsets.UTF_8)
        return returnData.setData(content)
    }

    suspend fun save(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        val path = context.bodyAsJson?.getString("path", "") ?: ""
        val content = context.bodyAsJson?.getString("content", "") ?: ""
        if (path.isEmpty()) {
            return returnData.setErrorMsg("请输入文件路径")
        }

        val baseDir = getFileHome(context, isSave = true) ?: return returnData.setErrorMsg("非法访问")
        val file = resolveSecurePath(baseDir, path) ?: return returnData.setErrorMsg("路径不合法")

        if (!file.parentFile.exists()) {
            file.parentFile.mkdirs()
        }
        file.writeText(content, Charsets.UTF_8)
        return returnData.setData("")
    }

    suspend fun mkdir(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        val path = context.bodyAsJson?.getString("path", "") ?: ""
        if (path.isEmpty()) {
            return returnData.setErrorMsg("请输入目录路径")
        }

        val baseDir = getFileHome(context, isSave = true) ?: return returnData.setErrorMsg("非法访问")
        val dir = resolveSecurePath(baseDir, path) ?: return returnData.setErrorMsg("路径不合法")

        if (!dir.exists()) {
            dir.mkdirs()
        }
        return returnData.setData("")
    }

    suspend fun delete(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        val path = context.bodyAsJson?.getString("path", "") ?: ""
        if (path.isEmpty()) {
            return returnData.setErrorMsg("请输入文件路径")
        }

        val baseDir = getFileHome(context, isDelete = true) ?: return returnData.setErrorMsg("非法访问")
        val file = resolveSecurePath(baseDir, path) ?: return returnData.setErrorMsg("路径不合法")

        if (file.exists()) {
            file.deleteRecursively()
        }
        return returnData.setData("")
    }

    suspend fun deleteMulti(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        val body = context.bodyAsString
        if (body.isNullOrEmpty()) {
            return returnData.setErrorMsg("参数错误")
        }
        val paths: List<String> = try {
            val arr = JsonArray(body)
            (0 until arr.size()).map { arr.getString(it) }
        } catch (e: Exception) {
            return returnData.setErrorMsg("参数错误")
        }

        val baseDir = getFileHome(context, isDelete = true) ?: return returnData.setErrorMsg("非法访问")
        for (path in paths) {
            val file = resolveSecurePath(baseDir, path) ?: continue
            if (file.exists()) {
                file.deleteRecursively()
            }
        }
        return returnData.setData("")
    }

    suspend fun importPreview(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        val path = if (context.request().method() == HttpMethod.POST) {
            context.bodyAsJson?.getString("path", "") ?: ""
        } else {
            context.queryParam("path").firstOrNull() ?: ""
        }
        if (path.isEmpty()) {
            return returnData.setErrorMsg("请输入文件路径")
        }

        val baseDir = getFileHome(context) ?: return returnData.setErrorMsg("非法访问")
        val file = resolveSecurePath(baseDir, path) ?: return returnData.setErrorMsg("路径不合法")

        if (!file.exists()) {
            return returnData.setErrorMsg("文件不存在")
        }

        val supportedExtensions = listOf("txt", "epub", "umd", "cbz", "pdf")
        val fileList = if (file.isDirectory) {
            file.listFiles()?.filter { f ->
                supportedExtensions.any { ext -> f.name.endsWith(".$ext", ignoreCase = true) }
            }?.map { f ->
                mapOf(
                    "name" to f.name,
                    "path" to f.relativeTo(baseDir).path,
                    "size" to f.length()
                )
            } ?: emptyList()
        } else {
            if (supportedExtensions.any { ext -> file.name.endsWith(".$ext", ignoreCase = true) }) {
                listOf(mapOf(
                    "name" to file.name,
                    "path" to file.relativeTo(baseDir).path,
                    "size" to file.length()
                ))
            } else {
                emptyList()
            }
        }

        return returnData.setData(fileList)
    }

    suspend fun restore(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        val path = context.bodyAsJson?.getString("path", "") ?: ""
        if (path.isEmpty()) {
            return returnData.setErrorMsg("请输入文件路径")
        }

        val userNameSpace = getUserNameSpace(context)
        val baseDir = getFileHome(context) ?: return returnData.setErrorMsg("非法访问")
        val file = resolveSecurePath(baseDir, path) ?: return returnData.setErrorMsg("路径不合法")

        if (!file.exists()) {
            return returnData.setErrorMsg("文件不存在")
        }

        if (!file.name.endsWith(".zip", ignoreCase = true)) {
            return returnData.setErrorMsg("仅支持zip格式的备份文件")
        }

        try {
            val dataDir = File(getWorkDir("storage", "data", userNameSpace))
            file.unzip(dataDir.absolutePath)
        } catch (e: Exception) {
            return returnData.setErrorMsg("恢复失败: ${e.message}")
        }
        return returnData.setData("")
    }

    suspend fun parse(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        val path = if (context.request().method() == HttpMethod.POST) {
            context.bodyAsJson?.getString("path", "") ?: ""
        } else {
            context.queryParam("path").firstOrNull() ?: ""
        }
        if (path.isEmpty()) {
            return returnData.setErrorMsg("请输入文件路径")
        }

        val baseDir = getFileHome(context) ?: return returnData.setErrorMsg("非法访问")
        val file = resolveSecurePath(baseDir, path) ?: return returnData.setErrorMsg("路径不合法")

        if (!file.exists() || file.isDirectory) {
            return returnData.setErrorMsg("文件不存在")
        }

        val content = try {
            file.readText(Charsets.UTF_8)
        } catch (e: Exception) {
            return returnData.setErrorMsg("文件读取失败: ${e.message}")
        }

        // Try to parse as JSON
        try {
            val jsonArray = JsonArray(content)
            return returnData.setData(jsonArray.list)
        } catch (e: Exception) {
            // Not a JSON array
        }
        try {
            val jsonObj = JsonObject(content)
            return returnData.setData(jsonObj.map)
        } catch (e: Exception) {
            // Not a JSON object
        }

        return returnData.setData(content)
    }
}
