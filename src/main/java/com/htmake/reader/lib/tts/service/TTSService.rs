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
    pub output_format: Option<OutputFormat>,
    pub using_azure_api: bool,
    pub synthesising: bool,
    pub current_text: Option<String>,
    pub audio_buffer: Buffer,
    pub ok_http_client: Option<OkHttpClient>,
    pub ws: Option<WebSocket>,
    pub latch: Option<CountDownLatch>,
    pub web_socket_listener: Option<Box<dyn Fn(WebSocketEvent)>>,
}

impl TTSService {
    // private TTSService(OutputFormat outputFormat, boolean usingAzureApi) {
    pub fn new(output_format: Option<OutputFormat>, using_azure_api: bool) -> TTSService {
        let audio_buffer = Buffer::new();
        let web_socket_listener: Option<Box<dyn Fn(WebSocketEvent)>> = Some(Box::new(|event| {
            // WebSocketListener callbacks transcribed
            match event {
                // @Override
                // public void onClosed(WebSocket webSocket, int code, String reason) {
                //     super.onClosed(webSocket, code, reason);
                //     log.debug("onClosed:" + reason);
                //     ws = null;
                //     synthesising = false;
                // }
                WebSocketEvent::onClosed(reason) => {
                    log::debug(format!("onClosed:{}", reason));
                    ws = None;
                    synthesising = false;
                }
                // @Override
                // public void onClosing(WebSocket webSocket, int code, String reason) {
                //     super.onClosing(webSocket, code, reason);
                //     log.debug("onClosing:" + reason);
                //     ws = null;
                //     synthesising = false;
                // }
                WebSocketEvent::onClosing(reason) => {
                    log::debug(format!("onClosing:{}", reason));
                    ws = None;
                    synthesising = false;
                }
                // @Override
                // public void onFailure(WebSocket webSocket, Throwable t, Response response) {
                //     super.onFailure(webSocket, t, response);
                //     log.debug("onFailure" + t.getMessage(), t);
                //     ws = null;
                //     synthesising = false;
                // }
                WebSocketEvent::onFailure(message) => {
                    log::debug(format!("onFailure{}", message));
                    ws = None;
                    synthesising = false;
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
                        audio_buffer.clear();
                    } else if text.contains(TtsConstants::TURN_END) {
                        latch.count_down();
                        synthesising = false;
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
                    let audio_index = bytes.last_index_of(TtsConstants::AUDIO_START.as_bytes()) + TtsConstants::AUDIO_START.len();
                    let has_content_type = bytes.last_index_of(TtsConstants::AUDIO_CONTENT_TYPE.as_bytes()) + TtsConstants::AUDIO_CONTENT_TYPE.len() != -1;
                    if audio_index != -1 && has_content_type {
                        try {
                            audio_buffer.write(bytes.substring(audio_index));
                        } catch (e) {
                            log::error(format!("onMessage Error,{}", e.message));
                        }
                    }
                }
            }
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
            ssml.set_style(null);
        }

        // If SSML has a different output format, send config
        if ssml.get_output_format().is_some() && !self.output_format.unwrap().eq(&ssml.get_output_format().unwrap()) {
            self.send_config(ssml.get_output_format().unwrap());
        }

        log::info("ssml:{:?}", ssml.to_string());
        if !self.get_or_create_ws().send(ssml.to_string()) {
            panic!(TtsException::of("语音合成请求发送失败..."));
        }
        self.current_text = Some(ssml.get_synthesis_text());
        try {
            self.latch.as_ref().unwrap().await(30, TimeUnit::SECONDS);
            return self.audio_buffer.read_byte_array();
        } catch (e: InterruptedException) {
            panic!(e);
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
            .url(url)
            .add_header("User-Agent", TtsConstants::UA)
            .add_header("Origin", origin)
            .build();

        self.ws = Some(self.get_ok_http_client().new_web_socket(request, self.web_socket_listener.clone()));
        self.send_config(self.output_format.unwrap());
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
        log::info("audio config:{:?}", config.to_string());
        if !self.get_or_create_ws().send(config.to_string()) {
            panic!(TtsException::of("语音输出格式配置失败..."));
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
