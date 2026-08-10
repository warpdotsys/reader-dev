// package io.legado.app.help.coroutine

pub(crate) trait CoroutineContainer {
    fn add(&self, coroutine: std::rc::Rc<Coroutine<Box<dyn std::any::Any>>>) -> bool;

    fn add_all(&self, coroutines: &[std::rc::Rc<Coroutine<Box<dyn std::any::Any>>>]) -> bool;

    fn remove(&self, coroutine: std::rc::Rc<Coroutine<Box<dyn std::any::Any>>>) -> bool;

    fn delete(&self, coroutine: std::rc::Rc<Coroutine<Box<dyn std::any::Any>>>) -> bool;

    fn clear(&self);
}
