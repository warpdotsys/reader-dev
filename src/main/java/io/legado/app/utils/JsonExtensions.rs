pub fn jsonPath() -> ParseContext {
    JsonPath::using(
        Configuration::builder()
            .options(Option::SUPPRESS_EXCEPTIONS)
            .build()
    )
}

pub fn readString(context: &mut ReadContext, path: &str) -> Option<String> {
    context.read::<String>(path)
}

pub fn readBool(context: &mut ReadContext, path: &str) -> Option<bool> {
    context.read::<bool>(path)
}

pub fn readInt(context: &mut ReadContext, path: &str) -> Option<i32> {
    context.read::<i32>(path)
}

pub fn readLong(context: &mut ReadContext, path: &str) -> Option<i64> {
    context.read::<i64>(path)
}
