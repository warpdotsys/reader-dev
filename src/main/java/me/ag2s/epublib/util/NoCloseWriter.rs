use crate::prelude::*;
use std::io;

/**
 * Writer with the close() disabled.
 * We write multiple documents to a ZipOutputStream.
 * Some of the formatters call a close() after writing their data.
 * We don't want them to do that, so we wrap regular Writers in this NoCloseWriter.
 *
 * @author paul
 */
#[allow(dead_code)]
pub struct NoCloseWriter {
    writer: Box<dyn WriteStr>,
}

impl NoCloseWriter {
    pub fn new(writer: Box<dyn WriteStr>) -> Self {
        NoCloseWriter { writer }
    }

    pub fn close(&self) {
    }

    pub fn flush(&mut self) -> Result<(), io::Error> {
        self.writer.flush()
    }

    pub fn write(&mut self, cbuf: &[char], off: usize, len: usize) -> Result<(), io::Error> {
        self.writer.write_str(&cbuf[off..off + len].iter().collect::<String>())
    }
}

use std::io::Write;

// fix: E0225——`Box<dyn Write + WriteStr>` 双非 auto trait 对象非法，
//      改为超 trait 方案（编译器建议）：`WriteStr: io::Write`，Box<dyn WriteStr> 即覆盖二者
pub trait WriteStr: io::Write {
    fn write_str(&mut self, s: &str) -> Result<(), io::Error>;
}
