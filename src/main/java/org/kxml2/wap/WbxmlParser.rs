use crate::prelude::*;
/* Copyright (c) 2002,2003,2004 Stefan Haustein, Oberhausen, Rhld., Germany
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

// Contributors: Bjorn Aadland, Chris Bartley, Nicola Fankhauser,
//               Victor Havin,  Christian Kurzke, Bogdan Onoiu,
//                Elias Ross, Jain Sanjay, David Santoro.

// package org.kxml2.wap;

// import java.io.*;
// import java.util.Vector;
// import java.util.Hashtable;

// import org.xmlpull.v1.*;

/** InputStream interface (java.io.InputStream), as far as needed by WbxmlParser. */
pub trait InputStream {
    fn read(&mut self) -> i32;
    fn read_buf(&mut self, buf: &mut [u8], off: usize, len: usize) -> i32;
}

/** java.lang.Object stand-in for the wap extension data */
pub enum WapExtensionData {
    Bytes(Vec<u8>),
    Int(i32),
    Str(String),
    None,
}

/** XmlPullParserException / IOException / RuntimeException stand-ins */
pub enum WbxmlError {
    XmlPullParserException(String),
    IOException(String),
    RuntimeException(String),
}

/** Marker type implementing the Wbxml trait so its constants are addressable (WbxmlConsts::X). */
pub struct WbxmlConsts;

impl crate::org_kxml2_wap_wbxml::Wbxml for WbxmlConsts {}

// public class WbxmlParser implements XmlPullParser {

pub struct WbxmlParser {

    /** Parser event type for Wbxml-specific events. The Wbxml event code can be
     * accessed with getWapCode() */

    // (Java: static final private String UNEXPECTED_EOF = "Unexpected EOF";)
    // (Java: static final private String ILLEGAL_TYPE = "Wrong event type";)

    pub in_stream: Option<Box<dyn InputStream>>,

    pub tag_table: i32,
    pub attr_start_table: i32,
    pub attr_value_table: i32,

    pub attr_start_table_data: Option<Vec<Option<String>>>,
    pub attr_value_table_data: Option<Vec<Option<String>>>,
    pub tag_table_data: Option<Vec<Option<String>>>,
    pub string_table: Option<Vec<u8>>,
    pub cache_string_table: Option<HashMap<i32, String>>,
    pub process_nsp: bool,

    pub depth: i32,
    pub element_stack: Vec<Option<String>>,
    pub nsp_stack: Vec<Option<String>>,
    pub nsp_counts: Vec<i32>,

    pub attribute_count: i32,
    pub attributes: Vec<Option<String>>,
    pub next_id: i32,

    // Java: private Vector tables = new Vector();  (Vector of String[])
    pub tables: Vec<Option<Vec<Option<String>>>>,

    pub version: i32,
    pub public_identifier_id: i32,

    //    StartTag current;
    //    ParseEvent next;

    pub prefix: Option<String>,
    pub namespace: Option<String>,
    pub name: Option<String>,
    pub text: Option<String>,

    pub wap_extension_data: WapExtensionData,
    pub wap_code: i32,

    pub type_: i32,

    pub degenerated: bool,
    pub is_whitespace: bool,
    pub encoding: Option<String>,
}

impl WbxmlParser {

    // (Java: static final String HEX_DIGITS = "0123456789abcdef";)
    pub const HEX_DIGITS: &'static str = "0123456789abcdef";

    pub const WAP_EXTENSION: i32 = 64;
    pub const UNEXPECTED_EOF: &'static str = "Unexpected EOF";
    pub const ILLEGAL_TYPE: &'static str = "Wrong event type";

    // (org.xmlpull.v1.XmlPullParser constants, since WbxmlParser implements XmlPullParser)
    pub const START_DOCUMENT: i32 = 0;
    pub const END_DOCUMENT: i32 = 1;
    pub const START_TAG: i32 = 2;
    pub const END_TAG: i32 = 3;
    pub const TEXT: i32 = 4;
    pub const CDSECT: i32 = 5;
    pub const ENTITY_REF: i32 = 6;
    pub const IGNORABLE_WHITESPACE: i32 = 7;
    pub const PROCESSING_INSTRUCTION: i32 = 8;
    pub const COMMENT: i32 = 9;
    pub const DOCDECL: i32 = 10;
    pub const FEATURE_PROCESS_NAMESPACES: &'static str = "http://xmlpull.org/v1/doc/features.html#process-namespaces";
    pub const NO_NAMESPACE: &'static str = "";

    pub const TYPES: [&'static str; 11] = [
        "START_DOCUMENT",
        "END_DOCUMENT",
        "START_TAG",
        "END_TAG",
        "TEXT",
        "CDSECT",
        "ENTITY_REF",
        "IGNORABLE_WHITESPACE",
        "PROCESSING_INSTRUCTION",
        "COMMENT",
        "DOCDECL",
    ];

    pub fn new() -> WbxmlParser {
        WbxmlParser {
            in_stream: None,
            tag_table: 0,
            attr_start_table: 1,
            attr_value_table: 2,
            attr_start_table_data: None,
            attr_value_table_data: None,
            tag_table_data: None,
            string_table: None,
            cache_string_table: None,
            process_nsp: false,
            depth: 0,
            element_stack: vec![None; 16],
            nsp_stack: vec![None; 8],
            nsp_counts: vec![0; 4],
            attribute_count: 0,
            attributes: vec![None; 16],
            next_id: -2,
            tables: Vec::new(),
            version: 0,
            public_identifier_id: 0,
            prefix: None,
            namespace: None,
            name: None,
            text: None,
            wap_extension_data: WapExtensionData::None,
            wap_code: 0,
            type_: 0,
            degenerated: false,
            is_whitespace: false,
            encoding: None,
        }
    }

    pub fn get_feature(&self, feature: String) -> bool {
        if XmlPullParserFeatures::FEATURE_PROCESS_NAMESPACES
            .eq(&feature)
        { return self.process_nsp; }
        else
        { return false; }
    }

    pub fn get_input_encoding(&self) -> Option<String> {
        return self.encoding.clone();
    }

    pub fn define_entity_replacement_text(
        &mut self,
        entity: String,
        value: String)
        -> Result<(), WbxmlError> {

        // just ignore, has no effect
        Ok(())
    }

    pub fn get_property(&self, property: String) -> Option<i32> {
        return None;
    }

    pub fn get_namespace_count(&self, depth: i32) -> i32 {
        if depth > self.depth
        // Java: throw new IndexOutOfBoundsException();
        { panic!("IndexOutOfBoundsException"); }
        return self.nsp_counts[depth as usize];
    }

    pub fn get_namespace_prefix(&self, pos: i32) -> Option<String> {
        return self.nsp_stack[(pos << 1) as usize].clone();
    }

    pub fn get_namespace_uri(&self, pos: i32) -> Option<String> {
        return self.nsp_stack[((pos << 1) + 1) as usize].clone();
    }

    pub fn get_namespace(&self, prefix: Option<String>) -> Option<String> {

        if "xml".eq(prefix.as_deref().unwrap_or(""))
        { return Some("http://www.w3.org/XML/1998/namespace".to_string()); }
        if "xmlns".eq(prefix.as_deref().unwrap_or(""))
        { return Some("http://www.w3.org/2000/xmlns/".to_string()); }

        let mut i = (self.get_namespace_count(self.depth) << 1) - 2;
        while i >= 0 {
            if prefix.is_none() {
                if self.nsp_stack[i as usize].is_none()
                { return self.nsp_stack[(i + 1) as usize].clone(); }
            }
            else if prefix.as_ref().unwrap() == self.nsp_stack[i as usize].as_ref().unwrap()
            { return self.nsp_stack[(i + 1) as usize].clone(); }
            i -= 2;
        }
        return None;
    }

    pub fn get_depth(&self) -> i32 {
        return self.depth;
    }

    pub fn get_position_description(&self) -> String {

        let mut buf =
            String::new();
        buf.push_str(if self.type_ < WbxmlParser::TYPES.len() as i32 { WbxmlParser::TYPES[self.type_ as usize] } else { "unknown" });
        buf.push(' ');

        if self.type_ == WbxmlParser::START_TAG || self.type_ == WbxmlParser::END_TAG {
            if self.degenerated {
                buf.push_str("(empty) ");
            }
            buf.push('<');
            if self.type_ == WbxmlParser::END_TAG {
                buf.push('/');
            }

            if self.prefix.is_some() {
                buf.push_str(&format!(
                    "{{{}}}{}:",
                    self.namespace.as_deref().unwrap_or("null"),
                    self.prefix.as_deref().unwrap_or("null")));
            }
            buf.push_str(self.name.as_deref().unwrap_or("null"));

            let cnt = self.attribute_count << 2;
            let mut i = 0;
            while i < cnt {
                buf.push(' ');
                if self.attributes[(i + 1) as usize].is_some()
                {
                    buf.push_str(&format!(
                        "{{{}}}{}:",
                        self.attributes[i as usize].as_deref().unwrap_or("null"),
                        self.attributes[(i + 1) as usize].as_deref().unwrap_or("null")));
                }
                buf.push_str(&format!(
                    "{}='{}'",
                    self.attributes[(i + 2) as usize].as_deref().unwrap_or("null"),
                    self.attributes[(i + 3) as usize].as_deref().unwrap_or("null")));
                i += 4;
            }

            buf.push('>');
        }
        else if self.type_ == WbxmlParser::IGNORABLE_WHITESPACE {
        }
        else if self.type_ != WbxmlParser::TEXT {
            buf.push_str(&self.get_text().unwrap_or(String::new()));
        }
        else if self.is_whitespace {
            buf.push_str("(whitespace)");
        }
        else {
            let mut text = self.get_text().unwrap_or(String::new());
            if text.chars().count() > 16 {
                text = text.chars().take(16).collect::<String>() + "...";
            }
            buf.push_str(&text);
        }

        return buf;
    }

    pub fn get_line_number(&self) -> i32 {
        return -1;
    }

    pub fn get_column_number(&self) -> i32 {
        return -1;
    }

    pub fn is_whitespace(&self)
        -> Result<bool, WbxmlError> {
        if self.type_ != WbxmlParser::TEXT
            && self.type_ != WbxmlParser::IGNORABLE_WHITESPACE
            && self.type_ != WbxmlParser::CDSECT
        { return Err(WbxmlError::XmlPullParserException(WbxmlParser::ILLEGAL_TYPE.to_string())); }
        return Ok(self.is_whitespace);
    }

    pub fn get_text(&self) -> Option<String> {
        return self.text.clone();
    }

    pub fn get_text_characters(&self, poslen: &mut [i32; 2]) -> Option<Vec<char>> {
        if self.type_ >= WbxmlParser::TEXT {
            poslen[0] = 0;
            poslen[1] = self.text.as_ref().unwrap().chars().count() as i32;
            let buf: Vec<char> = self.text.as_ref().unwrap().chars().collect();
            return Some(buf);
        }

        poslen[0] = -1;
        poslen[1] = -1;
        return None;
    }

    pub fn get_namespace_current(&self) -> Option<String> {
        return self.namespace.clone();
    }

    pub fn get_name(&self) -> Option<String> {
        return self.name.clone();
    }

    pub fn get_prefix(&self) -> Option<String> {
        return self.prefix.clone();
    }

    pub fn is_empty_element_tag(&self)
        -> Result<bool, WbxmlError> {
        if self.type_ != WbxmlParser::START_TAG
        { return Err(WbxmlError::XmlPullParserException(WbxmlParser::ILLEGAL_TYPE.to_string())); }
        return Ok(self.degenerated);
    }

    pub fn get_attribute_count(&self) -> i32 {
        return self.attribute_count;
    }

    pub fn get_attribute_type(&self, index: i32) -> &'static str {
        return "CDATA";
    }

    pub fn is_attribute_default(&self, index: i32) -> bool {
        return false;
    }

    pub fn get_attribute_namespace(&self, index: i32) -> Option<String> {
        if index >= self.attribute_count
        // Java: throw new IndexOutOfBoundsException();
        { panic!("IndexOutOfBoundsException"); }
        return self.attributes[(index << 2) as usize].clone();
    }

    pub fn get_attribute_name(&self, index: i32) -> Option<String> {
        if index >= self.attribute_count
        // Java: throw new IndexOutOfBoundsException();
        { panic!("IndexOutOfBoundsException"); }
        return self.attributes[((index << 2) + 2) as usize].clone();
    }

    pub fn get_attribute_prefix(&self, index: i32) -> Option<String> {
        if index >= self.attribute_count
        // Java: throw new IndexOutOfBoundsException();
        { panic!("IndexOutOfBoundsException"); }
        return self.attributes[((index << 2) + 1) as usize].clone();
    }

    pub fn get_attribute_value(&self, index: i32) -> Option<String> {
        if index >= self.attribute_count
        // Java: throw new IndexOutOfBoundsException();
        { panic!("IndexOutOfBoundsException"); }
        return self.attributes[((index << 2) + 3) as usize].clone();
    }

    pub fn get_attribute_value_named(
        &self,
        namespace: Option<String>,
        name: String) -> Option<String> {

        let mut i = (self.attribute_count << 2) - 4;
        while i >= 0 {
            if self.attributes[(i + 2) as usize].as_ref().unwrap() == &name
                && (namespace.is_none()
                    || self.attributes[i as usize].as_ref().unwrap() == namespace.as_ref().unwrap())
            { return self.attributes[(i + 3) as usize].clone(); }
            i -= 4;
        }

        return None;
    }

    pub fn get_event_type(&self) -> Result<i32, WbxmlError> {
        return Ok(self.type_);
    }


    // TODO: Reuse resolveWapExtension here? Raw Wap extensions would still be accessible
    // via nextToken();  ....?

    pub fn next(&mut self) -> Result<i32, WbxmlError> {

        self.is_whitespace = true;
        let mut min_type = 9999;

        loop {

            let save = self.text.clone();

            self.next_impl()?;

            if self.type_ < min_type {
                min_type = self.type_;
            }

            if min_type > WbxmlParser::CDSECT {
                continue; // no "real" event so far
            }

            if min_type >= WbxmlParser::TEXT {  // text, see if accumulate

                if save.is_some() { self.text = Some(if self.text.is_none() { save.unwrap() } else { save.unwrap() + self.text.as_ref().unwrap() }); }

                match self.peek_id()? {
                    WbxmlConsts::ENTITY |
                    WbxmlConsts::STR_I |
                    WbxmlConsts::STR_T |
                    WbxmlConsts::LITERAL |
                    WbxmlConsts::LITERAL_C |
                    WbxmlConsts::LITERAL_A |
                    WbxmlConsts::LITERAL_AC => continue,
                    _ => {}
                }
            }

            break;
        }

        self.type_ = min_type;

        if self.type_ > WbxmlParser::TEXT {
            self.type_ = WbxmlParser::TEXT;
        }

        return Ok(self.type_);
    }


    pub fn next_token(&mut self) -> Result<i32, WbxmlError> {

        self.is_whitespace = true;
        self.next_impl()?;
        return Ok(self.type_);
    }



    pub fn next_tag(&mut self) -> Result<i32, WbxmlError> {

        self.next()?;
        if self.type_ == WbxmlParser::TEXT && self.is_whitespace {
            self.next()?;
        }

        if self.type_ != WbxmlParser::END_TAG && self.type_ != WbxmlParser::START_TAG
        { return Err(WbxmlError::XmlPullParserException("unexpected type".to_string())); }

        return Ok(self.type_);
    }


    pub fn next_text(&mut self) -> Result<String, WbxmlError> {
        if self.type_ != WbxmlParser::START_TAG
        { return Err(WbxmlError::XmlPullParserException("precondition: START_TAG".to_string())); }

        self.next()?;

        let result;

        if self.type_ == WbxmlParser::TEXT {
            result = self.get_text().unwrap_or(String::new());
            self.next()?;
        }
        else {
            result = String::new();
        }

        if self.type_ != WbxmlParser::END_TAG
        { return Err(WbxmlError::XmlPullParserException("END_TAG expected".to_string())); }

        return Ok(result);
    }


    pub fn require(&mut self, type_: i32, namespace: Option<String>, name: Option<String>)
        -> Result<(), WbxmlError> {

        if type_ != self.type_
            || (namespace.is_some() && !namespace.as_ref().unwrap().eq(&self.get_namespace_current().unwrap_or(String::new())))
            || (name.is_some() && !name.as_ref().unwrap().eq(&self.get_name().unwrap_or(String::new())))
        {
            // Java: exception(
            //     "expected: " + (type == WAP_EXTENSION ? "WAP Ext." : (TYPES[type] + " {" + namespace + "}" + name)));
            let desc = if type_ == WbxmlParser::WAP_EXTENSION {
                "WAP Ext.".to_string()
            } else {
                format!("{} {{{}}}{}", WbxmlParser::TYPES[type_ as usize], namespace.unwrap_or(String::new()), name.unwrap_or(String::new()))
            };
            return Err(WbxmlError::XmlPullParserException("expected: ".to_string() + &desc));
        }

        Ok(())
    }


    pub fn set_input_reader(&mut self, reader: i32) -> Result<(), WbxmlError> {
        return Err(WbxmlError::XmlPullParserException("InputStream required".to_string()));
    }

    pub fn set_input(&mut self, in_stream: Box<dyn InputStream>, enc: Option<String>)
        -> Result<(), WbxmlError> {

        self.in_stream = Some(in_stream);

        let result = (|| -> Result<(), WbxmlError> {
            self.version = self.read_byte()?;
            self.public_identifier_id = self.read_int()?;

            if self.public_identifier_id == 0 {
                self.read_int()?;
            }

            let charset = self.read_int()?; // skip charset

            if enc.is_none() {
                match charset {
                    4 => self.encoding = Some("ISO-8859-1".to_string()),
                    106 => self.encoding = Some("UTF-8".to_string()),
                    // add more if you need them
                    // http://www.iana.org/assignments/character-sets
                    // case MIBenum: encoding = Name  break;
                    _ => return Err(WbxmlError::RuntimeException(format!("UnsupportedEncodingException {}", charset))),
                }
            }
            else {
                self.encoding = enc;
            }

            let str_tab_size = self.read_int()?;
            self.string_table = Some(vec![0u8; str_tab_size as usize]);

            let mut ok = 0 as usize;
            while ok < str_tab_size as usize {
                let cnt = self.in_stream.as_mut().unwrap().read_buf(self.string_table.as_mut().unwrap(), ok, str_tab_size as usize - ok);
                if cnt <= 0 { break; }
                ok += cnt as usize;
            }

            self.select_page(0, true)?;
            self.select_page(0, false)?;

            Ok(())
        })();

        return match result {
            Err(WbxmlError::RuntimeException(_)) | Err(WbxmlError::IOException(_)) => {
                // Java: catch (IOException e) { exception("Illegal input format"); }
                Err(WbxmlError::XmlPullParserException("Illegal input format".to_string()))
            }
            other => other,
        };
    }

    pub fn set_feature(&mut self, feature: String, value: bool)
        -> Result<(), WbxmlError> {
        if XmlPullParserFeatures::FEATURE_PROCESS_NAMESPACES.eq(&feature) {
            self.process_nsp = value;
        }
        else
        { return Err(WbxmlError::XmlPullParserException(format!("unsupported feature: {}", feature))); }

        Ok(())
    }

    pub fn set_property(&mut self, property: String, value: i32)
        -> Result<(), WbxmlError> {
        return Err(WbxmlError::XmlPullParserException(format!("unsupported property: {}", property)));
    }

    // ---------------------- private / internal methods

    fn adjust_nsp(&mut self)
        -> Result<bool, WbxmlError> {

        let mut any = false;

        let mut i = 0;
        while i < self.attribute_count << 2 {
            // * 4 - 4; i >= 0; i -= 4) {

            let mut attr_name = self.attributes[(i + 2) as usize].clone().unwrap_or(String::new());
            let cut = attr_name.find(':').map(|v| v as i32).unwrap_or(-1);
            let prefix;

            if cut != -1 {
                prefix = attr_name[..cut as usize].to_string();
                attr_name = attr_name[cut as usize + 1..].to_string();
            }
            else if attr_name.eq("xmlns") {
                prefix = attr_name.clone();
                attr_name = String::new();
            }
            else {
                i += 4;
                continue;
            }

            if !prefix.eq("xmlns") {
                any = true;
            }
            else {
                let j = (self.nsp_counts[self.depth as usize] << 1) as usize;
                self.nsp_counts[self.depth as usize] += 1;

                self.nsp_stack = Self::ensure_capacity(self.nsp_stack.clone(), j + 2);
                self.nsp_stack[j] = if attr_name.is_empty() { None } else { Some(attr_name.clone()) };
                self.nsp_stack[j + 1] = self.attributes[(i + 3) as usize].clone();

                if !attr_name.is_empty()
                    && self.attributes[(i + 3) as usize].as_deref() == Some("")
                { return Err(WbxmlError::XmlPullParserException("illegal empty namespace".to_string())); }

                //  prefixMap = new PrefixMap (prefixMap, attrName, attr.getValue ());

                //System.out.println (prefixMap);

                // Java: System.arraycopy(attributes, i + 4, attributes, i, ((--attributeCount) << 2) - i);
                self.attribute_count -= 1;
                let n = (self.attribute_count << 2) - i;
                for k in 0..n {
                    self.attributes[(i + k) as usize] = self.attributes[(i + 4 + k) as usize].clone();
                }

                i -= 4;
            }
            i += 4;
        }

        if any {
            let mut i = (self.attribute_count << 2) - 4;
            while i >= 0 {

                let mut attr_name = self.attributes[(i + 2) as usize].clone().unwrap_or(String::new());
                let cut = attr_name.find(':').map(|v| v as i32).unwrap_or(-1);

                if cut == 0
                // Java: throw new RuntimeException("illegal attribute name: " + attrName + " at " + this);
                { return Err(WbxmlError::RuntimeException(format!("illegal attribute name: {} at {}", attr_name, ""))); }

                else if cut != -1 {
                    let attr_prefix =
                        attr_name[..cut as usize].to_string();

                    attr_name = attr_name[cut as usize + 1..].to_string();

                    let attr_ns = self.get_namespace(Some(attr_prefix.clone()));

                    if attr_ns.is_none()
                    // Java: throw new RuntimeException("Undefined Prefix: " + attrPrefix + " in " + this);
                    { return Err(WbxmlError::RuntimeException(format!("Undefined Prefix: {} in {}", attr_prefix, ""))); }

                    self.attributes[i as usize] = attr_ns.clone();
                    self.attributes[(i + 1) as usize] = Some(attr_prefix);
                    self.attributes[(i + 2) as usize] = Some(attr_name.clone());

                    let mut j = (self.attribute_count << 2) - 4;
                    while j > i {
                        if attr_name.eq(self.attributes[(j + 2) as usize].as_ref().unwrap_or(&String::new()))
                            && attr_ns.as_ref().unwrap().eq(self.attributes[j as usize].as_ref().unwrap())
                        { return Err(WbxmlError::XmlPullParserException(format!(
                            "Duplicate Attribute: {{{}}}{}",
                            attr_ns.as_deref().unwrap_or("null"), attr_name))); }
                        j -= 4;
                    }
                }
                i -= 4;
            }
        }

        let cut = self.name.as_ref().unwrap().find(':').map(|v| v as i32).unwrap_or(-1);

        if cut == 0
        { return Err(WbxmlError::XmlPullParserException(format!("illegal tag name: {}", self.name.as_deref().unwrap_or("null")))); }
        else if cut != -1 {
            let name = self.name.clone().unwrap();
            self.prefix = Some(name[..cut as usize].to_string());
            self.name = Some(name[cut as usize + 1..].to_string());
        }

        self.namespace = self.get_namespace(self.prefix.clone());

        if self.namespace.is_none() {
            if self.prefix.is_some()
            { return Err(WbxmlError::XmlPullParserException(format!("undefined prefix: {}", self.prefix.as_deref().unwrap_or("null")))); }
            self.namespace = Some(WbxmlParser::NO_NAMESPACE.to_string());
        }

        return Ok(any);
    }

    fn set_table(&mut self, page: i32, type_: i32, table: Vec<Option<String>>) {
        if self.string_table.is_some()
        // Java: throw new RuntimeException("setXxxTable must be called before setInput!");
        { panic!("setXxxTable must be called before setInput!"); }
        while self.tables.len() < (3 * page + 3) as usize {
            self.tables.push(None);
        }
        self.tables[(page * 3 + type_) as usize] = Some(table);
    }





    fn exception(&self, desc: String) -> WbxmlError {
        // Java: throw new XmlPullParserException(desc, this, None);
        return WbxmlError::XmlPullParserException(desc);
    }


    fn select_page(&mut self, nr: i32, tags: bool) -> Result<(), WbxmlError> {
        if self.tables.len() == 0 && nr == 0 { return Ok(()); }

        if nr * 3 > self.tables.len() as i32
        { return Err(self.exception(format!("Code Page {} undefined!", nr))); }

        if tags {
            self.tag_table_data = self.tables[(nr * 3 + 0) as usize].clone();
        }
        else {
            self.attr_start_table_data = self.tables[(nr * 3 + 1) as usize].clone();
            self.attr_value_table_data = self.tables[(nr * 3 + 2) as usize].clone();
        }

        Ok(())
    }

    fn next_impl(&mut self)
        -> Result<(), WbxmlError> {

        if self.type_ == WbxmlParser::END_TAG {
            self.depth -= 1;
        }

        if self.degenerated {
            self.type_ = WbxmlParser::END_TAG;
            self.degenerated = false;
            return Ok(());
        }

        self.text = None;
        self.prefix = None;
        self.name = None;

        let mut id = self.peek_id()?;
        while id == WbxmlConsts::SWITCH_PAGE {
            self.next_id = -2;
            let page = self.read_byte()?;
            self.select_page(page, true)?;
            id = self.peek_id()?;
        }
        self.next_id = -2;

        match id {
            -1 => {
                self.type_ = WbxmlParser::END_DOCUMENT;
            }

            WbxmlConsts::END => {
                let sp = ((self.depth - 1) << 2) as usize;

                self.type_ = WbxmlParser::END_TAG;
                self.namespace = self.element_stack[sp].clone();
                self.prefix = self.element_stack[sp + 1].clone();
                self.name = self.element_stack[sp + 2].clone();
            }

            WbxmlConsts::ENTITY => {
                self.type_ = WbxmlParser::ENTITY_REF;
                let c = char::from_u32(self.read_int()? as u16 as u32).unwrap();
                self.text = Some(format!("{}", c));
                self.name = Some(format!("#{}", c as i32));
            }

            WbxmlConsts::STR_I => {
                self.type_ = WbxmlParser::TEXT;
                self.text = Some(self.read_str_i()?);
            }

            WbxmlConsts::EXT_I_0 |
            WbxmlConsts::EXT_I_1 |
            WbxmlConsts::EXT_I_2 |
            WbxmlConsts::EXT_T_0 |
            WbxmlConsts::EXT_T_1 |
            WbxmlConsts::EXT_T_2 |
            WbxmlConsts::EXT_0 |
            WbxmlConsts::EXT_1 |
            WbxmlConsts::EXT_2 |
            WbxmlConsts::OPAQUE => {

                self.type_ = WbxmlParser::WAP_EXTENSION;
                self.wap_code = id;
                self.wap_extension_data = self.parse_wap_extension(id)?;
            }

            WbxmlConsts::PI => {
                // Java: throw new RuntimeException("PI curr. not supp.");
                return Err(WbxmlError::RuntimeException("PI curr. not supp.".to_string()));
                // readPI;
                // break;
            }

            WbxmlConsts::STR_T => {
                self.type_ = WbxmlParser::TEXT;
                self.text = Some(self.read_str_t()?);
            }

            _ => {
                self.parse_element(id)?;
            }
        }
        //        }
        //      while (next == None);

        //        return next;

        Ok(())
    }

    /** Overwrite this method to intercept all wap events */

    pub fn parse_wap_extension(&mut self, id: i32) -> Result<WapExtensionData, WbxmlError> {

        match id {
            WbxmlConsts::EXT_I_0 |
            WbxmlConsts::EXT_I_1 |
            WbxmlConsts::EXT_I_2 => {
                return Ok(WapExtensionData::Str(self.read_str_i()?));
            }

            WbxmlConsts::EXT_T_0 |
            WbxmlConsts::EXT_T_1 |
            WbxmlConsts::EXT_T_2 => {
                // Java: return new Integer(readInt());
                return Ok(WapExtensionData::Int(self.read_int()?));
            }

            WbxmlConsts::EXT_0 |
            WbxmlConsts::EXT_1 |
            WbxmlConsts::EXT_2 => {
                return Ok(WapExtensionData::None);
            }

            WbxmlConsts::OPAQUE => {
                let mut count = self.read_int()?;
                let mut buf = vec![0u8; count as usize];

                while count > 0 {
                    let blen = buf.len() as i32;
                    count -= self.in_stream.as_mut().unwrap().read_buf(&mut buf, (blen - count) as usize, count as usize);
                }

                return Ok(WapExtensionData::Bytes(buf));
            } // case OPAQUE


            _ => {
                // Java: exception("illegal id: "+id);
                // Java: return None; // dead code
                return Err(self.exception(format!("illegal id: {}", id)));
            }
        } // SWITCH
    }

    pub fn read_attr(&mut self) -> Result<(), WbxmlError> {

        let mut id = self.read_byte()?;
        let mut i = 0;

        while id != 1 {

            while id == WbxmlConsts::SWITCH_PAGE {
                let page = self.read_byte()?;
                self.select_page(page, false)?;
                id = self.read_byte()?;
            }

            let mut name = self.resolve_id(self.attr_start_table_data.clone(), id)?;
            let mut value: String;

            let cut = name.find('=').map(|v| v as i32).unwrap_or(-1);

            if cut == -1 {
                value = String::new();
            }
            else {
                value = name[cut as usize + 1..].to_string();
                name = name[..cut as usize].to_string();
            }

            id = self.read_byte()?;
            while id > 128
                || id == WbxmlConsts::SWITCH_PAGE
                || id == WbxmlConsts::ENTITY
                || id == WbxmlConsts::STR_I
                || id == WbxmlConsts::STR_T
                || (id >= WbxmlConsts::EXT_I_0 && id <= WbxmlConsts::EXT_I_2)
                || (id >= WbxmlConsts::EXT_T_0 && id <= WbxmlConsts::EXT_T_2) {

                match id {
                    WbxmlConsts::SWITCH_PAGE => {
                        let page = self.read_byte()?;
                        self.select_page(page, false)?;
                    }

                    WbxmlConsts::ENTITY => {
                        value.push(char::from_u32(self.read_int()? as u16 as u32).unwrap());
                    }

                    WbxmlConsts::STR_I => {
                        value.push_str(&self.read_str_i()?);
                    }

                    WbxmlConsts::EXT_I_0 |
                    WbxmlConsts::EXT_I_1 |
                    WbxmlConsts::EXT_I_2 |
                    WbxmlConsts::EXT_T_0 |
                    WbxmlConsts::EXT_T_1 |
                    WbxmlConsts::EXT_T_2 |
                    WbxmlConsts::EXT_0 |
                    WbxmlConsts::EXT_1 |
                    WbxmlConsts::EXT_2 |
                    WbxmlConsts::OPAQUE => {
                        let data = self.parse_wap_extension(id)?;
                        value.push_str(&self.resolve_wap_extension(id, data));
                    }

                    WbxmlConsts::STR_T => {
                        value.push_str(&self.read_str_t()?);
                    }

                    _ => {
                        value.push_str(&self.resolve_id(self.attr_value_table_data.clone(), id)?);
                    }
                }

                id = self.read_byte()?;
            }

            self.attributes = Self::ensure_capacity(self.attributes.clone(), (i + 4) as usize);

            self.attributes[i as usize] = Some(String::new());
            self.attributes[(i + 1) as usize] = None;
            self.attributes[(i + 2) as usize] = Some(name);
            self.attributes[(i + 3) as usize] = Some(value);

            self.attribute_count += 1;
            i += 4;
        }

        Ok(())
    }

    fn peek_id(&mut self) -> Result<i32, WbxmlError> {
        if self.next_id == -2 {
            self.next_id = self.in_stream.as_mut().unwrap().read();
        }
        return Ok(self.next_id);
    }

    /** overwrite for own WAP extension handling in attributes and high level parsing
     * (above nextToken() level) */

    // Java: protected String resolveWapExtension(int id, Object data){
    pub fn resolve_wap_extension(&self, id: i32, data: WapExtensionData) -> String {

        if let WapExtensionData::Bytes(b) = &data {
            let mut sb = String::new();

            for i in 0..b.len() {
                sb.push(WbxmlParser::HEX_DIGITS.chars().nth(((b[i] >> 4) & 0x0f) as usize).unwrap());
                sb.push(WbxmlParser::HEX_DIGITS.chars().nth((b[i] & 0x0f) as usize).unwrap());
            }
            return sb;
        }

        return format!("$({})", wap_extension_data_to_string(&data));
    }

    fn resolve_id(&mut self, tab: Option<Vec<Option<String>>>, id: i32) -> Result<String, WbxmlError> {
        let idx = (id & 0x07f) - 5;
        if idx == -1 {
            self.wap_code = -1;
            return Ok(self.read_str_t()?);
        }
        if idx < 0
            || tab.is_none()
            || idx as usize >= tab.as_ref().unwrap().len()
            || tab.as_ref().unwrap()[idx as usize].is_none()
        // Java: throw new IOException("id " + id + " undef.");
        { return Err(WbxmlError::IOException(format!("id {} undef.", id))); }

        self.wap_code = idx + 5;

        return Ok(tab.unwrap()[idx as usize].clone().unwrap());
    }

    fn parse_element(&mut self, id: i32)
        -> Result<(), WbxmlError> {

        self.type_ = WbxmlParser::START_TAG;
        self.name = Some(self.resolve_id(self.tag_table_data.clone(), id & 0x03f)?);

        self.attribute_count = 0;
        if (id & 128) != 0 {
            self.read_attr()?;
        }

        self.degenerated = (id & 64) == 0;

        let sp = (self.depth << 2) as usize;
        self.depth += 1;

        // transfer to element stack

        self.element_stack = Self::ensure_capacity(self.element_stack.clone(), sp + 4);
        self.element_stack[sp + 3] = self.name.clone();

        if self.depth as usize >= self.nsp_counts.len() {
            let mut bigger = vec![0; self.depth as usize + 4];
            for i in 0..self.nsp_counts.len() {
                bigger[i] = self.nsp_counts[i];
            }
            self.nsp_counts = bigger;
        }

        self.nsp_counts[self.depth as usize] = self.nsp_counts[(self.depth - 1) as usize];

        let mut i = self.attribute_count - 1;
        while i > 0 {
            let mut j = 0;
            while j < i {
                if self.get_attribute_name(i)
                    .eq(&self.get_attribute_name(j))
                { return Err(self.exception(format!(
                    "Duplicate Attribute: {}",
                    self.get_attribute_name(i).unwrap_or(String::new())))); }
                j += 1;
            }
            i -= 1;
        }

        if self.process_nsp {
            self.adjust_nsp()?;
        }
        else {
            self.namespace = Some(String::new());
        }

        self.element_stack[sp] = self.namespace.clone();
        self.element_stack[sp + 1] = self.prefix.clone();
        self.element_stack[sp + 2] = self.name.clone();

        Ok(())
    }

    fn ensure_capacity(
        arr: Vec<Option<String>>,
        required: usize) -> Vec<Option<String>> {
        if arr.len() >= required {
            return arr;
        }
        let mut bigger = vec![None; required + 16];
        for i in 0..arr.len() {
            bigger[i] = arr[i].clone();
        }
        return bigger;
    }

    fn read_byte(&mut self) -> Result<i32, WbxmlError> {
        let i = self.in_stream.as_mut().unwrap().read();
        if i == -1
        // Java: throw new IOException("Unexpected EOF");
        { return Err(WbxmlError::IOException("Unexpected EOF".to_string())); }
        return Ok(i);
    }

    fn read_int(&mut self) -> Result<i32, WbxmlError> {
        let mut result = 0;
        let mut i;

        // Java: do {
        //     i = readByte();
        //     result = (result << 7) | (i & 0x7f);
        // }
        // while ((i & 0x80) != 0);
        loop {
            i = self.read_byte()?;
            result = (result << 7) | (i & 0x7f);
            if (i & 0x80) == 0 { break; }
        }

        return Ok(result);
    }

    fn read_str_i(&mut self) -> Result<String, WbxmlError> {
        let mut buf: Vec<u8> = Vec::new();
        let mut wsp = true;
        loop {
            let i = self.in_stream.as_mut().unwrap().read();
            if i == 0 {
                break;
            }
            if i == -1 {
                // Java: throw new IOException(UNEXPECTED_EOF);
                return Err(WbxmlError::IOException(WbxmlParser::UNEXPECTED_EOF.to_string()));
            }
            if i > 32 {
                wsp = false;
            }
            buf.push(i as u8);
        }
        self.is_whitespace = wsp;
        // Java: String result = new String(buf.toByteArray(), encoding);
        let result = String::from_utf8_lossy(&buf).to_string();
        return Ok(result);
    }

    fn read_str_t(&mut self) -> Result<String, WbxmlError> {
        let pos = self.read_int()?;
        // As the main reason of stringTable is compression we build a cache of Strings
        // stringTable is supposed to help create Strings from parts which means some cache hit rate
        // This will help to minimize the Strings created when invoking readStrT() repeatedly
        if self.cache_string_table.is_none() {
            //Lazy init if device is not using StringTable but inline 0x03 strings
            self.cache_string_table = Some(HashMap::new());
        }
        let mut for_return = self.cache_string_table.as_ref().unwrap().get(&pos).cloned();
        if for_return.is_none() {

            let mut end = pos;
            while (end as usize) < self.string_table.as_ref().unwrap().len() && self.string_table.as_ref().unwrap()[end as usize] != '\0' as u8 {
                end += 1;
            }
            // Java: forReturn = new String(stringTable, pos, end-pos, encoding);
            for_return = Some(String::from_utf8_lossy(&self.string_table.as_ref().unwrap()[pos as usize..end as usize]).to_string());
            self.cache_string_table.as_mut().unwrap().insert(pos, for_return.clone().unwrap());
        }
        return Ok(for_return.unwrap());
    }

    /**
     * Sets the tag table for a given page.
     * The first string in the array defines tag 5, the second tag 6 etc.
     */

    pub fn set_tag_table(&mut self, page: i32, table: &[Option<&'static str>]) {
        self.set_table(page, 0, table.iter().map(|s| s.map(|x| x.to_string())).collect());

        //        this.tagTable = tagTable;
        //      if (page != 0)
        //        throw new RuntimeException("code pages curr. not supp.");
    }

    /** Sets the attribute start Table for a given page.
     *    The first string in the array defines attribute
     *  5, the second attribute 6 etc. Please use the
     *  character '=' (without quote!) as delimiter
     *  between the attribute name and the (start of the) value
     */

    pub fn set_attr_start_table(
        &mut self,
        page: i32,
        table: &[Option<&'static str>]) {

        self.set_table(page, 1, table.iter().map(|s| s.map(|x| x.to_string())).collect());
    }

    /** Sets the attribute value Table for a given page.
     *    The first string in the array defines attribute value 0x85,
     *  the second attribute value 0x86 etc.
     */

    pub fn set_attr_value_table(
        &mut self,
        page: i32,
        table: &[Option<&'static str>]) {

        self.set_table(page, 2, table.iter().map(|s| s.map(|x| x.to_string())).collect());
    }

    /** Returns the token ID for start tags or the event type for wap proprietary events
     * such as OPAQUE.
     */

    pub fn get_wap_code(&self) -> i32 {
        return self.wap_code;
    }

    pub fn get_wap_extension_data(&self) -> &WapExtensionData {
        return &self.wap_extension_data;
    }
}

/** org.xmlpull.v1.XmlPullParser constants used by WbxmlParser */
pub struct XmlPullParserFeatures;

impl XmlPullParserFeatures {
    pub const FEATURE_PROCESS_NAMESPACES: &'static str = "http://xmlpull.org/v1/doc/features.html#process-namespaces";
}

/** java.lang.String.valueOf(Object) stand-in for resolveWapExtension */
pub fn wap_extension_data_to_string(data: &WapExtensionData) -> String {
    return match data {
        WapExtensionData::Bytes(b) => format!("[B@{}", b.len()),
        WapExtensionData::Int(i) => format!("{}", i),
        WapExtensionData::Str(s) => s.clone(),
        WapExtensionData::None => "null".to_string(),
    };
}
