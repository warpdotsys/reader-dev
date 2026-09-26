package com.htmake.reader.api.controller

import com.htmake.reader.api.ReturnData
import io.legado.app.data.entities.TxtTocRule
import io.legado.app.help.DefaultData
import io.legado.app.utils.GSON
import io.legado.app.utils.fromJsonArray
import io.vertx.core.json.JsonObject
import io.vertx.ext.web.RoutingContext
import java.util.regex.Pattern
import kotlin.coroutines.CoroutineContext

/**
 * TXT 目录规则的用户存储。
 *
 * 原版只公开 getTxtTocRules：内置规则来自资源文件，用户规则保存在
 * storage/data/<namespace>/txtTocRule.json。本控制器沿用该 JSON 格式，
 * 不把内置的负数 id 写入用户文件，也不会跨命名空间覆盖规则。
 */
class TxtTocRuleController(coroutineContext: CoroutineContext) : BaseController(coroutineContext) {

    private fun readCustomRules(namespace: String): MutableList<TxtTocRule> =
        GSON.fromJsonArray<TxtTocRule>(getUserStorage(namespace, "txtTocRule"))
            .getOrNull()
            ?.toMutableList()
            ?: mutableListOf()

    private fun saveCustomRules(namespace: String, rules: List<TxtTocRule>) {
        saveUserStorage(namespace, "txtTocRule", rules)
    }

    suspend fun getTxtTocRules(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        val namespace = getUserNameSpace(context)
        val rules = ArrayList<TxtTocRule>(DefaultData.txtTocRules.size)
        rules.addAll(DefaultData.txtTocRules)
        rules.addAll(readCustomRules(namespace))
        return returnData.setData(rules)
    }

    suspend fun saveTxtTocRule(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        val body = context.bodyAsJson ?: return returnData.setErrorMsg("参数错误")
        val name = body.getString("name")?.trim().orEmpty()
        val rule = body.getString("rule")?.trim().orEmpty()
        if (name.isEmpty()) return returnData.setErrorMsg("名称不能为空")
        if (rule.isEmpty()) return returnData.setErrorMsg("规则不能为空")
        try {
            Pattern.compile(rule)
        } catch (_: Exception) {
            return returnData.setErrorMsg("正则表达式无效")
        }

        val namespace = getUserNameSpace(context)
        val customRules = readCustomRules(namespace)
        val requestedId = body.getValue("id").asPositiveLongOrNull()
        // 负数是 JAR 内置规则的 id；不允许利用新增接口修改它们。
        if (body.getValue("id").asLongOrNull()?.let { it < 0 } == true) {
            return returnData.setErrorMsg("内置规则不可修改")
        }
        val id = requestedId ?: nextId(customRules)
        val serialNumber = body.getInteger("serialNumber")
            ?: customRules.maxOfOrNull { it.serialNumber }?.plus(1)
            ?: 0
        val entry = TxtTocRule(
            id = id,
            name = name,
            rule = rule,
            serialNumber = serialNumber,
            enable = body.getBoolean("enable", true)
        )
        val index = customRules.indexOfFirst { it.id == id }
        if (index >= 0) customRules[index] = entry else customRules.add(entry)
        saveCustomRules(namespace, customRules)
        return returnData.setData(mapOf("id" to id))
    }

    suspend fun deleteTxtTocRule(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        val rawId = context.bodyAsJson?.getValue("id") ?: return returnData.setErrorMsg("参数错误")
        val id = rawId.asLongOrNull() ?: return returnData.setErrorMsg("参数错误")
        if (id < 0) return returnData.setErrorMsg("内置规则不可删除")
        val namespace = getUserNameSpace(context)
        val rules = readCustomRules(namespace)
        val remaining = rules.filterNot { it.id == id }
        if (remaining.size != rules.size) saveCustomRules(namespace, remaining)
        return returnData.setData("")
    }

    /**
     * 将内置规则复制为可编辑的用户规则。以 name + rule 判重，重复调用不会
     * 覆盖用户已经编辑的规则；返回本次真正新增的数量。
     */
    suspend fun importDefaultTxtTocRules(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        val namespace = getUserNameSpace(context)
        val rules = readCustomRules(namespace)
        var nextId = nextId(rules)
        var added = 0
        DefaultData.txtTocRules.forEach { default ->
            if (rules.none { it.name == default.name && it.rule == default.rule }) {
                rules += default.copy(id = nextId++)
                added++
            }
        }
        if (added > 0) saveCustomRules(namespace, rules)
        return returnData.setData(mapOf("count" to added))
    }

    private fun nextId(rules: List<TxtTocRule>): Long {
        var id = System.currentTimeMillis().coerceAtLeast(1L)
        val existing = rules.map { it.id }.toHashSet()
        while (id in existing) id++
        return id
    }

    private fun Any?.asLongOrNull(): Long? = when (this) {
        is Number -> toLong()
        is String -> toLongOrNull()
        else -> null
    }

    private fun Any?.asPositiveLongOrNull(): Long? = asLongOrNull()?.takeIf { it > 0 }
}
