// package com.htmake.reader.utils

// import java.util.concurrent.ConcurrentHashMap

/**
 * Thread-safe LRU cache with ConcurrentHashMap and doubly-linked list.
 */
// class LRUCache<K, V>(size: Int) {
pub struct LRUCache<K, V> {
    // private var cacheCapacity = size
    // private val caches = ConcurrentHashMap<K, CacheNode>(size)
    // private var first: CacheNode? = null
    // private var last: CacheNode? = null
    pub cache_capacity: usize,
    pub caches: std::collections::HashMap<K, Rc<RefCell<CacheNode<K, V>>>>,
    pub first: Option<Rc<RefCell<CacheNode<K, V>>>>,
    pub last: Option<Rc<RefCell<CacheNode<K, V>>>>,
}

// inner class CacheNode {
pub struct CacheNode<K, V> {
    // var pre: CacheNode? = null
    // var next: CacheNode? = null
    // var key: K? = null
    // var value: V? = null
    pub pre: Option<Rc<RefCell<CacheNode<K, V>>>>,
    pub next: Option<Rc<RefCell<CacheNode<K, V>>>>,
    pub key: Option<K>,
    pub value: Option<V>,
}

impl<K: Eq + Hash + Clone, V: Clone> LRUCache<K, V> {
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
        let node: Rc<RefCell<CacheNode<K, V>>>;
        if existing_node.is_none() {
            if self.caches.len() >= self.cache_capacity {
                self.caches.remove(&self.last.as_ref().unwrap().borrow().key.clone().unwrap());
                self.remove_last();
            }
            node = Rc::new(RefCell::new(CacheNode {
                pre: None,
                next: None,
                key: Some(key.clone()),
                value: None,
            }));
        } else {
            node = existing_node.unwrap();
        }
        node.borrow_mut().value = Some(value);
        self.move_to_first(node.clone());
        self.caches.insert(key, node);
    }

    // fun get(key: K): V? {
    pub fn get(&mut self, key: &K) -> Option<V> {
        let node = self.caches.get(key).cloned()?;
        self.move_to_first(node.clone());
        return node.borrow().value.clone();
    }

    // fun remove(key: K): CacheNode? {
    pub fn remove(&mut self, key: &K) -> Option<Rc<RefCell<CacheNode<K, V>>>> {
        let node = self.caches.get(key).cloned()?;
        if node.borrow().pre.is_some() {
            node.borrow().pre.clone().unwrap().borrow_mut().next = node.borrow().next.clone();
        }
        if node.borrow().next.is_some() {
            node.borrow().next.clone().unwrap().borrow_mut().pre = node.borrow().pre.clone();
        }
        if node.borrow().key == self.first.as_ref().map(|f| f.borrow().key.clone()).flatten() {
            self.first = node.borrow().next.clone();
        }
        if node.borrow().key == self.last.as_ref().map(|l| l.borrow().key.clone()).flatten() {
            self.last = node.borrow().pre.clone();
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
    pub fn move_to_first(&mut self, node: Rc<RefCell<CacheNode<K, V>>>) {
        if self.first.as_ref().map(|f| f.borrow().key.clone()) == node.borrow().key {
            return;
        }
        if node.borrow().next.is_some() {
            node.borrow().next.clone().unwrap().borrow_mut().pre = node.borrow().pre.clone();
        }
        if node.borrow().pre.is_some() {
            node.borrow().pre.clone().unwrap().borrow_mut().next = node.borrow().next.clone();
        }
        if self.last.as_ref().map(|l| l.borrow().key.clone()) == node.borrow().key {
            self.last = node.borrow().pre.clone();
        }
        if self.first.is_none() || self.last.is_none() {
            self.first = Some(node.clone());
            self.last = Some(node.clone());
            return;
        }
        node.borrow_mut().next = self.first.clone();
        self.first.clone().unwrap().borrow_mut().pre = Some(node.clone());
        self.first = Some(node.clone());
        self.first.clone().unwrap().borrow_mut().pre = None;
    }

    // private fun removeLast() {
    pub fn remove_last(&mut self) {
        if self.last.is_some() {
            if self.last.as_ref().unwrap().borrow().pre.is_some() {
                self.last.as_ref().unwrap().borrow().pre.clone().unwrap().borrow_mut().next = None;
            } else {
                self.first = None;
            }
            self.last = self.last.as_ref().unwrap().borrow().pre.clone();
        }
    }

    // override fun toString(): String {
    pub fn to_string(&self) -> String {
        let mut sb = StringBuilder::new();
        let mut node = self.first.clone();
        while node.is_some() {
            sb.append(format!("{:?}:{:?} ", node.as_ref().unwrap().borrow().key, node.as_ref().unwrap().borrow().value));
            node = node.as_ref().unwrap().borrow().next.clone();
        }
        return sb.to_string();
    }
}
