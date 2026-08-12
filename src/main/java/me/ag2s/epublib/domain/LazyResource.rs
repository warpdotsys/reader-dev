use crate::prelude::*;
// fix: E0659 歧义——prelude glob 同时导出 mediatype 模块与 stubs 的 MediaType；MediaTypes::determine_media_type 返回 stubs::MediaType（占位），故此处显式导入 stubs::MediaType 保持一致
use crate::stubs::MediaType;
// package me.ag2s.epublib.domain;

// import me.ag2s.epublib.util.IOUtil;
// import java.io.ByteArrayInputStream;
// import java.io.IOException;
// import java.io.InputStream;

/**
 * A Resource that loads its data only on-demand from a EPUB book file.
 * This way larger books can fit into memory and can be opened faster.
 */
pub struct LazyResource {
  id: String,
  title: String,
  href: String,
  properties: String,
  original_href: String,
  media_type: Option<MediaType>,
  input_encoding: String,
  data: Option<Vec<u8>>,
  tag: String,
  resource_provider: Option<Box<dyn LazyResourceProvider>>,
  cached_size: i64,
}

impl LazyResource {

  pub fn with_href(resource_provider: impl LazyResourceProvider + 'static, href: String) -> LazyResource {
    LazyResource::with_size(resource_provider, -1, href)
  }
  pub fn with_original_href(resource_provider: impl LazyResourceProvider + 'static, href: String, original_href: String) -> LazyResource {
    LazyResource::with_size_and_original_href(resource_provider, -1, href, original_href)
  }

  /**
   * Creates a Lazy resource, by not actually loading the data for this entry.
   *
   * The data will be loaded on the first call to getData()
   *
   * @param resourceProvider The resource provider loads data on demand.
   * @param size The size of this resource.
   * @param href The resource's href within the epub.
   */
  pub fn with_size(
          resource_provider: impl LazyResourceProvider + 'static, size: i64, href: String) -> LazyResource {
    let mut result = LazyResource::base(None, None, href.clone(), href.clone(), MediaTypes::determine_media_type(&href));
    result.resource_provider = Some(Box::new(resource_provider));
    result.cached_size = size;
    result
  }
  pub fn with_size_and_original_href(
      resource_provider: impl LazyResourceProvider + 'static, size: i64, href: String, original_href: String) -> LazyResource {
    let mut result = LazyResource::base(None, None, href.clone(), original_href.clone(), MediaTypes::determine_media_type(&href));
    result.resource_provider = Some(Box::new(resource_provider));
    result.cached_size = size;
    result
  }

  // fix: ResourcesLoader 转录调用 `LazyResource::new(provider, size, href)`（Kotlin 构造函数），别名到 with_size
  pub fn new(resource_provider: impl LazyResourceProvider + 'static, size: i64, href: String) -> LazyResource {
    LazyResource::with_size(resource_provider, size, href)
  }

  fn base(id: Option<String>, data: Option<Vec<u8>>, href: String, original_href: String, media_type: Option<MediaType>) -> LazyResource {
    LazyResource {
      id: id.unwrap_or_default(),
      title: String::new(),
      href: href,
      properties: String::new(),
      original_href: original_href,
      media_type: media_type,
      // fix: E0790——Constants 转录为 trait，关联常量无法以 `Constants::CHARACTER_ENCODING` 访问，改用字面量（值即 "UTF-8"）
      input_encoding: "UTF-8".to_string(),
      data: data,
      tag: String::from("LazyResource"),
      resource_provider: None,
      cached_size: -1,
    }
  }

  /**
   * Gets the contents of the Resource as an InputStream.
   *
   * @return The contents of the Resource.
   *
   * @throws IOException IOException
   */
  pub fn get_input_stream(&mut self) -> Result<ByteArrayInputStream, IOException> {
    if self.is_initialized() {
      return Ok(ByteArrayInputStream::new(self.get_data()?.clone()));
    } else {
      // fix: get_resource_stream 返回 Box<dyn InputStream>，读出全部字节包装为 ByteArrayInputStream
      let mut stream = self.resource_provider.as_ref().unwrap().get_resource_stream(&self.original_href)?;
      let mut buf: Vec<u8> = Vec::new();
      let mut chunk = [0u8; 4096];
      loop {
        let chunk_len = chunk.len();
        let n = stream.read(&mut chunk, 0, chunk_len);
        if n <= 0 {
          break;
        }
        buf.extend_from_slice(&chunk[..n as usize]);
      }
      return Ok(ByteArrayInputStream::new(buf));
    }
  }

  /**
   * Initializes the resource by loading its data into memory.
   *
   * @throws IOException IOException
   */
  pub fn initialize(&mut self) -> Result<(), IOException> {
    self.get_data()?;
    Ok(())
  }

  /**
   * The contents of the resource as a byte[]
   *
   * If this resource was lazy-loaded and the data was not yet loaded,
   * it will be loaded into memory at this point.
   *  This included opening the zip file, so expect a first load to be slow.
   *
   * @return The contents of the resource
   */
  pub fn get_data(&mut self) -> Result<&Vec<u8>, IOException> {

    if self.data.is_none() {

      // Log.d(TAG, "Initializing lazy resource: " + this.getHref());

      let mut in_stream = self.resource_provider.as_ref().unwrap().get_resource_stream(&self.original_href)?;
      // fix: IOUtil::to_byte_array_size 期望 IOUtil 的 InputStream 结构体，与 Box<dyn InputStream> 体系不匹配，直接循环读取
      let mut read_data: Option<Vec<u8>> = Some(Vec::new());
      let mut chunk = [0u8; 4096];
      loop {
        let chunk_len = chunk.len();
        let n = in_stream.read(&mut chunk, 0, chunk_len);
        if n <= 0 {
          break;
        }
        read_data.as_mut().unwrap().extend_from_slice(&chunk[..n as usize]);
      }
      if read_data.is_none() {
        return Err(IOException::new(
            format!("Could not load the contents of resource: {}", self.get_href())));
      } else {
        self.data = read_data;
      }

      in_stream.close();
    }

    return Ok(self.data.as_ref().unwrap());
  }

  /**
   * Tells this resource to release its cached data.
   *
   * If this resource was not lazy-loaded, this is a no-op.
   */
  pub fn close(&mut self) {
    if self.resource_provider.is_some() {
      self.data = None;
    }
  }

  /**
   * Returns if the data for this resource has been loaded into memory.
   *
   * @return true if data was loaded.
   */
  pub fn is_initialized(&self) -> bool {
    return self.data.is_some();
  }

  /**
   * Returns the size of this resource in bytes.
   *
   * @return the size.
   */
  pub fn get_size(&self) -> i64 {
    if self.data.is_some() {
      return self.data.as_ref().unwrap().len() as i64;
    }

    return self.cached_size;
  }

  pub fn get_id(&self) -> &String {
    &self.id
  }

  pub fn get_href(&self) -> &String {
    &self.href
  }
}
