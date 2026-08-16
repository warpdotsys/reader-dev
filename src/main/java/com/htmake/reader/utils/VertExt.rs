use crate::prelude::*;
// 显式导入消解跨模块 glob 导入歧义（优先于 prelude 的 glob 导入）
use crate::stubs::{
    Any, File, FileUtils, IntTypeAdapter, JsonArray, JsonObject, LongTypeAdapter,
    MapDeserializerDoubleAsIntFix,
};
use crate::com_htmake_reader_config_appconfig::AppConfig;
use crate::com_htmake_reader_entity_mongofile::MongoFile;
use crate::com_htmake_reader_utils_mongomanager::MongoManager;
use crate::com_htmake_reader_utils_springcontextutils::SpringContextUtils;
use crate::io_legado_app_data_entities_book::Book;
use crate::io_legado_app_utils_md5utils::MD5Utils;
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
            .register_type_adapter(TypeToken::<std::collections::HashMap<String, Any>>::new().get_type(), MapDeserializerDoubleAsIntFix::new())
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
            .register_type_adapter(TypeToken::<std::collections::HashMap<String, Any>>::new().get_type(), MapDeserializerDoubleAsIntFix::new())
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
pub static STORAGE_FINAL_PATH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
pub static WORK_DIR_PATH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
pub static WORK_DIR_INIT: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
pub const MAX_CACHE_SIZE: usize = 1000;
// fix: LRUCache 内部使用 Rc<RefCell>（!Sync），无法放入 static，改用 thread_local
thread_local! {
    static STORAGE_LOCKS: std::cell::RefCell<LRUCache<String, ReadWriteLock>> =
        std::cell::RefCell::new(LRUCache::new(MAX_CACHE_SIZE));
}

// fun getWorkDir(subPath: String = ""): String {
pub fn get_work_dir(sub_path: &str) -> String {
    if !*WORK_DIR_INIT.get_or_init(|| false) && WORK_DIR_PATH.get().map(|p| p.is_empty()).unwrap_or(true) {
        // fix: 直接读 READER_APP_WORKDIR 环境变量（原依赖 SpringContextUtils——APPLICATION_CONTEXT 未初始化
        //      恒 None，容器内回落 cwd 相对路径 → 读不到 /storage 挂载，登录/数据读取全失败）
        if let Ok(wd) = std::env::var("READER_APP_WORKDIR") {
            if !wd.is_empty() && wd != "." {
                let work_dir_file = File::new(&wd);
                if work_dir_file.exists() && !work_dir_file.is_directory() {
                    logger().error(format!("reader.app.workDir={} is not a directory", wd));
                } else {
                    if !work_dir_file.exists() {
                        logger().info(format!("reader.app.workDir={} not exists, creating", wd));
                        work_dir_file.mkdirs();
                    }
                    let _ = WORK_DIR_PATH.set(work_dir_file.absolute_path);
                }
            }
        }
        let app_config = SpringContextUtils::get_bean_by_name_and_class("appConfig", AppConfig::class);
        if let Some(cfg) = app_config {
            if !cfg.work_dir.is_empty() && cfg.work_dir != "." {
                let work_dir_file = File::new(&cfg.work_dir);
                if work_dir_file.exists() && !work_dir_file.is_directory() {
                    logger().error(format!("reader.app.workDir={} is not a directory", cfg.work_dir));
                } else {
                    if !work_dir_file.exists() {
                        logger().info(format!("reader.app.workDir={} not exists, creating", cfg.work_dir));
                        work_dir_file.mkdirs();
                    }
                    // fix: 旧工具链 OnceLock::get_or_init 返回 &T，初始化赋值改用 set
                    let _ = WORK_DIR_PATH.set(work_dir_file.absolute_path);
                }
            }
        }
        if WORK_DIR_PATH.get().map(|p| p.is_empty()).unwrap_or(true) {
            let os_name = System::get_property("os.name");
            let current_dir = System::get_property("user.dir");
            logger().info(format!("osName: {} currentDir: {}", os_name, current_dir));
            if os_name.starts_with("Mac OS") && !current_dir.starts_with("/Users/") {
                // fix: 旧工具链 OnceLock::get_or_init 返回 &T，初始化赋值改用 set
                let _ = WORK_DIR_PATH.set(Paths::get(System::get_property("user.home"), ".reader").to_string());
            } else {
                // fix: 旧工具链 OnceLock::get_or_init 返回 &T，初始化赋值改用 set
                let _ = WORK_DIR_PATH.set(current_dir);
            }
        }
        logger().info(format!("Using workdir: {}", WORK_DIR_PATH.get().unwrap()));
        // fix: 旧工具链 OnceLock::get_or_init 返回 &T，初始化赋值改用 set
        let _ = WORK_DIR_INIT.set(true);
    }
    let path = Paths::get(WORK_DIR_PATH.get().unwrap(), sub_path);

    return path.to_string();
}

// fun getWorkDir(vararg subDirFiles: String): String {
pub fn get_work_dir_multi(sub_dir_files: &[&str]) -> String {
    return get_work_dir(&get_relative_path(sub_dir_files));
}

// fun getRelativePath(vararg subDirFiles: String): String {
pub fn get_relative_path<T: AsRef<str>>(sub_dir_files: &[T]) -> String {
    let mut path = StringBuilder::new();
    for it in sub_dir_files {
        if !it.as_ref().is_empty() {
            path.append(File::SEPARATOR.to_string() + it.as_ref());
        }
    }
    // fix: Kotlin `path.toString().let { }` → 局部变量
    let it = path.to_string();
    return if it.starts_with("/") {
        it.substring(1)
    } else {
        it
    };
}

// fun getStoragePath(): String {
pub fn get_storage_path() -> String {
    if !STORAGE_FINAL_PATH.get().map(|p| p.is_empty()).unwrap_or(true) {
        return STORAGE_FINAL_PATH.get().unwrap().clone();
    }
    let mut storage_path = String::new();
    // fix: 直接读 READER_APP_WORKDIR（原依赖 app_config——未初始化时回落 cwd 相对路径，容器内读不到 /storage）
    if let Ok(wd) = std::env::var("READER_APP_WORKDIR") {
        if !wd.is_empty() && wd != "." {
            storage_path = format!("{}/storage", wd.trim_end_matches('/'));
        }
    }
    if storage_path.is_empty() {
        let app_config = SpringContextUtils::get_bean_by_name_and_class("appConfig", AppConfig::class);
        if app_config.is_some() {
            storage_path = get_work_dir("storage");
            // fix: 旧工具链 OnceLock::get_or_init 返回 &T，初始化赋值改用 set
            let _ = STORAGE_FINAL_PATH.set(storage_path.clone());
        } else {
            storage_path = File::new("storage").path();
        }
    } else {
        let _ = STORAGE_FINAL_PATH.set(storage_path.clone());
    }
    logger().info(format!("Using storagePath: {}", storage_path));
    return storage_path;
}

// fun saveStorage(vararg name: String, value: Any, pretty: Boolean = false, ext: String = ".json") {
pub fn save_storage(name: &[String], value: Any, pretty: bool, ext: &str) {
    // fix: Kotlin `value is String` 等智能转换 → match
    let to_json: String = match &value {
        Any::Str(s) => s.clone(),
        Any::JsonObject(o) => o.to_string(),
        Any::JsonArray(a) => a.to_string(),
        _ => {
            if pretty {
                pretty_gson().to_json(value.clone())
            } else {
                gson().to_json(value.clone())
            }
        }
    };

    let storage_path = get_storage_path();
    let storage_dir = File::new(&storage_path);
    if !storage_dir.exists() {
        storage_dir.mkdirs();
    }

    let filename = name.last().unwrap().clone();
    // fix: Kotlin vararg 展开（copyOfRange + "$filename$ext"）→ Vec<String>
    let mut path_parts: Vec<String> = name[0..name.len() - 1].to_vec();
    path_parts.push(filename.clone() + ext);
    let path = get_relative_path(&path_parts);
    let file = File::new(&storage_path).resolve(&path);
    logger().info(format!("Save file to storage name: {:?} path: {}", name, file.absolute_file()));

    if !file.parent_file.as_ref().unwrap().exists() {
        file.parent_file.as_ref().unwrap().mkdirs();
    }

    let lock = storage_lock(&file);
    let mut acquired = false;
    // fix: try/catch/finally → 闭包 + if-let（finally 在闭包后执行，保持语义）
    let try_result: Result<(), StubError> = (|| {
        acquired = lock.write_lock().try_lock(10, TimeUnit::SECONDS);
        if !acquired {
            return Err(StubError::new(format!("保存文件超时: {}", file.absolute_path)));
        }

        let base_name = file.name_without_extension();
        let temp = Files::create_temp_file(file.parent_file.as_ref().unwrap().to_path().to_absolute_path(), base_name.as_str(), ".temp");
        Files::write(&temp, to_json.as_bytes());

        let file_path = file.to_path();
        let backup_path = file.parent_file.as_ref().unwrap().to_path().resolve(&(base_name.to_string() + ".backup.json")).to_absolute_path();
        if Files::exists(&file_path) {
            Files::move_path(&file_path, &backup_path, StandardCopyOption::ATOMIC_MOVE);
        }
        Files::move_path(&temp, &file_path, StandardCopyOption::ATOMIC_MOVE);
        Files::delete_if_exists(&temp);

        if base_name.len() >= 32 {
            Files::delete_if_exists(&backup_path);
        }
        if base_name == "users" {
            let mut verify_parts: Vec<String> = name[0..name.len() - 1].to_vec();
            verify_parts.push(".".to_string() + &base_name + ".key");
            let verify_file = File::new(&storage_path).resolve(&get_relative_path(&verify_parts));
            if !verify_file.exists() {
                verify_file.create_new_file();
            }
            let verification = MD5Utils::md5Encode(Some(format!("userCount={}", count_occurrences(&to_json, "username")).as_str())).take_last(16);
            verify_file.write_text(&verification);
        }
        save_mongo_file(&path, &to_json);
        Ok(())
    })();
    if acquired {
        lock.write_lock().unlock();
    }
    if let Err(e) = try_result {
        logger().error(format!("保存文件失败: {}", e));
        panic!("保存文件失败: {}", file.absolute_path);
    }
}

// fun getStorage(vararg name: String, ext: String = ".json"): String?  {
pub fn get_storage(name: &[String], ext: &str) -> Option<String> {
    let storage_path = get_storage_path();
    let filename = name.last().unwrap().clone();
    // fix: Kotlin vararg 展开（copyOfRange + "$filename$ext"）→ Vec<String>
    let mut path_parts: Vec<String> = name[0..name.len() - 1].to_vec();
    path_parts.push(filename.clone() + ext);
    let path = get_relative_path(&path_parts);
    let file = File::new(&storage_path).resolve(&path);
    logger().info(format!("Read file from storage name: {:?} path: {}", name, file.absolute_file()));
    if !file.exists() {
        // fix: Kotlin `isNullOrEmpty()` + 智能转换 → if let
        if let Some(content) = read_mongo_file(&path) {
            if !content.is_empty() {
                if !file.parent_file.as_ref().unwrap().exists() {
                    file.parent_file.as_ref().unwrap().mkdirs();
                }
                file.create_new_file();
                file.write_text(&content);
                return Some(content);
            }
        }
        return None;
    }

    let lock = storage_lock(&file);
    let mut acquired = false;
    // fix: try/catch/finally → 闭包 + match（finally 在闭包后执行，保持语义）
    let try_result: Result<String, StubError> = (|| {
        acquired = lock.read_lock().try_lock(10, TimeUnit::SECONDS);
        if !acquired {
            return Err(StubError::new(format!("读取文件超时: {}", file.absolute_path)));
        }
        let mut content = file.read_text();
        if content.is_empty() {
            // fix: Kotlin `isNullOrEmpty()` + 智能转换 → if let
            if let Some(mongo_content) = read_mongo_file(&path) {
                if !mongo_content.is_empty() {
                    file.write_text(&mongo_content);
                    content = mongo_content;
                }
            }
        }
        if filename == "users" {
            let mut verify_parts: Vec<String> = name[0..name.len() - 1].to_vec();
            verify_parts.push(".".to_string() + &filename + ".key");
            let verify_file = File::new(&storage_path).resolve(&get_relative_path(&verify_parts));
            if verify_file.exists() {
                let verification = MD5Utils::md5Encode(Some(format!("userCount={}", count_occurrences(&content, "username")).as_str())).take_last(16);
                if verify_file.read_text() != verification {
                    return Err(StubError::new("用户数据被篡改，请联系开发者修复".to_string()));
                }
            }
        }
        Ok(content)
    })();
    if acquired {
        lock.read_lock().unlock();
    }
    return match try_result {
        Ok(content) => Some(content),
        Err(e) => {
            logger().error(format!("读取文件失败: {}", e));
            panic!("读取文件失败: {}", file.absolute_path);
        }
    };
}

// fun asJsonArray(value: Any?): JsonArray? {
pub fn as_json_array(value: Option<Any>) -> Option<JsonArray> {
    // fix: Kotlin `value is JsonArray` 智能转换 → 模式匹配
    if let Some(Any::JsonArray(v)) = &value {
        return Some(v.clone());
    } else if let Some(Any::Str(s)) = &value {
        // fix: try/catch → 闭包 + match
        let try_result: Result<JsonArray, StubError> = (|| { Ok(JsonArray::new_parsed(s)) })();
        return match try_result {
            Ok(arr) => Some(arr),
            Err(e) => {
                logger().error(format!("解析内容出错: {}  内容: \n{:?}", e, value));
                panic!("{}", e);
            }
        };
    }
    return None;
}

// fun asJsonObject(value: Any?): JsonObject? {
pub fn as_json_object(value: Option<Any>) -> Option<JsonObject> {
    // fix: Kotlin `value is JsonObject` 智能转换 → 模式匹配
    if let Some(Any::JsonObject(v)) = &value {
        return Some(v.clone());
    } else if let Some(Any::Str(s)) = &value {
        // fix: try/catch → 闭包 + match
        let try_result: Result<JsonObject, StubError> = (|| { Ok(JsonObject::new_parsed(s)) })();
        return match try_result {
            Ok(obj) => Some(obj),
            Err(e) => {
                logger().error(format!("解析内容出错: {}  内容: \n{:?}", e, value));
                panic!("{}", e);
            }
        };
    }
    return None;
}

//convert a data class to a map
// fun <T> T.serializeToMap(): Map<String, Any> {
pub fn serialize_to_map<T>(this: T) -> std::collections::HashMap<String, Any>
where
    T: std::any::Any + serde::Serialize + 'static,
{
    return convert(this);
}

//convert string to a map
// fun <T> T.toMap(): Map<String, Any> {
pub fn to_map<T>(this: T) -> std::collections::HashMap<String, Any>
where
    T: std::any::Any + serde::Serialize + 'static,
{
    return convert(this);
}

//convert a map to a data class
// inline fun <reified T> Map<String, Any>.toDataClass(): T {
pub fn to_data_class<T>(this: std::collections::HashMap<String, Any>) -> T
where
    T: serde::de::DeserializeOwned,
{
    return convert(this);
}

//convert an object of type I to type O
// inline fun <I, reified O> I.convert(): O {
pub fn convert<I, O>(this: I) -> O
where
    I: std::any::Any + serde::Serialize + 'static,
    O: serde::de::DeserializeOwned,
{
    // fix: Kotlin `this is String` 智能转换 → downcast_ref
    let json = if let Some(s) = (&this as &dyn std::any::Any).downcast_ref::<String>() {
        s.clone()
    } else {
        gson().to_json(&this)
    };
    return gson().from_json(&json, TypeToken::<O>::new().get_type());
}

// @Suppress("UNCHECKED_CAST")
// fun <R> readInstanceProperty(instance: Any, propertyName: String): R {
pub fn read_instance_property<R>(instance: &dyn std::any::Any, property_name: &str) -> R
where
    R: From<Any>,
{
    let property = instance.class().member_properties()
        .into_iter()
        // don't cast here to <Any, R>, it would succeed silently
        .find(|it| it.name == property_name);
    // force a invalid cast exception if incorrect type here
    return property.unwrap().get(instance).into();
}

// @Suppress("UNCHECKED_CAST")
// fun setInstanceProperty(instance: Any, propertyName: String, propertyValue: Any) {
pub fn set_instance_property(instance: &dyn std::any::Any, property_name: &str, property_value: Any) {
    let property = instance.class().member_properties()
        .into_iter()
        .find(|it| it.name == property_name);
    // fix: Kotlin `property is KMutableProperty` 智能转换 → as_mutable()
    if let Some(property) = property {
        if let Some(mp) = property.as_mutable() {
            mp.setter().call(instance, property_value);
        }
    }
}

// fun Book.fillData(newBook: Book, keys: List<String>): Book {
pub fn fill_data(this: Book, new_book: Book, keys: Vec<String>) -> Book {
    // fix: Kotlin `keys.let { }` → for 循环；字段级合并（原反射占位 property.unwrap() 会 panic）
    let mut book = this;
    for key in keys {
        match key.as_str() {
            "name" => {
                if book.name.is_empty() {
                    book.name = new_book.name.clone();
                }
            }
            "author" => {
                if book.author.is_empty() {
                    book.author = new_book.author.clone();
                }
            }
            "coverUrl" => {
                if book.cover_url.as_ref().map_or(true, |s| s.is_empty()) {
                    book.cover_url = new_book.cover_url.clone();
                }
            }
            "tocUrl" => {
                if book.toc_url.is_empty() {
                    book.toc_url = new_book.toc_url.clone();
                }
            }
            "intro" => {
                if book.intro.as_ref().map_or(true, |s| s.is_empty()) {
                    book.intro = new_book.intro.clone();
                }
            }
            "latestChapterTitle" => {
                if book.latest_chapter_title.as_ref().map_or(true, |s| s.is_empty()) {
                    book.latest_chapter_title = new_book.latest_chapter_title.clone();
                }
            }
            "wordCount" => {
                if book.word_count.as_ref().map_or(true, |s| s.is_empty()) {
                    book.word_count = new_book.word_count.clone();
                }
            }
            _ => {}
        }
    }
    return book;
}

// fun getRandomString(length: Int) : String {
pub fn get_random_string(length: i32) -> String {
    let allowed_chars = "ABCDEFGHIJKLMNOPQRSTUVWXTZabcdefghiklmnopqrstuvwxyz0123456789".to_string();
    return (1..=length)
        .map(|_| allowed_chars.random())
        .collect::<String>();
}

// fun genEncryptedPassword(password: String, salt: String): String {
pub fn gen_encrypted_password(password: &str, salt: &str) -> String {
    return MD5Utils::md5Encode(Some((MD5Utils::md5Encode(Some((password.to_string() + salt).as_str())) + salt).as_str()));
}

// fun jsonEncode(value: Any, pretty: Boolean = false): String {
pub fn json_encode(value: Any, pretty: bool) -> String {
    if pretty {
        return pretty_gson().to_json(value.clone());
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
        // fix: Kotlin `substring(0, length - 1)` → substring_range
        path = path.substring_range(0, path.len() - 1);
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
    // fix: Kotlin `listFiles() ?: return result`（list_files 占位返回 Vec，无需判空）
    let files = dir.list_files();
    for file in files {
        if file.is_directory() {
            result.extend(deep_list_files(&file, allow_extensions.clone()));
            continue;
        }
        let extension = FileUtils::get_extension(&file.name);
        // fix: Kotlin `contentDeepToString().contains(extension)` → Debug 格式化包含判断
        if allow_extensions.is_none() || format!("{:?}", allow_extensions.as_ref().unwrap()).contains(&extension) {
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
    let regex = Regex::new(r"^[A-Za-z0-9._%+-]+@(163|126|qq|yahoo|sina|sohu|yeah|139|189|21cn|outlook|gmail|icloud).com$").unwrap();
    // fix: Kotlin `Regex.matches()` → `regex::Regex.is_match()`
    return regex.is_match(email);
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

    let filename = name.last().unwrap().clone();
    // fix: Kotlin vararg 展开（copyOfRange + "$filename$ext"）→ Vec<String>
    let mut path_parts: Vec<String> = name[0..name.len() - 1].to_vec();
    path_parts.push(filename.clone() + ext);
    let relative_path = get_relative_path(&path_parts);
    return File::new(&storage_path).resolve(&relative_path);
}

// private fun storageLock(file: File): ReadWriteLock {
//     synchronized(storageLocks) {
//         return storageLocks.get(file.absolutePath)
//             ?: ReentrantReadWriteLock().also { storageLocks.put(file.absolutePath, it) }
//     }
// }
pub fn storage_lock(file: &File) -> ReadWriteLock {
    let mutex = std::sync::Mutex::new(());
    let _guard = mutex.lock();
    return STORAGE_LOCKS.with(|storage_locks| {
        let mut storage_locks = storage_locks.borrow_mut();
        return storage_locks.get(&file.absolute_path)
            .unwrap_or_else(|| {
                let lock = ReadWriteLock::new();
                storage_locks.put(file.absolute_path.clone(), lock.clone());
                lock
            });
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
    logger().info(format!("Get mongoFile {}", path));
    let collection = get_mongo_file_storage()?;
    let doc = collection.find(Filters::eq("path", path)).first();
    return Some(doc?.content);
}

// fun saveMongoFile(path: String, content: String): Boolean {
pub fn save_mongo_file(path: &str, content: &str) -> bool {
    if !MongoManager::is_init() {
        return false;
    }
    logger().info(format!("Save mongoFile {}", path));
    // fix: Kotlin `?: return false` → let-else
    let Some(collection) = get_mongo_file_storage() else {
        return false;
    };
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
    // fix: try/catch → 闭包 + match；Kotlin 具名参数 MongoFile(path=, content=) → 结构体字面量
    let try_result: Result<(), StubError> = (|| {
        collection.insert_one(MongoFile {
            path: path.to_string(),
            content: content.to_string(),
            created_at: System::current_time_millis(),
            updated_at: System::current_time_millis(),
        });
        Ok(())
    })();
    return match try_result {
        Ok(_) => true,
        Err(e) => {
            logger().info(format!("Save mongoFile {} failed", path));
            e.print_stack_trace();
            false
        }
    };
}

// fun countOccurrences(text: String, sub: String): Int {
pub fn count_occurrences(text: &str, sub: &str) -> i32 {
    if sub.is_empty() { return 0; }
    let mut count = 0;
    let mut index: i32 = 0;
    loop {
        index = text.index_of(sub, index as usize);
        if index == -1 { break; }
        count += 1;
        index += sub.len() as i32;
    }
    return count;
}

// fun parseJsonStringList(
//     file: File,
//     fields: Set<String>? = None,
//     exclude: Set<String>? = None,
//     startIndex: Int = 0,
//     endIndex: Int = Int.MAX_VALUE,
//     checkNotEmpty: Set<String>? = None,
//     filter: ((ObjectNode) -> Boolean)? = None
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
    // fix: try/catch → 闭包 + match；`objectMapper.factory().createParser(file).use { }` → 直接展开
    let try_result: Result<Option<JsonArray>, StubError> = (|| {
        let object_mapper = ObjectMapper::new();
        let mut result_list = JsonArray::new();
        let mut current_index = -1;
        let parser = object_mapper.factory().create_parser(file);
        if parser.next_token() == JsonToken::START_ARRAY {
            while parser.next_token() != JsonToken::END_ARRAY {
                if parser.current_token() != JsonToken::START_OBJECT {
                    continue;
                }
                // fix: Kotlin `fields.isNullOrEmpty()` → is_none || is_empty
                if fields.is_none() || fields.as_ref().unwrap().is_empty() {
                    if filter.is_none() {
                        current_index += 1;
                        if current_index < start_index {
                            parser.skip_children();
                            continue;
                        }
                        if current_index > end_index {
                            break;
                        }
                        let mut object_node = parser.read_value_as_object_node();
                        // fix: Kotlin `exclude?.forEach {}` → if let + for
                        if let Some(exclude_list) = &exclude {
                            for it in exclude_list {
                                object_node.remove(it);
                            }
                        }
                        result_list.add(object_node.to_string());
                        continue;
                    }
                    let object_node = parser.read_value_as_object_node();
                    if filter.unwrap()(object_node.clone()) {
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
            let mut item = crate::stubs::JsonObject::new();
            let object_node = parser.read_value_as_object_node();
            if let Some(f) = fields.as_ref() {
                for (k, v) in &object_node.0 {
                    if f.contains(k) {
                        item.put(k, v);
                    }
                }
            }
            if let Some(ce) = check_not_empty.as_ref() {
                for k in ce {
                    // fix: Kotlin 仅对象存在该字段时 put（缺失 → 键不存在；原无条件 put——缺失产生显式 false）
                    if let Some(v) = object_node.0.get(k) {
                        item.put(k, !v.is_empty());
                    }
                }
            }
            result_list.add(item.to_string());
            }
        }
        Ok(Some(result_list))
    })();
    return match try_result {
        Ok(result) => result,
        Err(e) => {
            logger().error(format!("解析文件内容出错: {} 文件: \n{}", e, file));
            panic!("{}", e);
        }
    };
}
