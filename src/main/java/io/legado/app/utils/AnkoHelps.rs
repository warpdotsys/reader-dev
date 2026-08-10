pub fn attempt<T>(f: impl FnOnce() -> T) -> AttemptResult<T> {
    let mut value: Option<T> = None;
    let mut error: Option<Box<dyn std::any::Any + Send>> = None;
    match std::panic::catch_unwind(f) {
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
