use crate::prelude::*;
use std::collections::VecDeque;

pub struct CollectionUtil;

// fix: Java 嵌套类 IteratorEnumerationAdapter 移出 impl 到模块级（Rust 不允许在 impl 中嵌套 struct/impl）
/**
 * Wraps an Enumeration around an Iterator
 * @author paul.siegmann
 *
 * @param <T>
 */
pub struct IteratorEnumerationAdapter<T> {
    iterator: Box<dyn Iterator<Item = T>>,
}

impl<T> IteratorEnumerationAdapter<T> {
    pub fn new(iter: Box<dyn Iterator<Item = T>>) -> Self {
        IteratorEnumerationAdapter { iterator: iter }
    }

    pub fn has_more_elements(&mut self) -> bool {
        self.iterator.next().is_some()
    }

    pub fn next_element(&mut self) -> Option<T> {
        self.iterator.next()
    }
}

impl CollectionUtil {

    /**
     * Creates an Enumeration out of the given Iterator.
     * @param <T>  g
     * @param it g
     * @return an Enumeration created out of the given Iterator.
     */
    #[allow(dead_code)]
    pub fn create_enumeration_from_iterator<T>(it: Box<dyn Iterator<Item = T>>) -> IteratorEnumerationAdapter<T> {
        IteratorEnumerationAdapter::new(it)
    }

    /**
     * Returns the first element of the list, None if the list is None or empty.
     *
     * @param <T> f
     * @param list f
     * @return the first element of the list, None if the list is None or empty.
     */
    // fix: 增加 T: Clone 约束——Rust 需从 &T 克隆出所有权 T（Java `list.get(0)` 直接返回引用）
    pub fn first<T: Clone>(list: &VecDeque<T>) -> Option<T> {
        if list.is_empty() {
            return None;
        }
        Some(list.get(0).unwrap().clone())
    }

    /**
     * Whether the given collection is None or has no elements.
     *
     * @param collection g
     * @return Whether the given collection is None or has no elements.
     */
    pub fn is_empty<T>(collection: &VecDeque<T>) -> bool {
        collection.is_empty()
    }
}
