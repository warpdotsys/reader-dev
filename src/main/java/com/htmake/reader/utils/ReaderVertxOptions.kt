package com.htmake.reader.utils

import io.vertx.core.VertxOptions

/** 显式配置阻塞线程池；未设置时保持 Vert.x 3.8.5 的原默认行为。 */
fun readerVertxOptions(environment: Map<String, String> = System.getenv()): VertxOptions {
    val options = VertxOptions()
    fun configured(name: String): Int? {
        val raw = environment[name] ?: return null
        require(raw.matches(Regex("[1-9][0-9]{0,2}"))) { "$name 必须是 1 到 128 的整数" }
        val value = raw.toInt()
        require(value in 1..128) { "$name 必须是 1 到 128 的整数" }
        return value
    }
    configured("READER_VERTX_WORKER_POOL_SIZE")?.let { options.setWorkerPoolSize(it) }
    configured("READER_VERTX_INTERNAL_BLOCKING_POOL_SIZE")?.let { options.setInternalBlockingPoolSize(it) }
    return options
}
