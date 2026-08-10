use std::collections::HashMap;

use crate::me::ag2s::epublib::domain::{Author, Date, Identifier, Metadata};
use crate::me::ag2s::epublib::epub::{DOMUtil, PackageDocumentBase};
use crate::me::ag2s::epublib::util::StringUtil;

/**
 * Reads the package document metadata.
 * <p>
 * In its own separate class because the PackageDocumentReader became a bit large and unwieldy.
 *
 * @author paul
 */
// package
pub struct PackageDocumentMetadataReader;

impl PackageDocumentMetadataReader {

    const TAG: &'static str = "me.ag2s.epublib.epub.PackageDocumentMetadataReader";

    pub fn read_metadata(package_document: &Document) -> Metadata {
        let mut result = Metadata::new();
        let metadata_element = DOMUtil::get_first_element_by_tag_name_ns(
            package_document.get_document_element(), PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_METADATA);
        if metadata_element == null {
            // Log.e(TAG, "Package does not contain element " + OPFTags.metadata);
            eprintln!("{} {} Package does not contain element {}", PackageDocumentMetadataReader::TAG, " ", PackageDocumentBase::OPF_TAGS_METADATA);
            return result;
        }
        result.set_titles(DOMUtil::get_elements_text_child(&metadata_element, PackageDocumentBase::NAMESPACE_DUBLIN_CORE, PackageDocumentBase::DC_TAGS_TITLE));
        result.set_publishers(DOMUtil::get_elements_text_child(&metadata_element, PackageDocumentBase::NAMESPACE_DUBLIN_CORE, PackageDocumentBase::DC_TAGS_PUBLISHER));
        result.set_descriptions(DOMUtil::get_elements_text_child(&metadata_element, PackageDocumentBase::NAMESPACE_DUBLIN_CORE, PackageDocumentBase::DC_TAGS_DESCRIPTION));
        result.set_rights(DOMUtil::get_elements_text_child(&metadata_element, PackageDocumentBase::NAMESPACE_DUBLIN_CORE, PackageDocumentBase::DC_TAGS_RIGHTS));
        result.set_types(DOMUtil::get_elements_text_child(&metadata_element, PackageDocumentBase::NAMESPACE_DUBLIN_CORE, PackageDocumentBase::DC_TAGS_TYPE));
        result.set_subjects(DOMUtil::get_elements_text_child(&metadata_element, PackageDocumentBase::NAMESPACE_DUBLIN_CORE, PackageDocumentBase::DC_TAGS_SUBJECT));
        result.set_identifiers(read_identifiers(&metadata_element));
        result.set_authors(read_creators(&metadata_element));
        result.set_contributors(read_contributors(&metadata_element));
        result.set_dates(read_dates(&metadata_element));
        result.set_other_properties(read_other_properties(&metadata_element));
        result.set_meta_attributes(read_meta_properties(&metadata_element));
        let language_tag = DOMUtil::get_first_element_by_tag_name_ns(&metadata_element, PackageDocumentBase::NAMESPACE_DUBLIN_CORE, PackageDocumentBase::DC_TAGS_LANGUAGE);
        if language_tag != null {
            result.set_language(DOMUtil::get_text_children_content(language_tag));
        }

        result
    }

    /**
     * consumes meta tags that have a property attribute as defined in the standard. For example:
     * &lt;meta property="rendition:layout"&gt;pre-paginated&lt;/meta&gt;
     *
     * @param metadataElement metadataElement
     * @return Map<QName, String>
     */
    fn read_other_properties(metadata_element: &Element) -> HashMap<String, String> {
        let mut result = HashMap::new();

        let meta_tags = metadata_element.get_elements_by_tag_name(PackageDocumentBase::OPF_TAGS_META);
        for i in 0..meta_tags.get_length() {
            let meta_node = meta_tags.item(i);
            let property = meta_node.get_attributes().get_named_item(PackageDocumentBase::OPF_ATTRIBUTES_PROPERTY);
            if property != null {
                let name = property.get_node_value();
                let value = meta_node.get_text_content();
                result.insert(name, value);
            }
        }

        result
    }

    /**
     * consumes meta tags that have a property attribute as defined in the standard. For example:
     * &lt;meta property="rendition:layout"&gt;pre-paginated&lt;/meta&gt;
     *
     * @param metadataElement metadataElement
     * @return Map<String, String>
     */
    fn read_meta_properties(metadata_element: &Element) -> HashMap<String, String> {
        let mut result = HashMap::new();

        let meta_tags = metadata_element.get_elements_by_tag_name(PackageDocumentBase::OPF_TAGS_META);
        for i in 0..meta_tags.get_length() {
            let meta_element = meta_tags.item(i);
            let name = meta_element.get_attribute(PackageDocumentBase::OPF_ATTRIBUTES_NAME);
            let value = meta_element.get_attribute(PackageDocumentBase::OPF_ATTRIBUTES_CONTENT);
            result.insert(name, value);
        }

        result
    }

    fn get_book_id_id(document: &Document) -> String {
        let package_element = DOMUtil::get_first_element_by_tag_name_ns(
            document.get_document_element(), PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_PACKAGE_TAG);
        if package_element == null {
            return null;
        }
        DOMUtil::get_attribute(package_element, PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_ATTRIBUTES_UNIQUE_IDENTIFIER)
    }

    fn read_creators(metadata_element: &Element) -> Vec<Author> {
        read_authors(PackageDocumentBase::DC_TAGS_CREATOR, metadata_element)
    }

    fn read_contributors(metadata_element: &Element) -> Vec<Author> {
        read_authors(PackageDocumentBase::DC_TAGS_CONTRIBUTOR, metadata_element)
    }

    fn read_authors(author_tag: &str, metadata_element: &Element) -> Vec<Author> {
        let elements = metadata_element.get_elements_by_tag_name_ns(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, author_tag);
        let mut result = Vec::with_capacity(elements.get_length());
        for i in 0..elements.get_length() {
            let author_element = elements.item(i);
            let author = create_author(author_element);
            if author != null {
                result.push(author);
            }
        }
        result
    }

    fn read_dates(metadata_element: &Element) -> Vec<Date> {
        let elements = metadata_element.get_elements_by_tag_name_ns(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, PackageDocumentBase::DC_TAGS_DATE);
        let mut result = Vec::with_capacity(elements.get_length());
        for i in 0..elements.get_length() {
            let date_element = elements.item(i);
            let date;
            match Date::new(DOMUtil::get_text_children_content(date_element),
                DOMUtil::get_attribute(date_element, PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_ATTRIBUTES_EVENT)) {
                Ok(d) => {
                    date = d;
                    result.push(date);
                }
                Err(_e) => {
                    // Log.e(TAG, e.getMessage());
                    e.printStackTrace();
                }
            }
        }
        result
    }

    fn create_author(author_element: &Element) -> Author {
        let author_string = DOMUtil::get_text_children_content(author_element);
        if StringUtil::is_blank(&author_string) {
            return null;
        }
        let space_pos = author_string.rfind(' ');
        let result;
        if space_pos < 0 {
            result = Author::new(author_string);
        } else {
            result = Author::new_full(author_string[0..space_pos].to_string(),
                author_string[space_pos + 1..].to_string());
        }
        result.set_role(DOMUtil::get_attribute(author_element, PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_ATTRIBUTES_ROLE));
        result
    }

    fn read_identifiers(metadata_element: &Element) -> Vec<Identifier> {
        let identifier_elements = metadata_element.get_elements_by_tag_name_ns(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, PackageDocumentBase::DC_TAGS_IDENTIFIER);
        if identifier_elements.get_length() == 0 {
            // Log.e(TAG, "Package does not contain element " + DCTags.identifier);
            eprintln!("{} {} Package does not contain element {}", PackageDocumentMetadataReader::TAG, " ", PackageDocumentBase::DC_TAGS_IDENTIFIER);
            return Vec::new();
        }
        let book_id_id = get_book_id_id(metadata_element.get_owner_document());
        let mut result = Vec::with_capacity(identifier_elements.get_length());
        for i in 0..identifier_elements.get_length() {
            let identifier_element = identifier_elements.item(i);
            let scheme_name = DOMUtil::get_attribute(identifier_element, PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::DC_ATTRIBUTES_SCHEME);
            let identifier_value = DOMUtil::get_text_children_content(identifier_element);
            if StringUtil::is_blank(&identifier_value) {
                continue;
            }
            let mut identifier = Identifier::new(scheme_name, identifier_value);
            if identifier_element.get_attribute("id") == book_id_id {
                identifier.set_book_id(true);
            }
            result.push(identifier);
        }
        result
    }
}

pub struct Document;
pub struct Element;
pub struct NodeList;
pub struct Node;
pub struct NamedNodeMap;
pub struct QName;

impl Document {
    pub fn get_document_element(&self) -> Element { todo!() }
}

impl Element {
    pub fn get_elements_by_tag_name(&self, _name: &str) -> NodeList { todo!() }
    pub fn get_elements_by_tag_name_ns(&self, _namespace: &str, _name: &str) -> NodeList { todo!() }
    pub fn get_attribute(&self, _name: &str) -> String { todo!() }
    pub fn get_owner_document(&self) -> &Document { todo!() }
}

impl NodeList {
    pub fn get_length(&self) -> usize { todo!() }
    pub fn item(&self, _i: usize) -> Element { todo!() }
}

impl Node {
    pub fn get_attributes(&self) -> &NamedNodeMap { todo!() }
    pub fn get_text_content(&self) -> String { todo!() }
}

impl NamedNodeMap {
    pub fn get_named_item(&self, _name: &str) -> Node { todo!() }
}

impl Node {
    pub fn get_node_value(&self) -> String { todo!() }
}
