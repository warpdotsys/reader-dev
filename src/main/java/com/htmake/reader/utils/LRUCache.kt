package com.htmake.reader.utils

import java.util.concurrent.ConcurrentHashMap

/**
 * Thread-safe LRU cache with ConcurrentHashMap and doubly-linked list.
 */
class LRUCache<K, V>(size: Int) {

    private var cacheCapacity = size
    private val caches = ConcurrentHashMap<K, CacheNode>(size)
    private var first: CacheNode? = null
    private var last: CacheNode? = null

    inner class CacheNode {
        var pre: CacheNode? = null
        var next: CacheNode? = null
        var key: K? = null
        var value: V? = null
    }

    fun put(key: K, value: V) {
        val existingNode = caches[key]
        val node: CacheNode
        if (existingNode == null) {
            if (caches.size >= cacheCapacity) {
                caches.remove(last?.key)
                removeLast()
            }
            node = CacheNode()
            node.key = key
        } else {
            node = existingNode
        }
        node.value = value
        moveToFirst(node)
        caches[key] = node
    }

    fun get(key: K): V? {
        val node = caches[key] ?: return null
        moveToFirst(node)
        return node.value
    }

    fun remove(key: K): CacheNode? {
        val node = caches[key] ?: return null
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

    fun clear() {
        caches.clear()
        first = null
        last = null
    }

    private fun moveToFirst(node: CacheNode) {
        if (node == first) return
        if (node.next != null) {
            node.next!!.pre = node.pre
        }
        if (node.pre != null) {
            node.pre!!.next = node.next
        }
        if (node == last) {
            last = node.pre
        }
        if (first == null || last == null) {
            first = node
            last = node
            return
        }
        node.next = first
        first!!.pre = node
        first = node
        first!!.pre = null
    }

    private fun removeLast() {
        if (last != null) {
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
            sb.append(String.format("%s:%s ", node.key, node.value))
            node = node.next
        }
        return sb.toString()
    }
}
