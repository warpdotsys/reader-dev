#[allow(unused)]
pub fn printOnDebug(t: &dyn std::fmt::Debug) {
    eprintln!("{:?}", t);
}
