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

impl MongoManager {
    // private var mongoClient: MongoClient? = null
    static MONGO_CLIENT: std::sync::OnceLock<Option<MongoClient>> = std::sync::OnceLock::new();

    // fun isInit(): Boolean {
    pub fn is_init() -> bool {
        return MONGO_CLIENT.get_or_init(|| None).is_some();
    }

    // fun connect(uri: String) {
    pub fn connect(uri: &str) {
        try {
            let client = MongoClients::create(uri);
            MONGO_CLIENT.get_or_init(|| None);
            *MONGO_CLIENT.get_mut() = Some(client);
        } catch (e: MongoException) {
            logger.info(format!("mongodb 连接失败，请检查链接({})是否正确", uri));
            e.printStackTrace();
        }
    }

    // fun db(name: String): MongoDatabase? {
    pub fn db(name: &str) -> Option<MongoDatabase> {
        if !Self::is_init() {
            return None;
        }
        let pojo_codec_provider = PojoCodecProvider::builder().automatic(true).build();
        let codec_registry = CodecRegistries::from_registries(
            MongoClientSettings::get_default_codec_registry(),
            CodecRegistries::from_providers(pojo_codec_provider as CodecProvider),
        );
        return MONGO_CLIENT.get().unwrap().as_ref().unwrap().get_database(name).with_codec_registry(codec_registry);
    }

    // fun fileStorage(dbName: String, collection: String): MongoCollection<MongoFile>? {
    pub fn file_storage(db_name: &str, collection: &str) -> Option<MongoCollection<MongoFile>> {
        let database = Self::db(db_name)?;
        return database.get_collection(collection, MongoFile::class);
    }
}
