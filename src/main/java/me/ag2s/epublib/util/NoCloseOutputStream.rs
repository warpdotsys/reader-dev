use crate::prelude::*;
use std::io;

/**
 * OutputStream with the close() disabled.
 * We write multiple documents to a ZipOutputStream.
 * Some of the formatters call a close() after writing their data.
 * We don't want them to do that, so we wrap regular OutputStreams in this NoCloseOutputStream.
 *
 * @author paul
 */
#[allow(dead_code)]
pub struct NoCloseOutputStream {
    output_stream: Box<dyn Write>,
}

impl NoCloseOutputStream {
    pub fn new(output_stream: Box<dyn Write>) -> Self {
        NoCloseOutputStream { output_stream }
    }

    pub fn write(&mut self, b: u8) -> Result<(), io::Error> {
        self.output_stream.write_all(&[b])
    }

    /**
     * A close() that does not call it's parent's close()
     */
    pub fn close(&self) {
    }
}

use std::io::Write;
