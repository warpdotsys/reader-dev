package com.htmake.reader.lib.tts.constant;

public interface TtsConstants {

    String TRUSTED_CLIENT_TOKEN = "6A5AA1D4EAFF4E9FB37E23D68491D6F4";

    String VOICE_LIST_URL = "https://speech.platform.bing.com/consumer/speech/synthesize/readaloud/voices/list";

    String EDGE_SPEECH_WSS = "wss://speech.platform.bing.com/consumer/speech/synthesize/readaloud/edge/v1";

    String EDGE_SPEECH_ORIGIN = "chrome-extension://jdiccldimpdaibmpdkjnbmckianbfold";

    String AZURE_SPEECH_WSS = "wss://eastus.api.speech.microsoft.com/cognitiveservices/websocket/v1";

    String AZURE_SPEECH_ORIGIN = "https://azure.microsoft.com";

    String UA = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/111.0.0.0 Safari/537.36 Edg/111.0.1661.44";

    String TURN_START = "turn.start";

    String TURN_END = "turn.end";

    String AUDIO_START = "Path:audio\r\n";

    String AUDIO_CONTENT_TYPE = "Content-Type:audio";
}
