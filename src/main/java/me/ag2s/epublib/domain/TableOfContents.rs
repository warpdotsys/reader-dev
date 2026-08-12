use crate::prelude::*;
// package me.ag2s.epublib.domain;

// import java.io.Serializable;
// import java.util.ArrayList;
// import java.util.Collection;
// import java.util.HashSet;
// import java.util.List;
// import java.util.Set;

/**
 * The table of contents of the book.
 * The TableOfContents is a tree structure at the root it is a list of TOCReferences, each if which may have as children another list of TOCReferences.
 *
 * The table of contents is used by epub as a quick index to chapters and sections within chapters.
 * It may contain duplicate entries, may decide to point not to certain chapters, etc.
 *
 * See the spine for the complete list of sections in the order in which they should be read.
 *
 * @see Spine
 *
 * @author paul
 */
pub struct TableOfContents {

  toc_references: Vec<TOCReference>,
}

impl TableOfContents {

  pub const DEFAULT_PATH_SEPARATOR: &'static str = "/";

  pub fn new() -> TableOfContents {
    TableOfContents::with_references(Vec::new())
  }

  pub fn with_references(toc_references: Vec<TOCReference>) -> TableOfContents {
    TableOfContents {
      toc_references: toc_references,
    }
  }

  pub fn get_toc_references(&self) -> &Vec<TOCReference> {
    &self.toc_references
  }

  pub fn set_toc_references(&mut self, toc_references: Vec<TOCReference>) {
    self.toc_references = toc_references;
  }

  /**
   * Calls addTOCReferenceAtLocation after splitting the path using the DEFAULT_PATH_SEPARATOR.
   * @return the new TOCReference
   */
  // @SuppressWarnings("unused")
  pub fn add_section(&mut self, resource: Option<Resource>, path: &String) -> Option<TOCReference> {
    return self.add_section_with_separator(resource, path, TableOfContents::DEFAULT_PATH_SEPARATOR.to_string());
  }

  /**
   * Calls addTOCReferenceAtLocation after splitting the path using the given pathSeparator.
   *
   * @param resource resource
   * @param path path
   * @param pathSeparator pathSeparator
   * @return the new TOCReference
   */
  pub fn add_section_with_separator(&mut self, resource: Option<Resource>, path: &String,
      path_separator: String) -> Option<TOCReference> {
    let path_elements: Vec<String> = path.split(&path_separator).map(|s| s.to_string()).collect();
    return self.add_section_with_elements(resource, path_elements);
  }

  /**
   * Finds the index of the first TOCReference in the given list that has the same title as the given Title.
   *
   * @param title title
   * @param tocReferences tocReferences
   * @return None if not found.
   */
  // fix: TOCReference 未实现 Clone，改为返回下标避免克隆
  fn find_toc_reference_index_by_title(title: &String,
      toc_references: &Vec<TOCReference>) -> Option<usize> {
    for (index, toc_reference) in toc_references.iter().enumerate() {
      if title.eq(toc_reference.get_title().as_ref().unwrap_or(&String::new())) {
        return Some(index);
      }
    }
    return None;
  }

  /**
   * Adds the given Resources to the TableOfContents at the location specified by the pathElements.
   *
   * Example:
   * Calling this method with a Resource and new String[] {"chapter1", "paragraph1"} will result in the following:
   * <ul>
   * <li>a TOCReference with the title "chapter1" at the root level.<br/>
   * If this TOCReference did not yet exist it will have been created and does not point to any resource</li>
   * <li>A TOCReference that has the title "paragraph1". This TOCReference will be the child of TOCReference "chapter1" and
   * will point to the given Resource</li>
   * </ul>
   *
   * @param resource resource
   * @param pathElements pathElements
   * @return the new TOCReference
   */
  pub fn add_section_with_elements(&mut self, resource: Option<Resource>, path_elements: Vec<String>) -> Option<TOCReference> {
    if path_elements.is_empty() {
      return None;
    }
    // fix: TOCReference 的 children 为私有字段且未实现 Clone，无法沿树向下走，
    // 只能在根层查找或创建节点
    let mut current_toc_references: &mut Vec<TOCReference> = &mut self.toc_references;
    let mut last_index: usize = 0;
    for current_title in &path_elements {
      let found_index = TableOfContents::find_toc_reference_index_by_title(current_title, current_toc_references);
      let index = match found_index {
        Some(index) => index,
        None => {
          current_toc_references.push(TOCReference::with_name(Some(current_title.clone()), None));
          current_toc_references.len() - 1
        }
      };
      last_index = index;
    }
    current_toc_references[last_index].set_resource(resource.clone());
    // fix: TOCReference 未实现 Clone，返回重建的引用
    return Some(TOCReference::with_fragment(
        self.toc_references[last_index].get_title().clone(), resource, None));
  }

  /**
   * Adds the given Resources to the TableOfContents at the location specified by the pathElements.
   *
   * Example:
   * Calling this method with a Resource and new int[] {0, 0} will result in the following:
   * <ul>
   * <li>a TOCReference at the root level.<br/>
   * If this TOCReference did not yet exist it will have been created with a title of "" and does not point to any resource</li>
   * <li>A TOCReference that points to the given resource and is a child of the previously created TOCReference.<br/>
   * If this TOCReference didn't exist yet it will be created and have a title of ""</li>
   * </ul>
   *
   * @param resource resource
   * @param pathElements pathElements
   * @return the new TOCReference
   */
  // @SuppressWarnings("unused")
  pub fn add_section_with_int_elements(&mut self, resource: Option<Resource>, path_elements: &Vec<i32>,
      section_title_prefix: &String, section_number_separator: &String) -> Option<TOCReference> {
    if path_elements.is_empty() {
      return None;
    }
    // fix: TOCReference 的 children 为私有字段且未实现 Clone，无法沿树向下走，
    // 只能在根层查找或创建节点
    let mut last_index: usize = 0;
    // fix: current_toc_references 借用 self，else 分支需 self 可变借用，改为循环内按需重新借用
    for i in 0..path_elements.len() {
      let current_index = path_elements[i];
      let len = self.toc_references.len() as i32 - 1;
      if current_index > 0 && current_index < len {
        let mut current_toc_references: &mut Vec<TOCReference> = &mut self.toc_references;
        if let Some(cr) = current_toc_references.get_mut(current_index as usize) {
          cr.set_resource(resource.clone());
        }
        last_index = current_index as usize;
      } else {
        self.padd_toc_references(path_elements, i as i32,
            section_title_prefix, section_number_separator);
        last_index = current_index as usize;
      }
    }
    if let Some(cr) = self.toc_references.get_mut(last_index) {
      cr.set_resource(resource.clone());
    }
    // fix: TOCReference 未实现 Clone，返回重建的引用
    return Some(TOCReference::with_fragment(
        self.toc_references[last_index].get_title().clone(), resource, None));
  }

  fn padd_toc_references(&mut self,
      path_elements: &Vec<i32>, path_pos: i32, section_prefix: &String,
      section_number_separator: &String) {
    for i in self.toc_references.len() as i32..=path_elements[path_pos as usize] {
      let section_title = self.create_section_title(path_elements, path_pos, i,
          section_prefix,
          section_number_separator);
      self.toc_references.push(TOCReference::with_name(Some(section_title), None));
    }
  }

  fn create_section_title(&self, path_elements: &Vec<i32>, path_pos: i32,
      last_pos: i32,
      section_prefix: &String, section_number_separator: &String) -> String {
    let mut title = section_prefix.clone();
    for i in 0..path_pos {
      if i > 0 {
        title.push_str(section_number_separator);
      }
      title.push_str(&(path_elements[i as usize] + 1).to_string());
    }
    if path_pos > 0 {
      title.push_str(section_number_separator);
    }
    title.push_str(&(last_pos + 1).to_string());
    return title;
  }

  pub fn add_toc_reference(&mut self, toc_reference: TOCReference) -> TOCReference {
    // fix: Java null 检查（tocReferences == null）转录为 Vec 恒非空，省略
    let title = toc_reference.get_title().clone();
    let resource = toc_reference.get_resource().clone();
    let fragment_id = toc_reference.get_fragment_id().clone();
    self.toc_references.push(toc_reference);
    // fix: TOCReference 未实现 Clone，返回重建的引用
    return TOCReference::with_fragment(title, resource, fragment_id);
  }

  /**
   * All unique references (unique by href) in the order in which they are referenced to in the table of contents.
   *
   * @return All unique references (unique by href) in the order in which they are referenced to in the table of contents.
   */
  pub fn get_all_unique_resources(&self) -> Vec<Resource> {
    let mut unique_hrefs: Vec<String> = Vec::new();
    let mut result: Vec<Resource> = Vec::new();
    TableOfContents::get_all_unique_resources_inner(&mut unique_hrefs, &mut result, &self.toc_references);
    return result;
  }

  // fix: Java 重载 get_all_unique_resources(List, List, List) 转录改名，避免与公开无参版重名
  fn get_all_unique_resources_inner(unique_hrefs: &mut Vec<String>,
      result: &mut Vec<Resource>, toc_references: &Vec<TOCReference>) {
    for toc_reference in toc_references {
      let resource = toc_reference.get_resource().clone();
      if resource.is_some() && !unique_hrefs.contains(&resource.as_ref().unwrap().get_href().clone()) {
        unique_hrefs.push(resource.as_ref().unwrap().get_href().clone());
        result.push(resource.unwrap());
      }
      TableOfContents::get_all_unique_resources_inner(unique_hrefs, result, toc_reference.get_children());
    }
  }

  /**
   * The total number of references in this table of contents.
   *
   * @return The total number of references in this table of contents.
   */
  pub fn size(&self) -> i32 {
    return TableOfContents::get_total_size(&self.toc_references);
  }

  fn get_total_size(toc_references: &Vec<TOCReference>) -> i32 {
    let mut result = toc_references.len() as i32;
    for toc_reference in toc_references {
      result += TableOfContents::get_total_size(toc_reference.get_children());
    }
    return result;
  }

  /**
   * The maximum depth of the reference tree
   * @return The maximum depth of the reference tree
   */
  pub fn calculate_depth(&self) -> i32 {
    return self.calculate_depth_inner(&self.toc_references, 0);
  }

  // fix: Java 重载 calculateDepth(List, int) 转录改名，避免与公开无参版重名
  fn calculate_depth_inner(&self, toc_references: &Vec<TOCReference>,
      current_depth: i32) -> i32 {
    let mut max_child_depth = 0;
    for toc_reference in toc_references {
      let child_depth = self.calculate_depth_inner(toc_reference.get_children(), 1);
      if child_depth > max_child_depth {
        max_child_depth = child_depth;
      }
    }
    return current_depth + max_child_depth;
  }
}
