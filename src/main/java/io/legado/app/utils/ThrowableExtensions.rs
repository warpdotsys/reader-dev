use crate::prelude::*;
// fix: E0521 借用逃逸——stubs 中 ThrowableExt 仅对 `dyn Error + 'static` 实现，
// 因此参数需显式标注 'static，禁止借用逃逸出函数体
pub fn msg(t: &(dyn std::error::Error + 'static)) -> String {
    let stackTrace = t.stack_trace_to_string();
    let lMsg = t.msg().unwrap_or_else(|| "noErrorMsg".to_string());
    if !stackTrace.is_empty() {
        stackTrace
    } else {
        lMsg
    }
}
