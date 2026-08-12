use crate::prelude::*;
// package com.htmake.reader.api.controller

// fix: 显式导入消除 stubs / 其他模块 glob 重导出的 File 歧义（显式导入优先于 glob）
use crate::stubs::File;

// fix: stubs / VertExt 同名 get_work_dir 的 glob 歧义 → 本地包装，
//      按 Kotlin vararg 参数（getWorkDir(vararg subDirFiles)）转发
fn get_work_dir(base: &str, sub_dir_files: Vec<String>) -> String {
    crate::com_htmake_reader_utils_vertext::get_work_dir_multi(
        &std::iter::once(base)
            .chain(sub_dir_files.iter().map(|s| s.as_str()))
            .collect::<Vec<&str>>(),
    )
}

// private val logger = KotlinLogging.logger {}

// class FileController(coroutineContext: CoroutineContext) : BaseController(coroutineContext) {
pub struct FileController {
    base: BaseController,
}

impl FileController {
    // private fun resolveSecurePath(baseDir: File, relativePath: String): File? {
    //     val basePath = baseDir.toPath().toAbsolutePath().normalize()
    //     val resolved = basePath.resolve(relativePath.removePrefix("/").removePrefix("\\")).normalize()
    //     return resolved.takeIf { it.startsWith(basePath) }?.toFile()
    // }
    fn resolve_secure_path(base_dir: &File, relative_path: &String) -> Option<File> {
        let base_path = base_dir.to_path().to_absolute_path().normalize();
        let resolved = base_path.resolve(relative_path.trim_start_matches("/").trim_start_matches("\\")).normalize();
        if resolved.starts_with(&base_path) {
            return Some(resolved.to_file());
        }
        return None;
    }

    // private fun requestedHome(context: RoutingContext): String = when {
    //     context.request().method() == HttpMethod.POST && context.fileUploads().isNotEmpty() ->
    //         context.request().getParam("home") ?: ""
    //     context.request().method() == HttpMethod.POST -> context.bodyAsJson?.getString("home", "") ?: ""
    //     else -> context.queryParam("home").firstOrNull() ?: ""
    // }
    fn requested_home(&self, context: &RoutingContext) -> String {
        if context.request().method() == HttpMethod::POST && !context.file_uploads().unwrap_or_default().is_empty() {
            return context.request().get_param("home").unwrap_or(String::from(""));
        }
        if context.request().method() == HttpMethod::POST {
            return context.body_as_json().map(|j| j.get_string_default("home", "")).unwrap_or(String::from(""));
        }
        return context.query_param("home").unwrap_or(String::from(""));
    }

    // private fun requestPath(context: RoutingContext, key: String = "path"): String = if (context.request().method() == HttpMethod.POST) {
    //     context.bodyAsJson?.getString(key) ?: ""
    // } else {
    //     context.queryParam(key).firstOrNull() ?: ""
    // }
    fn request_path(&self, context: &RoutingContext, key: String) -> String {
        if context.request().method() == HttpMethod::POST {
            return context.body_as_json().map(|j| j.get_string(&key)).unwrap_or(String::from(""));
        }
        return context.query_param(&key).unwrap_or(String::from(""));
    }

    // private fun getFileHome(context: RoutingContext): File? = context.get<File>("__FILE_HOME__")
    fn get_file_home(&self, context: &RoutingContext) -> Option<File> {
        return context.get_file("__FILE_HOME__");
    }

    // suspend fun checkAccess(context: RoutingContext, isSave: Boolean = false, isDelete: Boolean = false): ReturnData? {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     context.put("__FILE_HOME__", None)
    //     val directory = when (requestedHome(context)) {
    //         "__WEBDAV__" -> {
    //             if (appConfig.secure) {
    //                 val userInfo = context.get<User>("userInfo")
    //                     ?: return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //                 if (!userInfo.enable_webdav) return returnData.setErrorMsg("未开启webdav功能")
    //             }
    //             File(getUserWebdavHome(context))
    //         }
    //         "__LOCAL_STORE__" -> {
    //             if (appConfig.secure) {
    //                 val userInfo = context.get<User>("userInfo")
    //                     ?: return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //                 if (!userInfo.enable_local_store) return returnData.setErrorMsg("未开启本地书仓功能")
    //             }
    //             if ((isSave || isDelete) && !checkManagerAuth(context)) {
    //                 return returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码")
    //             }
    //             File(getWorkDir("storage", "localStore"))
    //         }
    //         "__HOME__" -> File(getWorkDir("storage", "data", getUserNameSpace(context)))
    //         "__STORAGE__" -> {
    //             if (!checkManagerAuth(context)) return returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码")
    //             File(getWorkDir("storage"))
    //         }
    //         else -> {
    //             // 空 home 回退用户数据目录（JAR 继承 bug：home= 空值误报"非法访问"，
    //             // 兼容旧客户端/手动构造 URL 的 file/list 等请求）
    //             if (requestedHome(context).isEmpty()) {
    //                 File(getWorkDir("storage", "data", getUserNameSpace(context)))
    //             } else {
    //                 return returnData.setErrorMsg("非法访问")
    //             }
    //         }
    //     }
    //     directory.mkdirs()
    //     context.put("__FILE_HOME__", directory)
    //     logger.info { "context.__FILE_HOME__ $directory" }
    //     return None
    // }
    pub fn check_access(&self, context: &RoutingContext, is_save: bool, is_delete: bool) -> Option<ReturnData> {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
            return Some(return_data);
        }
        context.put("__FILE_HOME__", None::<File>);
        let requested_home = self.requested_home(context);
        // fix: BaseController.app_config 为私有字段 → 经 Spring bean 读取
        let secure = SpringContextUtils::get_bean_by_name_and_class::<AppConfig>("appConfig", AppConfig::class)
            .map(|c| c.secure)
            .unwrap_or(false);
        let directory = if requested_home == "__WEBDAV__" {
            if secure {
                let user_info = context.get_user::<User>("userInfo");
                let user_info = match user_info {
                    Some(u) => u,
                    None => {
                        return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
                        return Some(return_data);
                    }
                };
                if !user_info.enable_webdav {
                    return_data.set_error_msg(String::from("未开启webdav功能"));
                    return Some(return_data);
                }
            }
            File::new(&self.base.get_user_webdav_home(context as &dyn std::any::Any))
        } else if requested_home == "__LOCAL_STORE__" {
            if secure {
                let user_info = context.get_user::<User>("userInfo");
                let user_info = match user_info {
                    Some(u) => u,
                    None => {
                        return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
                        return Some(return_data);
                    }
                };
                if !user_info.enable_local_store {
                    return_data.set_error_msg(String::from("未开启本地书仓功能"));
                    return Some(return_data);
                }
            }
            if (is_save || is_delete) && !self.base.check_manager_auth(context) {
                return_data.set_data(Box::new(String::from("NEED_SECURE_KEY")), String::from("请输入管理密码"));
                return Some(return_data);
            }
            File::new(&get_work_dir("storage", vec![String::from("localStore")]))
        } else if requested_home == "__HOME__" {
            File::new(&get_work_dir("storage", vec![String::from("data"), self.base.get_user_name_space(context)]))
        } else if requested_home == "__STORAGE__" {
            if !self.base.check_manager_auth(context) {
                return_data.set_data(Box::new(String::from("NEED_SECURE_KEY")), String::from("请输入管理密码"));
                return Some(return_data);
            }
            File::new(&get_work_dir("storage", vec![]))
        } else {
            // 空 home 回退用户数据目录（JAR 继承 bug：home= 空值误报"非法访问"，
            // 兼容旧客户端/手动构造 URL 的 file/list 等请求）
            if requested_home.is_empty() {
                File::new(&get_work_dir("storage", vec![String::from("data"), self.base.get_user_name_space(context)]))
            } else {
                return_data.set_error_msg(String::from("非法访问"));
                return Some(return_data);
            }
        };
        directory.mkdirs();
        context.put("__FILE_HOME__", Some(directory.clone()));
        logger().info(format!("context.__FILE_HOME__ {}", directory.to_string()));
        return None;
    }

    // suspend fun list(context: RoutingContext): ReturnData {
    //     checkAccess(context)?.let { return it }
    //     val returnData = ReturnData()
    //     val baseDir = getFileHome(context) ?: return returnData.setErrorMsg("参数错误")
    //     val path = requestPath(context).ifEmpty { "/" }
    //     val file = resolveSecurePath(baseDir, path) ?: return returnData.setErrorMsg("路径不存在")
    //     logger.info { "file: $path $file" }
    //     if (!file.exists()) {
    //         if (path != "/") return returnData.setErrorMsg("路径不存在")
    //         file.mkdirs()
    //     }
    //     if (!file.isDirectory) return returnData.setErrorMsg("路径不是目录")
    //     val files = file.listFiles() ?: emptyArray()
    //     val fileList = files.filterNot { it.name.startsWith(".") }.map { item ->
    //         mapOf(
    //             "name" to item.name,
    //             "size" to item.length(),
    //             "path" to "/" + item.relativeTo(baseDir).path.replace(File.separatorChar, '/'),
    //             "lastModified" to item.lastModified(),
    //             "isDirectory" to item.isDirectory
    //         )
    //     }
    //     return returnData.setData(fileList)
    // }
    pub fn list(&self, context: &RoutingContext) -> ReturnData {
        if let Some(result) = self.check_access(context, false, false) {
            return result;
        }
        let mut return_data = ReturnData::new();
        let base_dir = match self.get_file_home(context) {
            Some(v) => v,
            None => {
                return_data.set_error_msg(String::from("参数错误"));
                return return_data;
            }
        };
        let path = if self.request_path(context, String::from("path")).is_empty() { String::from("/") } else { self.request_path(context, String::from("path")) };
        let file = match Self::resolve_secure_path(&base_dir, &path) {
            Some(v) => v,
            None => {
                return_data.set_error_msg(String::from("路径不存在"));
                return return_data;
            }
        };
        logger().info(format!("file: {} {}", path, file.to_string()));
        if !file.exists() {
            if path != "/" {
                return_data.set_error_msg(String::from("路径不存在"));
                return return_data;
            }
            file.mkdirs();
        }
        if !file.is_directory() {
            return_data.set_error_msg(String::from("路径不是目录"));
            return return_data;
        }
        let files = file.list_files();
        let file_list: Vec<std::collections::HashMap<String, Box<dyn std::any::Any>>> = files.iter().filter(|item| !item.name().starts_with(".")).map(|item| {
            let mut m: std::collections::HashMap<String, Box<dyn std::any::Any>> = std::collections::HashMap::new();
            m.insert(String::from("name"), Box::new(item.name()));
            m.insert(String::from("size"), Box::new(item.length()));
            m.insert(String::from("path"), Box::new(format!("/{}", item.relative_to(&base_dir).path().replace(std::path::MAIN_SEPARATOR, "/"))));
            m.insert(String::from("lastModified"), Box::new(item.last_modified()));
            m.insert(String::from("isDirectory"), Box::new(item.is_directory()));
            m
        }).collect();
        return_data.set_data(Box::new(file_list), String::from(""));
        return return_data;
    }

    // suspend fun upload(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (context.fileUploads().isEmpty()) return returnData.setErrorMsg("请上传文件")
    //     checkAccess(context, isSave = true)?.let { return it }
    //     val baseDir = getFileHome(context) ?: return returnData.setErrorMsg("参数错误")
    //     val path = (context.request().getParam("path") ?: "").ifEmpty { "/" }
    //     val targetDir = resolveSecurePath(baseDir, path) ?: return returnData.setErrorMsg("路径不存在")
    //     val fileList = ArrayList<Map<String, Any>>()
    //     context.fileUploads().forEach { upload ->
    //         val source = File(upload.uploadedFileName())
    //         if (!source.exists()) return@forEach
    //         val destination = resolveSecurePath(targetDir, File(upload.fileName()).name) ?: return@forEach
    //         destination.parentFile.mkdirs()
    //         if (destination.exists()) destination.delete()
    //         if (source.copyTo(destination, overwrite = false).exists()) {
    //             fileList += mapOf(
    //                 "name" to destination.name,
    //                 "size" to destination.length(),
    //                 "path" to "/" + destination.relativeTo(baseDir).path.replace(File.separatorChar, '/'),
    //                 "lastModified" to destination.lastModified(),
    //                 "isDirectory" to destination.isDirectory
    //             )
    //         }
    //         source.deleteRecursively()
    //     }
    //     return returnData.setData(fileList)
    // }
    pub fn upload(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if context.file_uploads().unwrap_or_default().is_empty() {
            return_data.set_error_msg(String::from("请上传文件"));
            return return_data;
        }
        if let Some(result) = self.check_access(context, true, false) {
            return result;
        }
        let base_dir = match self.get_file_home(context) {
            Some(v) => v,
            None => {
                return_data.set_error_msg(String::from("参数错误"));
                return return_data;
            }
        };
        let path = if context.request().get_param("path").unwrap_or(String::from("")).is_empty() { String::from("/") } else { context.request().get_param("path").unwrap() };
        let target_dir = match Self::resolve_secure_path(&base_dir, &path) {
            Some(v) => v,
            None => {
                return_data.set_error_msg(String::from("路径不存在"));
                return return_data;
            }
        };
        let mut file_list: Vec<std::collections::HashMap<String, Box<dyn std::any::Any>>> = Vec::new();
        for upload in context.file_uploads().unwrap_or_default() {
            let source = File::new(&upload.uploaded_file_name());
            if !source.exists() {
                continue;
            }
            let destination = match Self::resolve_secure_path(&target_dir, &File::new(&upload.file_name()).name()) {
                Some(v) => v,
                None => continue,
            };
            if let Some(parent) = destination.parent_file() {
                parent.mkdirs();
            }
            if destination.exists() {
                destination.delete();
            }
            if source.copy_to(&destination, false).exists() {
                let mut m: std::collections::HashMap<String, Box<dyn std::any::Any>> = std::collections::HashMap::new();
                m.insert(String::from("name"), Box::new(destination.name()));
                m.insert(String::from("size"), Box::new(destination.length()));
                m.insert(String::from("path"), Box::new(format!("/{}", destination.relative_to(&base_dir).path().replace(std::path::MAIN_SEPARATOR, "/"))));
                m.insert(String::from("lastModified"), Box::new(destination.last_modified()));
                m.insert(String::from("isDirectory"), Box::new(destination.is_directory()));
                file_list.push(m);
            }
            source.delete_recursively();
        }
        return_data.set_data(Box::new(file_list), String::from(""));
        return return_data;
    }

    // suspend fun download(context: RoutingContext) {
    //     val accessResult = checkAccess(context)
    //     if (accessResult != None) {
    //         context.success(accessResult)
    //         return
    //     }
    //     val returnData = ReturnData()
    //     val path = requestPath(context)
    //     val stream = if (context.request().method() == HttpMethod.POST) {
    //         context.bodyAsJson?.getInteger("stream", 0) ?: 0
    //     } else {
    //         context.queryParam("stream").firstOrNull()?.toIntOrNull() ?: 0
    //     }
    //     if (path.isEmpty()) {
    //         context.success(returnData.setErrorMsg("参数错误"))
    //         return
    //     }
    //     val baseDir = getFileHome(context)
    //     val file = baseDir?.let { resolveSecurePath(it, path) }
    //     if (file == None) {
    //         context.success(returnData.setErrorMsg("参数错误"))
    //         return
    //     }
    //     logger.info { "file: $path $file" }
    //     if (!file.exists()) {
    //         context.success(returnData.setErrorMsg("路径不存在"))
    //         return
    //     }
    //     val response = context.response().putHeader("Cache-Control", "86400")
    //     if (stream <= 0) response.putHeader("Content-Disposition", "attachment; filename=${URLEncoder.encode(file.name, "UTF-8")}")
    //     response.sendFile(file.toString())
    // }
    pub fn download(&self, context: &RoutingContext) {
        let access_result = self.check_access(context, false, false);
        if let Some(access_result) = access_result {
            context.success(&access_result);
            return;
        }
        let mut return_data = ReturnData::new();
        let path = self.request_path(context, String::from("path"));
        let stream = if context.request().method() == HttpMethod::POST {
            context.body_as_json().map(|j| j.get_integer("stream", 0)).unwrap_or(0)
        } else {
            context.query_param("stream").and_then(|s| s.parse::<i32>().ok()).unwrap_or(0)
        };
        if path.is_empty() {
            return_data.set_error_msg(String::from("参数错误"));
            context.success(&return_data);
            return;
        }
        let base_dir = self.get_file_home(context);
        let file = base_dir.as_ref().and_then(|it| Self::resolve_secure_path(it, &path));
        if file.is_none() {
            return_data.set_error_msg(String::from("参数错误"));
            context.success(&return_data);
            return;
        }
        let file = file.unwrap();
        logger().info(format!("file: {} {}", path, file.to_string()));
        if !file.exists() {
            return_data.set_error_msg(String::from("路径不存在"));
            context.success(&return_data);
            return;
        }
        let mut response_ctx = context.response();
        let response = response_ctx.put_header("Cache-Control", "86400");
        if stream <= 0 {
            response.put_header("Content-Disposition", &format!("attachment; filename={}", url_encode_charset(file.name(), "UTF-8")));
        }
        response.send_file(file.to_string());
    }

    // suspend fun get(context: RoutingContext): ReturnData {
    //     checkAccess(context)?.let { return it }
    //     val returnData = ReturnData()
    //     val path = requestPath(context)
    //     if (path.isEmpty()) return returnData.setErrorMsg("参数错误")
    //     val file = getFileHome(context)?.let { resolveSecurePath(it, path) } ?: return returnData.setErrorMsg("参数错误")
    //     logger.info { "file: $path $file" }
    //     if (!file.exists()) return returnData.setErrorMsg("路径不存在")
    //     return returnData.setData(file.readText())
    // }
    pub fn get(&self, context: &RoutingContext) -> ReturnData {
        if let Some(result) = self.check_access(context, false, false) {
            return result;
        }
        let mut return_data = ReturnData::new();
        let path = self.request_path(context, String::from("path"));
        if path.is_empty() {
            return_data.set_error_msg(String::from("参数错误"));
            return return_data;
        }
        let file = match self.get_file_home(context).and_then(|it| Self::resolve_secure_path(&it, &path)) {
            Some(v) => v,
            None => {
                return_data.set_error_msg(String::from("参数错误"));
                return return_data;
            }
        };
        logger().info(format!("file: {} {}", path, file.to_string()));
        if !file.exists() {
            return_data.set_error_msg(String::from("路径不存在"));
            return return_data;
        }
        return_data.set_data(Box::new(file.read_text()), String::from(""));
        return return_data;
    }

    // suspend fun save(context: RoutingContext): ReturnData {
    //     checkAccess(context, isSave = true)?.let { return it }
    //     val returnData = ReturnData()
    //     val path = context.bodyAsJson?.getString("path", "") ?: ""
    //     val content = context.bodyAsJson?.getString("content", "") ?: ""
    //     if (path.isEmpty()) return returnData.setErrorMsg("参数错误")
    //     val file = getFileHome(context)?.let { resolveSecurePath(it, path) } ?: return returnData.setErrorMsg("参数错误")
    //     logger.info { "file: $path $file" }
    //     file.parentFile.mkdirs()
    //     file.writeText(content)
    //     return returnData.setData("")
    // }
    pub fn save(&self, context: &RoutingContext) -> ReturnData {
        if let Some(result) = self.check_access(context, true, false) {
            return result;
        }
        let mut return_data = ReturnData::new();
        let path = context.body_as_json().map(|j| j.get_string_default("path", "")).unwrap_or(String::from(""));
        let content = context.body_as_json().map(|j| j.get_string_default("content", "")).unwrap_or(String::from(""));
        if path.is_empty() {
            return_data.set_error_msg(String::from("参数错误"));
            return return_data;
        }
        let file = match self.get_file_home(context).and_then(|it| Self::resolve_secure_path(&it, &path)) {
            Some(v) => v,
            None => {
                return_data.set_error_msg(String::from("参数错误"));
                return return_data;
            }
        };
        logger().info(format!("file: {} {}", path, file.to_string()));
        if let Some(parent) = file.parent_file() {
            parent.mkdirs();
        }
        file.write_text(&content);
        return_data.set_data(Box::new(String::from("")), String::from(""));
        return return_data;
    }

    // suspend fun mkdir(context: RoutingContext): ReturnData {
    //     checkAccess(context, isSave = true)?.let { return it }
    //     val returnData = ReturnData()
    //     val path = context.bodyAsJson?.getString("path", "") ?: ""
    //     val name = context.bodyAsJson?.getString("name", "") ?: ""
    //     if (path.isEmpty() || name.isEmpty() || name.startsWith(".")) return returnData.setErrorMsg("参数错误")
    //     val parent = getFileHome(context)?.let { resolveSecurePath(it, path) } ?: return returnData.setErrorMsg("参数错误")
    //     val directory = resolveSecurePath(parent, name) ?: return returnData.setErrorMsg("参数错误")
    //     logger.info { "file: $path $directory" }
    //     if (directory.exists()) return returnData.setErrorMsg("路径已存在")
    //     directory.mkdirs()
    //     return returnData.setData("")
    // }
    pub fn mkdir(&self, context: &RoutingContext) -> ReturnData {
        if let Some(result) = self.check_access(context, true, false) {
            return result;
        }
        let mut return_data = ReturnData::new();
        let path = context.body_as_json().map(|j| j.get_string_default("path", "")).unwrap_or(String::from(""));
        let name = context.body_as_json().map(|j| j.get_string_default("name", "")).unwrap_or(String::from(""));
        if path.is_empty() || name.is_empty() || name.starts_with(".") {
            return_data.set_error_msg(String::from("参数错误"));
            return return_data;
        }
        let parent = match self.get_file_home(context).and_then(|it| Self::resolve_secure_path(&it, &path)) {
            Some(v) => v,
            None => {
                return_data.set_error_msg(String::from("参数错误"));
                return return_data;
            }
        };
        let directory = match Self::resolve_secure_path(&parent, &name) {
            Some(v) => v,
            None => {
                return_data.set_error_msg(String::from("参数错误"));
                return return_data;
            }
        };
        logger().info(format!("file: {} {}", path, directory.to_string()));
        if directory.exists() {
            return_data.set_error_msg(String::from("路径已存在"));
            return return_data;
        }
        directory.mkdirs();
        return_data.set_data(Box::new(String::from("")), String::from(""));
        return return_data;
    }

    // suspend fun delete(context: RoutingContext): ReturnData {
    //     checkAccess(context, isDelete = true)?.let { return it }
    //     val returnData = ReturnData()
    //     val path = requestPath(context)
    //     if (path.isEmpty()) return returnData.setErrorMsg("参数错误")
    //     val file = getFileHome(context)?.let { resolveSecurePath(it, path) } ?: return returnData.setErrorMsg("参数错误")
    //     logger.info { "file: $path $file" }
    //     if (!file.exists()) return returnData.setErrorMsg("路径不存在")
    //     file.deleteRecursively()
    //     return returnData.setData("")
    // }
    pub fn delete(&self, context: &RoutingContext) -> ReturnData {
        if let Some(result) = self.check_access(context, false, true) {
            return result;
        }
        let mut return_data = ReturnData::new();
        let path = self.request_path(context, String::from("path"));
        if path.is_empty() {
            return_data.set_error_msg(String::from("参数错误"));
            return return_data;
        }
        let file = match self.get_file_home(context).and_then(|it| Self::resolve_secure_path(&it, &path)) {
            Some(v) => v,
            None => {
                return_data.set_error_msg(String::from("参数错误"));
                return return_data;
            }
        };
        logger().info(format!("file: {} {}", path, file.to_string()));
        if !file.exists() {
            return_data.set_error_msg(String::from("路径不存在"));
            return return_data;
        }
        file.delete_recursively();
        return_data.set_data(Box::new(String::from("")), String::from(""));
        return return_data;
    }

    // suspend fun deleteMulti(context: RoutingContext): ReturnData {
    //     checkAccess(context, isDelete = true)?.let { return it }
    //     val returnData = ReturnData()
    //     val paths = context.bodyAsJson?.getJsonArray("path") ?: return returnData.setErrorMsg("参数错误")
    //     val baseDir = getFileHome(context) ?: return returnData.setErrorMsg("参数错误")
    //     paths.forEach { value ->
    //         val path = value as? String ?: return@forEach
    //         if (path.isNotEmpty()) resolveSecurePath(baseDir, path)?.deleteRecursively()
    //     }
    //     return returnData.setData("")
    // }
    pub fn delete_multi(&self, context: &RoutingContext) -> ReturnData {
        if let Some(result) = self.check_access(context, false, true) {
            return result;
        }
        let mut return_data = ReturnData::new();
        let paths = match context.body_as_json().and_then(|j| j.get_json_array("path")) {
            Some(v) => v,
            None => {
                return_data.set_error_msg(String::from("参数错误"));
                return return_data;
            }
        };
        let base_dir = match self.get_file_home(context) {
            Some(v) => v,
            None => {
                return_data.set_error_msg(String::from("参数错误"));
                return return_data;
            }
        };
        for path in paths.0 {
            if !path.is_empty() {
                if let Some(file) = Self::resolve_secure_path(&base_dir, &path) {
                    file.delete_recursively();
                }
            }
        }
        return_data.set_data(Box::new(String::from("")), String::from(""));
        return return_data;
    }

    // suspend fun importPreview(context: RoutingContext): ReturnData {
    //     checkAccess(context)?.let { return it }
    //     val returnData = ReturnData()
    //     val paths = context.bodyAsJson?.getJsonArray("path") ?: return returnData.setErrorMsg("参数错误")
    //     val baseDir = getFileHome(context) ?: return returnData.setErrorMsg("参数错误")
    //     val userNameSpace = getUserNameSpace(context)
    //     val rootDir = getWorkDir().let { if (it.endsWith(File.separator)) it else it + File.separator }
    //     val fileList = ArrayList<Map<String, Any>>()
    //     paths.forEach { value ->
    //         val path = value as? String ?: return@forEach
    //         if (path.isEmpty()) return@forEach
    //         val file = resolveSecurePath(baseDir, path) ?: return@forEach
    //         logger.info { "localFile: $path $file" }
    //         logger.debug("rootDir: {} path: {}", rootDir, file.path)
    //         if (!file.exists() || file.isDirectory) return@forEach
    //         val ext = getFileExt(file.name)
    //         if (ext !in setOf("txt", "epub", "umd", "cbz", "pdf")) {
    //             return returnData.setErrorMsg("不支持导入${ext}格式的书籍文件")
    //         }
    //         var relativePath = file.path
    //         if (relativePath.startsWith(rootDir)) relativePath = relativePath.removePrefix(rootDir)
    //         logger.debug("relative path: {}", relativePath)
    //         val book = Book.initLocalBook(relativePath.replace("\\", "/"), relativePath, rootDir)
    //         book.setUserNameSpace(userNameSpace)
    //         try {
    //             fileList += mapOf("book" to book, "chapters" to LocalBook.getChapterList(book))
    //         } catch (_: TocEmptyException) {
    //             fileList += mapOf("book" to book, "chapters" to arrayListOf<Any>())
    //         }
    //     }
    //     return returnData.setData(fileList)
    // }
    pub fn import_preview(&self, context: &RoutingContext) -> ReturnData {
        if let Some(result) = self.check_access(context, false, false) {
            return result;
        }
        let mut return_data = ReturnData::new();
        let paths = match context.body_as_json().and_then(|j| j.get_json_array("path")) {
            Some(v) => v,
            None => {
                return_data.set_error_msg(String::from("参数错误"));
                return return_data;
            }
        };
        let base_dir = match self.get_file_home(context) {
            Some(v) => v,
            None => {
                return_data.set_error_msg(String::from("参数错误"));
                return return_data;
            }
        };
        let user_name_space = self.base.get_user_name_space(context);
        let work_dir = get_work_dir("storage", vec![]);
        let root_dir = if work_dir.ends_with(&std::path::MAIN_SEPARATOR.to_string()) { work_dir } else { work_dir + &std::path::MAIN_SEPARATOR.to_string() };
        let mut file_list: Vec<std::collections::HashMap<String, Box<dyn std::any::Any>>> = Vec::new();
        for path in paths.0 {
            if path.is_empty() {
                continue;
            }
            let file = match Self::resolve_secure_path(&base_dir, &path) {
                Some(v) => v,
                None => continue,
            };
            logger().info(format!("localFile: {} {}", path, file.to_string()));
            logger().debug(format!("rootDir: {} path: {}", root_dir, file.path()));
            if !file.exists() || file.is_directory() {
                continue;
            }
            let ext = self.base.get_file_ext(file.name(), String::from(""));
            if !vec!["txt", "epub", "umd", "cbz", "pdf"].contains(&ext.as_str()) {
                return_data.set_error_msg(format!("不支持导入{}格式的书籍文件", ext));
                return return_data;
            }
            let mut relative_path = file.path();
            if relative_path.starts_with(&root_dir) {
                relative_path = relative_path.trim_start_matches(&root_dir).to_string();
            }
            logger().debug(format!("relative path: {}", relative_path));
            let mut book = Book::init_local_book(relative_path.replace("\\", "/"), relative_path, root_dir.clone());
            book.set_user_name_space(user_name_space.clone());
            let chapters = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| LocalBook::get_chapter_list(&mut book))) {
                Ok(chapters) => chapters,
                Err(_) => Vec::new(),
            };
            let mut m: std::collections::HashMap<String, Box<dyn std::any::Any>> = std::collections::HashMap::new();
            m.insert(String::from("book"), Box::new(book));
            m.insert(String::from("chapters"), Box::new(chapters));
            file_list.push(m);
        }
        return_data.set_data(Box::new(file_list), String::from(""));
        return return_data;
    }

    // suspend fun restore(context: RoutingContext): ReturnData {
    //     checkAccess(context)?.let { return it }
    //     val returnData = ReturnData()
    //     val path = requestPath(context).ifEmpty { "/" }
    //     if (getFileExt(path) != "zip") return returnData.setErrorMsg("路径不是zip备份文件")
    //     val file = getFileHome(context)?.let { resolveSecurePath(it, path) } ?: return returnData.setErrorMsg("参数错误")
    //     logger.info { "file: $path $file" }
    //     if (!file.exists()) return returnData.setErrorMsg("路径不存在")
    //     if (!BookController(coroutineContext).syncFromWebdav(file.toString(), getUserNameSpace(context))) {
    //         return returnData.setErrorMsg("恢复失败")
    //     }
    //     return returnData.setData("")
    // }
    pub fn restore(&self, context: &RoutingContext) -> ReturnData {
        if let Some(result) = self.check_access(context, false, false) {
            return result;
        }
        let mut return_data = ReturnData::new();
        let path = if self.request_path(context, String::from("path")).is_empty() { String::from("/") } else { self.request_path(context, String::from("path")) };
        if self.base.get_file_ext(path.clone(), String::from("")) != "zip" {
            return_data.set_error_msg(String::from("路径不是zip备份文件"));
            return return_data;
        }
        let file = match self.get_file_home(context).and_then(|it| Self::resolve_secure_path(&it, &path)) {
            Some(v) => v,
            None => {
                return_data.set_error_msg(String::from("参数错误"));
                return return_data;
            }
        };
        logger().info(format!("file: {} {}", path, file.to_string()));
        if !file.exists() {
            return_data.set_error_msg(String::from("路径不存在"));
            return return_data;
        }
        if !BookController::new().sync_from_webdav(file.to_string(), self.base.get_user_name_space(context)) {
            return_data.set_error_msg(String::from("恢复失败"));
            return return_data;
        }
        return_data.set_data(Box::new(String::from("")), String::from(""));
        return return_data;
    }

    // suspend fun parse(context: RoutingContext): ReturnData {
    //     checkAccess(context)?.let { return it }
    //     val returnData = ReturnData()
    //     val path = requestPath(context).ifEmpty { "/" }
    //     val import = if (context.request().method() == HttpMethod.POST) {
    //         context.bodyAsJson?.getInteger("import", 0) ?: 0
    //     } else {
    //         context.queryParam("import").firstOrNull()?.toIntOrNull() ?: 0
    //     }
    //     val baseDir = getFileHome(context) ?: return returnData.setErrorMsg("参数错误")
    //     val directory = resolveSecurePath(baseDir, path) ?: return returnData.setErrorMsg("路径不存在")
    //     logger.info { "file: $path $directory" }
    //     if (!directory.exists()) return returnData.setErrorMsg("路径不存在")
    //     if (!directory.isDirectory) return returnData.setErrorMsg("路径不是目录")
    //     val userNameSpace = getUserNameSpace(context)
    //     val rootDir = getWorkDir().let { if (it.endsWith(File.separator)) it else it + File.separator }
    //     val bookController = BookController(coroutineContext)
    //     val fileList = ArrayList<Map<String, Any>>()
    //     listFilesRecursively(directory).forEach { file ->
    //         if (file.name.startsWith(".") || !file.isFile || getFileExt(file.name) !in setOf("txt", "epub", "umd", "cbz", "pdf")) return@forEach
    //         logger.debug("rootDir: {} path: {}", rootDir, file.path)
    //         var relativePath = file.path
    //         if (relativePath.startsWith(rootDir)) relativePath = relativePath.removePrefix(rootDir)
    //         logger.debug("relative path: {}", relativePath)
    //         val book = Book.initLocalBook(relativePath.replace("\\", "/"), relativePath, rootDir)
    //         book.setUserNameSpace(userNameSpace)
    //         logger.debug("book {}", book)
    //         if (import > 0) {
    //             val result = bookController.saveBookToShelf(book, userNameSpace, context)
    //             if (result.second == None && result.first.isInShelf) fileList += mapOf("name" to file.name)
    //         } else {
    //             fileList += mapOf(
    //                 "name" to file.name,
    //                 "size" to file.length(),
    //                 "path" to "/" + file.relativeTo(baseDir).path.replace(File.separatorChar, '/'),
    //                 "lastModified" to file.lastModified(),
    //                 "book" to book
    //             )
    //         }
    //     }
    //     return returnData.setData(fileList)
    // }
    pub fn parse(&self, context: &RoutingContext) -> ReturnData {
        if let Some(result) = self.check_access(context, false, false) {
            return result;
        }
        let mut return_data = ReturnData::new();
        let path = if self.request_path(context, String::from("path")).is_empty() { String::from("/") } else { self.request_path(context, String::from("path")) };
        let import = if context.request().method() == HttpMethod::POST {
            context.body_as_json().map(|j| j.get_integer("import", 0)).unwrap_or(0)
        } else {
            context.query_param("import").and_then(|s| s.parse::<i32>().ok()).unwrap_or(0)
        };
        let base_dir = match self.get_file_home(context) {
            Some(v) => v,
            None => {
                return_data.set_error_msg(String::from("参数错误"));
                return return_data;
            }
        };
        let directory = match Self::resolve_secure_path(&base_dir, &path) {
            Some(v) => v,
            None => {
                return_data.set_error_msg(String::from("路径不存在"));
                return return_data;
            }
        };
        logger().info(format!("file: {} {}", path, directory.to_string()));
        if !directory.exists() {
            return_data.set_error_msg(String::from("路径不存在"));
            return return_data;
        }
        if !directory.is_directory() {
            return_data.set_error_msg(String::from("路径不是目录"));
            return return_data;
        }
        let user_name_space = self.base.get_user_name_space(context);
        let work_dir = get_work_dir("storage", vec![]);
        let root_dir = if work_dir.ends_with(&std::path::MAIN_SEPARATOR.to_string()) { work_dir } else { work_dir + &std::path::MAIN_SEPARATOR.to_string() };
        let book_controller = BookController::new();
        let mut file_list: Vec<std::collections::HashMap<String, Box<dyn std::any::Any>>> = Vec::new();
        for file in list_files_recursively(&directory) {
            if file.name().starts_with(".") || !file.is_file() || !vec!["txt", "epub", "umd", "cbz", "pdf"].contains(&self.base.get_file_ext(file.name(), String::from("")).as_str()) {
                continue;
            }
            logger().debug(format!("rootDir: {} path: {}", root_dir, file.path()));
            let mut relative_path = file.path();
            if relative_path.starts_with(&root_dir) {
                relative_path = relative_path.trim_start_matches(&root_dir).to_string();
            }
            logger().debug(format!("relative path: {}", relative_path));
            let mut book = Book::init_local_book(relative_path.replace("\\", "/"), relative_path, root_dir.clone());
            book.set_user_name_space(user_name_space.clone());
            logger().debug(format!("book {}", book.name));
            if import > 0 {
                let result = book_controller.save_book_to_shelf(book, user_name_space.clone(), context);
                if result.1.is_none() && result.0.is_in_shelf {
                    let mut m: std::collections::HashMap<String, Box<dyn std::any::Any>> = std::collections::HashMap::new();
                    m.insert(String::from("name"), Box::new(file.name()));
                    file_list.push(m);
                }
            } else {
                let mut m: std::collections::HashMap<String, Box<dyn std::any::Any>> = std::collections::HashMap::new();
                m.insert(String::from("name"), Box::new(file.name()));
                m.insert(String::from("size"), Box::new(file.length()));
                m.insert(String::from("path"), Box::new(format!("/{}", file.relative_to(&base_dir).path().replace(std::path::MAIN_SEPARATOR, "/"))));
                m.insert(String::from("lastModified"), Box::new(file.last_modified()));
                m.insert(String::from("book"), Box::new(book));
                file_list.push(m);
            }
        }
        return_data.set_data(Box::new(file_list), String::from(""));
        return return_data;
    }
}
