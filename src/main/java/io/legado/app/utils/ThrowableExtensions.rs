pub fn msg(t: &dyn std::error::Error) -> String {
    let stackTrace = stackTraceToString(t);
    let lMsg = t.localizedMessage().unwrap_or("noErrorMsg");
    if !stackTrace.is_empty() {
        stackTrace
    } else {
        lMsg
    }
}
