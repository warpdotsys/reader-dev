/*
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.
 * The ASF licenses this file to You under the Apache License, Version 2.0
 * (the "License"); you may not use this file except in compliance with
 * the License.  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/**
 * Byte Order Mark (BOM) representation - see {@link BOMInputStream}.
 *
 * @see BOMInputStream
 * @see <a href="http://en.wikipedia.org/wiki/Byte_order_mark">Wikipedia: Byte Order Mark</a>
 * @see <a href="http://www.w3.org/TR/2006/REC-xml-20060816/#sec-guessing">W3C: Autodetection of Character Encodings
 *      (Non-Normative)</a>
 * @since 2.0
 */
pub struct ByteOrderMark {
    serial_version_uid: i64,
    charset_name: String,
    bytes: Vec<i32>,
}

impl ByteOrderMark {

    const SERIAL_VERSION_UID: i64 = 1;

    /** UTF-8 BOM */
    pub static UTF_8: ByteOrderMark = ByteOrderMark::new("UTF-8", &[0xEF, 0xBB, 0xBF]);

    /** UTF-16BE BOM (Big-Endian) */
    pub static UTF_16BE: ByteOrderMark = ByteOrderMark::new("UTF-16BE", &[0xFE, 0xFF]);

    /** UTF-16LE BOM (Little-Endian) */
    pub static UTF_16LE: ByteOrderMark = ByteOrderMark::new("UTF-16LE", &[0xFF, 0xFE]);

    /**
     * UTF-32BE BOM (Big-Endian)
     * @since 2.2
     */
    pub static UTF_32BE: ByteOrderMark = ByteOrderMark::new("UTF-32BE", &[0x00, 0x00, 0xFE, 0xFF]);

    /**
     * UTF-32LE BOM (Little-Endian)
     * @since 2.2
     */
    pub static UTF_32LE: ByteOrderMark = ByteOrderMark::new("UTF-32LE", &[0xFF, 0xFE, 0x00, 0x00]);

    /**
     * Unicode BOM character; external form depends on the encoding.
     * @see <a href="http://unicode.org/faq/utf_bom.html#BOM">Byte Order Mark (BOM) FAQ</a>
     * @since 2.5
     */
    #[allow(dead_code)]
    pub const UTF_BOM: char = '\u{FEFF}';

    /**
     * Construct a new BOM.
     *
     * @param charsetName The name of the charset the BOM represents
     * @param bytes The BOM's bytes
     * @throws IllegalArgumentException if the charsetName is null or
     * zero length
     * @throws IllegalArgumentException if the bytes are null or zero
     * length
     */
    pub fn new(charset_name: &str, bytes: &[i32]) -> Self {
        if charset_name.is_empty() {
            panic!("No charsetName specified");
        }
        if bytes.len() == 0 {
            panic!("No bytes specified");
        }
        ByteOrderMark {
            serial_version_uid: 1,
            charset_name: charset_name.to_string(),
            bytes: bytes.to_vec(),
        }
    }

    /**
     * Return the name of the {@link java.nio.charset.Charset} the BOM represents.
     *
     * @return the character set name
     */
    pub fn get_charset_name(&self) -> &str {
        &self.charset_name
    }

    /**
     * Return the length of the BOM's bytes.
     *
     * @return the length of the BOM's bytes
     */
    pub fn length(&self) -> usize {
        self.bytes.len()
    }

    /**
     * The byte at the specified position.
     *
     * @param pos The position
     * @return The specified byte
     */
    pub fn get(&self, pos: usize) -> i32 {
        self.bytes[pos]
    }

    /**
     * Return a copy of the BOM's bytes.
     *
     * @return a copy of the BOM's bytes
     */
    pub fn get_bytes(&self) -> Vec<u8> {
        let mut copy = vec![0u8; self.bytes.len()];
        for i in 0..self.bytes.len() {
            copy[i] = self.bytes[i] as u8;
        }
        copy
    }

    /**
     * Indicates if this BOM's bytes equals another.
     *
     * @param obj The object to compare to
     * @return true if the bom's bytes are equal, otherwise
     * false
     */
    pub fn equals(&self, obj: &dyn Any) -> bool {
        if !obj.is::<ByteOrderMark>() {
            return false;
        }
        let bom = obj.downcast_ref::<ByteOrderMark>().unwrap();
        if self.bytes.len() != bom.length() {
            return false;
        }
        for i in 0..self.bytes.len() {
            if self.bytes[i] != bom.get(i) {
                return false;
            }
        }
        true
    }

    /**
     * Return the hashcode for this BOM.
     *
     * @return the hashcode for this BOM.
     * @see java.lang.Object#hashCode()
     */
    pub fn hash_code(&self) -> i32 {
        let mut hash_code = std::any::type_name::<ByteOrderMark>().hash_code();
        for b in &self.bytes {
            hash_code += b;
        }
        hash_code
    }

    /**
     * Provide a String representation of the BOM.
     *
     * @return the length of the BOM's bytes
     */
    pub fn to_string(&self) -> String {
        let mut builder = String::new();
        builder.push_str(std::any::type_name::<ByteOrderMark>().simple_name());
        builder.push('[');
        builder.push_str(&self.charset_name);
        builder.push_str(": ");
        for i in 0..self.bytes.len() {
            if i > 0 {
                builder.push_str(",");
            }
            builder.push_str("0x");
            builder.push_str(&format!("{:X}", 0xFF & self.bytes[i]).to_uppercase());
        }
        builder.push(']');
        builder
    }
}

use std::any::Any;

trait HashCodeExt {
    fn hash_code(&self) -> i32;
}

impl HashCodeExt for &str {
    fn hash_code(&self) -> i32 {
        let mut h = 0;
        for b in self.bytes() {
            h = 31 * h + b as i32;
        }
        h
    }
}

trait SimpleNameExt {
    fn simple_name(&self) -> &'static str;
}

impl SimpleNameExt for &'static str {
    fn simple_name(&self) -> &'static str {
        self.rsplit("::").next().unwrap_or(self)
    }
}
