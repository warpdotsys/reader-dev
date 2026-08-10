// package me.ag2s.epublib.domain;

// import java.io.Serializable;

pub struct ResourceReference {
  resource: Option<Resource>,
}

impl ResourceReference {

  pub fn new(resource: Option<Resource>) -> ResourceReference {
    ResourceReference {
      resource: resource,
    }
  }


  pub fn get_resource(&self) -> &Option<Resource> {
    &self.resource
  }

  /**
   * Besides setting the resource it also sets the fragmentId to null.
   *
   * @param resource resource
   */
  pub fn set_resource(&mut self, resource: Option<Resource>) {
    self.resource = resource;
  }


  /**
   * The id of the reference referred to.
   *
   * null of the reference is null or has a null id itself.
   *
   * @return The id of the reference referred to.
   */
  pub fn get_resource_id(&self) -> Option<String> {
    if self.resource.is_some() {
      return Some(self.resource.as_ref().unwrap().get_id().clone());
    }
    return None;
  }
}
