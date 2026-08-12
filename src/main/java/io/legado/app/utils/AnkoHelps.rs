use crate::prelude::*;
pub fn attempt<T>(f: impl FnOnce() -> T) -> AttemptResult<T> {
    let mut value: Option<T> = None;
    let mut error: Option<Box<dyn std::any::Any + Send>> = None;
    // fix: 闭包参数跨 unwind 边界 → AssertUnwindSafe 包裹（同 SourceAnalyzer 惯例）
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(v) => value = Some(v),
        Err(t) => error = Some(t),
    }
    AttemptResult::new(value, error)
}

pub struct AttemptResult<T> {
    pub value: Option<T>,
    pub error: Option<Box<dyn std::any::Any + Send>>,
}

impl<T> AttemptResult<T> {
    pub fn new(value: Option<T>, error: Option<Box<dyn std::any::Any + Send>>) -> AttemptResult<T> {
        AttemptResult { value, error }
    }

    pub fn then<R>(self, f: impl FnOnce(T) -> R) -> AttemptResult<R> {
        if self.isError() {
            return AttemptResult::new(None, self.error);
        }
        let v = self.value.unwrap();
        attempt(move || f(v))
    }

    pub fn isError(&self) -> bool {
        self.error.is_some()
    }

    pub fn hasValue(&self) -> bool {
        self.error.is_none()
    }
}
