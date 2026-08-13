use crate::prelude::*;
use std::collections::HashMap;

use crate::me::ag2s::epublib::domain::{Author, Date, Event, Identifier, Metadata};
use crate::me::ag2s::epublib::epub::{DOMUtil, PackageDocumentBase};
use crate::me::ag2s::epublib::util::StringUtil;
use crate::me_ag2s_epublib_epub_domutil::{Document, Element, Node, NodeList};
use crate::stubs::QName;

// fix: PackageDocumentBase 的这些关联常量是私有的（E0624），此处按原值以模块级常量镜像，保持原逻辑不变
const OPF_TAGS_METADATA: &'static str = "metadata";
const OPF_TAGS_META: &'static str = "meta";
const OPF_TAGS_PACKAGE_TAG: &'static str = "package";
const OPF_ATTRIBUTES_PROPERTY: &'static str = "property";
const OPF_ATTRIBUTES_NAME: &'static str = "name";
const OPF_ATTRIBUTES_CONTENT: &'static str = "content";
const OPF_ATTRIBUTES_UNIQUE_IDENTIFIER: &'static str = "unique-identifier";
const OPF_ATTRIBUTES_EVENT: &'static str = "event";
const OPF_ATTRIBUTES_ROLE: &'static str = "role";
const DC_TAGS_TITLE: &'static str = "title";
const DC_TAGS_CREATOR: &'static str = "creator";
const DC_TAGS_SUBJECT: &'static str = "subject";
const DC_TAGS_DESCRIPTION: &'static str = "description";
const DC_TAGS_PUBLISHER: &'static str = "publisher";
const DC_TAGS_CONTRIBUTOR: &'static str = "contributor";
const DC_TAGS_DATE: &'static str = "date";
const DC_TAGS_TYPE: &'static str = "type";
const DC_TAGS_IDENTIFIER: &'static str = "identifier";
const DC_TAGS_RIGHTS: &'static str = "rights";
const DC_TAGS_LANGUAGE: &'static str = "language";
const DC_ATTRIBUTES_SCHEME: &'static str = "scheme";

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
            &package_document.get_document_element(), PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_METADATA);
        if metadata_element.is_none() {
            // Log.e(TAG, "Package does not contain element " + OPFTags.metadata);
            eprintln!("{} {} Package does not contain element {}", PackageDocumentMetadataReader::TAG, " ", OPF_TAGS_METADATA);
            return result;
        }
        let metadata_element = metadata_element.unwrap();
        result.set_titles(DOMUtil::get_elements_text_child(&metadata_element, PackageDocumentBase::NAMESPACE_DUBLIN_CORE, DC_TAGS_TITLE));
        result.set_publishers(DOMUtil::get_elements_text_child(&metadata_element, PackageDocumentBase::NAMESPACE_DUBLIN_CORE, DC_TAGS_PUBLISHER));
        result.set_descriptions(DOMUtil::get_elements_text_child(&metadata_element, PackageDocumentBase::NAMESPACE_DUBLIN_CORE, DC_TAGS_DESCRIPTION));
        result.set_rights(DOMUtil::get_elements_text_child(&metadata_element, PackageDocumentBase::NAMESPACE_DUBLIN_CORE, DC_TAGS_RIGHTS));
        result.set_types(DOMUtil::get_elements_text_child(&metadata_element, PackageDocumentBase::NAMESPACE_DUBLIN_CORE, DC_TAGS_TYPE));
        result.set_subjects(DOMUtil::get_elements_text_child(&metadata_element, PackageDocumentBase::NAMESPACE_DUBLIN_CORE, DC_TAGS_SUBJECT));
        result.set_identifiers(Self::read_identifiers(&metadata_element));
        result.set_authors(Self::read_creators(&metadata_element));
        result.set_contributors(Self::read_contributors(&metadata_element));
        result.set_dates(Self::read_dates(&metadata_element));
        result.set_other_properties(Self::read_other_properties(&metadata_element));
        result.set_meta_attributes(Self::read_meta_properties(&metadata_element));
        let language_tag = DOMUtil::get_first_element_by_tag_name_ns(&metadata_element, PackageDocumentBase::NAMESPACE_DUBLIN_CORE, DC_TAGS_LANGUAGE);
        if let Some(language_tag) = language_tag {
            if let Some(language) = DOMUtil::get_text_children_content(&language_tag) {
                result.set_language(language);
            }
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
    fn read_other_properties(metadata_element: &Element) -> HashMap<QName, String> {
        let mut result: HashMap<QName, String> = HashMap::new();

        let meta_tags = metadata_element.get_elements_by_tag_name(OPF_TAGS_META);
        for i in 0..meta_tags.get_length() {
            let meta_node = meta_tags.item(i);
            let property = meta_node.get_attributes().get_named_item(OPF_ATTRIBUTES_PROPERTY);
            if let Some(property) = property {
                let name = property.get_node_value();
                let value = meta_node.get_text_content();
                result.insert(QName::new(name), value);
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

        let meta_tags = metadata_element.get_elements_by_tag_name(OPF_TAGS_META);
        for i in 0..meta_tags.get_length() {
            let meta_element = meta_tags.item(i);
            let name = meta_element.get_attribute(OPF_ATTRIBUTES_NAME);
            let value = meta_element.get_attribute(OPF_ATTRIBUTES_CONTENT);
            result.insert(name, value);
        }

        result
    }

    fn get_book_id_id(document: &Document) -> String {
        let package_element = DOMUtil::get_first_element_by_tag_name_ns(
            &document.get_document_element(), PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_PACKAGE_TAG);
        if package_element.is_none() {
            return String::new(); // fix: Java 返回 null, 以空串代替
        }
        DOMUtil::get_attribute(&package_element.unwrap(), PackageDocumentBase::NAMESPACE_OPF, OPF_ATTRIBUTES_UNIQUE_IDENTIFIER)
    }

    fn read_creators(metadata_element: &Element) -> Vec<Author> {
        Self::read_authors(DC_TAGS_CREATOR, metadata_element)
    }

    fn read_contributors(metadata_element: &Element) -> Vec<Author> {
        Self::read_authors(DC_TAGS_CONTRIBUTOR, metadata_element)
    }

    fn read_authors(author_tag: &str, metadata_element: &Element) -> Vec<Author> {
        let elements = metadata_element.get_elements_by_tag_name_ns(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, author_tag);
        let mut result = Vec::with_capacity(elements.get_length());
        for i in 0..elements.get_length() {
            let author_element = elements.item(i);
            let author = Self::create_author(&author_element);
            if let Some(author) = author {
                result.push(author);
            }
        }
        result
    }

    fn read_dates(metadata_element: &Element) -> Vec<Date> {
        let elements = metadata_element.get_elements_by_tag_name_ns(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, DC_TAGS_DATE);
        let mut result = Vec::with_capacity(elements.get_length());
        for i in 0..elements.get_length() {
            let date_element = elements.item(i);
            let date_string = DOMUtil::get_text_children_content(&date_element);
            if date_string.is_none() {
                // Log.e(TAG, e.getMessage());
                continue;
            }
            let event_string = DOMUtil::get_attribute(&date_element, PackageDocumentBase::NAMESPACE_OPF, OPF_ATTRIBUTES_EVENT);
            let date = Date::with_date_string_and_event(date_string.unwrap(), Event::from_value(&event_string));
            result.push(date);
        }
        result
    }

    fn create_author(author_element: &Element) -> Option<Author> {
        let author_string = DOMUtil::get_text_children_content(author_element);
        if author_string.is_none() || StringUtil::is_blank(author_string.as_deref().unwrap()) {
            return None;
        }
        let author_string = author_string.unwrap();
        let space_pos = author_string.rfind(' ');
        let mut result;
        if space_pos.is_none() {
            result = Author::new(author_string);
        } else {
            let space_pos = space_pos.unwrap();
            result = Author::with_names(author_string[0..space_pos].to_string(),
                author_string[space_pos + 1..].to_string());
        }
        result.set_role(DOMUtil::get_attribute(author_element, PackageDocumentBase::NAMESPACE_OPF, OPF_ATTRIBUTES_ROLE));
        Some(result)
    }

    fn read_identifiers(metadata_element: &Element) -> Vec<Identifier> {
        let identifier_elements = metadata_element.get_elements_by_tag_name_ns(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, DC_TAGS_IDENTIFIER);
        if identifier_elements.get_length() == 0 {
            // Log.e(TAG, "Package does not contain element " + DCTags.identifier);
            eprintln!("{} {} Package does not contain element {}", PackageDocumentMetadataReader::TAG, " ", DC_TAGS_IDENTIFIER);
            return Vec::new();
        }
        let book_id_id = Self::get_book_id_id(metadata_element.get_owner_document());
        let mut result = Vec::with_capacity(identifier_elements.get_length());
        for i in 0..identifier_elements.get_length() {
            let identifier_element = identifier_elements.item(i);
            let scheme_name = DOMUtil::get_attribute(&identifier_element, PackageDocumentBase::NAMESPACE_OPF, DC_ATTRIBUTES_SCHEME);
            let identifier_value = DOMUtil::get_text_children_content(&identifier_element);
            if identifier_value.is_none() || StringUtil::is_blank(identifier_value.as_deref().unwrap()) {
                continue;
            }
            let mut identifier = Identifier::with_value(scheme_name, identifier_value.unwrap());
            if identifier_element.get_attribute("id") == book_id_id {
                identifier.set_book_id(true);
            }
            result.push(identifier);
        }
        result
    }
}

// ---- DOM 类型复用 DOMUtil 的本地 stub（与 NCXDocumentV2 相同约定），此处仅补齐本文件所需方法 ----

impl Document {
    // fix: 占位实现, Java getDocumentElement 返回根 Element
    pub fn get_document_element(&self) -> Element {
        Element::null()
    }
}

impl Element {
    // fix: Java getOwnerDocument 返回所属 Document（无引用持有，返回空文档）
    pub fn get_owner_document(&self) -> &Document {
        static PLACEHOLDER: std::sync::OnceLock<Document> = std::sync::OnceLock::new();
        PLACEHOLDER.get_or_init(|| Document::new())
    }

    // fix: 占位实现, Java getAttributes 返回属性集合 NamedNodeMap
    pub fn get_attributes(&self) -> &NamedNodeMap {
        static PLACEHOLDER: NamedNodeMap = NamedNodeMap;
        &PLACEHOLDER
    }
}

impl Node {
    // fix: 占位实现, Java getTextContent 返回节点文本
    pub fn get_text_content(&self) -> String {
        String::new()
    }

    // fix: 占位实现, Java getNodeValue 返回属性值
    pub fn get_node_value(&self) -> String {
        String::new()
    }
}

pub struct NamedNodeMap;

impl NamedNodeMap {
    // fix: Java getNamedItem 未找到时返回 null, 以 Option 表达
    pub fn get_named_item(&self, _name: &str) -> Option<Node> {
        None
    }
}

