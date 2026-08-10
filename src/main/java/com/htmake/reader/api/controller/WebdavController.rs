// package com.htmake.reader.api.controller

// private val logger = KotlinLogging.logger {}

// class WebdavController(coroutineContext: CoroutineContext, router: Router, onHandlerError: (RoutingContext, Exception) -> Unit): BaseController(coroutineContext) {
pub struct WebdavController {
    base: BaseController,
    // onHandlerError: (RoutingContext, Exception) -> Unit
    on_handler_error: Box<dyn Fn(&RoutingContext, &Exception)>,
}

impl WebdavController {
    // private fun decodedPath(path: String): String {
    //     return URLDecoder.decode(path.replace("/reader3/webdav/", "/", true), "UTF-8")
    // }
    fn decoded_path(path: &String) -> String {
        return url_decode(&path.replace("/reader3/webdav/", "/"), "UTF-8");
    }

    // private fun requestPath(context: RoutingContext): String {
    //     return decodedPath(context.request().path())
    // }
    fn request_path(context: &RoutingContext) -> String {
        return Self::decoded_path(&context.request().path());
    }

    // private fun resolveWebdavPath(context: RoutingContext, path: String): File? {
    //     val home = File(getUserWebdavHome(context)).toPath().toAbsolutePath().normalize()
    //     val relative = path.replace('\\', '/').removePrefix("/")
    //     val resolved = home.resolve(relative).normalize()
    //     return resolved.takeIf { it.startsWith(home) }?.toFile()
    // }
    fn resolve_webdav_path(&self, context: &RoutingContext, path: &String) -> Option<File> {
        let home = File::new(&self.base.get_user_webdav_home(context as &dyn std::any::Any)).to_path().to_absolute_path().normalize();
        let relative = path.replace('\\', "/").trim_start_matches("/").to_string();
        let resolved = home.resolve(&relative).normalize();
        if resolved.starts_with(&home) {
            return Some(resolved.to_file());
        }
        return None;
    }

    // private fun destinationPath(context: RoutingContext): File? {
    //     val destination = context.request().getHeader("Destination") ?: return null
    //     val destinationPath = runCatching { URL(destination).path }.getOrNull() ?: return null
    //     return resolveWebdavPath(context, decodedPath(destinationPath))
    // }
    fn destination_path(&self, context: &RoutingContext) -> Option<File> {
        let destination = match context.request().get_header("Destination") {
            Some(v) => v,
            None => return None,
        };
        let destination_path = match std::panic::catch_unwind(|| URL::new(&destination).path()) {
            Ok(p) => p,
            Err(_) => return None,
        };
        return self.resolve_webdav_path(context, &Self::decoded_path(&destination_path));
    }

    // init {
    //     // webdav 服务
    //     router.route("/reader3/webdav*").globalHandler {
    //         it.addHeadersEndHandler { _ ->
    //             var res = it.response()
    //             res.putHeader("DAV", "1,2")
    //             res.putHeader("Access-Control-Allow-Origin", "*")
    //             res.putHeader("Access-Control-Allow-Credentials", "true")
    //             res.putHeader("Access-Control-Expose-Headers", "DAV, content-length, Allow")
    //             res.putHeader("MS-Author-Via", "DAV")
    //             res.putHeader("Allow", "OPTIONS,DELETE,GET,PUT,PROPFIND,MKCOL,MOVE,COPY,LOCK,UNLOCK")
    //             if (appConfig.secure) {
    //                 res.putHeader("WWW-Authenticate", "Basic realm=\"Default realm\"")
    //             }
    //         }
    //         val rawMethod = it.request().rawMethod()
    //         if (!checkAuthorization(it)) {
    //             if (
    //                 rawMethod.equals("PROPFIND") ||
    //                 rawMethod.equals("MKCOL") ||
    //                 rawMethod.equals("PUT") ||
    //                 rawMethod.equals("GET") ||
    //                 rawMethod.equals("DELETE") ||
    //                 rawMethod.equals("MOVE") ||
    //                 rawMethod.equals("COPY") ||
    //                 rawMethod.equals("LOCK") ||
    //                 rawMethod.equals("UNLOCK")
    //             ) {
    //                 it.response().setStatusCode(401).end()
    //                 return@globalHandler
    //             } else if(rawMethod.equals("OPTIONS")) {
    //                 // CORS 预检请求不校验认证：浏览器/WebDAV 客户端预检会携带 Authorization 头，
    //                 // 此处返回 401 会导致客户端报"非法访问"、无法同步（JAR 继承 bug）
    //                 it.response().setStatusCode(200).end()
    //                 return@globalHandler
    //             }
    //         }
    //         when (rawMethod) {
    //             "PROPFIND" -> launch(Dispatchers.IO) {
    //                 try {
    //                     webdavList(it)
    //                 } catch (e: Exception) {
    //                     onHandlerError(it, e)
    //                 }
    //             }
    //             "MKCOL" -> launch(Dispatchers.IO) {
    //                 try {
    //                     webdavMkdir(it)
    //                 } catch (e: Exception) {
    //                     onHandlerError(it, e)
    //                 }
    //             }
    //             "PUT" -> launch(Dispatchers.IO) {
    //                 try {
    //                     webdavUpload(it)
    //                 } catch (e: Exception) {
    //                     onHandlerError(it, e)
    //                 }
    //             }
    //             "GET" -> launch(Dispatchers.IO) {
    //                 try {
    //                     webdavDownload(it)
    //                 } catch (e: Exception) {
    //                     onHandlerError(it, e)
    //                 }
    //             }
    //             "DELETE" -> launch(Dispatchers.IO) {
    //                 try {
    //                     webdavDelete(it)
    //                 } catch (e: Exception) {
    //                     onHandlerError(it, e)
    //                 }
    //             }
    //             "MOVE" -> launch(Dispatchers.IO) {
    //                 try {
    //                     webdavMove(it)
    //                 } catch (e: Exception) {
    //                     onHandlerError(it, e)
    //                 }
    //             }
    //             "COPY" -> launch(Dispatchers.IO) {
    //                 try {
    //                     webdavCopy(it)
    //                 } catch (e: Exception) {
    //                     onHandlerError(it, e)
    //                 }
    //             }
    //             "LOCK" -> launch(Dispatchers.IO) {
    //                 try {
    //                     webdavLock(it)
    //                 } catch (e: Exception) {
    //                     onHandlerError(it, e)
    //                 }
    //             }
    //             "UNLOCK" -> launch(Dispatchers.IO) {
    //                 try {
    //                     webdavUnLock(it)
    //                 } catch (e: Exception) {
    //                     onHandlerError(it, e)
    //                 }
    //             }
    //             "OPTIONS" -> it.response().setStatusCode(200).end()
    //             else -> it.response().setStatusCode(405).end()
    //         }
    //     }
    // }
    pub fn new(base: BaseController, router: &mut Router, on_handler_error: Box<dyn Fn(&RoutingContext, &Exception)>) -> WebdavController {
        // webdav 服务
        let webdav_controller = WebdavController {
            base,
            on_handler_error,
        };
        router.route("/reader3/webdav*").global_handler(|it| {
            it.add_headers_end_handler(|_| {
                let res = it.response();
                res.put_header("DAV", String::from("1,2"));
                res.put_header("Access-Control-Allow-Origin", String::from("*"));
                res.put_header("Access-Control-Allow-Credentials", String::from("true"));
                res.put_header("Access-Control-Expose-Headers", String::from("DAV, content-length, Allow"));
                res.put_header("MS-Author-Via", String::from("DAV"));
                res.put_header("Allow", String::from("OPTIONS,DELETE,GET,PUT,PROPFIND,MKCOL,MOVE,COPY,LOCK,UNLOCK"));
                if webdav_controller.base.app_config.secure {
                    res.put_header("WWW-Authenticate", String::from("Basic realm=\"Default realm\""));
                }
            });
            let raw_method = it.request().raw_method();
            if !webdav_controller.check_authorization(it) {
                if raw_method == "PROPFIND"
                    || raw_method == "MKCOL"
                    || raw_method == "PUT"
                    || raw_method == "GET"
                    || raw_method == "DELETE"
                    || raw_method == "MOVE"
                    || raw_method == "COPY"
                    || raw_method == "LOCK"
                    || raw_method == "UNLOCK"
                {
                    it.response().set_status_code(401).end();
                    return;
                } else if raw_method == "OPTIONS" {
                    // CORS 预检请求不校验认证：浏览器/WebDAV 客户端预检会携带 Authorization 头，
                    // 此处返回 401 会导致客户端报"非法访问"、无法同步（JAR 继承 bug）
                    it.response().set_status_code(200).end();
                    return;
                }
            }
            match raw_method.as_str() {
                "PROPFIND" => {
                    // launch(Dispatchers.IO) {
                    let result = std::panic::catch_unwind(|| {
                        webdav_controller.webdav_list(it);
                    });
                    if let Err(e) = result {
                        (webdav_controller.on_handler_error)(it, &Exception::new(e.to_string()));
                    }
                    // }
                }
                "MKCOL" => {
                    // launch(Dispatchers.IO) {
                    let result = std::panic::catch_unwind(|| {
                        webdav_controller.webdav_mkdir(it);
                    });
                    if let Err(e) = result {
                        (webdav_controller.on_handler_error)(it, &Exception::new(e.to_string()));
                    }
                    // }
                }
                "PUT" => {
                    // launch(Dispatchers.IO) {
                    let result = std::panic::catch_unwind(|| {
                        webdav_controller.webdav_upload(it);
                    });
                    if let Err(e) = result {
                        (webdav_controller.on_handler_error)(it, &Exception::new(e.to_string()));
                    }
                    // }
                }
                "GET" => {
                    // launch(Dispatchers.IO) {
                    let result = std::panic::catch_unwind(|| {
                        webdav_controller.webdav_download(it);
                    });
                    if let Err(e) = result {
                        (webdav_controller.on_handler_error)(it, &Exception::new(e.to_string()));
                    }
                    // }
                }
                "DELETE" => {
                    // launch(Dispatchers.IO) {
                    let result = std::panic::catch_unwind(|| {
                        webdav_controller.webdav_delete(it);
                    });
                    if let Err(e) = result {
                        (webdav_controller.on_handler_error)(it, &Exception::new(e.to_string()));
                    }
                    // }
                }
                "MOVE" => {
                    // launch(Dispatchers.IO) {
                    let result = std::panic::catch_unwind(|| {
                        webdav_controller.webdav_move(it);
                    });
                    if let Err(e) = result {
                        (webdav_controller.on_handler_error)(it, &Exception::new(e.to_string()));
                    }
                    // }
                }
                "COPY" => {
                    // launch(Dispatchers.IO) {
                    let result = std::panic::catch_unwind(|| {
                        webdav_controller.webdav_copy(it);
                    });
                    if let Err(e) = result {
                        (webdav_controller.on_handler_error)(it, &Exception::new(e.to_string()));
                    }
                    // }
                }
                "LOCK" => {
                    // launch(Dispatchers.IO) {
                    let result = std::panic::catch_unwind(|| {
                        webdav_controller.webdav_lock(it);
                    });
                    if let Err(e) = result {
                        (webdav_controller.on_handler_error)(it, &Exception::new(e.to_string()));
                    }
                    // }
                }
                "UNLOCK" => {
                    // launch(Dispatchers.IO) {
                    let result = std::panic::catch_unwind(|| {
                        webdav_controller.webdav_un_lock(it);
                    });
                    if let Err(e) = result {
                        (webdav_controller.on_handler_error)(it, &Exception::new(e.to_string()));
                    }
                    // }
                }
                "OPTIONS" => it.response().set_status_code(200).end(),
                _ => it.response().set_status_code(405).end(),
            }
        });
        webdav_controller
    }

    // fun checkAuthorization(context: RoutingContext): Boolean {
    //     if (!appConfig.secure) {
    //         return true
    //     }
    //     var authorization = context.request().getHeader("Authorization")
    //     if (authorization == null || authorization.isEmpty()) {
    //         return false
    //     }
    //
    //     // Basic YTox
    //     val auth = EncoderUtils.base64Decode(authorization.replace("Basic ", "", true)).split(":", limit=2)
    //     if (auth.size < 2) {
    //         return false
    //     }
    //     val username = auth[0]
    //     val password = auth[1]
    //     var userMap = mutableMapOf<String, Map<String, Any>>()
    //     var userMapJson: JsonObject? = asJsonObject(getStorage("data", "users"))
    //     if (userMapJson != null) {
    //         userMap = userMapJson.map as MutableMap<String, Map<String, Any>>
    //     }
    //     var existedUser = userMap.getOrDefault(username, null)
    //     if (existedUser == null) {
    //         return false
    //     }
    //     var userInfo: User? = existedUser.toDataClass()
    //     if (userInfo == null) {
    //         return false
    //     }
    //     var passwordEncrypted = genEncryptedPassword(password, userInfo.salt)
    //     if (passwordEncrypted != userInfo.password) {
    //         logger.info("user: {} password error", userInfo.username)
    //         return false
    //     }
    //
    //     if (!userInfo.enable_webdav) {
    //         logger.info("user: {} enable_webdav: false", userInfo.username)
    //         return false
    //     }
    //
    //     context.put("username", userInfo.username)
    //
    //     return true
    // }
    pub fn check_authorization(&self, context: &RoutingContext) -> bool {
        if !self.base.app_config.secure {
            return true;
        }
        let authorization = context.request().get_header("Authorization");
        if authorization.is_none() || authorization.as_ref().unwrap().is_empty() {
            return false;
        }
        let authorization = authorization.unwrap();

        // Basic YTox
        let auth = base64_decode(&authorization.replace("Basic ", "")).splitn(2, ':').map(|s| s.to_string()).collect::<Vec<String>>();
        if auth.len() < 2 {
            return false;
        }
        let username = auth[0].clone();
        let password = auth[1].clone();
        let mut user_map: std::collections::HashMap<String, std::collections::HashMap<String, Box<dyn std::any::Any>>> = std::collections::HashMap::new();
        let user_map_json: Option<JsonObject> = as_json_object(get_storage("data", vec![String::from("users")]));
        if let Some(json) = user_map_json {
            user_map = json.map().clone();
        }
        let existed_user = user_map.get(&username).cloned();
        if existed_user.is_none() {
            return false;
        }
        let user_info: Option<User> = existed_user.unwrap().to_data_class();
        if user_info.is_none() {
            return false;
        }
        let user_info = user_info.unwrap();
        let password_encrypted = gen_encrypted_password(&password, &user_info.salt);
        if password_encrypted != user_info.password {
            logger.info(format!("user: {} password error", user_info.username));
            return false;
        }

        if !user_info.enable_webdav {
            logger.info(format!("user: {} enable_webdav: false", user_info.username));
            return false;
        }

        context.put("username", user_info.username.clone());

        return true;
    }

    // suspend fun webdavList(context: RoutingContext) {
    //     val file = resolveWebdavPath(context, requestPath(context))
    //     if (file == null) {
    //         context.response().setStatusCode(404).end()
    //         return
    //     }
    //     if (!file.exists()) {
    //         context.response().setStatusCode(404).end()
    //         return
    //     }
    //
    //     var xml =
    //     """<?xml version="1.0" encoding="utf-8"?>
    //         <D:multistatus xmlns:D="DAV:">
    //             %s
    //         </D:multistatus>
    //     """
    //
    //     var dirResponse =
    //     """<D:response>
    //             <D:href>%s</D:href>
    //             <D:propstat>
    //                 <D:status>HTTP/1.1 200 OK</D:status>
    //                 <D:prop>
    //                     <D:getlastmodified>%s</D:getlastmodified>
    //                     <D:creationdate>%s</D:creationdate>
    //                     <D:resourcetype>
    //                         <D:collection />
    //                     </D:resourcetype>
    //                     <D:displayname>%s</D:displayname>
    //                 </D:prop>
    //             </D:propstat>
    //         </D:response>
    //     """
    //
    //     var fileResponse =
    //     """<D:response>
    //             <D:href>%s</D:href>
    //             <D:propstat>
    //                 <D:status>HTTP/1.1 200 OK</D:status>
    //                 <D:prop>
    //                     <D:getlastmodified>%s</D:getlastmodified>
    //                     <D:creationdate>%s</D:creationdate>
    //                     <D:resourcetype />
    //                     <D:displayname>%s</D:displayname>
    //                     <D:getcontentlength>%s</D:getcontentlength>
    //                     <D:getcontenttype>%s</D:getcontenttype>
    //                 </D:prop>
    //             </D:propstat>
    //         </D:response>
    //     """
    //
    //     var fileUrl = context.request().absoluteURI()
    //
    //     // 只支持一级
    //     var formatter = { f: File, url: String, showName: Boolean ->
    //         var name = if(showName) f.name else ""
    //         var modifiedDate = SimpleDateFormat("yyyy-MM-dd HH:mm:ss").format(f.lastModified())
    //         if (f.isFile()) {
    //             String.format(fileResponse, url, modifiedDate, modifiedDate, name, f.length(), "")
    //         } else {
    //             String.format(dirResponse, url, modifiedDate, modifiedDate, name)
    //         }
    //     }
    //
    //     var response = ""
    //     if (file.isFile()) {
    //         response = String.format(xml, formatter(file, fileUrl, true))
    //         context.response().setStatusCode(207).end(response)
    //         return
    //     }
    //
    //     if (file.isDirectory()) {
    //         fileUrl = if (fileUrl.endsWith("/")) fileUrl else fileUrl + "/"
    //         response = formatter(file, fileUrl, false)
    //         file.listFiles().forEach {
    //             val fileName = URLEncoder.encode(it.name, "UTF-8")
    //             response = response + formatter(it, fileUrl + fileName, true)
    //         }
    //         response = String.format(xml, response)
    //         context.response().setStatusCode(207).end(response)
    //         return
    //     }
    //
    //     context.response().setStatusCode(404).end()
    // }
    pub fn webdav_list(&self, context: &RoutingContext) {
        let file = self.resolve_webdav_path(context, &Self::request_path(context));
        if file.is_none() {
            context.response().set_status_code(404).end();
            return;
        }
        let file = file.unwrap();
        if !file.exists() {
            context.response().set_status_code(404).end();
            return;
        }

        let xml = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
            <D:multistatus xmlns:D=\"DAV:\">\n\
                %s\n\
            </D:multistatus>\n";

        let dir_response = "<D:response>\n\
                <D:href>%s</D:href>\n\
                <D:propstat>\n\
                    <D:status>HTTP/1.1 200 OK</D:status>\n\
                    <D:prop>\n\
                        <D:getlastmodified>%s</D:getlastmodified>\n\
                        <D:creationdate>%s</D:creationdate>\n\
                        <D:resourcetype>\n\
                            <D:collection />\n\
                        </D:resourcetype>\n\
                        <D:displayname>%s</D:displayname>\n\
                    </D:prop>\n\
                </D:propstat>\n\
            </D:response>\n";

        let file_response = "<D:response>\n\
                <D:href>%s</D:href>\n\
                <D:propstat>\n\
                    <D:status>HTTP/1.1 200 OK</D:status>\n\
                    <D:prop>\n\
                        <D:getlastmodified>%s</D:getlastmodified>\n\
                        <D:creationdate>%s</D:creationdate>\n\
                        <D:resourcetype />\n\
                        <D:displayname>%s</D:displayname>\n\
                        <D:getcontentlength>%s</D:getcontentlength>\n\
                        <D:getcontenttype>%s</D:getcontenttype>\n\
                    </D:prop>\n\
                </D:propstat>\n\
            </D:response>\n";

        let mut file_url = context.request().absolute_uri();

        // 只支持一级
        // var formatter = { f: File, url: String, showName: Boolean ->
        //     var name = if(showName) f.name else ""
        //     var modifiedDate = SimpleDateFormat("yyyy-MM-dd HH:mm:ss").format(f.lastModified())
        //     if (f.isFile()) {
        //         String.format(fileResponse, url, modifiedDate, modifiedDate, name, f.length(), "")
        //     } else {
        //         String.format(dirResponse, url, modifiedDate, modifiedDate, name)
        //     }
        // }
        fn formatter(f: &File, url: &String, show_name: bool, file_response: &str, dir_response: &str) -> String {
            let name = if show_name { f.name() } else { String::from("") };
            let modified_date = simple_date_format("yyyy-MM-dd HH:mm:ss", f.last_modified());
            if f.is_file() {
                format!(file_response, url, modified_date, modified_date, name, f.length(), "")
            } else {
                format!(dir_response, url, modified_date, modified_date, name)
            }
        }

        let mut response = String::from("");
        if file.is_file() {
            response = format!(xml, formatter(&file, &file_url, true, file_response, dir_response));
            context.response().set_status_code(207).end(response);
            return;
        }

        if file.is_directory() {
            if !file_url.ends_with("/") {
                file_url = file_url + "/";
            }
            response = formatter(&file, &file_url, false, file_response, dir_response);
            for it in file.list_files() {
                let file_name = url_encode(&it.name(), "UTF-8");
                response = response + &formatter(&it, &(file_url.clone() + &file_name), true, file_response, dir_response);
            }
            response = format!(xml, response);
            context.response().set_status_code(207).end(response);
            return;
        }

        context.response().set_status_code(404).end();
    }

    // suspend fun webdavMkdir(context: RoutingContext) {
    //     val file = resolveWebdavPath(context, requestPath(context))
    //     if (file == null) {
    //         context.response().setStatusCode(400).end()
    //         return
    //     }
    //     if (file.exists()) {
    //         // 文件夹存在时，返回成功
    //         context.response().setStatusCode(201).end()
    //         return
    //     }
    //     try {
    //         file.mkdirs()
    //         context.response().setStatusCode(201).end()
    //     } catch(e: Exception) {
    //         context.response().setStatusCode(500).end()
    //     }
    // }
    pub fn webdav_mkdir(&self, context: &RoutingContext) {
        let file = self.resolve_webdav_path(context, &Self::request_path(context));
        if file.is_none() {
            context.response().set_status_code(400).end();
            return;
        }
        let file = file.unwrap();
        if file.exists() {
            // 文件夹存在时，返回成功
            context.response().set_status_code(201).end();
            return;
        }
        let result = std::panic::catch_unwind(|| {
            file.mkdirs();
            context.response().set_status_code(201).end();
        });
        if result.is_err() {
            context.response().set_status_code(500).end();
        }
    }

    // suspend fun webdavUpload(context: RoutingContext) {
    //     val file = resolveWebdavPath(context, requestPath(context))
    //     if (file == null) {
    //         context.response().setStatusCode(400).end()
    //         return
    //     }
    //     if (!file.parentFile.exists()) {
    //         context.response().setStatusCode(409).end()
    //         return
    //     }
    //     if (file.isDirectory()) {
    //         context.response().setStatusCode(405).end()
    //         return
    //     }
    //     if (file.exists()) {
    //         file.delete();
    //     }
    //     try {
    //         file.writeBytes(context.getBody().getBytes())
    //         // 同步用户进度
    //         if (file.toString().indexOf("/bookProgress/") > 0 && file.toString().indexOf(".json") > 0) {
    //             val userNameSpace = getUserNameSpace(context)
    //             BookController(coroutineContext).syncBookProgressFromWebdav(file, userNameSpace)
    //         }
    //         context.response().setStatusCode(201).end()
    //     } catch(e: Exception) {
    //         context.response().setStatusCode(500).end()
    //     }
    // }
    pub fn webdav_upload(&self, context: &RoutingContext) {
        let file = self.resolve_webdav_path(context, &Self::request_path(context));
        if file.is_none() {
            context.response().set_status_code(400).end();
            return;
        }
        let file = file.unwrap();
        if !file.parent_file().exists() {
            context.response().set_status_code(409).end();
            return;
        }
        if file.is_directory() {
            context.response().set_status_code(405).end();
            return;
        }
        if file.exists() {
            file.delete();
        }
        let result = std::panic::catch_unwind(|| {
            file.write_bytes(context.get_body().get_bytes());
            // 同步用户进度
            if file.to_string().find("/bookProgress/").is_some() && file.to_string().find(".json").is_some() {
                let user_name_space = self.base.get_user_name_space(context);
                BookController::new().sync_book_progress_from_webdav(&file, user_name_space);
            }
            context.response().set_status_code(201).end();
        });
        if result.is_err() {
            context.response().set_status_code(500).end();
        }
    }

    // suspend fun webdavDownload(context: RoutingContext) {
    //     val file = resolveWebdavPath(context, requestPath(context))
    //     if (file == null) {
    //         context.response().setStatusCode(404).end()
    //         return
    //     }
    //     if (!file.exists()) {
    //         context.response().setStatusCode(404).end()
    //         return
    //     }
    //     if (file.isDirectory()) {
    //         context.response().setStatusCode(405).end()
    //         return
    //     }
    //     context.response().putHeader("Cache-Control", "86400")
    //                     .putHeader("Content-Disposition", "attachment; filename=" + URLEncoder.encode(file.name, "UTF-8"))
    //                     .sendFile(file.toString())
    // }
    pub fn webdav_download(&self, context: &RoutingContext) {
        let file = self.resolve_webdav_path(context, &Self::request_path(context));
        if file.is_none() {
            context.response().set_status_code(404).end();
            return;
        }
        let file = file.unwrap();
        if !file.exists() {
            context.response().set_status_code(404).end();
            return;
        }
        if file.is_directory() {
            context.response().set_status_code(405).end();
            return;
        }
        context.response().put_header("Cache-Control", String::from("86400"))
            .put_header("Content-Disposition", format!("attachment; filename={}", url_encode(&file.name(), "UTF-8")))
            .send_file(file.to_string());
    }

    // suspend fun webdavDelete(context: RoutingContext) {
    //     val file = resolveWebdavPath(context, requestPath(context))
    //     if (file == null) {
    //         context.response().setStatusCode(404).end()
    //         return
    //     }
    //     if (!file.exists()) {
    //         context.response().setStatusCode(404).end()
    //         return
    //     }
    //     file.deleteRecursively()
    //     context.response().setStatusCode(200).end()
    // }
    pub fn webdav_delete(&self, context: &RoutingContext) {
        let file = self.resolve_webdav_path(context, &Self::request_path(context));
        if file.is_none() {
            context.response().set_status_code(404).end();
            return;
        }
        let file = file.unwrap();
        if !file.exists() {
            context.response().set_status_code(404).end();
            return;
        }
        file.delete_recursively();
        context.response().set_status_code(200).end();
    }

    // suspend fun webdavMove(context: RoutingContext) {
    //     val file = resolveWebdavPath(context, requestPath(context))
    //     if (file == null) {
    //         context.response().setStatusCode(412).end()
    //         return
    //     }
    //     if (!file.exists()) {
    //         context.response().setStatusCode(412).end()
    //         return
    //     }
    //     var destination = context.request().getHeader("Destination")
    //     if (destination == null) {
    //         context.response().setStatusCode(400).end()
    //         return
    //     }
    //     val destinationFile = destinationPath(context)
    //     if (destinationFile == null) {
    //         context.response().setStatusCode(400).end()
    //         return
    //     }
    //
    //     var overwrite = context.request().getHeader("Overwrite")
    //     if (destinationFile.exists()) {
    //         if (overwrite == null || overwrite.isEmpty()) {
    //             context.response().setStatusCode(412).end()
    //             return
    //         }
    //         destinationFile.deleteRecursively()
    //     }
    //     file.renameTo(destinationFile)
    //
    //     context.response().setStatusCode(201).end()
    // }
    pub fn webdav_move(&self, context: &RoutingContext) {
        let file = self.resolve_webdav_path(context, &Self::request_path(context));
        if file.is_none() {
            context.response().set_status_code(412).end();
            return;
        }
        let file = file.unwrap();
        if !file.exists() {
            context.response().set_status_code(412).end();
            return;
        }
        let destination = context.request().get_header("Destination");
        if destination.is_none() {
            context.response().set_status_code(400).end();
            return;
        }
        let destination_file = self.destination_path(context);
        if destination_file.is_none() {
            context.response().set_status_code(400).end();
            return;
        }
        let destination_file = destination_file.unwrap();

        let overwrite = context.request().get_header("Overwrite");
        if destination_file.exists() {
            if overwrite.is_none() || overwrite.unwrap().is_empty() {
                context.response().set_status_code(412).end();
                return;
            }
            destination_file.delete_recursively();
        }
        file.rename_to(&destination_file);

        context.response().set_status_code(201).end();
    }

    // suspend fun webdavCopy(context: RoutingContext) {
    //     val file = resolveWebdavPath(context, requestPath(context))
    //     if (file == null) {
    //         context.response().setStatusCode(412).end()
    //         return
    //     }
    //     if (!file.exists()) {
    //         context.response().setStatusCode(412).end()
    //         return
    //     }
    //     var destination = context.request().getHeader("Destination")
    //     if (destination == null) {
    //         context.response().setStatusCode(400).end()
    //         return
    //     }
    //     val destinationFile = destinationPath(context)
    //     if (destinationFile == null) {
    //         context.response().setStatusCode(400).end()
    //         return
    //     }
    //
    //     var overwrite = context.request().getHeader("Overwrite")
    //     if (destinationFile.exists()) {
    //         if (overwrite == null || overwrite.isEmpty()) {
    //             context.response().setStatusCode(412).end()
    //             return
    //         }
    //         destinationFile.deleteRecursively()
    //     }
    //     file.copyRecursively(destinationFile)
    //
    //     context.response().setStatusCode(201).end()
    // }
    pub fn webdav_copy(&self, context: &RoutingContext) {
        let file = self.resolve_webdav_path(context, &Self::request_path(context));
        if file.is_none() {
            context.response().set_status_code(412).end();
            return;
        }
        let file = file.unwrap();
        if !file.exists() {
            context.response().set_status_code(412).end();
            return;
        }
        let destination = context.request().get_header("Destination");
        if destination.is_none() {
            context.response().set_status_code(400).end();
            return;
        }
        let destination_file = self.destination_path(context);
        if destination_file.is_none() {
            context.response().set_status_code(400).end();
            return;
        }
        let destination_file = destination_file.unwrap();

        let overwrite = context.request().get_header("Overwrite");
        if destination_file.exists() {
            if overwrite.is_none() || overwrite.unwrap().is_empty() {
                context.response().set_status_code(412).end();
                return;
            }
            destination_file.delete_recursively();
        }
        file.copy_recursively(&destination_file);

        context.response().set_status_code(201).end();
    }

    // suspend fun webdavLock(context: RoutingContext) {
    //     var response =
    //     """<?xml version="1.0" encoding="utf-8"?>
    //     <D:prop xmlns:D="DAV:">
    //         <D:lockdiscovery>
    //             <D:activelock>
    //                 <D:locktype>
    //                     <write />
    //                 </D:locktype>
    //                 <D:lockscope>
    //                     <exclusive />
    //                 </D:lockscope>
    //                 <D:locktoken>
    //                     <D:href>%s</D:href>
    //                 </D:locktoken>
    //                 <D:lockroot>
    //                     <D:href>%s</D:href>
    //                 </D:lockroot>
    //                 <D:depth>infinity</D:depth>
    //                 <D:owner>
    //                     <a:href xmlns:a="DAV:">http://www.apple.com/webdav_fs/</a:href>
    //                 </D:owner>
    //                 <D:timeout>%s</D:timeout>
    //             </D:activelock>
    //         </D:lockdiscovery>
    //     </D:prop>
    //     """
    //     var lockToken = "urn:uuid:" + UUID.randomUUID().toString()
    //
    //     var timeout = context.request().getHeader("Timeout")
    //     if (timeout == null) {
    //         timeout = "Second-3600"
    //     }
    //
    //     var fileUrl = context.request().absoluteURI()
    //
    //     context.response().putHeader("Lock-Token", lockToken).setStatusCode(200).end(String.format(response, lockToken, fileUrl, timeout))
    // }
    pub fn webdav_lock(&self, context: &RoutingContext) {
        let response = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
        <D:prop xmlns:D=\"DAV:\">\n\
            <D:lockdiscovery>\n\
                <D:activelock>\n\
                    <D:locktype>\n\
                        <write />\n\
                    </D:locktype>\n\
                    <D:lockscope>\n\
                        <exclusive />\n\
                    </D:lockscope>\n\
                    <D:locktoken>\n\
                        <D:href>%s</D:href>\n\
                    </D:locktoken>\n\
                    <D:lockroot>\n\
                        <D:href>%s</D:href>\n\
                    </D:lockroot>\n\
                    <D:depth>infinity</D:depth>\n\
                    <D:owner>\n\
                        <a:href xmlns:a=\"DAV:\">http://www.apple.com/webdav_fs/</a:href>\n\
                    </D:owner>\n\
                    <D:timeout>%s</D:timeout>\n\
                </D:activelock>\n\
            </D:lockdiscovery>\n\
        </D:prop>\n";
        let lock_token = "urn:uuid:".to_string() + &uuid_random().to_string();

        let mut timeout = context.request().get_header("Timeout");
        if timeout.is_none() {
            timeout = Some(String::from("Second-3600"));
        }
        let timeout = timeout.unwrap();

        let file_url = context.request().absolute_uri();

        context.response().put_header("Lock-Token", lock_token.clone()).set_status_code(200).end(format!(response, lock_token, file_url, timeout));
    }

    // suspend fun webdavUnLock(context: RoutingContext) {
    //     var lockToken = context.request().getHeader("Lock-Token")
    //     if (lockToken == null) {
    //         context.response().setStatusCode(400).end()
    //         return
    //     }
    //     context.response().putHeader("Lock-Token", lockToken).setStatusCode(204).end()
    // }
    pub fn webdav_un_lock(&self, context: &RoutingContext) {
        let lock_token = context.request().get_header("Lock-Token");
        if lock_token.is_none() {
            context.response().set_status_code(400).end();
            return;
        }
        let lock_token = lock_token.unwrap();
        context.response().put_header("Lock-Token", lock_token).set_status_code(204).end();
    }

    // suspend fun backupToWebdav(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     if (appConfig.secure) {
    //         var userInfo = context.get("userInfo") as User?
    //         if (userInfo == null) {
    //             return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //         }
    //         if (!userInfo.enable_webdav) {
    //             return returnData.setErrorMsg("未开启webdav功能")
    //         }
    //     }
    //     val bookController = BookController(coroutineContext)
    //
    //     val userNameSpace = getUserNameSpace(context)
    //     val latestZipFilePath = bookController.getLastBackFileFromWebdav(userNameSpace)
    //     if (!bookController.saveToWebdav(userNameSpace, latestZipFilePath)) {
    //         return returnData.setErrorMsg("备份失败")
    //     }
    //     return returnData.setData("")
    // }
    pub fn backup_to_webdav(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if self.base.app_config.secure {
            let user_info = context.get_user::<User>("userInfo");
            if user_info.is_none() {
                return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
            }
            let user_info = user_info.unwrap();
            if !user_info.enable_webdav {
                return return_data.set_error_msg(String::from("未开启webdav功能"));
            }
        }
        let book_controller = BookController::new();

        let user_name_space = self.base.get_user_name_space(context);
        let latest_zip_file_path = book_controller.get_last_back_file_from_webdav(&user_name_space);
        if !book_controller.save_to_webdav(&user_name_space, latest_zip_file_path) {
            return return_data.set_error_msg(String::from("备份失败"));
        }
        return return_data.set_data(Box::new(String::from("")), String::from(""));
    }
}
