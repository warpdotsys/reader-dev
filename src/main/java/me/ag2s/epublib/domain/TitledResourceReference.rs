use crate::prelude::*;
// package me.ag2s.epublib.domain;

// import java.io.Serializable;

// import me.ag2s.epublib.Constants;
// import me.ag2s.epublib.util.StringUtil;

pub struct TitledResourceReference {
    resource: Option<Resource>,
    fragment_id: Option<String>,
    title: Option<String>,
}

impl TitledResourceReference {

    /**
     * 这会使title为null
     *
     * @param resource resource
     */
    // @Deprecated
    // @SuppressWarnings("unused")
    pub fn new(resource: Option<Resource>) -> TitledResourceReference {
        TitledResourceReference::with_title(resource, None)
    }

    pub fn with_title(resource: Option<Resource>, title: Option<String>) -> TitledResourceReference {
        TitledResourceReference::with_fragment(resource, title, None)
    }

    pub fn with_fragment(resource: Option<Resource>, title: Option<String>,
                                   fragment_id: Option<String>) -> TitledResourceReference {
        TitledResourceReference {
            resource: resource,
            title: title,
            fragment_id: fragment_id,
        }
    }

    pub fn get_fragment_id(&self) -> &Option<String> {
        &self.fragment_id
    }

    pub fn set_fragment_id(&mut self, fragment_id: Option<String>) {
        self.fragment_id = fragment_id;
    }

    pub fn get_title(&self) -> &Option<String> {
        &self.title
    }

    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }


    /**
     * If the fragmentId is blank it returns the resource href, otherwise
     * it returns the resource href + '#' + the fragmentId.
     *
     * @return If the fragmentId is blank it returns the resource href,
     * otherwise it returns the resource href + '#' + the fragmentId.
     */
    pub fn get_complete_href(&self) -> String {
        if StringUtil::is_blank(self.fragment_id.as_ref().unwrap_or(&String::new())) {
            return self.resource.as_ref().unwrap().get_href().clone();
        } else {
            // fix: Constants 为 trait，无法直接引用关联常量；Java 中 FRAGMENT_SEPARATOR_CHAR == '#'
            return self.resource.as_ref().unwrap().get_href().clone() + &'#'.to_string()
                    + self.fragment_id.as_ref().unwrap();
        }
    }

    // @Override
    pub fn get_resource(&self) -> &Option<Resource> {
        //resource为null时不设置标题
        if self.resource.is_some() && self.title.is_some() {
            self.resource.as_ref().unwrap().clone().set_title(self.title.clone().unwrap());
        }

        return &self.resource;
    }

    pub fn set_resource_with_fragment(&mut self, resource: Option<Resource>, fragment_id: Option<String>) {
        self.resource = resource;
        self.fragment_id = fragment_id;
    }

    /**
     * Sets the resource to the given resource and sets the fragmentId to None.
     */
    pub fn set_resource(&mut self, resource: Option<Resource>) {
        self.set_resource_with_fragment(resource, None);
    }
}
