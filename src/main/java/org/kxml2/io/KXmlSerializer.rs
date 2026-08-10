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


// package org.kxml2.io;

// import java.io.*;
// import org.xmlpull.v1.*;

/** Writer interface (java.io.Writer), as far as needed by the kxml2 io classes. */
pub trait Writer {
    fn write_char(&mut self, c: i32);
    fn write_str(&mut self, s: String);
    fn flush(&mut self);
}

/** OutputStream interface (java.io.OutputStream), as far as needed by the kxml2 io classes. */
pub trait OutputStream {
    fn write(&mut self, b: i32);
}

// public class KXmlSerializer implements XmlSerializer {

pub struct KXmlSerializer {

    //    static final String UNDEFINED = ":";

    pub writer: Option<Box<dyn Writer>>,

    pub pending: bool,
    pub auto: i32,
    pub depth: i32,

    pub element_stack: Vec<Option<String>>,
    //nsp/prefix/name
    pub nsp_counts: Vec<i32>,
    pub nsp_stack: Vec<Option<String>>,
    //prefix/nsp; both empty are ""
    pub indent: Vec<bool>,
    pub unicode: bool,
    pub encoding: Option<String>,
}

impl KXmlSerializer {

    pub fn new() -> KXmlSerializer {
        KXmlSerializer {
            writer: None,
            pending: false,
            auto: 0,
            depth: 0,
            element_stack: vec![None; 12],
            nsp_counts: vec![0; 4],
            nsp_stack: vec![None; 8],
            indent: vec![false; 4],
            unicode: false,
            encoding: None,
        }
    }

    fn check(&mut self, close: bool) -> Result<(), String> {
        if !self.pending {
            return Ok(());
        }

        self.depth += 1;
        self.pending = false;

        if self.indent.len() <= self.depth as usize {
            let mut hlp = vec![false; self.depth as usize + 4];
            for i in 0..(self.depth as usize) {
                hlp[i] = self.indent[i];
            }
            self.indent = hlp;
        }
        self.indent[self.depth as usize] = self.indent[self.depth as usize - 1];

        let mut i = self.nsp_counts[self.depth as usize - 1];
        while i < self.nsp_counts[self.depth as usize] {
            self.writer.as_mut().unwrap().write_char(' ' as i32);
            self.writer.as_mut().unwrap().write_str("xmlns".to_string());
            if !"".eq(self.nsp_stack[(i * 2) as usize].as_deref().unwrap_or("")) {
                self.writer.as_mut().unwrap().write_char(':' as i32);
                self.writer.as_mut().unwrap().write_str(self.nsp_stack[(i * 2) as usize].clone().unwrap());
            }
            else if "".eq(self.get_namespace().as_deref().unwrap_or("")) && !"".eq(self.nsp_stack[(i * 2 + 1) as usize].as_deref().unwrap_or("")) {
                // Java: throw new IllegalStateException("Cannot set default namespace for elements in no namespace");
                return Err("Cannot set default namespace for elements in no namespace".to_string());
            }
            self.writer.as_mut().unwrap().write_str("=\"".to_string());
            self.write_escaped(self.nsp_stack[(i * 2 + 1) as usize].clone().unwrap_or(String::new()), '"' as i32);
            self.writer.as_mut().unwrap().write_char('"' as i32);
            i += 1;
        }

        if self.nsp_counts.len() <= (self.depth + 1) as usize {
            let mut hlp = vec![0; self.depth as usize + 8];
            for i in 0..(self.depth as usize + 1) {
                hlp[i] = self.nsp_counts[i];
            }
            self.nsp_counts = hlp;
        }

        self.nsp_counts[(self.depth + 1) as usize] = self.nsp_counts[self.depth as usize];
        //   nspCounts[depth + 2] = nspCounts[depth];

        self.writer.as_mut().unwrap().write_str((if close { " />" } else { ">" }).to_string());

        Ok(())
    }

    fn write_escaped(&mut self, s: String, quot: i32) -> Result<(), String> {

        let chars: Vec<char> = s.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            match c {
                '\n' | '\r' | '\t' => {
                    if quot == -1 {
                        self.writer.as_mut().unwrap().write_char(c as i32);
                    }
                    else {
                        self.writer.as_mut().unwrap().write_str(format!("&#{};", c as i32));
                    }
                }
                '&' => {
                    self.writer.as_mut().unwrap().write_str("&amp;".to_string());
                }
                '>' => {
                    self.writer.as_mut().unwrap().write_str("&gt;".to_string());
                }
                '<' => {
                    self.writer.as_mut().unwrap().write_str("&lt;".to_string());
                }
                '"' | '\'' => {
                    if c as i32 == quot {
                        self.writer.as_mut().unwrap().write_str(
                            if c == '"' { "&quot;".to_string() } else { "&apos;".to_string() });
                        // Java: break;  (break out of the switch, continue the for loop)
                    }
                    else {
                        // Java: falls through to default
                        self.default_escape(chars.clone(), &mut i, c)?;
                    }
                }
                _ => {
                    self.default_escape(chars.clone(), &mut i, c)?;
                }
            }
            i += 1;
        }

        Ok(())
    }

    /** the default case of the Java switch in writeEscaped */
    fn default_escape(&mut self, chars: Vec<char>, i: &mut usize, c: char) -> Result<(), String> {
        //if(c < ' ')
        //    throw new IllegalArgumentException("Illegal control code:"+((int) c));

        if *i < chars.len() - 1 {
            let c_low = chars[*i + 1];
            // c is high surrogate and cLow is low surrogate
            if c as i32 >= 0xd800 && c as i32 <= 0xdbff && c_low as i32 >= 0xdc00 && c_low as i32 <= 0xdfff {
                // write surrogate pair as single code point
                let n = ((c as i32 - 0xd800) << 10) + (c_low as i32 - 0xdc00) + 0x010000;
                self.writer.as_mut().unwrap().write_str(format!("&#{};", n));
                *i += 1; // Skip the low surrogate
                return Ok(());
            }
            // Does nothing smart about orphan surrogates, just output them "as is"
        }
        if c as i32 >= ' ' as i32 && c != '@' && (c as i32 < 127 || self.unicode) {
            self.writer.as_mut().unwrap().write_char(c as i32);
        }
        else {
            self.writer.as_mut().unwrap().write_str(format!("&#{};", c as i32));
        }
        Ok(())
    }

    /*
        fn write_indent(&mut self) {
            self.writer.write_str("\r\n".to_string());
            for i in 0..self.depth {
                self.writer.write_char(' ' as i32);
            }
        }*/

    pub fn docdecl(&mut self, dd: String) -> Result<(), String> {
        self.writer.as_mut().unwrap().write_str("<!DOCTYPE".to_string());
        self.writer.as_mut().unwrap().write_str(dd);
        self.writer.as_mut().unwrap().write_str(">".to_string());
        Ok(())
    }

    pub fn end_document(&mut self) -> Result<(), String> {
        while self.depth > 0 {
            self.end_tag(
                self.element_stack[(self.depth * 3 - 3) as usize].clone(),
                self.element_stack[(self.depth * 3 - 1) as usize].clone())?;
        }
        self.flush()
    }

    pub fn entity_ref(&mut self, name: String) -> Result<(), String> {
        self.check(false)?;
        self.writer.as_mut().unwrap().write_char('&' as i32);
        self.writer.as_mut().unwrap().write_str(name);
        self.writer.as_mut().unwrap().write_char(';' as i32);
        Ok(())
    }

    pub fn get_feature(&self, name: String) -> bool {
        //return false;
        return (
            "http://xmlpull.org/v1/doc/features.html#indent-output"
                .eq(
                    &name))
            ? self.indent[self.depth as usize]
            : false;
    }

    pub fn get_prefix(&mut self, namespace: Option<String>, create: bool) -> Option<String> {
        // Java: try {
        //     return getPrefix(namespace, false, create);
        // }
        // catch (IOException e) {
        //     throw new RuntimeException(e.toString());
        // }
        return self.get_prefix_internal(namespace, false, create);
    }

    fn get_prefix_internal(
        &mut self,
        namespace: Option<String>,
        include_default: bool,
        create: bool)
        -> Option<String> {

        let mut i = self.nsp_counts[(self.depth + 1) as usize] * 2 - 2;
        while i >= 0 {
            if self.nsp_stack[(i + 1) as usize].as_deref() == namespace.as_deref()
                && (include_default || !"".eq(self.nsp_stack[i as usize].as_deref().unwrap_or(""))) {
                let mut cand = self.nsp_stack[i as usize].clone();
                let mut j = i + 2;
                while j < self.nsp_counts[(self.depth + 1) as usize] * 2 {
                    if self.nsp_stack[j as usize] == cand {
                        cand = None;
                        break;
                    }
                    j += 1;
                }
                if cand.is_some() {
                    return cand;
                }
            }
            i -= 2;
        }

        if !create {
            return None;
        }

        let mut prefix: Option<String> = None;

        if "".eq(namespace.as_deref().unwrap_or("")) {
            prefix = Some(String::new());
        }
        else {
            loop {
                prefix = Some(format!("n{}", self.auto));
                self.auto += 1;
                let mut i = self.nsp_counts[(self.depth + 1) as usize] * 2 - 2;
                while i >= 0 {
                    if prefix == self.nsp_stack[i as usize] {
                        prefix = None;
                        break;
                    }
                    i -= 2;
                }
                if prefix.is_some() {
                    break;
                }
            }
        }

        let p = self.pending;
        self.pending = false;
        self.set_prefix(prefix.clone(), namespace);
        self.pending = p;
        return prefix;
    }

    pub fn get_property(&self, name: String) -> i32 {
        // Java: throw new RuntimeException("Unsupported property");
        panic!("Unsupported property");
    }

    pub fn ignorable_whitespace(&mut self, s: String) -> Result<(), String> {
        self.text(s)
    }

    pub fn set_feature(&mut self, name: String, value: bool) {
        if "http://xmlpull.org/v1/doc/features.html#indent-output"
            .eq(&name) {
            self.indent[self.depth as usize] = value;
        }
        else
        // Java: throw new RuntimeException("Unsupported Feature");
        { panic!("Unsupported Feature"); }
    }

    pub fn set_property(&self, name: String, value: i32) {
        // Java: throw new RuntimeException(
        //     "Unsupported Property:" + value);
        panic!("Unsupported Property:{}", value);
    }

    pub fn set_prefix(&mut self, prefix: Option<String>, namespace: Option<String>) -> Result<(), String> {

        self.check(false)?;
        let mut prefix = prefix;
        let mut namespace = namespace;
        if prefix.is_none() {
            prefix = Some(String::new());
        }
        if namespace.is_none() {
            namespace = Some(String::new());
        }

        let defined = self.get_prefix_internal(namespace.clone(), true, false);

        // boil out if already defined

        if prefix == defined {
            return Ok(());
        }

        let pos = (self.nsp_counts[(self.depth + 1) as usize]) as usize;
        self.nsp_counts[(self.depth + 1) as usize] += 1;
        let pos = (pos << 1) as usize;

        if self.nsp_stack.len() < pos + 1 {
            let mut hlp = vec![None; self.nsp_stack.len() + 16];
            for i in 0..pos {
                hlp[i] = self.nsp_stack[i].clone();
            }
            self.nsp_stack = hlp;
        }

        let mut pos = pos;
        self.nsp_stack[pos] = prefix;
        pos += 1;
        self.nsp_stack[pos] = namespace;

        Ok(())
    }

    pub fn set_output_writer(&mut self, writer: Box<dyn Writer>) {
        self.writer = Some(writer);

        // elementStack = new String[12]; //nsp/prefix/name
        //nspCounts = new int[4];
        //nspStack = new String[8]; //prefix/nsp
        //indent = new boolean[4];

        self.nsp_counts[0] = 2;
        self.nsp_counts[1] = 2;
        self.nsp_stack[0] = Some(String::new());
        self.nsp_stack[1] = Some(String::new());
        self.nsp_stack[2] = Some("xml".to_string());
        self.nsp_stack[3] = Some("http://www.w3.org/XML/1998/namespace".to_string());
        self.pending = false;
        self.auto = 0;
        self.depth = 0;

        self.unicode = false;
    }

    pub fn set_output_stream(&mut self, os: Option<Box<dyn OutputStream>>, encoding: Option<String>) -> Result<(), String> {
        if os.is_none()
        // Java: throw new IllegalArgumentException();
        { return Err("IllegalArgumentException".to_string()); }
        self.set_output_writer(Box::new(OutputStreamWriter::new(os.unwrap(), encoding.clone())));
        self.encoding = encoding.clone();
        if encoding.is_some()
            && encoding.as_ref().unwrap().to_lowercase().starts_with("utf") {
            self.unicode = true;
        }
        Ok(())
    }

    pub fn start_document(
        &mut self,
        encoding: Option<String>,
        standalone: Option<bool>)
        -> Result<(), String> {
        self.writer.as_mut().unwrap().write_str("<?xml version='1.0' ".to_string());

        if encoding.is_some() {
            self.encoding = encoding.clone();
            if encoding.unwrap().to_lowercase().starts_with("utf") {
                self.unicode = true;
            }
        }

        if self.encoding.is_some() {
            self.writer.as_mut().unwrap().write_str("encoding='".to_string());
            self.writer.as_mut().unwrap().write_str(self.encoding.clone().unwrap());
            self.writer.as_mut().unwrap().write_str("' ".to_string());
        }

        if standalone.is_some() {
            self.writer.as_mut().unwrap().write_str("standalone='".to_string());
            self.writer.as_mut().unwrap().write_str(
                if standalone.unwrap() { "yes".to_string() } else { "no".to_string() });
            self.writer.as_mut().unwrap().write_str("' ".to_string());
        }
        self.writer.as_mut().unwrap().write_str("?>".to_string());

        Ok(())
    }

    pub fn start_tag(&mut self, namespace: Option<String>, name: String) -> Result<&KXmlSerializer, String> {
        self.check(false)?;

        //        if (namespace == null)
        //            namespace = "";

        if self.indent[self.depth as usize] {
            self.writer.as_mut().unwrap().write_str("\r\n".to_string());
            for i in 0..self.depth {
                self.writer.as_mut().unwrap().write_str("  ".to_string());
            }
        }

        let esp = self.depth * 3;

        if self.element_stack.len() < (esp + 3) as usize {
            let mut hlp = vec![None; self.element_stack.len() + 12];
            for i in 0..esp as usize {
                hlp[i] = self.element_stack[i].clone();
            }
            self.element_stack = hlp;
        }

        let prefix =
            if namespace.is_none() {
                Some(String::new())
            }
            else {
                self.get_prefix_internal(namespace.clone(), true, true)
            };

        if "".eq(namespace.as_deref().unwrap_or("")) {
            let mut i = self.nsp_counts[self.depth as usize];
            while i < self.nsp_counts[(self.depth + 1) as usize] {
                if "".eq(self.nsp_stack[(i * 2) as usize].as_deref().unwrap_or("")) && !"".eq(self.nsp_stack[(i * 2 + 1) as usize].as_deref().unwrap_or("")) {
                    // Java: throw new IllegalStateException("Cannot set default namespace for elements in no namespace");
                    return Err("Cannot set default namespace for elements in no namespace".to_string());
                }
                i += 1;
            }
        }

        let mut esp = esp as usize;
        self.element_stack[esp] = namespace;
        esp += 1;
        self.element_stack[esp] = prefix;
        esp += 1;
        self.element_stack[esp] = Some(name.clone());

        self.writer.as_mut().unwrap().write_char('<' as i32);
        if !"".eq(prefix.as_deref().unwrap_or("")) {
            self.writer.as_mut().unwrap().write_str(prefix.unwrap());
            self.writer.as_mut().unwrap().write_char(':' as i32);
        }

        self.writer.as_mut().unwrap().write_str(name);

        self.pending = true;

        return Ok(self);
    }

    pub fn attribute(
        &mut self,
        namespace: Option<String>,
        name: String,
        value: String)
        -> Result<&KXmlSerializer, String> {
        if !self.pending
        // Java: throw new IllegalStateException("illegal position for attribute");
        { return Err("illegal position for attribute".to_string()); }

        //        int cnt = nspCounts[depth];

        let mut namespace = namespace;
        if namespace.is_none() {
            namespace = Some(String::new());
        }

        //        depth--;
        //        pending = false;

        let prefix =
            if "".eq(namespace.as_deref().unwrap_or("")) {
                Some(String::new())
            }
            else {
                self.get_prefix_internal(namespace, false, true)
            };

        //        pending = true;
        //        depth++;

        /*        if (cnt != nspCounts[depth]) {
                    writer.write(' ');
                    writer.write("xmlns");
                    if (nspStack[cnt * 2] != null) {
                        writer.write(':');
                        writer.write(nspStack[cnt * 2]);
                    }
                    writer.write("=\"");
                    writeEscaped(nspStack[cnt * 2 + 1], '"');
                    writer.write('"');
                }
                */

        self.writer.as_mut().unwrap().write_char(' ' as i32);
        if !"".eq(prefix.as_deref().unwrap_or("")) {
            self.writer.as_mut().unwrap().write_str(prefix.unwrap());
            self.writer.as_mut().unwrap().write_char(':' as i32);
        }
        self.writer.as_mut().unwrap().write_str(name);
        self.writer.as_mut().unwrap().write_char('=' as i32);
        let q = if value.find('"').is_none() { '"' } else { '\'' };
        self.writer.as_mut().unwrap().write_char(q as i32);
        self.write_escaped(value, q as i32)?;
        self.writer.as_mut().unwrap().write_char(q as i32);

        return Ok(self);
    }

    pub fn flush(&mut self) -> Result<(), String> {
        self.check(false)?;
        self.writer.as_mut().unwrap().flush();
        Ok(())
    }
    /*
        pub fn close(&mut self) {
            self.check()?;
            self.writer.close();
        }
    */
    pub fn end_tag(&mut self, namespace: Option<String>, name: String) -> Result<&KXmlSerializer, String> {

        if !self.pending {
            self.depth -= 1;
        }
        //        if (namespace == null)
        //          namespace = "";

        if (namespace.is_none()
            && self.element_stack[(self.depth * 3) as usize].is_some())
            || (namespace.is_some()
                && !namespace.as_ref().unwrap().eq(self.element_stack[(self.depth * 3) as usize].as_ref().unwrap()))
            || !self.element_stack[(self.depth * 3 + 2) as usize].clone().unwrap().eq(&name)
        // Java: throw new IllegalArgumentException("</{"+namespace+"}"+name+"> does not match start");
        { return Err(format!("</{{{}}}{}> does not match start", namespace.as_deref().unwrap_or("null"), name)); }

        if self.pending {
            self.check(true)?;
            self.depth -= 1;
        }
        else {
            if self.indent[(self.depth + 1) as usize] {
                self.writer.as_mut().unwrap().write_str("\r\n".to_string());
                for i in 0..self.depth {
                    self.writer.as_mut().unwrap().write_str("  ".to_string());
                }
            }

            self.writer.as_mut().unwrap().write_str("</".to_string());
            let prefix = self.element_stack[(self.depth * 3 + 1) as usize].clone();
            if !"".eq(prefix.as_deref().unwrap_or("")) {
                self.writer.as_mut().unwrap().write_str(prefix.unwrap());
                self.writer.as_mut().unwrap().write_char(':' as i32);
            }
            self.writer.as_mut().unwrap().write_str(name);
            self.writer.as_mut().unwrap().write_char('>' as i32);
        }

        self.nsp_counts[(self.depth + 1) as usize] = self.nsp_counts[self.depth as usize];
        return Ok(self);
    }

    pub fn get_namespace(&self) -> Option<String> {
        return if self.get_depth() == 0 { None } else { self.element_stack[(self.get_depth() * 3 - 3) as usize].clone() };
    }

    pub fn get_name(&self) -> Option<String> {
        return if self.get_depth() == 0 { None } else { self.element_stack[(self.get_depth() * 3 - 1) as usize].clone() };
    }

    pub fn get_depth(&self) -> i32 {
        return if self.pending { self.depth + 1 } else { self.depth };
    }

    pub fn text(&mut self, text: String) -> Result<&KXmlSerializer, String> {
        self.check(false)?;
        self.indent[self.depth as usize] = false;
        self.write_escaped(text, -1)?;
        return Ok(self);
    }

    pub fn text_chars(&mut self, text: Vec<char>, start: i32, len: i32) -> Result<&KXmlSerializer, String> {
        self.text(text[start as usize..(start + len) as usize].iter().collect())
    }

    pub fn cdsect(&mut self, data: String) -> Result<(), String> {
        self.check(false)?;
        self.writer.as_mut().unwrap().write_str("<![CDATA[".to_string());
        self.writer.as_mut().unwrap().write_str(data);
        self.writer.as_mut().unwrap().write_str("]]>".to_string());
        Ok(())
    }

    pub fn comment(&mut self, comment: String) -> Result<(), String> {
        self.check(false)?;
        self.writer.as_mut().unwrap().write_str("<!--".to_string());
        self.writer.as_mut().unwrap().write_str(comment);
        self.writer.as_mut().unwrap().write_str("-->".to_string());
        Ok(())
    }

    pub fn processing_instruction(&mut self, pi: String) -> Result<(), String> {
        self.check(false)?;
        self.writer.as_mut().unwrap().write_str("<?".to_string());
        self.writer.as_mut().unwrap().write_str(pi);
        self.writer.as_mut().unwrap().write_str("?>".to_string());
        Ok(())
    }
}

/** java.io.OutputStreamWriter stand-in, wrapping an OutputStream in a Writer */
pub struct OutputStreamWriter {
    pub out: Box<dyn OutputStream>,
    pub encoding: Option<String>,
}

impl OutputStreamWriter {
    pub fn new(out: Box<dyn OutputStream>, encoding: Option<String>) -> OutputStreamWriter {
        OutputStreamWriter { out, encoding }
    }
}

impl Writer for OutputStreamWriter {
    fn write_char(&mut self, c: i32) {
        self.out.write(c);
    }

    fn write_str(&mut self, s: String) {
        for c in s.chars() {
            self.out.write(c as i32);
        }
    }

    fn flush(&mut self) {
        // no-op stand-in
    }
}
