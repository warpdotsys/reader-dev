// package me.ag2s.epublib.domain;

// import java.io.Serializable;
// import java.util.ArrayList;
// import java.util.List;

/**
 * The guide is a selection of special pages of the book.
 * Examples of these are the cover, list of illustrations, etc.
 *
 * It is an optional part of an epub, and support for the various types
 * of references varies by reader.
 *
 * The only part of this that is heavily used is the cover page.
 *
 * @author paul
 *
 */
pub struct Guide {

  references: Vec<GuideReference>,
  cover_page_index: i32,
}

impl Guide {

  pub const DEFAULT_COVER_TITLE: &'static str = GuideReference::COVER;

  const COVERPAGE_NOT_FOUND: i32 = -1;
  const COVERPAGE_UNITIALIZED: i32 = -2;

  pub fn new() -> Guide {
    Guide {
      references: Vec::new(),
      cover_page_index: -1,
    }
  }

  pub fn get_references(&self) -> &Vec<GuideReference> {
    &self.references
  }

  pub fn set_references(&mut self, references: Vec<GuideReference>) {
    self.references = references;
    self.uncheck_cover_page();
  }

  fn uncheck_cover_page(&mut self) {
    self.cover_page_index = Guide::COVERPAGE_UNITIALIZED;
  }

  pub fn get_cover_reference(&mut self) -> Option<&GuideReference> {
    self.check_cover_page();
    if self.cover_page_index >= 0 {
      return Some(&self.references[self.cover_page_index as usize]);
    }
    return None;
  }
  // @SuppressWarnings("UnusedReturnValue")
  pub fn set_cover_reference(&mut self, guide_reference: GuideReference) -> i32 {
    if self.cover_page_index >= 0 {
      self.references[self.cover_page_index as usize] = guide_reference;
    } else {
      self.references.insert(0, guide_reference);
      self.cover_page_index = 0;
    }
    return self.cover_page_index;
  }

  fn check_cover_page(&mut self) {
    if self.cover_page_index == Guide::COVERPAGE_UNITIALIZED {
      self.init_cover_page();
    }
  }


  fn init_cover_page(&mut self) {
    let mut result = Guide::COVERPAGE_NOT_FOUND;
    for i in 0..self.references.len() {
      let guide_reference = &self.references[i];
      if guide_reference.get_type().eq(GuideReference::COVER) {
        result = i as i32;
        break;
      }
    }
    self.cover_page_index = result;
  }

  /**
   * The coverpage of the book.
   *
   * @return The coverpage of the book.
   */
  pub fn get_cover_page(&mut self) -> Option<Resource> {
    let guide_reference = self.get_cover_reference();
    if guide_reference.is_none() {
      return None;
    }
    return Some(guide_reference.unwrap().get_resource().clone());
  }

  pub fn set_cover_page(&mut self, cover_page: Option<Resource>) {
    let coverpage_guide_reference = GuideReference::with_type(cover_page.unwrap().clone(),
        GuideReference::COVER.to_string(), Guide::DEFAULT_COVER_TITLE.to_string());
    self.set_cover_reference(coverpage_guide_reference);
  }

  // @SuppressWarnings("UnusedReturnValue")
  pub fn add_reference(&mut self, reference: GuideReference) -> ResourceReference {
    self.references.push(reference.clone());
    self.uncheck_cover_page();
    return reference;
  }

  /**
   * A list of all GuideReferences that have the given
   * referenceTypeName (ignoring case).
   *
   * @param referenceTypeName referenceTypeName
   * @return A list of all GuideReferences that have the given
   *    referenceTypeName (ignoring case).
   */
  pub fn get_guide_references_by_type(
      &self, reference_type_name: &String) -> Vec<GuideReference> {
    let mut result: Vec<GuideReference> = Vec::new();
    for guide_reference in &self.references {
      if reference_type_name.eq_ignore_ascii_case(guide_reference.get_type()) {
        result.push(guide_reference.clone());
      }
    }
    return result;
  }
}
