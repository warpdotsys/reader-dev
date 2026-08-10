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

// import org.xmlpull.v1.*;
/** The document consists of some legacy events and a single root
    element. This class basically adds some consistency checks to
    Node. */

// public class Document extends Node {

pub struct Document {
    pub base: Node,
    pub root_index: i32,
    pub encoding: Option<String>,
    pub standalone: Option<bool>,
}

impl Document {

    pub fn new() -> Document {
        Document {
            base: Node::new(),
            root_index: -1,
            encoding: None,
            standalone: None,
        }
    }

    /** returns "#document" */

    pub fn get_encoding(&self) -> Option<String> {
        return self.encoding.clone();
    }

    pub fn set_encoding(&mut self, enc: Option<String>) {
        self.encoding = enc;
    }

    pub fn set_standalone(&mut self, standalone: Option<bool>) {
        self.standalone = standalone;
    }

    pub fn get_standalone(&self) -> Option<bool> {
        return self.standalone;
    }


    pub fn get_name(&self) -> &'static str {
        return "#document";
    }

    /** Adds a child at the given index position. Throws
    an exception when a second root element is added */

    pub fn add_child(&mut self, index: usize, type_: i32, child: Option<NodeChild>) {
        if type_ == Node::ELEMENT {
         //   if (rootIndex != -1)
           //     throw new RuntimeException("Only one document root element allowed");

            self.root_index = index as i32;
        }
        else if self.root_index >= index as i32 {
            self.root_index += 1;
        }

        self.base.add_child(&ParentNode::Document(Box::new((*self).clone())), index, type_, child);
    }

    /** reads the document and checks if the last event
    is END_DOCUMENT. If not, an exception is thrown.
    The end event is consumed. For parsing partial
        XML structures, consider using Node.parse (). */

    pub fn parse(&mut self, parser: &mut dyn XmlPullParser) -> Result<(), XmlPullError> {

        parser.require(XmlPullParser::START_DOCUMENT, None, None);
        parser.next_token();

        self.encoding = parser.get_input_encoding();
        self.standalone = match parser.get_property("http://xmlpull.org/v1/doc/properties.html#xmldecl-standalone") {
            Some(prop) => prop.as_bool(),
            None => None,
        };

        Node::parse(&mut self.base, &ParentNode::Document(Box::new((*self).clone())), parser)?;

        if parser.get_event_type() != XmlPullParser::END_DOCUMENT
        // Java: throw new RuntimeException("Document end expected!");
        { return Err(XmlPullError::RuntimeException("Document end expected!".to_string())); }

        Ok(())
    }

    pub fn remove_child(&mut self, index: usize) {
        if index as i32 == self.root_index {
            self.root_index = -1;
        }
        else if (index as i32) < self.root_index {
            self.root_index -= 1;
        }

        self.base.remove_child(index);
    }

    /** returns the root element of this document. */

    pub fn get_root_element(&self) -> Result<&Element, String> {
        if self.root_index == -1
        // Java: throw new RuntimeException("Document has no root element!");
        { return Err("Document has no root element!".to_string()); }

        // Java: return (Element) getChild(rootIndex);
        return match self.base.get_child(self.root_index as usize) {
            NodeChild::Element(e) => Ok(e),
            _ => Err("Document has no root element!".to_string()),
        };
    }


    /** Writes this node to the given XmlWriter. For node and document,
        this method is identical to writeChildren, except that the
        stream is flushed automatically. */

    pub fn write(&self, writer: &mut dyn XmlSerializer) -> Result<(), XmlWriteError> {

        writer.start_document(self.encoding.clone(), self.standalone);
        self.base.write_children(writer)?;
        writer.end_document();

        Ok(())
    }


}
