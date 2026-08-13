use crate::prelude::*;
// package com.htmake.reader.lib.tts.service;

// import com.htmake.reader.lib.tts.constant.OutputFormat;
// import com.htmake.reader.lib.tts.constant.TtsConstants;
// import com.htmake.reader.lib.tts.exceptions.TtsException;
// import com.htmake.reader.lib.tts.model.SSML;
// import com.htmake.reader.lib.tts.model.SpeechConfig;
// import com.htmake.reader.lib.tts.util.Tools;
// import okhttp3.OkHttpClient;
// import okhttp3.Request;
// import okhttp3.Response;
// import okhttp3.WebSocket;
// import okhttp3.WebSocketListener;
// import okio.Buffer;
// import okio.ByteString;
// import org.slf4j.Logger;
// import org.slf4j.LoggerFactory;

// import java.nio.charset.StandardCharsets;
// import java.util.Objects;
// import java.util.concurrent.CountDownLatch;
// import java.util.concurrent.TimeUnit;

// public class TTSService {
pub struct TTSService {
    // public static final Logger log = LoggerFactory.getLogger(TTSService.class);

    // private OutputFormat outputFormat;
    // private boolean usingAzureApi;
    // private volatile boolean synthesising;
    // private String currentText;
    // private final Buffer audioBuffer;
    // private OkHttpClient okHttpClient;
    // private WebSocket ws;
    // private CountDownLatch latch;
    // protected WebSocketListener webSocketListener;
    // fix: 监听器改为 Rc<dyn Fn(&mut TTSService, WebSocketEvent)> —— 闭包需修改自身字段，逻辑委托 handle_ws_event；Rc 供 clone
    pub output_format: Option<OutputFormat>,
    pub using_azure_api: bool,
    pub synthesising: bool,
    pub current_text: Option<String>,
    pub audio_buffer: Buffer,
    pub ok_http_client: Option<OkHttpClient>,
    pub ws: Option<WebSocket>,
    pub latch: Option<CountDownLatch>,
    pub web_socket_listener: Option<Rc<dyn Fn(&mut TTSService, WebSocketEvent)>>,
}

impl TTSService {
    // private TTSService(OutputFormat outputFormat, boolean usingAzureApi) {
    pub fn new(output_format: Option<OutputFormat>, using_azure_api: bool) -> TTSService {
        let audio_buffer = Buffer::new();
        // fix: 匿名 WebSocketListener 无法直接转录（闭包捕获构造中的字段），逻辑移入 handle_ws_event 方法
        let web_socket_listener: Option<Rc<dyn Fn(&mut TTSService, WebSocketEvent)>> = Some(Rc::new(|service: &mut TTSService, event: WebSocketEvent| {
            service.handle_ws_event(event)
        }));
        TTSService {
            output_format,
            using_azure_api,
            synthesising: false,
            current_text: None,
            audio_buffer,
            ok_http_client: None,
            ws: None,
            latch: None,
            web_socket_listener,
        }
    }

    // WebSocketListener 匿名子类回调逻辑（原构造器内 new WebSocketListener(){...}）
    fn handle_ws_event(&mut self, event: WebSocketEvent) {
        match event {
            // @Override
            // public void onClosed(WebSocket webSocket, int code, String reason) {
            //     super.onClosed(webSocket, code, reason);
            //     log.debug("onClosed:" + reason);
            //     ws = None;
            //     synthesising = false;
            // }
            WebSocketEvent::onClosed(reason) => {
                log::debug(format!("onClosed:{}", reason));
                self.ws = None;
                self.synthesising = false;
            }
            // @Override
            // public void onClosing(WebSocket webSocket, int code, String reason) {
            //     super.onClosing(webSocket, code, reason);
            //     log.debug("onClosing:" + reason);
            //     ws = None;
            //     synthesising = false;
            // }
            WebSocketEvent::onClosing(reason) => {
                log::debug(format!("onClosing:{}", reason));
                self.ws = None;
                self.synthesising = false;
            }
            // @Override
            // public void onFailure(WebSocket webSocket, Throwable t, Response response) {
            //     super.onFailure(webSocket, t, response);
            //     log.debug("onFailure" + t.getMessage(), t);
            //     ws = None;
            //     synthesising = false;
            // }
            WebSocketEvent::onFailure(message) => {
                log::debug(format!("onFailure{}", message));
                self.ws = None;
                self.synthesising = false;
            }
            // @Override
            // public void onMessage(WebSocket webSocket, String text) {
            //     super.onMessage(webSocket, text);
            //     if (text.contains(TtsConstants.TURN_START)) {
            //         audioBuffer.clear();
            //     } else if (text.contains(TtsConstants.TURN_END)) {
            //         latch.countDown();
            //         synthesising = false;
            //     }
            // }
            WebSocketEvent::onMessageText(text) => {
                if text.contains(TtsConstants::TURN_START) {
                    self.audio_buffer.clear();
                } else if text.contains(TtsConstants::TURN_END) {
                    self.latch.as_ref().unwrap().count_down();
                    self.synthesising = false;
                }
            }
            // @Override
            // public void onMessage(WebSocket webSocket, ByteString bytes) {
            //     super.onMessage(webSocket, bytes);
            //     int audioIndex = bytes.lastIndexOf(
            //         TtsConstants.AUDIO_START.getBytes(StandardCharsets.UTF_8)
            //     ) + TtsConstants.AUDIO_START.length();
            //     boolean hasContentType = bytes.lastIndexOf(
            //         TtsConstants.AUDIO_CONTENT_TYPE.getBytes(StandardCharsets.UTF_8)
            //     ) + TtsConstants.AUDIO_CONTENT_TYPE.length() != -1;
            //     if (audioIndex != -1 && hasContentType) {
            //         try {
            //             audioBuffer.write(bytes.substring(audioIndex));
            //         } catch (Exception e) {
            //             log.error("onMessage Error," + e.getMessage(), e);
            //         }
            //     }
            // }
            WebSocketEvent::onMessageBytes(bytes) => {
                // fix: len() 为 usize → as i32（Java 中为 int）
                let audio_index = bytes.last_index_of(TtsConstants::AUDIO_START.as_bytes()) + TtsConstants::AUDIO_START.len() as i32;
                let has_content_type = bytes.last_index_of(TtsConstants::AUDIO_CONTENT_TYPE.as_bytes()) + TtsConstants::AUDIO_CONTENT_TYPE.len() as i32 != -1;
                if audio_index != -1 && has_content_type {
                    // fix: try/catch → 闭包 + if-let（catch 仅记录日志）
                    let try_result: Result<(), StubError> = (|this: &mut Self| {
                        this.audio_buffer.write(&bytes.substring(audio_index as usize));
                        Ok(())
                    })(self);
                    if let Err(e) = try_result {
                        log::error(format!("onMessage Error,{}", e.msg));
                    }
                }
            }
        }
    }
    // public static TTSServiceBuilder builder() {
    //     return new TTSServiceBuilder();
    // }
    pub fn builder() -> TTSServiceBuilder {
        return TTSServiceBuilder::new();
    }

    // public byte[] sendText(SSML ssml) {
    pub fn send_text(&mut self, ssml: &mut SSML) -> Vec<u8> {
        while self.synthesising {
            log::info("空转等待上一个语音合成");
            Tools::sleep(1);
        }
        self.latch = Some(CountDownLatch::new(1));
        self.synthesising = true;

        // Style is only supported in Azure API
        if ssml.get_style().is_some() && !self.using_azure_api {
            // fix: set_style 签名为非 Option（SSML.set_style(TtsStyleEnum)），清空样式直接赋值 pub 字段（等价 setStyle(null)）
            ssml.style = None;
        }

        // If SSML has a different output format, send config
        if ssml.get_output_format().is_some() && !self.output_format.unwrap().eq(&ssml.get_output_format().unwrap()) {
            self.send_config(ssml.get_output_format());
        }

        log::info(format!("ssml:{}", ssml.to_string()));
        if !self.get_or_create_ws().send(ssml.to_string()) {
            panic!("{}", TtsException::of("语音合成请求发送失败...".to_string()).message);
        }
        self.current_text = Some(ssml.get_synthesis_text());
        // fix: try/catch → 闭包 + match（catch 仅 panic，等价 panic!(e)）；await 为关键字 → r#await
        //      原 latch.r#await 依赖 WebSocketListener 回调（stub 无事件源）→ 轮询 poll_events 驱动
        let try_result: Result<Vec<u8>, StubError> = (|| {
            let deadline = System::current_time_millis() + 30_000;
            loop {
                if let Some(ws) = &self.ws {
                    let events = ws.poll_events();
                    for ev in events {
                        self.handle_ws_event(ev);
                    }
                }
                if self.latch.as_ref().map(|l| l.count() <= 0).unwrap_or(true) {
                    break;
                }
                if System::current_time_millis() > deadline {
                    return Err(StubError::new("语音合成超时".to_string()));
                }
                Tools::sleep(20);
            }
            Ok(self.audio_buffer.read_byte_array())
        })();
        match try_result {
            Ok(bytes) => return bytes,
            Err(e) => panic!("{}", e),
        }
    }

    // private synchronized WebSocket getOrCreateWs() {
    pub fn get_or_create_ws(&mut self) -> WebSocket {
        if self.ws.is_some() {
            return self.ws.clone().unwrap();
        }

        let url;
        let origin;
        if self.using_azure_api {
            url = "wss://eastus.api.speech.microsoft.com/cognitiveservices/websocket/v1?Retry-After=200&TrafficType=AzureDemo&Authorization=bearer undefined&X-ConnectionId=".to_string() + &Tools::get_random_id();
            origin = TtsConstants::AZURE_SPEECH_ORIGIN;
        } else {
            url = "wss://speech.platform.bing.com/consumer/speech/synthesize/readaloud/edge/v1?Retry-After=200&TrustedClientToken=6A5AA1D4EAFF4E9FB37E23D68491D6F4&ConnectionId=".to_string() + &Tools::get_random_id();
            origin = TtsConstants::EDGE_SPEECH_ORIGIN;
        }

        let request = Request::builder()
            .url(&url)
            .add_header("User-Agent", TtsConstants::UA)
            .add_header("Origin", origin)
            .build();

        self.ws = Some(self.get_ok_http_client().new_web_socket(request, self.web_socket_listener.clone()));
        self.send_config(self.output_format);
        return self.ws.clone().unwrap();
    }

    // private OkHttpClient getOkHttpClient() {
    pub fn get_ok_http_client(&mut self) -> OkHttpClient {
        if self.ok_http_client.is_none() {
            self.ok_http_client = Some(OkHttpClient::builder()
                .ping_interval(20, TimeUnit::SECONDS)
                .build());
        }
        return self.ok_http_client.clone().unwrap();
    }

    // private void sendConfig(OutputFormat format) {
    pub fn send_config(&mut self, format: Option<OutputFormat>) {
        let config = SpeechConfig::of(format);
        log::info(format!("audio config:{}", config.to_string()));
        if !self.get_or_create_ws().send(config.to_string()) {
            panic!("{}", TtsException::of("语音输出格式配置失败...".to_string()).message);
        }
        self.output_format = config.get_output_format();
    }
}

// public static class TTSServiceBuilder {
pub struct TTSServiceBuilder {
    // private OutputFormat outputFormat;
    // private boolean usingAzureApi;
    pub output_format: Option<OutputFormat>,
    pub using_azure_api: bool,
}

impl TTSServiceBuilder {
    // public TTSServiceBuilder() {
    // }
    pub fn new() -> TTSServiceBuilder {
        TTSServiceBuilder {
            output_format: None,
            using_azure_api: false,
        }
    }

    // public TTSServiceBuilder usingOutputFormat(OutputFormat outputFormat) {
    //     this.outputFormat = outputFormat;
    //     return this;
    // }
    pub fn using_output_format(&mut self, output_format: OutputFormat) -> &mut TTSServiceBuilder {
        self.output_format = Some(output_format);
        return self;
    }

    // public TTSServiceBuilder usingAzureApi(boolean usingAzureApi) {
    //     this.usingAzureApi = usingAzureApi;
    //     return this;
    // }
    pub fn using_azure_api(&mut self, using_azure_api: bool) -> &mut TTSServiceBuilder {
        self.using_azure_api = using_azure_api;
        return self;
    }

    // public TTSService build() {
    //     return new TTSService(outputFormat, usingAzureApi);
    // }
    pub fn build(&mut self) -> TTSService {
        return TTSService::new(self.output_format, self.using_azure_api);
    }
}
