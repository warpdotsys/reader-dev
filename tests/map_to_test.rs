use reader::stubs::JsonObject;

#[test]
fn test_book_group_deser() {
    let j = JsonObject(r#"{"groupName":"测试分组","groupId":0,"show":true}"#.to_string());
    match serde_json::from_str::<reader::io_legado_app_data_entities_bookgroup::BookGroup>(&j.0) {
        Ok(bg) => println!("OK: group_name={} show={}", bg.group_name, bg.show),
        Err(e) => println!("ERR: {}", e),
    }
    let g: Option<reader::io_legado_app_data_entities_bookgroup::BookGroup> =
        JsonObject::map_to_deser(&j);
    println!("map_to_deser -> {:?}", g.is_some());
}
