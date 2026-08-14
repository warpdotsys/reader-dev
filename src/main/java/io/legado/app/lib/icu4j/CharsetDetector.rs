use crate::prelude::*;
// fix: E0659 `Reader` 歧义（glob 导入冲突）；与 CharsetMatch::get_reader() 返回的占位 Reader 保持同一类型
use crate::stubs::CharsetMatchReader as Reader;
// © 2016 and later: Unicode, Inc. and others.
// License & terms of use: http://www.unicode.org/copyright.html
/*
  ******************************************************************************
  Copyright (C) 2005-2016, International Business Machines Corporation and    *
  others. All Rights Reserved.                                                *
  ******************************************************************************
 */
// package io.legado.app.lib.icu4j;
//
// import java.io.IOException;
// import java.io.InputStream;
// import java.io.Reader;
// import java.util.ArrayList;
// import java.util.Arrays;
// import java.util.Collections;
// import java.util.List;

/**
 * <code>CharsetDetector</code> provides a facility for detecting the
 * charset or encoding of character data in an unknown format.
 * The input data can either be from an input stream or an array of bytes.
 * The result of the detection operation is a list of possibly matching
 * charsets, or, for simple use, you can just ask for a Java Reader that
 * will will work over the input data.
 * <p>
 * Character set detection is at best an imprecise operation.  The detection
 * process will attempt to identify the charset that best matches the characteristics
 * of the byte data, but the process is partly statistical in nature, and
 * the results can not be guaranteed to always be correct.
 * <p>
 * For best accuracy in charset detection, the input data should be primarily
 * in a single language, and a minimum of a few hundred bytes worth of plain text
 * in the language are needed.  The detection process will attempt to
 * ignore html or xml style markup that could otherwise obscure the content.
 * <p>
 *
 * @stable ICU 3.4
 */
// @SuppressWarnings({"JavaDoc", "unused", "RedundantSuppression"})
// public class CharsetDetector {
pub struct CharsetDetector {
    /*
     *  The following items are accessed by individual CharsetRecongizers during
     *     the recognition process
     *
     */
    pub(crate) f_input_bytes: Vec<u8>,   // The text to be checked.  Markup will have been
    //   removed if appropriate.
    pub(crate) f_input_len: i32,          // Length of the byte data in fInputBytes.

    f_byte_stats: Vec<u16>,    // byte frequency statistics for the input text.
    //   Value is percent, not absolute.
    //   Value is rounded up, so zero really means zero occurences.

    f_c1_bytes: bool,          // True if any bytes in the range 0x80 - 0x9F are in the input;

    f_declared_encoding: Option<String>,

    f_raw_input: Option<Vec<u8>>, // Original, untouched input bytes.
    //  If user gave us a byte array, this is it.
    //  If user gave us a stream, it's read to a
    //  buffer here.
    f_raw_length: i32,          // Length of data in fRawInput array.

    f_input_stream: Option<Box<dyn InputStream>>, // User's input stream, or None if the user
    //   gave us a byte array.

    //
    //  Stuff private to CharsetDetector
    //
    f_strip_tags: bool,        // If true, setText() will strip tags from input text.

    f_enabled_recognizers: Option<Vec<bool>>, // If not None, active set of charset recognizers had
    // been changed from the default. The array index is
    // corresponding to ALL_RECOGNIZER. See setDetectableCharset().
}

// private static final int kBufSize = 8000;
pub const K_BUF_SIZE: usize = 8000;

/*
 * List of recognizers for all charsets known to the implementation.
 */
// private static final List<CSRecognizerInfo> ALL_CS_RECOGNIZERS;
// static {
//     List<CSRecognizerInfo> list = new ArrayList<>();
//
//     list.add(new CSRecognizerInfo(new CharsetRecog_UTF8(), true));
//     list.add(new CSRecognizerInfo(new CharsetRecog_Unicode.CharsetRecog_UTF_16_BE(), true));
//     list.add(new CSRecognizerInfo(new CharsetRecog_Unicode.CharsetRecog_UTF_16_LE(), true));
//     list.add(new CSRecognizerInfo(new CharsetRecog_Unicode.CharsetRecog_UTF_32_BE(), true));
//     list.add(new CSRecognizerInfo(new CharsetRecog_Unicode.CharsetRecog_UTF_32_LE(), true));
//
//     list.add(new CSRecognizerInfo(new CharsetRecog_mbcs.CharsetRecog_sjis(), true));
//     list.add(new CSRecognizerInfo(new CharsetRecog_2022.CharsetRecog_2022JP(), true));
//     list.add(new CSRecognizerInfo(new CharsetRecog_2022.CharsetRecog_2022CN(), true));
//     list.add(new CSRecognizerInfo(new CharsetRecog_2022.CharsetRecog_2022KR(), true));
//     list.add(new CSRecognizerInfo(new CharsetRecog_mbcs.CharsetRecog_euc.CharsetRecog_gb_18030(), true));
//     list.add(new CSRecognizerInfo(new CharsetRecog_mbcs.CharsetRecog_euc.CharsetRecog_euc_jp(), true));
//     list.add(new CSRecognizerInfo(new CharsetRecog_mbcs.CharsetRecog_euc.CharsetRecog_euc_kr(), true));
//     list.add(new CSRecognizerInfo(new CharsetRecog_mbcs.CharsetRecog_big5(), true));
//
//     list.add(new CSRecognizerInfo(new CharsetRecog_sbcs.CharsetRecog_8859_1(), true));
//     list.add(new CSRecognizerInfo(new CharsetRecog_sbcs.CharsetRecog_8859_2(), true));
//     list.add(new CSRecognizerInfo(new CharsetRecog_sbcs.CharsetRecog_8859_5_ru(), true));
//     list.add(new CSRecognizerInfo(new CharsetRecog_sbcs.CharsetRecog_8859_6_ar(), true));
//     list.add(new CSRecognizerInfo(new CharsetRecog_sbcs.CharsetRecog_8859_7_el(), true));
//     list.add(new CSRecognizerInfo(new CharsetRecog_sbcs.CharsetRecog_8859_8_I_he(), true));
//     list.add(new CSRecognizerInfo(new CharsetRecog_sbcs.CharsetRecog_8859_8_he(), true));
//     list.add(new CSRecognizerInfo(new CharsetRecog_sbcs.CharsetRecog_windows_1251(), true));
//     list.add(new CSRecognizerInfo(new CharsetRecog_sbcs.CharsetRecog_windows_1256(), true));
//     list.add(new CSRecognizerInfo(new CharsetRecog_sbcs.CharsetRecog_KOI8_R(), true));
//     list.add(new CSRecognizerInfo(new CharsetRecog_sbcs.CharsetRecog_8859_9_tr(), true));
//
//     // IBM 420/424 recognizers are disabled by default
//     list.add(new CSRecognizerInfo(new CharsetRecog_sbcs.CharsetRecog_IBM424_he_rtl(), false));
//     list.add(new CSRecognizerInfo(new CharsetRecog_sbcs.CharsetRecog_IBM424_he_ltr(), false));
//     list.add(new CSRecognizerInfo(new CharsetRecog_sbcs.CharsetRecog_IBM420_ar_rtl(), false));
//     list.add(new CSRecognizerInfo(new CharsetRecog_sbcs.CharsetRecog_IBM420_ar_ltr(), false));
//
//     //noinspection Java9CollectionFactory
//     ALL_CS_RECOGNIZERS = Collections.unmodifiableList(list);
// }
pub fn all_cs_recognizers() -> &'static Vec<CSRecognizerInfo> {
    use std::sync::OnceLock;
    static ALL_CS_RECOGNIZERS: OnceLock<Vec<CSRecognizerInfo>> = OnceLock::new();
    ALL_CS_RECOGNIZERS.get_or_init(|| {
        // List<CSRecognizerInfo> list = new ArrayList<>();
        let mut list: Vec<CSRecognizerInfo> = Vec::new();

        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_UTF8 {}), true));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_UTF_16_BE {}), true));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_UTF_16_LE {}), true));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_UTF_32_BE {}), true));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_UTF_32_LE {}), true));

        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_sjis {}), true));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_2022JP {}), true));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_2022CN {}), true));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_2022KR {}), true));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_gb_18030 {}), true));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_euc_jp {}), true));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_euc_kr {}), true));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_big5 {}), true));

        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_8859_1 {}), true));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_8859_2 {}), true));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_8859_5_ru {}), true));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_8859_6_ar {}), true));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_8859_7_el {}), true));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_8859_8_I_he {}), true));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_8859_8_he {}), true));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_windows_1251 {}), true));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_windows_1256 {}), true));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_KOI8_R {}), true));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_8859_9_tr {}), true));

        // IBM 420/424 recognizers are disabled by default
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_IBM424_he_rtl {}), false));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_IBM424_he_ltr {}), false));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_IBM420_ar_rtl {}), false));
        list.push(CSRecognizerInfo::new(Box::new(CharsetRecog_IBM420_ar_ltr {}), false));

        //noinspection Java9CollectionFactory
        // ALL_CS_RECOGNIZERS = Collections.unmodifiableList(list);
        list
    })
}

impl CharsetDetector {
    /**
     * Constructor
     *
     * @stable ICU 3.4
     */
    // public CharsetDetector() {
    pub fn new() -> CharsetDetector {
        CharsetDetector {
            f_input_bytes: vec![0_u8; K_BUF_SIZE],
            f_input_len: 0,
            f_byte_stats: vec![0_u16; 256],
            f_c1_bytes: false,
            f_declared_encoding: None,
            f_raw_input: None,
            f_raw_length: 0,
            f_input_stream: None,
            f_strip_tags: false,
            f_enabled_recognizers: None,
        }
    }

    // fix: 私有字段访问器（CharsetMatch 等跨模块转录文件使用）
    pub(crate) fn has_input_stream(&self) -> bool {
        self.f_input_stream.is_some()
    }
    pub(crate) fn raw_input(&self) -> &Option<Vec<u8>> {
        &self.f_raw_input
    }
    pub(crate) fn raw_length(&self) -> i32 {
        self.f_raw_length
    }

    /**
     * Set the declared encoding for charset detection.
     * The declared encoding of an input text is an encoding obtained
     * from an http header or xml declaration or similar source that
     * can be provided as additional information to the charset detector.
     * A match between a declared encoding and a possible detected encoding
     * will raise the quality of that detected encoding by a small delta,
     * and will also appear as a "reason" for the match.
     * <p>
     * A declared encoding that is incompatible with the input data being
     * analyzed will not be added to the list of possible encodings.
     *
     * @param encoding The declared encoding
     * @stable ICU 3.4
     */
    // public CharsetDetector setDeclaredEncoding(String encoding) {
    //     fDeclaredEncoding = encoding;
    //     return this;
    // }
    pub fn set_declared_encoding(&mut self, encoding: &str) -> &mut CharsetDetector {
        self.f_declared_encoding = Some(encoding.to_string());
        self
    }

    /**
     * Set the input text (byte) data whose charset is to be detected.
     *
     * @param in the input text of unknown encoding
     * @return This CharsetDetector
     * @stable ICU 3.4
     */
    // public CharsetDetector setText(byte[] in) {
    //     fRawInput = in;
    //     fRawLength = in.length;
    //
    //     return this;
    // }
    pub fn set_text(&mut self, input: Vec<u8>) -> &mut CharsetDetector {
        self.f_raw_input = Some(input.clone());
        self.f_raw_length = input.len() as i32;

        self
    }

    /**
     * Set the input text (byte) data whose charset is to be detected.
     * <p>
     * The input stream that supplies the character data must have markSupported()
     * == true; the charset detection process will read a small amount of data,
     * then return the stream to its original position via
     * the InputStream.reset() operation.  The exact amount that will
     * be read depends on the characteristics of the data itself.
     *
     * @param in the input text of unknown encoding
     * @return This CharsetDetector
     * @stable ICU 3.4
     */

    // public CharsetDetector setText(InputStream in) throws IOException {
    pub fn set_text_stream(&mut self, input: Box<dyn InputStream>) -> Result<&mut CharsetDetector, IOException> {
        self.f_input_stream = Some(input);
        self.f_input_stream.as_mut().unwrap().mark(K_BUF_SIZE);
        // fRawInput = new byte[kBufSize];   // Always make a new buffer because the
        //                                    // previous one may have come from the caller,
        //                                    // in which case we can't touch it.
        self.f_raw_input = Some(vec![0_u8; K_BUF_SIZE]);
        self.f_raw_length = 0;
        let mut remaining_length = K_BUF_SIZE;
        while remaining_length > 0 {
            // read() may give data in smallish chunks, esp. for remote sources.  Hence, this loop.
            // int bytesRead = fInputStream.read(fRawInput, fRawLength, remainingLength);
            let bytes_read = self.f_input_stream.as_mut().unwrap().read(
                self.f_raw_input.as_mut().unwrap(),
                self.f_raw_length as usize,
                remaining_length,
            );
            // if (bytesRead <= 0) {
            //     break;
            // }
            if bytes_read <= 0 {
                break;
            }
            self.f_raw_length += bytes_read as i32;
            remaining_length -= bytes_read as usize;
        }
        self.f_input_stream.as_mut().unwrap().reset()?;

        Ok(self)
    }

    /**
     * Return the charset that best matches the supplied input data.
     * <p>
     * Note though, that because the detection
     * only looks at the start of the input data,
     * there is a possibility that the returned charset will fail to handle
     * the full set of input data.
     * p>
     * aise an exception if
     * <ul>
     *   <li>no charset appears to match the data.</li>
     *   <li>no input text has been provided</li>
     * </ul>
     *
     * @return a CharsetMatch object representing the best matching charset, or
     * <code>None</code> if there are no matches.
     * @stable ICU 3.4
     */
    // public CharsetMatch detect() {
    // //   TODO:  A better implementation would be to copy the detect loop from
    // //          detectAll(), and cut it short as soon as a match with a high confidence
    // //          is found.  This is something to be done later, after things are otherwise
    // //          working.
    //     CharsetMatch[] matches = detectAll();
    //
    //     if (matches == None || matches.length == 0) {
    //         return None;
    //     }
    //
    //     return matches[0];
    // }
    pub fn detect(&mut self) -> Option<CharsetMatch> {
        //   TODO:  A better implementation would be to copy the detect loop from
        //          detectAll(), and cut it short as soon as a match with a high confidence
        //          is found.  This is something to be done later, after things are otherwise
        //          working.
        let matches = self.detect_all();

        // if (matches == None || matches.length == 0) {
        //     return None;
        // }
        if matches.is_empty() {
            return None;
        }

        // return matches[0];
        matches.into_iter().next()
    }

    /**
     * Return an array of all charsets that appear to be plausible
     * matches with the input data.  The array is ordered with the
     * best quality match first.
     * <p>
     * aise an exception if
     * <ul>
     *   <li>no charsets appear to match the input data.</li>
     *   <li>no input text has been provided</li>
     * </ul>
     *
     * @return An array of CharsetMatch objects representing possibly matching charsets.
     * @stable ICU 3.4
     */
    // public CharsetMatch[] detectAll() {
    pub fn detect_all(&mut self) -> Vec<CharsetMatch> {
        // ArrayList<CharsetMatch> matches = new ArrayList<>();
        let mut matches: Vec<CharsetMatch> = Vec::new();

        self.munge_input(); // Strip html markup, collect byte stats.

        //  Iterate over all possible charsets, remember all that
        //    give a match quality > 0.
        // for (int i = 0; i < ALL_CS_RECOGNIZERS.size(); i++) {
        //     CSRecognizerInfo rcinfo = ALL_CS_RECOGNIZERS.get(i);
        //     boolean active = (fEnabledRecognizers != None) ? fEnabledRecognizers[i] : rcinfo.isDefaultEnabled;
        //     if (active) {
        //         CharsetMatch m = rcinfo.recognizer.match(this);
        //         if (m != None) {
        //             matches.add(m);
        //         }
        //     }
        // }
        let recognizers = all_cs_recognizers();
        for (i, rcinfo) in recognizers.iter().enumerate() {
            // boolean active = (fEnabledRecognizers != None) ? fEnabledRecognizers[i] : rcinfo.isDefaultEnabled;
            let active = match &self.f_enabled_recognizers {
                Some(enabled) => enabled[i],
                None => rcinfo.is_default_enabled,
            };
            if active {
                // CharsetMatch m = rcinfo.recognizer.match(this);
                let m = rcinfo.recognizer.match_det(self);
                if m.is_some() {
                    matches.push(m.unwrap());
                }
            }
        }
        // Collections.sort(matches);      // CharsetMatch compares on confidence
        // Collections.reverse(matches);   //  Put best match first.
        matches.sort_by(|a, b| a.compare_to(b).cmp(&0));
        matches.reverse();
        // CharsetMatch[] resultArray = new CharsetMatch[matches.size()];
        // resultArray = matches.toArray(resultArray);
        // return resultArray;
        matches
    }

    /**
     * Autodetect the charset of an inputStream, and return a Java Reader
     * to access the converted input data.
     * <p>
     * This is a convenience method that is equivalent to
     * <code>this.setDeclaredEncoding(declaredEncoding).setText(in).detect().getReader();</code>
     * <p>
     * For the input stream that supplies the character data, markSupported()
     * must be true; the  charset detection will read a small amount of data,
     * then return the stream to its original position via
     * the InputStream.reset() operation.  The exact amount that will
     * be read depends on the characteristics of the data itself.
     * <p>
     * Raise an exception if no charsets appear to match the input data.
     *
     * @param in               The source of the byte data in the unknown charset.
     * @param declaredEncoding A declared encoding for the data, if available,
     *                         or None or an empty string if none is available.
     * @stable ICU 3.4
     */
    // public Reader getReader(InputStream in, String declaredEncoding) {
    pub fn get_reader(&mut self, input: Box<dyn InputStream>, declared_encoding: &str) -> Option<Reader> {
        self.f_declared_encoding = Some(declared_encoding.to_string());

        // try {
        //     setText(in);
        //     CharsetMatch match = detect();
        //     if (match == None) {
        //         return None;
        //     }
        //     return match.getReader();
        // } catch (IOException e) {
        //     return None;
        // }
        match (|| -> Result<Option<Reader>, IOException> {
            self.set_text_stream(input)?;
            let m = self.detect();

            if m.is_none() {
                return Ok(None);
            }

            Ok(m.unwrap().get_reader())
        })() {
            Ok(reader) => reader,
            Err(_e) => None,
        }
    }

    /**
     * Autodetect the charset of an inputStream, and return a String
     * containing the converted input data.
     * <p>
     * This is a convenience method that is equivalent to
     * <code>this.setDeclaredEncoding(declaredEncoding).setText(in).detect().getString();</code>
     * <p>
     * Raise an exception if no charsets appear to match the input data.
     *
     * @param in               The source of the byte data in the unknown charset.
     * @param declaredEncoding A declared encoding for the data, if available,
     *                         or None or an empty string if none is available.
     * @stable ICU 3.4
     */
    // public String getString(byte[] in, String declaredEncoding) {
    pub fn get_string(&mut self, input: Vec<u8>, declared_encoding: &str) -> Option<String> {
        self.f_declared_encoding = Some(declared_encoding.to_string());

        // try {
        //     setText(in);
        //     CharsetMatch match = detect();
        //     if (match == None) {
        //         return None;
        //     }
        //     return match.getString(-1);
        // } catch (IOException e) {
        //     return None;
        // }
        match (|| -> Result<Option<String>, IOException> {
            self.set_text(input);
            let m = self.detect();

            if m.is_none() {
                return Ok(None);
            }

            match m.unwrap().get_string_max(-1) {
                Ok(s) => Ok(Some(s)),
                Err(_) => Ok(None),
            }
        })() {
            Ok(s) => s,
            Err(_e) => None,
        }
    }

    /**
     * Get the names of all charsets supported by <code>CharsetDetector</code> class.
     * <p>
     * <b>Note:</b> Multiple different charset encodings in a same family may use
     * a single shared name in this implementation. For example, this method returns
     * an array including "ISO-8859-1" (ISO Latin 1), but not including "windows-1252"
     * (Windows Latin 1). However, actual detection result could be "windows-1252"
     * when the input data matches Latin 1 code points with any points only available
     * in "windows-1252".
     *
     * @return an array of the names of all charsets supported by
     * <code>CharsetDetector</code> class.
     * @stable ICU 3.4
     */
    // public static String[] getAllDetectableCharsets() {
    pub fn get_all_detectable_charsets() -> Vec<String> {
        // String[] allCharsetNames = new String[ALL_CS_RECOGNIZERS.size()];
        let mut all_charset_names: Vec<String> = Vec::new();
        let recognizers = all_cs_recognizers();
        // for (int i = 0; i < allCharsetNames.length; i++) {
        //     allCharsetNames[i] = ALL_CS_RECOGNIZERS.get(i).recognizer.getName();
        // }
        for rcinfo in recognizers.iter() {
            all_charset_names.push(rcinfo.recognizer.get_name());
        }
        // return allCharsetNames;
        all_charset_names
    }

    /**
     * Test whether or not input filtering is enabled.
     *
     * @return <code>true</code> if input text will be filtered.
     * @stable ICU 3.4
     * @see #enableInputFilter
     */
    // public boolean inputFilterEnabled() {
    //     return fStripTags;
    // }
    pub fn input_filter_enabled(&self) -> bool {
        self.f_strip_tags
    }

    /**
     * Enable filtering of input text. If filtering is enabled,
     * text within angle brackets ("&lt;" and "&gt;") will be removed
     * before detection.
     *
     * @param filter <code>true</code> to enable input text filtering.
     * @return The previous setting.
     * @stable ICU 3.4
     */
    // public boolean enableInputFilter(boolean filter) {
    //     boolean previous = fStripTags;
    //
    //     fStripTags = filter;
    //
    //     return previous;
    // }
    pub fn enable_input_filter(&mut self, filter: bool) -> bool {
        let previous = self.f_strip_tags;

        self.f_strip_tags = filter;

        previous
    }

    /*
     *  MungeInput - after getting a set of raw input data to be analyzed, preprocess
     *               it by removing what appears to be html markup.
     */
    // private void MungeInput() {
    pub(crate) fn munge_input(&mut self) {
        // int srci;
        let mut srci;
        let mut dsti = 0;
        let mut b: u8;
        let mut in_markup = false;
        let mut open_tags = 0;
        let mut bad_tags = 0;

        //
        //  html / xml markup stripping.
        //     quick and dirty, not 100% accurate, but hopefully good enough, statistically.
        //     discard everything within < brackets >
        //     Count how many total '<' and illegal (nested) '<' occur, so we can make some
        //     guess as to whether the input was actually marked up at all.
        // if (fStripTags) {
        if self.f_strip_tags {
            // for (srci = 0; srci < fRawLength && dsti < fInputBytes.length; srci++) {
            let mut i = 0;
            while i < self.f_raw_length as usize && dsti < self.f_input_bytes.len() {
                srci = i;
                b = self.f_raw_input.clone().unwrap()[srci];
                // if (b == (byte) '<') {
                if b == b'<' {
                    // if (inMarkup) {
                    //     badTags++;
                    // }
                    if in_markup {
                        bad_tags += 1;
                    }
                    in_markup = true;
                    open_tags += 1;
                }

                // if (!inMarkup) {
                //     fInputBytes[dsti++] = b;
                // }
                if !in_markup {
                    self.f_input_bytes[dsti] = b;
                    dsti += 1;
                }

                // if (b == (byte) '>') {
                //     inMarkup = false;
                // }
                if b == b'>' {
                    in_markup = false;
                }
                i += 1;
            }

            // fInputLen = dsti;
            self.f_input_len = dsti as i32;
        }

        //
        //  If it looks like this input wasn't marked up, or if it looks like it's
        //    essentially nothing but markup abandon the markup stripping.
        //    Detection will have to work on the unstripped input.
        //
        // if (openTags < 5 || openTags / 5 < badTags ||
        //         (fInputLen < 100 && fRawLength > 600)) {
        if open_tags < 5 || open_tags / 5 < bad_tags
            || (self.f_input_len < 100 && self.f_raw_length > 600)
        {
            let mut limit = self.f_raw_length as usize;

            // if (limit > kBufSize) {
            //     limit = kBufSize;
            // }
            if limit > K_BUF_SIZE {
                limit = K_BUF_SIZE;
            }

            // for (srci = 0; srci < limit; srci++) {
            //     fInputBytes[srci] = fRawInput[srci];
            // }
            let raw = self.f_raw_input.clone().unwrap_or_default();
            for srci in 0..limit {
                self.f_input_bytes[srci] = raw[srci];
            }
            // fInputLen = srci;
            self.f_input_len = limit as i32;
        }

        //
        // Tally up the byte occurence statistics.
        //   These are available for use by the various detectors.
        //
        // Arrays.fill(fByteStats, (short) 0);
        self.f_byte_stats = vec![0_u16; 256];
        // for (srci = 0; srci < fInputLen; srci++) {
        //     int val = fInputBytes[srci] & 0x00ff;
        //     fByteStats[val]++;
        // }
        for srci in 0..self.f_input_len as usize {
            let val = self.f_input_bytes[srci] & 0x00ff;
            self.f_byte_stats[val as usize] += 1;
        }

        // fC1Bytes = false;
        self.f_c1_bytes = false;
        // for (int i = 0x80; i <= 0x9F; i += 1) {
        //     if (fByteStats[i] != 0) {
        //         fC1Bytes = true;
        //         break;
        //     }
        // }
        for i in 0x80..=0x9F {
            if self.f_byte_stats[i] != 0 {
                self.f_c1_bytes = true;
                break;
            }
        }
    }

    /**
     * Get the names of charsets that can be recognized by this CharsetDetector instance.
     *
     * @return an array of the names of charsets that can be recognized by this CharsetDetector
     * instance.
     * @internal
     * @deprecated This API is ICU internal only.
     */
    // @Deprecated
    // public String[] getDetectableCharsets() {
    #[deprecated(note = "This API is ICU internal only.")]
    pub fn get_detectable_charsets(&self) -> Vec<String> {
        // List<String> csnames = new ArrayList<>(ALL_CS_RECOGNIZERS.size());
        let mut csnames: Vec<String> = Vec::new();
        let recognizers = all_cs_recognizers();
        // for (int i = 0; i < ALL_CS_RECOGNIZERS.size(); i++) {
        //     CSRecognizerInfo rcinfo = ALL_CS_RECOGNIZERS.get(i);
        //     boolean active = (fEnabledRecognizers == None) ? rcinfo.isDefaultEnabled : fEnabledRecognizers[i];
        //     if (active) {
        //         csnames.add(rcinfo.recognizer.getName());
        //     }
        // }
        for (i, rcinfo) in recognizers.iter().enumerate() {
            // boolean active = (fEnabledRecognizers == None) ? rcinfo.isDefaultEnabled : fEnabledRecognizers[i];
            let active = match &self.f_enabled_recognizers {
                None => rcinfo.is_default_enabled,
                Some(enabled) => enabled[i],
            };
            if active {
                csnames.push(rcinfo.recognizer.get_name());
            }
        }
        // return csnames.toArray(new String[0]);
        csnames
    }

    /**
     * Enable or disable individual charset encoding.
     * A name of charset encoding must be included in the names returned by
     * {@link #getAllDetectableCharsets()}.
     *
     * @param encoding the name of charset encoding.
     * @param enabled  <code>true</code> to enable, or <code>false</code> to disable the
     *                 charset encoding.
     * @return A reference to this <code>CharsetDetector</code>.
     * @throws IllegalArgumentException when the name of charset encoding is
     *                                  not supported.
     * @internal
     * @deprecated This API is ICU internal only.
     */
    // @Deprecated
    // public CharsetDetector setDetectableCharset(String encoding, boolean enabled) {
    #[deprecated(note = "This API is ICU internal only.")]
    pub fn set_detectable_charset(&mut self, encoding: &str, enabled: bool) -> Result<&mut CharsetDetector, String> {
        let mut mod_idx = -1;
        let mut is_default_val = false;
        let recognizers = all_cs_recognizers();
        // for (int i = 0; i < ALL_CS_RECOGNIZERS.size(); i++) {
        //     CSRecognizerInfo csrinfo = ALL_CS_RECOGNIZERS.get(i);
        //     if (csrinfo.recognizer.getName().equals(encoding)) {
        //         modIdx = i;
        //         isDefaultVal = (csrinfo.isDefaultEnabled == enabled);
        //         break;
        //     }
        // }
        for (i, csrinfo) in recognizers.iter().enumerate() {
            if csrinfo.recognizer.get_name() == encoding {
                mod_idx = i as i32;
                is_default_val = (csrinfo.is_default_enabled == enabled);
                break;
            }
        }
        // if (modIdx < 0) {
        //     // No matching encoding found
        //     throw new IllegalArgumentException("Invalid encoding: " + "\"" + encoding + "\"");
        // }
        if mod_idx < 0 {
            // No matching encoding found
            return Err(format!("Invalid encoding: {}{}{}", "\"", encoding, "\""));
        }

        // if (fEnabledRecognizers == None && !isDefaultVal) {
        //     // Create an array storing the non default setting
        //     fEnabledRecognizers = new boolean[ALL_CS_RECOGNIZERS.size()];
        //
        //     // Initialize the array with default info
        //     for (int i = 0; i < ALL_CS_RECOGNIZERS.size(); i++) {
        //         fEnabledRecognizers[i] = ALL_CS_RECOGNIZERS.get(i).isDefaultEnabled;
        //     }
        // }
        if self.f_enabled_recognizers.is_none() && !is_default_val {
            // Create an array storing the non default setting
            let mut enabled_recognizers: Vec<bool> = Vec::new();

            // Initialize the array with default info
            for rcinfo in recognizers.iter() {
                enabled_recognizers.push(rcinfo.is_default_enabled);
            }
            self.f_enabled_recognizers = Some(enabled_recognizers);
        }

        // if (fEnabledRecognizers != None) {
        //     fEnabledRecognizers[modIdx] = enabled;
        // }
        if let Some(enabled_recognizers) = self.f_enabled_recognizers.as_mut() {
            enabled_recognizers[mod_idx as usize] = enabled;
        }

        Ok(self)
    }
}

// private static class CSRecognizerInfo {
//     CharsetRecognizer recognizer;
//     boolean isDefaultEnabled;
//
//     CSRecognizerInfo(CharsetRecognizer recognizer, boolean isDefaultEnabled) {
//         this.recognizer = recognizer;
//         this.isDefaultEnabled = isDefaultEnabled;
//     }
// }
pub(crate) struct CSRecognizerInfo {
    // fix: 显式 +Send +Sync，满足 OnceLock<Vec<CSRecognizerInfo>> 的 Sync 约束（E0277）
    pub(crate) recognizer: Box<dyn CharsetRecognizer + Send + Sync>,
    pub(crate) is_default_enabled: bool,
}

impl CSRecognizerInfo {
    pub(crate) fn new(recognizer: Box<dyn CharsetRecognizer + Send + Sync>, is_default_enabled: bool) -> CSRecognizerInfo {
        CSRecognizerInfo {
            recognizer,
            is_default_enabled,
        }
    }
}

// fix: 原 CharsetRecog_sbcs 转录未包含的 6 个识别器（启发式：字节分布 + 解码质量）
//      单字节字符集识别——统计非 ASCII 字节中落在该语言高频字节区的占比，扣减解码质量罚分
fn sbcs_match(
    name: &str,
    enc_label: Option<&str>,
    common_bytes: &[u8],
    det: &CharsetDetector,
    self_ref: &dyn CharsetRecognizer,
) -> Option<CharsetMatch> {
    let stats = &det.f_byte_stats;
    let mut non_ascii: u32 = 0;
    let mut common: u32 = 0;
    for b in 0x80u16..256 {
        let cnt = stats[b as usize] as u32;
        if cnt > 0 {
            non_ascii += cnt;
            if common_bytes.contains(&(b as u8)) {
                common += cnt;
            }
        }
    }
    if non_ascii == 0 {
        return None;
    }
    let mut quality_penalty: i32 = 0;
    if let Some(label) = enc_label {
        if let Some(enc) = encoding_rs::Encoding::for_label(label.as_bytes()) {
            let input = &det.f_input_bytes[..det.f_input_len.max(0) as usize];
            let (_, _, had_errors) = enc.decode(input);
            if had_errors {
                quality_penalty = 35;
            }
        }
    }
    let ratio = (common * 100 / non_ascii) as i32;
    let confidence = ratio - quality_penalty;
    if confidence <= 0 {
        return None;
    }
    Some(CharsetMatch::new(det, self_ref, confidence))
}

pub(crate) struct CharsetRecog_windows_1256;
impl CharsetRecognizer for CharsetRecog_windows_1256 {
    fn get_name(&self) -> String {
        "windows-1256".to_string()
    }
    fn match_det(&self, det: &CharsetDetector) -> Option<CharsetMatch> {
        // 阿拉伯字母位于 1256 的 0xC8-0xFF 区（0xC0-0xC7 是拉丁，阿拉伯文本不出现）
        let common: Vec<u8> = (0xC8u8..=0xFF).collect();
        sbcs_match("windows-1256", Some("windows-1256"), &common, det, self)
    }
}

pub(crate) struct CharsetRecog_KOI8_R;
impl CharsetRecognizer for CharsetRecog_KOI8_R {
    fn get_name(&self) -> String {
        "KOI8-R".to_string()
    }
    fn match_det(&self, det: &CharsetDetector) -> Option<CharsetMatch> {
        // KOI8-R 西里尔：小写 0xC0-0xDF、大写 0xE0-0xFF
        let mut common: Vec<u8> = (0xC0u8..=0xDF).collect();
        common.extend(0xE0u8..=0xFF);
        sbcs_match("KOI8-R", Some("KOI8-R"), &common, det, self)
    }
}

pub(crate) struct CharsetRecog_IBM424_he_rtl;
impl CharsetRecognizer for CharsetRecog_IBM424_he_rtl {
    fn get_name(&self) -> String {
        "IBM424_rtl".to_string()
    }
    fn match_det(&self, det: &CharsetDetector) -> Option<CharsetMatch> {
        // EBCDIC 希伯来字母区 0xE0-0xFA（无 encoding_rs 支持，纯分布且置信度压低）
        let common: Vec<u8> = (0xE0u8..=0xFA).collect();
        sbcs_match("IBM424_rtl", None, &common, det, self)
            .filter(|m| m.get_confidence() >= 45)
    }
}

pub(crate) struct CharsetRecog_IBM424_he_ltr;
impl CharsetRecognizer for CharsetRecog_IBM424_he_ltr {
    fn get_name(&self) -> String {
        "IBM424_ltr".to_string()
    }
    fn match_det(&self, det: &CharsetDetector) -> Option<CharsetMatch> {
        let common: Vec<u8> = (0xE0u8..=0xFA).collect();
        sbcs_match("IBM424_ltr", None, &common, det, self)
            .filter(|m| m.get_confidence() >= 45)
    }
}

pub(crate) struct CharsetRecog_IBM420_ar_rtl;
impl CharsetRecognizer for CharsetRecog_IBM420_ar_rtl {
    fn get_name(&self) -> String {
        "IBM420_rtl".to_string()
    }
    fn match_det(&self, det: &CharsetDetector) -> Option<CharsetMatch> {
        // EBCDIC 阿拉伯 0x80-0xFA（无 encoding_rs 支持，纯分布且置信度压低）
        let common: Vec<u8> = (0x80u8..=0xFA).collect();
        sbcs_match("IBM420_rtl", None, &common, det, self)
            .filter(|m| m.get_confidence() >= 45)
    }
}

pub(crate) struct CharsetRecog_IBM420_ar_ltr;
impl CharsetRecognizer for CharsetRecog_IBM420_ar_ltr {
    fn get_name(&self) -> String {
        "IBM420_ltr".to_string()
    }
    fn match_det(&self, det: &CharsetDetector) -> Option<CharsetMatch> {
        let common: Vec<u8> = (0x80u8..=0xFA).collect();
        sbcs_match("IBM420_ltr", None, &common, det, self)
            .filter(|m| m.get_confidence() >= 45)
    }
}
