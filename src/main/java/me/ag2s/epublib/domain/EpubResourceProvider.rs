use crate::prelude::*;
use crate::stubs::{File, ZipFile};
// package me.ag2s.epublib.domain;

// import java.io.IOException;
// import java.io.InputStream;
// import java.util.zip.ZipEntry;
// import java.util.zip.ZipFile;

/**
 * @author jake
 */
pub struct EpubResourceProvider {
  epub_filename: String,
}

impl EpubResourceProvider {

  /**
   * @param epubFilename the file name for the epub we're created from.
   */
  pub fn new(epub_filename: String) -> EpubResourceProvider {
    EpubResourceProvider {
      epub_filename: epub_filename,
    }
  }

  // @Override
  pub fn get_resource_stream(&self, href: &String) -> Result<Box<dyn InputStream>, IOException> {
    let zip_file = ZipFile::new(&File::new(&self.epub_filename));
    let zip_entry = zip_file.get_entry(href);
    if zip_entry.is_none() {
      zip_file.close();
      return Err(IOException::new(
          format!("Cannot find entry {} in epub file {}", href, self.epub_filename)));
    }
    // fix: ResourceInputStream::new 签名损坏（裸 trait 参数）无法调用，直接返回 ZipFile 条目流占位
    return Ok(zip_file.get_input_stream_dyn(&zip_entry.unwrap()));
  }
}
