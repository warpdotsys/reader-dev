use std::collections::VecDeque;

pub struct CollectionUtil;

impl CollectionUtil {

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
     * Returns the first element of the list, null if the list is null or empty.
     *
     * @param <T> f
     * @param list f
     * @return the first element of the list, null if the list is null or empty.
     */
    pub fn first<T>(list: &VecDeque<T>) -> Option<T> {
        if list.is_empty() {
            return None;
        }
        Some(list.get(0).clone())
    }

    /**
     * Whether the given collection is null or has no elements.
     *
     * @param collection g
     * @return Whether the given collection is null or has no elements.
     */
    pub fn is_empty<T>(collection: &VecDeque<T>) -> bool {
        collection.is_empty()
    }
}
