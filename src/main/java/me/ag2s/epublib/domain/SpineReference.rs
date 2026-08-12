use crate::prelude::*;
// package me.ag2s.epublib.domain;

// import java.io.Serializable;


/**
 * A Section of a book.
 * Represents both an item in the package document and a item in the index.
 *
 * @author paul
 */
// fix: 补充 Clone（Spine.add_spine_reference 需要；Resource 已实现 Clone）
#[derive(Clone)]
pub struct SpineReference {
    resource: Option<Resource>,
    linear: bool,
}

impl SpineReference {

    pub fn new(resource: Option<Resource>) -> SpineReference {
        SpineReference::with_linear(resource, true)
    }


    pub fn with_linear(resource: Option<Resource>, linear: bool) -> SpineReference {
        SpineReference {
            resource: resource,
            linear: linear,
        }
    }

    /**
     * Linear denotes whether the section is Primary or Auxiliary.
     * Usually the cover page has linear set to false and all the other sections
     * have it set to true.
     * <p>
     * It's an optional property that readers may also ignore.
     *
     * <blockquote>primary or auxiliary is useful for Reading Systems which
     * opt to present auxiliary content differently than primary content.
     * For example, a Reading System might opt to render auxiliary content in
     * a popup window apart from the main window which presents the primary
     * content. (For an example of the types of content that may be considered
     * auxiliary, refer to the example below and the subsequent discussion.)</blockquote>
     *
     * @return whether the section is Primary or Auxiliary.
     * @see <a href="http://www.idpf.org/epub/20/spec/OPF_2.0.1_draft.htm#Section2.4">OPF Spine specification</a>
     */
    pub fn is_linear(&self) -> bool {
        return self.linear;
    }

    pub fn set_linear(&mut self, linear: bool) {
        self.linear = linear;
    }

    pub fn get_resource(&self) -> &Option<Resource> {
        &self.resource
    }

    pub fn get_resource_id(&self) -> Option<String> {
        if self.resource.is_some() {
            return Some(self.resource.as_ref().unwrap().get_id().clone());
        }
        return None;
    }

}
