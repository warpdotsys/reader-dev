package com.htmake.reader.lib.tts.exceptions;

public class TtsException extends RuntimeException {

    private TtsException(String message) {
        super(message);
    }

    public static TtsException of(String message) {
        return new TtsException(message);
    }
}
