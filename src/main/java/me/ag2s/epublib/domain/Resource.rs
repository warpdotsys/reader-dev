// package me.ag2s.epublib.domain;

// import me.ag2s.epublib.Constants;
// import me.ag2s.epublib.util.IOUtil;
// import me.ag2s.epublib.util.StringUtil;
// import me.ag2s.epublib.util.commons.io.XmlStreamReader;
// import java.io.ByteArrayInputStream;
// import java.io.IOException;
// import java.io.InputStream;
// import java.io.Reader;
// import java.io.Serializable;

/**
 * Represents a resource that is part of the epub.
 * A resource can be a html file, image, xml, etc.
 *
 * @author paul
 *
 */
pub struct Resource {
  id: String,
  title: String,
  href: String,
  properties: String,
  original_href: String,
  media_type: Option<MediaType>,
  input_encoding: String,
  data: Option<Vec<u8>>,
}

impl Resource {

  /**
   * Creates an empty Resource with the given href.
   *
   * Assumes that if the data is of a text type (html/css/etc) then the encoding will be UTF-8
   *
   * @param href The location of the resource within the epub. Example: "chapter1.html".
   */
  pub fn with_href(href: String) -> Resource {
    Resource::with_id_data(None, Some(Vec::new()), href.clone(), MediaTypes::determine_media_type(&href))
  }

  /**
   * Creates a Resource with the given data and MediaType.
   * The href will be automatically generated.
   *
   * Assumes that if the data is of a text type (html/css/etc) then the encoding will be UTF-8
   *
   * @param data The Resource's contents
   * @param mediaType The MediaType of the Resource
   */
  pub fn with_data(data: Vec<u8>, media_type: MediaType) -> Resource {
    Resource::with_id_data(None, Some(data), None, Some(media_type))
  }

  /**
   * Creates a resource with the given data at the specified href.
   * The MediaType will be determined based on the href extension.
   *
   * Assumes that if the data is of a text type (html/css/etc) then the encoding will be UTF-8
   *
   * @see MediaTypes#determineMediaType(String)
   *
   * @param data The Resource's contents
   * @param href The location of the resource within the epub. Example: "chapter1.html".
   */
  pub fn with_data_and_href(data: Vec<u8>, href: String) -> Resource {
    Resource::with_id_data_href(None, Some(data), href.clone(), MediaTypes::determine_media_type(&href),
        Constants::CHARACTER_ENCODING.to_string())
  }

  /**
   * Creates a resource with the data from the given Reader at the specified href.
   * The MediaType will be determined based on the href extension.
   *
   * @see MediaTypes#determineMediaType(String)
   *
   * @param in The Resource's contents
   * @param href The location of the resource within the epub. Example: "cover.jpg".
   */
  pub fn with_reader(in_reader: &mut Reader, href: String) -> Result<Resource, IOException> {
    Ok(Resource::with_id_data_href(None, Some(IOUtil::to_byte_array(in_reader, Constants::CHARACTER_ENCODING)?), href.clone(),
        MediaTypes::determine_media_type(&href),
        Constants::CHARACTER_ENCODING.to_string()))
  }

  /**
   * Creates a resource with the data from the given InputStream at the specified href.
   * The MediaType will be determined based on the href extension.
   *
   * @see MediaTypes#determineMediaType(String)
   *
   * Assumes that if the data is of a text type (html/css/etc) then the encoding will be UTF-8
   *
   * It is recommended to us the {@link #Resource(Reader, String)} method for creating textual
   * (html/css/etc) resources to prevent encoding problems.
   * Use this method only for binary Resources like images, fonts, etc.
   *
   *
   * @param in The Resource's contents
   * @param href The location of the resource within the epub. Example: "cover.jpg".
   */
  pub fn with_input_stream(in_stream: &mut InputStream, href: String) -> Result<Resource, IOException> {
    Ok(Resource::with_id_data(None, Some(IOUtil::to_byte_array(in_stream)?), href.clone(),
        MediaTypes::determine_media_type(&href)))
  }

  /**
   * Creates a resource with the given id, data, mediatype at the specified href.
   * Assumes that if the data is of a text type (html/css/etc) then the encoding will be UTF-8
   *
   * @param id The id of the Resource. Internal use only. Will be auto-generated if it has a null-value.
   * @param data The Resource's contents
   * @param href The location of the resource within the epub. Example: "chapter1.html".
   * @param mediaType The resources MediaType
   */
  pub fn with_id_data(id: Option<String>, data: Option<Vec<u8>>, href: Option<String>, media_type: Option<MediaType>) -> Resource {
    Resource::with_id_data_href(id, data, href, media_type, Constants::CHARACTER_ENCODING.to_string())
  }
  pub fn with_id_data_and_original_href(id: Option<String>, data: Option<Vec<u8>>, href: Option<String>, original_href: String, media_type: Option<MediaType>) -> Resource {
    Resource::with_id_data_href_and_original_href(id, data, href, original_href, media_type, Constants::CHARACTER_ENCODING.to_string())
  }


  /**
   * Creates a resource with the given id, data, mediatype at the specified href.
   * If the data is of a text type (html/css/etc) then it will use the given inputEncoding.
   *
   * @param id The id of the Resource. Internal use only. Will be auto-generated if it has a null-value.
   * @param data The Resource's contents
   * @param href The location of the resource within the epub. Example: "chapter1.html".
   * @param mediaType The resources MediaType
   * @param inputEncoding If the data is of a text type (html/css/etc) then it will use the given inputEncoding.
   */
  pub fn with_id_data_href(id: Option<String>, data: Option<Vec<u8>>, href: Option<String>, media_type: Option<MediaType>,
      input_encoding: String) -> Resource {
    Resource {
      id: id.unwrap_or_default(),
      href: href.clone().unwrap_or_default(),
      original_href: href.clone().unwrap_or_default(),
      media_type: media_type,
      input_encoding: input_encoding,
      data: data,
      title: String::new(),
      properties: String::new(),
    }
  }
  pub fn with_id_data_href_and_original_href(id: Option<String>, data: Option<Vec<u8>>, href: Option<String>, original_href: String, media_type: Option<MediaType>,
      input_encoding: String) -> Resource {
    Resource {
      id: id.unwrap_or_default(),
      href: href.unwrap_or_default(),
      original_href: original_href,
      media_type: media_type,
      input_encoding: input_encoding,
      data: data,
      title: String::new(),
      properties: String::new(),
    }
  }

  /**
   * Gets the contents of the Resource as an InputStream.
   *
   * @return The contents of the Resource.
   *
   * @throws IOException IOException
   */
  pub fn get_input_stream(&self) -> Result<InputStream, IOException> {
    return Ok(ByteArrayInputStream::new(self.get_data()?.clone()));
  }

  /**
   * The contents of the resource as a byte[]
   *
   * @return The contents of the resource
   */
  pub fn get_data(&self) -> Result<&Vec<u8>, IOException> {
    return Ok(self.data.as_ref().unwrap());
  }

  /**
   * Tells this resource to release its cached data.
   *
   * If this resource was not lazy-loaded, this is a no-op.
   */
  pub fn close(&mut self) {
  }

  /**
   * Sets the data of the Resource.
   * If the data is a of a different type then the original data then make sure to change the MediaType.
   *
   * @param data the data of the Resource
   */
  pub fn set_data(&mut self, data: Vec<u8>) {
    self.data = Some(data);
  }

  /**
   * Returns the size of this resource in bytes.
   *
   * @return the size.
   */
  pub fn get_size(&self) -> i64 {
    return self.data.as_ref().unwrap().len() as i64;
  }

  /**
   * If the title is found by scanning the underlying html document then it is cached here.
   *
   * @return the title
   */
  pub fn get_title(&self) -> &String {
    &self.title
  }

  /**
   * Sets the Resource's id: Make sure it is unique and a valid identifier.
   *
   * @param id Resource's id
   */
  pub fn set_id(&mut self, id: String) {
    self.id = id;
  }

  /**
   * The resources Id.
   *
   * Must be both unique within all the resources of this book and a valid identifier.
   * @return The resources Id.
   */
  pub fn get_id(&self) -> &String {
    &self.id
  }

  /**
   * The location of the resource within the contents folder of the epub file.
   *
   * Example:<br/>
   * images/cover.jpg<br/>
   * content/chapter1.xhtml<br/>
   *
   * @return The location of the resource within the contents folder of the epub file.
   */
  pub fn get_href(&self) -> &String {
    &self.href
  }

  /**
   * Sets the Resource's href.
   *
   * @param href Resource's href.
   */
  pub fn set_href(&mut self, href: String) {
    self.href = href;
  }

  /**
   * The character encoding of the resource.
   * Is allowed to be null for non-text resources like images.
   *
   * @return The character encoding of the resource.
   */
  pub fn get_input_encoding(&self) -> &String {
    &self.input_encoding
  }

  /**
   * Sets the Resource's input character encoding.
   *
   * @param encoding Resource's input character encoding.
   */
  pub fn set_input_encoding(&mut self, encoding: String) {
    self.input_encoding = encoding;
  }

  /**
   * Gets the contents of the Resource as Reader.
   *
   * Does all sorts of smart things (courtesy of apache commons io XMLStreamREader) to handle encodings, byte order markers, etc.
   *
   * @return the contents of the Resource as Reader.
   * @throws IOException IOException
   */
  pub fn get_reader(&self) -> Result<Reader, IOException> {
    return Ok(XmlStreamReader::new(ByteArrayInputStream::new(self.get_data()?.clone()),
        Some(self.get_input_encoding().clone())));
  }

  /**
   * Gets the hashCode of the Resource's href.
   *
   */
  pub fn hash_code(&self) -> i32 {
    return self.href.hash_code();
  }

  /**
   * Checks to see of the given resourceObject is a resource and whether its href is equal to this one.
   *
   * @return whether the given resourceObject is a resource and whether its href is equal to this one.
   */
  pub fn equals(&self, resource_object: &dyn Any) -> bool {
    if !(resource_object.is::<Resource>()) {
      return false;
    }
    return self.href.eq(&resource_object.downcast_ref::<Resource>().unwrap().get_href());
  }

  /**
   * This resource's mediaType.
   *
   * @return This resource's mediaType.
   */
  pub fn get_media_type(&self) -> &Option<MediaType> {
    &self.media_type
  }

  pub fn set_media_type(&mut self, media_type: Option<MediaType>) {
    self.media_type = media_type;
  }

  pub fn set_title(&mut self, title: String) {
    self.title = title;
  }

  pub fn get_properties(&self) -> &String {
    &self.properties
  }

  pub fn set_properties(&mut self, properties: String) {
    self.properties = properties;
  }
  // @SuppressWarnings("NullableProblems")
  pub fn to_string(&self) -> String {
    return StringUtil::to_string(&vec![
        ("id", Some(&self.id)),
        ("title", Some(&self.title)),
        ("encoding", Some(&self.input_encoding)),
        ("mediaType", Some(&self.media_type.as_ref().map(|v| v.to_string()).unwrap_or_default())),
        ("href", Some(&self.href)),
        ("size", Some(&(if self.data.is_none() { 0 } else { self.data.as_ref().unwrap().len() }).to_string()))]);
  }
}
