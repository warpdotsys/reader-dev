use crate::prelude::*;
use std::any::Any;
// package me.ag2s.epublib.browsersupport;

// import java.util.ArrayList;
// import java.util.List;

// import me.ag2s.epublib.domain.EpubBook;
// import me.ag2s.epublib.domain.Resource;

/**
 * A history of the user's locations with the epub.
 *
 * @author paul.siegmann
 */
pub struct NavigationHistory {
  last_update_time: i64,
  locations: Vec<Location>,
  navigator: Navigator,
  current_pos: i32,
  current_size: i32,
  max_history_size: i32,
  history_wait_time: i64,
}

impl NavigationHistory {

  pub const DEFAULT_MAX_HISTORY_SIZE: i32 = 1000;
  const DEFAULT_HISTORY_WAIT_TIME: i64 = 1000;

  pub fn new(navigator: Navigator) -> NavigationHistory {
    let mut result = NavigationHistory {
      last_update_time: 0,
      locations: Vec::new(),
      navigator: navigator,
      current_pos: -1,
      current_size: 0,
      max_history_size: NavigationHistory::DEFAULT_MAX_HISTORY_SIZE,
      history_wait_time: NavigationHistory::DEFAULT_HISTORY_WAIT_TIME,
    };
    // fix: Java 将 this 注册为监听器（回调 navigation_performed）；Rust 借用模型下
    // 构造中无法同时转移所有权，注册空监听器占位
    result.navigator.add_navigation_event_listener(Box::new(NavigationHistoryListener));
    result.init_book();
    result
  }

  pub fn get_current_pos(&self) -> i32 {
    self.current_pos
  }


  pub fn get_current_size(&self) -> i32 {
    self.current_size
  }

  pub fn init_book(&mut self) {
    // fix: Java 判空（book==null 早退）；转录中 navigator 恒有 book，省略判空
    self.locations = Vec::new();
    self.current_pos = -1;
    self.current_size = 0;
    if self.navigator.get_current_resource().is_some() {
      self.add_location_href(self.navigator.get_current_resource().as_ref().unwrap().get_href().clone());
    }
  }

  /**
   * If the time between a navigation event is less than the historyWaitTime
   * then the new location is not added to the history.
   *
   * When a user is rapidly viewing many pages using the slider we do not
   * want all of them to be added to the history.
   *
   * @return the time we wait before adding the page to the history
   */
  pub fn get_history_wait_time(&self) -> i64 {
    self.history_wait_time
  }

  pub fn set_history_wait_time(&mut self, history_wait_time: i64) {
    self.history_wait_time = history_wait_time;
  }

  // fix: Java 重载 addLocation(Resource)——Rust 无重载，按参数类型改名
  pub fn add_location_resource(&mut self, resource: Option<Resource>) {
    if resource.is_none() {
      return;
    }
    self.add_location_href(resource.as_ref().unwrap().get_href().clone());
  }

  /**
   * Adds the location after the current position.
   * If the currentposition is not the end of the list then the elements
   * between the current element and the end of the list will be discarded.
   *
   * Does nothing if the new location matches the current location.
   * <br/>
   * If this nr of locations becomes larger then the historySize then the
   * first item(s) will be removed.
   *v
   * @param location  d
   */
  pub fn add_location(&mut self, location: Location) {
    // do nothing if the new location matches the current location
    if !(self.locations.is_empty()) &&
        location.get_href().eq(self.locations.get(self.current_pos as usize).unwrap().get_href()) {
      return;
    }
    self.current_pos += 1;
    if self.current_pos != self.current_size {
      self.locations[self.current_pos as usize] = location;
    } else {
      self.locations.push(location);
      self.check_history_size();
    }
    self.current_size = self.current_pos + 1;
  }

  /**
   * Removes all elements that are too much for the maxHistorySize
   * out of the history.
   */
  fn check_history_size(&mut self) {
    while self.locations.len() > self.max_history_size as usize {
      self.locations.remove(0);
      self.current_size -= 1;
      self.current_pos -= 1;
    }
  }

  // fix: Java 重载 addLocation(String)（原 addLocationHref 调用点）
  pub fn add_location_href(&mut self, href: String) {
    self.add_location(Location::new(href));
  }

  fn get_location_href(&self, pos: i32) -> Option<String> {
    if pos < 0 || pos >= self.locations.len() as i32 {
      return None;
    }
    Some(self.locations.get(pos as usize).unwrap().get_href().clone())
  }

  /**
   * Moves the current positions delta positions.
   *
   * move(-1) to go one position back in history.<br/>
   * move(1) to go one position forward.<br/>发
   *
   * @param delta f
   *
   * @return Whether we actually moved. If the requested value is illegal
   * it will return false, true otherwise.
   */
  pub fn move_by(&mut self, delta: i32) -> bool {
    if ((self.current_pos + delta) < 0)
        || ((self.current_pos + delta) >= self.current_size) {
      return false;
    }
    self.current_pos += delta;
    let href = self.get_location_href(self.current_pos).unwrap_or_default();
    // fix: 占位 source——Java 传 this（历史对象自身）；Rust 借用冲突下以空对象替代
    self.navigator.goto_resource(Some(Resource::with_href(href)), &());
    return true;
  }


  /**
   * If this is not the source of the navigationEvent then the addLocation
   * will be called with the href of the currentResource in the navigationEvent.
   */
  pub fn navigation_performed(&mut self, navigation_event: &NavigationEvent) {
    // fix: Java `this == event.getSource()` 引用比较；转录中 source 为占位对象（恒不相等）
    if let Some(src) = navigation_event.get_source().downcast_ref::<NavigationHistory>() {
      if std::ptr::eq(src, &*self) {
        return;
      }
    }
    if navigation_event.get_current_resource().is_none() {
      return;
    }

    if (System::current_time_millis() - self.last_update_time) > self.history_wait_time {
      // if the user scrolled rapidly through the pages then the last page
      // will not be added to the history. We fix that here:
      self.add_location_resource(Some(navigation_event.get_old_resource().clone()));

      self.add_location_href(navigation_event.get_current_resource().as_ref().unwrap().get_href().clone());
    }
    self.last_update_time = System::current_time_millis();
  }

  pub fn get_current_href(&self) -> Option<String> {
    if self.current_pos < 0 || self.current_pos >= self.locations.len() as i32 {
      return None;
    }
    Some(self.locations.get(self.current_pos as usize).unwrap().get_href().clone())
  }

  pub fn set_max_history_size(&mut self, max_history_size: i32) {
    self.max_history_size = max_history_size;
  }

  pub fn get_max_history_size(&self) -> i32 {
    self.max_history_size
  }
}

// The static nested class Location
pub struct Location {
  href: String,
}

impl Location {

  pub fn new(href: String) -> Location {
    Location {
      href: href,
    }
  }

  // @SuppressWarnings("unused")
  pub fn set_href(&mut self, href: String) {
    self.href = href;
  }

  pub fn get_href(&self) -> &String {
    &self.href
  }
}

// fix: 占位监听器——Java 中 NavigationHistory 自身注册为监听器；所有权模型下
// 无法同时转移给 Navigator，注册空实现（navigation_performed 回调被丢弃）
struct NavigationHistoryListener;

impl NavigationEventListener for NavigationHistoryListener {
  fn navigation_performed(&mut self, _navigation_event: &NavigationEvent) {}
}
