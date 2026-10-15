package com.htmake.reader.api.controller

import com.htmake.reader.config.AppConfig
import com.htmake.reader.entity.User
import com.htmake.reader.utils.SpringContextUtils
import com.htmake.reader.utils.asJsonArray
import com.htmake.reader.utils.getStorage
import com.htmake.reader.utils.saveStorage
import com.htmake.reader.utils.storageFinalPath
import com.htmake.reader.utils.toMap
import com.htmake.reader.utils.workDirInit
import com.htmake.reader.utils.workDirPath
import io.vertx.core.Vertx
import io.vertx.core.json.Json
import io.vertx.core.json.JsonArray
import io.vertx.core.json.JsonObject
import io.vertx.ext.web.Router
import io.vertx.ext.web.client.WebClient
import io.vertx.ext.web.handler.CookieHandler
import io.vertx.ext.web.handler.SessionHandler
import io.vertx.ext.web.sstore.LocalSessionStore
import io.legado.app.constant.BookType
import io.legado.app.data.entities.Book
import com.fasterxml.jackson.databind.DeserializationFeature
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import org.junit.Assert.assertFalse
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import org.springframework.context.support.StaticApplicationContext
import org.springframework.core.env.StandardEnvironment
import java.io.File
import java.nio.file.Files
import java.nio.file.Path
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

class OpdsControllerTest {
    @get:Rule val temporaryFolder = TemporaryFolder()
    @Test fun credentialsUseStrictIndependentPolicy() {
        assertTrue(OpdsController.isValidOpdsUsername("reader.user-01"))
        assertFalse(OpdsController.isValidOpdsUsername("ab"))
        assertFalse(OpdsController.isValidOpdsUsername("user name"))
        assertTrue(OpdsController.isValidOpdsPassword("twelve-chars"))
        assertFalse(OpdsController.isValidOpdsPassword("short"))
        assertFalse(OpdsController.isValidOpdsPassword("twelve\nchars"))
    }

    @Test fun passwordRegistryStoresOnlySaltedHashAndVerifiesInConstantTimePrimitive() {
        val record = OpdsController.passwordRecord("owner-a", "generated-test-password")
        assertTrue(record.containsKey("salt"))
        assertTrue(record.containsKey("hash"))
        assertTrue(record.containsKey("iterations"))
        assertFalse(record.encode().contains("generated-test-password"))
        assertTrue(OpdsController.matchesPassword(record, "generated-test-password"))
        assertFalse(OpdsController.matchesPassword(record, "wrong-generated-password"))
    }

    @Test fun opdsMediaTypesAreProtocolSpecific() {
        assertTrue(OpdsController.OPDS_NAVIGATION_TYPE.contains("profile=opds-catalog"))
        assertTrue(OpdsController.OPDS_NAVIGATION_TYPE.contains("kind=navigation"))
        assertTrue(OpdsController.OPDS_ACQUISITION_TYPE.contains("kind=acquisition"))
    }

    @Test fun xmlEscapingRemovesForbiddenControlsWithoutChangingAllowedWhitespace() {
        assertEquals("A&amp;B&lt;C&gt;&quot;&apos;\n", opdsXml("A\u0001&B<C>\"'\n"))
    }

    @Test fun httpCatalogRequiresBasicAndReturnsAnOpdsAtomFeedFromAnIsolatedFixture() {
        val oldContext = SpringContextUtils.getApplicationContext()
        val oldStorage = storageFinalPath
        val oldWorkDir = workDirPath
        val oldWorkDirInit = workDirInit
        val oldFailOnUnknown = Json.mapper.isEnabled(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES)
        val vertx = Vertx.vertx()
        val appContext = StaticApplicationContext()
        try {
            // ReaderApplication disables this option before serving requests.
            // JsonObject.mapFrom(Book) includes derived display properties, so
            // the isolated fixture must use the same mapper contract.
            Json.mapper.configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false)
            val fixtureRoot = temporaryFolder.newFolder("opds-http-fixture")
            storageFinalPath = File(fixtureRoot, "storage").absolutePath
            workDirPath = fixtureRoot.absolutePath
            workDirInit = true
            val appConfig = AppConfig().apply { secure = true; workDir = fixtureRoot.absolutePath }
            appContext.beanFactory.registerSingleton("appConfig", appConfig)
            appContext.beanFactory.registerSingleton("environment", StandardEnvironment())
            appContext.beanFactory.registerSingleton("webClient", WebClient.create(vertx))
            appContext.refresh()
            SpringContextUtils().setApplicationContext(appContext)

            val owner = "fixtureowner"
            val createdAt = 101L
            val user = User(username = owner, created_at = createdAt)
            val otherOwner = "otherowner"
            saveStorage("data", "users", value = mapOf(owner to user.toMap(), otherOwner to User(username = otherOwner, created_at = 202L).toMap()))
            val opdsUser = "fixture-opds"
            val password = "generated-password-12"
            saveStorage("data", "opds-users", value = io.vertx.core.json.JsonObject().put(opdsUser,
                OpdsController.passwordRecord(owner, password, createdAt)))
            saveStorage("data", owner, "opds", value = io.vertx.core.json.JsonObject().put("username", opdsUser))

            val ownerBookFile = File(fixtureRoot, "storage/data/$owner/books/generated.txt").apply { parentFile.mkdirs(); writeText("OPDS generated local text", Charsets.UTF_8) }
            val localBook = Book(bookUrl = "local://fixture-good", origin = BookType.local, name = "A\u0001 & <book>", author = "fixture", originName = "storage/data/$owner/books/${ownerBookFile.name}")
            val otherBook = Book(bookUrl = "local://fixture-other", origin = BookType.local, name = "other", author = "fixture", originName = "storage/data/$otherOwner/books/other.txt")
            val otherFile = File(fixtureRoot, "storage/data/$otherOwner/books/other.txt").apply { parentFile.mkdirs(); writeText("other", Charsets.UTF_8) }
            saveStorage("data", owner, "bookshelf", value = JsonArray().add(JsonObject.mapFrom(localBook)))
            saveStorage("data", otherOwner, "bookshelf", value = JsonArray().add(JsonObject.mapFrom(otherBook)))

            // This fixture contains generated data only.  Keep the conversion
            // boundary observable so a failed acquisition assertion says whether
            // persistence, Book mapping, or the resolved local path is at fault.
            val persistedShelf = asJsonArray(getStorage("data", owner, "bookshelf"))
                ?: throw AssertionError("OPDS generated fixture shelf was not persisted")
            val persistedBook = try {
                persistedShelf.getJsonObject(0).mapTo(Book::class.java).also {
                    it.setRootDir(fixtureRoot.absolutePath)
                    it.setUserNameSpace(owner)
                }
            } catch (error: Exception) {
                throw AssertionError("OPDS generated fixture Book mapping failed: ${error.javaClass.simpleName}: ${error.message}", error)
            }
            val resolvedFixturePath = persistedBook.getLocalFile().toPath().toAbsolutePath().normalize()
            println("OPDS generated fixture: shelf=${persistedShelf.size()}, local=${persistedBook.isLocalBook()}, origin=${persistedBook.origin}, originName=${persistedBook.originName}, resolved=$resolvedFixturePath, regular=${Files.isRegularFile(resolvedFixturePath)}")
            assertTrue("Persisted generated fixture must still be a local book", persistedBook.isLocalBook())
            assertEquals("Persisted generated path changed", ownerBookFile.toPath().toAbsolutePath().normalize(), resolvedFixturePath)
            assertTrue("Generated fixture file disappeared: $resolvedFixturePath", Files.isRegularFile(resolvedFixturePath))

            val controller = OpdsController(kotlin.coroutines.EmptyCoroutineContext)
            val router = Router.router(vertx)
            router.route().handler(CookieHandler.create())
            router.route().handler(SessionHandler.create(LocalSessionStore.create(vertx)))
            router.post("/fixture-login").handler { ctx -> ctx.session().put("username", owner); ctx.response().end() }
            router.get("/reader3/getOpdsSettings").handler { ctx ->
                GlobalScope.launch(Dispatchers.Default) { ctx.response().putHeader("Content-Type", "application/json").end(Json.encode(controller.getSettings(ctx))) }
            }
            router.post("/reader3/saveOpdsSettings").handler { ctx ->
                ctx.request().bodyHandler { body ->
                    ctx.setBody(body)
                    GlobalScope.launch(Dispatchers.Default) { ctx.response().putHeader("Content-Type", "application/json").end(Json.encode(controller.saveSettings(ctx))) }
                }
            }
            router.get("/reader3/opds/catalog").handler { ctx ->
                GlobalScope.launch(Dispatchers.Default) { controller.catalog(ctx) }
            }
            router.get("/reader3/opds/local").handler { ctx -> GlobalScope.launch(Dispatchers.Default) { controller.localCatalog(ctx) } }
            router.get("/reader3/opds/download/:id").handler { ctx -> GlobalScope.launch(Dispatchers.Default) { controller.download(ctx) } }
            val serverReady = CountDownLatch(1)
            var port = -1
            val server = vertx.createHttpServer().requestHandler(router).listen(0, "127.0.0.1") { result ->
                assertTrue(result.succeeded())
                port = result.result().actualPort()
                serverReady.countDown()
            }
            assertTrue(serverReady.await(10, TimeUnit.SECONDS))

            val sessionCookie = fixtureLogin(vertx, port)
            val saved = httpPost(vertx, port, "/reader3/saveOpdsSettings", sessionCookie,
                JsonObject().put("username", opdsUser).put("password", password).encode())
            assertEquals(200, saved.first)
            assertTrue(saved.third.contains("\"enabled\":true"))
            val settings = httpGet(vertx, port, null, path = "/reader3/getOpdsSettings", cookie = sessionCookie)
            assertEquals(200, settings.first)
            assertTrue(settings.third.contains("\"passwordSet\":true"))
            val retained = httpPost(vertx, port, "/reader3/saveOpdsSettings", sessionCookie,
                JsonObject().put("username", opdsUser).put("password", "").encode())
            assertTrue(retained.third.contains("\"enabled\":true"))

            val unauthorized = httpGet(vertx, port, null)
            assertEquals(401, unauthorized.first)
            assertTrue(unauthorized.second.contains("Basic"))
            val tokenOnly = httpGet(vertx, port, null, "?accessToken=$owner:ignored")
            assertEquals(401, tokenOnly.first)
            val encoded = java.util.Base64.getEncoder().encodeToString("$opdsUser:$password".toByteArray(Charsets.UTF_8))
            val wrongPassword = java.util.Base64.getEncoder().encodeToString("$opdsUser:wrong-generated-password".toByteArray(Charsets.UTF_8))
            assertEquals(401, httpGet(vertx, port, "Basic $wrongPassword").first)
            val authorized = httpGet(vertx, port, "Basic $encoded")
            assertEquals(200, authorized.first)
            assertTrue(authorized.second.contains(OpdsController.OPDS_NAVIGATION_TYPE))
            assertTrue(authorized.third.contains("<feed xmlns=\"http://www.w3.org/2005/Atom\""))
            assertFalse(authorized.third.contains("\u0001"))
            val local = httpGet(vertx, port, "Basic $encoded", path = "/reader3/opds/local")
            assertEquals(200, local.first)
            assertTrue(local.third.contains("http://opds-spec.org/acquisition/open-access"))
            val goodId = java.util.Base64.getUrlEncoder().withoutPadding().encodeToString(localBook.bookUrl.toByteArray(Charsets.UTF_8))
            val download = httpGet(vertx, port, "Basic $encoded", path = "/reader3/opds/download/$goodId")
            if (System.getProperty("os.name").lowercase().contains("linux")) {
                assertEquals(200, download.first)
                assertEquals("OPDS generated local text", download.third)
            } else {
                // Some Windows providers lack SecureDirectoryStream.  The product
                // deliberately fails closed instead of reopening an unsafe path.
                assertTrue(download.first == 200 || download.first == 404)
            }
            val otherId = java.util.Base64.getUrlEncoder().withoutPadding().encodeToString(otherBook.bookUrl.toByteArray(Charsets.UTF_8))
            assertEquals(404, httpGet(vertx, port, "Basic $encoded", path = "/reader3/opds/download/$otherId").first)

            val linkFile = File(fixtureRoot, "storage/data/$owner/books/escape.txt").toPath()
            val outside = temporaryFolder.newFile("opds-outside.txt").toPath()
            Files.write(outside, "outside".toByteArray(Charsets.UTF_8))
            var linkCreated = false
            try { Files.createSymbolicLink(linkFile, outside); linkCreated = true } catch (_: UnsupportedOperationException) { } catch (_: SecurityException) { } catch (_: java.io.IOException) { }
            if (System.getProperty("os.name").lowercase().contains("linux")) assertTrue("Linux CI must exercise the symlink negative case", linkCreated)
            if (linkCreated) {
                val linkedBook = Book(bookUrl = "local://fixture-link", origin = BookType.local, name = "link", author = "fixture", originName = "storage/data/$owner/books/escape.txt")
                saveStorage("data", owner, "bookshelf", value = JsonArray().add(JsonObject.mapFrom(localBook)).add(JsonObject.mapFrom(linkedBook)))
                val linkId = java.util.Base64.getUrlEncoder().withoutPadding().encodeToString(linkedBook.bookUrl.toByteArray(Charsets.UTF_8))
                assertEquals(404, httpGet(vertx, port, "Basic $encoded", path = "/reader3/opds/download/$linkId").first)
            }

            // Recreating the Reader username changes created_at, so its former
            // OPDS credential is invalid even though the string owner is reused.
            saveStorage("data", "users", value = mapOf(owner to User(username = owner, created_at = 303L).toMap(), otherOwner to User(username = otherOwner, created_at = 202L).toMap()))
            assertEquals(401, httpGet(vertx, port, "Basic $encoded").first)
            val stopped = CountDownLatch(1)
            server.close { stopped.countDown() }
            assertTrue(stopped.await(10, TimeUnit.SECONDS))
        } finally {
            Json.mapper.configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, oldFailOnUnknown)
            appContext.close()
            SpringContextUtils().setApplicationContext(oldContext)
            storageFinalPath = oldStorage
            workDirPath = oldWorkDir
            workDirInit = oldWorkDirInit
            val stopped = CountDownLatch(1)
            vertx.close { stopped.countDown() }
            stopped.await(10, TimeUnit.SECONDS)
        }
    }

    private fun fixtureLogin(vertx: Vertx, port: Int): String {
        val done = CountDownLatch(1)
        var cookie = ""
        vertx.createHttpClient().post(port, "127.0.0.1", "/fixture-login") { response ->
            cookie = response.getHeader("Set-Cookie")?.substringBefore(';') ?: ""
            response.bodyHandler { done.countDown() }
        }.end()
        assertTrue(done.await(10, TimeUnit.SECONDS))
        assertTrue(cookie.isNotEmpty())
        return cookie
    }

    private fun httpPost(vertx: Vertx, port: Int, path: String, cookie: String, body: String): Triple<Int, String, String> {
        val done = CountDownLatch(1)
        var status = -1; var header = ""; var responseBody = ""
        val request = vertx.createHttpClient().post(port, "127.0.0.1", path) { response ->
            status = response.statusCode(); header = response.getHeader("Content-Type") ?: ""
            response.bodyHandler { buffer -> responseBody = buffer.toString("UTF-8"); done.countDown() }
        }
        request.putHeader("Cookie", cookie).putHeader("Content-Type", "application/json").end(body)
        assertTrue(done.await(10, TimeUnit.SECONDS))
        return Triple(status, header, responseBody)
    }

    private fun httpGet(vertx: Vertx, port: Int, authorization: String?, suffix: String = "", path: String = "/reader3/opds/catalog", cookie: String? = null): Triple<Int, String, String> {
        val done = CountDownLatch(1)
        var status = -1
        var authenticate = ""
        var body = ""
        val request = vertx.createHttpClient().get(port, "127.0.0.1", "$path$suffix") { response ->
            status = response.statusCode()
            authenticate = response.getHeader("WWW-Authenticate") ?: response.getHeader("Content-Type") ?: ""
            response.bodyHandler { buffer -> body = buffer.toString("UTF-8"); done.countDown() }
        }
        if (authorization != null) request.putHeader("Authorization", authorization)
        if (cookie != null) request.putHeader("Cookie", cookie)
        request.end()
        assertTrue(done.await(10, TimeUnit.SECONDS))
        return Triple(status, authenticate, body)
    }
}
