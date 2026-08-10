/* Copyright (c) 2002,2003, Stefan Haustein, Oberhausen, Rhld., Germany
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or
 * sell copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The  above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
 * IN THE SOFTWARE. */

// package org.kxml2.kdom;

// import java.util.*;
// import java.io.*;
// import org.xmlpull.v1.*;
/** A common base class for Document and Element, also used for
    storing XML fragments. */

//pub struct Node { //implements XmlIO{

pub struct Node {
    pub children: Option<Vec<NodeChild>>,
    pub types: Option<Vec<char>>,
}

/** A child of a Node. In Java, children is a Vector of Object holding
 *  Element or String instances. */
pub enum NodeChild {
    Element(Box<Element>),
    String(String),
}

/** Java type "Node", used for the parent pointer of an Element. */
pub enum ParentNode {
    Element(Box<Element>),
    Document(Box<Document>),
}

/** The XmlPullParser interface (org.xmlpull.v1.XmlPullParser), as far as
 *  needed by the kdom classes. */
pub trait XmlPullParser {
    const START_DOCUMENT: i32 = 0;
    const END_DOCUMENT: i32 = 1;
    const START_TAG: i32 = 2;
    const END_TAG: i32 = 3;
    const TEXT: i32 = 4;
    const CDSECT: i32 = 5;
    const ENTITY_REF: i32 = 6;
    const IGNORABLE_WHITESPACE: i32 = 7;
    const PROCESSING_INSTRUCTION: i32 = 8;
    const COMMENT: i32 = 9;
    const DOCDECL: i32 = 10;

    fn get_event_type(&self) -> i32;
    fn get_namespace(&self) -> Option<String>;
    fn get_name(&self) -> Option<String>;
    fn get_text(&self) -> Option<String>;
    fn get_attribute_count(&self) -> i32;
    fn get_attribute_namespace(&self, index: i32) -> Option<String>;
    fn get_attribute_name(&self, index: i32) -> Option<String>;
    fn get_attribute_value(&self, index: i32) -> Option<String>;
    fn get_namespace_count(&self, depth: i32) -> i32;
    fn get_namespace_prefix(&self, pos: i32) -> Option<String>;
    fn get_namespace_uri(&self, pos: i32) -> Option<String>;
    fn get_depth(&self) -> i32;
    fn get_input_encoding(&self) -> Option<String>;
    fn get_property(&self, property: &str) -> Option<Box<dyn PropertyValue>>;
    fn is_empty_element_tag(&self) -> bool;
    fn next_token(&mut self);
    fn require(&mut self, type_: i32, namespace: Option<String>, name: Option<String>);
}

/** stand-in for java.lang.Object property values (org.xmlpull.v1) */
pub trait PropertyValue {
    fn as_bool(&self) -> Option<bool>;
}

/** The XmlSerializer interface (org.xmlpull.v1.XmlSerializer), as far as
 *  needed by the kdom classes. */
pub trait XmlSerializer {
    fn start_document(&mut self, encoding: Option<String>, standalone: Option<bool>);
    fn end_document(&mut self);
    fn start_tag(&mut self, namespace: Option<String>, name: Option<String>);
    fn end_tag(&mut self, namespace: Option<String>, name: Option<String>);
    fn attribute(&mut self, namespace: Option<String>, name: Option<String>, value: Option<String>);
    fn text(&mut self, text: Option<String>);
    fn ignorable_whitespace(&mut self, s: Option<String>);
    fn cdsect(&mut self, s: Option<String>);
    fn comment(&mut self, s: Option<String>);
    fn entity_ref(&mut self, s: Option<String>);
    fn processing_instruction(&mut self, s: Option<String>);
    fn docdecl(&mut self, s: Option<String>);
    fn set_prefix(&mut self, prefix: Option<String>, namespace: Option<String>);
    fn flush(&mut self);
}

/** XmlPullParserException / RuntimeException stand-in (org.xmlpull.v1 / java.lang) */
pub enum XmlPullError {
    XmlPullParserException(String),
    RuntimeException(String),
}

/** IOException stand-in (java.io) */
pub enum XmlWriteError {
    IOException(String),
}

impl Node {

    // (Java: public static final int ...)

    pub const DOCUMENT: i32 = 0;
    pub const ELEMENT: i32 = 2;
    pub const TEXT: i32 = 4;
    pub const CDSECT: i32 = 5;
    pub const ENTITY_REF: i32 = 6;
    pub const IGNORABLE_WHITESPACE: i32 = 7;
    pub const PROCESSING_INSTRUCTION: i32 = 8;
    pub const COMMENT: i32 = 9;
    pub const DOCDECL: i32 = 10;

    pub fn new() -> Node {
        Node {
            children: None,
            types: None,
        }
    }

    /** inserts the given child object of the given type at the
    given index. */

    pub fn add_child(&mut self, this: &ParentNode, index: usize, type_: i32, child: Option<NodeChild>) {

        if child.is_none()
        // Java: throw new NullPointerException();
        { panic!("NullPointerException"); }

        if self.children.is_none() {
            self.children = Some(Vec::new());
            self.types = Some(Vec::new());
        }

        if type_ == Node::ELEMENT {
            if !matches!(child, Some(NodeChild::Element(_)))
            // Java: throw new RuntimeException("Element obj expected)");
            { panic!("Element obj expected)"); }

            // Java: ((Element) child).setParent(this);
            let mut child = child.unwrap();
            if let NodeChild::Element(e) = &mut child {
                e.set_parent(Some(parent_ref_of(this)));
            }
            let child = child;
            self.children.as_mut().unwrap().insert(index, child);
            self.types.as_mut().unwrap().insert(index, type_ as u8 as char);
        }
        else {
            if !matches!(child, Some(NodeChild::String(_)))
            // Java: throw new RuntimeException("String expected");
            { panic!("String expected"); }

            let child = child.unwrap();
            self.children.as_mut().unwrap().insert(index, child);
            self.types.as_mut().unwrap().insert(index, type_ as u8 as char);
        }
    }

    /** convenience method for addChild (getChildCount (), child) */

    pub fn add_child_simple(&mut self, this: &ParentNode, type_: i32, child: Option<NodeChild>) {
        self.add_child(this, self.get_child_count(), type_, child);
    }

    /** Builds a default element with the given properties. Elements
    should always be created using this method instead of the
    constructor in order to enable construction of specialized
    subclasses by deriving custom Document classes. Please note:
    For no namespace, please use Xml.NO_NAMESPACE, null is not a
    legal value. Currently, null is converted to Xml.NO_NAMESPACE,
    but future versions may throw an exception. */

    pub fn create_element(namespace: Option<String>, name: String) -> Element {

        let mut e = Element::new();
        e.namespace = match namespace { Some(ns) => Some(ns), None => Some(String::new()) };
        e.name = Some(name);
        return e;
    }

    /** Returns the child object at the given index.  For child
        elements, an Element object is returned. For all other child
        types, a String is returned. */

    pub fn get_child(&self, index: usize) -> &NodeChild {
        return self.children.as_ref().unwrap().get(index).unwrap();
    }

    /** Returns the number of child objects */

    pub fn get_child_count(&self) -> usize {
        return match &self.children { None => 0, Some(c) => c.len() };
    }

    /** returns the element at the given index. If the node at the
    given index is a text node, null is returned */

    pub fn get_element(&self, index: usize) -> Option<&Element> {
        let child = self.get_child(index);
        return match child {
            NodeChild::Element(e) => Some(e),
            _ => None,
        };
    }

    /** Returns the element with the given namespace and name. If the
        element is not found, or more than one matching elements are
        found, an exception is thrown. */

    pub fn get_element_named(&self, namespace: Option<String>, name: String) -> Result<&Element, String> {

        let i = self.index_of(namespace.clone(), name.clone(), 0);
        let j = self.index_of(namespace.clone(), name.clone(), i + 1);

        if i == -1 || j != -1 {
            // Java: throw new RuntimeException(
            //     "Element {"
            //         + namespace
            //         + "}"
            //         + name
            //         + (i == -1 ? " not found in " : " more than once in ")
            //         + this);
            return Err(format!(
                "Element {{{}}}{}{}{}",
                namespace.as_deref().unwrap_or("null"),
                name,
                if i == -1 { " not found in " } else { " more than once in " },
                this));
        }

        return Ok(self.get_element(i as usize).unwrap());
    }

    /* returns "#document-fragment". For elements, the element name is returned

    pub fn get_name(&self) -> String {
        return "#document-fragment".to_string();
    }

    /** Returns the namespace of the current element. For Node
        and Document, Xml.NO_NAMESPACE is returned.

    pub fn get_namespace(&self) -> String {
        return "".to_string();
    }

    pub fn get_namespace_count(&self) -> i32 {
        return 0;
    }

    /** returns the text content if the element has text-only
    content. Throws an exception for mixed content

    pub fn get_text(&self) -> String {

        let mut buf = String::new();
        let len = self.get_child_count();

        for i in 0..len {
            if self.is_text(i) {
                buf.push_str(&self.get_text(i).unwrap());
            }
            else if self.get_type(i) == Node::ELEMENT {
                panic!("not text-only content!");
            }
        }

        return buf;
    }
    */

    /** Returns the text node with the given index or null if the node
        with the given index is not a text node. */

    pub fn get_text(&self, index: usize) -> Option<String> {
        if self.is_text(index) {
            return match self.get_child(index) {
                NodeChild::String(s) => Some(s.clone()),
                _ => None,
            };
        }
        return None;
    }

    /** Returns the type of the child at the given index. Possible
    types are ELEMENT, TEXT, COMMENT, and PROCESSING_INSTRUCTION */

    pub fn get_type(&self, index: usize) -> i32 {
        return self.types.as_ref().unwrap()[index] as i32;
    }

    /** Convenience method for indexOf (getNamespace (), name,
        startIndex).

    pub fn index_of_name(&self, name: String, start_index: i32) -> i32 {
        return self.index_of(None, name, start_index);
    }
    */

    /** Performs search for an element with the given namespace and
    name, starting at the given start index. A null namespace
    matches any namespace, please use Xml.NO_NAMESPACE for no
    namespace).  returns -1 if no matching element was found. */

    pub fn index_of(&self, namespace: Option<String>, name: String, start_index: i32) -> i32 {

        let len = self.get_child_count() as i32;

        let mut i = start_index;
        while i < len {

            let child = self.get_element(i as usize);

            // Java: child != null
            //     && name.equals(child.getName())
            //     && (namespace == null || namespace.equals(child.getNamespace()))
            if child.is_some()
                && name == child.unwrap().get_name().unwrap()
                && (namespace.is_none() || namespace.as_ref().unwrap() == child.unwrap().get_namespace().as_ref().unwrap()) {
                return i;
            }
            i += 1;
        }
        return -1;
    }

    pub fn is_text(&self, i: usize) -> bool {
        let t = self.get_type(i);
        return t == Node::TEXT || t == Node::IGNORABLE_WHITESPACE || t == Node::CDSECT;
    }

    /** Recursively builds the child elements from the given parser
    until an end tag or end document is found.
        The end tag is not consumed. */

    pub fn parse(node: &mut Node, this: &ParentNode, parser: &mut dyn XmlPullParser) -> Result<(), XmlPullError> {

        let mut leave = false;

        loop {
            let type_ = parser.get_event_type();

   //         System.out.println(parser.getPositionDescription());

            match type_ {

                XmlPullParser::START_TAG => {
                    // Java: Element child =
                    //     createElement(
                    //         parser.getNamespace(),
                    //         parser.getName());
                    let child = Element::create_element_dispatch(this, parser.get_namespace(), parser.get_name());
                    //    child.setAttributes (event.getAttributes ());
                    node.add_child_simple(this, Node::ELEMENT, Some(NodeChild::Element(Box::new(child))));

                    // order is important here since
                    // setparent may perform some init code!

                    // Java: child.parse(parser);
                    Element::parse_last(node, parser)?;
                }

                XmlPullParser::END_DOCUMENT | XmlPullParser::END_TAG => {
                    leave = true;
                }

                _ => {
                    if parser.get_text().is_some() {
                        node.add_child_simple(
                            this,
                            if type_ == XmlPullParser::ENTITY_REF { Node::TEXT } else { type_ },
                            Some(NodeChild::String(parser.get_text().unwrap())));
                    }
                    else if type_ == XmlPullParser::ENTITY_REF
                        && parser.get_name().is_some() {
                        node.add_child_simple(this, Node::ENTITY_REF, Some(NodeChild::String(parser.get_name().unwrap())));
                    }
                    parser.next_token();
                }
            }
        }
        while !leave;

        Ok(())
    }

    /** Removes the child object at the given index */

    pub fn remove_child(&mut self, idx: usize) {
        self.children.as_mut().unwrap().remove(idx);

        /***  Modification by HHS - start ***/
        //      types.deleteCharAt (index);
        /***/
        let n = self.types.as_ref().unwrap().len() - 1;

        let mut i = idx;
        while i < n {
            self.types.as_mut().unwrap()[i] = self.types.as_ref().unwrap()[i + 1];
            i += 1;
        }

        self.types.as_mut().unwrap().truncate(n);

        /***  Modification by HHS - end   ***/
    }

    /* returns a valid XML representation of this Element including
        attributes and children.
    pub fn to_string(&self) -> String {
        try {
            ByteArrayOutputStream bos = new ByteArrayOutputStream();
            XmlWriter xw = new XmlWriter(new OutputStreamWriter(bos));
            write(xw);
            xw.close();
            return new String(bos.toByteArray());
        }
        catch (IOException e) {
            throw new RuntimeException(e.toString());
        }
    }
    */

    /** Writes this node to the given XmlWriter. For node and document,
        this method is identical to writeChildren, except that the
        stream is flushed automatically. */

    pub fn write(&self, writer: &mut dyn XmlSerializer) -> Result<(), XmlWriteError> {
        self.write_children(writer)?;
        writer.flush();
        Ok(())
    }

    /** Writes the children of this node to the given XmlWriter. */

    pub fn write_children(&self, writer: &mut dyn XmlSerializer) -> Result<(), XmlWriteError> {
        if self.children.is_none() {
            return Ok(());
        }

        let len = self.children.as_ref().unwrap().len();

        for i in 0..len {
            let type_ = self.get_type(i);
            let child = self.get_child(i);
            match type_ {
                Node::ELEMENT => {
                    if let NodeChild::Element(e) = child {
                        e.write(writer)?;
                    }
                }

                Node::TEXT => {
                    if let NodeChild::String(s) = child {
                        writer.text(Some(s.clone()));
                    }
                }

                Node::IGNORABLE_WHITESPACE => {
                    if let NodeChild::String(s) = child {
                        writer.ignorable_whitespace(Some(s.clone()));
                    }
                }

                Node::CDSECT => {
                    if let NodeChild::String(s) = child {
                        writer.cdsect(Some(s.clone()));
                    }
                }

                Node::COMMENT => {
                    if let NodeChild::String(s) = child {
                        writer.comment(Some(s.clone()));
                    }
                }

                Node::ENTITY_REF => {
                    if let NodeChild::String(s) = child {
                        writer.entity_ref(Some(s.clone()));
                    }
                }

                Node::PROCESSING_INSTRUCTION => {
                    if let NodeChild::String(s) = child {
                        writer.processing_instruction(Some(s.clone()));
                    }
                }

                Node::DOCDECL => {
                    if let NodeChild::String(s) = child {
                        writer.docdecl(Some(s.clone()));
                    }
                }

                _ => {
                    // Java: throw new RuntimeException("Illegal type: " + type);
                    panic!("Illegal type: {}", type_);
                }
            }
        }

        Ok(())
    }
}

/** returns a copy of the parent reference to be stored in an element
 *  (Java: the parent is a reference to the enclosing Node) */
pub fn parent_ref_of(this: &ParentNode) -> ParentNode {
    return match this {
        ParentNode::Element(e) => ParentNode::Element(Box::new((**e).clone())),
        ParentNode::Document(d) => ParentNode::Document(Box::new((**d).clone())),
    };
}
