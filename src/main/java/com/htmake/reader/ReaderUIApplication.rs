// package com.htmake.reader

// import com.fasterxml.jackson.databind.DeserializationFeature
// import com.fasterxml.jackson.module.kotlin.registerKotlinModule
// import io.vertx.core.Future
// import io.vertx.core.Vertx
// import io.vertx.core.http.*
// import io.vertx.core.http.impl.HttpUtils
// import io.vertx.core.json.Json
// import io.vertx.ext.web.client.WebClient
// import io.vertx.ext.web.client.WebClientOptions
// import io.vertx.core.json.JsonObject
// import mu.KotlinLogging
// import com.htmake.reader.api.YueduApi
// import com.htmake.reader.entity.Size
//
// import com.htmake.reader.verticle.RestVerticle
// import com.htmake.reader.utils.SpringContextUtils
// import com.htmake.reader.SpringEvent
//
// import com.htmake.reader.utils.getStorage
// import com.htmake.reader.utils.saveStorage
// import com.htmake.reader.utils.asJsonObject
//
// import org.springframework.beans.factory.annotation.Autowired
// import org.springframework.boot.SpringApplication
// import org.springframework.boot.autoconfigure.SpringBootApplication
// import org.springframework.boot.context.event.ApplicationEnvironmentPreparedEvent
// import org.springframework.boot.context.event.ApplicationReadyEvent
// import org.springframework.context.annotation.Bean
// import org.springframework.context.ConfigurableApplicationContext
// import org.springframework.context.ApplicationListener
// import org.springframework.context.ApplicationEvent
// import uk.org.lidalia.sysoutslf4j.context.SysOutOverSLF4J
// import javax.annotation.PostConstruct
//
// import javafx.application.Application
// import javafx.application.Platform
// import javafx.scene.Scene
// import javafx.scene.web.WebView
// import javafx.scene.web.WebErrorEvent
// import javafx.stage.Stage
// import javafx.stage.WindowEvent
// import javafx.stage.StageStyle
// import javafx.event.EventHandler
// import com.sun.javafx.application.LauncherImpl
// import com.sun.javafx.scene.text.FontHelper
// import javafx.scene.text.Font
//
// import javafx.scene.control.ProgressBar
// import javafx.scene.control.Dialog
// import javafx.scene.control.ButtonType
// import javafx.scene.image.ImageView
// import javafx.scene.layout.VBox
// import javafx.scene.paint.Color;
// import javafx.scene.image.Image;
// import javafx.beans.value.ChangeListener
// import javafx.beans.value.ObservableValue
// import javafx.concurrent.Worker
//
// import org.springframework.core.env.Environment
// import org.springframework.core.env.ConfigurableEnvironment
// import org.springframework.core.env.MapPropertySource
//
// import java.util.concurrent.CompletableFuture

// private val logger = KotlinLogging.logger {}
// private var launchArgs = arrayOf<String>()

pub struct ReaderUIApplication {
    // private lateinit var primaryStage: Stage;
    pub primary_stage: Option<Stage>,
    // private lateinit var splashStage: Stage;
    pub splash_stage: Option<Stage>,

    pub web_url: String,
    pub env: ConfigurableEnvironment,
    pub window_config_map: std::collections::HashMap<String, Any>,

    pub is_spring_boot_launched: bool,
    pub spring_boot_error: String,
    pub show_ui: bool,

    pub default_icons: Vec<Image>,
}

impl ReaderUIApplication {
    pub fn new() -> Self {
        ReaderUIApplication {
            primary_stage: None,
            splash_stage: None,
            web_url: String::new(),
            env: ConfigurableEnvironment::new(),
            window_config_map: std::collections::HashMap::new(),
            is_spring_boot_launched: false,
            spring_boot_error: String::new(),
            show_ui: false,
            default_icons: Vec::new(),
        }
    }

    pub fn boot(&mut self) {
        launch(launch_args());
    }

    // override fun init()
    pub fn init(&mut self) {
        std::thread::spawn(move || {
            let mut app = SpringApplication::new(ReaderApplication::class);
            let env_listener = ApplicationListener::new(move |event: ApplicationEnvironmentPreparedEvent| {
                self.env = event.get_environment();
                // 加载 windowConfig
                let window_config_source = self.load_property_source_from_window_config();
                self.env.get_property_sources().add_first(window_config_source);
                // 获取应用相关配置
                self.show_ui = self.env.get_property("reader.app.showUI", Boolean::class)?.unwrap_or(false);
                logger.info("showUI: {:?}", self.show_ui);
                let debug = self.env.get_property::<Boolean>("reader.app.debug", Boolean::class);
                logger.info("debug: {:?}", debug);
                let server_port = self.env.get_property::<Int>("reader.server.port", Int::class);
                logger.info("serverPort: {:?}", server_port);
                let mut port = 8080;
                if server_port != null && server_port > 0 {
                    port = server_port;
                }
                self.web_url = self.env.get_property::<String>("reader.server.webUrl")?.unwrap_or_else(|| "http://localhost:".to_string() + &port.to_string());
                let sep = if self.web_url.contains("?") { "&" } else { "?" };
                if debug != null && debug {
                    self.web_url = self.web_url + sep + "debug=1&nopwa=1";
                } else {
                    self.web_url = self.web_url + sep + "nopwa=1";
                }
                logger.info("webUrl: {:?}", self.web_url);
                // System.setProperty("reader.system.fonts", Font.getFontNames().joinToString(separator = ","))
                if self.show_ui && self.primary_stage.is_some() {
                    Platform::run_later(Runnable::new(|| {
                        self.show_splash_screen();
                    }));
                }
            });
            app.add_listeners(env_listener);
            let spring_listener = ApplicationListener::new(move |event: SpringEvent| {
                let event_type = event.get_event();
                if event_type == "READY" {
                    self.is_spring_boot_launched = true;
                    if self.show_ui && self.primary_stage.is_some() && !self.web_url.is_empty() {
                        Platform::run_later(Runnable::new(|| {
                            self.splash_stage.hide();
                            self.splash_stage.set_scene(None);
                            self.show_web_screen(self.primary_stage.clone().unwrap(), self.web_url.clone());
                        }));
                    }
                } else if event_type == "START_ERROR" {
                    self.spring_boot_error = event.get_message();
                    if self.show_ui {
                        Platform::run_later(Runnable::new(|| {
                            if self.splash_stage.is_some() {
                                self.splash_stage.hide();
                                self.splash_stage.set_scene(None);
                            }
                            self.show_alert(self.spring_boot_error.clone());
                            self.stop();
                        }));
                    } else {
                        logger.error(&self.spring_boot_error);
                        self.stop();
                    }
                }
            });
            app.add_listeners(spring_listener);
            app.run(launch_args());
        });
    }

    // override fun start(stage: Stage)
    pub fn start(&mut self, stage: Stage) {
        try {
            logger.info("javafx start: {:?}", stage);
            self.primary_stage = Some(stage.clone());
            if self.show_ui {
                self.default_icons = vec![
                    Image::new(ReaderUIApplication::class.get_resource("/icons/16x16.png").to_external_form()),
                    Image::new(ReaderUIApplication::class.get_resource("/icons/24x24.png").to_external_form()),
                    Image::new(ReaderUIApplication::class.get_resource("/icons/32x32.png").to_external_form()),
                    Image::new(ReaderUIApplication::class.get_resource("/icons/48x48.png").to_external_form()),
                    Image::new(ReaderUIApplication::class.get_resource("/icons/64x64.png").to_external_form()),
                    Image::new(ReaderUIApplication::class.get_resource("/icons/128x128.png").to_external_form()),
                ];
                if self.is_spring_boot_launched {
                    self.show_web_screen(stage, self.web_url.clone());
                } else {
                    if !self.spring_boot_error.is_empty() {
                        self.show_alert(self.spring_boot_error.clone());
                        self.stop();
                    } else {
                        self.show_splash_screen();
                    }
                }
            }
        } catch (e: Exception) {
            e.printStackTrace();
        }
    }

    pub fn show_splash_screen(&mut self) {
        self.splash_stage = Some(Stage::new());
        let image_view = ImageView::new(ReaderUIApplication::class.get_resource("/images/loading.gif").to_external_form());
        // var splashProgressBar = ProgressBar();
        // splashProgressBar.setPrefWidth(imageView.getImage().getWidth());
        // splashProgressBar.setPrefHeight(10.0);

        let vbox = VBox::new();
        vbox.get_children().add_all(vec![image_view]);
        // vbox.setStyle("-fx-background-color: transparent;" +
        //               "-fx-padding: 0;" +
        //               "-fx-border-style: solid inside;" +
        //               "-fx-border-width: 1;" +
        //               "-fx-border-insets: 0;" +
        //               "-fx-border-radius: 0;" +
        //               "-fx-border-color: #999;");

        let splash_scene = Scene::new(vbox, Color::TRANSPARENT);
        self.splash_stage.as_ref().unwrap().set_scene(Some(splash_scene));
        self.splash_stage.as_ref().unwrap().get_icons().add_all(self.default_icons.clone());
        self.splash_stage.as_ref().unwrap().init_style(StageStyle::TRANSPARENT);
        logger.info("showSplashScreen: {:?}", self.splash_stage);
        self.splash_stage.as_ref().unwrap().show();
    }

    pub fn show_alert(&mut self, message: String, wait: bool) {
        let mut alert = Dialog::new();
        alert.get_dialog_pane().set_content_text(message);
        alert.get_dialog_pane().get_button_types().add(ButtonType::OK);
        if wait {
            alert.show_and_wait();
        } else {
            alert.show();
        }
    }

    pub fn show_confirm(&mut self, message: String) -> bool {
        let mut confirm = Dialog::new();
        confirm.get_dialog_pane().set_content_text(message);
        confirm.get_dialog_pane().get_button_types().add_all(vec![ButtonType::YES, ButtonType::NO]);
        let result = confirm.show_and_wait().filter(|b| ButtonType::YES == b).is_present();

        return result;
    }

    pub fn load_property_source_from_window_config(&mut self) -> MapPropertySource {
        self.load_window_config();
        let mut window_config_port = 0;
        let mut window_config_source: std::collections::HashMap<String, Any> = std::collections::HashMap::new();
        try {
            // 支持配置 接口服务
            let server_config = self.window_config_map.get("serverConfig").map(|v| v.downcast_ref::<std::collections::HashMap<String, Any>>()).flatten().cloned();
            if let Some(sc) = server_config {
                window_config_source = sc;
            }

            let server_port = self.window_config_map.get("serverPort");
            if server_port.is_some() {
                window_config_port = server_port.downcast_ref::<Int>().unwrap().clone();
                if window_config_port > 0 {
                    window_config_source.insert("reader.server.port".to_string(), window_config_port.clone());
                }
            }
            let show_ui = self.window_config_map.get("showUI").unwrap_or(&true).downcast_ref::<Boolean>().cloned().unwrap_or(true);
            window_config_source.insert("reader.app.showUI".to_string(), show_ui);
            let debug = self.window_config_map.get("debug");
            if debug.is_some() {
                window_config_source.insert("reader.app.debug".to_string(), debug.downcast_ref::<Boolean>().unwrap().clone());
            }
        } catch (e: Exception) {
            e.printStackTrace();
        }

        logger.info("windowConfigSource: {:?}", window_config_source);
        return MapPropertySource::new("windowConfig", window_config_source);
    }

    pub fn load_window_config(&mut self) {
        let window_config_object = as_json_object(get_storage(vec!["windowConfig".to_string()]));
        if window_config_object.is_some() {
            self.window_config_map = window_config_object.unwrap().map;
        }
        logger.info("windowConfigMap: {:?}", self.window_config_map);
    }

    pub fn get_window_config_double_property(&self, name: String, default_val: f64) -> f64 {
        let value = self.window_config_map.get(&name).unwrap_or(&default_val);
        return match value {
            Int => value.to_double(),
            Double => value,
            _ => default_val,
        };
    }

    pub fn apply_window_config(&mut self, stage: &mut Stage) -> Size {
        let mut width = 1280.0;
        let mut height = 800.0;
        try {
            self.load_window_config();
            let set_window_position = self.window_config_map.get("setWindowPosition").unwrap_or(&false).downcast_ref::<Boolean>().cloned().unwrap_or(false);
            if set_window_position {
                let position_x = self.get_window_config_double_property("positionX".to_string(), 0.0);
                let position_y = self.get_window_config_double_property("positionY".to_string(), 0.0);
                stage.set_x(position_x);
                stage.set_y(position_y);
            }
            let remember_size = self.window_config_map.get("rememberSize").unwrap_or(&true).downcast_ref::<Boolean>().cloned().unwrap_or(true);
            let remember_position = self.window_config_map.get("rememberPosition").unwrap_or(&false).downcast_ref::<Boolean>().cloned().unwrap_or(false);
            if remember_size {
                stage.width_property().add_listener(|_, _, w| {
                    self.window_config_map.insert("width".to_string(), w);
                });
                // stage.heightProperty().addListener{_, _, h ->
                //     windowConfigMap.put("height", h)
                // }
                stage.scene_property().add_listener(|_, _, s| {
                    s.height_property().add_listener(|_, _, h| {
                        self.window_config_map.insert("height".to_string(), h);
                    });
                });
            }
            if remember_position {
                stage.x_property().add_listener(|_, _, x| {
                    self.window_config_map.insert("positionX".to_string(), x);
                });
                stage.y_property().add_listener(|_, _, y| {
                    self.window_config_map.insert("positionY".to_string(), y);
                });
            }
            let set_window_size = self.window_config_map.get("setWindowSize").unwrap_or(&true).downcast_ref::<Boolean>().cloned().unwrap_or(true);
            if set_window_size {
                width = self.get_window_config_double_property("width".to_string(), width);
                height = self.get_window_config_double_property("height".to_string(), height);
            }
        } catch (e: Exception) {
            self.show_alert("窗口配置加载失败，请检查窗口配置文件(windowConfig.json)".to_string(), false);
            e.printStackTrace();
        }
        return Size::new(width, height);
    }

    pub fn show_web_screen(&mut self, stage: &mut Stage, url: String) {
        // 配置主窗口
        let window_size = self.apply_window_config(stage);
        System::set_property("sun.net.http.allowRestrictedHeaders", "true");
        // logger.info("Font.getFontNames: {}", Font.getFontNames())
        // logger.info("showWebScreen: {}", url)
        let mut web_view = WebView::new();
        let mut web_engine = web_view.get_engine();
        web_engine.set_on_error(|event| {
            logger.info("error: {:?}", event);
        });
        web_engine.set_on_alert(|event| {
            self.show_alert(event.data.to_string(), true);
        });
        web_engine.set_confirm_handler(|message| {
            self.show_confirm(message)
        });
        let mut reload_count = 0;
        web_engine.get_load_worker().state_property().add_listener(|_, old_state, new_state| {
            logger.info("State from {:?} to {:?} , exception: {:?}", old_state, new_state, web_engine.get_load_worker().get_exception());
            if new_state == Worker::State::FAILED {
                if reload_count < 5 {
                    reload_count += 1;
                    logger.info("reload {:?}", url);
                    web_engine.load(url.clone());
                }
            }
        });
        web_engine.title_property().add_listener(|_, _, t| {
            if t.is_some() && !t.unwrap().is_empty() {
                stage.set_title(t.unwrap());
            }
        });
        web_engine.load(url);
        let scene = Scene::new(web_view, window_size.width, window_size.height);
        stage.set_scene(scene);
        stage.set_title("阅读");
        stage.get_icons().add_all(self.default_icons.clone());
        stage.init_style(StageStyle::UNIFIED);
        stage.show();
    }

    // override fun stop()
    pub fn stop(&mut self) {
        save_storage(vec!["windowConfig".to_string()], self.window_config_map.clone(), true);
        super_stop();
        let context = SpringContextUtils::get_application_context();
        logger.info("application stop: {:?}", context);
        System::exit(SpringApplication::exit(context));
    }
}

pub fn main(args: Vec<String>) {
    logger.info("args: {:?}", args);
    set_launch_args(args);
    let mut app = ReaderUIApplication::new();
    app.boot();
}

