use crate::prelude::*;
pub fn jsonPath() -> ReadContext {
    JsonPath::using(
        Configuration::builder()
            .options(JsonPathOption::SUPPRESS_EXCEPTIONS)
            .build()
    )
}

pub fn readString(context: &mut ReadContext, path: &str) -> Option<String> {
    context.read::<String>(path).ok()
}

pub fn readBool(context: &mut ReadContext, path: &str) -> Option<bool> {
    context.read::<bool>(path).ok()
}

pub fn readInt(context: &mut ReadContext, path: &str) -> Option<i32> {
    context.read::<i32>(path).ok()
}

pub fn readLong(context: &mut ReadContext, path: &str) -> Option<i64> {
    context.read::<i64>(path).ok()
}
