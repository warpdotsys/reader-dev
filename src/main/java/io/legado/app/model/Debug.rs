use crate::prelude::*;
// package io.legado.app.model
// import io.legado.app.data.entities.Book
// import io.legado.app.data.entities.BookChapter
// import io.legado.app.model.webBook.WebBook
// import mu.KotlinLogging

// private val logger = KotlinLogging.logger {}

pub struct Debug;

impl DebugLog for Debug {
    fn clone_box(&self) -> Box<dyn DebugLog> {
        Box::new(Debug)
    }
}
