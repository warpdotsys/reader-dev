// package me.ag2s.epublib.browsersupport;

// import java.util.EventObject;

// import me.ag2s.epublib.domain.EpubBook;
// import me.ag2s.epublib.domain.Resource;
// import me.ag2s.epublib.util.StringUtil;

/**
 * Used to tell NavigationEventListener just what kind of navigation action
 * the user just did.
 *
 * @author paul
 *
 */
// @SuppressWarnings("unused")
pub struct NavigationEvent {
  old_resource: Option<Resource>,
  old_spine_pos: i32,
    navigator: Option<Navigator>,
    old_book: Option<EpubBook>,
    old_section_pos: i32,
  old_fragment_id: Option<String>,
}

impl NavigationEvent {

  pub fn new(source: &dyn Any) -> NavigationEvent {
    NavigationEvent {
      old_resource: None,
      old_spine_pos: 0,
      navigator: None,
      old_book: None,
      old_section_pos: 0,
      old_fragment_id: None,
    }
  }

  pub fn with_navigator(source: &dyn Any, navigator: &Navigator) -> NavigationEvent {
    let mut result = NavigationEvent::new(source);
    result.navigator = Some(navigator.clone());
    result.old_book = Some(navigator.get_book().clone());
    result.old_fragment_id = Some(navigator.get_current_fragment_id().clone());
    result.old_section_pos = navigator.get_current_section_pos();
    result.old_resource = Some(navigator.get_current_resource().clone());
    result.old_spine_pos = navigator.get_current_spine_pos();
    result
  }

  /**
   * The previous position within the section.
   *
   * @return The previous position within the section.
   */
  pub fn get_old_section_pos(&self) -> i32 {
    self.old_section_pos
  }

  pub fn get_navigator(&self) -> &Navigator {
    self.navigator.as_ref().unwrap()
  }

  pub fn get_old_fragment_id(&self) -> &String {
    self.old_fragment_id.as_ref().unwrap()
  }

  // package
  fn set_old_fragment_id(&mut self, old_fragment_id: String) {
    self.old_fragment_id = Some(old_fragment_id);
  }

    pub fn get_old_book(&self) -> &EpubBook {
        self.old_book.as_ref().unwrap()
    }

  // package
  fn set_old_page_pos(&mut self, old_page_pos: i32) {
    self.old_section_pos = old_page_pos;
  }

  pub fn get_current_section_pos(&self) -> i32 {
    self.get_navigator().get_current_section_pos()
  }

  pub fn get_old_spine_pos(&self) -> i32 {
    self.old_spine_pos
  }

  pub fn get_current_spine_pos(&self) -> i32 {
    self.get_navigator().get_current_spine_pos()
  }

  pub fn get_current_fragment_id(&self) -> &String {
    self.get_navigator().get_current_fragment_id()
  }

  pub fn is_book_changed(&self) -> bool {
    if self.old_book.is_none() {
      return true;
    }
    self.old_book.as_ref().unwrap() != self.get_navigator().get_book()
  }

  pub fn is_spine_pos_changed(&self) -> bool {
    self.get_old_spine_pos() != self.get_current_spine_pos()
  }

  pub fn is_fragment_changed(&self) -> bool {
    StringUtil::equals(self.get_old_fragment_id(), self.get_current_fragment_id())
  }

  pub fn get_old_resource(&self) -> &Resource {
    self.old_resource.as_ref().unwrap()
  }

  pub fn get_current_resource(&self) -> &Resource {
    self.get_navigator().get_current_resource()
  }

  pub fn set_old_resource(&mut self, old_resource: Resource) {
    self.old_resource = Some(old_resource);
  }


  pub fn set_old_spine_pos(&mut self, old_spine_pos: i32) {
    self.old_spine_pos = old_spine_pos;
  }


  pub fn set_navigator(&mut self, navigator: Navigator) {
    self.navigator = Some(navigator);
  }


    pub fn set_old_book(&mut self, old_book: EpubBook) {
        self.old_book = Some(old_book);
    }

    pub fn get_current_book(&self) -> &EpubBook {
        self.get_navigator().get_book()
    }

  pub fn is_resource_changed(&self) -> bool {
    self.old_resource.as_ref().unwrap() != self.get_current_resource()
  }

  // @SuppressWarnings("NullableProblems")
  pub fn to_string(&self) -> String {
    StringUtil::to_string(
        &vec![
        ("oldSectionPos", Some(&self.old_section_pos.to_string())),
        ("oldResource", self.old_resource.as_ref().map(|v| v.to_string())),
        ("oldBook", self.old_book.as_ref().map(|v| v.to_string())),
        ("oldFragmentId", self.old_fragment_id.as_ref().map(|v| v.clone())),
        ("oldSpinePos", Some(&self.old_spine_pos.to_string())),
        ("currentPagePos", Some(&self.get_current_section_pos().to_string())),
        ("currentResource", Some(&self.get_current_resource().to_string())),
        ("currentBook", Some(&self.get_current_book().to_string())),
        ("currentFragmentId", Some(&self.get_current_fragment_id().clone())),
        ("currentSpinePos", Some(&self.get_current_spine_pos().to_string()))
    ])
  }

  pub fn is_section_pos_changed(&self) -> bool {
    self.old_section_pos != self.get_current_section_pos()
  }
}
