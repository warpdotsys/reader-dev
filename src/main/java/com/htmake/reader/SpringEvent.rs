// package com.htmake.reader;

// import org.springframework.context.ApplicationEvent;

// public class SpringEvent extends ApplicationEvent {
pub struct SpringEvent {
    // private String event;
    // private String message;
    pub event: String,
    pub message: String,
}

impl SpringEvent {
    // public SpringEvent(Object source, String event, String message) {
    //     super(source);
    //     this.event = event;
    //     this.message = message;
    // }
    pub fn new(source: Object, event: String, message: String) -> SpringEvent {
        let mut s = SpringEvent {
            event: String::new(),
            message: String::new(),
        };
        super(source);
        s.event = event;
        s.message = message;
        s
    }

    // public String getEvent() {
    //     return event;
    // }
    pub fn get_event(&self) -> String {
        return self.event.clone();
    }

    // public void setEvent(String event) {
    //     this.event = event;
    // }
    pub fn set_event(&mut self, event: String) {
        self.event = event;
    }

    // public String getMessage() {
    //     return message;
    // }
    pub fn get_message(&self) -> String {
        return self.message.clone();
    }

    // public void setMessage(String message) {
    //     this.message = message;
    // }
    pub fn set_message(&mut self, message: String) {
        self.message = message;
    }
}
