use crate::prelude::*;
// package me.ag2s.epublib.domain;

// import me.ag2s.epublib.util.StringUtil;
// import java.io.Serializable;


/**
 * These are references to elements of the book's guide.
 *
 * @see Guide
 *
 * @author paul
 *
 */
#[derive(Clone)]
pub struct GuideReference {
  resource: Option<Resource>,
  title: Option<String>,
  fragment_id: Option<String>,
  type_: Option<String>,
}

impl GuideReference {

  /**
   * the book cover(s), jacket information, etc.
   */
  pub const COVER: &'static str = "cover";

  /**
   * human-readable page with title, author, publisher, and other metadata
   */
  pub const TITLE_PAGE: &'static str = "title-page";

  /**
   * Human-readable table of contents.
   * Not to be confused the epub file table of contents
   *
   */
  pub const TOC: &'static str = "toc";

  /**
   * back-of-book style index
   */
  pub const INDEX: &'static str = "index";
  pub const GLOSSARY: &'static str = "glossary";
  pub const ACKNOWLEDGEMENTS: &'static str = "acknowledgements";
  pub const BIBLIOGRAPHY: &'static str = "bibliography";
  pub const COLOPHON: &'static str = "colophon";
  pub const COPYRIGHT_PAGE: &'static str = "copyright-page";
  pub const DEDICATION: &'static str = "dedication";

  /**
   * an epigraph is a phrase, quotation, or poem that is set at the
   * beginning of a document or component.
   *
   * source: http://en.wikipedia.org/wiki/Epigraph_%28literature%29
   */
  pub const EPIGRAPH: &'static str = "epigraph";

  pub const FOREWORD: &'static str = "foreword";

  /**
   * list of illustrations
   */
  pub const LOI: &'static str = "loi";

  /**
   * list of tables
   */
  pub const LOT: &'static str = "lot";
  pub const NOTES: &'static str = "notes";
  pub const PREFACE: &'static str = "preface";

  /**
   * A page of content (e.g. "Chapter 1")
   */
  pub const TEXT: &'static str = "text";

  pub fn new(resource: Option<Resource>) -> GuideReference {
    GuideReference::with_title(resource, None)
  }

  pub fn with_title(resource: Option<Resource>, title: Option<String>) -> GuideReference {
    GuideReference::with_type_and_title(resource, None, title)
  }

  pub fn with_type_and_title(resource: Option<Resource>, type_: Option<String>,
      title: Option<String>) -> GuideReference {
    GuideReference::with_fragment(resource, type_, title, None)
  }

  pub fn with_type(resource: Option<Resource>, type_: String, title: String) -> GuideReference {
    GuideReference::with_fragment(resource, Some(type_), Some(title), None)
  }

  pub fn with_fragment(resource: Option<Resource>, type_: Option<String>, title: Option<String>,
      fragment_id: Option<String>) -> GuideReference {
    let mut result = GuideReference {
      resource: resource.clone(),
      title: title.clone(),
      fragment_id: fragment_id.clone(),
      type_: None,
    };
    result.type_ = if StringUtil::is_not_blank(&type_.as_ref().unwrap_or(&String::new())) { Some(type_.unwrap().to_lowercase()) } else { None };
    result
  }

  pub fn get_type(&self) -> &String {
    &self.type_.as_ref().unwrap()
  }

  pub fn set_type(&mut self, type_: String) {
    self.type_ = Some(type_);
  }

  pub fn get_resource(&self) -> &Option<Resource> {
    &self.resource
  }

  pub fn get_title(&self) -> &Option<String> {
    &self.title
  }

  pub fn set_title(&mut self, title: Option<String>) {
    self.title = title;
  }

  pub fn get_fragment_id(&self) -> &Option<String> {
    &self.fragment_id
  }

  pub fn set_fragment_id(&mut self, fragment_id: Option<String>) {
    self.fragment_id = fragment_id;
  }

  pub fn get_complete_href(&self) -> String {
    if StringUtil::is_blank(self.get_fragment_id().as_ref().unwrap_or(&String::new())) {
      return self.get_resource().as_ref().unwrap().get_href().clone();
    } else {
      // fix: Constants 为 trait，无法直接引用关联常量；Java 中 FRAGMENT_SEPARATOR_CHAR == '#'
      return self.get_resource().as_ref().unwrap().get_href().clone() + &'#'.to_string()
          + &self.fragment_id.as_ref().unwrap().clone();
    }
  }

  pub fn set_resource(&mut self, resource: Option<Resource>) {
    self.set_resource_with_fragment(resource, None);
  }

  pub fn set_resource_with_fragment(&mut self, resource: Option<Resource>, fragment_id: Option<String>) {
    self.resource = resource;
    self.fragment_id = fragment_id;
  }
}
