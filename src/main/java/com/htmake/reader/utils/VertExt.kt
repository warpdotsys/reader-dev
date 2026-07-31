@file:JvmName("ExtKt")
@file:JvmMultifileClass

package com.htmake.reader.utils

import com.google.gson.Gson
import com.google.gson.GsonBuilder
import io.vertx.core.json.JsonObject
import io.vertx.core.json.JsonArray
import mu.KotlinLogging
import java.io.File
import java.nio.file.Files
import java.nio.file.Paths
import java.nio.file.StandardCopyOption
import java.util.concurrent.TimeUnit
import java.util.concurrent.locks.ReadWriteLock
import java.util.concurrent.locks.ReentrantReadWriteLock
import com.htmake.reader.config.AppConfig
import com.google.gson.reflect.TypeToken
import kotlin.reflect.KProperty1
import kotlin.reflect.KMutableProperty
import kotlin.reflect.full.memberProperties
import io.legado.app.data.entities.Book
import io.legado.app.utils.FileUtils
import io.legado.app.utils.MD5Utils
import io.legado.app.utils.MapDeserializerDoubleAsIntFix
import java.util.UUID
import java.util.Base64 as JavaBase64
import java.security.KeyFactory
import java.security.spec.X509EncodedKeySpec
import io.legado.app.utils.EncoderUtils
import javax.net.ssl.SSLSocketFactory
import java.io.BufferedReader
import java.io.InputStreamReader
import java.io.OutputStreamWriter
import com.mongodb.client.MongoCollection
import com.htmake.reader.entity.MongoFile
import com.htmake.reader.entity.License
import com.fasterxml.jackson.core.JsonToken
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.databind.node.ObjectNode

/**
 * @Auther: zoharSoul
 * @Date: 2019-05-21 16:17
 * @Description:
 */
val logger = KotlinLogging.logger {}

val gson = GsonBuilder()
    .registerTypeAdapter(object : TypeToken<Map<String, Any>>() {}.type, MapDeserializerDoubleAsIntFix())
    .registerTypeAdapter(Int::class.javaPrimitiveType!!, IntTypeAdapter())
    .registerTypeAdapter(Long::class.javaPrimitiveType!!, LongTypeAdapter())
    .disableHtmlEscaping()
    .create()
val prettyGson = GsonBuilder()
    .registerTypeAdapter(object : TypeToken<Map<String, Any>>() {}.type, MapDeserializerDoubleAsIntFix())
    .registerTypeAdapter(Int::class.javaPrimitiveType!!, IntTypeAdapter())
    .registerTypeAdapter(Long::class.javaPrimitiveType!!, LongTypeAdapter())
    .disableHtmlEscaping()
    .setPrettyPrinting()
    .create()

var storageFinalPath = ""
var workDirPath = ""
var workDirInit = false
private const val MAX_CACHE_SIZE = 1000
private val storageLocks = LRUCache<String, ReadWriteLock>(MAX_CACHE_SIZE)

fun getWorkDir(subPath: String = ""): String {
    if (!workDirInit && workDirPath.isEmpty()) {
        val appConfig = SpringContextUtils.getBean("appConfig", AppConfig::class.java)
        if (appConfig != null && appConfig.workDir.isNotEmpty() && appConfig.workDir != ".") {
            val workDirFile = File(appConfig.workDir)
            if (workDirFile.exists() && !workDirFile.isDirectory) {
                logger.error("reader.app.workDir={} is not a directory", appConfig.workDir)
            } else {
                if (!workDirFile.exists()) {
                    logger.info("reader.app.workDir={} not exists, creating", appConfig.workDir)
                    workDirFile.mkdirs()
                }
                workDirPath = workDirFile.absolutePath
            }
        }
        if (workDirPath.isEmpty()) {
            val osName = System.getProperty("os.name")
            val currentDir = System.getProperty("user.dir")
            logger.info("osName: {} currentDir: {}", osName, currentDir)
            if (osName.startsWith("Mac OS", true) && !currentDir.startsWith("/Users/")) {
                workDirPath = Paths.get(System.getProperty("user.home"), ".reader").toString()
            } else {
                workDirPath = currentDir
            }
        }
        logger.info("Using workdir: {}", workDirPath)
        workDirInit = true
    }
    val path = Paths.get(workDirPath, subPath)

    return path.toString()
}

fun getWorkDir(vararg subDirFiles: String): String {
    return getWorkDir(getRelativePath(*subDirFiles))
}

fun getRelativePath(vararg subDirFiles: String): String {
    val path = StringBuilder("")
    subDirFiles.forEach {
        if (it.isNotEmpty()) {
            path.append(File.separator).append(it)
        }
    }
    return path.toString().let{
        if (it.startsWith("/")) {
            it.substring(1)
        } else {
            it
        }
    }
}

fun getStoragePath(): String {
    if (storageFinalPath.isNotEmpty()) {
        return storageFinalPath;
    }
    var storagePath = ""
    val appConfig = SpringContextUtils.getBean("appConfig", AppConfig::class.java)
    if (appConfig != null) {
        storagePath = getWorkDir("storage")
        storageFinalPath = storagePath
    } else {
        storagePath = File("storage").path
    }
    logger.info("Using storagePath: {}", storagePath)
    return storagePath;
}

fun saveStorage(vararg name: String, value: Any, pretty: Boolean = false, ext: String = ".json") {
    val toJson: String = if (value is String) {
        value
    } else if (value is JsonObject || value is JsonArray) {
        value.toString()
    } else if (pretty) {
        prettyGson.toJson(value)
    } else {
        gson.toJson(value)
    }

    val storagePath = getStoragePath()
    val storageDir = File(storagePath)
    if (!storageDir.exists()) {
        storageDir.mkdirs()
    }

    val filename = name.last()
    val path = getRelativePath(*name.copyOfRange(0, name.size - 1), "$filename$ext")
    val file = File(storagePath, path)
    logger.info("Save file to storage name: {} path: {}", name, file.absoluteFile)

    if (!file.parentFile.exists()) {
        file.parentFile.mkdirs()
    }

    val lock = storageLock(file)
    var acquired = false
    try {
        acquired = lock.writeLock().tryLock(10, TimeUnit.SECONDS)
        if (!acquired) {
            throw Exception("保存文件超时: ${file.absolutePath}")
        }

        val baseName = file.nameWithoutExtension
        val temp = Files.createTempFile(file.parentFile.toPath().toAbsolutePath(), baseName, ".temp")
        Files.write(temp, toJson.toByteArray(Charsets.UTF_8))

        val filePath = file.toPath()
        val backupPath = file.parentFile.toPath().resolve("$baseName.backup.json").toAbsolutePath()
        if (Files.exists(filePath)) {
            Files.move(filePath, backupPath, StandardCopyOption.ATOMIC_MOVE)
        }
        Files.move(temp, filePath, StandardCopyOption.ATOMIC_MOVE)
        Files.deleteIfExists(temp)

        if (baseName.length >= 32) {
            Files.deleteIfExists(backupPath)
        }
        if (baseName == "users") {
            val verifyFile = File(storagePath, getRelativePath(*name.copyOfRange(0, name.size - 1), ".$baseName.key"))
            if (!verifyFile.exists()) {
                verifyFile.createNewFile()
            }
            val verification = MD5Utils.md5Encode("userCount=${countOccurrences(toJson, "username")}").toString().takeLast(16)
            verifyFile.writeText(verification)
        }
        saveMongoFile(path, toJson)
    } catch (e: Exception) {
        logger.error("保存文件失败: ", e)
        throw Exception("保存文件失败: ${file.absolutePath}")
    } finally {
        if (acquired) {
            lock.writeLock().unlock()
        }
    }
}

fun getStorage(vararg name: String, ext: String = ".json"): String?  {
    val storagePath = getStoragePath()
    val storageDir = File(storagePath)
    if (!storageDir.exists()) {
        storageDir.mkdirs()
    }

    val filename = name.last()
    val path = getRelativePath(*name.copyOfRange(0, name.size - 1), "$filename$ext")
    val file = File(storagePath, path)
    logger.info("Read file from storage name: {} path: {}", name, file.absoluteFile)
    if (!file.exists()) {
        val content = readMongoFile(path)
        if (!content.isNullOrEmpty()) {
            if (!file.parentFile.exists()) {
                file.parentFile.mkdirs()
            }
            file.createNewFile()
            file.writeText(content)
            return content
        }
        return null
    }

    val lock = storageLock(file)
    var acquired = false
    try {
        acquired = lock.readLock().tryLock(10, TimeUnit.SECONDS)
        if (!acquired) {
            throw Exception("读取文件超时: ${file.absolutePath}")
        }
        var content = file.readText()
        if (content.isEmpty()) {
            val mongoContent = readMongoFile(path)
            if (!mongoContent.isNullOrEmpty()) {
                file.writeText(mongoContent)
                content = mongoContent
            }
        }
        if (filename == "users") {
            val verifyFile = File(storagePath, getRelativePath(*name.copyOfRange(0, name.size - 1), ".$filename.key"))
            if (verifyFile.exists()) {
                val verification = MD5Utils.md5Encode("userCount=${countOccurrences(content, "username")}").toString().takeLast(16)
                if (verifyFile.readText() != verification) {
                    throw Exception("用户数据被篡改，请联系开发者修复")
                }
            }
        }
        return content
    } catch (e: Exception) {
        logger.error("读取文件失败: ", e)
        throw Exception("读取文件失败: ${file.absolutePath}")
    } finally {
        if (acquired) {
            lock.readLock().unlock()
        }
    }
}

fun asJsonArray(value: Any?): JsonArray? {
    if (value is JsonArray) {
        return value
    } else if (value is String) {
        return try {
            JsonArray(value)
        } catch (e: Exception) {
            logger.error("解析内容出错: {}  内容: \n{}", e, value)
            throw e
        }
    }
    return null
}

fun asJsonObject(value: Any?): JsonObject? {
    if (value is JsonObject) {
        return value
    } else if (value is String) {
        return try {
            JsonObject(value)
        } catch (e: Exception) {
            logger.error("解析内容出错: {}  内容: \n{}", e, value)
            throw e
        }
    }
    return null
}

//convert a data class to a map
fun <T> T.serializeToMap(): Map<String, Any> {
    return convert()
}

//convert string to a map
fun <T> T.toMap(): Map<String, Any> {
    return convert()
}

//convert a map to a data class
inline fun <reified T> Map<String, Any>.toDataClass(): T {
    return convert()
}

//convert an object of type I to type O
inline fun <I, reified O> I.convert(): O {
    val json = if (this is String) {
        this
    } else {
        gson.toJson(this)
    }
    return gson.fromJson(json, object : TypeToken<O>() {}.type)
}

@Suppress("UNCHECKED_CAST")
fun <R> readInstanceProperty(instance: Any, propertyName: String): R {
    val property = instance::class.memberProperties
                     // don't cast here to <Any, R>, it would succeed silently
                     .first { it.name == propertyName } as KProperty1<Any, *>
    // force a invalid cast exception if incorrect type here
    return property.get(instance) as R
}

@Suppress("UNCHECKED_CAST")
fun setInstanceProperty(instance: Any, propertyName: String, propertyValue: Any) {
    val property = instance::class.memberProperties
                     .first { it.name == propertyName }
    if(property is KMutableProperty<*>) {
        property.setter.call(instance, propertyValue)
    }
}

fun Book.fillData(newBook: Book, keys: List<String>): Book {
    keys.let {
        for (key in it) {
            var current = readInstanceProperty<String>(this, key)
            if (current.isNullOrEmpty()) {
                var cacheValue = readInstanceProperty<String>(newBook, key)
                if (!cacheValue.isNullOrEmpty()) {
                    setInstanceProperty(this, key, cacheValue)
                }
            }
        }
    }
    return this
}

fun getRandomString(length: Int) : String {
    val allowedChars = "ABCDEFGHIJKLMNOPQRSTUVWXTZabcdefghiklmnopqrstuvwxyz0123456789"
    return (1..length)
        .map { allowedChars.random() }
        .joinToString("")
}

fun genEncryptedPassword(password: String, salt: String): String {
    return MD5Utils.md5Encode(
        MD5Utils.md5Encode(password + salt).toString() + salt
    ).toString()
}

fun jsonEncode(value: Any, pretty: Boolean = false): String {
    if (pretty) {
        return prettyGson.toJson(value)
    }
    return gson.toJson(value)
}

fun listFilesRecursively(dir: File): List<File> {
    val result = ArrayList<File>()
    if (!dir.exists()) {
        return result
    }
    if (dir.isFile) {
        result.add(dir)
        return result
    }
    val files = dir.listFiles()!!
    for (file in files) {
        result.add(file)
        if (file.isDirectory) {
            result.addAll(listFilesRecursively(file))
        }
    }
    return result
}

fun String.toDir(absolute: Boolean = false): String {
    var path = this
    if (path.endsWith("/")) {
        path = path.substring(0, path.length - 1)
    }
    if (absolute && !path.startsWith("/")) {
        path = "/$path"
    }
    return path
}

inline fun <reified T> arrayType(clazz: Class<T>): Class<Array<T>> {
    @Suppress("UNCHECKED_CAST")
    return java.lang.reflect.Array.newInstance(clazz, 0)::class.java as Class<Array<T>>
}

fun deepListFiles(dir: File, allowExtensions: Array<String>?): List<File> {
    val result = ArrayList<File>()
    val files = dir.listFiles() ?: return result
    for (file in files) {
        if (file.isDirectory) {
            result.addAll(deepListFiles(file, allowExtensions))
            continue
        }
        val extension = FileUtils.getExtension(file.name)
        if (allowExtensions == null || allowExtensions.contentDeepToString().contains(extension, ignoreCase = false)) {
            result.add(file)
        }
    }
    return result
}

fun getTraceId(): String {
    return UUID.randomUUID().toString().subSequence(0, 8).toString()
}

fun validateEmail(email: String): Boolean {
    val regex = Regex("^[A-Za-z0-9._%+-]+@(163|126|qq|yahoo|sina|sohu|yeah|139|189|21cn|outlook|gmail|icloud).com$")
    return regex.matches(email)
}

fun encodeBase64(text: String): String {
    return JavaBase64.getEncoder().encodeToString(text.toByteArray(Charsets.UTF_8))
}

var _licenseValid: Boolean = true

fun setLicenseValid(value: Boolean) {
    _licenseValid = value
}

fun getStorageFile(vararg name: String, ext: String = ".json"): File {
    val storagePath = getStoragePath()
    val storageDir = File(storagePath)
    if (!storageDir.exists()) {
        storageDir.mkdirs()
    }

    val filename = name.last()
    val relativePath = getRelativePath(*name.copyOfRange(0, name.size - 1), "${filename}${ext}")
    return File(storagePath, relativePath)
}

private fun storageLock(file: File): ReadWriteLock {
    synchronized(storageLocks) {
        return storageLocks.get(file.absolutePath)
            ?: ReentrantReadWriteLock().also { storageLocks.put(file.absolutePath, it) }
    }
}

fun getMongoFileStorage(): MongoCollection<MongoFile>? {
    val appConfig = SpringContextUtils.getBean("appConfig", AppConfig::class.java)
    return MongoManager.fileStorage(appConfig.mongoDbName, "storage")
}

fun readMongoFile(path: String): String? {
    if (!MongoManager.isInit()) {
        return null
    }
    logger.info("Get mongoFile {}", path)
    val collection = getMongoFileStorage() ?: return null
    val doc = collection.find(com.mongodb.client.model.Filters.eq("path", path)).first()
    return doc?.content
}

fun saveMongoFile(path: String, content: String): Boolean {
    if (!MongoManager.isInit()) {
        return false
    }
    logger.info("Save mongoFile {}", path)
    val collection = getMongoFileStorage() ?: return false
    val filter = com.mongodb.client.model.Filters.eq("path", path)
    val existing = collection.find(filter).first()
    if (existing != null) {
        existing.content = content
        existing.updated_at = System.currentTimeMillis()
        val result = collection.replaceOne(
            filter,
            existing,
            com.mongodb.client.model.ReplaceOptions().upsert(true)
        )
        return result.modifiedCount > 0L
    }
    return try {
        collection.insertOne(MongoFile(path = path, content = content))
        true
    } catch (e: Exception) {
        logger.info("Save mongoFile {} failed", path)
        e.printStackTrace()
        false
    }
}

fun countOccurrences(text: String, sub: String): Int {
    if (sub.isEmpty()) return 0
    var count = 0
    var index = 0
    while (true) {
        index = text.indexOf(sub, index)
        if (index == -1) break
        count++
        index += sub.length
    }
    return count
}

fun getInstalledLicense(ignoreInvalid: Boolean = false): License {
    val licenseKeyString = getStorage("data", "license", ext = ".key")
    if (licenseKeyString.isNullOrEmpty() || (!ignoreInvalid && !_licenseValid)) {
        return License()
    }
    val license = decryptToLicense(licenseKeyString)
    logger.info("license: {}", license)
    return license?.takeIf { it.verified } ?: License()
}

fun decryptToLicense(encrypted: String): License? {
    if (encrypted.isEmpty()) return null
    val decrypted = decryptData(encrypted) ?: return null
    return decrypted.toMap().toDataClass()
}

fun decryptData(content: String): String? {
    val publicKeyString = "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAj0G3qEPjVTvVd7pXFUVYZFHT8KaoG4onc5rLUKqFQ2DCh/5hFK9t2nKh2XB+C2Jp/GSK2ONwD7ceXenmA6uvr90uCK/gp6j62XFVRvc8sIm0d/bGbzZFJRk3HKtxEckBmASduPObY691DVVixxNtUrSJktx/TZaB42pUQk4j+7FuOVNNPra44hDdnyGhmYBBf2B4kjXVMjL+0NCblFIN1+qjmcol44k6NFKFF54q05bjR3CRyYdAnNTCOyt9va0oB6lDlKHplSZmAOH9JGMUki/HDJbABESXMnyIpux27w9SQ8aJStYttnJWHALO1hiFJsxbz5KUkldH6Ny1p/2W5QIDAQAB"
    val publicKey = KeyFactory.getInstance("RSA").generatePublic(
        X509EncodedKeySpec(JavaBase64.getDecoder().decode(publicKeyString))
    )
    return EncoderUtils.decryptSegmentByPublicKey(content, publicKey)
}

fun sendEmail(toEmail: String, subject: String, body: String): Boolean {
    val host = "smtp.qiye.aliyun.com"
    val port = 465
    return try {
        val socket = SSLSocketFactory.getDefault().createSocket(host, port)
        val writer = OutputStreamWriter(socket.getOutputStream())
        val reader = BufferedReader(InputStreamReader(socket.getInputStream(), Charsets.UTF_8))
        val response = reader.readLine()
        if (response?.startsWith("220") != true) {
            logger.error("Error connecting to the SMTP server.")
            writer.close()
            reader.close()
            socket.close()
            return false
        }
        val commandList = getCommand(listOf(toEmail), subject, body)
        var result = false
        var index = 0
        while (index < commandList.size && sendEmailCommand(writer, reader, commandList[index])) {
            result = true
            index++
        }
        writer.close()
        reader.close()
        socket.close()
        result && index == commandList.size
    } catch (e: Exception) {
        e.printStackTrace()
        false
    }
}

private fun sendEmailCommand(
    writer: OutputStreamWriter,
    reader: BufferedReader,
    command: Pair<String, Int>
): Boolean {
    val (value, expected) = command
    logger.debug("Send command {}, expect code {}", value.trim(), expected)
    writer.write(value)
    writer.flush()
    val response = reader.readLine()
    logger.debug("Response {}", response)
    if (response.isNullOrEmpty()) {
        logger.error("SMTP server no response.")
        return false
    }
    if (!response.startsWith(expected.toString())) {
        logger.error("Error response from SMTP server.")
        return false
    }
    return true
}

fun getCommand(to: List<String>, subject: String, body: String): List<Pair<String, Int>> {
    val username = "no-reply@onmy.top"
    val password = "no-reply@1."
    val from = "no-reply@onmy.top"
    val fromName = "Reader"
    val separator = "----=_Part_${System.currentTimeMillis()}${UUID.randomUUID()}"
    val commands = mutableListOf("HELO sendmail\r\n" to 250)

    if (username.isNotEmpty()) {
        commands.add("AUTH LOGIN\r\n" to 334)
        commands.add("${encodeBase64(username)}\r\n" to 334)
        commands.add("${encodeBase64(password)}\r\n" to 235)
    }
    commands.add("MAIL FROM: <$from>\r\n" to 250)

    var header = "FROM: $fromName<$from>\r\n"
    to.forEachIndexed { index, recipient ->
        commands.add("RCPT TO: <$recipient>\r\n" to 250)
        header += when {
            to.size == 1 -> "TO: <$recipient>\r\n"
            index == 0 -> "TO: <$recipient>"
            index == to.lastIndex -> ",<$recipient>\r\n"
            else -> ",<$recipient>"
        }
    }
    header += "Subject: =?UTF-8?B?${encodeBase64(subject)}?=\r\n"
    header += "Content-Type: multipart/alternative;\r\n\tboundary=\"$separator\"\r\nMIME-Version: 1.0\r\n"
    header += "\r\n--$separator\r\nContent-Type:text/html; charset=utf-8\r\nContent-Transfer-Encoding: base64\r\n\r\n"
    header += "${encodeBase64(body)}\r\n--$separator\r\n\r\n.\r\n"
    commands.add("DATA\r\n" to 354)
    commands.add(header to 250)
    commands.add("QUIT\r\n" to 221)
    return commands
}

fun parseJsonStringList(
    file: File,
    fields: Set<String>? = null,
    exclude: Set<String>? = null,
    startIndex: Int = 0,
    endIndex: Int = Int.MAX_VALUE,
    checkNotEmpty: Set<String>? = null,
    filter: ((ObjectNode) -> Boolean)? = null
): JsonArray? {
    if (!file.exists()) {
        return null
    }
    return try {
        val objectMapper = ObjectMapper()
        val resultList = JsonArray()
        var currentIndex = -1
        objectMapper.factory.createParser(file).use { parser ->
            if (parser.nextToken() == JsonToken.START_ARRAY) {
                while (parser.nextToken() != JsonToken.END_ARRAY) {
                    if (parser.currentToken() != JsonToken.START_OBJECT) {
                        continue
                    }
                    if (fields.isNullOrEmpty()) {
                        if (filter == null) {
                            currentIndex++
                            if (currentIndex < startIndex) {
                                parser.skipChildren()
                                continue
                            }
                            if (currentIndex > endIndex) {
                                break
                            }
                            val objectNode = parser.readValueAsTree<ObjectNode>()
                            exclude?.forEach { objectNode.remove(it) }
                            resultList.add(objectNode.toString())
                            continue
                        }
                        val objectNode = parser.readValueAsTree<ObjectNode>()
                        if (filter(objectNode)) {
                            currentIndex++
                        }
                        if (currentIndex < startIndex) {
                            continue
                        }
                        if (currentIndex > endIndex) {
                            break
                        }
                        resultList.add(objectNode.toString())
                        continue
                    }

                    currentIndex++
                    if (currentIndex < startIndex) {
                        parser.skipChildren()
                        continue
                    }
                    if (currentIndex > endIndex) {
                        break
                    }
                    val item = JsonObject()
                    while (parser.nextToken() != JsonToken.END_OBJECT) {
                        val fieldName = parser.currentName
                        parser.nextToken()
                        when {
                            fields.contains(fieldName) -> item.put(fieldName, parser.valueAsString)
                            checkNotEmpty?.contains(fieldName) == true -> item.put(
                                fieldName,
                                !parser.valueAsString.isNullOrEmpty()
                            )
                            else -> parser.skipChildren()
                        }
                    }
                    resultList.add(item.toString())
                }
            }
        }
        resultList
    } catch (e: Exception) {
        logger.error("解析文件内容出错: {} 文件: \n{}", e, file)
        throw e
    }
}
