use crate::prelude::*;
use crate::org_kxml2_kdom_node::{Node, XmlPullError, XmlSerializer, END_TAG};
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

// import java.io.*;
// import java.util.*;

// import org.xmlpull.v1.*;

/**
 * In order to create an element, please use the createElement method
 * instead of invoking the constructor directly. The right place to
 * add user defined initialization code is the init method. */

// pub struct Element extends Node {

#[derive(Clone)]
pub struct Element {
    pub base: Node,
    pub namespace: Option<String>,
    pub name: Option<String>,
    // Java: protected Vector attributes;  (Vector of String[3])
    pub attributes: Option<Vec<[Option<String>; 3]>>,
    pub parent: Option<ParentNode>,
    // Java: protected Vector prefixes;  (Vector of String[2])
    pub prefixes: Option<Vec<[Option<String>; 2]>>,
}

impl Element {

    pub fn new() -> Element {
        Element {
            base: Node::new(),
            namespace: None,
            name: None,
            attributes: None,
            parent: None,
            prefixes: None,
        }
    }

    /**
     * called when all properties are set, but before children
     * are parsed. Please do not use setParent for initialization
     * code any longer. */

    pub fn init(&mut self) {
    }




    /**
     * removes all children and attributes */

    pub fn clear(&mut self) {
        self.attributes = None;
        self.base.children = None;
    }

    /**
     * Forwards creation request to parent if any, otherwise
     * calls super.createElement. */

    pub fn create_element(
        &self,
        namespace: Option<String>,
        name: String) -> Element {

        // Java: return (this.parent == None)
        //     ? super.createElement(namespace, name)
        //     : this.parent.createElement(namespace, name);
        return match &self.parent {
            None => Node::create_element(namespace, name),
            Some(p) => create_element_from_parent(Some(p), namespace, name),
        };
    }

    /**
     * Returns the number of attributes of this element. */

    pub fn get_attribute_count(&self) -> usize {
        return match &self.attributes { None => 0, Some(a) => a.len() };
    }

    pub fn get_attribute_namespace(&self, index: usize) -> Option<String> {
        return self.attributes.as_ref().unwrap()[index][0].clone();
    }

//    pub fn get_attribute_prefix(&self, index: usize) -> Option<String> {
//        return self.attributes.as_ref().unwrap()[index][1].clone();
//    }

    pub fn get_attribute_name(&self, index: usize) -> Option<String> {
        return self.attributes.as_ref().unwrap()[index][1].clone();
    }


    pub fn get_attribute_value(&self, index: usize) -> Option<String> {
        return self.attributes.as_ref().unwrap()[index][2].clone();
    }


    pub fn get_attribute_value_named(&self, namespace: Option<String>, name: String) -> Option<String> {
        let mut i = 0;
        while i < self.get_attribute_count() {
            if name == self.get_attribute_name(i).unwrap()
                && (namespace.is_none() || namespace.as_ref().unwrap() == self.get_attribute_namespace(i).as_ref().unwrap()) {
                return self.get_attribute_value(i);
            }
            i += 1;
        }
        return None;
    }

    /**
     * Returns the root node, determined by ascending to the
     * all parents un of the root element. */

    pub fn get_root(&self) -> ParentNode {

        let mut current = self;

        // Java: while (current.parent != None) {
        //     if (!(current.parent instanceof Element)) return current.parent;
        //     current = (Element) current.parent;
        // }
        loop {
            match &current.parent {
                None => break,
                Some(ParentNode::Element(e)) => {
                    current = e;
                }
                Some(ParentNode::Document(_)) => {
                    return current.parent.clone().unwrap();
                }
            }
        }

        return ParentNode::Element(Box::new((*current).clone()));
    }

    /**
     * returns the (local) name of the element */

    pub fn get_name(&self) -> Option<String> {
        return self.name.clone();
    }

    /**
     * returns the namespace of the element */

    pub fn get_namespace(&self) -> Option<String> {
        return self.namespace.clone();
    }


    /**
     * returns the namespace for the given prefix */

    pub fn get_namespace_uri(&self, prefix: Option<String>) -> Option<String> {
        let cnt = self.get_namespace_count();
        let mut i = 0;
        while i < cnt {
            // Java: prefix == getNamespacePrefix (i) ||
            //     (prefix != None && prefix.equals (getNamespacePrefix (i)))
            if prefix == self.get_namespace_prefix(i)
                || (prefix.is_some() && prefix.as_ref() == self.get_namespace_prefix(i).as_ref()) {
                return self.get_namespace_uri_index(i);
            }
            i += 1;
        }
        return match &self.parent {
            Some(ParentNode::Element(e)) => e.get_namespace_uri(prefix),
            _ => None,
        };
    }


    /**
     * returns the number of declared namespaces, NOT including
     * parent elements */

    pub fn get_namespace_count(&self) -> usize {
        return match &self.prefixes { None => 0, Some(p) => p.len() };
    }


    pub fn get_namespace_prefix(&self, i: usize) -> Option<String> {
        return self.prefixes.as_ref().unwrap()[i][0].clone();
    }

    pub fn get_namespace_uri_index(&self, i: usize) -> Option<String> {
        return self.prefixes.as_ref().unwrap()[i][1].clone();
    }


    /**
     * Returns the parent node of this element */

    pub fn get_parent(&self) -> Option<ParentNode> {
        return self.parent.clone();
    }

    /*
     * Returns the parent element if available, None otherwise

    pub fn get_parent_element(&self) -> Option<Element> {
        return match &self.parent {
            Some(ParentNode::Element(e)) => Some((**e).clone()),
            _ => None,
        };
    }
    */

    /**
     * Builds the child elements from the given Parser. By overwriting
     * parse, an element can take complete control over parsing its
     * subtree. */

    pub fn parse(&mut self, parser: &mut dyn XmlPullParser) -> Result<(), XmlPullError> {

        // Java: for (int i = parser.getNamespaceCount (parser.getDepth () - 1);
        //    i < parser.getNamespaceCount (parser.getDepth ()); i++) {
        let mut i = parser.get_namespace_count(parser.get_depth() - 1);
        while i < parser.get_namespace_count(parser.get_depth()) {
            self.set_prefix(parser.get_namespace_prefix(i), parser.get_namespace_uri(i));
            i += 1;
        }


        // Java: for (int i = 0; i < parser.getAttributeCount (); i++)
        let mut i = 0;
        while i < parser.get_attribute_count() {
            self.set_attribute(parser.get_attribute_namespace(i),
                //          parser.getAttributePrefix (i),
                parser.get_attribute_name(i),
                parser.get_attribute_value(i));
            i += 1;
        }


        //        if (prefixMap == None) throw new RuntimeException ("!!");

        self.init();


        if parser.is_empty_element_tag() {
            parser.next_token();
        }
        else {
            let elem_clone = (*self).clone();
            Node::parse(&mut self.base, &ParentNode::Element(Box::new(elem_clone)), parser)?;

            if self.base.get_child_count() == 0 {
                let elem_clone2 = (*self).clone();
                self.base.add_child_simple(&ParentNode::Element(Box::new(elem_clone2)), Node::IGNORABLE_WHITESPACE, Some(NodeChild::String(String::new())));
            }
        }

        parser.require(
            END_TAG,
            self.get_namespace(),
            self.get_name());

        parser.next_token();

        Ok(())
    }

    /** parses the element that was most recently appended to the given
     *  node's children (Java: child.parse(parser) inside Node.parse) */
    pub fn parse_last(node: &mut Node, parser: &mut dyn XmlPullParser) -> Result<(), XmlPullError> {
        let last = node.children.as_mut().unwrap().last_mut().unwrap();
        return match last {
            NodeChild::Element(e) => e.parse(parser),
            _ => unreachable!(),
        };
    }

    /** Java: (this.parent == None) ? super.createElement(namespace, name)
     *        : this.parent.createElement(namespace, name);
     *  called from Node.parse for the createElement virtual dispatch */
    pub fn create_element_dispatch(this: &ParentNode, namespace: Option<String>, name: Option<String>) -> Element {
        return match this {
            ParentNode::Element(e) => e.create_element(namespace, name.unwrap_or_default()),
            ParentNode::Document(_) => Node::create_element(namespace, name.unwrap_or_default()),
        };
    }

    /**
     * Sets the given attribute; a value of None removes the attribute */

    pub fn set_attribute(&mut self, namespace: Option<String>, name: Option<String>, value: Option<String>) {
        if self.attributes.is_none() {
            self.attributes = Some(Vec::new());
        }

        // Java: if (namespace == None) namespace = "";
        let namespace = match namespace { Some(ns) => ns, None => String::new() };

        let mut i = (self.attributes.as_ref().unwrap().len() as i32) - 1;
        while i >= 0 {
            // Java: String[] attribut = (String[]) attributes.elementAt(i);
            let ns = self.attributes.as_ref().unwrap()[i as usize][0].clone().unwrap();
            let n = self.attributes.as_ref().unwrap()[i as usize][1].clone().unwrap();
            if ns == namespace && n == name.clone().unwrap() {

                if value.is_none() {
                    self.attributes.as_mut().unwrap().remove(i as usize);
                }
                else {
                    self.attributes.as_mut().unwrap()[i as usize][2] = value.clone();
                }
                return;
            }
            i -= 1;
        }

        // Java: attributes.addElement (new String [] {namespace, name, value});
        self.attributes.as_mut().unwrap().push([Some(namespace), name, value]);
    }


    /**
     * Sets the given prefix; a namespace value of None removess the
     * prefix */

    pub fn set_prefix(&mut self, prefix: Option<String>, namespace: Option<String>) {
        if self.prefixes.is_none() {
            self.prefixes = Some(Vec::new());
        }
        self.prefixes.as_mut().unwrap().push([prefix, namespace]);
    }


    /**
     * sets the name of the element */

    pub fn set_name(&mut self, name: Option<String>) {
        self.name = name;
    }

    /**
     * sets the namespace of the element. Please note: For no
     * namespace, please use Xml.NO_NAMESPACE, None is not a legal
     * value. Currently, None is converted to Xml.NO_NAMESPACE, but
     * future versions may throw an exception. */

    pub fn set_namespace(&mut self, namespace: String) {
        // Java: if (namespace == None)
        //     throw new NullPointerException ("Use \"\" for empty namespace");
        self.namespace = Some(namespace);
    }

    /**
     * Sets the Parent of this element. Automatically called from the
     * add method.  Please use with care, you can simply
     * create inconsitencies in the document tree structure using
     * this method!  */

    // Java: protected void setParent(Node parent)
    pub fn set_parent(&mut self, parent: Option<ParentNode>) {
        self.parent = parent;
    }


    /**
     * Writes this element and all children to the given XmlWriter. */

    pub fn write(&self, writer: &mut dyn XmlSerializer) -> Result<(), XmlWriteError> {

        if self.prefixes.is_some() {
            let mut i = 0;
            while i < self.prefixes.as_ref().unwrap().len() {
                writer.set_prefix(self.get_namespace_prefix(i), self.get_namespace_uri_index(i));
                i += 1;
            }
        }

        writer.start_tag(
            self.get_namespace(),
            self.get_name());

        let len = self.get_attribute_count();

        let mut i = 0;
        while i < len {
            writer.attribute(
                self.get_attribute_namespace(i),
                self.get_attribute_name(i),
                self.get_attribute_value(i));
            i += 1;
        }

        self.base.write_children(writer)?;

        writer.end_tag(self.get_namespace(), self.get_name());

        Ok(())
    }
}

/** returns a copy of the parent reference to be stored in an element
 *  (Java: the parent is a reference to the enclosing Node) */
pub fn create_element_from_parent(parent: Option<&ParentNode>, namespace: Option<String>, name: String) -> Element {
    return match parent {
        None => Node::create_element(namespace, name),
        Some(ParentNode::Element(e)) => e.create_element(namespace, name),
        Some(ParentNode::Document(_)) => Node::create_element(namespace, name),
    };
}
