use crate::prelude::*;
use crate::stubs::java_util_Date;
// package me.ag2s.epublib.domain;

// import java.io.Serializable;
// import java.text.SimpleDateFormat;
// import java.util.Locale;

// import me.ag2s.epublib.epub.PackageDocumentBase;

/**
 * A Date used by the book's metadata.
 * <p>
 * Examples: creation-date, modification-date, etc
 *
 * @author paul
 */
#[derive(Clone)]
pub struct Date {
    event: Option<Event>,
    date_string: String,
}

impl Date {

    pub fn new() -> Date {
        Date::with_date_and_event(java_util_Date::new(), Some(Event::CREATION))
    }

    pub fn with_date(date: java_util_Date) -> Date {
        Date::with_date_and_event(date, None)
    }

    pub fn with_date_string(date_string: String) -> Date {
        Date::with_date_string_and_event(date_string, None)
    }

    // fix: Rust 不支持重载，原 Kotlin withDate(Date, Event?) 重载更名为 with_date_and_event（与 with_date_string_and_event 命名一致）
    pub fn with_date_and_event(date: java_util_Date, event: Option<Event>) -> Date {
        Date::with_date_string_and_event((SimpleDateFormat::new_2args(PackageDocumentBase::DATE_FORMAT, Locale::US)).format(date.0),
                event)
    }

    pub fn with_date_string_and_event(date_string: String, event: Option<Event>) -> Date {
        Date {
            date_string: date_string,
            event: event,
        }
    }

    pub fn with_date_and_event_string(date: java_util_Date, event: String) -> Date {
        Date::with_date_string_and_event_string((SimpleDateFormat::new_2args(PackageDocumentBase::DATE_FORMAT, Locale::US)).format(date.0),
                event)
    }

    pub fn with_date_string_and_event_string(date_string: String, event: String) -> Date {
        let mut result = Date::with_date_string_and_event(Date::check_date(&date_string), Event::from_value(&event));
        result.date_string = date_string;
        result
    }

    fn check_date(date_string: &String) -> String {
        if date_string.is_empty() {
            panic!(
                    "Cannot create a date from a blank string")
        }
        date_string.clone()
    }

    pub fn get_value(&self) -> &String {
        &self.date_string
    }

    pub fn get_event(&self) -> &Option<Event> {
        &self.event
    }

    pub fn set_event(&mut self, event: Option<Event>) {
        self.event = event;
    }

    // @Override
    // @SuppressWarnings("NullableProblems")
    pub fn to_string(&self) -> String {
        if self.event.is_none() {
            return self.date_string.clone();
        }
        format!("{}:{}", self.event.as_ref().unwrap().to_string(), self.date_string)
    }
}

#[derive(Clone)]
pub enum Event {
    PUBLICATION,
    MODIFICATION,
    CREATION,
}

impl Event {

    pub fn value(&self) -> &'static str {
        match self {
            Event::PUBLICATION => "publication",
            Event::MODIFICATION => "modification",
            Event::CREATION => "creation",
        }
    }

    pub fn values() -> Vec<Event> {
        vec![Event::PUBLICATION, Event::MODIFICATION, Event::CREATION]
    }

    pub fn from_value(v: &String) -> Option<Event> {
        for c in Event::values() {
            if c.value().eq(v) {
                return Some(c);
            }
        }
        return None;
    }

    // @Override
    // @SuppressWarnings("NullableProblems")
    pub fn to_string(&self) -> String {
        self.value().to_string()
    }
}
