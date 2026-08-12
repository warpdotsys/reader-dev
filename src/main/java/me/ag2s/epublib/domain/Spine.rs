use crate::prelude::*;
// package me.ag2s.epublib.domain;

// import me.ag2s.epublib.util.StringUtil;
// import java.io.Serializable;
// import java.util.ArrayList;
// import java.util.Collection;
// import java.util.List;

/**
 * The spine sections are the sections of the book in the order in which the book should be read.
 *
 * This contrasts with the Table of Contents sections which is an index into the Book's sections.
 *
 * @see TableOfContents
 *
 * @author paul
 */
pub struct Spine {
  toc_resource: Option<Resource>,
  spine_references: Vec<SpineReference>,
}

impl Spine {

  pub fn new() -> Spine {
    Spine::with_references(Vec::new())
  }

  /**
   * Creates a spine out of all the resources in the table of contents.
   *
   * @param tableOfContents tableOfContents
   */
  pub fn from_toc(table_of_contents: &TableOfContents) -> Spine {
    Spine {
      toc_resource: None,
      spine_references: Spine::create_spine_references(&table_of_contents.get_all_unique_resources()),
    }
  }

  pub fn with_references(spine_references: Vec<SpineReference>) -> Spine {
    Spine {
      toc_resource: None,
      spine_references: spine_references,
    }
  }

  pub fn create_spine_references(
      resources: &Vec<Resource>) -> Vec<SpineReference> {
    let mut result: Vec<SpineReference> = Vec::with_capacity(
            resources.len());
    for resource in resources {
      result.push(SpineReference::new(Some(resource.clone())));
    }
    return result;
  }

  pub fn get_spine_references(&self) -> &Vec<SpineReference> {
    &self.spine_references
  }

  pub fn set_spine_references(&mut self, spine_references: Vec<SpineReference>) {
    self.spine_references = spine_references;
  }

  /**
   * Gets the resource at the given index.
   * None if not found.
   *
   * @param index index
   * @return the resource at the given index.
   */
  pub fn get_resource(&self, index: i32) -> Option<Resource> {
    if index < 0 || index >= self.spine_references.len() as i32 {
      return None;
    }
    return self.spine_references[index as usize].get_resource().clone();
  }

  /**
   * Finds the first resource that has the given resourceId.
   *
   * None if not found.
   *
   * @param resourceId resourceId
   * @return the first resource that has the given resourceId.
   */
  pub fn find_first_resource_by_id(&self, resource_id: &String) -> i32 {
    if StringUtil::is_blank(resource_id) {
      return -1;
    }

    for i in 0..self.spine_references.len() {
      let spine_reference = &self.spine_references[i];
      if resource_id.eq(&spine_reference.get_resource_id().unwrap_or_default()) {
        return i as i32;
      }
    }
    return -1;
  }

  /**
   * Adds the given spineReference to the spine references and returns it.
   *
   * @param spineReference spineReference
   * @return the given spineReference
   */
  pub fn add_spine_reference(&mut self, spine_reference: SpineReference) -> SpineReference {
    // fix: Java null 检查（spineReferences == null）转录为 Vec 恒非空，省略
    self.spine_references.push(spine_reference.clone());
    return spine_reference;
  }

  /**
   * Adds the given resource to the spine references and returns it.
   *
   * @return the given spineReference
   */
  // @SuppressWarnings("unused")
  pub fn add_resource(&mut self, resource: Resource) -> SpineReference {
    return self.add_spine_reference(SpineReference::new(Some(resource)));
  }

  /**
   * The number of elements in the spine.
   *
   * @return The number of elements in the spine.
   */
  pub fn size(&self) -> i32 {
    return self.spine_references.len() as i32;
  }

  /**
   * As per the epub file format the spine officially maintains a reference to the Table of Contents.
   * The epubwriter will look for it here first, followed by some clever tricks to find it elsewhere if not found.
   * Put it here to be sure of the expected behaviours.
   *
   * @param tocResource tocResource
   */
  pub fn set_toc_resource(&mut self, toc_resource: Option<Resource>) {
    self.toc_resource = toc_resource;
  }

  /**
   * The resource containing the XML for the tableOfContents.
   * When saving an epub file this resource needs to be in this place.
   *
   * @return The resource containing the XML for the tableOfContents.
   */
  pub fn get_toc_resource(&self) -> &Option<Resource> {
    &self.toc_resource
  }

  /**
   * The position within the spine of the given resource.
   *
   * @param currentResource currentResource
   * @return something &lt; 0 if not found.
   *
   */
  pub fn get_resource_index(&self, current_resource: &Option<Resource>) -> i32 {
    if current_resource.is_none() {
      return -1;
    }
    return self.get_resource_index_by_href(current_resource.as_ref().unwrap().get_href());
  }

  /**
   * The first position within the spine of a resource with the given href.
   *
   * @return something &lt; 0 if not found.
   *
   */
  // fix: Java 重载 getResourceIndex(String) 转录改名，避免与 Option<Resource> 版重名
  pub fn get_resource_index_by_href(&self, resource_href: &String) -> i32 {
    let mut result = -1;
    if StringUtil::is_blank(resource_href) {
      return result;
    }
    for i in 0..self.spine_references.len() {
      if resource_href.eq(self.spine_references[i].get_resource().as_ref().unwrap().get_href()) {
        result = i as i32;
        break;
      }
    }
    return result;
  }

  /**
   * Whether the spine has any references
   * @return Whether the spine has any references
   */
  pub fn is_empty(&self) -> bool {
    return self.spine_references.is_empty();
  }
}
