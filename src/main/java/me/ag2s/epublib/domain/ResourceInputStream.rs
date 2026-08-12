use crate::prelude::*;
// fix: 显式导入消解 prelude 多 glob 歧义（ZipFile ← resourcesloader/stubs；FileInputStream ← resourceutil/stubs）
use crate::stubs::{FileInputStream, ZipFile};
// package me.ag2s.epublib.domain;

// import java.io.FilterInputStream;
// import java.io.IOException;
// import java.io.InputStream;
// import java.util.zip.ZipFile;

/**
 * A wrapper class for closing a ZipFile object when the InputStream derived
 * from it is closed.
 *
 * @author ttopalov
 */
pub struct ResourceInputStream {
  in_stream: FileInputStream,
  zip_file: ZipFile,
}

impl ResourceInputStream {

  /**
   * Constructor.
   *
   * @param in
   *            The InputStream object.
   * @param zipFile
   *            The ZipFile object.
   */
  pub fn new(in_stream: FileInputStream, zip_file: ZipFile) -> ResourceInputStream {
    ResourceInputStream {
      in_stream: in_stream,
      zip_file: zip_file,
    }
  }

  // @Override
  pub fn close(&mut self) -> Result<(), IOException> {
    self.in_stream.close().map_err(|e| StubError::new(e.to_string()))?;
    self.zip_file.close();
    Ok(())
  }
}
