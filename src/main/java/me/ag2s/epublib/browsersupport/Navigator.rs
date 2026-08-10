// package me.ag2s.epublib.browsersupport;

// import java.io.Serializable;
// import java.util.ArrayList;
// import java.util.List;

// import me.ag2s.epublib.domain.EpubBook;
// import me.ag2s.epublib.domain.Resource;

/**
 * A helper class for epub browser applications.
 * <p>
 * It helps moving from one resource to the other, from one resource
 * to the other and keeping other elements of the application up-to-date
 * by calling the NavigationEventListeners.
 *
 * @author paul
 */
pub struct Navigator {
  book: Option<EpubBook>,
  current_spine_pos: i32,
  current_resource: Option<Resource>,
  current_page_pos: i32,
  current_fragment_id: Option<String>,

  event_listeners: Vec<Box<dyn NavigationEventListener>>,
}

impl Navigator {

  pub fn new() -> Navigator {
    Navigator::with_book(None)
  }

  pub fn with_book(book: Option<EpubBook>) -> Navigator {
    let mut result = Navigator {
      book: book.clone(),
      current_spine_pos: 0,
      current_resource: None,
      current_page_pos: 0,
      current_fragment_id: None,
      event_listeners: Vec::new(),
    };
    if book.is_some() {
      result.current_resource = book.as_ref().unwrap().get_cover_page().clone();
    }
    result
  }

  fn handle_event_listeners(&self, navigation_event: &NavigationEvent) {
    for i in 0..self.event_listeners.len() {
      let navigation_event_listener = &self.event_listeners[i];
      navigation_event_listener.navigation_performed(navigation_event);
    }
  }

  pub fn add_navigation_event_listener(
      &mut self, navigation_event_listener: Box<dyn NavigationEventListener>) -> bool {
    self.event_listeners.push(navigation_event_listener);
    true
  }

  pub fn remove_navigation_event_listener(
      &mut self, navigation_event_listener: Box<dyn NavigationEventListener>) -> bool {
    let original_size = self.event_listeners.len();
    self.event_listeners.retain(|l| !std::ptr::eq(l.as_ref(), navigation_event_listener.as_ref()));
    self.event_listeners.len() != original_size
  }

  pub fn goto_first_spine_section(&mut self, source: &dyn Any) -> i32 {
    self.goto_spine_section(0, source)
  }

  pub fn goto_previous_spine_section(&mut self, source: &dyn Any) -> i32 {
    self.goto_previous_spine_section_with_pos(0, source)
  }

  pub fn goto_previous_spine_section_with_pos(&mut self, page_pos: i32, source: &dyn Any) -> i32 {
    if self.current_spine_pos < 0 {
      return self.goto_spine_section_with_pos(0, page_pos, source);
    } else {
      return self.goto_spine_section_with_pos(self.current_spine_pos - 1, page_pos, source);
    }
  }

  pub fn has_next_spine_section(&self) -> bool {
    (self.current_spine_pos < (self.book.as_ref().unwrap().get_spine().size() - 1))
  }

  pub fn has_previous_spine_section(&self) -> bool {
    (self.current_spine_pos > 0)
  }

  pub fn goto_next_spine_section(&mut self, source: &dyn Any) -> i32 {
    if self.current_spine_pos < 0 {
      return self.goto_spine_section(0, source);
    } else {
      return self.goto_spine_section(self.current_spine_pos + 1, source);
    }
  }

  pub fn goto_resource_href(&mut self, resource_href: String, source: &dyn Any) -> i32 {
    let resource = self.book.as_ref().unwrap().get_resources().get_by_href(&resource_href).clone();
    self.goto_resource(resource, source)
  }


  pub fn goto_resource(&mut self, resource: Option<Resource>, source: &dyn Any) -> i32 {
    self.goto_resource_with_fragment(resource, 0, None, source)
  }

  pub fn goto_resource_with_fragment_id(&mut self, resource: Option<Resource>, fragment_id: Option<String>, source: &dyn Any) -> i32 {
    self.goto_resource_with_fragment(resource, 0, fragment_id, source)
  }

  pub fn goto_resource_with_pos(&mut self, resource: Option<Resource>, page_pos: i32, source: &dyn Any) -> i32 {
    self.goto_resource_with_fragment(resource, page_pos, None, source)
  }

  pub fn goto_resource_with_fragment(&mut self, resource: Option<Resource>, page_pos: i32, fragment_id: Option<String>,
      source: &dyn Any) -> i32 {
    if resource.is_none() {
      return -1;
    }
    let navigation_event = NavigationEvent::with_navigator(source, self);
    self.current_resource = resource.clone();
    self.current_spine_pos = self.book.as_ref().unwrap().get_spine().get_resource_index(&self.current_resource);
    self.current_page_pos = page_pos;
    self.current_fragment_id = fragment_id;
    self.handle_event_listeners(&navigation_event);

    return self.current_spine_pos;
  }

  pub fn goto_resource_id(&mut self, resource_id: String, source: &dyn Any) -> i32 {
    self.goto_spine_section(self.book.as_ref().unwrap().get_spine().find_first_resource_by_id(&resource_id),
        source)
  }

  pub fn goto_spine_section(&mut self, new_spine_pos: i32, source: &dyn Any) -> i32 {
    self.goto_spine_section_with_pos(new_spine_pos, 0, source)
  }

  /**
   * Go to a specific section.
   * Illegal spine positions are silently ignored.
   *
   * @param newSpinePos f
   * @param source f
   * @return The current position within the spine
   */
  pub fn goto_spine_section_with_pos(&mut self, new_spine_pos: i32, new_page_pos: i32, source: &dyn Any) -> i32 {
    if new_spine_pos == self.current_spine_pos {
      return self.current_spine_pos;
    }
    if new_spine_pos < 0 || new_spine_pos >= self.book.as_ref().unwrap().get_spine().size() {
      return self.current_spine_pos;
    }
    let navigation_event = NavigationEvent::with_navigator(source, self);
    self.current_spine_pos = new_spine_pos;
    self.current_page_pos = new_page_pos;
    self.current_resource = self.book.as_ref().unwrap().get_spine().get_resource(self.current_spine_pos).clone();
    self.handle_event_listeners(&navigation_event);
    return self.current_spine_pos;
  }

  pub fn goto_last_spine_section(&mut self, source: &dyn Any) -> i32 {
    self.goto_spine_section(self.book.as_ref().unwrap().get_spine().size() - 1, source)
  }

  pub fn goto_book(&mut self, book: EpubBook, source: &dyn Any) {
    let navigation_event = NavigationEvent::with_navigator(source, self);
    self.book = Some(book);
    self.current_fragment_id = None;
    self.current_page_pos = 0;
    self.current_resource = None;
    self.current_spine_pos = self.book.as_ref().unwrap().get_spine().get_resource_index(&self.current_resource);
    self.handle_event_listeners(&navigation_event);
  }

  /**
   * The current position within the spine.
   *
   * @return something &lt; 0 if the current position is not within the spine.
   */
  pub fn get_current_spine_pos(&self) -> i32 {
    self.current_spine_pos
  }

  pub fn get_current_resource(&self) -> &Option<Resource> {
    &self.current_resource
  }

  /**
   * Sets the current index and resource without calling the eventlisteners.
   *
   * If you want the eventListeners called use gotoSection(index);
   *
   * @param currentIndex f
   */
  pub fn set_current_spine_pos(&mut self, current_index: i32) {
    self.current_spine_pos = current_index;
    self.current_resource = self.book.as_ref().unwrap().get_spine().get_resource(current_index).clone();
  }

  pub fn get_book(&self) -> &EpubBook {
    self.book.as_ref().unwrap()
  }

  /**
   * Sets the current index and resource without calling the eventlisteners.
   *
   * If you want the eventListeners called use gotoSection(index);
   *
   */
  pub fn set_current_resource(&mut self, current_resource: Option<Resource>) -> i32 {
    self.current_spine_pos = self.book.as_ref().unwrap().get_spine().get_resource_index(&current_resource);
    self.current_resource = current_resource;
    return self.current_spine_pos;
  }

  pub fn get_current_fragment_id(&self) -> &Option<String> {
    &self.current_fragment_id
  }

  pub fn get_current_section_pos(&self) -> i32 {
    self.current_page_pos
  }
}
