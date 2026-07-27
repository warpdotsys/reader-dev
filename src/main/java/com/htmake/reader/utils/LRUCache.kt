package com.htmake.reader.utils

import java.util.concurrent.ConcurrentHashMap

/**
 * Thread-safe LRU cache with ConcurrentHashMap and doubly-linked list.
 */
class LRUCache<K, V>(private val cacheCapacity: Int) {

    private val caches = ConcurrentHashMap<K, CacheNode>()
    private var first: CacheNode? = null
    private var last: CacheNode? = null

    inner class CacheNode {
        var pre: CacheNode? = null
        var next: CacheNode? = null
        var key: K? = null
        var value: V? = null
    }

    @Synchronized
    fun put(key: K, value: V) {
        val node = caches[key]
        if (node == null) {
            if (caches.size >= cacheCapacity) {
                removeLast()
            }
            val newNode = CacheNode()
            newNode.key = key
            newNode.value = value
            moveToFirst(newNode)
            caches[key] = newNode
        } else {
            node.value = value
            moveToFirst(node)
        }
    }

    @Synchronized
    fun get(key: K): V? {
        val node = caches[key] ?: return null
        moveToFirst(node)
        return node.value
    }

    @Synchronized
    fun remove(key: K): CacheNode? {
        val node = caches.remove(key) ?: return null
        if (node.pre != null) {
            node.pre!!.next = node.next
        }
        if (node.next != null) {
            node.next!!.pre = node.pre
        }
        if (node == first) {
            first = node.next
        }
        if (node == last) {
            last = node.pre
        }
        return node
    }

    @Synchronized
    fun size(): Int = caches.size

    @Synchronized
    fun keys(): Set<K> = caches.keys.toSet()

    @Synchronized
    fun clear() {
        caches.clear()
        first = null
        last = null
    }

    private fun moveToFirst(node: CacheNode) {
        if (node == first) return
        // Remove from current position
        if (node.pre != null) {
            node.pre!!.next = node.next
        }
        if (node.next != null) {
            node.next!!.pre = node.pre
        }
        if (node == last) {
            last = node.pre
        }
        // Move to first
        node.pre = null
        node.next = first
        if (first != null) {
            first!!.pre = node
        }
        first = node
        if (last == null) {
            last = node
        }
    }

    private fun removeLast() {
        if (last != null) {
            val key = last!!.key
            if (key != null) {
                caches.remove(key)
            }
            if (last!!.pre != null) {
                last!!.pre!!.next = null
            } else {
                first = null
            }
            last = last!!.pre
        }
    }

    override fun toString(): String {
        val sb = StringBuilder()
        var node = first
        while (node != null) {
            sb.append("${node.key}=${node.value}")
            node = node.next
            if (node != null) sb.append(", ")
        }
        return sb.toString()
    }
}
