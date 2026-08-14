use crate::prelude::*;
use crate::stubs::io::vertx::Route;
use crate::stubs::{
    Authenticator, ConnectionSpec, Credentials, HttpLoggingInterceptor, InetSocketAddress, Level,
    Proxy, ProxyType,
};
// package io.legado.app.help.http
//
// // import io.legado.app.help.http.cronet.CronetInterceptor
// import kotlinx.coroutines.suspendCancellableCoroutine
// import okhttp3.ConnectionSpec
// import okhttp3.Credentials
// import okhttp3.Interceptor
// import okhttp3.OkHttpClient
// import okhttp3.Route
// import okhttp3.Authenticator
// import okhttp3.Response
// import okhttp3.Request
// import okhttp3.logging.HttpLoggingInterceptor
// import java.net.InetSocketAddress
// import java.net.Proxy
// import java.util.concurrent.ConcurrentHashMap
// import java.util.concurrent.TimeUnit
// import kotlin.coroutines.resume
// import java.io.IOException
// import io.legado.app.constant.AppConst
// import io.legado.app.model.DebugLog

// private val proxyClientCache: ConcurrentHashMap<String, OkHttpClient> by lazy {
//     ConcurrentHashMap()
// }
pub static proxy_client_cache: std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<String, OkHttpClient>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

// val okHttpClient: OkHttpClient by lazy {
pub fn ok_http_client() -> OkHttpClient {
    // val specs = arrayListOf(
    //     ConnectionSpec.MODERN_TLS,
    //     ConnectionSpec.COMPATIBLE_TLS,
    //     ConnectionSpec.CLEARTEXT
    // )
    let specs = vec![
        ConnectionSpec::MODERN_TLS,
        ConnectionSpec::COMPATIBLE_TLS,
        ConnectionSpec::CLEARTEXT,
    ];

    // val builder = OkHttpClient.Builder()
    //     .connectTimeout(15, TimeUnit.SECONDS)
    //     .writeTimeout(15, TimeUnit.SECONDS)
    //     .readTimeout(15, TimeUnit.SECONDS)
    //     .sslSocketFactory(SSLHelper.unsafeSSLSocketFactory, SSLHelper.unsafeTrustManager)
    //     .retryOnConnectionFailure(true)
    //     .hostnameVerifier(SSLHelper.unsafeHostnameVerifier)
    //     .connectionSpecs(specs)
    //     .followRedirects(true)
    //     .followSslRedirects(true)
    //     .addInterceptor(Interceptor { chain ->
    //         val request = chain.request()
    //         val builder = request.newBuilder()
    //         if (request.header("User-Agent") == None) {
    //             builder.addHeader("User-Agent", AppConst.userAgent)
    //         } else if (request.header("User-Agent") == "null") {
    //             builder.removeHeader("User-Agent")
    //         }
    //         builder
    //             .addHeader("Keep-Alive", "10")
    //             .addHeader("Connection", "Keep-Alive")
    //             .addHeader("Cache-Control", "no-cache")
    //             .build()
    //         chain.proceed(builder.build())
    //     })
    // fix: E0716——stub 链式方法返回 &Self，先绑定 builder 本体再链式调用，避免临时值悬垂
    let builder = OkHttpClient::builder();
    let builder = builder
        .connect_timeout(15, TimeUnit::SECONDS)
        .write_timeout(15, TimeUnit::SECONDS)
        .read_timeout(15, TimeUnit::SECONDS)
        .ssl_socket_factory(SSLHelper::unsafe_ssl_socket_factory(), SSLHelper::unsafe_trust_manager())
        .retry_on_connection_failure(true)
        .hostname_verifier(SSLHelper::unsafe_hostname_verifier())
        .connection_specs(specs)
        .follow_redirects(true)
        .follow_ssl_redirects(true);
    builder.add_interceptor(Box::new(|chain| {
        let request = chain.request();
        let builder = request.new_builder();
        if request.header("User-Agent").is_none() {
            builder.add_header("User-Agent", AppConst::userAgent());
        } else if request.header("User-Agent") == Some("null") {
            builder.remove_header("User-Agent");
        }
        builder
            .add_header("Keep-Alive", "10")
            .add_header("Connection", "Keep-Alive")
            .add_header("Cache-Control", "no-cache");
        chain.proceed(builder.build())
    }));
    // if (AppConfig.isCronet) {
    //     builder.addInterceptor(CronetInterceptor())
    // }

    // builder.build()
    builder.build()
}

/**
 * 缓存代理okHttp
 */
// fun getProxyClient(proxy: String? = None, debugLog: DebugLog? = None): OkHttpClient {
pub fn get_proxy_client(proxy: Option<&str>, debug_log: Option<&dyn DebugLog>) -> OkHttpClient {
    // if (proxy.isNullOrBlank()) {
    //     if (debugLog == None) {
    //         return okHttpClient
    //     }
    //     val builder = okHttpClient.newBuilder()
    //     val logInterceptor = HttpLoggingInterceptor(debugLog);//创建拦截对象
    //     logInterceptor.setLevel(HttpLoggingInterceptor.Level.BODY);//这一句一定要记得写，否则没有数据输出
    //
    //     builder.addNetworkInterceptor(logInterceptor)  //设置打印拦截日志
    //     return builder.build()
    // }
    let proxy = match proxy {
        Some(proxy) if !proxy.trim().is_empty() => proxy,
        _ => {
            if debug_log.is_none() {
                return ok_http_client();
            }
            let builder = ok_http_client().new_builder();
            // val logInterceptor = HttpLoggingInterceptor(debugLog);//创建拦截对象
            let log_interceptor = HttpLoggingInterceptor::new(debug_log); //创建拦截对象
            log_interceptor.set_level(Level::BODY); //这一句一定要记得写，否则没有数据输出

            builder.add_network_interceptor(log_interceptor); //设置打印拦截日志
            return builder.build();
        }
    };
    // if (debugLog == None) {
    //     proxyClientCache[proxy]?.let {
    //         return it
    //     }
    // }
    if debug_log.is_none() {
        if let Some(it) = proxy_client_cache.lock().unwrap().get(proxy) {
            return it.clone();
        }
    }
    // val r = Regex("(http|socks4|socks5)://(.*):(\\d{2,5})(@.*@.*)?")
    let r = regex::Regex::new("(http|socks4|socks5)://(.*):(\\d{2,5})(@.*@.*)?").unwrap();
    // val ms = r.findAll(proxy)
    let ms = r.find_iter(proxy);
    // val group = ms.first()
    let group = ms.into_iter().next().unwrap();
    let captures = r.captures(proxy).unwrap();
    // var username = ""       //代理服务器验证用户名
    let mut username = ""; //代理服务器验证用户名
    // var password = ""       //代理服务器验证密码
    let mut password = ""; //代理服务器验证密码
    // val type = if (group.groupValues[1] == "http") "http" else "socks"
    let type_ = if &captures[1] == "http" { "http" } else { "socks" };
    // val host = group.groupValues[2]
    let host = captures[2].to_string();
    // val port = group.groupValues[3].toInt()
    let port = captures[3].parse::<i32>().unwrap();
    // if (group.groupValues[4] != "") {
    //     username = group.groupValues[4].split("@")[1]
    //     password = group.groupValues[4].split("@")[2]
    // }
    if captures.get(4).map(|m| m.as_str()).unwrap_or("") != "" {
        username = captures[4].split('@').nth(1).unwrap();
        password = captures[4].split('@').nth(2).unwrap();
    }
    // if (type != "direct" && host != "") {
    if type_ != "direct" && host != "" {
        let mut builder = ok_http_client().new_builder();
        // if (type == "http") {
        //     builder.proxy(Proxy(Proxy.Type.HTTP, InetSocketAddress(host, port)))
        // } else {
        //     builder.proxy(Proxy(Proxy.Type.SOCKS, InetSocketAddress(host, port)))
        // }
        if type_ == "http" {
            builder.proxy(Proxy::new(ProxyType::HTTP, InetSocketAddress::new(&host, port)));
        } else {
            builder.proxy(Proxy::new(ProxyType::SOCKS, InetSocketAddress::new(&host, port)));
        }
        // if (username != "" && password != "") {
        //     val proxyAuthenticator = object: Authenticator {
        //         @Throws(IOException::class)
        //         override fun authenticate(route: Route?, response: Response): Request {
        //             //设置代理服务器账号密码
        //             val credential = Credentials.basic(username, password);
        //             return response.request.newBuilder()
        //                    .header("Proxy-Authorization", credential)
        //                    .build();
        //         }
        //     }
        //     builder.proxyAuthenticator(proxyAuthenticator);
        //     // builder.proxyAuthenticator { _, response -> //设置代理服务器账号密码
        //     //     val credential: String = Credentials.basic(username, password)
        //     //     response.request.newBuilder()
        //     //         .header("Proxy-Authorization", credential)
        //     //         .build()
        //     // }
        // }
        if username != "" && password != "" {
            // fix: 代理认证真实传递（原 Authenticator 闭包被 proxy_authenticator 丢弃）
            let proxy_authenticator = Authenticator::with_credentials(username, password);
            builder.proxy_authenticator(proxy_authenticator);
        }
        // if (debugLog != None) {
        //     val logInterceptor = HttpLoggingInterceptor(debugLog);//创建拦截对象
        //     logInterceptor.setLevel(HttpLoggingInterceptor.Level.BODY);//这一句一定要记得写，否则没有数据输出
        //
        //     builder.addNetworkInterceptor(logInterceptor)  //设置打印拦截日志
        //     return builder.build()
        // }
        if debug_log.is_some() {
            let log_interceptor = HttpLoggingInterceptor::new(debug_log); //创建拦截对象
            log_interceptor.set_level(Level::BODY); //这一句一定要记得写，否则没有数据输出

            builder.add_network_interceptor(log_interceptor); //设置打印拦截日志
            return builder.build();
        }
        // val proxyClient = builder.build()
        let proxy_client = builder.build();
        // proxyClientCache[proxy] = proxyClient
        proxy_client_cache.lock().unwrap().insert(proxy.to_string(), proxy_client.clone());
        return proxy_client;
    }
    // return okHttpClient
    ok_http_client()
}
