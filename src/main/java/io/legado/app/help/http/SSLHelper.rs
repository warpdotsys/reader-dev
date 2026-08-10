// package io.legado.app.help.http
//
// //import android.annotation.SuppressLint
// import java.io.IOException
// import java.io.InputStream
// import java.security.KeyManagementException
// import java.security.KeyStore
// import java.security.NoSuchAlgorithmException
// import java.security.SecureRandom
// import java.security.cert.CertificateException
// import java.security.cert.CertificateFactory
// import java.security.cert.X509Certificate
// import javax.net.ssl.*

// object SSLHelper {
pub struct SSLHelper;

impl SSLHelper {
    // val sslSocketFactory: SSLParams?
    //     get() = getSslSocketFactoryBase(null, null, null)
    pub fn ssl_socket_factory() -> Option<SSLParams> {
        get_ssl_socket_factory_base(None, None, None, &[])
    }

    /**
     * 为了解决客户端不信任服务器数字证书的问题，网络上大部分的解决方案都是让客户端不对证书做任何检查，
     * 这是一种有很大安全漏洞的办法
     */
    // val unsafeTrustManager: X509TrustManager = object : X509TrustManager {
    // //        @SuppressLint("TrustAllX509TrustManager")
    //     @Throws(CertificateException::class)
    //     override fun checkClientTrusted(chain: Array<X509Certificate>, authType: String) {
    //     }
    //
    // //        @SuppressLint("TrustAllX509TrustManager")
    //     @Throws(CertificateException::class)
    //     override fun checkServerTrusted(chain: Array<X509Certificate>, authType: String) {
    //     }
    //
    //     override fun getAcceptedIssuers(): Array<X509Certificate> {
    //         return arrayOf()
    //     }
    // }
    pub fn unsafe_trust_manager() -> X509TrustManager {
        X509TrustManager::new(
            // checkClientTrusted(chain, authType) {}
            Box::new(|_chain, _auth_type| {}),
            // checkServerTrusted(chain, authType) {}
            Box::new(|_chain, _auth_type| {}),
            // getAcceptedIssuers() { arrayOf() }
            Box::new(|| vec![]),
        )
    }

    // val unsafeSSLSocketFactory: SSLSocketFactory by lazy {
    //     try {
    //         val sslContext = SSLContext.getInstance("SSL")
    //         sslContext.init(null, arrayOf(unsafeTrustManager), SecureRandom())
    //         sslContext.socketFactory
    //     } catch (e: Exception) {
    //         throw RuntimeException(e)
    //     }
    // }
    pub fn unsafe_ssl_socket_factory() -> SSLSocketFactory {
        use std::sync::OnceLock;
        static UNSAFE_SSL_SOCKET_FACTORY: OnceLock<SSLSocketFactory> = OnceLock::new();
        UNSAFE_SSL_SOCKET_FACTORY.get_or_init(|| {
            // try {
            //     val sslContext = SSLContext.getInstance("SSL")
            //     sslContext.init(null, arrayOf(unsafeTrustManager), SecureRandom())
            //     sslContext.socketFactory
            // } catch (e: Exception) {
            //     throw RuntimeException(e)
            // }
            match (|| -> Result<SSLSocketFactory, Box<dyn std::error::Error>> {
                let ssl_context = SSLContext::get_instance("SSL")?;
                let factory = ssl_context.init(None, vec![unsafe_trust_manager()], SecureRandom::new())?.socket_factory();
                Ok(factory)
            })() {
                Ok(factory) => factory,
                Err(e) => panic!("{:?}", e),
            }
        })
        .clone()
    }

    /**
     * 此类是用于主机名验证的基接口。 在握手期间，如果 URL 的主机名和服务器的标识主机名不匹配，
     * 则验证机制可以回调此接口的实现程序来确定是否应该允许此连接。策略可以是基于证书的或依赖于其他验证方案。
     * 当验证 URL 主机名使用的默认规则失败时使用这些回调。如果主机名是可接受的，则返回 true
     */
    // val unsafeHostnameVerifier: HostnameVerifier = HostnameVerifier { _, _ -> true }
    pub fn unsafe_hostname_verifier() -> HostnameVerifier {
        // HostnameVerifier { _, _ -> true }
        HostnameVerifier::new(Box::new(|_, _| true))
    }

    /**
     * https单向认证
     * 可以额外配置信任服务端的证书策略，否则默认是按CA证书去验证的，若不是CA可信任的证书，则无法通过验证
     */
    // fun getSslSocketFactory(trustManager: X509TrustManager): SSLParams? {
    //     return getSslSocketFactoryBase(trustManager, null, null)
    // }
    pub fn get_ssl_socket_factory(trust_manager: X509TrustManager) -> Option<SSLParams> {
        get_ssl_socket_factory_base(Some(trust_manager), None, None, &[])
    }

    /**
     * https单向认证
     * 用含有服务端公钥的证书校验服务端证书
     */
    // fun getSslSocketFactory(vararg certificates: InputStream): SSLParams? {
    //     return getSslSocketFactoryBase(null, null, null, *certificates)
    // }
    pub fn get_ssl_socket_factory_certificates(certificates: &[InputStream]) -> Option<SSLParams> {
        get_ssl_socket_factory_base(None, None, None, certificates)
    }

    /**
     * https双向认证
     * bksFile 和 password -> 客户端使用bks证书校验服务端证书
     * certificates -> 用含有服务端公钥的证书校验服务端证书
     */
    // fun getSslSocketFactory(bksFile: InputStream, password: String, vararg certificates: InputStream): SSLParams? {
    //     return getSslSocketFactoryBase(null, bksFile, password, *certificates)
    // }
    pub fn get_ssl_socket_factory_bks(
        bks_file: InputStream,
        password: &str,
        certificates: &[InputStream],
    ) -> Option<SSLParams> {
        get_ssl_socket_factory_base(None, Some(bks_file), Some(password.to_string()), certificates)
    }

    /**
     * https双向认证
     * bksFile 和 password -> 客户端使用bks证书校验服务端证书
     * X509TrustManager -> 如果需要自己校验，那么可以自己实现相关校验，如果不需要自己校验，那么传null即可
     */
    // fun getSslSocketFactory(bksFile: InputStream, password: String, trustManager: X509TrustManager): SSLParams? {
    //     return getSslSocketFactoryBase(trustManager, bksFile, password)
    // }
    pub fn get_ssl_socket_factory_bks_trust(
        bks_file: InputStream,
        password: &str,
        trust_manager: X509TrustManager,
    ) -> Option<SSLParams> {
        get_ssl_socket_factory_base(Some(trust_manager), Some(bks_file), Some(password.to_string()), &[])
    }

    // private fun getSslSocketFactoryBase(
    //     trustManager: X509TrustManager?,
    //     bksFile: InputStream?,
    //     password: String?,
    //     vararg certificates: InputStream
    // ): SSLParams? {
    //     val sslParams = SSLParams()
    //     try {
    //         val keyManagers = prepareKeyManager(bksFile, password)
    //         val trustManagers = prepareTrustManager(*certificates)
    //         val manager: X509TrustManager = trustManager ?: chooseTrustManager(trustManagers)
    //         // 创建TLS类型的SSLContext对象， that uses our TrustManager
    //         val sslContext = SSLContext.getInstance("TLS")
    //         // 用上面得到的trustManagers初始化SSLContext，这样sslContext就会信任keyStore中的证书
    //         // 第一个参数是授权的密钥管理器，用来授权验证，比如授权自签名的证书验证。第二个是被授权的证书管理器，用来验证服务器端的证书
    //         sslContext.init(keyManagers, arrayOf<TrustManager>(manager), null)
    //         // 通过sslContext获取SSLSocketFactory对象
    //         sslParams.sSLSocketFactory = sslContext.socketFactory
    //         sslParams.trustManager = manager
    //         return sslParams
    //     } catch (e: NoSuchAlgorithmException) {
    //         e.printStackTrace()
    //     } catch (e: KeyManagementException) {
    //         e.printStackTrace()
    //     }
    //     return null
    // }
    fn get_ssl_socket_factory_base(
        trust_manager: Option<X509TrustManager>,
        bks_file: Option<InputStream>,
        password: Option<String>,
        certificates: &[InputStream],
    ) -> Option<SSLParams> {
        let mut ssl_params = SSLParams::default();
        // try {
        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            let key_managers = prepare_key_manager(bks_file.as_ref(), password.as_deref());
            let trust_managers = prepare_trust_manager(certificates);
            // val manager: X509TrustManager = trustManager ?: chooseTrustManager(trustManagers)
            let manager: X509TrustManager = trust_manager.unwrap_or_else(|| choose_trust_manager(&trust_managers));
            // 创建TLS类型的SSLContext对象， that uses our TrustManager
            let ssl_context = SSLContext::get_instance("TLS")?;
            // 用上面得到的trustManagers初始化SSLContext，这样sslContext就会信任keyStore中的证书
            // 第一个参数是授权的密钥管理器，用来授权验证，比如授权自签名的证书验证。第二个是被授权的证书管理器，用来验证服务器端的证书
            ssl_context.init(key_managers, vec![Box::new(manager.clone())], None)?;
            // 通过sslContext获取SSLSocketFactory对象
            ssl_params.s_ssl_socket_factory = ssl_context.socket_factory();
            ssl_params.trust_manager = manager;
            Ok(())
        })();
        // } catch (e: NoSuchAlgorithmException) {
        //     e.printStackTrace()
        // } catch (e: KeyManagementException) {
        //     e.printStackTrace()
        // }
        match result {
            // return sslParams
            Ok(()) => Some(ssl_params),
            // return null
            Err(e) => {
                e.print_stack_trace();
                None
            }
        }
    }

    // private fun prepareKeyManager(bksFile: InputStream?, password: String?): Array<KeyManager>? {
    //     try {
    //         if (bksFile == null || password == null) return null
    //         val clientKeyStore = KeyStore.getInstance("BKS")
    //         clientKeyStore.load(bksFile, password.toCharArray())
    //         val kmf = KeyManagerFactory.getInstance(KeyManagerFactory.getDefaultAlgorithm())
    //         kmf.init(clientKeyStore, password.toCharArray())
    //         return kmf.keyManagers
    //     } catch (e: Exception) {
    //         e.printStackTrace()
    //     }
    //     return null
    // }
    fn prepare_key_manager(bks_file: Option<&InputStream>, password: Option<&str>) -> Option<Vec<KeyManager>> {
        // try {
        //     if (bksFile == null || password == null) return null
        //     ...
        // } catch (e: Exception) {
        //     e.printStackTrace()
        // }
        // return null
        match (|| -> Result<Vec<KeyManager>, Box<dyn std::error::Error>> {
            let bks_file = match bks_file {
                Some(bks_file) => bks_file,
                None => return Ok(vec![]),
            };
            let password = match password {
                Some(password) => password,
                None => return Ok(vec![]),
            };
            let client_key_store = KeyStore::get_instance("BKS")?;
            client_key_store.load(Some(bks_file), &password.chars().collect::<Vec<_>>())?;
            let kmf = KeyManagerFactory::get_instance(KeyManagerFactory::get_default_algorithm())?;
            kmf.init(&client_key_store, &password.chars().collect::<Vec<_>>())?;
            Ok(kmf.key_managers())
        })() {
            Ok(managers) => Some(managers),
            Err(e) => {
                e.print_stack_trace();
                None
            }
        }
    }

    // private fun prepareTrustManager(vararg certificates: InputStream): Array<TrustManager> {
    //     val certificateFactory = CertificateFactory.getInstance("X.509")
    //     // 创建一个默认类型的KeyStore，存储我们信任的证书
    //     val keyStore = KeyStore.getInstance(KeyStore.getDefaultType())
    //     keyStore.load(null)
    //     for ((index, certStream) in certificates.withIndex()) {
    //         val certificateAlias = Integer.toString(index)
    //         // 证书工厂根据证书文件的流生成证书 cert
    //         val cert = certificateFactory.generateCertificate(certStream)
    //         // 将 cert 作为可信证书放入到keyStore中
    //         keyStore.setCertificateEntry(certificateAlias, cert)
    //         try {
    //             certStream.close()
    //         } catch (e: IOException) {
    //             e.printStackTrace()
    //         }
    //     }
    //     //我们创建一个默认类型的TrustManagerFactory
    //     val tmf = TrustManagerFactory.getInstance(TrustManagerFactory.getDefaultAlgorithm())
    //     //用我们之前的keyStore实例初始化TrustManagerFactory，这样tmf就会信任keyStore中的证书
    //     tmf.init(keyStore)
    //     //通过tmf获取TrustManager数组，TrustManager也会信任keyStore中的证书
    //     return tmf.trustManagers
    // }
    fn prepare_trust_manager(certificates: &[InputStream]) -> Vec<TrustManager> {
        let certificate_factory = CertificateFactory::get_instance("X.509");
        // 创建一个默认类型的KeyStore，存储我们信任的证书
        let key_store = KeyStore::get_instance(KeyStore::get_default_type());
        key_store.load(None);
        for (index, cert_stream) in certificates.iter().enumerate() {
            let certificate_alias = index.to_string();
            // 证书工厂根据证书文件的流生成证书 cert
            let cert = certificate_factory.generate_certificate(cert_stream);
            // 将 cert 作为可信证书放入到keyStore中
            key_store.set_certificate_entry(&certificate_alias, cert);
            // try {
            //     certStream.close()
            // } catch (e: IOException) {
            //     e.printStackTrace()
            // }
            if let Err(e) = cert_stream.close() {
                e.print_stack_trace();
            }
        }
        //我们创建一个默认类型的TrustManagerFactory
        let tmf = TrustManagerFactory::get_instance(TrustManagerFactory::get_default_algorithm());
        //用我们之前的keyStore实例初始化TrustManagerFactory，这样tmf就会信任keyStore中的证书
        tmf.init(&key_store);
        //通过tmf获取TrustManager数组，TrustManager也会信任keyStore中的证书
        tmf.trust_managers()
    }

    // private fun chooseTrustManager(trustManagers: Array<TrustManager>): X509TrustManager {
    //     for (trustManager in trustManagers) {
    //         if (trustManager is X509TrustManager) {
    //             return trustManager
    //         }
    //     }
    //     throw NullPointerException()
    // }
    fn choose_trust_manager(trust_managers: &[TrustManager]) -> X509TrustManager {
        for trust_manager in trust_managers {
            if let Some(trust_manager) = trust_manager.downcast_ref::<X509TrustManager>() {
                return trust_manager.clone();
            }
        }
        panic!("NullPointerException")
    }
}

// class SSLParams {
//     lateinit var sSLSocketFactory: SSLSocketFactory
//     lateinit var trustManager: X509TrustManager
// }
#[derive(Default)]
pub struct SSLParams {
    pub s_ssl_socket_factory: SSLSocketFactory,
    pub trust_manager: X509TrustManager,
}
