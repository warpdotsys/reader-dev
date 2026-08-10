// package me.ag2s.epublib.domain;

// import java.io.Serializable;
// import java.util.Arrays;
// import java.util.Collection;

/**
 * MediaType is used to tell the type of content a resource is.
 *
 * Examples of mediatypes are image/gif, text/css and application/xhtml+xml
 *
 * All allowed mediaTypes are maintained bye the MediaTypeService.
 *
 * @see MediaTypes
 *
 * @author paul
 */
pub struct MediaType {
  name: String,
  default_extension: String,
  extensions: Vec<String>,
}

impl MediaType {

  pub fn new(name: String, default_extension: String) -> MediaType {
    MediaType::with_extensions(name, default_extension, vec![default_extension.clone()])
  }

  pub fn with_extensions(name: String, default_extension: String,
      extensions: Vec<String>) -> MediaType {
    MediaType::with_extension_collection(name, default_extension, extensions)
  }

  pub fn hash_code(&self) -> i32 {
    if self.name.is_empty() {
      return 0;
    }
    return self.name.hash_code();
  }

  pub fn with_extension_collection(name: String, default_extension: String,
      mextensions: Vec<String>) -> MediaType {
    MediaType {
      name: name,
      default_extension: default_extension,
      extensions: mextensions,
    }
  }

  pub fn get_name(&self) -> &String {
    &self.name
  }


  pub fn get_default_extension(&self) -> &String {
    &self.default_extension
  }


  pub fn get_extensions(&self) -> &Vec<String> {
    &self.extensions
  }

  pub fn equals(&self, other_media_type: &dyn Any) -> bool {
    if !(other_media_type.is::<MediaType>()) {
      return false;
    }
    return self.name.eq(&other_media_type.downcast_ref::<MediaType>().unwrap().get_name());
  }
  // @SuppressWarnings("NullableProblems")
  pub fn to_string(&self) -> String {
    return self.name.clone();
  }
}
