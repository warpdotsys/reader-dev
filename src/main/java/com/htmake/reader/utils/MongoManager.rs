use crate::prelude::*;
// package com.htmake.reader.utils

// import com.mongodb.MongoException
// import com.mongodb.MongoClientSettings
// import com.mongodb.client.MongoClient
// import com.mongodb.client.MongoClients
// import com.mongodb.client.MongoCollection
// import com.mongodb.client.MongoDatabase
// import org.bson.codecs.configuration.CodecRegistries
// import org.bson.codecs.configuration.CodecProvider
// import org.bson.codecs.pojo.PojoCodecProvider
// import com.htmake.reader.entity.MongoFile

// object MongoManager {
pub struct MongoManager;

// fix: 原 impl 内 static 移到模块级（Rust 不允许关联 static）
// private var mongoClient: MongoClient? = None
pub static MONGO_CLIENT: std::sync::OnceLock<Option<MongoClient>> = std::sync::OnceLock::new();

impl MongoManager {
    // fun isInit(): Boolean {
    pub fn is_init() -> bool {
        return MONGO_CLIENT.get_or_init(|| None).is_some();
    }

    // fun connect(uri: String) {
    pub fn connect(uri: &str) {
        // fix: try/catch → 闭包 + if-let
        let try_result: Result<(), StubError> = (|| {
            let client = MongoClients::create(uri);
            // fix: OnceLock 一次性写入（原 `get_mut()` + 覆盖 写法非法）；重复 connect 时保留首次连接
            let _ = MONGO_CLIENT.set(Some(client));
            Ok(())
        })();
        if let Err(e) = try_result {
            logger().info(format!("mongodb 连接失败，请检查链接({})是否正确", uri));
            e.printStackTrace();
        }
    }

    // fun db(name: String): MongoDatabase? {
    pub fn db(name: &str) -> Option<MongoDatabase> {
        if !Self::is_init() {
            return None;
        }
        let pojo_codec_provider = PojoCodecProvider::builder().automatic(true).build();
        let codec_registry = CodecRegistries::from_registries(&[
            MongoClientSettings::get_default_codec_registry(),
            CodecRegistries::from_providers(pojo_codec_provider),
        ]);
        return MONGO_CLIENT
            .get()
            .unwrap()
            .as_ref()
            .unwrap()
            .get_database(name)
            .map(|db| db.with_codec_registry(codec_registry));
    }

    // fun fileStorage(dbName: String, collection: String): MongoCollection<MongoFile>? {
    pub fn file_storage(db_name: &str, collection: &str) -> Option<MongoCollection<MongoFile>> {
        let database = Self::db(db_name)?;
        return database.get_collection(collection, MongoFile::class);
    }
}
