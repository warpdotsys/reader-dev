// package me.ag2s.epublib.domain;

// import me.ag2s.epublib.Constants;
// import me.ag2s.epublib.util.StringUtil;

// import java.io.Serializable;
// import java.util.ArrayList;
// import java.util.Arrays;
// import java.util.Collection;
// import java.util.HashMap;
// import java.util.List;
// import java.util.Map;

/**
 * All the resources that make up the book.
 * XHTML files, images and epub xml documents must be here.
 *
 * @author paul
 */
pub struct Resources {

    last_id: i32,

    resources: HashMap<String, Resource>,
}

impl Resources {

    const IMAGE_PREFIX: &'static str = "image_";
    const ITEM_PREFIX: &'static str = "item_";

    pub fn new() -> Resources {
        Resources {
            last_id: 1,
            resources: HashMap::new(),
        }
    }

    /**
     * Adds a resource to the resources.
     * <p>
     * Fixes the resources id and href if necessary.
     *
     * @param resource resource
     * @return the newly added resource
     */
    pub fn add(&mut self, mut resource: Resource) -> Resource {
        self.fix_resource_href(&mut resource);
        self.fix_resource_id(&mut resource);
        self.resources.insert(resource.get_href().clone(), resource.clone());
        return resource;
    }

    /**
     * Checks the id of the given resource and changes to a unique identifier if it isn't one already.
     *
     * @param resource resource
     */
    pub fn fix_resource_id(&mut self, resource: &mut Resource) {
        let mut resource_id = resource.get_id().clone();

        // first try and create a unique id based on the resource's href
        if StringUtil::is_blank(resource.get_id()) {
            resource_id = StringUtil::substring_before_last(resource.get_href(), '.');
            resource_id = StringUtil::substring_after_last(&resource_id, '/');
        }

        resource_id = self.make_valid_id(resource_id, resource);

        // check if the id is unique. if not: create one from scratch
        if StringUtil::is_blank(&resource_id) || self.contains_id(&resource_id) {
            resource_id = self.create_unique_resource_id(resource);
        }
        resource.set_id(resource_id);
    }

    /**
     * Check if the id is a valid identifier. if not: prepend with valid identifier
     *
     * @param resource resource
     * @return a valid id
     */
    fn make_valid_id(&self, mut resource_id: String, resource: &Resource) -> String {
        if StringUtil::is_not_blank(&resource_id) && !Character
                ::is_java_identifier_start(resource_id.chars().nth(0).unwrap()) {
            resource_id = self.get_resource_item_prefix(resource) + &resource_id;
        }
        return resource_id;
    }

    fn get_resource_item_prefix(&self, resource: &Resource) -> String {
        let result: String;
        if MediaTypes::is_bitmap_image(resource.get_media_type().as_ref().unwrap()) {
            result = Resources::IMAGE_PREFIX.to_string();
        } else {
            result = Resources::ITEM_PREFIX.to_string();
        }
        return result;
    }

    /**
     * Creates a new resource id that is guaranteed to be unique for this set of Resources
     *
     * @param resource resource
     * @return a new resource id that is guaranteed to be unique for this set of Resources
     */
    fn create_unique_resource_id(&mut self, resource: &Resource) -> String {
        let mut counter = self.last_id;
        if counter == i32::MAX {
            if self.resources.len() == i32::MAX as usize {
                panic!(
                        format!("Resources contains {} elements: no new elements can be added", i32::MAX));
            } else {
                counter = 1;
            }
        }
        let prefix = self.get_resource_item_prefix(resource);
        let mut result = prefix.clone() + &counter.to_string();
        while self.contains_id(&result) {
            counter += 1;
            result = prefix.clone() + &counter.to_string();
        }
        self.last_id = counter;
        return result;
    }

    /**
     * Whether the map of resources already contains a resource with the given id.
     *
     * @param id id
     * @return Whether the map of resources already contains a resource with the given id.
     */
    pub fn contains_id(&self, id: &String) -> bool {
        if StringUtil::is_blank(id) {
            return false;
        }
        for resource in self.resources.values() {
            if id.eq(resource.get_id()) {
                return true;
            }
        }
        return false;
    }

    /**
     * Gets the resource with the given id.
     *
     * @param id id
     * @return null if not found
     */
    pub fn get_by_id(&self, id: &String) -> Option<Resource> {
        if StringUtil::is_blank(id) {
            return None;
        }
        for resource in self.resources.values() {
            if id.eq(resource.get_id()) {
                return Some(resource.clone());
            }
        }
        return None;
    }

    pub fn get_by_properties(&self, properties: &String) -> Option<Resource> {
        if StringUtil::is_blank(properties) {
            return None;
        }
        for resource in self.resources.values() {
            if properties.eq(resource.get_properties()) {
                return Some(resource.clone());
            }
        }
        return None;
    }

    /**
     * Remove the resource with the given href.
     *
     * @param href href
     * @return the removed resource, null if not found
     */
    pub fn remove(&mut self, href: &String) -> Option<Resource> {
        return self.resources.remove(href);
    }

    fn fix_resource_href(&mut self, resource: &mut Resource) {
        if StringUtil::is_not_blank(resource.get_href())
                && !self.resources.contains_key(resource.get_href()) {
            return;
        }
        if StringUtil::is_blank(resource.get_href()) {
            if resource.get_media_type().is_none() {
                panic!(
                        "Resource must have either a MediaType or a href");
            }
            let mut i = 1;
            let mut href = self.create_href(resource.get_media_type().as_ref().unwrap(), i);
            while self.resources.contains_key(&href) {
                i += 1;
                href = self.create_href(resource.get_media_type().as_ref().unwrap(), i);
            }
            resource.set_href(href);
        }
    }

    fn create_href(&self, media_type: &MediaType, counter: i32) -> String {
        if MediaTypes::is_bitmap_image(media_type) {
            return Resources::IMAGE_PREFIX.to_string() + &counter.to_string() + media_type.get_default_extension();
        } else {
            return Resources::ITEM_PREFIX.to_string() + &counter.to_string() + media_type.get_default_extension();
        }
    }


    pub fn is_empty(&self) -> bool {
        return self.resources.is_empty();
    }

    /**
     * The number of resources
     *
     * @return The number of resources
     */
    pub fn size(&self) -> i32 {
        return self.resources.len() as i32;
    }

    /**
     * The resources that make up this book.
     * Resources can be xhtml pages, images, xml documents, etc.
     *
     * @return The resources that make up this book.
     */
    // @SuppressWarnings("unused")
    pub fn get_resource_map(&self) -> &HashMap<String, Resource> {
        &self.resources
    }

    pub fn get_all(&self) -> Vec<Resource> {
        return self.resources.values().cloned().collect();
    }


    /**
     * Whether there exists a resource with the given href
     *
     * @param href href
     * @return Whether there exists a resource with the given href
     */
    pub fn not_contains_by_href(&self, href: &String) -> bool {
        if StringUtil::is_blank(href) {
            return true;
        } else {
            return !self.resources.contains_key(
                    &StringUtil::substring_before(href, Constants::FRAGMENT_SEPARATOR_CHAR));
        }
    }
    /**
     * Whether there exists a resource with the given href
     *
     * @param href href
     * @return Whether there exists a resource with the given href
     */
    // @SuppressWarnings("unused")
    pub fn contains_by_href(&self, href: &String) -> bool {
        return !self.not_contains_by_href(href);
    }

    /**
     * Sets the collection of Resources to the given collection of resources
     *
     * @param resources resources
     */
    pub fn set_collection(&mut self, resources: &Vec<Resource>) {
        self.resources.clear();
        self.add_all(resources);
    }

    /**
     * Adds all resources from the given Collection of resources to the existing collection.
     *
     * @param resources resources
     */
    pub fn add_all(&mut self, resources: &Vec<Resource>) {
        for mut resource in resources.clone() {
            self.fix_resource_href(&mut resource);
            self.resources.insert(resource.get_href().clone(), resource);
        }
    }

    /**
     * Sets the collection of Resources to the given collection of resources
     *
     * @param resources A map with as keys the resources href and as values the Resources
     */
    pub fn set(&mut self, resources: HashMap<String, Resource>) {
        self.resources = resources.clone();
    }


    /**
     * First tries to find a resource with as id the given idOrHref, if that
     * fails it tries to find one with the idOrHref as href.
     *
     * @param idOrHref idOrHref
     * @return the found Resource
     */
    pub fn get_by_id_or_href(&self, id_or_href: &String) -> Option<Resource> {
        let mut resource = self.get_by_id(id_or_href);
        if resource.is_none() {
            resource = self.get_by_href(id_or_href);
        }
        return resource;
    }


    /**
     * Gets the resource with the given href.
     * If the given href contains a fragmentId then that fragment id will be ignored.
     *
     * @param href href
     * @return null if not found.
     */
    pub fn get_by_href(&self, href: &String) -> Option<Resource> {
        if StringUtil::is_blank(href) {
            return None;
        }
        let href = StringUtil::substring_before(href, Constants::FRAGMENT_SEPARATOR_CHAR);
        return self.resources.get(&href).cloned();
    }

    /**
     * Gets the first resource (random order) with the give mediatype.
     * <p>
     * Useful for looking up the table of contents as it's supposed to be the only resource with NCX mediatype.
     *
     * @param mediaType mediaType
     * @return the first resource (random order) with the give mediatype.
     */
    pub fn find_first_resource_by_media_type(&self, media_type: &MediaType) -> Option<Resource> {
        return Resources::find_first_resource_by_media_type(&self.get_all(), media_type);
    }

    /**
     * Gets the first resource (random order) with the give mediatype.
     * <p>
     * Useful for looking up the table of contents as it's supposed to be the only resource with NCX mediatype.
     *
     * @param mediaType mediaType
     * @return the first resource (random order) with the give mediatype.
     */
    pub fn find_first_resource_by_media_type(
            resources: &Vec<Resource>, media_type: &MediaType) -> Option<Resource> {
        for resource in resources {
            if resource.get_media_type().as_ref().unwrap() == media_type {
                return Some(resource.clone());
            }
        }
        return None;
    }

    /**
     * All resources that have the given MediaType.
     *
     * @param mediaType mediaType
     * @return All resources that have the given MediaType.
     */
    pub fn get_resources_by_media_type(&self, media_type: &MediaType) -> Vec<Resource> {
        let mut result: Vec<Resource> = Vec::new();
        if media_type.is_none() {
            return result;
        }
        for resource in self.get_all() {
            if resource.get_media_type().as_ref().unwrap() == media_type {
                result.push(resource);
            }
        }
        return result;
    }

    /**
     * All Resources that match any of the given list of MediaTypes
     *
     * @param mediaTypes mediaType
     * @return All Resources that match any of the given list of MediaTypes
     */
    // @SuppressWarnings("unused")
    pub fn get_resources_by_media_types(&self, media_types: &Vec<MediaType>) -> Vec<Resource> {
        let mut result: Vec<Resource> = Vec::new();
        if media_types.is_none() {
            return result;
        }

        // this is the fastest way of doing this according to
        // http://stackoverflow.com/questions/1128723/in-java-how-can-i-test-if-an-array-contains-a-certain-value
        let media_types_list = media_types.clone();
        for resource in self.get_all() {
            if media_types_list.contains(resource.get_media_type().as_ref().unwrap()) {
                result.push(resource);
            }
        }
        return result;
    }


    /**
     * All resource hrefs
     *
     * @return all resource hrefs
     */
    pub fn get_all_hrefs(&self) -> Vec<String> {
        return self.resources.keys().cloned().collect();
    }
}
