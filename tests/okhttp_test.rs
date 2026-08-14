use reader::stubs::Request;

#[test]
fn test_okhttp_execute() {
    let req = Request {
        url: "http://127.0.0.1:18999/search?key=test".to_string(),
        method: "GET".to_string(),
        ..Default::default()
    };
    let r = reader::runtime::okhttp::execute(&req, None);
    println!("result: {:?}", r.as_ref().map(|resp| (resp.status, resp.body_text.len())));
}

#[test]
fn test_multipart_build() {
    use reader::stubs::{MultipartBody, RequestBody};
    let mb = MultipartBody::builder();
    mb.add_form_data_part("field1", "", RequestBody::from_text("value1"));
    mb.add_form_data_part("file", "a.txt", RequestBody::from_text("hello"));
    let body = mb.build();
    assert!(body.text.contains("name=\"field1\""), "field1 name part");
    assert!(body.text.contains("value1"), "field1 value");
    assert!(body.text.contains("filename=\"a.txt\""), "file filename");
    assert!(body.text.contains("hello"), "file content");
    assert!(body.text.ends_with("--\r\n"), "boundary close");
}