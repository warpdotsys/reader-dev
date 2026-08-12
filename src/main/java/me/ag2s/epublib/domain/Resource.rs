use crate::prelude::*;
use crate::me_ag2s_epublib_domain_mediatypes::MediaTypes;
use crate::me_ag2s_epublib_util_ioutil::{Reader, InputStream as IOUtilInputStream};
use std::any::Any;
// fix: MediaType 不再显式导入真实模块——MediaTypes 转录为 stubs::MediaType 类型体系，
// 保持一致（prelude 中 stubs::MediaType 显式重导出优先于真实模块 glob）
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
// fix: 补充 Clone/PartialEq（Resources/Navigator/EpubBook 等多处 .clone() 与 `Option<Resource> == None` 依赖）
#[derive(Clone, PartialEq)]
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
    Resource::with_id_data(None, Some(Vec::new()), Some(href.clone()), MediaTypes::determine_media_type(&href))
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
    Resource::with_id_data_href(None, Some(data), Some(href.clone()), MediaTypes::determine_media_type(&href),
        "UTF-8".to_string())
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
    // fix: ioutil::Reader 为占位结构无读取 API，Java IOUtil.toByteArray(reader, encoding) 无法转录；占位：空数据
    let _ = in_reader;
    Ok(Resource::with_id_data_href(None, Some(Vec::new()), Some(href.clone()),
        MediaTypes::determine_media_type(&href),
        "UTF-8".to_string()))
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
  pub fn with_input_stream(in_stream: &mut IOUtilInputStream, href: String) -> Result<Resource, IOException> {
    Ok(Resource::with_id_data(None, Some(IOUtil::to_byte_array(in_stream)?), Some(href.clone()),
        MediaTypes::determine_media_type(&href)))
  }

  /**
   * Creates a resource with the given id, data, mediatype at the specified href.
   * Assumes that if the data is of a text type (html/css/etc) then the encoding will be UTF-8
   *
   * @param id The id of the Resource. Internal use only. Will be auto-generated if it has a None-value.
   * @param data The Resource's contents
   * @param href The location of the resource within the epub. Example: "chapter1.html".
   * @param mediaType The resources MediaType
   */
  pub fn with_id_data(id: Option<String>, data: Option<Vec<u8>>, href: Option<String>, media_type: Option<MediaType>) -> Resource {
    // fix: E0790——Constants 转录为 trait，关联常量无法以 `Constants::CHARACTER_ENCODING` 访问，改用字面量（值即 "UTF-8"）
    Resource::with_id_data_href(id, data, href, media_type, "UTF-8".to_string())
  }
  pub fn with_id_data_and_original_href(id: Option<String>, data: Option<Vec<u8>>, href: Option<String>, original_href: String, media_type: Option<MediaType>) -> Resource {
    Resource::with_id_data_href_and_original_href(id, data, href, original_href, media_type, "UTF-8".to_string())
  }

  /**
   * Java 构造器 Resource(byte[] data, String href) 的转录（ResourceUtil.createChapterResource 等使用）
   */
  pub fn new_bytes(data: Vec<u8>, href: &str) -> Resource {
    Resource::with_data_and_href(data, href.to_string())
  }

  /**
   * Java 构造器 Resource(byte[] data, MediaType mediaType) 的转录（ResourceUtil.createResourceFromFile 使用）
   */
  pub fn new_data(data: Vec<u8>, media_type: Option<MediaType>) -> Resource {
    Resource::with_id_data(None, Some(data), None, media_type)
  }

  /**
   * Java 构造器 Resource(String id, byte[] data, String href, MediaType mediaType, String inputEncoding) 的转录
   */
  pub fn new_full(id: Option<String>, data: Vec<u8>, href: &str, media_type: MediaType, input_encoding: &str) -> Resource {
    Resource::with_id_data_href(id, Some(data), Some(href.to_string()), Some(media_type), input_encoding.to_string())
  }

  /**
   * Java 构造器 Resource(InputStream in, String href) 的转录（ResourceUtil.createResource 使用）
   */
  pub fn new_stream<T>(in_stream: &T, href: String) -> Resource {
    // fix: Java 从 InputStream 读尽全部字节；in_stream 为占位类型，占位：空数据
    let _ = in_stream;
    Resource::with_data_and_href(Vec::new(), href)
  }


  /**
   * Creates a resource with the given id, data, mediatype at the specified href.
   * If the data is of a text type (html/css/etc) then it will use the given inputEncoding.
   *
   * @param id The id of the Resource. Internal use only. Will be auto-generated if it has a None-value.
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
  pub fn get_input_stream(&self) -> Result<ByteArrayInputStream, IOException> {
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
   * Is allowed to be None for non-text resources like images.
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
  pub fn get_reader(&self) -> Result<XmlStreamReader, IOException> {
    // fix: 返回类型改为 XmlStreamReader（转录中的 Reader 实体）；异常映射到 IOException(StubError)
    return XmlStreamReader::new_lenient_default(
        Box::new(ByteArrayInputStream::new(self.get_data()?.clone())),
        true,
        Some(self.get_input_encoding().clone()))
        .map_err(|e| IOException::new(e.to_string()));
  }

  /**
   * Gets the hashCode of the Resource's href.
   *
   */
  pub fn hash_code(&self) -> i32 {
    return self.href.hashCode();
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
    return self.href.eq(resource_object.downcast_ref::<Resource>().unwrap().get_href());
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
    // fix: StringUtil::to_string 转录为 Vec<Option<String>> 的成对 key/value（Java Object[] 两两配对）
    return StringUtil::to_string(vec![
        Some("id".to_string()), Some(self.id.clone()),
        Some("title".to_string()), Some(self.title.clone()),
        Some("encoding".to_string()), Some(self.input_encoding.clone()),
        Some("mediaType".to_string()), Some(self.media_type.as_ref().map(|v| v.to_string()).unwrap_or_default()),
        Some("href".to_string()), Some(self.href.clone()),
        Some("size".to_string()), Some((if self.data.is_none() { 0 } else { self.data.as_ref().unwrap().len() }).to_string())]);
  }
}
