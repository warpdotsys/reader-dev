// package io.legado.app.adapters

/**
 * Singleton helper holding the current ReaderAdapterInterface instance.
 */
pub struct ReaderAdapterHelper;

pub static mut READER_ADAPTER: Option<Box<dyn ReaderAdapterInterface>> = None;

impl ReaderAdapterHelper {

    pub fn reader_adapter() -> &'static dyn ReaderAdapterInterface {
        static DEFAULT: DefaultAdpater = DefaultAdpater;
        unsafe { READER_ADAPTER.as_deref().unwrap_or(&DEFAULT) }
    }

    pub fn set_adapter(adapter: Box<dyn ReaderAdapterInterface>) {
        unsafe { READER_ADAPTER = Some(adapter); }
    }

    pub fn get_adapter() -> &'static dyn ReaderAdapterInterface {
        Self::reader_adapter()
    }
}
