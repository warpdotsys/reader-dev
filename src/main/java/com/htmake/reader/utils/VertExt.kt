package com.htmake.reader.utils

import com.google.common.base.Throwables
import com.google.gson.Gson
import com.google.gson.GsonBuilder
import io.vertx.core.Handler
import io.vertx.core.json.JsonObject
import io.vertx.core.json.JsonArray
import io.vertx.ext.web.RoutingContext
import mu.KotlinLogging
import com.htmake.reader.entity.BasicError
import java.net.URLDecoder
import java.net.URLEncoder
import java.io.File
import java.nio.file.Paths
import com.htmake.reader.config.AppConfig
import com.google.gson.reflect.TypeToken
import java.lang.reflect.ParameterizedType
import java.lang.reflect.Type
import kotlin.reflect.KProperty1
import kotlin.reflect.KMutableProperty
import kotlin.reflect.full.memberProperties
import io.legado.app.data.entities.Book
import io.legado.app.utils.MD5Utils
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
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.databind.node.ObjectNode

/**
 * @Auther: zoharSoul
 * @Date: 2019-05-21 16:17
 * @Description:
 */
val logger = KotlinLogging.logger {}

val gson = GsonBuilder().disableHtmlEscaping().create()
val prettyGson = GsonBuilder().setPrettyPrinting().disableHtmlEscaping().create()

var storageFinalPath = ""
var workDirPath = ""
var workDirInit = false

fun RoutingContext.success(any: Any?) {
    val toJson: String = if (any is JsonObject) {
        any.toString()
    } else {
        gson.toJson(any)
    }
    this.response()
            .putHeader("content-type", "application/json; charset=utf-8")
            .end(toJson)
}

fun RoutingContext.error(throwable: Throwable) {
    val path = URLDecoder.decode(this.request().absoluteURI(), "UTF-8")
    val basicError = BasicError(
            "Internal Server Error",
            throwable.toString(),
            throwable.message.toString(),
            path,
            500,
            System.currentTimeMillis()
    )

    val errorJson = gson.toJson(basicError)
    logger.error("Internal Server Error", throwable)
    logger.error { errorJson }

    this.response()
            .putHeader("content-type", "application/json; charset=utf-8")
            .setStatusCode(500)
            .end(errorJson)
}

fun getWorkDir(subPath: String = ""): String {
    if (!workDirInit && workDirPath.isEmpty()) {
        var osName = System.getProperty("os.name")
        var currentDir = System.getProperty("user.dir")
        logger.info("osName: {} currentDir: {}", osName, currentDir)
        // MacOS 存放目录为用户目录
        if (osName.startsWith("Mac OS", true) && !currentDir.startsWith("/Users/")) {
            workDirPath = Paths.get(System.getProperty("user.home"), ".reader").toString()
        } else {
            workDirPath = currentDir
        }
        workDirInit = true
    }
    var path = Paths.get(workDirPath, subPath);

    return path.toString();
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
    var appConfig = SpringContextUtils.getBean("appConfig", AppConfig::class.java)
    var storageDir = File("storage")
    if (appConfig != null) {
        // logger.info("storagePath from appConfig: {}", appConfig.storagePath)
        storageDir = File(appConfig.storagePath)
    }
    if (storageDir.isAbsolute()) {
        return storageDir.toString();
    }
    var storagePath = getWorkDir(storageDir.toString())
    if (appConfig != null) {
        storageFinalPath = storagePath
    }
    return storagePath;
}

fun saveStorage(vararg name: String, value: Any, pretty: Boolean = false) {
    val toJson: String = if (value is JsonObject || value is JsonArray) {
        value.toString()
    } else if (pretty) {
        prettyGson.toJson(value)
    } else {
        gson.toJson(value)
    }

    var storagePath = getStoragePath()
    var storageDir = File(storagePath)
    if (!storageDir.exists()) {
        storageDir.mkdirs()
    }

    val filename = name.last()
    val file = File(getRelativePath(storagePath, *name.copyOfRange(0, name.size - 1), "${filename}.json"))
    // val file = File(storagePath + "/${name}.json")
    logger.info("Save file to storage name: {} path: {}", name, file.absoluteFile)

    if (!file.parentFile.exists()) {
        file.parentFile.mkdirs()
    }

    if (!file.exists()) {
        file.createNewFile()
    }
    file.writeText(toJson)
}

fun getStorage(vararg name: String): String?  {
    var storagePath = getStoragePath()
    var storageDir = File(storagePath)
    if (!storageDir.exists()) {
        storageDir.mkdirs()
    }

    val filename = name.last()
    val file = File(getRelativePath(storagePath, *name.copyOfRange(0, name.size - 1), "${filename}.json"))
    logger.info("Read file from storage name: {} path: {}", name, file.absoluteFile)
    if (!file.exists()) {
        return null
    }
    return file.readText()
}

fun asJsonArray(value: Any?): JsonArray? {
    if (value is JsonArray) {
        return value
    } else if (value is String) {
        return JsonArray(value)
    }
    return null
}

fun asJsonObject(value: Any?): JsonObject? {
    if (value is JsonObject) {
        return value
    } else if (value is String) {
        return JsonObject(value)
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

fun parseJsonStringList(jsonStr: String?): List<String> {
    if (jsonStr.isNullOrBlank()) {
        return emptyList()
    }
    return try {
        gson.fromJson(jsonStr, object : TypeToken<List<String>>() {}.type)
    } catch (e: Exception) {
        emptyList()
    }
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
        if (file.isFile) {
            result.add(file)
        } else if (file.isDirectory) {
            result.addAll(listFilesRecursively(file))
        }
    }
    return result
}

fun String.toDir(removeTrailing: Boolean = false): String {
    return if (removeTrailing) {
        if (this.endsWith("/")) {
            this.substring(0, this.length - 1)
        } else {
            this
        }
    } else {
        if (this.endsWith(File.separator)) {
            this
        } else {
            this + File.separator
        }
    }
}

inline fun <reified T> arrayType(clazz: Class<T>): Class<Array<T>> {
    @Suppress("UNCHECKED_CAST")
    return java.lang.reflect.Array.newInstance(clazz, 0)::class.java as Class<Array<T>>
}

fun deepListFiles(dir: File, vararg extensions: String): List<File> {
    val result = ArrayList<File>()
    if (!dir.exists()) {
        return result
    }
    if (dir.isFile) {
        if (extensions.isEmpty() || extensions.any { dir.name.endsWith(it, ignoreCase = true) }) {
            result.add(dir)
        }
        return result
    }
    val files = dir.listFiles() ?: return result
    for (file in files) {
        if (file.isFile) {
            if (extensions.isEmpty() || extensions.any { file.name.endsWith(it, ignoreCase = true) }) {
                result.add(file)
            }
        } else if (file.isDirectory) {
            result.addAll(deepListFiles(file, *extensions))
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

fun decodeBase64(text: String): String {
    return String(JavaBase64.getDecoder().decode(text), Charsets.UTF_8)
}

var _licenseValid: Boolean = true

fun setLicenseValid(value: Boolean) {
    _licenseValid = value
}

fun getStorageFile(vararg name: String, ext: String = ".json"): File {
    var storagePath = getStoragePath()
    var storageDir = File(storagePath)
    if (!storageDir.exists()) {
        storageDir.mkdirs()
    }

    val filename = name.last()
    val relativePath = getRelativePath(storagePath, *name.copyOfRange(0, name.size - 1), "${filename}${ext}")
    return File(storagePath + File.separator + relativePath)
}

fun getMongoFileStorage(): MongoCollection<MongoFile>? {
    val appConfig = SpringContextUtils.getBean("appConfig", AppConfig::class.java) ?: return null
    return MongoManager.fileStorage(appConfig.mongoDbName, "files")
}

fun readMongoFile(path: String): String {
    val collection = getMongoFileStorage() ?: return ""
    val filter = com.mongodb.client.model.Filters.eq("path", path)
    val doc = collection.find(filter).first() ?: return ""
    return doc.content
}

fun saveMongoFile(path: String, content: String): Boolean {
    val collection = getMongoFileStorage() ?: return false
    val filter = com.mongodb.client.model.Filters.eq("path", path)
    val existing = collection.find(filter).first()
    return try {
        if (existing != null) {
            val update = com.mongodb.client.model.Updates.combine(
                com.mongodb.client.model.Updates.set("content", content),
                com.mongodb.client.model.Updates.set("updated_at", System.currentTimeMillis())
            )
            collection.updateOne(filter, update)
        } else {
            val mongoFile = MongoFile(path = path, content = content)
            collection.insertOne(mongoFile)
        }
        true
    } catch (e: Exception) {
        logger.error("Failed to save mongo file: {}", e.message)
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
    try {
        val licenseFile = getStorageFile("data", "license", ext = ".key")
        if (!licenseFile.exists()) return License()
        val licenseStr = licenseFile.readText()
        if (licenseStr.isBlank() || (!ignoreInvalid && !_licenseValid)) return License()
        val license = decryptToLicense(licenseStr)
        logger.info("license: {}", license)
        return license?.takeIf { it.verified } ?: License()
    } catch (e: Exception) {
        logger.error("Failed to get installed license: {}", e.message)
        return License()
    }
}

fun decryptToLicense(encrypted: String): License? {
    return try {
        val decrypted = decryptData(encrypted)
        if (decrypted.isBlank()) return null
        gson.fromJson(decrypted, License::class.java)
    } catch (e: Exception) {
        logger.error("Failed to decrypt license: {}", e.message)
        null
    }
}

fun decryptData(encrypted: String): String {
    return try {
        val publicKey = KeyFactory.getInstance("RSA").generatePublic(
            X509EncodedKeySpec(JavaBase64.getDecoder().decode("MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAj0G3qEPjVTvVd7pXFUVYZFHT8KaoG4onc5rLUKqFQ2DCh/5hFK9t2nKh2XB+C2Jp/GSK2ONwD7ceXenmA6uvr90uCK/gp6j62XFVRvc8sIm0d/bGbzZFJRk3HKtxEckBmASduPObY691DVVixxNtUrSJktx/TZaB42pUQk4j+7FuOVNNPra44hDdnyGhmYBBf2B4kjXVMjL+0NCblFIN1+qjmcol44k6NFKFF54q05bjR3CRyYdAnNTCOyt9va0oB6lDlKHplSZmAOH9JGMUki/HDJbABESXMnyIpux27w9SQ8aJStYttnJWHALO1hiFJsxbz5KUkldH6Ny1p/2W5QIDAQAB"))
        )
        EncoderUtils.decryptSegmentByPublicKey(encrypted, publicKey)
    } catch (e: Exception) {
        logger.error("Failed to decrypt data: {}", e.message)
        ""
    }
}

fun sendEmail(to: String, subject: String, body: String): Boolean {
    val host = System.getProperty("reader.smtp.host") ?: System.getenv("READER_SMTP_HOST") ?: return false
    val port = (System.getProperty("reader.smtp.port") ?: System.getenv("READER_SMTP_PORT") ?: "465").toIntOrNull() ?: return false
    val username = System.getProperty("reader.smtp.username") ?: System.getenv("READER_SMTP_USERNAME") ?: return false
    val password = System.getProperty("reader.smtp.password") ?: System.getenv("READER_SMTP_PASSWORD") ?: return false
    val from = System.getProperty("reader.smtp.from") ?: System.getenv("READER_SMTP_FROM") ?: username

    return runCatching {
        val socket = SSLSocketFactory.getDefault().createSocket(host, port)
        socket.use { securedSocket ->
            val reader = BufferedReader(InputStreamReader(securedSocket.inputStream, Charsets.UTF_8))
            val writer = OutputStreamWriter(securedSocket.outputStream, Charsets.UTF_8)
            fun command(value: String, expected: Int): Boolean {
                writer.write(value)
                writer.flush()
                return reader.readLine()?.startsWith(expected.toString()) == true
            }
            if (reader.readLine()?.startsWith("220") != true) return@use false
            if (!command("EHLO reader\r\n", 250) || !command("AUTH LOGIN\r\n", 334)) return@use false
            if (!command("${encodeBase64(username)}\r\n", 334) || !command("${encodeBase64(password)}\r\n", 235)) return@use false
            if (!command("MAIL FROM:<$from>\r\n", 250) || !command("RCPT TO:<$to>\r\n", 250) || !command("DATA\r\n", 354)) return@use false
            val boundary = "----Reader-${UUID.randomUUID()}"
            val message = buildString {
                append("From: <$from>\r\nTo: <$to>\r\n")
                append("Subject: =?UTF-8?B?${encodeBase64(subject)}?=\r\n")
                append("MIME-Version: 1.0\r\nContent-Type: multipart/alternative; boundary=\"$boundary\"\r\n\r\n")
                append("--$boundary\r\nContent-Type: text/plain; charset=UTF-8\r\nContent-Transfer-Encoding: base64\r\n\r\n")
                append(encodeBase64(body)).append("\r\n--$boundary--\r\n.\r\n")
            }
            val delivered = command(message, 250)
            command("QUIT\r\n", 221)
            delivered
        }
    }.onFailure { logger.error("Failed to send email", it) }.getOrDefault(false)
}

fun getCommand(commands: List<String>, workDir: String = "", timeout: String = ""): List<Pair<String, Int>> {
    val results = mutableListOf<Pair<String, Int>>()
    for (command in commands) {
        try {
            val parts = command.split(" ")
            val processBuilder = ProcessBuilder(parts)
            if (workDir.isNotEmpty()) {
                processBuilder.directory(File(workDir))
            }
            processBuilder.redirectErrorStream(true)
            val process = processBuilder.start()
            val output = process.inputStream.bufferedReader().readText()
            val exitCode = if (timeout.isNotEmpty()) {
                val timeoutMs = try { timeout.toLong() } catch (e: Exception) { 30000L }
                process.waitFor(timeoutMs, java.util.concurrent.TimeUnit.MILLISECONDS)
                process.exitValue()
            } else {
                process.waitFor()
            }
            results.add(Pair(output, exitCode))
        } catch (e: Exception) {
            logger.error("Failed to execute command '{}': {}", command, e.message)
            results.add(Pair(e.message ?: "error", -1))
        }
    }
    return results
}

fun parseJsonStringList(
    file: File,
    includeKeys: Set<String> = emptySet(),
    excludeKeys: Set<String> = emptySet(),
    offset: Int = 0,
    limit: Int = Int.MAX_VALUE,
    filterKeys: Set<String> = emptySet(),
    filter: ((ObjectNode) -> Boolean)? = null
): JsonArray {
    val result = JsonArray()
    if (!file.exists()) return result
    try {
        val mapper = ObjectMapper()
        val tree = mapper.readTree(file)
        if (!tree.isArray) return result

        var index = 0
        var added = 0
        for (element in tree) {
            if (element !is ObjectNode) {
                index++
                continue
            }

            // Apply filter function
            if (filter != null && !filter(element)) {
                index++
                continue
            }

            // Apply filterKeys: only include items where any filterKey field is non-empty
            if (filterKeys.isNotEmpty()) {
                val matchesFilter = filterKeys.any { key ->
                    val node = element.get(key)
                    node != null && !node.isNull && node.asText().isNotBlank()
                }
                if (!matchesFilter) {
                    index++
                    continue
                }
            }

            // Apply offset
            if (index < offset) {
                index++
                continue
            }

            // Apply limit
            if (added >= limit) break

            // Build filtered object with includeKeys/excludeKeys
            val obj = JsonObject()
            val fieldNames = element.fieldNames()
            while (fieldNames.hasNext()) {
                val fieldName = fieldNames.next()
                if (includeKeys.isNotEmpty() && fieldName !in includeKeys) continue
                if (excludeKeys.isNotEmpty() && fieldName in excludeKeys) continue
                val value = element.get(fieldName)
                when {
                    value.isTextual -> obj.put(fieldName, value.asText())
                    value.isInt -> obj.put(fieldName, value.asInt())
                    value.isLong -> obj.put(fieldName, value.asLong())
                    value.isDouble -> obj.put(fieldName, value.asDouble())
                    value.isBoolean -> obj.put(fieldName, value.asBoolean())
                    value.isNull -> obj.putNull(fieldName)
                    else -> obj.put(fieldName, value.toString())
                }
            }
            result.add(obj)
            added++
            index++
        }
    } catch (e: Exception) {
        logger.error("Failed to parse JSON string list from file {}: {}", file.path, e.message)
    }
    return result
}
