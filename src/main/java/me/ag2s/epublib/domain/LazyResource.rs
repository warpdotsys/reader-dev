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
  resource_provider: Option<LazyResourceProvider>,
  cached_size: i64,
}

impl LazyResource {

  pub fn with_href(resource_provider: LazyResourceProvider, href: String) -> LazyResource {
    LazyResource::with_size(resource_provider, -1, href)
  }
  pub fn with_original_href(resource_provider: LazyResourceProvider, href: String, original_href: String) -> LazyResource {
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
          resource_provider: LazyResourceProvider, size: i64, href: String) -> LazyResource {
    let mut result = LazyResource::base(None, None, href.clone(), href.clone(), MediaTypes::determine_media_type(&href));
    result.resource_provider = Some(resource_provider);
    result.cached_size = size;
    result
  }
  pub fn with_size_and_original_href(
      resource_provider: LazyResourceProvider, size: i64, href: String, original_href: String) -> LazyResource {
    let mut result = LazyResource::base(None, None, href.clone(), original_href.clone(), MediaTypes::determine_media_type(&href));
    result.resource_provider = Some(resource_provider);
    result.cached_size = size;
    result
  }

  fn base(id: Option<String>, data: Option<Vec<u8>>, href: String, original_href: String, media_type: Option<MediaType>) -> LazyResource {
    LazyResource {
      id: id.unwrap_or_default(),
      title: String::new(),
      href: href,
      properties: String::new(),
      original_href: original_href,
      media_type: media_type,
      input_encoding: Constants::CHARACTER_ENCODING.to_string(),
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
  pub fn get_input_stream(&mut self) -> Result<InputStream, IOException> {
    if self.is_initialized() {
      return Ok(ByteArrayInputStream::new(self.get_data()?));
    } else {
      return self.resource_provider.as_ref().unwrap().get_resource_stream(&self.original_href);
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
      let read_data = IOUtil::to_byte_array(&mut in_stream, self.cached_size as i32);
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
