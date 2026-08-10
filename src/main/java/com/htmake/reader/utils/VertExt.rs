// @file:JvmName("ExtKt")
// @file:JvmMultifileClass

// package com.htmake.reader.utils

// import com.google.gson.Gson
// import com.google.gson.GsonBuilder
// import io.vertx.core.json.JsonObject
// import io.vertx.core.json.JsonArray
// import mu.KotlinLogging
// import java.io.File
// import java.nio.file.Files
// import java.nio.file.Paths
// import java.nio.file.StandardCopyOption
// import java.util.concurrent.TimeUnit
// import java.util.concurrent.locks.ReadWriteLock
// import java.util.concurrent.locks.ReentrantReadWriteLock
// import com.htmake.reader.config.AppConfig
// import com.google.gson.reflect.TypeToken
// import kotlin.reflect.KProperty1
// import kotlin.reflect.KMutableProperty
// import kotlin.reflect.full.memberProperties
// import io.legado.app.data.entities.Book
// import io.legado.app.utils.FileUtils
// import io.legado.app.utils.MD5Utils
// import io.legado.app.utils.MapDeserializerDoubleAsIntFix
// import java.util.UUID
// import java.util.Base64 as JavaBase64
// import com.mongodb.client.MongoCollection
// import com.htmake.reader.entity.MongoFile
// import com.fasterxml.jackson.core.JsonToken
// import com.fasterxml.jackson.databind.ObjectMapper
// import com.fasterxml.jackson.databind.node.ObjectNode

/**
 * @Auther: zoharSoul
 * @Date: 2019-05-21 16:17
 * @Description:
 */
// val logger = KotlinLogging.logger {}

// val gson = GsonBuilder()
//     .registerTypeAdapter(object : TypeToken<Map<String, Any>>() {}.type, MapDeserializerDoubleAsIntFix())
//     .registerTypeAdapter(Int::class.javaPrimitiveType!!, IntTypeAdapter())
//     .registerTypeAdapter(Long::class.javaPrimitiveType!!, LongTypeAdapter())
//     .disableHtmlEscaping()
//     .create()
pub fn gson() -> &'static Gson {
    static GSON: std::sync::OnceLock<Gson> = std::sync::OnceLock::new();
    GSON.get_or_init(|| {
        GsonBuilder::new()
            .register_type_adapter(TypeToken::new::<std::collections::HashMap<String, Any>>().get_type(), MapDeserializerDoubleAsIntFix::new())
            .register_type_adapter(i32::class.java_primitive_type(), IntTypeAdapter::new())
            .register_type_adapter(i64::class.java_primitive_type(), LongTypeAdapter::new())
            .disable_html_escaping()
            .create()
    })
}

// val prettyGson = GsonBuilder()
//     .registerTypeAdapter(object : TypeToken<Map<String, Any>>() {}.type, MapDeserializerDoubleAsIntFix())
//     .registerTypeAdapter(Int::class.javaPrimitiveType!!, IntTypeAdapter())
//     .registerTypeAdapter(Long::class.javaPrimitiveType!!, LongTypeAdapter())
//     .disableHtmlEscaping()
//     .setPrettyPrinting()
//     .create()
pub fn pretty_gson() -> &'static Gson {
    static PRETTY_GSON: std::sync::OnceLock<Gson> = std::sync::OnceLock::new();
    PRETTY_GSON.get_or_init(|| {
        GsonBuilder::new()
            .register_type_adapter(TypeToken::new::<std::collections::HashMap<String, Any>>().get_type(), MapDeserializerDoubleAsIntFix::new())
            .register_type_adapter(i32::class.java_primitive_type(), IntTypeAdapter::new())
            .register_type_adapter(i64::class.java_primitive_type(), LongTypeAdapter::new())
            .disable_html_escaping()
            .set_pretty_printing()
            .create()
    })
}

// var storageFinalPath = ""
// var workDirPath = ""
// var workDirInit = false
// private const val MAX_CACHE_SIZE = 1000
// private val storageLocks = LRUCache<String, ReadWriteLock>(MAX_CACHE_SIZE)
static STORAGE_FINAL_PATH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
static WORK_DIR_PATH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
static WORK_DIR_INIT: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
const MAX_CACHE_SIZE: usize = 1000;
static STORAGE_LOCKS: std::sync::OnceLock<LRUCache<String, ReadWriteLock>> = std::sync::OnceLock::new();

// fun getWorkDir(subPath: String = ""): String {
pub fn get_work_dir(sub_path: &str) -> String {
    if !*WORK_DIR_INIT.get_or_init(|| false) && WORK_DIR_PATH.get().map(|p| p.is_empty()).unwrap_or(true) {
        let app_config = SpringContextUtils::get_bean_by_name_and_class("appConfig", AppConfig::class);
        if let Some(cfg) = app_config {
            if !cfg.work_dir.is_empty() && cfg.work_dir != "." {
                let work_dir_file = File::new(&cfg.work_dir);
                if work_dir_file.exists() && !work_dir_file.is_directory {
                    logger.error(format!("reader.app.workDir={} is not a directory", cfg.work_dir));
                } else {
                    if !work_dir_file.exists() {
                        logger.info(format!("reader.app.workDir={} not exists, creating", cfg.work_dir));
                        work_dir_file.mkdirs();
                    }
                    *WORK_DIR_PATH.get_or_init(|| String::new()) = work_dir_file.absolute_path;
                }
            }
        }
        if WORK_DIR_PATH.get().map(|p| p.is_empty()).unwrap_or(true) {
            let os_name = System::get_property("os.name");
            let current_dir = System::get_property("user.dir");
            logger.info(format!("osName: {} currentDir: {}", os_name, current_dir));
            if os_name.starts_with("Mac OS") && !current_dir.starts_with("/Users/") {
                *WORK_DIR_PATH.get_or_init(|| String::new()) = Paths::get(System::get_property("user.home"), ".reader").to_string();
            } else {
                *WORK_DIR_PATH.get_or_init(|| String::new()) = current_dir;
            }
        }
        logger.info(format!("Using workdir: {}", WORK_DIR_PATH.get().unwrap()));
        *WORK_DIR_INIT.get_or_init(|| false) = true;
    }
    let path = Paths::get(WORK_DIR_PATH.get().unwrap(), sub_path);

    return path.to_string();
}

// fun getWorkDir(vararg subDirFiles: String): String {
pub fn get_work_dir_multi(sub_dir_files: &[&str]) -> String {
    return get_work_dir(&get_relative_path(sub_dir_files));
}

// fun getRelativePath(vararg subDirFiles: String): String {
pub fn get_relative_path(sub_dir_files: &[&str]) -> String {
    let mut path = StringBuilder::new("");
    sub_dir_files.for_each(|it| {
        if !it.is_empty() {
            path.append(File::SEPARATOR.to_string() + it);
        }
    });
    return path.to_string().let(|it| {
        if it.starts_with("/") {
            it.substring(1)
        } else {
            it
        }
    });
}

// fun getStoragePath(): String {
pub fn get_storage_path() -> String {
    if !STORAGE_FINAL_PATH.get().map(|p| p.is_empty()).unwrap_or(true) {
        return STORAGE_FINAL_PATH.get().unwrap().clone();
    }
    let mut storage_path = String::new();
    let app_config = SpringContextUtils::get_bean_by_name_and_class("appConfig", AppConfig::class);
    if app_config.is_some() {
        storage_path = get_work_dir("storage");
        *STORAGE_FINAL_PATH.get_or_init(|| String::new()) = storage_path.clone();
    } else {
        storage_path = File::new("storage").path();
    }
    logger.info(format!("Using storagePath: {}", storage_path));
    return storage_path;
}

// fun saveStorage(vararg name: String, value: Any, pretty: Boolean = false, ext: String = ".json") {
pub fn save_storage(name: &[String], value: Any, pretty: bool, ext: &str) {
    let to_json: String = if value is String {
        value.to_string()
    } else if value is JsonObject || value is JsonArray {
        value.to_string()
    } else if pretty {
        pretty_gson().to_json(value)
    } else {
        gson().to_json(value)
    };

    let storage_path = get_storage_path();
    let storage_dir = File::new(&storage_path);
    if !storage_dir.exists() {
        storage_dir.mkdirs();
    }

    let filename = name.last().clone();
    let path = get_relative_path(&[&name[0..name.len() - 1], &(filename + ext)]);
    let file = File::new(&storage_path).resolve(&path);
    logger.info(format!("Save file to storage name: {:?} path: {}", name, file.absolute_file()));

    if !file.parent_file().exists() {
        file.parent_file().mkdirs();
    }

    let lock = storage_lock(&file);
    let mut acquired = false;
    try {
        acquired = lock.write_lock().try_lock(10, TimeUnit::SECONDS);
        if !acquired {
            panic!(format!("保存文件超时: {}", file.absolute_path));
        }

        let base_name = file.name_without_extension();
        let temp = Files::create_temp_file(file.parent_file().to_path().to_absolute_path(), base_name, ".temp");
        Files::write(&temp, to_json.as_bytes());

        let file_path = file.to_path();
        let backup_path = file.parent_file().to_path().resolve(&(base_name.to_string() + ".backup.json")).to_absolute_path();
        if Files::exists(&file_path) {
            Files::move_path(&file_path, &backup_path, StandardCopyOption::ATOMIC_MOVE);
        }
        Files::move_path(&temp, &file_path, StandardCopyOption::ATOMIC_MOVE);
        Files::delete_if_exists(&temp);

        if base_name.len() >= 32 {
            Files::delete_if_exists(&backup_path);
        }
        if base_name == "users" {
            let verify_file = File::new(&storage_path).resolve(&get_relative_path(&[&name[0..name.len() - 1], &(".".to_string() + &base_name + ".key")]));
            if !verify_file.exists() {
                verify_file.create_new_file();
            }
            let verification = MD5Utils::md5_encode(format!("userCount={}", count_occurrences(&to_json, "username"))).take_last(16);
            verify_file.write_text(&verification);
        }
        save_mongo_file(&path, &to_json);
    } catch (e: Exception) {
        logger.error(format!("保存文件失败: {}", e));
        panic!(format!("保存文件失败: {}", file.absolute_path));
    } finally {
        if acquired {
            lock.write_lock().unlock();
        }
    }
}

// fun getStorage(vararg name: String, ext: String = ".json"): String?  {
pub fn get_storage(name: &[String], ext: &str) -> Option<String> {
    let storage_path = get_storage_path();
    let filename = name.last().clone();
    let path = get_relative_path(&[&name[0..name.len() - 1], &(filename + ext)]);
    let file = File::new(&storage_path).resolve(&path);
    logger.info(format!("Read file from storage name: {:?} path: {}", name, file.absolute_file()));
    if !file.exists() {
        let content = read_mongo_file(&path);
        if !content.is_empty() {
            if !file.parent_file().exists() {
                file.parent_file().mkdirs();
            }
            file.create_new_file();
            file.write_text(&content);
            return Some(content);
        }
        return None;
    }

    let lock = storage_lock(&file);
    let mut acquired = false;
    try {
        acquired = lock.read_lock().try_lock(10, TimeUnit::SECONDS);
        if !acquired {
            panic!(format!("读取文件超时: {}", file.absolute_path));
        }
        let mut content = file.read_text();
        if content.is_empty() {
            let mongo_content = read_mongo_file(&path);
            if !mongo_content.is_empty() {
                file.write_text(&mongo_content);
                content = mongo_content;
            }
        }
        if filename == "users" {
            let verify_file = File::new(&storage_path).resolve(&get_relative_path(&[&name[0..name.len() - 1], &(".".to_string() + &filename + ".key")]));
            if verify_file.exists() {
                let verification = MD5Utils::md5_encode(format!("userCount={}", count_occurrences(&content, "username"))).take_last(16);
                if verify_file.read_text() != verification {
                    panic!("用户数据被篡改，请联系开发者修复");
                }
            }
        }
        return Some(content);
    } catch (e: Exception) {
        logger.error(format!("读取文件失败: {}", e));
        panic!(format!("读取文件失败: {}", file.absolute_path));
    } finally {
        if acquired {
            lock.read_lock().unlock();
        }
    }
}

// fun asJsonArray(value: Any?): JsonArray? {
pub fn as_json_array(value: Option<Any>) -> Option<JsonArray> {
    if value is JsonArray {
        return Some(value);
    } else if value is String {
        return try {
            Some(JsonArray::new(value))
        } catch (e: Exception) {
            logger.error(format!("解析内容出错: {}  内容: \n{}", e, value));
            panic!(e);
        };
    }
    return None;
}

// fun asJsonObject(value: Any?): JsonObject? {
pub fn as_json_object(value: Option<Any>) -> Option<JsonObject> {
    if value is JsonObject {
        return Some(value);
    } else if value is String {
        return try {
            Some(JsonObject::new(value))
        } catch (e: Exception) {
            logger.error(format!("解析内容出错: {}  内容: \n{}", e, value));
            panic!(e);
        };
    }
    return None;
}

//convert a data class to a map
// fun <T> T.serializeToMap(): Map<String, Any> {
pub fn serialize_to_map<T>(this: T) -> std::collections::HashMap<String, Any> {
    return convert(this);
}

//convert string to a map
// fun <T> T.toMap(): Map<String, Any> {
pub fn to_map<T>(this: T) -> std::collections::HashMap<String, Any> {
    return convert(this);
}

//convert a map to a data class
// inline fun <reified T> Map<String, Any>.toDataClass(): T {
pub fn to_data_class<T>(this: std::collections::HashMap<String, Any>) -> T {
    return convert(this);
}

//convert an object of type I to type O
// inline fun <I, reified O> I.convert(): O {
pub fn convert<I, O>(this: I) -> O {
    let json = if this is String {
        this.to_string()
    } else {
        gson().to_json(this)
    };
    return gson().from_json(&json, TypeToken::new::<O>().get_type());
}

// @Suppress("UNCHECKED_CAST")
// fun <R> readInstanceProperty(instance: Any, propertyName: String): R {
pub fn read_instance_property<R>(instance: Any, property_name: &str) -> R {
    let property = instance.class().member_properties()
        // don't cast here to <Any, R>, it would succeed silently
        .first(|it| it.name == property_name) as KProperty1;
    // force a invalid cast exception if incorrect type here
    return property.get(instance) as R;
}

// @Suppress("UNCHECKED_CAST")
// fun setInstanceProperty(instance: Any, propertyName: String, propertyValue: Any) {
pub fn set_instance_property(instance: Any, property_name: &str, property_value: Any) {
    let property = instance.class().member_properties()
        .first(|it| it.name == property_name);
    if property is KMutableProperty {
        property.setter().call(instance, property_value);
    }
}

// fun Book.fillData(newBook: Book, keys: List<String>): Book {
pub fn fill_data(this: Book, new_book: Book, keys: Vec<String>) -> Book {
    keys.let(|it| {
        for key in it {
            let mut current = read_instance_property::<String>(this, &key);
            if current.is_empty() {
                let cache_value = read_instance_property::<String>(new_book, &key);
                if !cache_value.is_empty() {
                    set_instance_property(this, &key, cache_value);
                }
            }
        }
    });
    return this;
}

// fun getRandomString(length: Int) : String {
pub fn get_random_string(length: i32) -> String {
    let allowed_chars = "ABCDEFGHIJKLMNOPQRSTUVWXTZabcdefghiklmnopqrstuvwxyz0123456789";
    return (1..=length)
        .map(|_| allowed_chars.random())
        .join_to_string("");
}

// fun genEncryptedPassword(password: String, salt: String): String {
pub fn gen_encrypted_password(password: &str, salt: &str) -> String {
    return MD5Utils::md5_encode(
        MD5Utils::md5_encode(password + salt) + salt,
    );
}

// fun jsonEncode(value: Any, pretty: Boolean = false): String {
pub fn json_encode(value: Any, pretty: bool) -> String {
    if pretty {
        return pretty_gson().to_json(value);
    }
    return gson().to_json(value);
}

// fun listFilesRecursively(dir: File): List<File> {
pub fn list_files_recursively(dir: &File) -> Vec<File> {
    let mut result: Vec<File> = Vec::new();
    if !dir.exists() {
        return result;
    }
    if dir.is_file() {
        result.push(dir.clone());
        return result;
    }
    let files = dir.list_files();
    for file in files {
        result.push(file.clone());
        if file.is_directory() {
            result.extend(list_files_recursively(&file));
        }
    }
    return result;
}

// fun String.toDir(absolute: Boolean = false): String {
pub fn to_dir(this: &str, absolute: bool) -> String {
    let mut path = this.to_string();
    if path.ends_with("/") {
        path = path.substring(0, path.len() - 1);
    }
    if absolute && !path.starts_with("/") {
        path = "/".to_string() + &path;
    }
    return path;
}

// inline fun <reified T> arrayType(clazz: Class<T>): Class<Array<T>> {
//     @Suppress("UNCHECKED_CAST")
//     return java.lang.reflect.Array.newInstance(clazz, 0)::class.java as Class<Array<T>>
// }
pub fn array_type<T>(clazz: Class<T>) -> Class<Vec<T>> {
    return java_reflect_array_new_instance(clazz, 0).class().as_type();
}

// fun deepListFiles(dir: File, allowExtensions: Array<String>?): List<File> {
pub fn deep_list_files(dir: &File, allow_extensions: Option<Vec<String>>) -> Vec<File> {
    let mut result: Vec<File> = Vec::new();
    let files = dir.list_files()?;
    for file in files {
        if file.is_directory() {
            result.extend(deep_list_files(&file, allow_extensions.clone()));
            continue;
        }
        let extension = FileUtils::get_extension(&file.name);
        if allow_extensions.is_none() || allow_extensions.clone().unwrap().content_deep_to_string().contains(&extension) {
            result.push(file.clone());
        }
    }
    return result;
}

// fun getTraceId(): String {
pub fn get_trace_id() -> String {
    return Uuid::new_v4().to_string().sub_sequence(0, 8);
}

// fun validateEmail(email: String): Boolean {
pub fn validate_email(email: &str) -> bool {
    let regex = Regex::new(r"^[A-Za-z0-9._%+-]+@(163|126|qq|yahoo|sina|sohu|yeah|139|189|21cn|outlook|gmail|icloud).com$");
    return regex.matches(email);
}

// fun encodeBase64(text: String): String {
pub fn encode_base64(text: &str) -> String {
    return JavaBase64::get_encoder().encode_to_string(text.as_bytes());
}

// fun getStorageFile(vararg name: String, ext: String = ".json"): File {
pub fn get_storage_file(name: &[String], ext: &str) -> File {
    let storage_path = get_storage_path();
    let storage_dir = File::new(&storage_path);
    if !storage_dir.exists() {
        storage_dir.mkdirs();
    }

    let filename = name.last().clone();
    let relative_path = get_relative_path(&[&name[0..name.len() - 1], &(filename + ext)]);
    return File::new(&storage_path).resolve(&relative_path);
}

// private fun storageLock(file: File): ReadWriteLock {
//     synchronized(storageLocks) {
//         return storageLocks.get(file.absolutePath)
//             ?: ReentrantReadWriteLock().also { storageLocks.put(file.absolutePath, it) }
//     }
// }
pub fn storage_lock(file: &File) -> ReadWriteLock {
    let storage_locks = STORAGE_LOCKS.get_or_init(|| LRUCache::new(MAX_CACHE_SIZE));
    let mutex = std::sync::Mutex::new(());
    let _guard = mutex.lock();
    return storage_locks.get(&file.absolute_path)
        .unwrap_or_else(|| {
            let lock = ReentrantReadWriteLock::new();
            storage_locks.put(file.absolute_path.clone(), lock.clone());
            lock
        });
}

// fun getMongoFileStorage(): MongoCollection<MongoFile>? {
pub fn get_mongo_file_storage() -> Option<MongoCollection<MongoFile>> {
    let app_config = SpringContextUtils::get_bean_by_name_and_class::<AppConfig>("appConfig", AppConfig::class)?;
    return MongoManager::file_storage(&app_config.mongo_db_name, "storage");
}

// fun readMongoFile(path: String): String? {
pub fn read_mongo_file(path: &str) -> Option<String> {
    if !MongoManager::is_init() {
        return None;
    }
    logger.info(format!("Get mongoFile {}", path));
    let collection = get_mongo_file_storage()?;
    let doc = collection.find(Filters::eq("path", path)).first();
    return doc?.content;
}

// fun saveMongoFile(path: String, content: String): Boolean {
pub fn save_mongo_file(path: &str, content: &str) -> bool {
    if !MongoManager::is_init() {
        return false;
    }
    logger.info(format!("Save mongoFile {}", path));
    let collection = get_mongo_file_storage()?;
    let filter = Filters::eq("path", path);
    let existing = collection.find(filter.clone()).first();
    if existing.is_some() {
        let mut existing = existing.unwrap();
        existing.content = content.to_string();
        existing.updated_at = System::current_time_millis();
        let result = collection.replace_one(
            filter,
            existing,
            ReplaceOptions::new().upsert(true),
        );
        return result.modified_count > 0;
    }
    return try {
        collection.insert_one(MongoFile::new(path = path, content = content));
        true
    } catch (e: Exception) {
        logger.info(format!("Save mongoFile {} failed", path));
        e.printStackTrace();
        false
    };
}

// fun countOccurrences(text: String, sub: String): Int {
pub fn count_occurrences(text: &str, sub: &str) -> i32 {
    if sub.is_empty() { return 0; }
    let mut count = 0;
    let mut index = 0;
    loop {
        index = text.index_of(sub, index);
        if index == -1 { break; }
        count += 1;
        index += sub.len();
    }
    return count;
}

// fun parseJsonStringList(
//     file: File,
//     fields: Set<String>? = null,
//     exclude: Set<String>? = null,
//     startIndex: Int = 0,
//     endIndex: Int = Int.MAX_VALUE,
//     checkNotEmpty: Set<String>? = null,
//     filter: ((ObjectNode) -> Boolean)? = null
// ): JsonArray? {
pub fn parse_json_string_list(
    file: &File,
    fields: Option<std::collections::HashSet<String>>,
    exclude: Option<std::collections::HashSet<String>>,
    start_index: i32,
    end_index: i32,
    check_not_empty: Option<std::collections::HashSet<String>>,
    filter: Option<&dyn Fn(ObjectNode) -> bool>,
) -> Option<JsonArray> {
    if !file.exists() {
        return None;
    }
    return try {
        let object_mapper = ObjectMapper::new();
        let result_list = JsonArray::new();
        let mut current_index = -1;
        object_mapper.factory().create_parser(file).use(|parser| {
            if parser.next_token() == JsonToken::START_ARRAY {
                while parser.next_token() != JsonToken::END_ARRAY {
                    if parser.current_token() != JsonToken::START_OBJECT {
                        continue;
                    }
                    if fields.is_empty() {
                        if filter.is_none() {
                            current_index += 1;
                            if current_index < start_index {
                                parser.skip_children();
                                continue;
                            }
                            if current_index > end_index {
                                break;
                            }
                            let object_node = parser.read_value_as_tree::<ObjectNode>();
                            exclude?.for_each(|it| { object_node.remove(&it); });
                            result_list.add(object_node.to_string());
                            continue;
                        }
                        let object_node = parser.read_value_as_tree::<ObjectNode>();
                        if filter(object_node.clone()) {
                            current_index += 1;
                        }
                        if current_index < start_index {
                            continue;
                        }
                        if current_index > end_index {
                            break;
                        }
                        result_list.add(object_node.to_string());
                        continue;
                    }

                    current_index += 1;
                    if current_index < start_index {
                        parser.skip_children();
                        continue;
                    }
                    if current_index > end_index {
                        break;
                    }
                    let item = JsonObject::new();
                    while parser.next_token() != JsonToken::END_OBJECT {
                        let field_name = parser.current_name();
                        parser.next_token();
                        if fields.contains(&field_name) {
                            item.put(&field_name, parser.value_as_string());
                        } else if check_not_empty?.contains(&field_name) {
                            item.put(&field_name, !parser.value_as_string().is_empty());
                        } else {
                            parser.skip_children();
                        }
                    }
                    result_list.add(item.to_string());
                }
            }
        });
        Some(result_list)
    } catch (e: Exception) {
        logger.error(format!("解析文件内容出错: {} 文件: \n{}", e, file));
        panic!(e);
    };
}
