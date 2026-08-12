trait Super {
    fn get_name(&self) -> String;
    fn match_det(&self) -> i32;
    fn get_language(&self) -> Option<String> { None }
}
trait Sub: Super {
    fn next_char(&self) -> bool;
}
struct X;
impl Sub for X {
    fn next_char(&self) -> bool { true }
    fn match_det(&self) -> i32 { 5 }
    fn get_name(&self) -> String { "X".into() }
    fn get_language(&self) -> Option<String> { Some("en".into()) }
}
fn main() {}
