use crate::prelude::*;

pub fn probe_min(a: i32) -> i32 {
    let ar = AnalyzeRule::new(1i32);
    a + ar.to_string().len() as i32
}
