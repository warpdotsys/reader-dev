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

pub struct Document {
    pub root: Option<Element>,
}

#[derive(Clone, PartialEq)]
pub struct Element {
    pub tag_name: String,
    pub attributes: std::collections::HashMap<String, String>,
    pub children: Vec<Node>,
    pub is_text: bool,
    pub text_data: String,
    pub null: bool,
}

#[derive(Clone, PartialEq)]
pub struct Node {
    pub element: Option<Element>,
    pub text: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct NodeList {
    pub items: Vec<Element>,
}

pub struct Text {
    pub data: String,
}

impl Document {
    pub fn new() -> Document {
        Document { root: None }
    }
    /// quick-xml 解析 XML 构建 DOM 树
    pub fn parse(xml: &str) -> Document {
        let mut reader = quick_xml::Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        let children = parse_dom_children(&mut reader, &mut buf);
        Document {
            root: children.into_iter().find(|e| !e.is_text),
        }
    }
    pub fn get_elements_by_tag_name_ns(&self, _namespace: &str, tag_name: &str) -> NodeList {
        collect_by_tag(self.root.as_ref(), tag_name)
    }
    pub fn get_elements_by_tag_name(&self, tag_name: &str) -> NodeList {
        collect_by_tag(self.root.as_ref(), tag_name)
    }
}

impl Element {
    pub fn null() -> Element {
        Element {
            tag_name: String::new(),
            attributes: std::collections::HashMap::new(),
            children: Vec::new(),
            is_text: false,
            text_data: String::new(),
            null: true,
        }
    }
    pub fn is_null(&self) -> bool {
        self.null
    }
    pub fn get_data(&self) -> &str {
        &self.text_data
    }
    pub fn get_node_type(&self) -> u16 {
        if self.is_text {
            3
        } else {
            1
        }
    }
    pub fn get_local_name(&self) -> String {
        // 本地名 = 冒号分隔的最后一段（OPF 命名空间前缀省略）
        self.tag_name.rsplit(':').next().unwrap_or(&self.tag_name).to_string()
    }
    pub fn get_attribute_ns(&self, _namespace: &str, attribute: &str) -> String {
        self.attributes
            .get(attribute)
            .cloned()
            .unwrap_or_default()
    }
    pub fn get_attribute(&self, attribute: &str) -> String {
        self.attributes
            .get(attribute)
            .cloned()
            .unwrap_or_default()
    }
    pub fn get_elements_by_tag_name_ns(&self, _namespace: &str, tag_name: &str) -> NodeList {
        collect_by_tag(Some(self), tag_name)
    }
    pub fn get_elements_by_tag_name(&self, tag_name: &str) -> NodeList {
        collect_by_tag(Some(self), tag_name)
    }
    pub fn get_child_nodes(&self) -> NodeList {
        let items = self
            .children
            .iter()
            .filter_map(|n| n.as_element())
            .collect();
        NodeList { items }
    }
}

impl Node {
    pub const TEXT_NODE: u16 = 3;
    pub const ELEMENT_NODE: u16 = 1;
    pub fn get_node_type(&self) -> u16 {
        match &self.element {
            Some(e) => {
                if e.is_text {
                    Node::TEXT_NODE
                } else {
                    Node::ELEMENT_NODE
                }
            }
            None => Node::TEXT_NODE,
        }
    }
    pub fn get_data(&self) -> &str {
        match (&self.element, &self.text) {
            (Some(e), _) => &e.text_data,
            (None, Some(t)) => t,
            _ => "",
        }
    }
    pub fn is_null(&self) -> bool {
        self.element.is_none() && self.text.is_none()
    }
    pub fn as_element(&self) -> Option<Element> {
        self.element.clone()
    }
}

impl NodeList {
    pub fn get_length(&self) -> usize {
        self.items.len()
    }
    pub fn item(&self, index: usize) -> Element {
        self.items.get(index).cloned().unwrap_or_else(Element::null)
    }
    pub fn first(&self) -> Option<Element> {
        self.items.first().cloned()
    }
}

fn element_from_start(e: &quick_xml::events::BytesStart) -> Element {
    Element {
        tag_name: String::from_utf8_lossy(e.name().as_ref()).to_string(),
        attributes: e
            .attributes()
            .filter_map(|a| a.ok())
            .map(|a| {
                (
                    String::from_utf8_lossy(a.key.as_ref()).to_string(),
                    String::from_utf8_lossy(&a.value).to_string(),
                )
            })
            .collect(),
        children: Vec::new(),
        is_text: false,
        text_data: String::new(),
        null: false,
    }
}

fn parse_dom_children(reader: &mut quick_xml::Reader<&[u8]>, buf: &mut Vec<u8>) -> Vec<Element> {
    let mut children: Vec<Element> = Vec::new();
    loop {
        buf.clear();
        match reader.read_event_into(buf) {
            Ok(quick_xml::events::Event::Start(e)) => {
                // fix: 自闭合标签（manifest item / spine itemref 等）——Empty 事件单独处理
                let mut element = element_from_start(&e);
                element.children = parse_dom_children(reader, buf)
                    .into_iter()
                    .map(|e| Node {
                        element: Some(e),
                        text: None,
                    })
                    .collect();
                children.push(element);
            }
            Ok(quick_xml::events::Event::Empty(e)) => {
                children.push(element_from_start(&e));
            }
            Ok(quick_xml::events::Event::Text(t)) => {
                if let Ok(text) = t.unescape() {
                    let text = text.trim().to_string();
                    if !text.is_empty() {
                        children.push(Element {
                            tag_name: String::from("#text"),
                            attributes: std::collections::HashMap::new(),
                            children: Vec::new(),
                            is_text: true,
                            text_data: text,
                            null: false,
                        });
                    }
                }
            }
            Ok(quick_xml::events::Event::End(_))
            | Ok(quick_xml::events::Event::Eof)
            | Err(_) => break,
            _ => {}
        }
    }
    children
}

fn collect_by_tag(root: Option<&Element>, tag_name: &str) -> NodeList {
    let mut items = Vec::new();
    if let Some(root) = root {
        collect_by_tag_inner(root, tag_name, &mut items);
    }
    NodeList { items }
}

fn collect_by_tag_inner(element: &Element, tag_name: &str, out: &mut Vec<Element>) {
    // fix: XML 命名空间前缀（dc:title 等）——按本地名匹配（原精确匹配导致带前缀标签永不命中）
    let want = tag_name.rsplit(':').next().unwrap_or(tag_name);
    if !element.is_text && element.get_local_name().eq_ignore_ascii_case(want) {
        out.push(element.clone());
    }
    for node in &element.children {
        if let Some(child) = node.as_element() {
            collect_by_tag_inner(&child, tag_name, out);
        }
    }
}
