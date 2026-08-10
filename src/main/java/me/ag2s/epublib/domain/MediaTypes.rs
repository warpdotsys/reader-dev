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

  pub const XHTML: MediaType = MediaType::new("application/xhtml+xml".to_string(),
      ".xhtml".to_string(), vec![".htm".to_string(), ".html".to_string(), ".xhtml".to_string()]);
  pub const EPUB: MediaType = MediaType::new("application/epub+zip".to_string(),
      ".epub".to_string());
  pub const NCX: MediaType = MediaType::new("application/x-dtbncx+xml".to_string(),
      ".ncx".to_string());

  pub const JAVASCRIPT: MediaType = MediaType::new("text/javascript".to_string(),
      ".js".to_string());
  pub const CSS: MediaType = MediaType::new("text/css".to_string(), ".css".to_string());

  // images
  pub const JPG: MediaType = MediaType::new("image/jpeg".to_string(), ".jpg".to_string(),
      vec![".jpg".to_string(), ".jpeg".to_string()]);
  pub const PNG: MediaType = MediaType::new("image/png".to_string(), ".png".to_string());
  pub const GIF: MediaType = MediaType::new("image/gif".to_string(), ".gif".to_string());

  pub const SVG: MediaType = MediaType::new("image/svg+xml".to_string(), ".svg".to_string());

  // fonts
  pub const TTF: MediaType = MediaType::new(
      "application/x-truetype-font".to_string(), ".ttf".to_string());
  pub const OPENTYPE: MediaType = MediaType::new(
      "application/vnd.ms-opentype".to_string(), ".otf".to_string());
  pub const WOFF: MediaType = MediaType::new("application/font-woff".to_string(),
      ".woff".to_string());

  // audio
  pub const MP3: MediaType = MediaType::new("audio/mpeg".to_string(), ".mp3".to_string());
  pub const OGG: MediaType = MediaType::new("audio/ogg".to_string(), ".ogg".to_string());

  // video
  pub const MP4: MediaType = MediaType::new("video/mp4".to_string(), ".mp4".to_string());

  pub const SMIL: MediaType = MediaType::new("application/smil+xml".to_string(),
      ".smil".to_string());
  pub const XPGT: MediaType = MediaType::new(
      "application/adobe-page-template+xml".to_string(), ".xpgt".to_string());
  pub const PLS: MediaType = MediaType::new("application/pls+xml".to_string(),
      ".pls".to_string());

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

  pub fn is_bitmap_image(media_type: &MediaType) -> bool {
    return media_type == &MediaTypes::JPG || media_type == &MediaTypes::PNG || media_type == &MediaTypes::GIF;
  }

  /**
   * Gets the MediaType based on the file extension.
   * Null of no matching extension found.
   *
   * @param filename filename
   * @return the MediaType based on the file extension.
   */
  pub fn determine_media_type(filename: &String) -> Option<MediaType> {
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
