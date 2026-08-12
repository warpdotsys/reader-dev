pub struct OkHttpClient;
pub fn new_call_str_response(client: &OkHttpClient, retry: i32, builder: impl FnOnce(&mut i32)) -> i32 { retry }
pub fn get_proxy_client(proxy: Option<&str>, debug_log: Option<&i32>) -> OkHttpClient { OkHttpClient }
pub fn url(b: &mut i32, u: &str) {}
pub fn post_json(b: &mut i32, j: Option<&str>) {}
fn main() {
    let debug_log = None::<i32>;
    let r = get_proxy_client(None, debug_log).new_call_str_response(0, || {
        url("api_url");
        post_json(Some("{}"));
    });
    let _ = r;
}
