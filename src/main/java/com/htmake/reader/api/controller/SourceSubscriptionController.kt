package com.htmake.reader.api.controller

import com.htmake.reader.api.ReturnData
import com.htmake.reader.entity.User
import com.htmake.reader.utils.BrowserNetworkPolicy
import com.htmake.reader.utils.asJsonArray
import io.legado.app.data.entities.BookSource
import io.vertx.core.json.JsonArray
import io.vertx.core.json.JsonObject
import io.vertx.ext.web.RoutingContext
import kotlinx.coroutines.CoroutineScope
import kotlin.coroutines.CoroutineContext

/**
 * A persisted subscription deliberately uses the legacy key and fields as well:
 *
 * storage/data/<user>/remoteBookSourceSub.json
 * [{"name":"…","link":"https://…","lastSyncTime":…, "enabled":true}]
 *
 * `url` is written too for the Vue 3 contract.  Keeping `link` means an existing
 * Vue 2 client and the legacy scheduled updater can consume the same record.
 */
internal const val SOURCE_SUBSCRIPTION_STORAGE_KEY = "remoteBookSourceSub"
internal const val SOURCE_SUBSCRIPTION_MAX_BYTES = 2 * 1024 * 1024
internal const val SOURCE_SUBSCRIPTION_TIMEOUT_MS = 10_000L

/** Converts legacy {link,...} records and Vue 3 {url,...} records to one API shape. */
internal fun normalizeSourceSubscriptions(value: JsonArray?): JsonArray {
    val result = JsonArray()
    value ?: return result
    for (i in 0 until value.size()) {
        val item = value.getJsonObject(i) ?: continue
        val url = item.getString("url")?.trim().takeUnless { it.isNullOrEmpty() }
            ?: item.getString("link")?.trim().takeUnless { it.isNullOrEmpty() }
            ?: continue
        result.add(JsonObject()
            .put("url", url)
            .put("name", item.getString("name")?.trim().takeUnless { it.isNullOrEmpty() } ?: url)
            .put("enabled", item.getBoolean("enabled", true))
            .put("lastSyncTime", item.getLong("lastSyncTime", 0L)))
    }
    return result
}

/** Writes a subscription in the historical on-disk format plus the Vue 3 URL alias. */
internal fun sourceSubscriptionRecord(url: String, name: String, enabled: Boolean, lastSyncTime: Long): JsonObject =
    JsonObject()
        .put("name", name)
        .put("link", url)
        .put("url", url)
        .put("enabled", enabled)
        .put("lastSyncTime", lastSyncTime)

/** Missing means enabled: this is the legacy file's historical behaviour. */
internal fun isSourceSubscriptionAutoRefreshEnabled(item: JsonObject): Boolean = item.getBoolean("enabled", true)

/** Only records created by the Vue 3 API opt into the hardened scheduled path. */
internal fun isManagedSourceSubscription(item: JsonObject): Boolean = item.containsKey("url")

internal fun selectedRemoteBookSources(remoteSources: JsonArray, selectedUrls: Set<String>?): JsonArray {
    val result = JsonArray()
    val seen = linkedSetOf<String>()
    for (i in 0 until remoteSources.size()) {
        val sourceJson = remoteSources.getJsonObject(i) ?: continue
        val source = BookSource.fromJson(sourceJson.toString()).getOrNull() ?: continue
        val url = source.bookSourceUrl.trim()
        if (url.isBlank() || !seen.add(url) || (selectedUrls != null && !selectedUrls.contains(url))) continue
        result.add(JsonObject.mapFrom(source))
    }
    return result
}

internal class SourceSubscriptionController(
    coroutineContext: CoroutineContext,
    private val bookSourceController: BookSourceController,
    private val networkPolicy: BrowserNetworkPolicy = BrowserNetworkPolicy(),
) : BaseController(coroutineContext), CoroutineScope {
    private val remoteFetcher = RemoteSourceSubscriptionFetcher(networkPolicy)

    suspend fun getSourceSubs(context: RoutingContext): ReturnData {
        val auth = authorize(context) ?: return ReturnData().setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        return ReturnData().setData(normalizeSourceSubscriptions(loadSubscriptions(auth.namespace)).list)
    }

    suspend fun previewSourceSub(context: RoutingContext): ReturnData {
        val auth = authorize(context) ?: return ReturnData().setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        val url = context.bodyAsJson?.getString("url")?.trim().orEmpty()
        if (url.isEmpty()) return ReturnData().setErrorMsg("请输入远程书源链接")
        return try {
            val sources = fetchRemoteSources(url)
            val existing = existingUrls(auth.namespace)
            ReturnData().setData(mapOf("sources" to sources.list, "existing" to existing.toList()))
        } catch (e: RemoteSourceSubscriptionException) {
            ReturnData().setErrorMsg(e.message ?: "远程书源订阅失败")
        }
    }

    suspend fun saveSourceSub(context: RoutingContext): ReturnData {
        val auth = authorize(context) ?: return ReturnData().setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        val body = context.bodyAsJson ?: return ReturnData().setErrorMsg("参数错误")
        val url = body.getString("url")?.trim().orEmpty()
        val name = body.getString("name")?.trim().takeUnless { it.isNullOrEmpty() } ?: url
        if (url.isEmpty()) return ReturnData().setErrorMsg("请输入远程书源链接")
        val selected = body.getJsonArray("selectedUrls")?.mapNotNull { it as? String }?.map { it.trim() }?.toSet()
        return try {
            val sources = selectedRemoteBookSources(fetchRemoteSources(url), selected)
            if (sources.isEmpty) return ReturnData().setErrorMsg("未选择可导入的有效书源")
            val saved = bookSourceController.saveUserBookSources(auth.namespace, auth.user, sources)
            if (!saved.isSuccess) return ReturnData().setErrorMsg(saved.errorMsg)
            upsert(auth.namespace, url, name, enabled = currentEnabled(auth.namespace, url), lastSyncTime = System.currentTimeMillis())
            ReturnData().setData(mapOf("count" to sources.size(), "name" to name), saved.errorMsg)
        } catch (e: RemoteSourceSubscriptionException) {
            ReturnData().setErrorMsg(e.message ?: "远程书源订阅失败")
        }
    }

    suspend fun refreshSourceSub(context: RoutingContext): ReturnData {
        val auth = authorize(context) ?: return ReturnData().setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        val url = context.bodyAsJson?.getString("url")?.trim().orEmpty()
        if (url.isEmpty()) return ReturnData().setErrorMsg("参数错误")
        val subscription = find(auth.namespace, url) ?: return ReturnData().setErrorMsg("订阅不存在")
        return try {
            val sources = selectedRemoteBookSources(fetchRemoteSources(url), null)
            if (sources.isEmpty) return ReturnData().setErrorMsg("远程书源不含有效书源")
            val saved = bookSourceController.saveUserBookSources(auth.namespace, auth.user, sources)
            if (!saved.isSuccess) return ReturnData().setErrorMsg(saved.errorMsg)
            upsert(auth.namespace, url, subscription.getString("name") ?: url,
                subscription.getBoolean("enabled", true), System.currentTimeMillis())
            ReturnData().setData(mapOf("count" to sources.size()), saved.errorMsg)
        } catch (e: RemoteSourceSubscriptionException) {
            ReturnData().setErrorMsg(e.message ?: "刷新订阅失败")
        }
    }

    suspend fun deleteSourceSub(context: RoutingContext): ReturnData {
        val auth = authorize(context) ?: return ReturnData().setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        val url = context.bodyAsJson?.getString("url")?.trim().orEmpty()
        if (url.isEmpty()) return ReturnData().setErrorMsg("参数错误")
        val deleted = delete(auth.namespace, setOf(url))
        if (deleted == 0) return ReturnData().setErrorMsg("订阅不存在")
        return ReturnData().setData("")
    }

    suspend fun deleteSourceSubs(context: RoutingContext): ReturnData {
        val auth = authorize(context) ?: return ReturnData().setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        val body = context.bodyAsJson ?: return ReturnData().setErrorMsg("参数错误")
        val urls = body.getJsonArray("urls")?.mapNotNull { it as? String }?.map { it.trim() }?.filter { it.isNotEmpty() }?.toSet()
            ?: return ReturnData().setErrorMsg("参数错误")
        if (urls.isEmpty()) return ReturnData().setErrorMsg("参数错误")
        return ReturnData().setData(mapOf("deleted" to delete(auth.namespace, urls)))
    }

    suspend fun setSourceSubEnabled(context: RoutingContext): ReturnData {
        val auth = authorize(context) ?: return ReturnData().setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        val body = context.bodyAsJson ?: return ReturnData().setErrorMsg("参数错误")
        val url = body.getString("url")?.trim().orEmpty()
        val enabled = body.getBoolean("enabled") ?: return ReturnData().setErrorMsg("参数错误")
        val item = find(auth.namespace, url) ?: return ReturnData().setErrorMsg("订阅不存在")
        upsert(auth.namespace, url, item.getString("name") ?: url, enabled, item.getLong("lastSyncTime", 0L))
        return ReturnData().setData(mapOf("enabled" to enabled))
    }

    private suspend fun authorize(context: RoutingContext): SubscriptionAuth? {
        if (!checkAuth(context) || !bookSourceController.canEditBookSource(context)) return null
        return SubscriptionAuth(getUserNameSpace(context), context.get("userInfo") as User?)
    }

    private fun loadSubscriptions(namespace: String): JsonArray =
        (asJsonArray(getUserStorage(namespace, SOURCE_SUBSCRIPTION_STORAGE_KEY)) ?: JsonArray()).copy()

    private fun find(namespace: String, url: String): JsonObject? =
        loadSubscriptions(namespace).firstOrNull { item ->
            (item as? JsonObject)?.let { it.getString("url") ?: it.getString("link") } == url
        } as? JsonObject

    private fun currentEnabled(namespace: String, url: String): Boolean = find(namespace, url)?.getBoolean("enabled", true) ?: true

    private fun upsert(namespace: String, url: String, name: String, enabled: Boolean, lastSyncTime: Long) {
        val list = loadSubscriptions(namespace)
        var found = -1
        for (i in 0 until list.size()) {
            val item = list.getJsonObject(i) ?: continue
            if ((item.getString("url") ?: item.getString("link")) == url) {
                found = i
                break
            }
        }
        val record = sourceSubscriptionRecord(url, name, enabled, lastSyncTime)
        if (found >= 0) list.set(found, record) else list.add(record)
        saveUserStorage(namespace, SOURCE_SUBSCRIPTION_STORAGE_KEY, list)
    }

    private fun delete(namespace: String, urls: Set<String>): Int {
        val list = loadSubscriptions(namespace)
        val kept = JsonArray()
        var deleted = 0
        for (i in 0 until list.size()) {
            val item = list.getJsonObject(i) ?: continue
            val url = item.getString("url") ?: item.getString("link")
            if (url != null && urls.contains(url)) deleted++ else kept.add(item)
        }
        if (deleted > 0) saveUserStorage(namespace, SOURCE_SUBSCRIPTION_STORAGE_KEY, kept)
        return deleted
    }

    private fun existingUrls(namespace: String): Set<String> {
        val sources = bookSourceController.getUserBookSourceJson(namespace) ?: return emptySet()
        return (0 until sources.size()).mapNotNull { sources.getJsonObject(it)?.getString("bookSourceUrl") }.toSet()
    }

    private suspend fun fetchRemoteSources(url: String): JsonArray = remoteFetcher.fetch(url)
}

private data class SubscriptionAuth(val namespace: String, val user: User?)
