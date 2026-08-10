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
  pub fn get_resource_stream(&self, href: &String) -> Result<InputStream, IOException> {
    let mut zip_file = ZipFile::new(&self.epub_filename)?;
    let zip_entry = zip_file.get_entry(href);
    if zip_entry.is_none() {
      zip_file.close();
      return Err(IOException::new(
          format!("Cannot find entry {} in epub file {}", href, self.epub_filename)));
    }
    return Ok(ResourceInputStream::new(zip_file.get_input_stream(&zip_entry.unwrap()), zip_file));
  }
}
