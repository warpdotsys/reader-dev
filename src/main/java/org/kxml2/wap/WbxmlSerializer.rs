use crate::prelude::*;
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

//Contributors: Jonathan Cox, Bogdan Onoiu, Jerry Tian

// package org.kxml2.wap;

// import java.io.*;
// import java.util.*;

// import org.xmlpull.v1.*;

// TODO: make some of the "direct" WBXML token writing methods public??

/**
 * A class for writing WBXML. Does not support namespaces yet.
 */
// public class WbxmlSerializer implements XmlSerializer {

/** OutputStream interface (java.io.OutputStream), as far as needed by WbxmlSerializer. */
pub trait OutputStream {
    fn write_int(&mut self, b: i32);
    fn write_bytes(&mut self, b: &[u8]);
}

/** java.lang.Object stand-in for the data parameter of writeWapExtension */
pub enum WapExtensionData {
    Bytes(Vec<u8>),
    Str(String),
}

/** Marker type implementing the Wbxml trait so its constants are addressable (WbxmlConsts::X). */
pub struct WbxmlConsts;

impl crate::org_kxml2_wap_wbxml::Wbxml for WbxmlConsts {}

pub struct WbxmlSerializer {

    // Java: Hashtable stringTable = new Hashtable();
    pub string_table: HashMap<String, i32>,

    pub out: Option<Box<dyn OutputStream>>,

    // Java: ByteArrayOutputStream buf = new ByteArrayOutputStream();
    pub buf: Vec<u8>,
    // Java: ByteArrayOutputStream stringTableBuf = new ByteArrayOutputStream();
    pub string_table_buf: Vec<u8>,

    pub pending: Option<String>,
    pub depth: i32,
    pub name: Option<String>,
    pub namespace: Option<String>,
    // Java: Vector attributes = new Vector();
    pub attributes: Vec<String>,

    pub attr_start_table: HashMap<String, [i32; 2]>,
    pub attr_value_table: HashMap<String, [i32; 2]>,
    pub tag_table: HashMap<String, [i32; 2]>,

    pub attr_page: i32,
    pub tag_page: i32,

    pub encoding: Option<String>,

    pub header_sent: bool,
}

impl WbxmlSerializer {

    pub fn new() -> WbxmlSerializer {
        WbxmlSerializer {
            string_table: HashMap::new(),
            out: None,
            buf: Vec::new(),
            string_table_buf: Vec::new(),
            pending: None,
            depth: 0,
            name: None,
            namespace: None,
            attributes: Vec::new(),
            attr_start_table: HashMap::new(),
            attr_value_table: HashMap::new(),
            tag_table: HashMap::new(),
            attr_page: 0,
            tag_page: 0,
            encoding: None,
            header_sent: false,
        }
    }

    /**
     * Write an attribute.
     * Calls to attribute() MUST follow a call to startTag() immediately.
     * If there is no prefix defined for the given namespace,
     * a prefix will be defined automatically.
     */
    pub fn attribute(&mut self, namespace: String, name: String, value: String) -> &WbxmlSerializer {
        self.attributes.push(name);
        self.attributes.push(value);
        return self;
    }


    pub fn cdsect(&mut self, cdsect: String) -> Result<(), String> {
        self.text(cdsect)?;
        Ok(())
    }

    /**
     * Add comment. Ignore for WBXML.
     */
    pub fn comment(&mut self, comment: String) {
        // silently ignore comment
    }

    /**
     * Docdecl isn't supported for WBXML.
     */
    pub fn docdecl(&mut self, docdecl: String) {
        // Java: throw new RuntimeException ("Cannot write docdecl for WBXML");
        panic!("Cannot write docdecl for WBXML");
    }

    /**
     * EntityReference not supported for WBXML.
     */
    pub fn entity_ref(&mut self, er: String) {
        // Java: throw new RuntimeException ("EntityReference not supported for WBXML");
        panic!("EntityReference not supported for WBXML");
    }

    /**
     * Return current tag depth.
     */
    pub fn get_depth(&self) -> i32 {
        return self.depth;
    }

    /**
     * Return the current value of the feature with given name.
     */
    pub fn get_feature(&mut self, name: String) -> bool {
        return false;
    }

    /**
     * Returns the namespace URI of the current element as set by startTag().
     * Namespaces is not yet implemented.
     */
    pub fn get_namespace(&self) -> Option<String> {
        // Namespaces is not yet implemented. So only None can be setted
        return None;
    }

    /**
     * Returns the name of the current element as set by startTag().
     * It can only be None before first call to startTag() or when last endTag()
     * is called to close first startTag().
     */
    pub fn get_name(&self) -> Option<String> {
        return self.pending.clone();
    }

    /**
     * Prefix for namespace not supported for WBXML. Not yet implemented.
     */
    pub fn get_prefix(&self, nsp: String, create: bool) -> String {
        // Java: throw new RuntimeException ("NYI");
        panic!("NYI");
    }

    /**
     * Look up the value of a property.
     * @param name The name of property. Name is any fully-qualified URI.
     * @return The value of named property.
     */
    pub fn get_property(&self, name: String) -> Option<i32> {
        return None;
    }

    pub fn ignorable_whitespace(&mut self, sp: String) {
    }

    /**
     * Finish writing.
     * All unclosed start tags will be closed and output will be flushed.
     * After calling this method no more output can be serialized until
     * next call to setOutput().
     */
    pub fn end_document(&mut self) -> Result<(), String> {
        self.flush()
    }

    /**
     * Write all pending output to the stream.
     * After first call string table willn't be used and you can't add tag
     * which is not in tag table.
     */
    pub fn flush(&mut self) -> Result<(), String> {
        self.check_pending(false)?;

        if !self.header_sent {
            WbxmlSerializer::write_int(self.out.as_mut().unwrap().as_mut(), self.string_table_buf.len() as i32);
            self.out.as_mut().unwrap().write_bytes(&self.string_table_buf.clone());
            self.header_sent = true;
        }

        self.out.as_mut().unwrap().write_bytes(&self.buf.clone());
        self.buf.clear();

        Ok(())
    }

    pub fn check_pending(&mut self, degenerated: bool) -> Result<(), String> {
        if self.pending.is_none() {
            return Ok(());
        }

        let len = self.attributes.len();

        let mut idx = self.tag_table.get(self.pending.as_ref().unwrap()).map(|v| *v);

        // if no entry in known table, then add as literal
        if idx.is_none() {
            self.buf.push(
                if len == 0 {
                    if degenerated { WbxmlConsts::LITERAL as u8 } else { WbxmlConsts::LITERAL_C as u8 }
                }
                else {
                    if degenerated { WbxmlConsts::LITERAL_A as u8 } else { WbxmlConsts::LITERAL_AC as u8 }
                });

            self.write_str_t(self.pending.clone().unwrap(), false)?;
        }
        else {
            let idx = idx.unwrap();
            if idx[0] != self.tag_page {
                self.tag_page = idx[0];
                self.buf.push(WbxmlConsts::SWITCH_PAGE as u8);
                self.buf.push(self.tag_page as u8);
            }
            self.buf.push(
                if len == 0 {
                    if degenerated { idx[1] as u8 } else { (idx[1] | 64) as u8 }
                }
                else {
                    if degenerated { (idx[1] | 128) as u8 } else { (idx[1] | 192) as u8 }
                });
        }

        let mut i = 0;
        while i < len {
            let mut idx = self.attr_start_table.get(&self.attributes[i]).map(|v| *v);

            if idx.is_none() {
                self.buf.push(WbxmlConsts::LITERAL as u8);
                self.write_str_t(self.attributes[i].clone(), false)?;
            }
            else {
                let idx = idx.unwrap();
                if idx[0] != self.attr_page {
                    self.attr_page = idx[0];
                    self.buf.push(0);
                    self.buf.push(self.attr_page as u8);
                }
                self.buf.push(idx[1] as u8);
            }
            let mut idx = self.attr_value_table.get(&self.attributes[i + 1]).map(|v| *v);
            if idx.is_none() {
                self.write_str(self.attributes[i + 1].clone())?;
            }
            else {
                let idx = idx.unwrap();
                if idx[0] != self.attr_page {
                    self.attr_page = idx[0];
                    self.buf.push(0);
                    self.buf.push(self.attr_page as u8);
                }
                self.buf.push(idx[1] as u8);
            }
            i += 2;
        }

        if len > 0 {
            self.buf.push(WbxmlConsts::END as u8);
        }

        self.pending = None;
        self.attributes.clear();

        Ok(())
    }

    /**
     * Not Yet Implemented.
     */
    pub fn processing_instruction(&mut self, pi: String) {
        // Java: throw new RuntimeException ("PI NYI");
        panic!("PI NYI");
    }

    /**
     * Set feature identified by name. There are no supported functions.
     */
    pub fn set_feature(&mut self, name: String, value: bool) {
        // Java: throw new IllegalArgumentException ("unknown feature "+name);
        panic!("unknown feature {}", name);
    }

    /**
     * Set the output to the given writer. Wbxml requires an OutputStream.
     */
    pub fn set_output_writer(&mut self, writer: i32) {
        // Java: throw new RuntimeException ("Wbxml requires an OutputStream!");
        panic!("Wbxml requires an OutputStream!");
    }

    /**
     * Set to use binary output stream with given encoding.
     */
    pub fn set_output(&mut self, out: Box<dyn OutputStream>, encoding: Option<String>) -> Result<(), String> {

        self.encoding = if encoding.is_none() { Some("UTF-8".to_string()) } else { encoding };
        self.out = Some(out);

        self.buf = Vec::new();
        self.string_table_buf = Vec::new();
        self.header_sent = false;

        // ok, write header

        Ok(())
    }

    /**
     * Binds the given prefix to the given namespace. Not yet implemented.
     */
    pub fn set_prefix(&mut self, prefix: String, nsp: String) {
        // Java: throw new RuntimeException("NYI");
        panic!("NYI");
    }

    /**
     * Set the value of a property. There are no supported properties.
     */
    pub fn set_property(&mut self, property: String, value: i32) {
        // Java: throw new IllegalArgumentException ("unknown property "+property);
        panic!("unknown property {}", property);
    }

    /**
     * Write version and encoding information.
     * This method can only be called just after setOutput.
     * @param encoding Document encoding. Default is UTF-8.
     * @param standalone Not used in WBXML.
     */
    pub fn start_document(&mut self, encoding: Option<String>, standalone: Option<bool>) -> Result<(), String> {
        self.out.as_mut().unwrap().write_int(0x03); // version 1.3
        // http://www.openmobilealliance.org/tech/omna/omna-wbxml-public-docid.htm
        self.out.as_mut().unwrap().write_int(0x01); // unknown or missing public identifier

        // default encoding is UTF-8

        if encoding.is_some() {
            self.encoding = encoding.clone();
        }

        if self.encoding.clone().unwrap().to_uppercase().eq("UTF-8") {
            self.out.as_mut().unwrap().write_int(106);
        }
        else if self.encoding.clone().unwrap().to_uppercase().eq("ISO-8859-1") {
            self.out.as_mut().unwrap().write_int(0x04);
        }
        else {
            // Java: throw new UnsupportedEncodingException(encoding);
            return Err(format!("Unsupported encoding: {}", encoding.unwrap_or(String::new())));
        }

        Ok(())
    }


    pub fn start_tag(&mut self, namespace: Option<String>, name: String) -> Result<&WbxmlSerializer, String> {

        if namespace.is_some() && !"".eq(namespace.as_ref().unwrap())
        // Java: throw new RuntimeException ("NSP NYI");
        { return Err("NSP NYI".to_string()); }

        //current = new State(current, prefixMap, name);

        self.check_pending(false)?;
        self.pending = Some(name);
        self.depth += 1;

        return Ok(self);
    }

    pub fn text_chars(&mut self, chars: Vec<char>, start: i32, len: i32) -> Result<&WbxmlSerializer, String> {
        self.check_pending(false)?;
        self.write_str(chars[start as usize..(start + len) as usize].iter().collect())?;
        return Ok(self);
    }

    pub fn text(&mut self, text: String) -> Result<&WbxmlSerializer, String> {
        self.check_pending(false)?;
        self.write_str(text)?;
        return Ok(self);
    }

    /**
     * Used in text() and attribute() to write text.
     */
    fn write_str(&mut self, text: String) -> Result<(), String> {
        let mut p0 = 0 as usize;
        let mut last_cut = 0 as usize;
        let len = text.chars().count();

        if self.header_sent {
            // Java: writeStrI(buf, text);
            WbxmlSerializer::write_str_i_buf(&mut self.buf, &text);
            return Ok(());
        }

        let chars: Vec<char> = text.chars().collect();
        while p0 < len {
            while p0 < len && (chars[p0] as i32) < 'A' as i32 { // skip interpunctation
                p0 += 1;
            }
            let mut p1 = p0;
            while p1 < len && (chars[p1] as i32) >= 'A' as i32 {
                p1 += 1;
            }

            if p1 - p0 > 10 {
                if p0 > last_cut && chars[p0 - 1] == ' '
                    && !self.string_table.contains_key(&chars[p0..p1].iter().collect::<String>()) {
                    self.buf.push(WbxmlConsts::STR_T as u8);
                    self.write_str_t(chars[last_cut..p1].iter().collect(), false)?;
                }
                else {
                    let mut p0 = p0;
                    if p0 > last_cut && chars[p0 - 1] == ' ' {
                        p0 -= 1;
                    }

                    if p0 > last_cut {
                        self.buf.push(WbxmlConsts::STR_T as u8);
                        self.write_str_t(chars[last_cut..p0].iter().collect(), false)?;
                    }
                    self.buf.push(WbxmlConsts::STR_T as u8);
                    self.write_str_t(chars[p0..p1].iter().collect(), true)?;
                }
                last_cut = p1;
            }
            p0 = p1;
        }

        if last_cut < len {
            self.buf.push(WbxmlConsts::STR_T as u8);
            self.write_str_t(chars[last_cut..len].iter().collect(), false)?;
        }

        Ok(())
    }



    pub fn end_tag(&mut self, namespace: String, name: String) -> Result<&WbxmlSerializer, String> {
        //        current = current.prev;
        if self.pending.is_some() {
            self.check_pending(true)?;
        }
        else {
            self.buf.push(WbxmlConsts::END as u8);
        }
        self.depth -= 1;
        return Ok(self);
    }

    /**
     * @throws IOException
     */
    pub fn write_wap_extension(&mut self, type_: i32, data: WapExtensionData) -> Result<(), String> {
        self.check_pending(false)?;
        self.buf.push(type_ as u8);
        match type_ {
        WbxmlConsts::EXT_0 |
        WbxmlConsts::EXT_1 |
        WbxmlConsts::EXT_2 => {
        }

        WbxmlConsts::OPAQUE => {
            if let WapExtensionData::Bytes(bytes) = data {
                WbxmlSerializer::write_int_bytes(&mut self.buf, bytes.len() as i32);
                self.buf.extend(bytes);
            }
        }

        WbxmlConsts::EXT_I_0 |
        WbxmlConsts::EXT_I_1 |
        WbxmlConsts::EXT_I_2 => {
            if let WapExtensionData::Str(s) = data {
                // Java: writeStrI(buf, (String) data);
                WbxmlSerializer::write_str_i_buf(&mut self.buf, &s);
            }
        }

        WbxmlConsts::EXT_T_0 |
        WbxmlConsts::EXT_T_1 |
        WbxmlConsts::EXT_T_2 => {
            if let WapExtensionData::Str(s) = data {
                self.write_str_t(s, false)?;
            }
        }

        _ => {
            // Java: throw new IllegalArgumentException();
            return Err("IllegalArgumentException".to_string());
        }
        }
        Ok(())
    }

    // ------------- internal methods --------------------------

    pub fn write_int(out: &mut dyn OutputStream, i: i32) {
        let mut buf = [0u8; 5];
        let mut idx = 0 as usize;
        let mut i = i;

        // Java: do {
        //     buf[idx++] = (byte) (i & 0x7f);
        //     i = i >> 7;
        // }
        // while (i != 0);
        loop {
            buf[idx] = (i & 0x7f) as u8;
            idx += 1;
            i = i >> 7;
            if i == 0 { break; }
        }

        while idx > 1 {
            idx -= 1;
            out.write_int((buf[idx] | 0x80) as i32);
        }
        out.write_int(buf[0] as i32);
    }

    /** like writeInt, but into the internal byte buffer (helper for this translation) */
    fn write_int_bytes(buf: &mut Vec<u8>, i: i32) {
        let mut b = [0u8; 5];
        let mut idx = 0 as usize;
        let mut i = i;

        // Java: do {
        //     buf[idx++] = (byte) (i & 0x7f);
        //     i = i >> 7;
        // }
        // while (i != 0);
        loop {
            b[idx] = (i & 0x7f) as u8;
            idx += 1;
            i = i >> 7;
            if i == 0 { break; }
        }

        while idx > 1 {
            idx -= 1;
            buf.push(b[idx] | 0x80);
        }
        buf.push(b[0]);
    }

    // Java: static void writeStrI(OutputStream out, String s) throws IOException
    fn write_str_i_buf(buf: &mut Vec<u8>, s: &String) {
        let data: Vec<u8> = s.as_bytes().to_vec();
        buf.extend(data);
        buf.push(0);
    }

    // Java: private final void writeStrT(String s, boolean mayPrependSpace) throws IOException {
    fn write_str_t(&mut self, s: String, may_prepend_space: bool) -> Result<(), String> {

        let idx = self.string_table.get(&s).map(|v| *v);
        let val = match idx {
            None => self.add_to_string_table(s, may_prepend_space)?,
            Some(i) => i,
        };
        WbxmlSerializer::write_int_bytes(&mut self.buf, val);

        Ok(())
    }


    /**
     * Add string to string table. Not permitted after string table has been flushed.
     *
     * @param s string to be added to the string table
     * @param mayPrependSpace is set, a space is prepended to the string to archieve better compression results
     * @return offset of s in the string table
     */
    pub fn add_to_string_table(&mut self, s: String, may_prepend_space: bool) -> Result<i32, String> {
        if self.header_sent {
            // Java: throw new IOException("stringtable sent");
            return Err("stringtable sent".to_string());
        }

        let i = self.string_table_buf.len() as i32;
        let mut offset = i;
        let mut s = s;
        if s.chars().next().unwrap() as i32 >= '0' as i32 && may_prepend_space {
            s = format!(" {}", s);
            offset += 1;
        }

        self.string_table.insert(s.clone(), i);
        if s.chars().next().unwrap() == ' ' {
            self.string_table.insert(s[1..].to_string(), i + 1);
        }
        let j = s.rfind(' ');
        if j.is_some() && j.unwrap() > 1 {
            let t: String = s[j.unwrap()..].to_string();
            let k = t.as_bytes().len() as i32;
            self.string_table.insert(t.clone(), i + k);
            self.string_table.insert(s[j.unwrap() + 1..].to_string(), i + k + 1);
        }

        // Java: writeStrI(stringTableBuf, s);
        WbxmlSerializer::write_str_i_buf(&mut self.string_table_buf, &s);
        // Java: stringTableBuf.flush();
        return Ok(offset);
    }

    /**
     * Sets the tag table for a given page.
     * The first string in the array defines tag 5, the second tag 6 etc.
     */
    pub fn set_tag_table(&mut self, page: i32, tag_table: &[Option<&'static str>]) {
        // TODO: clear entries in tagTable?

        let mut i = 0;
        while i < tag_table.len() {
            if tag_table[i].is_some() {
                let idx = [page, (i + 5) as i32];
                self.tag_table.insert(tag_table[i].unwrap().to_string(), idx);
            }
            i += 1;
        }
    }

    /**
     * Sets the attribute start Table for a given page.
     * The first string in the array defines attribute
     * 5, the second attribute 6 etc.
     *  Please use the
     *  character '=' (without quote!) as delimiter
     *  between the attribute name and the (start of the) value
     */
    pub fn set_attr_start_table(&mut self, page: i32, attr_start_table: &[Option<&'static str>]) {

        let mut i = 0;
        while i < attr_start_table.len() {
            if attr_start_table[i].is_some() {
                let idx = [page, (i + 5) as i32];
                self.attr_start_table.insert(attr_start_table[i].unwrap().to_string(), idx);
            }
            i += 1;
        }
    }

    /**
     * Sets the attribute value Table for a given page.
     * The first string in the array defines attribute value 0x85,
     * the second attribute value 0x86 etc.
     * Must be called BEFORE use attribute(), flush() etc.
     */
    pub fn set_attr_value_table(&mut self, page: i32, attr_value_table: &[Option<&'static str>]) {
        // clear entries in this.table!
        let mut i = 0;
        while i < attr_value_table.len() {
            if attr_value_table[i].is_some() {
                let idx = [page, (i + 0x085) as i32];
                self.attr_value_table.insert(attr_value_table[i].unwrap().to_string(), idx);
            }
            i += 1;
        }
    }
}
