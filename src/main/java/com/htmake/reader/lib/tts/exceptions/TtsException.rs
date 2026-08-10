// package com.htmake.reader.lib.tts.exceptions;

// public class TtsException extends RuntimeException {
pub struct TtsException {
    pub message: String,
}

impl TtsException {
    // private TtsException(String message) {
    //     super(message);
    // }
    fn new(message: String) -> TtsException {
        TtsException { message }
    }

    // public static TtsException of(String message) {
    //     return new TtsException(message);
    // }
    pub fn of(message: String) -> TtsException {
        return TtsException::new(message);
    }
}
