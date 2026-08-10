// package me.ag2s.epublib.domain;

// import java.io.IOException;
// import java.io.InputStream;

/**
 * @author jake
 */
pub trait LazyResourceProvider {

  fn get_resource_stream(&self, href: &String) -> Result<InputStream, IOException>;
}
