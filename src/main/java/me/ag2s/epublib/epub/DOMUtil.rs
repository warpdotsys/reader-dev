use crate::prelude::*;
use crate::me::ag2s::epublib::util::StringUtil;

/**
 * Utility methods for working with the DOM.
 *
 * @author paul
 */
// package
pub struct DOMUtil;

impl DOMUtil {

    /**
     * First tries to get the attribute value by doing an getAttributeNS on the element, if that gets an empty element it does a getAttribute without namespace.
     *
     * @param element   element
     * @param namespace namespace
     * @param attribute attribute
     * @return String Attribute
     */
    pub fn get_attribute(element: &Element, namespace: &str, attribute: &str) -> String {
        let mut result = element.get_attribute_ns(namespace, attribute);
        if StringUtil::is_empty(&result) {
            result = element.get_attribute(attribute);
        }
        result
    }

    /**
     * Gets all descendant elements of the given parentElement with the given namespace and tagname and returns their text child as a list of String.
     *
     * @param parentElement parentElement
     * @param namespace     namespace
     * @param tagName       tagName
     * @return List<String>
     */
    pub fn get_elements_text_child(parent_element: &Element, namespace: &str, tag_name: &str) -> Vec<String> {
        let elements = parent_element.get_elements_by_tag_name_ns(namespace, tag_name);
        //ArrayList 初始化时指定长度提高性能
        let mut result = Vec::with_capacity(elements.get_length());
        for i in 0..elements.get_length() {
            // fix: item 返回所有权值需借用; Java 直接 add(null), Rust 以空串占位
            result.push(Self::get_text_children_content(&elements.item(i)).unwrap_or_default());
        }
        result
    }

    /**
     * Finds in the current document the first element with the given namespace and elementName and with the given findAttributeName and findAttributeValue.
     * It then returns the value of the given resultAttributeName.
     *
     * @param document            document
     * @param namespace           namespace
     * @param elementName         elementName
     * @param findAttributeName   findAttributeName
     * @param findAttributeValue  findAttributeValue
     * @param resultAttributeName resultAttributeName
     * @return String value
     */
    pub fn get_find_attribute_value(document: &Document, namespace: &str, element_name: &str, find_attribute_name: &str, find_attribute_value: &str, result_attribute_name: &str) -> Option<String> {
        let meta_tags = document.get_elements_by_tag_name_ns(namespace, element_name);
        for i in 0..meta_tags.get_length() {
            let meta_element = meta_tags.item(i);
            if find_attribute_value.eq_ignore_ascii_case(&meta_element.get_attribute(find_attribute_name))
                && StringUtil::is_not_blank(&meta_element.get_attribute(result_attribute_name)) {
                return Some(meta_element.get_attribute(result_attribute_name));
            }
        }
        None
    }

    /**
     * Gets the first element that is a child of the parentElement and has the given namespace and tagName
     *
     * @param parentElement parentElement
     * @param namespace     namespace
     * @param tagName       tagName
     * @return Element
     */
    pub fn get_elements_by_tag_name_ns(parent_element: &Element, namespace: &str, tag_name: &str) -> Option<NodeList> {
        let mut nodes = parent_element.get_elements_by_tag_name_ns(namespace, tag_name);
        if nodes.get_length() != 0 {
            return Some(nodes);
        }
        nodes = parent_element.get_elements_by_tag_name(tag_name);
        if nodes.get_length() == 0 {
            return None;
        }
        Some(nodes)
    }

    /**
     * Gets the first element that is a child of the parentElement and has the given namespace and tagName
     *
     * @param parentElement parentElement
     * @param namespace     namespace
     * @param tagName       tagName
     * @return Element
     */
    pub fn get_elements_by_tag_name_ns_doc(parent_element: &Document, namespace: &str, tag_name: &str) -> Option<NodeList> {
        let mut nodes = parent_element.get_elements_by_tag_name_ns(namespace, tag_name);
        if nodes.get_length() != 0 {
            return Some(nodes);
        }
        nodes = parent_element.get_elements_by_tag_name(tag_name);
        if nodes.get_length() == 0 {
            return None;
        }
        Some(nodes)
    }

    /**
     * Gets the first element that is a child of the parentElement and has the given namespace and tagName
     *
     * @param parentElement parentElement
     * @param namespace     namespace
     * @param tagName       tagName
     * @return Element
     */
    pub fn get_first_element_by_tag_name_ns(parent_element: &Element, namespace: &str, tag_name: &str) -> Option<Element> {
        let nodes = parent_element.get_elements_by_tag_name_ns(namespace, tag_name);
        if nodes.get_length() != 0 {
            return Some(nodes.item(0));
        }
        let nodes = parent_element.get_elements_by_tag_name(tag_name);
        if nodes.get_length() == 0 {
            return None;
        }
        Some(nodes.item(0))
    }

    /**
     * The contents of all Text nodes that are children of the given parentElement.
     * The result is trim()-ed.
     * <p>
     * The reason for this more complicated procedure instead of just returning the data of the firstChild is that
     * when the text is Chinese characters then on Android each Characater is represented in the DOM as
     * an individual Text node.
     *
     * @param parentElement parentElement
     * @return String value
     */
    pub fn get_text_children_content(parent_element: &Element) -> Option<String> {
        if parent_element.is_null() {
            return None;
        }
        let mut result = String::new();
        let child_nodes = parent_element.get_child_nodes();
        for i in 0..child_nodes.get_length() {
            let node = child_nodes.item(i);
            if node.is_null() || (node.get_node_type() != Node::TEXT_NODE) {
                continue;
            }
            result.push_str(node.get_data());
        }
        Some(result.trim().to_string())
    }
}

pub struct Document;
#[derive(PartialEq)]
pub struct Element;
#[derive(PartialEq)]
pub struct NodeList;
pub struct Node;
pub struct Text;

impl Document {
    pub fn get_elements_by_tag_name_ns(&self, _namespace: &str, _tag_name: &str) -> NodeList { todo!() }
    pub fn get_elements_by_tag_name(&self, _tag_name: &str) -> NodeList { todo!() }
}

impl Element {
    // fix: DOM stub 补充方法（get_text_children_content 使用；&Element 不可能为 null，恒 false）
    pub fn is_null(&self) -> bool { false }
    // fix: 占位实现（真实 DOM 由实现方提供）
    pub fn get_data(&self) -> &str { "" }
    pub fn get_attribute_ns(&self, _namespace: &str, _attribute: &str) -> String { todo!() }
    pub fn get_attribute(&self, _attribute: &str) -> String { todo!() }
    pub fn get_elements_by_tag_name_ns(&self, _namespace: &str, _tag_name: &str) -> NodeList { todo!() }
    pub fn get_elements_by_tag_name(&self, _tag_name: &str) -> NodeList { todo!() }
    pub fn get_child_nodes(&self) -> NodeList { todo!() }
}

impl NodeList {
    pub fn get_length(&self) -> usize { todo!() }
    pub fn item(&self, _index: usize) -> Element { todo!() }
}

impl Node {
    pub const TEXT_NODE: u16 = 3;
    pub fn get_node_type(&self) -> u16 { todo!() }
    pub fn get_data(&self) -> &str { todo!() }
}
