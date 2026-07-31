package com.htmake.reader.lib.tts.service;

import com.htmake.reader.lib.tts.constant.OutputFormat;
import com.htmake.reader.lib.tts.constant.TtsConstants;
import com.htmake.reader.lib.tts.exceptions.TtsException;
import com.htmake.reader.lib.tts.model.SSML;
import com.htmake.reader.lib.tts.model.SpeechConfig;
import com.htmake.reader.lib.tts.util.Tools;
import okhttp3.OkHttpClient;
import okhttp3.Request;
import okhttp3.Response;
import okhttp3.WebSocket;
import okhttp3.WebSocketListener;
import okio.Buffer;
import okio.ByteString;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.nio.charset.StandardCharsets;
import java.util.Objects;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;

public class TTSService {

    public static final Logger log = LoggerFactory.getLogger(TTSService.class);

    private OutputFormat outputFormat;
    private boolean usingAzureApi;
    private volatile boolean synthesising;
    private String currentText;
    private final Buffer audioBuffer;
    private OkHttpClient okHttpClient;
    private WebSocket ws;
    private CountDownLatch latch;
    protected WebSocketListener webSocketListener;

    private TTSService(OutputFormat outputFormat, boolean usingAzureApi) {
        this.audioBuffer = new Buffer();
        this.webSocketListener = new WebSocketListener() {
            @Override
            public void onClosed(WebSocket webSocket, int code, String reason) {
                super.onClosed(webSocket, code, reason);
                log.debug("onClosed:" + reason);
                ws = null;
                synthesising = false;
            }

            @Override
            public void onClosing(WebSocket webSocket, int code, String reason) {
                super.onClosing(webSocket, code, reason);
                log.debug("onClosing:" + reason);
                ws = null;
                synthesising = false;
            }

            @Override
            public void onFailure(WebSocket webSocket, Throwable t, Response response) {
                super.onFailure(webSocket, t, response);
                log.debug("onFailure" + t.getMessage(), t);
                ws = null;
                synthesising = false;
            }

            @Override
            public void onMessage(WebSocket webSocket, String text) {
                super.onMessage(webSocket, text);
                if (text.contains(TtsConstants.TURN_START)) {
                    audioBuffer.clear();
                } else if (text.contains(TtsConstants.TURN_END)) {
                    latch.countDown();
                    synthesising = false;
                }
            }

            @Override
            public void onMessage(WebSocket webSocket, ByteString bytes) {
                super.onMessage(webSocket, bytes);
                int audioIndex = bytes.lastIndexOf(
                    TtsConstants.AUDIO_START.getBytes(StandardCharsets.UTF_8)
                ) + TtsConstants.AUDIO_START.length();
                boolean hasContentType = bytes.lastIndexOf(
                    TtsConstants.AUDIO_CONTENT_TYPE.getBytes(StandardCharsets.UTF_8)
                ) + TtsConstants.AUDIO_CONTENT_TYPE.length() != -1;
                if (audioIndex != -1 && hasContentType) {
                    try {
                        audioBuffer.write(bytes.substring(audioIndex));
                    } catch (Exception e) {
                        log.error("onMessage Error," + e.getMessage(), e);
                    }
                }
            }
        };
        this.outputFormat = outputFormat;
        this.usingAzureApi = usingAzureApi;
    }

    public static TTSServiceBuilder builder() {
        return new TTSServiceBuilder();
    }

    public byte[] sendText(SSML ssml) {
        while (synthesising) {
            log.info("\u7a7a\u8f6c\u7b49\u5f85\u4e0a\u4e00\u4e2a\u8bed\u97f3\u5408\u6210");
            Tools.sleep(1);
        }
        latch = new CountDownLatch(1);
        synthesising = true;

        // Style is only supported in Azure API
        if (Objects.nonNull(ssml.getStyle()) && !usingAzureApi) {
            ssml.setStyle(null);
        }

        // If SSML has a different output format, send config
        if (Objects.nonNull(ssml.getOutputFormat()) && !outputFormat.equals(ssml.getOutputFormat())) {
            sendConfig(ssml.getOutputFormat());
        }

        log.info("ssml:{}", ssml);
        if (!getOrCreateWs().send(ssml.toString())) {
            throw TtsException.of("\u8bed\u97f3\u5408\u6210\u8bf7\u6c42\u53d1\u9001\u5931\u8d25...");
        }
        currentText = ssml.getSynthesisText();
        try {
            latch.await(30L, TimeUnit.SECONDS);
            return audioBuffer.readByteArray();
        } catch (InterruptedException e) {
            throw new RuntimeException(e);
        }
    }

    private synchronized WebSocket getOrCreateWs() {
        if (Objects.nonNull(ws)) {
            return ws;
        }

        String url;
        String origin;
        if (usingAzureApi) {
            url = "wss://eastus.api.speech.microsoft.com/cognitiveservices/websocket/v1?Retry-After=200&TrafficType=AzureDemo&Authorization=bearer undefined&X-ConnectionId=" + Tools.getRandomId();
            origin = TtsConstants.AZURE_SPEECH_ORIGIN;
        } else {
            url = "wss://speech.platform.bing.com/consumer/speech/synthesize/readaloud/edge/v1?Retry-After=200&TrustedClientToken=6A5AA1D4EAFF4E9FB37E23D68491D6F4&ConnectionId=" + Tools.getRandomId();
            origin = TtsConstants.EDGE_SPEECH_ORIGIN;
        }

        Request request = new Request.Builder()
                .url(url)
                .addHeader("User-Agent", TtsConstants.UA)
                .addHeader("Origin", origin)
                .build();

        ws = getOkHttpClient().newWebSocket(request, webSocketListener);
        sendConfig(outputFormat);
        return ws;
    }

    private OkHttpClient getOkHttpClient() {
        if (okHttpClient == null) {
            okHttpClient = new OkHttpClient.Builder()
                    .pingInterval(20L, TimeUnit.SECONDS)
                    .build();
        }
        return okHttpClient;
    }

    private void sendConfig(OutputFormat format) {
        SpeechConfig config = SpeechConfig.of(format);
        log.info("audio config:{}", config);
        if (!getOrCreateWs().send(config.toString())) {
            throw TtsException.of("\u8bed\u97f3\u8f93\u51fa\u683c\u5f0f\u914d\u7f6e\u5931\u8d25...");
        }
        this.outputFormat = config.getOutputFormat();
    }

    public static class TTSServiceBuilder {

        private OutputFormat outputFormat;
        private boolean usingAzureApi;

        public TTSServiceBuilder() {
        }

        public TTSServiceBuilder usingOutputFormat(OutputFormat outputFormat) {
            this.outputFormat = outputFormat;
            return this;
        }

        public TTSServiceBuilder usingAzureApi(boolean usingAzureApi) {
            this.usingAzureApi = usingAzureApi;
            return this;
        }

        public TTSService build() {
            return new TTSService(outputFormat, usingAzureApi);
        }
    }
}
