use crate::prelude::*;
#[allow(unused)]
pub fn printOnDebug(t: &dyn std::fmt::Debug) {
    eprintln!("{:?}", t);
}
