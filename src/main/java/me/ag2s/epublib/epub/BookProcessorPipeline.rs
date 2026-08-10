use std::collections::VecDeque;

use crate::me::ag2s::epublib::domain::EpubBook;
use crate::me::ag2s::epublib::epub::BookProcessor;

/**
 * A book processor that combines several other bookprocessors
 * <p>
 * Fixes coverpage/coverimage.
 * Cleans up the XHTML.
 *
 * @author paul.siegmann
 */
#[allow(dead_code)]
pub struct BookProcessorPipeline {
    static_tag: &'static str,
    book_processors: Option<Vec<Box<dyn BookProcessor>>>,
}

impl BookProcessorPipeline {
    pub fn new() -> Self {
        BookProcessorPipeline::new_with_pipeline(None)
    }

    pub fn new_with_pipeline(book_processing_pipeline: Option<Vec<Box<dyn BookProcessor>>>) -> Self {
        BookProcessorPipeline {
            static_tag: "me.ag2s.epublib.epub.BookProcessorPipeline",
            book_processors: book_processing_pipeline,
        }
    }

    pub fn process_book(&self, book: EpubBook) -> EpubBook {
        let mut book = book;
        if let Some(ref book_processors) = self.book_processors {
            for book_processor in book_processors {
                // Log.e(TAG, e.getMessage(), e);
                match book_processor.process_book(book) {
                    Ok(b) => book = b,
                    Err(e) => {
                        e.printStackTrace();
                    }
                }
            }
        }
        book
    }

    pub fn add_book_processor(&mut self, book_processor: Box<dyn BookProcessor>) {
        if self.book_processors.is_none() {
            self.book_processors = Some(Vec::new());
        }
        self.book_processors.as_mut().unwrap().push(book_processor);
    }

    pub fn add_book_processors(&mut self, book_processors: VecDeque<Box<dyn BookProcessor>>) {
        if self.book_processors.is_none() {
            self.book_processors = Some(Vec::new());
        }
        self.book_processors.as_mut().unwrap().extend(book_processors);
    }

    pub fn get_book_processors(&self) -> &Option<Vec<Box<dyn BookProcessor>>> {
        &self.book_processors
    }

    pub fn set_book_processing_pipeline(&mut self, book_processing_pipeline: Option<Vec<Box<dyn BookProcessor>>>) {
        self.book_processors = book_processing_pipeline;
    }
}
