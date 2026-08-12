use crate::prelude::*;
// package com.htmake.reader.utils

// import java.util.concurrent.ConcurrentHashMap

/**
 * Thread-safe LRU cache with ConcurrentHashMap and doubly-linked list.
 */
// class LRUCache<K, V>(size: Int) {
pub struct LRUCache<K, V> {
    // private var cacheCapacity = size
    // private val caches = ConcurrentHashMap<K, CacheNode>(size)
    // private var first: CacheNode? = None
    // private var last: CacheNode? = None
    pub cache_capacity: usize,
    pub caches: std::collections::HashMap<K, Arc<RwLock<CacheNode<K, V>>>>,
    pub first: Option<Arc<RwLock<CacheNode<K, V>>>>,
    pub last: Option<Arc<RwLock<CacheNode<K, V>>>>,
}

// inner class CacheNode {
pub struct CacheNode<K, V> {
    // var pre: CacheNode? = None
    // var next: CacheNode? = None
    // var key: K? = None
    // var value: V? = None
    pub pre: Option<Arc<RwLock<CacheNode<K, V>>>>,
    pub next: Option<Arc<RwLock<CacheNode<K, V>>>>,
    pub key: Option<K>,
    pub value: Option<V>,
}

impl<K: Eq + Hash + Clone + std::fmt::Debug, V: Clone + std::fmt::Debug> LRUCache<K, V> {
    pub fn new(size: usize) -> LRUCache<K, V> {
        LRUCache {
            cache_capacity: size,
            caches: std::collections::HashMap::with_capacity(size),
            first: None,
            last: None,
        }
    }

    // fun put(key: K, value: V) {
    pub fn put(&mut self, key: K, value: V) {
        let existing_node = self.caches.get(&key).cloned();
        let node: Arc<RwLock<CacheNode<K, V>>>;
        if existing_node.is_none() {
            if self.caches.len() >= self.cache_capacity {
                self.caches.remove(&self.last.as_ref().unwrap().read().unwrap().key.clone().unwrap());
                self.remove_last();
            }
            node = Arc::new(RwLock::new(CacheNode {
                pre: None,
                next: None,
                key: Some(key.clone()),
                value: None,
            }));
        } else {
            node = existing_node.unwrap();
        }
        node.write().unwrap().value = Some(value);
        self.move_to_first(node.clone());
        self.caches.insert(key, node);
    }

    // fun get(key: K): V? {
    pub fn get(&mut self, key: &K) -> Option<V> {
        let node = self.caches.get(key).cloned()?;
        self.move_to_first(node.clone());
        return node.read().unwrap().value.clone();
    }

    // fun remove(key: K): CacheNode? {
    pub fn remove(&mut self, key: &K) -> Option<Arc<RwLock<CacheNode<K, V>>>> {
        let node = self.caches.get(key).cloned()?;
        if node.read().unwrap().pre.is_some() {
            node.read().unwrap().pre.clone().unwrap().write().unwrap().next = node.read().unwrap().next.clone();
        }
        if node.read().unwrap().next.is_some() {
            node.read().unwrap().next.clone().unwrap().write().unwrap().pre = node.read().unwrap().pre.clone();
        }
        if node.read().unwrap().key == self.first.as_ref().map(|f| f.read().unwrap().key.clone()).flatten() {
            self.first = node.read().unwrap().next.clone();
        }
        if node.read().unwrap().key == self.last.as_ref().map(|l| l.read().unwrap().key.clone()).flatten() {
            self.last = node.read().unwrap().pre.clone();
        }
        return Some(node);
    }

    // fun clear() {
    pub fn clear(&mut self) {
        self.caches.clear();
        self.first = None;
        self.last = None;
    }

    // private fun moveToFirst(node: CacheNode) {
    pub fn move_to_first(&mut self, node: Arc<RwLock<CacheNode<K, V>>>) {
        if self.first.as_ref().and_then(|f| f.read().unwrap().key.clone()) == node.read().unwrap().key {
            return;
        }
        if node.read().unwrap().next.is_some() {
            node.read().unwrap().next.clone().unwrap().write().unwrap().pre = node.read().unwrap().pre.clone();
        }
        if node.read().unwrap().pre.is_some() {
            node.read().unwrap().pre.clone().unwrap().write().unwrap().next = node.read().unwrap().next.clone();
        }
        if self.last.as_ref().and_then(|l| l.read().unwrap().key.clone()) == node.read().unwrap().key {
            self.last = node.read().unwrap().pre.clone();
        }
        if self.first.is_none() || self.last.is_none() {
            self.first = Some(node.clone());
            self.last = Some(node.clone());
            return;
        }
        node.write().unwrap().next = self.first.clone();
        self.first.clone().unwrap().write().unwrap().pre = Some(node.clone());
        self.first = Some(node.clone());
        self.first.clone().unwrap().write().unwrap().pre = None;
    }

    // private fun removeLast() {
    pub fn remove_last(&mut self) {
        if self.last.is_some() {
            if self.last.as_ref().unwrap().read().unwrap().pre.is_some() {
                self.last.as_ref().unwrap().read().unwrap().pre.clone().unwrap().write().unwrap().next = None;
            } else {
                self.first = None;
            }
            let prev = self.last.as_ref().unwrap().read().unwrap().pre.clone();
            self.last = prev;
        }
    }

    // override fun toString(): String {
    pub fn to_string(&self) -> String {
        let mut sb = StringBuilder::new();
        let mut node = self.first.clone();
        while node.is_some() {
            sb.append(format!("{:?}:{:?} ", node.as_ref().unwrap().read().unwrap().key, node.as_ref().unwrap().read().unwrap().value));
            let next = node.as_ref().unwrap().read().unwrap().next.clone();
            node = next;
        }
        return sb.to_string();
    }
}
