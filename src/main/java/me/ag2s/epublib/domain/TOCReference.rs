// package me.ag2s.epublib.domain;

// import java.io.Serializable;
// import java.util.ArrayList;
// import java.util.Comparator;
// import java.util.List;

/**
 * An item in the Table of Contents.
 *
 * @see TableOfContents
 *
 * @author paul
 */
pub struct TOCReference {
  resource: Option<Resource>,
  fragment_id: Option<String>,
  title: Option<String>,
  children: Vec<TOCReference>,
}

impl TOCReference {

  // @Deprecated
  pub fn new() -> TOCReference {
    TOCReference::with_fragment(None, None, None)
  }

  // @SuppressWarnings("unused")
  pub fn get_comparator_by_title_ignore_case() -> fn(&TOCReference, &TOCReference) -> std::cmp::Ordering {
    return |toc_reference1: &TOCReference, toc_reference2: &TOCReference| {
      String::CASE_INSENSITIVE_ORDER.compare(toc_reference1.get_title().as_ref().unwrap_or(&String::new()), toc_reference2.get_title().as_ref().unwrap_or(&String::new()))
    };
  }

  pub fn with_name(name: Option<String>, resource: Option<Resource>) -> TOCReference {
    TOCReference::with_fragment(name, resource, None)
  }

  pub fn with_fragment(name: Option<String>, resource: Option<Resource>, fragment_id: Option<String>) -> TOCReference {
    TOCReference::with_children(name, resource, fragment_id, Vec::new())
  }

  pub fn with_children(title: Option<String>, resource: Option<Resource>, fragment_id: Option<String>,
      children: Vec<TOCReference>) -> TOCReference {
    TOCReference {
      resource: resource,
      title: title,
      fragment_id: fragment_id,
      children: children,
    }
  }

  pub fn get_children(&self) -> &Vec<TOCReference> {
    &self.children
  }

  pub fn add_child_section(&mut self, child_section: TOCReference) -> TOCReference {
    self.children.push(child_section.clone());
    return child_section;
  }

  pub fn set_children(&mut self, children: Vec<TOCReference>) {
    self.children = children;
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

  pub fn set_resource(&mut self, resource: Option<Resource>) {
    self.set_resource_with_fragment(resource, None);
  }

  pub fn set_resource_with_fragment(&mut self, resource: Option<Resource>, fragment_id: Option<String>) {
    self.resource = resource;
    self.fragment_id = fragment_id;
  }

  pub fn get_fragment_id(&self) -> &Option<String> {
    &self.fragment_id
  }
}
