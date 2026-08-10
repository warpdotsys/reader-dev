// package io.legado.app.help.coroutine

use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Mutex;

pub struct CompositeCoroutine {
    resources: Mutex<Option<HashSet<Rc<Coroutine<Box<dyn std::any::Any>>>>>>,
}

impl CompositeCoroutine {
    pub fn new() -> CompositeCoroutine {
        CompositeCoroutine {
            resources: Mutex::new(None),
        }
    }

    pub fn from_iter(coroutines: impl IntoIterator<Item = Rc<Coroutine<Box<dyn std::any::Any>>>>) -> CompositeCoroutine {
        // constructor(coroutines: Iterable<Coroutine<*>>) {
        //     this.resources = hashSetOf()
        //     for (d in coroutines) {
        //         this.resources?.add(d)
        //     }
        // }
        let mut set: HashSet<Rc<Coroutine<Box<dyn std::any::Any>>>> = HashSet::new();
        for d in coroutines {
            set.insert(d);
        }
        CompositeCoroutine {
            resources: Mutex::new(Some(set)),
        }
    }
}

impl CoroutineContainer for CompositeCoroutine {
    // override fun add(coroutine: Coroutine<*>): Boolean {
    //     synchronized(this) {
    //         var set: HashSet<Coroutine<*>>? = resources
    //         if (resources == null) {
    //             set = hashSetOf()
    //             resources = set
    //         }
    //         return set!!.add(coroutine)
    //     }
    // }
    fn add(&self, coroutine: Rc<Coroutine<Box<dyn std::any::Any>>>) -> bool {
        let mut guard = self.resources.lock().unwrap();
        let mut set = guard.take();
        if set.is_none() {
            set = Some(HashSet::new());
        }
        let result = set.as_mut().unwrap().insert(coroutine);
        *guard = set;
        result
    }

    // override fun addAll(vararg coroutines: Coroutine<*>): Boolean {
    //     synchronized(this) {
    //         var set: HashSet<Coroutine<*>>? = resources
    //         if (resources == null) {
    //             set = hashSetOf()
    //             resources = set
    //         }
    //         for (coroutine in coroutines) {
    //             val add = set!!.add(coroutine)
    //             if (!add) {
    //                 return false
    //             }
    //         }
    //     }
    //     return true
    // }
    fn add_all(&self, coroutines: &[Rc<Coroutine<Box<dyn std::any::Any>>>]) -> bool {
        let mut guard = self.resources.lock().unwrap();
        let mut set = guard.take();
        if set.is_none() {
            set = Some(HashSet::new());
        }
        for coroutine in coroutines {
            let add = set.as_mut().unwrap().insert(coroutine.clone());
            if !add {
                return false;
            }
        }
        *guard = set;
        true
    }

    // override fun remove(coroutine: Coroutine<*>): Boolean {
    //     if (delete(coroutine)) {
    //         coroutine.cancel()
    //         return true
    //     }
    //     return false
    // }
    fn remove(&self, coroutine: Rc<Coroutine<Box<dyn std::any::Any>>>) -> bool {
        if self.delete(coroutine.clone()) {
            coroutine.cancel(None);
            return true;
        }
        false
    }

    // override fun delete(coroutine: Coroutine<*>): Boolean {
    //     synchronized(this) {
    //         val set = resources
    //         if (set == null || !set.remove(coroutine)) {
    //             return false
    //         }
    //     }
    //     return true
    // }
    fn delete(&self, coroutine: Rc<Coroutine<Box<dyn std::any::Any>>>) -> bool {
        {
            let mut guard = self.resources.lock().unwrap();
            let set = guard.as_mut();
            if set.is_none() || !set.unwrap().remove(&coroutine) {
                return false;
            }
        }
        true
    }

    // override fun clear() {
    //     val set: HashSet<Coroutine<*>>?
    //     synchronized(this) {
    //         set = resources
    //         resources = null
    //     }
    //
    //     set?.forEachIndexed { _, coroutine ->
    //         coroutine.cancel()
    //     }
    // }
    fn clear(&self) {
        let set: Option<HashSet<Rc<Coroutine<Box<dyn std::any::Any>>>>;
        {
            let mut guard = self.resources.lock().unwrap();
            set = guard.take();
        }

        if let Some(set) = set {
            for coroutine in set {
                coroutine.cancel(None);
            }
        }
    }
}
