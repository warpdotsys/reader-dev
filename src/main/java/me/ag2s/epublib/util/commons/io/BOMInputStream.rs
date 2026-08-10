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

use std::io;

use crate::me::ag2s::epublib::util::commons::io::ByteOrderMark;
use crate::me::ag2s::epublib::util::commons::io::ProxyInputStream;
use crate::me::ag2s::epublib::util::commons::io::InputStream;
use crate::me::ag2s::epublib::util::IOUtil;

/**
 * This class is used to wrap a stream that includes an encoded {@link ByteOrderMark} as its first bytes.
 *
 * This class detects these bytes and, if required, can automatically skip them and return the subsequent byte as the
 * first byte in the stream.
 *
 * The {@link ByteOrderMark} implementation has the following pre-defined BOMs:
 * <ul>
 * <li>UTF-8 - {@link ByteOrderMark#UTF_8}</li>
 * <li>UTF-16BE - {@link ByteOrderMark#UTF_16LE}</li>
 * <li>UTF-16LE - {@link ByteOrderMark#UTF_16BE}</li>
 * <li>UTF-32BE - {@link ByteOrderMark#UTF_32LE}</li>
 * <li>UTF-32LE - {@link ByteOrderMark#UTF_32BE}</li>
 * </ul>
 *
 *
 * <h2>Example 1 - Detect and exclude a UTF-8 BOM</h2>
 *
 * <pre>
 * BOMInputStream bomIn = new BOMInputStream(in);
 * if (bomIn.hasBOM()) {
 *     // has a UTF-8 BOM
 * }
 * </pre>
 *
 * <h2>Example 2 - Detect a UTF-8 BOM (but don't exclude it)</h2>
 *
 * <pre>
 * boolean include = true;
 * BOMInputStream bomIn = new BOMInputStream(in, include);
 * if (bomIn.hasBOM()) {
 *     // has a UTF-8 BOM
 * }
 * </pre>
 *
 * <h2>Example 3 - Detect Multiple BOMs</h2>
 *
 * <pre>
 * BOMInputStream bomIn = new BOMInputStream(in,
 *   ByteOrderMark.UTF_16LE, ByteOrderMark.UTF_16BE,
 *   ByteOrderMark.UTF_32LE, ByteOrderMark.UTF_32BE
 *   );
 * if (bomIn.hasBOM() == false) {
 *     // No BOM found
 * } else if (bomIn.hasBOM(ByteOrderMark.UTF_16LE)) {
 *     // has a UTF-16LE BOM
 * } else if (bomIn.hasBOM(ByteOrderMark.UTF_16BE)) {
 *     // has a UTF-16BE BOM
 * } else if (bomIn.hasBOM(ByteOrderMark.UTF_32LE)) {
 *     // has a UTF-32LE BOM
 * } else if (bomIn.hasBOM(ByteOrderMark.UTF_32BE)) {
 *     // has a UTF-32BE BOM
 * }
 * </pre>
 *
 * @see ByteOrderMark
 * @see <a href="http://en.wikipedia.org/wiki/Byte_order_mark">Wikipedia - Byte Order Mark</a>
 * @since 2.0
 */
pub struct BOMInputStream {
    proxy: ProxyInputStream,
    include: bool,
    /**
     * BOMs are sorted from longest to shortest.
     */
    boms: Vec<ByteOrderMark>,
    byte_order_mark: Option<ByteOrderMark>,
    first_bytes: Option<Vec<i32>>,
    fb_length: usize,
    fb_index: usize,
    mark_fb_index: usize,
    marked_at_start: bool,
}

/**
 * Compares ByteOrderMark objects in descending length order.
 */
fn byte_order_mark_length_comparator(bom1: &ByteOrderMark, bom2: &ByteOrderMark) -> std::cmp::Ordering {
    let len1 = bom1.length();
    let len2 = bom2.length();
    len2.cmp(&len1)
}

impl BOMInputStream {

    /**
     * Constructs a new BOM InputStream that excludes a {@link ByteOrderMark#UTF_8} BOM.
     *
     * @param delegate
     *            the InputStream to delegate to
     */
    #[allow(dead_code)]
    pub fn new(delegate: Box<dyn InputStream>) -> Self {
        BOMInputStream::new_include(delegate, false, vec![ByteOrderMark::UTF_8])
    }

    /**
     * Constructs a new BOM InputStream that detects a a {@link ByteOrderMark#UTF_8} and optionally includes it.
     *
     * @param delegate
     *            the InputStream to delegate to
     * @param include
     *            true to include the UTF-8 BOM or false to exclude it
     */
    #[allow(dead_code)]
    pub fn new_include(delegate: Box<dyn InputStream>, include: bool) -> Self {
        BOMInputStream::new_boms(delegate, include, vec![ByteOrderMark::UTF_8])
    }

    /**
     * Constructs a new BOM InputStream that excludes the specified BOMs.
     *
     * @param delegate
     *            the InputStream to delegate to
     * @param boms
     *            The BOMs to detect and exclude
     */
    #[allow(dead_code)]
    pub fn new_exclude(delegate: Box<dyn InputStream>, boms: Vec<ByteOrderMark>) -> Self {
        BOMInputStream::new_boms(delegate, false, boms)
    }

    /**
     * Constructs a new BOM InputStream that detects the specified BOMs and optionally includes them.
     *
     * @param delegate
     *            the InputStream to delegate to
     * @param include
     *            true to include the specified BOMs or false to exclude them
     * @param boms
     *            The BOMs to detect and optionally exclude
     */
    pub fn new_boms(delegate: Box<dyn InputStream>, include: bool, boms: Vec<ByteOrderMark>) -> Self {
        let proxy = ProxyInputStream::new(delegate);
        if IOUtil::length_obj(&boms) == 0 {
            panic!("No BOMs specified");
        }
        // Sort the BOMs to match the longest BOM first because some BOMs have the same starting two bytes.
        // if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
        //     list.sort(ByteOrderMarkLengthComparator);
        // }
        BOMInputStream {
            proxy,
            include,
            boms,
            byte_order_mark: None,
            first_bytes: None,
            fb_length: 0,
            fb_index: 0,
            mark_fb_index: 0,
            marked_at_start: false,
        }
    }

    /**
     * Indicates whether the stream contains one of the specified BOMs.
     *
     * @return true if the stream has one of the specified BOMs, otherwise false if it does not
     * @throws IOException
     *             if an error reading the first bytes of the stream occurs
     */
    #[allow(dead_code)]
    pub fn has_bom(&mut self) -> Result<bool, io::Error> {
        Ok(self.get_bom()?.is_some())
    }

    /**
     * Indicates whether the stream contains the specified BOM.
     *
     * @param bom
     *            The BOM to check for
     * @return true if the stream has the specified BOM, otherwise false if it does not
     * @throws IllegalArgumentException
     *             if the BOM is not one the stream is configured to detect
     * @throws IOException
     *             if an error reading the first bytes of the stream occurs
     */
    #[allow(dead_code)]
    pub fn has_bom_of(&mut self, bom: &ByteOrderMark) -> Result<bool, io::Error> {
        if !self.boms.contains(bom) {
            panic!("Stream not configure to detect {}", bom.to_string());
        }
        self.get_bom()?;
        Ok(self.byte_order_mark != null && self.byte_order_mark.as_ref().unwrap().equals(bom))
    }

    /**
     * Return the BOM (Byte Order Mark).
     *
     * @return The BOM or null if none
     * @throws IOException
     *             if an error reading the first bytes of the stream occurs
     */
    pub fn get_bom(&mut self) -> Result<Option<&ByteOrderMark>, io::Error> {
        if self.first_bytes.is_none() {
            self.fb_length = 0;
            // BOMs are sorted from longest to shortest
            let max_bom_size = self.boms.get(0).unwrap().length();
            let mut first_bytes = vec![0i32; max_bom_size];
            // Read first maxBomSize bytes
            for i in 0..first_bytes.len() {
                first_bytes[i] = self.proxy.in_stream.read_byte();
                self.fb_length += 1;
                if first_bytes[i] < 0 {
                    break;
                }
            }
            self.first_bytes = Some(first_bytes);
            // match BOM in firstBytes
            self.byte_order_mark = self.find();
            if self.byte_order_mark.is_some() {
                if !self.include {
                    if self.byte_order_mark.as_ref().unwrap().length() < self.first_bytes.as_ref().unwrap().len() {
                        self.fb_index = self.byte_order_mark.as_ref().unwrap().length();
                    } else {
                        self.fb_length = 0;
                    }
                }
            }
        }
        Ok(self.byte_order_mark.as_ref())
    }

    /**
     * Return the BOM charset Name - {@link ByteOrderMark#getCharsetName()}.
     *
     * @return The BOM charset Name or null if no BOM found
     * @throws IOException
     *             if an error reading the first bytes of the stream occurs
     *
     */
    pub fn get_bom_charset_name(&mut self) -> Result<Option<String>, io::Error> {
        self.get_bom()?;
        Ok(self.byte_order_mark.as_ref().map(|b| b.get_charset_name().to_string()))
    }

    /**
     * This method reads and either preserves or skips the first bytes in the stream. It behaves like the single-byte
     * <code>read()</code> method, either returning a valid byte or -1 to indicate that the initial bytes have been
     * processed already.
     *
     * @return the byte read (excluding BOM) or -1 if the end of stream
     * @throws IOException
     *             if an I/O error occurs
     */
    fn read_first_bytes(&mut self) -> Result<i32, io::Error> {
        self.get_bom()?;
        Ok(if self.fb_index < self.fb_length { let v = self.first_bytes.as_ref().unwrap()[self.fb_index]; self.fb_index += 1; v } else { IOUtil::EOF })
    }

    /**
     * Find a BOM with the specified bytes.
     *
     * @return The matched BOM or null if none matched
     */
    fn find(&self) -> Option<ByteOrderMark> {
        for bom in &self.boms {
            if self.matches(bom) {
                return Some(bom.clone());
            }
        }
        None
    }

    /**
     * Check if the bytes match a BOM.
     *
     * @param bom
     *            The BOM
     * @return true if the bytes match the bom, otherwise false
     */
    fn matches(&self, bom: &ByteOrderMark) -> bool {
        // if (bom.length() != fbLength) {
        // return false;
        // }
        // firstBytes may be bigger than the BOM bytes
        let first_bytes = self.first_bytes.as_ref().unwrap();
        for i in 0..bom.length() {
            if bom.get(i) != first_bytes[i] {
                return false;
            }
        }
        true
    }

    // ----------------------------------------------------------------------------
    // Implementation of InputStream
    // ----------------------------------------------------------------------------

    /**
     * Invokes the delegate's <code>read()</code> method, detecting and optionally skipping BOM.
     *
     * @return the byte read (excluding BOM) or -1 if the end of stream
     * @throws IOException
     *             if an I/O error occurs
     */
    pub fn read(&mut self) -> Result<i32, io::Error> {
        let b = self.read_first_bytes()?;
        Ok(if b >= 0 { b } else { self.proxy.in_stream.read_byte() })
    }

    /**
     * Invokes the delegate's <code>read(byte[], int, int)</code> method, detecting and optionally skipping BOM.
     *
     * @param buf
     *            the buffer to read the bytes into
     * @param off
     *            The start offset
     * @param len
     *            The number of bytes to read (excluding BOM)
     * @return the number of bytes read or -1 if the end of stream
     * @throws IOException
     *             if an I/O error occurs
     */
    pub fn read_off(&mut self, buf: &mut [u8], mut off: usize, mut len: usize) -> Result<i32, io::Error> {
        let mut first_count = 0;
        let mut b = 0;
        while len > 0 && b >= 0 {
            b = self.read_first_bytes()?;
            if b >= 0 {
                buf[off] = (b & 0xFF) as u8;
                off += 1;
                len -= 1;
                first_count += 1;
            }
        }
        let second_count = self.proxy.in_stream.read_off(buf, off, len);
        Ok(if second_count < 0 { if first_count > 0 { first_count } else { IOUtil::EOF } } else { first_count + second_count })
    }

    /**
     * Invokes the delegate's <code>read(byte[])</code> method, detecting and optionally skipping BOM.
     *
     * @param buf
     *            the buffer to read the bytes into
     * @return the number of bytes read (excluding BOM) or -1 if the end of stream
     * @throws IOException
     *             if an I/O error occurs
     */
    pub fn read_bytes(&mut self, buf: &mut [u8]) -> Result<i32, io::Error> {
        self.read_off(buf, 0, buf.len())
    }

    /**
     * Invokes the delegate's <code>mark(int)</code> method.
     *
     * @param readlimit
     *            read ahead limit
     */
    pub fn mark(&mut self, readlimit: i32) {
        self.mark_fb_index = self.fb_index;
        self.marked_at_start = self.first_bytes.is_none();
        self.proxy.in_stream.mark(readlimit);
    }

    /**
     * Invokes the delegate's <code>reset()</code> method.
     *
     * @throws IOException
     *             if an I/O error occurs
     */
    pub fn reset(&mut self) -> Result<(), io::Error> {
        self.fb_index = self.mark_fb_index;
        if self.marked_at_start {
            self.first_bytes = None;
        }

        self.proxy.in_stream.reset()
    }

    /**
     * Invokes the delegate's <code>skip(long)</code> method, detecting and optionally skipping BOM.
     *
     * @param n
     *            the number of bytes to skip
     * @return the number of bytes to skipped or -1 if the end of stream
     * @throws IOException
     *             if an I/O error occurs
     */
    pub fn skip(&mut self, n: i64) -> Result<i64, io::Error> {
        let mut skipped = 0;
        while (n > skipped) && (self.read_first_bytes()? >= 0) {
            skipped += 1;
        }
        Ok(self.proxy.in_stream.skip(n - skipped)? + skipped)
    }
}

impl Clone for ByteOrderMark {
    fn clone(&self) -> Self {
        ByteOrderMark::new(self.get_charset_name(), &self.bytes)
    }
}
