use reader::stubs::Request;

#[test]
fn test_okhttp_execute() {
    let req = Request {
        url: "http://127.0.0.1:18999/search?key=test".to_string(),
        method: "GET".to_string(),
        ..Default::default()
    };
    let r = reader::runtime::okhttp::execute(&req);
    println!("result: {:?}", r.as_ref().map(|resp| (resp.status, resp.body_text.len())));
}
