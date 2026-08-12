use crate::prelude::*;
// fix: Vertx 位于 stubs 嵌套模块 io::vertx 内，prelude glob 不导出，需显式导入
use crate::stubs::io::vertx::Vertx;
// package com.htmake.reader

// import com.fasterxml.jackson.databind.DeserializationFeature
// import com.fasterxml.jackson.module.kotlin.registerKotlinModule
// import io.vertx.core.Future
// import io.vertx.core.Vertx
// import io.vertx.core.http.*
// import io.vertx.core.json.Json
// import io.vertx.ext.web.client.WebClient
// import io.vertx.ext.web.client.WebClientOptions
// import mu.KotlinLogging
// import com.htmake.reader.api.YueduApi
//
// import com.htmake.reader.verticle.RestVerticle
// import org.springframework.beans.factory.annotation.Autowired
// import org.springframework.boot.SpringApplication
// import org.springframework.boot.autoconfigure.SpringBootApplication
// import org.springframework.scheduling.annotation.EnableScheduling;
// import org.springframework.context.annotation.Bean
// import javax.annotation.PostConstruct

// private val logger = KotlinLogging.logger {}

// @SpringBootApplication
// @EnableScheduling
pub struct ReaderApplication {
    // @Autowired
    // private lateinit var yueduApi: YueduApi
    pub yuedu_api: Option<YueduApi>,
}

impl ReaderApplication {
    // companion object {
    //     val vertx by lazy { Vertx.vertx() }
    //     fun vertx() = vertx
    // }
    pub fn vertx() -> Vertx {
        static VERTX: std::sync::OnceLock<Vertx> = std::sync::OnceLock::new();
        VERTX.get_or_init(|| Vertx::vertx()).clone()
    }

    // @PostConstruct
    pub fn deploy_verticle(&mut self) {
        Json::mapper().apply(&mut |mapper: &mut ObjectMapper| {
            mapper.register_kotlin_module();
        });

        Json::pretty_mapper().apply(&mut |mapper: &mut ObjectMapper| {
            mapper.register_kotlin_module();
        });

        Json::mapper().configure(DeserializationFeature::FAIL_ON_UNKNOWN_PROPERTIES, false);

        Self::vertx().deploy_verticle(self.yuedu_api.as_ref().unwrap());
    }

    // @Bean
    pub fn web_client(&mut self) -> WebClient {
        let mut web_client_options = WebClientOptions::new();
        web_client_options.is_try_use_compression = true;
        web_client_options.log_activity = true;
        web_client_options.is_follow_redirects = true;
        web_client_options.is_trust_all = true;

        let http_client = Self::vertx().create_http_client(HttpClientOptions::new().set_trust_all(true));

        //        val webClient = WebClient.wrap(HttpClient(delegateHttpClient), webClientOptions)
        let web_client = WebClient::wrap(http_client, web_client_options);

        return web_client;
    }
}

pub fn main(args: Vec<String>) {
    logger().info(format!("Starting application with args: {:?}", args));
    let mut app = SpringApplication::new(ReaderApplication::class);
    app.run(args);
}

