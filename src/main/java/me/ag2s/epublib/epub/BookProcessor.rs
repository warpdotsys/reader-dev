use crate::me::ag2s::epublib::domain::EpubBook;

/**
 * Post-processes a book.
 *
 * Can be used to clean up a book after reading or before writing.
 *
 * @author paul
 */
pub trait BookProcessor {

    /**
     * A BookProcessor that returns the input book unchanged.
     */
    fn identity_bookprocessor(&self, book: EpubBook) -> EpubBook {
        book
    }

    fn process_book(&self, book: EpubBook) -> EpubBook;
}
