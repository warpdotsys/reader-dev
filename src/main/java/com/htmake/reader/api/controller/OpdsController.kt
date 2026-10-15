package com.htmake.reader.api.controller

import com.htmake.reader.api.ReturnData
import com.htmake.reader.utils.asJsonObject
import com.htmake.reader.utils.asJsonArray
import com.htmake.reader.utils.getStorage
import com.htmake.reader.utils.getWorkDir
import com.htmake.reader.utils.saveStorage
import io.legado.app.data.entities.Book
import io.vertx.core.json.JsonObject
import io.vertx.core.buffer.Buffer
import io.vertx.ext.web.RoutingContext
import java.io.File
import java.net.URLEncoder
import java.nio.charset.StandardCharsets
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.LinkOption
import java.nio.file.SecureDirectoryStream
import java.nio.file.StandardOpenOption
import java.nio.channels.SeekableByteChannel
import java.security.MessageDigest
import java.security.SecureRandom
import java.time.Instant
import java.util.Base64
import java.util.LinkedHashMap
import java.util.concurrent.Semaphore
import javax.crypto.SecretKeyFactory
import javax.crypto.spec.PBEKeySpec
import kotlin.coroutines.CoroutineContext
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException

/**
 * OPDS 1.2 endpoint.  It deliberately does not reuse Reader sessions or access
 * tokens: an OPDS client is authenticated only with a separately configured
 * HTTP Basic account.  The persistent registry contains a salted PBKDF2 hash,
 * never the OPDS password itself.
 */
class OpdsController(coroutineContext: CoroutineContext) : BaseController(coroutineContext) {
    suspend fun getSettings(context: RoutingContext): ReturnData {
        if (!appConfig.secure) return ReturnData().setErrorMsg("OPDS 需要启用用户登录")
        if (hasNamespaceOverride(context)) return ReturnData().setErrorMsg("OPDS 设置不允许指定用户空间")
        if (!checkAuth(context)) return ReturnData().setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        val owner = authenticatedOwner(context) ?: return ReturnData().setErrorMsg("当前用户无效")
        val setting = loadSettings(owner)
        return ReturnData().setData(settingsView(setting, owner))
    }

    suspend fun saveSettings(context: RoutingContext): ReturnData {
        if (!appConfig.secure) return ReturnData().setErrorMsg("OPDS 需要启用用户登录")
        if (hasNamespaceOverride(context)) return ReturnData().setErrorMsg("OPDS 设置不允许指定用户空间")
        if (!checkAuth(context)) return ReturnData().setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        val owner = authenticatedOwner(context) ?: return ReturnData().setErrorMsg("当前用户无效")
        val body = context.bodyAsJson ?: return ReturnData().setErrorMsg("参数错误")
        val username = body.getString("username", "").trim()
        val password = body.getString("password", "")
        return registryMutex.withLock {
            val current = loadSettings(owner)
            if (username.isEmpty()) {
                current?.getString("username")?.let { configured ->
                    val registry = loadRegistry()
                    if (registry.getJsonObject(configured)?.getString("owner") == owner) {
                        registry.remove(configured)
                        saveRegistry(registry)
                    }
                }
                // Keep an empty, lock-protected settings document rather than
                // deleting a storage file behind the shared storage lock.
                saveSettings(owner, JsonObject())
                return@withLock ReturnData().setData(settingsView(null, owner))
            }
            if (!isValidOpdsUsername(username)) {
                return@withLock ReturnData().setErrorMsg("OPDS 用户名须为 3-64 位 ASCII 字母、数字、点、下划线或连字符")
            }
            val currentUsername = current?.getString("username")
            val reusingPassword = username == currentUsername && password.isEmpty()
            if (!reusingPassword && !isValidOpdsPassword(password)) {
                return@withLock ReturnData().setErrorMsg("OPDS 密码须为 12-128 个字符")
            }
            val registry = loadRegistry()
            val existing = registry.getJsonObject(username)
            if (existing != null && existing.getString("owner") != owner) {
                return@withLock ReturnData().setErrorMsg("OPDS 用户名已被占用")
            }
            if (currentUsername != null && currentUsername != username) {
                if (registry.getJsonObject(currentUsername)?.getString("owner") == owner) {
                    registry.remove(currentUsername)
                }
            }
            val credentials = if (reusingPassword) {
                existing?.takeIf { recordOwnerMatches(it, owner) }
                    ?: return@withLock ReturnData().setErrorMsg("原 OPDS 凭据不可用，请重新设置密码")
            } else {
                // The route coroutine already runs away from Vert.x's event loop;
                // keep the intentionally expensive KDF explicit here as well.
                withContext(Dispatchers.IO) { passwordRecord(owner, password, ownerCreationTime(owner)) }
            }
            registry.put(username, credentials)
            saveRegistry(registry)
            saveSettings(owner, JsonObject().put("username", username))
            ReturnData().setData(settingsView(JsonObject().put("username", username), owner))
        }
    }

    suspend fun catalog(context: RoutingContext) {
        val owner = requireBasicOwner(context) ?: return
        val onlineBooks = listShelfBooks(owner).filter { !it.isLocalBook() && isSafeOnlineLink(it.bookUrl) }
        context.response().putHeader("Content-Type", OPDS_NAVIGATION_TYPE)
            .putHeader("Cache-Control", "no-store")
            .end(feed("Reader OPDS", "catalog-$owner", "/reader3/opds/catalog", true, buildString {
                append(navigationEntry("本地书籍", "/reader3/opds/local", "本地 TXT、EPUB、PDF 可直接下载", "subsection", OPDS_ACQUISITION_TYPE))
                if (onlineBooks.isNotEmpty()) {
                    append(navigationEntry("网络书籍（仅元数据）", "/reader3/opds/online", "网络书未作为本地文件缓存，不能通过 OPDS 下载", "subsection", OPDS_NAVIGATION_TYPE))
                }
            }))
    }

    suspend fun localCatalog(context: RoutingContext) {
        val owner = requireBasicOwner(context) ?: return
        val entries = listShelfBooks(owner)
            .filter { it.isLocalBook() }
            .mapNotNull { book -> downloadableFile(book, owner)?.let { file -> acquisitionEntry(book, owner, file) } }
        context.response().putHeader("Content-Type", OPDS_ACQUISITION_TYPE).putHeader("Cache-Control", "no-store")
            .end(feed("本地书籍", "local-$owner", "/reader3/opds/local", false, entries.joinToString("")))
    }

    suspend fun onlineCatalog(context: RoutingContext) {
        val owner = requireBasicOwner(context) ?: return
        // These are navigation entries on purpose: an uncached network book has no
        // local acquisition resource and must not be advertised as downloadable.
        val entries = listShelfBooks(owner).filter { !it.isLocalBook() && isSafeOnlineLink(it.bookUrl) }.joinToString("") { book ->
            "<entry><id>${xml("urn:reader:opds:online:${stableId(book.bookUrl)}")}</id><title>${xml(book.name)}</title>" +
                "<updated>${Instant.ofEpochMilli(book.lastCheckTime).toString()}</updated><content type=\"text\">网络书籍未缓存，OPDS 不提供下载。</content>" +
                "<link rel=\"alternate\" href=\"${xml(book.bookUrl)}\" type=\"text/html\"/></entry>"
        }
        context.response().putHeader("Content-Type", OPDS_NAVIGATION_TYPE).putHeader("Cache-Control", "no-store")
            .end(feed("网络书籍（仅元数据）", "online-$owner", "/reader3/opds/online", true, entries))
    }

    suspend fun download(context: RoutingContext) {
        val owner = requireBasicOwner(context) ?: return
        val encoded = context.pathParam("id") ?: run { context.response().setStatusCode(404).end(); return }
        val bookUrl = try { String(Base64.getUrlDecoder().decode(encoded), StandardCharsets.UTF_8) } catch (_: IllegalArgumentException) { "" }
        if (bookUrl.isEmpty()) { context.response().setStatusCode(404).end(); return }
        // Do not enter BookController here.  OPDS intentionally operates on the
        // saved shelf snapshot and must not trigger normal shelf refresh logic.
        val book = listShelfBooks(owner).firstOrNull { it.bookUrl == bookUrl }
        val file = book?.takeIf { it.isLocalBook() }?.let { downloadableFile(it, owner) }
        if (file == null) { context.response().setStatusCode(404).end(); return }
        // Do not call sendFile after a pathname check: that would reopen the
        // pathname and permit a malicious symlink replacement in between.
        val channel = openOwnedFile(owner, file.toPath()) ?: run { context.response().setStatusCode(404).end(); return }
        context.response().putHeader("Content-Type", mediaType(file.name))
            .putHeader("Content-Disposition", "attachment; filename*=UTF-8''${URLEncoder.encode(file.name, "UTF-8").replace("+", "%20")}")
            .putHeader("Cache-Control", "no-store").setChunked(true)
        if (!downloadWork.tryAcquire()) {
            channel.close()
            context.response().setStatusCode(503).end()
            return
        }
        try {
            withContext(Dispatchers.IO) { streamOpenedFile(context, channel) }
        } finally {
            downloadWork.release()
        }
    }

    private fun hasNamespaceOverride(context: RoutingContext): Boolean =
        context.queryParam("userNS").isNotEmpty() || context.queryParam("ns").isNotEmpty()

    private fun authenticatedOwner(context: RoutingContext): String? {
        val owner = context.get<String>("username") ?: if (appConfig.secure) return null else "default"
        return owner.takeIf { it.isNotBlank() }
    }

    private suspend fun requireBasicOwner(context: RoutingContext): String? {
        val header = context.request().getHeader("Authorization") ?: return basicDenied(context)
        val encoded = Regex("(?i)^Basic ([A-Za-z0-9+/]+={0,2})$").matchEntire(header.trim())?.groupValues?.get(1)
            ?: return basicDenied(context)
        val pair = try { String(Base64.getDecoder().decode(encoded), StandardCharsets.UTF_8) } catch (_: IllegalArgumentException) { return basicDenied(context) }
        val split = pair.indexOf(':')
        if (split <= 0) return basicDenied(context)
        val username = pair.substring(0, split)
        val password = pair.substring(split + 1)
        if (!isValidOpdsUsername(username)) return basicDenied(context)
        val limitKey = "${context.request().remoteAddress().host()}:$username"
        if (!basicAttempts.allow(limitKey)) return basicDenied(context)
        val record = registryMutex.withLock { loadRegistry().getJsonObject(username) }
            ?: run { basicAttempts.failed(limitKey); return basicDenied(context) }
        val owner = record.getString("owner") ?: return basicDenied(context)
        // Registry is not sufficient by itself.  This second durable record is
        // required so an interrupted two-file update can only disable a login,
        // never leave an orphaned credential usable.
        val configured = loadSettings(owner)?.getString("username")
        if (configured != username || !recordOwnerMatches(record, owner)) {
            basicAttempts.failed(limitKey)
            return basicDenied(context)
        }
        // PBKDF2 is intentionally bounded to two concurrent attempts.  A full
        // pool is indistinguishable from invalid credentials to a client.
        if (!passwordWork.tryAcquire()) return basicDenied(context)
        val passwordMatches = try {
            withContext(Dispatchers.IO) { matchesPassword(record, password) }
        } finally {
            passwordWork.release()
        }
        if (!passwordMatches) {
            basicAttempts.failed(limitKey)
            return basicDenied(context)
        }
        basicAttempts.succeeded(limitKey)
        return owner
    }

    private fun basicDenied(context: RoutingContext): String? {
        context.response().setStatusCode(401).putHeader("WWW-Authenticate", "Basic realm=\"Reader OPDS\", charset=\"UTF-8\"")
            .putHeader("Cache-Control", "no-store").end()
        return null
    }

    /** A recreated username is a different owner.  Its old OPDS credential stays invalid. */
    private fun recordOwnerMatches(record: JsonObject, owner: String): Boolean {
        if (record.getString("owner") != owner) return false
        if (!appConfig.secure) return false
        val user = getUserInfoClass(owner) ?: return false
        return record.getLong("ownerCreatedAt") == user.created_at
    }

    private fun ownerCreationTime(owner: String): Long =
        if (appConfig.secure) getUserInfoClass(owner)?.created_at ?: -1L else 0L

    private fun loadSettings(owner: String): JsonObject? =
        asJsonObject(getUserStorage(owner, SETTINGS_NAME))?.takeIf { !it.getString("username", "").isBlank() }
    private fun saveSettings(owner: String, value: JsonObject) = saveUserStorage(owner, SETTINGS_NAME, value.encode())
    private fun loadRegistry(): JsonObject = asJsonObject(getStorage("data", REGISTRY_NAME)) ?: JsonObject()
    private fun saveRegistry(value: JsonObject) = saveStorage("data", REGISTRY_NAME, value = value.encode())
    private fun settingsView(setting: JsonObject?, owner: String): Map<String, Any> {
        val username = setting?.getString("username") ?: ""
        val passwordSet = username.isNotEmpty() && loadRegistry().getJsonObject(username)?.let { recordOwnerMatches(it, owner) } == true
        return mapOf("enabled" to passwordSet, "username" to username, "passwordSet" to passwordSet)
    }

    private fun downloadableFile(book: Book, owner: String): File? {
        book.setRootDir(getWorkDir())
        book.setUserNameSpace(owner)
        val candidate = try { book.getLocalFile().toPath().toAbsolutePath().normalize() } catch (_: Exception) { return null }
        // getWorkDir(vararg) currently builds a root-relative path on Windows
        // ("\\storage\\data"), which discards the configured work directory.
        // Use File(parent, child) so OPDS always stays under the active workdir.
        val dataRoot = File(getWorkDir(), "storage${File.separator}data").toPath().toAbsolutePath().normalize()
        val ownerRoot = dataRoot.resolve(owner).normalize()
        return try {
            // Every component is checked without following links.  In particular,
            // a user directory symlinked to another user's storage is forbidden.
            if (ownerRoot.parent != dataRoot || Files.isSymbolicLink(dataRoot) || Files.isSymbolicLink(ownerRoot)) return null
            if (!Files.isDirectory(dataRoot, LinkOption.NOFOLLOW_LINKS) || !Files.isDirectory(ownerRoot, LinkOption.NOFOLLOW_LINKS)) return null
            if (!candidate.startsWith(ownerRoot) || !Files.isRegularFile(candidate, LinkOption.NOFOLLOW_LINKS)) return null
            var component = ownerRoot
            ownerRoot.relativize(candidate).forEach { part ->
                component = component.resolve(part)
                if (Files.isSymbolicLink(component)) return null
            }
            val realRoot = ownerRoot.toRealPath(LinkOption.NOFOLLOW_LINKS)
            val realFile = candidate.toRealPath(LinkOption.NOFOLLOW_LINKS)
            if (!realFile.startsWith(realRoot)) return null
            val extension = realFile.fileName.toString().substringAfterLast('.', "").lowercase()
            if (extension !in setOf("txt", "epub", "pdf")) return null
            realFile.toFile()
        } catch (_: Exception) { null }
    }

    /** OPDS never refreshes web sources; read the already-saved shelf directly. */
    private fun listShelfBooks(owner: String): List<Book> {
        val shelf = asJsonArray(getUserStorage(owner, "bookshelf")) ?: return emptyList()
        return (0 until shelf.size()).mapNotNull { index ->
            try {
                shelf.getJsonObject(index).mapTo(Book::class.java).also {
                    it.isInShelf = true
                    it.setRootDir(getWorkDir())
                    it.setUserNameSpace(owner)
                }
            } catch (_: Exception) { null }
        }
    }

    /**
     * Opens the final inode relative to already-open directory handles.  Linux's
     * SecureDirectoryStream gives this the same no-follow guarantee across the
     * validation/open boundary; unsupported filesystems fail closed.
     */
    private fun openOwnedFile(owner: String, candidate: Path): SeekableByteChannel? {
        val dataRoot = File(getWorkDir(), "storage${File.separator}data").toPath().toAbsolutePath().normalize()
        val ownerRoot = dataRoot.resolve(owner).normalize()
        if (ownerRoot.parent != dataRoot || !candidate.toAbsolutePath().normalize().startsWith(ownerRoot)) return null
        // Start from the filesystem root, not dataRoot by pathname.  Opening
        // each descendant through SecureDirectoryStream with NOFOLLOW prevents
        // dataRoot itself from being swapped for a symlink after validation.
        val fileSystemRoot = dataRoot.root ?: return null
        val relative = fileSystemRoot.relativize(candidate.toAbsolutePath().normalize())
        var current: SecureDirectoryStream<Path>? = null
        try {
            val rootStream = Files.newDirectoryStream(fileSystemRoot)
            current = rootStream as? SecureDirectoryStream<Path>
            if (current == null) {
                rootStream.close()
                return null
            }
            val parts = relative.iterator()
            while (parts.hasNext()) {
                val part = parts.next()
                val directory = current ?: return null
                if (!parts.hasNext()) {
                    return directory.newByteChannel(part, setOf(StandardOpenOption.READ, LinkOption.NOFOLLOW_LINKS))
                }
                val next = directory.newDirectoryStream(part, LinkOption.NOFOLLOW_LINKS) as? SecureDirectoryStream<Path> ?: return null
                directory.close()
                current = next
            }
            return null
        } catch (_: Exception) {
            return null
        } finally {
            try { current?.close() } catch (_: Exception) { }
        }
    }

    private suspend fun streamOpenedFile(context: RoutingContext, channel: SeekableByteChannel) {
        val response = context.response()
        fun closeChannel() { try { channel.close() } catch (_: Exception) { } }
        response.closeHandler { closeChannel() }
        response.exceptionHandler { closeChannel() }
        try {
            val bytes = ByteArray(64 * 1024)
            while (true) {
                val count = channel.read(java.nio.ByteBuffer.wrap(bytes))
                if (count <= 0) break
                response.write(Buffer.buffer(bytes.copyOf(count)))
                if (response.writeQueueFull()) {
                    awaitResponseDrain(response, ::closeChannel)
                }
            }
            if (!response.closed()) response.end()
        } catch (_: java.nio.channels.ClosedChannelException) {
            // A disconnected OPDS client closes the fixed descriptor; it is not a server error.
        } catch (error: Exception) {
            // Vert.x can surface a write error after its close handler.  Preserve
            // genuine server errors but do not turn a disconnected client into one.
            if (!response.closed()) throw error
        } finally {
            response.drainHandler(null)
            closeChannel()
        }
    }

    /**
     * A disconnect while Vert.x's write queue is full must wake the suspended
     * writer.  Otherwise its finally block never releases downloadWork.
     */
    private suspend fun awaitResponseDrain(
        response: io.vertx.core.http.HttpServerResponse,
        closeChannel: () -> Unit
    ) {
        suspendCancellableCoroutine<Unit> { continuation ->
            val finished = java.util.concurrent.atomic.AtomicBoolean(false)
            fun clear() { response.drainHandler(null) }
            fun complete(error: Throwable? = null) {
                if (!finished.compareAndSet(false, true)) return
                clear()
                if (continuation.isActive) {
                    if (error == null) continuation.resume(Unit)
                    else continuation.resumeWithException(error)
                }
            }
            fun disconnected() {
                closeChannel()
                complete(java.nio.channels.ClosedChannelException())
            }
            continuation.invokeOnCancellation {
                if (finished.compareAndSet(false, true)) {
                    clear()
                    closeChannel()
                }
            }
            response.drainHandler {
                complete()
            }
            response.closeHandler { disconnected() }
            response.exceptionHandler { disconnected() }
            // The queue can drain between writeQueueFull and handler installation.
            if (response.closed()) disconnected()
            else if (!response.writeQueueFull()) complete()
        }
    }

    private fun acquisitionEntry(book: Book, owner: String, file: File): String {
        val id = Base64.getUrlEncoder().withoutPadding().encodeToString(book.bookUrl.toByteArray(StandardCharsets.UTF_8))
        val updated = Instant.ofEpochMilli(book.lastCheckTime).toString()
        return "<entry><id>${xml("urn:reader:opds:book:${stableId(book.bookUrl)}")}</id><title>${xml(book.name)}</title>" +
            "<updated>$updated</updated><author><name>${xml(book.author.ifBlank { "未知作者" })}</name></author>" +
            "<content type=\"text\">${xml(book.getDisplayIntro() ?: "本地书籍")}</content>" +
            "<link rel=\"http://opds-spec.org/acquisition/open-access\" href=\"/reader3/opds/download/$id\" type=\"${mediaType(file.name)}\"/></entry>"
    }

    private fun feed(title: String, id: String, self: String, navigation: Boolean, entries: String): String =
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><feed xmlns=\"http://www.w3.org/2005/Atom\">" +
            "<id>${xml("urn:reader:opds:$id")}</id><title>${xml(title)}</title><updated>${Instant.now()}</updated>" +
            "<link rel=\"self\" href=\"$self\" type=\"${if (navigation) OPDS_NAVIGATION_TYPE else OPDS_ACQUISITION_TYPE}\"/>" +
            "<link rel=\"start\" href=\"/reader3/opds/catalog\" type=\"$OPDS_NAVIGATION_TYPE\"/>$entries</feed>"

    private fun navigationEntry(title: String, href: String, content: String, rel: String, type: String): String =
        "<entry><id>${xml("urn:reader:opds:$href")}</id><title>${xml(title)}</title><updated>${Instant.now()}</updated>" +
            "<content type=\"text\">${xml(content)}</content><link rel=\"$rel\" href=\"$href\" type=\"$type\"/></entry>"

    private fun mediaType(name: String): String = when (name.substringAfterLast('.', "").lowercase()) {
        "epub" -> "application/epub+zip"; "pdf" -> "application/pdf"; else -> "text/plain; charset=UTF-8"
    }
    private fun isSafeOnlineLink(value: String): Boolean = value.startsWith("https://") || value.startsWith("http://")
    private fun xml(value: String): String = opdsXml(value)
    private fun stableId(value: String): String = MessageDigest.getInstance("SHA-256").digest(value.toByteArray(StandardCharsets.UTF_8)).joinToString("") { "%02x".format(it) }

    companion object {
        private const val SETTINGS_NAME = "opds"
        private const val REGISTRY_NAME = "opds-users"
        private const val ITERATIONS = 120_000
        private const val KEY_BITS = 256
        const val OPDS_NAVIGATION_TYPE = "application/atom+xml;profile=opds-catalog;kind=navigation"
        const val OPDS_ACQUISITION_TYPE = "application/atom+xml;profile=opds-catalog;kind=acquisition"
        private val registryMutex = Mutex()
        private val random = SecureRandom()
        private val passwordWork = Semaphore(2, true)
        private val downloadWork = Semaphore(2, true)
        private val basicAttempts = BasicAttemptLimiter()

        fun isValidOpdsUsername(value: String): Boolean = Regex("[A-Za-z0-9._-]{3,64}").matches(value)
        fun isValidOpdsPassword(value: String): Boolean = value.length in 12..128
            && value.none { Character.isISOControl(it) }

        fun passwordRecord(owner: String, password: String, ownerCreatedAt: Long = 0L): JsonObject {
            val salt = ByteArray(16).also { random.nextBytes(it) }
            return JsonObject().put("owner", owner).put("salt", Base64.getEncoder().encodeToString(salt))
                .put("ownerCreatedAt", ownerCreatedAt)
                .put("kdf", "PBKDF2WithHmacSHA256").put("iterations", ITERATIONS)
                .put("hash", Base64.getEncoder().encodeToString(pbkdf2(password, salt)))
        }

        fun matchesPassword(record: JsonObject, password: String): Boolean {
            return try {
                if (record.getString("kdf") != "PBKDF2WithHmacSHA256" || record.getInteger("iterations") != ITERATIONS) return false
                val salt = Base64.getDecoder().decode(record.getString("salt"))
                val expected = Base64.getDecoder().decode(record.getString("hash"))
                if (salt.size != 16 || expected.size != KEY_BITS / 8) return false
                MessageDigest.isEqual(expected, pbkdf2(password, salt))
            } catch (_: Exception) { false }
        }

        private fun pbkdf2(password: String, salt: ByteArray): ByteArray {
            val spec = PBEKeySpec(password.toCharArray(), salt, ITERATIONS, KEY_BITS)
            return try { SecretKeyFactory.getInstance("PBKDF2WithHmacSHA256").generateSecret(spec).encoded } finally { spec.clearPassword() }
        }

        private class BasicAttemptLimiter {
            private data class Attempt(var failures: Int, var windowStartedAt: Long, var blockedUntil: Long)
            private val states = object : LinkedHashMap<String, Attempt>(513, 0.75f, true) {
                override fun removeEldestEntry(eldest: MutableMap.MutableEntry<String, Attempt>?): Boolean = size > 512
            }
            @Synchronized fun allow(key: String): Boolean {
                val now = System.currentTimeMillis()
                val attempt = states[key] ?: return true
                if (attempt.blockedUntil > now) return false
                if (now - attempt.windowStartedAt > 60_000L) states.remove(key)
                return true
            }
            @Synchronized fun failed(key: String) {
                val now = System.currentTimeMillis()
                val attempt = states[key]
                if (attempt == null || now - attempt.windowStartedAt > 60_000L) {
                    states[key] = Attempt(1, now, 0L)
                } else {
                    attempt.failures += 1
                    if (attempt.failures >= 5) attempt.blockedUntil = now + 60_000L
                }
            }
            @Synchronized fun succeeded(key: String) { states.remove(key) }
        }
    }
}

/** XML 1.0 forbids C0 controls except tab, LF and CR; catalog metadata is untrusted. */
internal fun opdsXml(value: String): String = buildString(value.length) {
    value.forEach { ch ->
        val code = ch.code
        if (code == 0x9 || code == 0xA || code == 0xD ||
            code in 0x20..0xD7FF || code in 0xE000..0xFFFD) append(ch)
    }
}.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
    .replace("\"", "&quot;").replace("'", "&apos;")
