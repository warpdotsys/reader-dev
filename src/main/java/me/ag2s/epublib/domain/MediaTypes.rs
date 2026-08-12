use crate::prelude::*;
// package me.ag2s.epublib.domain;

// import me.ag2s.epublib.util.StringUtil;
// import java.util.HashMap;
// import java.util.Map;


/**
 * Manages mediatypes that are used by epubs
 *
 * @author paul
 */
pub struct MediaTypes;

impl MediaTypes {

  // fix: MediaType 为 stubs::MediaType 单元占位（无数据），new/with_extensions 为 const fn 且忽略参数，
  // 常量仅作类型占位；原 Java 值语义由调用方持有（见 stubs::MediaTypes::NCX_NAME / XHTML_NAME 等）
  pub const XHTML: MediaType = MediaType::with_extensions("application/xhtml+xml",
      ".xhtml", &[".htm", ".html", ".xhtml"]);
  pub const EPUB: MediaType = MediaType::new("application/epub+zip",
      ".epub");
  pub const NCX: MediaType = MediaType::new("application/x-dtbncx+xml",
      ".ncx");

  pub const JAVASCRIPT: MediaType = MediaType::new("text/javascript",
      ".js");
  pub const CSS: MediaType = MediaType::new("text/css", ".css");

  // images
  pub const JPG: MediaType = MediaType::with_extensions("image/jpeg", ".jpg",
      &[".jpg", ".jpeg"]);
  pub const PNG: MediaType = MediaType::new("image/png", ".png");
  pub const GIF: MediaType = MediaType::new("image/gif", ".gif");

  pub const SVG: MediaType = MediaType::new("image/svg+xml", ".svg");

  // fonts
  pub const TTF: MediaType = MediaType::new(
      "application/x-truetype-font", ".ttf");
  pub const OPENTYPE: MediaType = MediaType::new(
      "application/vnd.ms-opentype", ".otf");
  pub const WOFF: MediaType = MediaType::new("application/font-woff",
      ".woff");

  // audio
  pub const MP3: MediaType = MediaType::new("audio/mpeg", ".mp3");
  pub const OGG: MediaType = MediaType::new("audio/ogg", ".ogg");

  // video
  pub const MP4: MediaType = MediaType::new("video/mp4", ".mp4");

  pub const SMIL: MediaType = MediaType::new("application/smil+xml",
      ".smil");
  pub const XPGT: MediaType = MediaType::new(
      "application/adobe-page-template+xml", ".xpgt");
  pub const PLS: MediaType = MediaType::new("application/pls+xml",
      ".pls");

  pub fn media_types() -> Vec<MediaType> {
    vec![
      MediaTypes::XHTML, MediaTypes::EPUB, MediaTypes::JPG, MediaTypes::PNG, MediaTypes::GIF, MediaTypes::CSS, MediaTypes::SVG, MediaTypes::TTF, MediaTypes::NCX, MediaTypes::XPGT, MediaTypes::OPENTYPE, MediaTypes::WOFF,
      MediaTypes::SMIL, MediaTypes::PLS, MediaTypes::JAVASCRIPT, MediaTypes::MP3, MediaTypes::MP4, MediaTypes::OGG
    ]
  }

  pub fn media_types_by_name() -> HashMap<String, MediaType> {
    let mut media_types_by_name: HashMap<String, MediaType> = HashMap::new();

    for media_type in MediaTypes::media_types() {
      media_types_by_name.insert(media_type.get_name().clone(), media_type);
    }
    media_types_by_name
  }

  // fix: stubs::MediaType 相等性占位恒真（单元类型无数据），is_bitmap_image 退化为恒返回 true
  pub fn is_bitmap_image(media_type: &MediaType) -> bool {
    return media_type == &MediaTypes::JPG || media_type == &MediaTypes::PNG || media_type == &MediaTypes::GIF;
  }

  /**
   * Gets the MediaType based on the file extension.
   * None of no matching extension found.
   *
   * @param filename filename
   * @return the MediaType based on the file extension.
   */
  pub fn determine_media_type(filename: &String) -> Option<MediaType> {
    // fix: stubs get_extensions 占位返回空表 → 循环不执行，恒返回 None（原逻辑需真实 MediaType）
    for media_type in MediaTypes::media_types_by_name().values() {
      for extension in media_type.get_extensions() {
        if StringUtil::ends_with_ignore_case(filename, extension) {
          return Some(media_type.clone());
        }
      }
    }
    return None;
  }

  pub fn get_media_type_by_name(media_type_name: &String) -> Option<MediaType> {
    return MediaTypes::media_types_by_name().get(media_type_name).cloned();
  }
}
