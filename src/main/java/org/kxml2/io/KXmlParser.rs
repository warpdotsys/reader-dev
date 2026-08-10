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

// Contributors: Paul Hackenberger (unterminated entity handling in relaxed mode)

// package org.kxml2.io;

// import java.io.*;
// import java.util.*;

// import org.xmlpull.v1.*;

/** A simple, pull based XML parser. This classe replaces the kXML 1
    XmlParser class and the corresponding event classes. */

/** Reader interface (java.io.Reader), as far as needed by KXmlParser. */
pub trait Reader {
    fn read(&mut self) -> i32;
    fn read_buf(&mut self, buf: &mut Vec<u16>, off: usize, len: usize) -> i32;
    fn to_string(&self) -> String;
}

/** InputStream interface (java.io.InputStream), as far as needed by KXmlParser. */
pub trait InputStream {
    fn read(&mut self) -> i32;
}

/** XmlPullParserException / IOException / RuntimeException stand-ins */
pub enum XmlPullError {
    XmlPullParserException(String),
    IOException(String),
    RuntimeException(String),
}

// public class KXmlParser implements XmlPullParser {

pub struct KXmlParser {

    pub location: Option<String>,

    // static final private String UNEXPECTED_EOF = "Unexpected EOF";
    // static final private String ILLEGAL_TYPE = "Wrong event type";
    // static final private int LEGACY = 999;
    // static final private int XML_DECL = 998;

    // general

    pub version: Option<String>,
    pub standalone: Option<bool>,

    pub process_nsp: bool,
    pub relaxed: bool,
    // Java: private Hashtable entityMap;
    pub entity_map: HashMap<String, String>,
    pub depth: i32,
    pub element_stack: Vec<Option<String>>,   // Java: new String[16]
    pub nsp_stack: Vec<Option<String>>,       // Java: new String[8]
    pub nsp_counts: Vec<i32>,                 // Java: new int[4]

    // source

    pub reader: Option<Box<dyn Reader>>,
    pub encoding: Option<String>,
    pub src_buf: Vec<u16>,                    // Java: char[]

    pub src_pos: i32,
    pub src_count: i32,

    pub line: i32,
    pub column: i32,

    // txtbuffer

    /** Target buffer for storing incoming text (including aggregated resolved entities) */
    pub txt_buf: Vec<u16>,                    // Java: char[] txtBuf = new char[128];
    /** Write position  */
    pub txt_pos: i32,

    // Event-related

    pub type_: i32,
    pub is_whitespace: bool,
    pub namespace: Option<String>,
    pub prefix: Option<String>,
    pub name: Option<String>,

    pub degenerated: bool,
    pub attribute_count: i32,
    pub attributes: Vec<Option<String>>,      // Java: new String[16]
    //    private int stackMismatch = 0;
    pub error: Option<String>,

    /**
     * A separate peek buffer seems simpler than managing
     * wrap around in the first level read buffer */

    pub peek: [i32; 2],                       // Java: int[] peek = new int[2];
    pub peek_count: i32,
    pub was_cr: bool,

    pub unresolved: bool,
    pub token: bool,
}

impl KXmlParser {

    pub const UNEXPECTED_EOF: &'static str = "Unexpected EOF";
    pub const ILLEGAL_TYPE: &'static str = "Wrong event type";
    pub const LEGACY: i32 = 999;
    pub const XML_DECL: i32 = 998;

    // (org.xmlpull.v1.XmlPullParser constants, since KXmlParser implements XmlPullParser)
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

    pub fn new() -> KXmlParser {
        KXmlParser {
            location: None,
            version: None,
            standalone: None,
            process_nsp: false,
            relaxed: false,
            entity_map: HashMap::new(),
            depth: 0,
            element_stack: vec![None; 16],
            nsp_stack: vec![None; 8],
            nsp_counts: vec![0; 4],
            reader: None,
            encoding: None,
            // Java: srcBuf = new char[Runtime.getRuntime().freeMemory() >= 1048576 ? 8192 : 128];
            src_buf: vec![0u16; 128],
            src_pos: 0,
            src_count: 0,
            line: 0,
            column: 0,
            txt_buf: vec![0u16; 128],
            txt_pos: 0,
            type_: 0,
            is_whitespace: true,
            namespace: None,
            prefix: None,
            name: None,
            degenerated: false,
            attribute_count: 0,
            attributes: vec![None; 16],
            error: None,
            peek: [0; 2],
            peek_count: 0,
            was_cr: false,
            unresolved: false,
            token: false,
        }
    }

    // Java: private final boolean isProp(String n1, boolean prop, String n2)
    fn is_prop(&self, n1: String, prop: bool, n2: String) -> bool {
        if !n1.starts_with("http://xmlpull.org/v1/doc/")
        { return false; }
        if prop {
            return n1[42..].eq(&n2);
        }
        else {
            return n1[40..].eq(&n2);
        }
    }

    fn adjust_nsp(&mut self) -> Result<bool, XmlPullError> {

        let mut any = false;

        let mut i = 0;
        while i < self.attribute_count << 2 {
            // * 4 - 4; i >= 0; i -= 4) {

            let mut attr_name = self.attributes[(i + 2) as usize].clone().unwrap_or(String::new());
            let cut = attr_name.find(':').map(|v| v as i32).unwrap_or(-1);
            let mut prefix: Option<String>;

            if cut != -1 {
                prefix = Some(attr_name[..cut as usize].to_string());
                attr_name = attr_name[cut as usize + 1..].to_string();
            }
            else if attr_name.eq("xmlns") {
                prefix = Some(attr_name.clone());
                attr_name = String::new();
            }
            else {
                i += 4;
                continue;
            }

            if !prefix.as_ref().unwrap().eq("xmlns") {
                any = true;
            }
            else {
                let j = (self.nsp_counts[self.depth as usize] << 1) as usize;
                self.nsp_counts[self.depth as usize] += 1;

                self.nsp_stack = KXmlParser::ensure_capacity(self.nsp_stack.clone(), j + 2);
                self.nsp_stack[j] = if attr_name.is_empty() { None } else { Some(attr_name.clone()) };
                self.nsp_stack[j + 1] = self.attributes[(i + 3) as usize].clone();

                if !attr_name.is_empty() && self.attributes[(i + 3) as usize].as_deref() == Some("")
                { self.error("illegal empty namespace".to_string())?; }

                //  prefixMap = new PrefixMap (prefixMap, attrName, attr.getValue ());

                //System.out.println (prefixMap);

                // Java: System.arraycopy(
                //     attributes,
                //     i + 4,
                //     attributes,
                //     i,
                //     ((--attributeCount) << 2) - i);
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

                if cut == 0 && !self.relaxed
                // Java: throw new RuntimeException(
                //     "illegal attribute name: " + attrName + " at " + this);
                { return Err(XmlPullError::RuntimeException(format!("illegal attribute name: {} at {}", attr_name, ""))); }

                else if cut != -1 {
                    let attr_prefix = attr_name[..cut as usize].to_string();

                    attr_name = attr_name[cut as usize + 1..].to_string();

                    let attr_ns = self.get_namespace(Some(attr_prefix.clone()));

                    if attr_ns.is_none() && !self.relaxed
                    // Java: throw new RuntimeException(
                    //     "Undefined Prefix: " + attrPrefix + " in " + this);
                    { return Err(XmlPullError::RuntimeException(format!("Undefined Prefix: {} in {}", attr_prefix, ""))); }

                    self.attributes[i as usize] = attr_ns;
                    self.attributes[(i + 1) as usize] = Some(attr_prefix);
                    self.attributes[(i + 2) as usize] = Some(attr_name);

                    /*
                                        if (!relaxed) {
                                            for (int j = (attributeCount << 2) - 4; j > i; j -= 4)
                                                if (attrName.equals(attributes[j + 2])
                                                    && attrNs.equals(attributes[j]))
                                                    exception(
                                                        "Duplicate Attribute: {"
                                                            + attrNs
                                                            + "}"
                                                            + attrName);
                                        }
                        */
                }
                i -= 4;
            }
        }

        let cut = self.name.as_ref().unwrap().find(':').map(|v| v as i32).unwrap_or(-1);

        if cut == 0
        { self.error(format!("illegal tag name: {}", self.name.as_deref().unwrap_or("null")))?; }

        if cut != -1 {
            let name = self.name.clone().unwrap();
            self.prefix = Some(name[..cut as usize].to_string());
            self.name = Some(name[cut as usize + 1..].to_string());
        }

        self.namespace = self.get_namespace(self.prefix.clone());

        if self.namespace.is_none() {
            if self.prefix.is_some()
            { self.error(format!("undefined prefix: {}", self.prefix.as_deref().unwrap_or("null")))?; }
            self.namespace = Some(KXmlParser::NO_NAMESPACE.to_string());
        }

        return Ok(any);
    }

    // Java: private final String[] ensureCapacity(String[] arr, int required)
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

    // Java: private final void error(String desc) throws XmlPullParserException
    fn error(&mut self, desc: String) -> Result<(), XmlPullError> {
        if self.relaxed {
            if self.error.is_none() {
                self.error = Some(format!("ERR: {}", desc));
            }
        }
        else
        { self.exception(desc)?; }
        Ok(())
    }

    // Java: private final void exception(String desc) throws XmlPullParserException
    fn exception(&self, desc: String) -> Result<(), XmlPullError> {
        // Java: throw new XmlPullParserException(
        //     desc.length() < 100 ? desc : desc.substring(0, 100) + "\n",
        //     this,
        //     null);
        let d = if desc.chars().count() < 100 {
            desc
        }
        else {
            desc.chars().take(100).collect::<String>() + "\n"
        };
        return Err(XmlPullError::XmlPullParserException(d));
    }

    /**
     * common base for next and nextToken. Clears the state, except from
     * txtPos and whitespace. Does not set the type variable */

    fn next_impl(&mut self) -> Result<(), XmlPullError> {

        if self.reader.is_none()
        { self.exception("No Input specified".to_string())?; }

        if self.type_ == KXmlParser::END_TAG {
            self.depth -= 1;
        }

        loop {
            self.attribute_count = -1;

            // degenerated needs to be handled before error because of possible
            // processor expectations(!)

            if self.degenerated {
                self.degenerated = false;
                self.type_ = KXmlParser::END_TAG;
                return Ok(());
            }


            if self.error.is_some() {
                for c in self.error.as_ref().unwrap().chars() {
                    self.push(c as i32);
                }
                //            text = error;
                self.error = None;
                self.type_ = KXmlParser::COMMENT;
                return Ok(());
            }


//            if (relaxed
//                && (stackMismatch > 0 || (peek(0) == -1 && depth > 0))) {
//                int sp = (depth - 1) << 2;
//                type = END_TAG;
//                namespace = elementStack[sp];
//                prefix = elementStack[sp + 1];
//                name = elementStack[sp + 2];
//                if (stackMismatch != 1)
//                    error = "missing end tag /" + name + " inserted";
//                if (stackMismatch > 0)
//                    stackMismatch--;
//                return;
//            }

            self.prefix = None;
            self.name = None;
            self.namespace = None;
            //            text = null;

            self.type_ = self.peek_type()?;

            match self.type_ {

                KXmlParser::ENTITY_REF => {
                    self.push_entity()?;
                    return Ok(());
                }

                KXmlParser::START_TAG => {
                    self.parse_start_tag(false)?;
                    return Ok(());
                }

                KXmlParser::END_TAG => {
                    self.parse_end_tag()?;
                    return Ok(());
                }

                KXmlParser::END_DOCUMENT => {
                    return Ok(());
                }

                KXmlParser::TEXT => {
                    self.push_text('<' as i32, !self.token)?;
                    if self.depth == 0 {
                        if self.is_whitespace {
                            self.type_ = KXmlParser::IGNORABLE_WHITESPACE;
                        }
                        // make exception switchable for instances.chg... !!!!
                        //    else
                        //    exception ("text '"+getText ()+"' not allowed outside root element");
                    }
                    return Ok(());
                }

                _ => {
                    self.type_ = self.parse_legacy(self.token)?;
                    if self.type_ != KXmlParser::XML_DECL {
                        return Ok(());
                    }
                }
            }
        }
    }

    fn parse_legacy(&mut self, mut push: bool)
        -> Result<i32, XmlPullError> {

        let mut req = String::new();
        let mut term: i32 = 0;
        let mut result: i32 = 0;
        let mut prev = 0;

        self.read()?; // <
        let mut c = self.read()?;

        if c == '?' as i32 {
            if (self.peek(0)? == 'x' as i32 || self.peek(0)? == 'X' as i32)
                && (self.peek(1)? == 'm' as i32 || self.peek(1)? == 'M' as i32) {

                if push {
                    self.push(self.peek(0)?);
                    self.push(self.peek(1)?);
                }
                self.read()?;
                self.read()?;

                if (self.peek(0)? == 'l' as i32 || self.peek(0)? == 'L' as i32) && self.peek(1)? <= ' ' as i32 {

                    if self.line != 1 || self.column > 4
                    { self.error("PI must not start with xml".to_string())?; }

                    self.parse_start_tag(true)?;

                    if self.attribute_count < 1 || !"version".eq(self.attributes[2].as_deref().unwrap_or(""))
                    { self.error("version expected".to_string())?; }

                    self.version = self.attributes[3].clone();

                    let mut pos = 1;

                    if pos < self.attribute_count
                        && "encoding".eq(self.attributes[6].as_deref().unwrap_or("")) {
                        self.encoding = self.attributes[7].clone();
                        pos += 1;
                    }

                    if pos < self.attribute_count
                        && "standalone".eq(self.attributes[(4 * pos + 2) as usize].as_deref().unwrap_or("")) {
                        let st = self.attributes[(3 + 4 * pos) as usize].clone().unwrap_or(String::new());
                        if "yes".eq(&st) {
                            self.standalone = Some(true);
                        }
                        else if "no".eq(&st) {
                            self.standalone = Some(false);
                        }
                        else
                        { self.error(format!("illegal standalone value: {}", st))?; }
                        pos += 1;
                    }

                    if pos != self.attribute_count
                    { self.error("illegal xmldecl".to_string())?; }

                    self.is_whitespace = true;
                    self.txt_pos = 0;

                    return Ok(KXmlParser::XML_DECL);
                }
            }

            /*            int c0 = read ();
                        int c1 = read ();
                        int */

            term = '?' as i32;
            result = KXmlParser::PROCESSING_INSTRUCTION;
        }
        else if c == '!' as i32 {
            if self.peek(0)? == '-' as i32 {
                result = KXmlParser::COMMENT;
                req = "--".to_string();
                term = '-' as i32;
            }
            else if self.peek(0)? == '[' as i32 {
                result = KXmlParser::CDSECT;
                req = "[CDATA[".to_string();
                term = ']' as i32;
                push = true;
            }
            else {
                result = KXmlParser::DOCDECL;
                req = "DOCTYPE".to_string();
                term = -1;
            }
        }
        else {
            self.error(format!("illegal: <{}", c))?;
            return Ok(KXmlParser::COMMENT);
        }

        for ch in req.chars() {
            self.read_char(ch as i32)?;
        }

        if result == KXmlParser::DOCDECL {
            self.parse_doctype(push)?;
        }
        else {
            loop {
                c = self.read()?;
                if c == -1 {
                    self.error(KXmlParser::UNEXPECTED_EOF.to_string())?;
                    return Ok(KXmlParser::COMMENT);
                }

                if push {
                    self.push(c);
                }

                if (term == '?' as i32 || c == term)
                    && self.peek(0)? == term
                    && self.peek(1)? == '>' as i32
                { break; }

                prev = c;
            }

            if term == '-' as i32 && prev == '-' as i32 && !self.relaxed
            { self.error("illegal comment delimiter: --->".to_string())?; }

            self.read()?;
            self.read()?;

            if push && term != '?' as i32 {
                self.txt_pos -= 1;
            }

        }
        return Ok(result);
    }

    /** precondition: &lt! consumed */

    fn parse_doctype(&mut self, push: bool)
        -> Result<(), XmlPullError> {

        let mut nesting = 1;
        let mut quoted = false;

        // read();

        loop {
            let i = self.read()?;
            match i {

                -1 => {
                    self.error(KXmlParser::UNEXPECTED_EOF.to_string())?;
                    return Ok(());
                }

                '\'' as i32 => {
                    quoted = !quoted;
                }

                '<' as i32 => {
                    if !quoted {
                        nesting += 1;
                    }
                }

                '>' as i32 => {
                    if !quoted {
                        nesting -= 1;
                        if nesting == 0 {
                            return Ok(());
                        }
                    }
                }

                _ => {}
            }
            if push {
                self.push(i);
            }
        }
    }

    /* precondition: &lt;/ consumed */

    fn parse_end_tag(&mut self)
        -> Result<(), XmlPullError> {

        self.read()?; // '<'
        self.read()?; // '/'
        self.name = Some(self.read_name()?);
        self.skip()?;
        self.read_char('>' as i32)?;

        let sp = ((self.depth - 1) << 2) as usize;

        if self.depth == 0 {
            self.error("element stack empty".to_string())?;
            self.type_ = KXmlParser::COMMENT;
            return Ok(());
        }

        if !self.relaxed {
            if !self.name.as_deref().unwrap_or("").eq(self.element_stack[sp + 3].as_deref().unwrap_or("")) {
                self.error(format!("expected: /{} read: {}", self.element_stack[sp + 3].as_deref().unwrap_or(""), self.name.as_deref().unwrap_or("")))?;

                // become case insensitive in relaxed mode

    //            int probe = sp;
    //            while (probe >= 0 && !name.toLowerCase().equals(elementStack[probe + 3].toLowerCase())) {
    //                stackMismatch++;
    //                probe -= 4;
    //            }
    //
    //            if (probe < 0) {
    //                stackMismatch = 0;
    //                //            text = "unexpected end tag ignored";
    //                type = COMMENT;
    //                return;
    //            }
            }

            self.namespace = self.element_stack[sp].clone();
            self.prefix = self.element_stack[sp + 1].clone();
            self.name = self.element_stack[sp + 2].clone();
        }

        Ok(())
    }

    fn peek_type(&mut self) -> Result<i32, XmlPullError> {
        match self.peek(0)? {
            -1 => {
                return Ok(KXmlParser::END_DOCUMENT);
            }
            '&' as i32 => {
                return Ok(KXmlParser::ENTITY_REF);
            }
            '<' as i32 => {
                match self.peek(1)? {
                    '/' as i32 => {
                        return Ok(KXmlParser::END_TAG);
                    }
                    '?' as i32 | '!' as i32 => {
                        return Ok(KXmlParser::LEGACY);
                    }
                    _ => {
                        return Ok(KXmlParser::START_TAG);
                    }
                }
            }
            _ => {
                return Ok(KXmlParser::TEXT);
            }
        }
    }

    fn get(&self, pos: i32) -> String {
        // Java: new String(txtBuf, pos, txtPos - pos)
        return String::from_utf16_lossy(&self.txt_buf[pos as usize..self.txt_pos as usize]);
    }

    /*
    fn pop(&mut self, pos: i32) -> String {
        let result = String::from_utf16_lossy(&self.txt_buf[pos as usize..self.txt_pos as usize]);
        self.txt_pos = pos;
        return result;
    }
    */

    fn push(&mut self, c: i32) {

        self.is_whitespace &= c <= ' ' as i32;

        if self.txt_pos as usize + 1 >= self.txt_buf.len() { // +1 to have enough space for 2 surrogates, if needed
            // Java: char[] bigger = new char[txtPos * 4 / 3 + 4];
            let mut bigger = vec![0u16; (self.txt_pos * 4 / 3 + 4) as usize];
            for i in 0..self.txt_pos as usize {
                bigger[i] = self.txt_buf[i];
            }
            self.txt_buf = bigger;
        }

        if c > 0xffff {
            // write high Unicode value as surrogate pair
            let offset = c - 0x010000;
            self.txt_buf[self.txt_pos as usize] = ((offset >> 10) + 0xd800) as u16; // high surrogate
            self.txt_pos += 1;
            self.txt_buf[self.txt_pos as usize] = ((offset & 0x3ff) + 0xdc00) as u16; // low surrogate
            self.txt_pos += 1;
        }
        else {
            self.txt_buf[self.txt_pos as usize] = c as u16;
            self.txt_pos += 1;
        }
    }

    /** Sets name and attributes */

    fn parse_start_tag(&mut self, xmldecl: bool)
        -> Result<(), XmlPullError> {

        if !xmldecl {
            self.read()?;
        }
        self.name = Some(self.read_name()?);
        self.attribute_count = 0;

        loop {
            self.skip()?;

            let c = self.peek(0)?;

            if xmldecl {
                if c == '?' as i32 {
                    self.read()?;
                    self.read_char('>' as i32)?;
                    return Ok(());
                }
            }
            else {
                if c == '/' as i32 {
                    self.degenerated = true;
                    self.read()?;
                    self.skip()?;
                    self.read_char('>' as i32)?;
                    break;
                }

                if c == '>' as i32 && !xmldecl {
                    self.read()?;
                    break;
                }
            }

            if c == -1 {
                self.error(KXmlParser::UNEXPECTED_EOF.to_string())?;
                //type = COMMENT;
                return Ok(());
            }

            let attr_name = self.read_name()?;

            if attr_name.chars().count() == 0 {
                self.error("attr name expected".to_string())?;
               //type = COMMENT;
                break;
            }

            let mut i = (self.attribute_count << 2) as usize;
            self.attribute_count += 1;

            self.attributes = KXmlParser::ensure_capacity(self.attributes.clone(), i + 4);

            self.attributes[i] = Some(String::new());
            i += 1;
            self.attributes[i] = None;
            i += 1;
            self.attributes[i] = Some(attr_name.clone());
            i += 1;

            self.skip()?;

            if self.peek(0)? != '=' as i32 {
                if !self.relaxed {
                    self.error(format!("Attr.value missing f. {}", attr_name))?;
                }
                self.attributes[i] = Some(attr_name);
            }
            else {
                self.read_char('=' as i32)?;
                self.skip()?;
                let mut delimiter = self.peek(0)?;

                if delimiter != '\'' as i32 && delimiter != '"' as i32 {
                    if !self.relaxed {
                        self.error("attr value delimiter missing!".to_string())?;
                    }
                    delimiter = ' ' as i32;
                }
                else {
                    self.read()?;
                }

                let p = self.txt_pos;
                self.push_text(delimiter, true)?;

                self.attributes[i] = Some(self.get(p));
                self.txt_pos = p;

                if delimiter != ' ' as i32 {
                    self.read()?; // skip endquote
                }
            }
        }

        let sp = (self.depth << 2) as usize;
        self.depth += 1;

        self.element_stack = KXmlParser::ensure_capacity(self.element_stack.clone(), sp + 4);
        self.element_stack[sp + 3] = self.name.clone();

        if self.depth as usize >= self.nsp_counts.len() {
            let mut bigger = vec![0; self.depth as usize + 4];
            for i in 0..self.nsp_counts.len() {
                bigger[i] = self.nsp_counts[i];
            }
            self.nsp_counts = bigger;
        }

        self.nsp_counts[self.depth as usize] = self.nsp_counts[(self.depth - 1) as usize];

        /*
                if(!relaxed){
                for (int i = attributeCount - 1; i > 0; i--) {
                    for (int j = 0; j < i; j++) {
                        if (getAttributeName(i).equals(getAttributeName(j)))
                            exception("Duplicate Attribute: " + getAttributeName(i));
                    }
                }
                }
        */
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

    /**
     * result: isWhitespace; if the setName parameter is set,
     * the name of the entity is stored in "name" */

    fn push_entity(&mut self)
        -> Result<(), XmlPullError> {

        self.push(self.read()?); // &


        let pos = self.txt_pos;

        loop {
            let c = self.peek(0)?;
            if c == ';' as i32 {
                self.read()?;
                break;
            }
            if c < 128
                && (c < '0' as i32 || c > '9' as i32)
                && (c < 'a' as i32 || c > 'z' as i32)
                && (c < 'A' as i32 || c > 'Z' as i32)
                && c != '_' as i32
                && c != '-' as i32
                && c != '#' as i32 {
                if !self.relaxed {
                    self.error("unterminated entity ref".to_string())?;
                }

                // Java: System.out.println("broken entitiy: "+get(pos-1));
                println!("broken entitiy: {}", self.get(pos - 1));

                //; ends with:"+(char)c);
    //                if (c != -1)
    //                    push(c);
                return Ok(());
            }

            self.push(self.read()?);
        }

        let code = self.get(pos);
        self.txt_pos = pos - 1;
        if self.token && self.type_ == KXmlParser::ENTITY_REF {
            self.name = Some(code.clone());
        }

        if code.chars().next().unwrap() == '#' {
            // Java: int c =
            //     (code.charAt(1) == 'x'
            //         ? Integer.parseInt(code.substring(2), 16)
            //         : Integer.parseInt(code.substring(1)));
            let c = if code.chars().nth(1).unwrap() == 'x' {
                i32::from_str_radix(&code[2..], 16).unwrap()
            }
            else {
                code[1..].parse::<i32>().unwrap()
            };
            self.push(c);
            return Ok(());
        }

        let result = self.entity_map.get(&code).cloned();

        self.unresolved = result.is_none();

        if self.unresolved {
            if !self.token {
                self.error(format!("unresolved: &{};", code))?;
            }
        }
        else {
            for ch in result.unwrap().chars() {
                self.push(ch as i32);
            }
        }

        Ok(())
    }

    /** types:
    '<': parse to any token (for nextToken ())
    '"': parse to quote
    ' ': parse to whitespace or '>'
    */

    fn push_text(&mut self, delimiter: i32, resolve_entities: bool)
        -> Result<(), XmlPullError> {

        let mut next = self.peek(0)?;
        let mut cbr_count = 0;

        while next != -1 && next != delimiter { // covers eof, '<', '"'

            if delimiter == ' ' as i32 {
                if next <= ' ' as i32 || next == '>' as i32 {
                    break;
                }
            }

            if next == '&' as i32 {
                if !resolve_entities {
                    break;
                }

                self.push_entity()?;
            }
            else if next == '\n' as i32 && self.type_ == KXmlParser::START_TAG {
                self.read()?;
                self.push(' ' as i32);
            }
            else {
                self.push(self.read()?);
            }

            if next == '>' as i32 && cbr_count >= 2 && delimiter != ']' as i32
            { self.error("Illegal: ]]>".to_string())?; }

            if next == ']' as i32 {
                cbr_count += 1;
            }
            else {
                cbr_count = 0;
            }

            next = self.peek(0)?;
        }

        Ok(())
    }

    // Java: private final void read(char c) throws IOException, XmlPullParserException
    fn read_char(&mut self, c: i32)
        -> Result<(), XmlPullError> {
        let a = self.read()?;
        if a != c
        { self.error(format!("expected: '{}' actual: '{}'", c as u16 as char, a as u16 as char))?; }
        Ok(())
    }

    // Java: private final int read() throws IOException
    fn read(&mut self) -> Result<i32, XmlPullError> {
        let result;

        if self.peek_count == 0 {
            result = self.peek(0)?;
        }
        else {
            result = self.peek[0];
            self.peek[0] = self.peek[1];
        }
        //            else {
        //                result = peek[0];
        //                System.arraycopy (peek, 1, peek, 0, peekCount-1);
        //            }
        self.peek_count -= 1;

        self.column += 1;

        if result == '\n' as i32 {

            self.line += 1;
            self.column = 1;
        }

        return Ok(result);
    }

    /** Does never read more than needed */

    fn peek(&mut self, pos: i32) -> Result<i32, XmlPullError> {

        while pos >= self.peek_count {

            let nw;

            if self.src_buf.len() <= 1 {
                nw = self.reader.as_mut().unwrap().read();
            }
            else if self.src_pos < self.src_count {
                nw = self.src_buf[self.src_pos as usize] as i32;
                self.src_pos += 1;
            }
            else {
                self.src_count = self.reader.as_mut().unwrap().read_buf(&mut self.src_buf, 0, self.src_buf.len());
                if self.src_count <= 0 {
                    nw = -1;
                }
                else {
                    nw = self.src_buf[0] as i32;
                }

                self.src_pos = 1;
            }

            if nw == '\r' as i32 {
                self.was_cr = true;
                self.peek[self.peek_count as usize] = '\n' as i32;
                self.peek_count += 1;
            }
            else {
                if nw == '\n' as i32 {
                    if !self.was_cr {
                        self.peek[self.peek_count as usize] = '\n' as i32;
                        self.peek_count += 1;
                    }
                }
                else {
                    self.peek[self.peek_count as usize] = nw;
                    self.peek_count += 1;
                }

                self.was_cr = false;
            }
        }

        return Ok(self.peek[pos as usize]);
    }

    fn read_name(&mut self)
        -> Result<String, XmlPullError> {

        let pos = self.txt_pos;
        let mut c = self.peek(0)?;
        if (c < 'a' as i32 || c > 'z' as i32)
            && (c < 'A' as i32 || c > 'Z' as i32)
            && c != '_' as i32
            && c != ':' as i32
            && c < 0x0c0
            && !self.relaxed
        { self.error("name expected".to_string())?; }

        // Java: do {
        //     push(read());
        //     c = peek(0);
        // }
        // while (...);
        loop {
            self.push(self.read()?);
            c = self.peek(0)?;
            if !((c >= 'a' as i32 && c <= 'z' as i32)
                || (c >= 'A' as i32 && c <= 'Z' as i32)
                || (c >= '0' as i32 && c <= '9' as i32)
                || c == '_' as i32
                || c == '-' as i32
                || c == ':' as i32
                || c == '.' as i32
                || c >= 0x0b7)
            { break; }
        }

        let result = self.get(pos);
        self.txt_pos = pos;
        return Ok(result);
    }

    fn skip(&mut self) -> Result<(), XmlPullError> {

        loop {
            let c = self.peek(0)?;
            if c > ' ' as i32 || c == -1 {
                break;
            }
            self.read()?;
        }

        Ok(())
    }

    //  public part starts here...

    pub fn set_input(&mut self, reader: Option<Box<dyn Reader>>) -> Result<(), XmlPullError> {
        self.reader = reader;

        self.line = 1;
        self.column = 0;
        self.type_ = KXmlParser::START_DOCUMENT;
        self.name = None;
        self.namespace = None;
        self.degenerated = false;
        self.attribute_count = -1;
        self.encoding = None;
        self.version = None;
        self.standalone = None;

        if self.reader.is_none() {
            return Ok(());
        }

        self.src_pos = 0;
        self.src_count = 0;
        self.peek_count = 0;
        self.depth = 0;

        // Java: entityMap = new Hashtable();
        self.entity_map = HashMap::new();
        self.entity_map.insert("amp".to_string(), "&".to_string());
        self.entity_map.insert("apos".to_string(), "'".to_string());
        self.entity_map.insert("gt".to_string(), ">".to_string());
        self.entity_map.insert("lt".to_string(), "<".to_string());
        self.entity_map.insert("quot".to_string(), "\"".to_string());

        Ok(())
    }

    pub fn set_input_stream(&mut self, is: Option<Box<dyn InputStream>>, _enc: Option<String>)
        -> Result<(), XmlPullError> {

        self.src_pos = 0;
        self.src_count = 0;
        let mut enc = _enc.clone();

        if is.is_none()
        // Java: throw new IllegalArgumentException();
        { return Err(XmlPullError::RuntimeException("IllegalArgumentException".to_string())); }

        let result = (|| -> Result<(), XmlPullError> {

            if enc.is_none() {
                // read four bytes

                // Java: int chk = 0;
                let mut chk = 0;

                // Java: while (srcCount < 4) {
                //     int i = is.read();
                //     if (i == -1) break;
                //     chk = (chk << 8) | i;
                //     srcBuf[srcCount++] = (char) i;
                // }
                while self.src_count < 4 {
                    let i = is.as_ref().unwrap().read();
                    if i == -1 { break; }
                    chk = (chk << 8) | i;
                    self.src_buf[self.src_count as usize] = i as u16;
                    self.src_count += 1;
                }

                if self.src_count == 4 {
                    match chk {
                        UTF32BE_BOM => {
                            enc = Some("UTF-32BE".to_string());
                            self.src_count = 0;
                        }

                        UTF32LE_BOM => {
                            enc = Some("UTF-32LE".to_string());
                            self.src_count = 0;
                        }

                        UTF32BE_3C => {
                            enc = Some("UTF-32BE".to_string());
                            self.src_buf[0] = '<' as u16;
                            self.src_count = 1;
                        }

                        UTF32LE_3C => {
                            enc = Some("UTF-32LE".to_string());
                            self.src_buf[0] = '<' as u16;
                            self.src_count = 1;
                        }

                        UTF16BE_3C => {
                            enc = Some("UTF-16BE".to_string());
                            self.src_buf[0] = '<' as u16;
                            self.src_buf[1] = '?' as u16;
                            self.src_count = 2;
                        }

                        UTF16LE_3C => {
                            enc = Some("UTF-16LE".to_string());
                            self.src_buf[0] = '<' as u16;
                            self.src_buf[1] = '?' as u16;
                            self.src_count = 2;
                        }

                        UTF8_3C => {
                            // Java: while (true) { ... }
                            loop {
                                let i = is.as_ref().unwrap().read();
                                if i == -1 { break; }
                                self.src_buf[self.src_count as usize] = i as u16;
                                self.src_count += 1;
                                if i == '>' as i32 {
                                    // Java: String s = new String(srcBuf, 0, srcCount);
                                    let s = String::from_utf16_lossy(&self.src_buf[0..self.src_count as usize]);
                                    let mut i0 = s.find("encoding").map(|v| v as i32).unwrap_or(-1);
                                    if i0 != -1 {
                                        // Java: while (s.charAt(i0) != '"' && s.charAt(i0) != '\'') i0++;
                                        while s.as_bytes()[i0 as usize] != '"' as u8
                                            && s.as_bytes()[i0 as usize] != '\'' as u8 {
                                            i0 += 1;
                                        }
                                        // Java: char deli = s.charAt(i0++);
                                        let deli = s.as_bytes()[i0 as usize] as char;
                                        i0 += 1;
                                        // Java: int i1 = s.indexOf(deli, i0);
                                        let i1 = s[i0 as usize..].find(deli).map(|v| v as i32 + i0).unwrap_or(-1);
                                        enc = Some(s[i0 as usize..i1 as usize].to_string());
                                    }
                                    break;
                                }
                            }

                            // Java: falls through to default (switch fallthrough)
                            if (chk & 0x0ffff0000u32 as i32) == 0x0FEFF0000u32 as i32 {
                                enc = Some("UTF-16BE".to_string());
                                self.src_buf[0] = ((self.src_buf[2] as i32 << 8) | self.src_buf[3] as i32) as u16;
                                self.src_count = 1;
                            }
                            else if (chk & 0x0ffff0000u32 as i32) == 0x0fffe0000u32 as i32 {
                                enc = Some("UTF-16LE".to_string());
                                self.src_buf[0] = ((self.src_buf[3] as i32 << 8) | self.src_buf[2] as i32) as u16;
                                self.src_count = 1;
                            }
                            else if (chk & 0x0ffffff00u32 as i32) == 0x0EFBBBF00u32 as i32 {
                                enc = Some("UTF-8".to_string());
                                self.src_buf[0] = self.src_buf[3];
                                self.src_count = 1;
                            }
                        }

                        _ => {
                            // Java: default: (of the switch above)
                            if (chk & 0x0ffff0000u32 as i32) == 0x0FEFF0000u32 as i32 {
                                enc = Some("UTF-16BE".to_string());
                                self.src_buf[0] = ((self.src_buf[2] as i32 << 8) | self.src_buf[3] as i32) as u16;
                                self.src_count = 1;
                            }
                            else if (chk & 0x0ffff0000u32 as i32) == 0x0fffe0000u32 as i32 {
                                enc = Some("UTF-16LE".to_string());
                                self.src_buf[0] = ((self.src_buf[3] as i32 << 8) | self.src_buf[2] as i32) as u16;
                                self.src_count = 1;
                            }
                            else if (chk & 0x0ffffff00u32 as i32) == 0x0EFBBBF00u32 as i32 {
                                enc = Some("UTF-8".to_string());
                                self.src_buf[0] = self.src_buf[3];
                                self.src_count = 1;
                            }
                        }
                    }
                }
            }

            if enc.is_none() {
                enc = Some("UTF-8".to_string());
            }

            let sc = self.src_count;
            // Java: setInput(new InputStreamReader(is, enc));
            self.set_input(Some(Box::new(InputStreamReader::new(is.unwrap(), enc.clone()))))?;
            self.encoding = _enc;
            self.src_count = sc;

            Ok(())
        })();

        return match result {
            Err(e) => {
                // Java: catch (Exception e) {
                //     throw new XmlPullParserException(
                //         "Invalid stream or encoding: " + e.toString(),
                //         this,
                //         e);
                Err(XmlPullError::XmlPullParserException(format!("Invalid stream or encoding: {:?}", e)))
            }
            Ok(()) => Ok(()),
        };
    }

    pub fn get_feature(&self, feature: String) -> bool {
        if XmlPullParserFeatures::FEATURE_PROCESS_NAMESPACES.eq(&feature)
        { return self.process_nsp; }
        else if self.is_prop(feature, false, "relaxed".to_string())
        { return self.relaxed; }
        else
        { return false; }
    }

    pub fn get_input_encoding(&self) -> Option<String> {
        return self.encoding.clone();
    }

    pub fn define_entity_replacement_text(&mut self, entity: String, value: String)
        -> Result<(), XmlPullError> {
        if self.entity_map.is_empty()
        // Java: throw new RuntimeException("entity replacement text must be defined after setInput!");
        { return Err(XmlPullError::RuntimeException("entity replacement text must be defined after setInput!".to_string())); }
        self.entity_map.insert(entity, value);
        Ok(())
    }

    pub fn get_property(&self, property: String) -> Option<Property> {
        if self.is_prop(property.clone(), true, "xmldecl-version".to_string())
        { return self.version.clone().map(|v| Property::Str(v)); }
        if self.is_prop(property.clone(), true, "xmldecl-standalone".to_string())
        { return self.standalone.map(|v| Property::Bool(v)); }
        if self.is_prop(property, true, "location".to_string())
        {
            // Java: return location != null ? location : reader.toString();
            return Some(Property::Location(
                if self.location.is_some() {
                    self.location.clone().unwrap()
                }
                else {
                    self.reader.as_ref().unwrap().to_string()
                }));
        }
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

        // Java: for (int i = (getNamespaceCount(depth) << 1) - 2; i >= 0; i -= 2) {
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

        // Java: StringBuffer buf =
        //     new StringBuffer(type < TYPES.length ? TYPES[type] : "unknown");
        let mut buf = String::from(if self.type_ < KXmlParser::TYPES.len() as i32 { KXmlParser::TYPES[self.type_ as usize] } else { "unknown" });
        buf.push(' ');

        if self.type_ == KXmlParser::START_TAG || self.type_ == KXmlParser::END_TAG {
            if self.degenerated {
                buf.push_str("(empty) ");
            }
            buf.push('<');
            if self.type_ == KXmlParser::END_TAG {
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
        else if self.type_ == KXmlParser::IGNORABLE_WHITESPACE {
            // Java: ; (empty statement)
        }
        else if self.type_ != KXmlParser::TEXT {
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

        buf.push_str(&format!("@{}:{}", self.line, self.column));
        if self.location.is_some() {
            buf.push_str(" in ");
            buf.push_str(self.location.as_deref().unwrap_or("null"));
        }
        else if self.reader.is_some() {
            buf.push_str(" in ");
            buf.push_str(&self.reader.as_ref().unwrap().to_string());
        }
        return buf;
    }

    pub fn get_line_number(&self) -> i32 {
        return self.line;
    }

    pub fn get_column_number(&self) -> i32 {
        return self.column;
    }

    pub fn is_whitespace(&self) -> Result<bool, XmlPullError> {
        if self.type_ != KXmlParser::TEXT && self.type_ != KXmlParser::IGNORABLE_WHITESPACE && self.type_ != KXmlParser::CDSECT
        { self.exception(KXmlParser::ILLEGAL_TYPE.to_string())?; }
        return Ok(self.is_whitespace);
    }

    pub fn get_text(&self) -> Option<String> {
        // Java: return type < TEXT
        //     || (type == ENTITY_REF && unresolved) ? null : get(0);
        return if self.type_ < KXmlParser::TEXT
            || (self.type_ == KXmlParser::ENTITY_REF && self.unresolved) {
            None
        }
        else {
            Some(self.get(0))
        };
    }

    pub fn get_text_characters(&self, poslen: &mut [i32; 2]) -> Option<Vec<u16>> {
        if self.type_ >= KXmlParser::TEXT {
            if self.type_ == KXmlParser::ENTITY_REF {
                poslen[0] = 0;
                poslen[1] = self.name.as_ref().unwrap().chars().count() as i32;
                // Java: return name.toCharArray();
                return Some(self.name.as_ref().unwrap().chars().map(|c| c as u16).collect());
            }
            poslen[0] = 0;
            poslen[1] = self.txt_pos;
            // Java: return txtBuf;
            return Some(self.txt_buf.clone());
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

    pub fn is_empty_element_tag(&self) -> Result<bool, XmlPullError> {
        if self.type_ != KXmlParser::START_TAG
        { self.exception(KXmlParser::ILLEGAL_TYPE.to_string())?; }
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

    pub fn get_attribute_value_named(&self, namespace: Option<String>, name: String) -> Option<String> {

        // Java: for (int i = (attributeCount << 2) - 4; i >= 0; i -= 4) {
        let mut i = (self.attribute_count << 2) - 4;
        while i >= 0 {
            if self.attributes[(i + 2) as usize].as_ref().unwrap() == &name
                && (namespace.is_none() || self.attributes[i as usize].as_ref().unwrap() == namespace.as_ref().unwrap())
            { return self.attributes[(i + 3) as usize].clone(); }
            i -= 4;
        }

        return None;
    }

    pub fn get_event_type(&self) -> Result<i32, XmlPullError> {
        return Ok(self.type_);
    }

    pub fn next(&mut self) -> Result<i32, XmlPullError> {

        self.txt_pos = 0;
        self.is_whitespace = true;
        let mut min_type = 9999;
        self.token = false;

        // Java: do {
        //     nextImpl();
        //     if (type < minType)
        //         minType = type;
        //     //    if (curr <= TEXT) type = curr;
        // }
        // while (minType > ENTITY_REF // ignorable
        //     || (minType >= TEXT && peekType() >= TEXT));
        loop {
            self.next_impl()?;
            if self.type_ < min_type {
                min_type = self.type_;
            }
            //        if (curr <= TEXT) type = curr;
            if !(min_type > KXmlParser::ENTITY_REF // ignorable
                || (min_type >= KXmlParser::TEXT && self.peek_type()? >= KXmlParser::TEXT))
            { break; }
        }

        self.type_ = min_type;
        if self.type_ > KXmlParser::TEXT {
            self.type_ = KXmlParser::TEXT;
        }

        return Ok(self.type_);
    }

    pub fn next_token(&mut self) -> Result<i32, XmlPullError> {

        self.is_whitespace = true;
        self.txt_pos = 0;

        self.token = true;
        self.next_impl()?;
        return Ok(self.type_);
    }

    //
    // utility methods to make XML parsing easier ...

    pub fn next_tag(&mut self) -> Result<i32, XmlPullError> {

        self.next()?;
        if self.type_ == KXmlParser::TEXT && self.is_whitespace {
            self.next()?;
        }

        if self.type_ != KXmlParser::END_TAG && self.type_ != KXmlParser::START_TAG
        { self.exception("unexpected type".to_string())?; }

        return Ok(self.type_);
    }

    pub fn require(&mut self, type_: i32, namespace: Option<String>, name: Option<String>)
        -> Result<(), XmlPullError> {

        if type_ != self.type_
            || (namespace.is_some() && !namespace.as_ref().unwrap().eq(&self.get_namespace_current().unwrap_or(String::new())))
            || (name.is_some() && !name.as_ref().unwrap().eq(&self.get_name().unwrap_or(String::new())))
        {
            // Java: exception(
            //     "expected: " + TYPES[type] + " {" + namespace + "}" + name);
            self.exception(format!(
                "expected: {} {{{}}}{}",
                KXmlParser::TYPES[type_ as usize],
                namespace.as_deref().unwrap_or("null"),
                name.as_deref().unwrap_or("null")))?;
        }

        Ok(())
    }

    pub fn next_text(&mut self) -> Result<String, XmlPullError> {
        if self.type_ != KXmlParser::START_TAG
        { self.exception("precondition: START_TAG".to_string())?; }

        self.next()?;

        let result;

        if self.type_ == KXmlParser::TEXT {
            result = self.get_text().unwrap_or(String::new());
            self.next()?;
        }
        else {
            result = String::new();
        }

        if self.type_ != KXmlParser::END_TAG
        { self.exception("END_TAG expected".to_string())?; }

        return Ok(result);
    }

    pub fn set_feature(&mut self, feature: String, value: bool)
        -> Result<(), XmlPullError> {
        if XmlPullParserFeatures::FEATURE_PROCESS_NAMESPACES.eq(&feature) {
            self.process_nsp = value;
        }
        else if self.is_prop(feature.clone(), false, "relaxed".to_string()) {
            self.relaxed = value;
        }
        else
        { self.exception(format!("unsupported feature: {}", feature))?; }

        Ok(())
    }

    pub fn set_property(&mut self, property: String, value: Option<String>)
        -> Result<(), XmlPullError> {
        if self.is_prop(property.clone(), true, "location".to_string()) {
            self.location = value;
        }
        else
        // Java: throw new XmlPullParserException("unsupported property: " + property);
        { return Err(XmlPullError::XmlPullParserException(format!("unsupported property: {}", property))); }

        Ok(())
    }

    /**
      * Skip sub tree that is currently porser positioned on.
      * <br>NOTE: parser must be on START_TAG and when funtion returns
      * parser will be positioned on corresponding END_TAG.
      */

    //    Implementation copied from Alek's mail...

    pub fn skip_sub_tree(&mut self) -> Result<(), XmlPullError> {
        self.require(KXmlParser::START_TAG, None, None)?;
        let mut level = 1;
        while level > 0 {
            let event_type = self.next()?;
            if event_type == KXmlParser::END_TAG {
                level -= 1;
            }
            else if event_type == KXmlParser::START_TAG {
                level += 1;
            }
        }

        Ok(())
    }
}

/** java.lang.Object property value stand-in for getProperty / setProperty */
pub enum Property {
    Str(String),
    Bool(bool),
    Location(String),
}

// Java: the int literals of the encoding sniffing switch in setInput(InputStream, String)
// (some of these are > Integer.MAX_VALUE and wrap around in Java, hence the casts)
pub const UTF32BE_BOM: i32 = 0x00000FEFF;
pub const UTF32LE_BOM: i32 = 0x0FFFE0000u32 as i32;
pub const UTF32BE_3C: i32 = 0x03c;
pub const UTF32LE_3C: i32 = 0x03c000000;
pub const UTF16BE_3C: i32 = 0x0003c003f;
pub const UTF16LE_3C: i32 = 0x03c003f00;
pub const UTF8_3C: i32 = 0x03c3f786d;

/** org.xmlpull.v1.XmlPullParser constants used by KXmlParser */
pub struct XmlPullParserFeatures;

impl XmlPullParserFeatures {
    pub const FEATURE_PROCESS_NAMESPACES: &'static str = "http://xmlpull.org/v1/doc/features.html#process-namespaces";
}

/** java.io.InputStreamReader stand-in, wrapping an InputStream into a Reader */
pub struct InputStreamReader {
    pub is: Box<dyn InputStream>,
    pub encoding: Option<String>,
}

impl InputStreamReader {
    pub fn new(is: Box<dyn InputStream>, encoding: Option<String>) -> InputStreamReader {
        InputStreamReader { is, encoding }
    }
}

impl Reader for InputStreamReader {
    fn read(&mut self) -> i32 {
        return self.is.read();
    }

    fn read_buf(&mut self, buf: &mut Vec<u16>, off: usize, len: usize) -> i32 {
        let mut k = 0;
        while k < len {
            let i = self.is.read();
            if i == -1 { break; }
            buf[off + k] = i as u16;
            k += 1;
        }
        return k as i32;
    }

    fn to_string(&self) -> String {
        return format!("InputStreamReader({})", self.encoding.as_deref().unwrap_or(""));
    }
}
