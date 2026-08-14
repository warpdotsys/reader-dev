use crate::prelude::*;
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

use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use crate::me::ag2s::epublib::util::commons::io::{BOMInputStream, ByteOrderMark, InputStream, XmlStreamReaderException};
use crate::me::ag2s::epublib::util::IOUtil;
use crate::stubs::File;

/**
 * Character stream that handles all the necessary Voodoo to figure out the
 * charset encoding of the XML document within the stream.
 * <p>
 * IMPORTANT: This class is not related in any way to the org.xml.sax.XMLReader.
 * This one IS a character stream.
 * </p>
 * <p>
 * All this has to be done without consuming characters from the stream, if not
 * the XML parser will not recognized the document as a valid XML. This is not
 * 100% true, but it's close enough (UTF-8 BOM is not handled by all parsers
 * right now, XmlStreamReader handles it and things work in all parsers).
 * </p>
 * <p>
 * The XmlStreamReader class handles the charset encoding of XML documents in
 * Files, raw streams and HTTP streams by offering a wide set of constructors.
 * </p>
 * <p>
 * By default the charset encoding detection is lenient, the constructor with
 * the lenient flag can be used for a script (following HTTP MIME and XML
 * specifications). All this is nicely explained by Mark Pilgrim in his blog, <a
 * href="http://diveintomark.org/archives/2004/02/13/xml-media-types">
 * Determining the character encoding of a feed</a>.
 * </p>
 * <p>
 * Originally developed for <a href="http://rome.dev.java.net">ROME</a> under
 * Apache License 2.0.
 * </p>
 *
 * //@seerr XmlStreamWriter
 * @since 2.0
 */
pub struct XmlStreamReader {
    reader: InputStreamReader,
    encoding: String,
    default_encoding: Option<String>,
}

impl XmlStreamReader {

    const BUFFER_SIZE: usize = IOUtil::DEFAULT_BUFFER_SIZE;

    const UTF_8: &'static str = "UTF-8";

    const US_ASCII: &'static str = "US-ASCII";

    const UTF_16BE: &'static str = "UTF-16BE";

    const UTF_16LE: &'static str = "UTF-16LE";

    const UTF_32BE: &'static str = "UTF-32BE";

    const UTF_32LE: &'static str = "UTF-32LE";

    const UTF_16: &'static str = "UTF-16";

    const UTF_32: &'static str = "UTF-32";

    const EBCDIC: &'static str = "CP1047";

    const BOMS: [ByteOrderMark; 5] = [
        ByteOrderMark::UTF_8,
        ByteOrderMark::UTF_16BE,
        ByteOrderMark::UTF_16LE,
        ByteOrderMark::UTF_32BE,
        ByteOrderMark::UTF_32LE
    ];

    // UTF_16LE and UTF_32LE have the same two starting BOM bytes.
    const XML_GUESS_BYTES: [ByteOrderMark; 6] = [
        ByteOrderMark::new(XmlStreamReader::UTF_8, &[0x3C, 0x3F, 0x78, 0x6D]),
        ByteOrderMark::new(XmlStreamReader::UTF_16BE, &[0x00, 0x3C, 0x00, 0x3F]),
        ByteOrderMark::new(XmlStreamReader::UTF_16LE, &[0x3C, 0x00, 0x3F, 0x00]),
        ByteOrderMark::new(XmlStreamReader::UTF_32BE, &[0x00, 0x00, 0x00, 0x3C,
            0x00, 0x00, 0x00, 0x3F, 0x00, 0x00, 0x00, 0x78, 0x00, 0x00, 0x00, 0x6D]),
        ByteOrderMark::new(XmlStreamReader::UTF_32LE, &[0x3C, 0x00, 0x00, 0x00,
            0x3F, 0x00, 0x00, 0x00, 0x78, 0x00, 0x00, 0x00, 0x6D, 0x00, 0x00, 0x00]),
        ByteOrderMark::new(XmlStreamReader::EBCDIC, &[0x4C, 0x6F, 0xA7, 0x94])
    ];

    /**
     * Returns the default encoding to use if none is set in HTTP content-type,
     * XML prolog and the rules based on content-type are not adequate.
     * <p>
     * If it is None the content-type based rules are used.
     *
     * @return the default encoding to use.
     */
    pub fn get_default_encoding(&self) -> &Option<String> {
        &self.default_encoding
    }

    /**
     * Creates a Reader for a File.
     * <p>
     * It looks for the UTF-8 BOM first, if none sniffs the XML prolog charset,
     * if this is also missing defaults to UTF-8.
     * <p>
     * It does a lenient charset encoding detection, check the constructor with
     * the lenient parameter for details.
     *
     * @param file File to create a Reader from.
     * @throws IOException thrown if there is a problem reading the file.
     */
    #[allow(dead_code)]
    pub fn new_from_file(file: File) -> Result<Self, XmlStreamReaderException> {
        XmlStreamReader::new(FileInputStream::new(file))
    }

    /**
     * Creates a Reader for a raw InputStream.
     * <p>
     * It follows the same logic used for files.
     * <p>
     * It does a lenient charset encoding detection, check the constructor with
     * the lenient parameter for details.
     *
     * @param inputStream InputStream to create a Reader from.
     * @throws IOException thrown if there is a problem reading the stream.
     */
    pub fn new(input_stream: Box<dyn InputStream>) -> Result<Self, XmlStreamReaderException> {
        XmlStreamReader::new_lenient(input_stream, true)
    }

    /**
     * Creates a Reader for a raw InputStream.
     * <p>
     * It follows the same logic used for files.
     * <p>
     * If lenient detection is indicated and the detection above fails as per
     * specifications it then attempts the following:
     * <p>
     * If the content type was 'text/html' it replaces it with 'text/xml' and
     * tries the detection again.
     * <p>
     * Else if the XML prolog had a charset encoding that encoding is used.
     * <p>
     * Else if the content type had a charset encoding that encoding is used.
     * <p>
     * Else 'UTF-8' is used.
     * <p>
     * If lenient detection is indicated an XmlStreamReaderException is never
     * thrown.
     *
     * @param inputStream InputStream to create a Reader from.
     * @param lenient indicates if the charset encoding detection should be
     *        relaxed.
     * @throws IOException thrown if there is a problem reading the stream.
     * @throws XmlStreamReaderException thrown if the charset encoding could not
     *         be determined according to the specs.
     */
    pub fn new_lenient(input_stream: Box<dyn InputStream>, lenient: bool) -> Result<Self, XmlStreamReaderException> {
        XmlStreamReader::new_lenient_default(input_stream, lenient, None)
    }

    /**
     * Creates a Reader for a raw InputStream.
     * <p>
     * It follows the same logic used for files.
     * <p>
     * If lenient detection is indicated and the detection above fails as per
     * specifications it then attempts the following:
     * <p>
     * If the content type was 'text/html' it replaces it with 'text/xml' and
     * tries the detection again.
     * <p>
     * Else if the XML prolog had a charset encoding that encoding is used.
     * <p>
     * Else if the content type had a charset encoding that encoding is used.
     * <p>
     * Else 'UTF-8' is used.
     * <p>
     * If lenient detection is indicated an XmlStreamReaderException is never
     * thrown.
     *
     * @param inputStream InputStream to create a Reader from.
     * @param lenient indicates if the charset encoding detection should be
     *        relaxed.
     * @param defaultEncoding The default encoding
     * @throws IOException thrown if there is a problem reading the stream.
     * @throws XmlStreamReaderException thrown if the charset encoding could not
     *         be determined according to the specs.
     */
    pub fn new_lenient_default(input_stream: Box<dyn InputStream>, lenient: bool, default_encoding: Option<String>) -> Result<Self, XmlStreamReaderException> {
        let mut bom = BOMInputStream::new_boms(Box::new(BufferedInputStream::new(input_stream, XmlStreamReader::BUFFER_SIZE)), false, XmlStreamReader::BOMS.to_vec());
        let bom_shared = Rc::new(RefCell::new(bom));
        let mut pis = BOMInputStream::new_boms(Box::new(BomDelegate(bom_shared.clone())), true, XmlStreamReader::XML_GUESS_BYTES.to_vec());
        let encoding = Self::do_raw_stream(&default_encoding, &bom_shared, &mut pis, lenient)?;
        Ok(XmlStreamReader {
            reader: InputStreamReader::new(pis, encoding.clone()),
            encoding,
            default_encoding,
        })
    }

    /**
     * Creates a Reader using the InputStream of a URL.
     * <p>
     * If the URL is not of type HTTP and there is not 'content-type' header in
     * the fetched data it uses the same logic used for Files.
     * <p>
     * If the URL is a HTTP Url or there is a 'content-type' header in the
     * fetched data it uses the same logic used for an InputStream with
     * content-type.
     * <p>
     * It does a lenient charset encoding detection, check the constructor with
     * the lenient parameter for details.
     *
     * @param url URL to create a Reader from.
     * @throws IOException thrown if there is a problem reading the stream of
     *         the URL.
     */
    #[allow(dead_code)]
    pub fn new_from_url(url: URL) -> Result<Self, XmlStreamReaderException> {
        XmlStreamReader::new_from_connection(url.open_connection(), None)
    }

    /**
     * Creates a Reader using the InputStream of a URLConnection.
     * <p>
     * If the URLConnection is not of type HttpURLConnection and there is not
     * 'content-type' header in the fetched data it uses the same logic used for
     * files.
     * <p>
     * If the URLConnection is a HTTP Url or there is a 'content-type' header in
     * the fetched data it uses the same logic used for an InputStream with
     * content-type.
     * <p>
     * It does a lenient charset encoding detection, check the constructor with
     * the lenient parameter for details.
     *
     * @param conn URLConnection to create a Reader from.
     * @param defaultEncoding The default encoding
     * @throws IOException thrown if there is a problem reading the stream of
     *         the URLConnection.
     */
    pub fn new_from_connection(conn: URLConnection, default_encoding: Option<String>) -> Result<Self, XmlStreamReaderException> {
        let lenient = true;
        let content_type = conn.get_content_type();
        let input_stream = conn.get_input_stream();
        let mut bom = BOMInputStream::new_boms(Box::new(BufferedInputStream::new(input_stream, XmlStreamReader::BUFFER_SIZE)), false, XmlStreamReader::BOMS.to_vec());
        let bom_shared = Rc::new(RefCell::new(bom));
        let mut pis = BOMInputStream::new_boms(Box::new(BomDelegate(bom_shared.clone())), true, XmlStreamReader::XML_GUESS_BYTES.to_vec());
        let encoding = if conn.is_http_url_connection() || content_type.is_some() {
            Self::process_http_stream(&default_encoding, &bom_shared, &mut pis, content_type.unwrap_or_default(), lenient)?
        } else {
            Self::do_raw_stream(&default_encoding, &bom_shared, &mut pis, lenient)?
        };
        Ok(XmlStreamReader {
            reader: InputStreamReader::new(pis, encoding.clone()),
            encoding,
            default_encoding,
        })
    }

    /**
     * Creates a Reader using an InputStream and the associated content-type
     * header.
     * <p>
     * First it checks if the stream has BOM. If there is not BOM checks the
     * content-type encoding. If there is not content-type encoding checks the
     * XML prolog encoding. If there is not XML prolog encoding uses the default
     * encoding mandated by the content-type MIME type.
     * <p>
     * It does a lenient charset encoding detection, check the constructor with
     * the lenient parameter for details.
     *
     * @param inputStream InputStream to create the reader from.
     * @param httpContentType content-type header to use for the resolution of
     *        the charset encoding.
     * @throws IOException thrown if there is a problem reading the file.
     */
    pub fn new_content_type(input_stream: Box<dyn InputStream>, http_content_type: String) -> Result<Self, XmlStreamReaderException> {
        XmlStreamReader::new_content_type_lenient(input_stream, http_content_type, true)
    }

    /**
     * Creates a Reader using an InputStream and the associated content-type
     * header. This constructor is lenient regarding the encoding detection.
     * <p>
     * First it checks if the stream has BOM. If there is not BOM checks the
     * content-type encoding. If there is not content-type encoding checks the
     * XML prolog encoding. If there is not XML prolog encoding uses the default
     * encoding mandated by the content-type MIME type.
     * <p>
     * If lenient detection is indicated and the detection above fails as per
     * specifications it then attempts the following:
     * <p>
     * If the content type was 'text/html' it replaces it with 'text/xml' and
     * tries the detection again.
     * <p>
     * Else if the XML prolog had a charset encoding that encoding is used.
     * <p>
     * Else if the content type had a charset encoding that encoding is used.
     * <p>
     * Else 'UTF-8' is used.
     * <p>
     * If lenient detection is indicated an XmlStreamReaderException is never
     * thrown.
     *
     * @param inputStream InputStream to create the reader from.
     * @param httpContentType content-type header to use for the resolution of
     *        the charset encoding.
     * @param lenient indicates if the charset encoding detection should be
     *        relaxed.
     * @param defaultEncoding The default encoding
     * @throws IOException thrown if there is a problem reading the file.
     * @throws XmlStreamReaderException thrown if the charset encoding could not
     *         be determined according to the specs.
     */
    pub fn new_content_type_lenient_default(input_stream: Box<dyn InputStream>, http_content_type: String,
                                            lenient: bool, default_encoding: Option<String>) -> Result<Self, XmlStreamReaderException> {
        let mut bom = BOMInputStream::new_boms(Box::new(BufferedInputStream::new(input_stream, XmlStreamReader::BUFFER_SIZE)), false, XmlStreamReader::BOMS.to_vec());
        let bom_shared = Rc::new(RefCell::new(bom));
        let mut pis = BOMInputStream::new_boms(Box::new(BomDelegate(bom_shared.clone())), true, XmlStreamReader::XML_GUESS_BYTES.to_vec());
        let encoding = Self::process_http_stream(&default_encoding, &bom_shared, &mut pis, http_content_type, lenient)?;
        Ok(XmlStreamReader {
            reader: InputStreamReader::new(pis, encoding.clone()),
            encoding,
            default_encoding,
        })
    }

    /**
     * Creates a Reader using an InputStream and the associated content-type
     * header. This constructor is lenient regarding the encoding detection.
     * <p>
     * First it checks if the stream has BOM. If there is not BOM checks the
     * content-type encoding. If there is not content-type encoding checks the
     * XML prolog encoding. If there is not XML prolog encoding uses the default
     * encoding mandated by the content-type MIME type.
     * <p>
     * If lenient detection is indicated and the detection above fails as per
     * specifications it then attempts the following:
     * <p>
     * If the content type was 'text/html' it replaces it with 'text/xml' and
     * tries the detection again.
     * <p>
     * Else if the XML prolog had a charset encoding that encoding is used.
     * <p>
     * Else if the content type had a charset encoding that encoding is used.
     * <p>
     * Else 'UTF-8' is used.
     * <p>
     * If lenient detection is indicated an XmlStreamReaderException is never
     * thrown.
     *
     * @param inputStream InputStream to create the reader from.
     * @param httpContentType content-type header to use for the resolution of
     *        the charset encoding.
     * @param lenient indicates if the charset encoding detection should be
     *        relaxed.
     * @throws IOException thrown if there is a problem reading the file.
     * @throws XmlStreamReaderException thrown if the charset encoding could not
     *         be determined according to the specs.
     */
    pub fn new_content_type_lenient(input_stream: Box<dyn InputStream>, http_content_type: String,
                                    lenient: bool) -> Result<Self, XmlStreamReaderException> {
        XmlStreamReader::new_content_type_lenient_default(input_stream, http_content_type, lenient, None)
    }

    /**
     * Returns the charset encoding of the XmlStreamReader.
     *
     * @return charset encoding.
     */
    pub fn get_encoding(&self) -> &str {
        &self.encoding
    }

    /**
     * Invokes the underlying reader's <code>read(char[], int, int)</code> method.
     * @param buf the buffer to read the characters into
     * @param offset The start offset
     * @param len The number of bytes to read
     * @return the number of characters read or -1 if the end of stream
     * @throws IOException if an I/O error occurs
     */
    pub fn read(&mut self, buf: &mut [char], offset: usize, len: usize) -> Result<i32, io::Error> {
        self.reader.read(buf, offset, len)
    }

    /**
     * Closes the XmlStreamReader stream.
     *
     * @throws IOException thrown if there was a problem closing the stream.
     */
    pub fn close(&mut self) -> Result<(), io::Error> {
        self.reader.close()
    }

    /**
     * Process the raw stream.
     *
     * @param bom BOMInputStream to detect byte order marks
     * @param pis BOMInputStream to guess XML encoding
     * @param lenient indicates if the charset encoding detection should be
     *        relaxed.
     * @return the encoding to be used
     * @throws IOException thrown if there is a problem reading the stream.
     */
    fn do_raw_stream(default_encoding: &Option<String>, bom: &Rc<RefCell<BOMInputStream>>, pis: &mut BOMInputStream, lenient: bool) -> Result<String, XmlStreamReaderException> {
        let bom_enc = bom.borrow_mut().get_bom_charset_name().map_err(io_err)?;
        let xml_guess_enc = pis.get_bom_charset_name().map_err(io_err)?;
        let xml_enc = Self::get_xml_prolog(pis, xml_guess_enc.as_deref())?;
        match Self::calculate_raw_encoding(default_encoding, bom_enc.as_deref(), xml_guess_enc.as_deref(), xml_enc.as_deref()) {
            Ok(enc) => Ok(enc),
            Err(ex) => {
                if lenient {
                    Self::do_lenient_detection(default_encoding, None, ex, bom_enc, xml_guess_enc, xml_enc)
                } else {
                    Err(ex)
                }
            }
        }
    }

    /**
     * Process a HTTP stream.
     *
     * @param bom BOMInputStream to detect byte order marks
     * @param pis BOMInputStream to guess XML encoding
     * @param httpContentType The HTTP content type
     * @param lenient indicates if the charset encoding detection should be
     *        relaxed.
     * @return the encoding to be used
     * @throws IOException thrown if there is a problem reading the stream.
     */
    fn process_http_stream(default_encoding: &Option<String>, bom: &Rc<RefCell<BOMInputStream>>, pis: &mut BOMInputStream, http_content_type: String,
                           lenient: bool) -> Result<String, XmlStreamReaderException> {
        let bom_enc = bom.borrow_mut().get_bom_charset_name().map_err(io_err)?;
        let xml_guess_enc = pis.get_bom_charset_name().map_err(io_err)?;
        let xml_enc = Self::get_xml_prolog(pis, xml_guess_enc.as_deref())?;
        match Self::calculate_http_encoding(default_encoding, http_content_type.as_str(), bom_enc.as_deref(), xml_guess_enc.as_deref(), xml_enc.as_deref(), lenient) {
            Ok(enc) => Ok(enc),
            Err(ex) => {
                if lenient {
                    Self::do_lenient_detection(default_encoding, Some(http_content_type), ex, bom_enc, xml_guess_enc, xml_enc)
                } else {
                    Err(ex)
                }
            }
        }
    }

    /**
     * Do lenient detection.
     *
     * @param httpContentType content-type header to use for the resolution of
     *        the charset encoding.
     * @param ex The thrown exception
     * @return the encoding
     * @throws IOException thrown if there is a problem reading the stream.
     */
    fn do_lenient_detection(default_encoding: &Option<String>, mut http_content_type: Option<String>, mut ex: XmlStreamReaderException,
                            bom_enc: Option<String>, xml_guess_enc: Option<String>, xml_enc: Option<String>) -> Result<String, XmlStreamReaderException> {
        if http_content_type.is_some() && http_content_type.as_ref().unwrap().starts_with("text/html") {
            let mut hct = http_content_type.as_ref().unwrap().clone();
            hct = hct["text/html".len()..].to_string();
            hct = "text/xml".to_string() + &hct;
            match Self::calculate_http_encoding(default_encoding, &hct, ex.get_bom_encoding().as_deref(),
                                                ex.get_xml_guess_encoding().as_deref(), ex.get_xml_encoding().as_deref(), true) {
                Ok(enc) => return Ok(enc),
                Err(ex2) => {
                    ex = ex2;
                }
            }
        }
        let mut encoding = ex.get_xml_encoding().clone();
        if encoding.is_none() {
            encoding = ex.get_content_type_encoding().clone();
        }
        if encoding.is_none() {
            encoding = if default_encoding.is_none() { Some(XmlStreamReader::UTF_8.to_string()) } else { default_encoding.clone() };
        }
        Ok(encoding.unwrap())
    }

    /**
     * Calculate the raw encoding.
     *
     * @param bomEnc BOM encoding
     * @param xmlGuessEnc XML Guess encoding
     * @param xmlEnc XML encoding
     * @return the raw encoding
     * @throws IOException thrown if there is a problem reading the stream.
     */
    fn calculate_raw_encoding(default_encoding: &Option<String>, bom_enc: Option<&str>, xml_guess_enc: Option<&str>,
                              xml_enc: Option<&str>) -> Result<String, XmlStreamReaderException> {

        // BOM is None
        if bom_enc.is_none() {
            if xml_guess_enc.is_none() || xml_enc.is_none() {
                return Ok(if default_encoding.is_none() { XmlStreamReader::UTF_8.to_string() } else { default_encoding.clone().unwrap() });
            }
            if xml_enc.unwrap() == XmlStreamReader::UTF_16 &&
                (xml_guess_enc.unwrap() == XmlStreamReader::UTF_16BE || xml_guess_enc.unwrap() == XmlStreamReader::UTF_16LE) {
                return Ok(xml_guess_enc.unwrap().to_string());
            }
            return Ok(xml_enc.unwrap().to_string());
        }

        // BOM is UTF-8
        if bom_enc.unwrap() == XmlStreamReader::UTF_8 {
            if xml_guess_enc.is_some() && xml_guess_enc.unwrap() != XmlStreamReader::UTF_8 {
                let msg = message_format(XmlStreamReader::RAW_EX_1, vec![bom_enc.unwrap().to_string(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()]);
                return Err(XmlStreamReaderException::new_full(&msg, None, None, bom_enc.unwrap().to_string(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()));
            }
            if xml_enc.is_some() && xml_enc.unwrap() != XmlStreamReader::UTF_8 {
                let msg = message_format(XmlStreamReader::RAW_EX_1, vec![bom_enc.unwrap().to_string(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()]);
                return Err(XmlStreamReaderException::new_full(&msg, None, None, bom_enc.unwrap().to_string(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()));
            }
            return Ok(bom_enc.unwrap().to_string());
        }

        // BOM is UTF-16BE or UTF-16LE
        if bom_enc.unwrap() == XmlStreamReader::UTF_16BE || bom_enc.unwrap() == XmlStreamReader::UTF_16LE {
            if xml_guess_enc.is_some() && xml_guess_enc.unwrap() != bom_enc.unwrap() {
                let msg = message_format(XmlStreamReader::RAW_EX_1, vec![bom_enc.unwrap().to_string(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()]);
                return Err(XmlStreamReaderException::new_full(&msg, None, None, bom_enc.unwrap().to_string(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()));
            }
            if xml_enc.is_some() && xml_enc.unwrap() != XmlStreamReader::UTF_16 && xml_enc.unwrap() != bom_enc.unwrap() {
                let msg = message_format(XmlStreamReader::RAW_EX_1, vec![bom_enc.unwrap().to_string(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()]);
                return Err(XmlStreamReaderException::new_full(&msg, None, None, bom_enc.unwrap().to_string(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()));
            }
            return Ok(bom_enc.unwrap().to_string());
        }

        // BOM is UTF-32BE or UTF-32LE
        if bom_enc.unwrap() == XmlStreamReader::UTF_32BE || bom_enc.unwrap() == XmlStreamReader::UTF_32LE {
            if xml_guess_enc.is_some() && xml_guess_enc.unwrap() != bom_enc.unwrap() {
                let msg = message_format(XmlStreamReader::RAW_EX_1, vec![bom_enc.unwrap().to_string(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()]);
                return Err(XmlStreamReaderException::new_full(&msg, None, None, bom_enc.unwrap().to_string(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()));
            }
            if xml_enc.is_some() && xml_enc.unwrap() != XmlStreamReader::UTF_32 && xml_enc.unwrap() != bom_enc.unwrap() {
                let msg = message_format(XmlStreamReader::RAW_EX_1, vec![bom_enc.unwrap().to_string(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()]);
                return Err(XmlStreamReaderException::new_full(&msg, None, None, bom_enc.unwrap().to_string(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()));
            }
            return Ok(bom_enc.unwrap().to_string());
        }

        // BOM is something else
        let msg = message_format(XmlStreamReader::RAW_EX_2, vec![bom_enc.unwrap().to_string(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()]);
        Err(XmlStreamReaderException::new_full(&msg, None, None, bom_enc.unwrap().to_string(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()))
    }

    /**
     * Calculate the HTTP encoding.
     *
     * @param httpContentType The HTTP content type
     * @param bomEnc BOM encoding
     * @param xmlGuessEnc XML Guess encoding
     * @param xmlEnc XML encoding
     * @param lenient indicates if the charset encoding detection should be
     *        relaxed.
     * @return the HTTP encoding
     * @throws IOException thrown if there is a problem reading the stream.
     */
    fn calculate_http_encoding(default_encoding: &Option<String>, http_content_type: &str,
                               bom_enc: Option<&str>, xml_guess_enc: Option<&str>, xml_enc: Option<&str>,
                               lenient: bool) -> Result<String, XmlStreamReaderException> {

        // Lenient and has XML encoding
        if lenient && xml_enc.is_some() {
            return Ok(xml_enc.unwrap().to_string());
        }

        // Determine mime/encoding content types from HTTP Content Type
        let c_t_mime = Self::get_content_type_mime(http_content_type);
        let c_t_enc = Self::get_content_type_encoding(http_content_type);
        let app_xml = Self::is_app_xml(c_t_mime.as_deref());
        let text_xml = Self::is_text_xml(c_t_mime.as_deref());

        // Mime type NOT "application/xml" or "text/xml"
        if !app_xml && !text_xml {
            let msg = message_format(XmlStreamReader::HTTP_EX_3, vec![c_t_mime.clone().unwrap_or_default(), c_t_enc.clone().unwrap_or_default(), bom_enc.map(|s| s.to_string()).unwrap_or_default(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()]);
            return Err(XmlStreamReaderException::new_full(&msg, c_t_mime, c_t_enc, bom_enc.map(|s| s.to_string()).unwrap_or_default(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()));
        }

        // No content type encoding
        if c_t_enc.is_none() {
            if app_xml {
                return Self::calculate_raw_encoding(default_encoding, bom_enc, xml_guess_enc, xml_enc);
            }
            return Ok(if default_encoding.is_none() { XmlStreamReader::US_ASCII.to_string() } else { default_encoding.clone().unwrap() });
        }

        // UTF-16BE or UTF-16LE content type encoding
        if c_t_enc.as_ref().unwrap() == XmlStreamReader::UTF_16BE || c_t_enc.as_ref().unwrap() == XmlStreamReader::UTF_16LE {
            if bom_enc.is_some() {
                let msg = message_format(XmlStreamReader::HTTP_EX_1, vec![c_t_mime.clone().unwrap_or_default(), c_t_enc.clone().unwrap_or_default(), bom_enc.map(|s| s.to_string()).unwrap_or_default(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()]);
                return Err(XmlStreamReaderException::new_full(&msg, c_t_mime, c_t_enc, bom_enc.map(|s| s.to_string()).unwrap_or_default(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()));
            }
            return Ok(c_t_enc.unwrap().clone());
        }

        // UTF-16 content type encoding
        if c_t_enc.as_ref().unwrap() == XmlStreamReader::UTF_16 {
            if bom_enc.is_some() && bom_enc.unwrap().starts_with(XmlStreamReader::UTF_16) {
                return Ok(bom_enc.unwrap().to_string());
            }
            let msg = message_format(XmlStreamReader::HTTP_EX_2, vec![c_t_mime.clone().unwrap_or_default(), c_t_enc.clone().unwrap_or_default(), bom_enc.map(|s| s.to_string()).unwrap_or_default(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()]);
            return Err(XmlStreamReaderException::new_full(&msg, c_t_mime, c_t_enc, bom_enc.map(|s| s.to_string()).unwrap_or_default(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()));
        }

        // UTF-32BE or UTF-132E content type encoding
        if c_t_enc.as_ref().unwrap() == XmlStreamReader::UTF_32BE || c_t_enc.as_ref().unwrap() == XmlStreamReader::UTF_32LE {
            if bom_enc.is_some() {
                let msg = message_format(XmlStreamReader::HTTP_EX_1, vec![c_t_mime.clone().unwrap_or_default(), c_t_enc.clone().unwrap_or_default(), bom_enc.map(|s| s.to_string()).unwrap_or_default(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()]);
                return Err(XmlStreamReaderException::new_full(&msg, c_t_mime, c_t_enc, bom_enc.map(|s| s.to_string()).unwrap_or_default(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()));
            }
            return Ok(c_t_enc.unwrap().clone());
        }

        // UTF-32 content type encoding
        if c_t_enc.as_ref().unwrap() == XmlStreamReader::UTF_32 {
            if bom_enc.is_some() && bom_enc.unwrap().starts_with(XmlStreamReader::UTF_32) {
                return Ok(bom_enc.unwrap().to_string());
            }
            let msg = message_format(XmlStreamReader::HTTP_EX_2, vec![c_t_mime.clone().unwrap_or_default(), c_t_enc.clone().unwrap_or_default(), bom_enc.map(|s| s.to_string()).unwrap_or_default(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()]);
            return Err(XmlStreamReaderException::new_full(&msg, c_t_mime, c_t_enc, bom_enc.map(|s| s.to_string()).unwrap_or_default(), xml_guess_enc.map(|s| s.to_string()).unwrap_or_default(), xml_enc.map(|s| s.to_string()).unwrap_or_default()));
        }

        Ok(c_t_enc.unwrap().clone())
    }

    /**
     * Returns MIME type or None if httpContentType is None.
     *
     * @param httpContentType the HTTP content type
     * @return The mime content type
     */
    fn get_content_type_mime(http_content_type: &str) -> Option<String> {
        let mut mime = None;
        if let Some(i) = http_content_type.find(";") {
            mime = Some(http_content_type[0..i].to_string());
        } else {
            mime = Some(http_content_type.to_string());
        }
        mime = Some(mime.unwrap().trim().to_string());
        mime
    }

    const CHARSET_PATTERN: &'static str = "charset=[\"']?([.[^; \"']]*)[\"']?";

    /**
     * Returns charset parameter value, None if not present, None if
     * httpContentType is None.
     *
     * @param httpContentType the HTTP content type
     * @return The content type encoding (upcased)
     */
    fn get_content_type_encoding(http_content_type: &str) -> Option<String> {
        let mut encoding = None;
        if let Some(i) = http_content_type.find(";") {
            let post_mime = http_content_type[i + 1..].to_string();
            let m = charset_pattern_find(&post_mime);
            encoding = m;
            encoding = encoding.map(|e| e.to_uppercase());
        }
        encoding
    }

    /**
     * Pattern capturing the encoding of the "xml" processing instruction.
     */
    pub const ENCODING_PATTERN: &'static str =
        "<\\?xml.*encoding[\\s]*=[\\s]*((?:\".[^\"]*\")|(?:'.[^']*'))";

    /**
     * Returns the encoding declared in the <?xml encoding=...?>, None if none.
     *
     * @param inputStream InputStream to create the reader from.
     * @param guessedEnc guessed encoding
     * @return the encoding declared in the <?xml encoding=...?>
     * @throws IOException thrown if there is a problem reading the stream.
     */
    fn get_xml_prolog(input_stream: &mut BOMInputStream, guessed_enc: Option<&str>) -> Result<Option<String>, XmlStreamReaderException> {
        let mut encoding = None;
        if guessed_enc.is_some() {
            let mut bytes = vec![0u8; XmlStreamReader::BUFFER_SIZE];
            input_stream.mark(XmlStreamReader::BUFFER_SIZE as i32);
            let mut offset = 0;
            let mut max = XmlStreamReader::BUFFER_SIZE;
            let mut c = input_stream.read_off(&mut bytes, offset, max).map_err(io_err)?;
            let mut first_gt = -1;
            let mut xml_prolog = "".to_string(); // avoid possible NPE warning (cannot happen; this just silences the warning)
            while c != -1 && first_gt == -1 && offset < XmlStreamReader::BUFFER_SIZE {
                offset += c as usize;
                max -= c as usize;
                c = input_stream.read_off(&mut bytes, offset, max).map_err(io_err)?;
                xml_prolog = String::from_utf8_lossy(&bytes[0..offset]).to_string();
                first_gt = xml_prolog.find('>').map(|i| i as i32).unwrap_or(-1);
            }
            if first_gt == -1 {
                if c == -1 {
                    return Err(XmlStreamReaderException::new_full("Unexpected end of XML stream", None, None, "".to_string(), guessed_enc.unwrap().to_string(), "".to_string()));
                }
                return Err(XmlStreamReaderException::new_full(
                    &("XML prolog or ROOT element not found on first ".to_string() + &offset.to_string() + " bytes"), None, None, "".to_string(), guessed_enc.unwrap().to_string(), "".to_string()));
            }
            let bytes_read = offset;
            if bytes_read > 0 {
                input_stream.reset().map_err(io_err)?;
                let prolog_str = xml_prolog[0..first_gt as usize + 1].to_string();
                let mut prolog = String::new();
                for line in prolog_str.lines() {
                    prolog.push_str(line);
                }
                if let Some(matched) = encoding_pattern_find(&prolog) {
                    let uppercased = matched.to_uppercase();
                    encoding = Some(uppercased[1..uppercased.len() - 1].to_string());
                }
            }
        }
        Ok(encoding)
    }

    /**
     * Indicates if the MIME type belongs to the APPLICATION XML family.
     *
     * @param mime The mime type
     * @return true if the mime type belongs to the APPLICATION XML family,
     * otherwise false
     */
    fn is_app_xml(mime: Option<&str>) -> bool {
        mime != None &&
            (mime.unwrap() == "application/xml" ||
                mime.unwrap() == "application/xml-dtd" ||
                mime.unwrap() == "application/xml-external-parsed-entity" ||
                mime.unwrap().starts_with("application/") && mime.unwrap().ends_with("+xml"))
    }

    /**
     * Indicates if the MIME type belongs to the TEXT XML family.
     *
     * @param mime The mime type
     * @return true if the mime type belongs to the TEXT XML family,
     * otherwise false
     */
    fn is_text_xml(mime: Option<&str>) -> bool {
        mime != None &&
            (mime.unwrap() == "text/xml" ||
                mime.unwrap() == "text/xml-external-parsed-entity" ||
                mime.unwrap().starts_with("text/") && mime.unwrap().ends_with("+xml"))
    }

    const RAW_EX_1: &'static str =
        "Invalid encoding, BOM [{0}] XML guess [{1}] XML prolog [{2}] encoding mismatch";

    const RAW_EX_2: &'static str =
        "Invalid encoding, BOM [{0}] XML guess [{1}] XML prolog [{2}] unknown BOM";

    const HTTP_EX_1: &'static str =
        "Invalid encoding, CT-MIME [{0}] CT-Enc [{1}] BOM [{2}] XML guess [{3}] XML prolog [{4}], BOM must be NULL";

    const HTTP_EX_2: &'static str =
        "Invalid encoding, CT-MIME [{0}] CT-Enc [{1}] BOM [{2}] XML guess [{3}] XML prolog [{4}], encoding mismatch";

    const HTTP_EX_3: &'static str =
        "Invalid encoding, CT-MIME [{0}] CT-Enc [{1}] BOM [{2}] XML guess [{3}] XML prolog [{4}], Invalid MIME";
}

pub fn message_format(pattern: &str, args: Vec<String>) -> String {
    let mut result = pattern.to_string();
    for (i, arg) in args.iter().enumerate() {
        result = result.replace(&format!("{{{}}}", i), arg);
    }
    result
}

pub fn charset_pattern_find(post_mime: &str) -> Option<String> {
    //charset=["']?([.[^; "']]*)[\"']?
    let lower = post_mime.to_lowercase();
    let idx = lower.find("charset=")?;
    let mut rest = &post_mime[idx + "charset=".len()..];
    if rest.starts_with('"') || rest.starts_with('\'') {
        rest = &rest[1..];
    }
    let mut value = String::new();
    for c in rest.chars() {
        if c == ';' || c == ' ' || c == '\'' || c == '"' {
            break;
        }
        value.push(c);
    }
    if value.is_empty() { None } else { Some(value) }
}

pub fn encoding_pattern_find(prolog: &str) -> Option<String> {
    //<\?xml.*encoding[\s]*=[\s]*((?:".[^"]*")|(?:'.[^']*'))
    let lower = prolog.to_lowercase();
    let xml_idx = lower.find("<?xml")?;
    let enc_idx = lower[xml_idx + 5..].find("encoding")?;
    let mut rest = &lower[xml_idx + 5 + enc_idx + 8..];
    let mut seen_eq = false;
    for (i, c) in rest.char_indices() {
        if c == '=' {
            seen_eq = true;
            rest = &rest[i + 1..];
            break;
        }
        if !c.is_whitespace() {
            return None;
        }
    }
    if !seen_eq {
        return None;
    }
    rest = rest.trim_start();
    if rest.starts_with('"') {
        let end = rest[1..].find('"')?;
        Some(rest[1..1 + end].to_string())
    } else if rest.starts_with('\'') {
        let end = rest[1..].find('\'')?;
        Some(rest[1..1 + end].to_string())
    } else {
        None
    }
}

// fix: 文件内私有最小 stub（避免与 stubs/prelude 的全局重导出产生 E0659 歧义）
fn io_err(e: io::Error) -> XmlStreamReaderException {
    XmlStreamReaderException::new(&e.to_string(), String::new(), String::new(), String::new())
}

struct FileInputStream;

impl FileInputStream {
    pub fn new(_file: File) -> Box<dyn InputStream> {
        Box::new(FileInputStream)
    }
}

impl InputStream for FileInputStream {
    fn read_byte(&mut self) -> i32 {
        IOUtil::EOF
    }
    fn read(&mut self, _bts: &mut [u8]) -> i32 {
        IOUtil::EOF // fix: 占位，未接入真实文件读取
    }
    fn read_off(&mut self, _bts: &mut [u8], _off: usize, _len: usize) -> i32 {
        IOUtil::EOF
    }
    fn skip(&mut self, _ln: i64) -> Result<i64, io::Error> {
        Ok(0)
    }
    fn available(&mut self) -> Result<i32, io::Error> {
        Ok(0)
    }
    fn close(&mut self) -> Result<(), io::Error> {
        Ok(())
    }
    fn mark(&mut self, _readlimit: i32) {}
    fn reset(&mut self) -> Result<(), io::Error> {
        Ok(())
    }
    fn mark_supported(&self) -> bool {
        false
    }
}

struct BufferedInputStream {
    inner: Box<dyn InputStream>,
}

impl BufferedInputStream {
    pub fn new(input: Box<dyn InputStream>, _size: usize) -> Self {
        BufferedInputStream { inner: input }
    }
}

impl InputStream for BufferedInputStream {
    fn read_byte(&mut self) -> i32 {
        self.inner.read_byte()
    }
    fn read(&mut self, bts: &mut [u8]) -> i32 {
        self.inner.read(bts)
    }
    fn read_off(&mut self, bts: &mut [u8], off: usize, len: usize) -> i32 {
        self.inner.read_off(bts, off, len)
    }
    fn skip(&mut self, ln: i64) -> Result<i64, io::Error> {
        self.inner.skip(ln)
    }
    fn available(&mut self) -> Result<i32, io::Error> {
        self.inner.available()
    }
    fn close(&mut self) -> Result<(), io::Error> {
        self.inner.close()
    }
    fn mark(&mut self, readlimit: i32) {
        self.inner.mark(readlimit)
    }
    fn reset(&mut self) -> Result<(), io::Error> {
        self.inner.reset()
    }
    fn mark_supported(&self) -> bool {
        self.inner.mark_supported()
    }
}

struct URL;

impl URL {
    pub fn open_connection(&self) -> URLConnection {
        URLConnection
    }
}

struct URLConnection;

impl URLConnection {
    pub fn get_content_type(&self) -> Option<String> {
        None // fix: 占位，未从连接头解析
    }
    pub fn get_input_stream(&self) -> Box<dyn InputStream> {
        Box::new(FileInputStream)
    }
    pub fn is_http_url_connection(&self) -> bool {
        false
    }
}

struct InputStreamReader {
    inner: Option<Box<dyn crate::stubs::InputStream>>,
    encoding: String,
    buf: Vec<u8>,
    pending: Vec<char>,
}

impl InputStreamReader {
    pub fn new(input: BOMInputStream, encoding: String) -> Self {
        // fix: 包装 BOMInputStream 为 stubs 字节流（原 read 恒 -1，XML 内容读不出）
        struct BomStream(BOMInputStream);
        impl crate::stubs::InputStream for BomStream {
            fn read(&mut self, b: &mut [u8], off: usize, len: usize) -> i32 {
                if off > b.len() {
                    return -1;
                }
                match self.0.read_off(&mut b[off..], 0, len) {
                    Ok(n) if n > 0 => n as i32,
                    _ => -1,
                }
            }
            fn close(&mut self) {}
        }
        InputStreamReader {
            inner: Some(Box::new(BomStream(input))),
            encoding,
            buf: Vec::new(),
            pending: Vec::new(),
        }
    }
    pub fn read(&mut self, buf: &mut [char], offset: usize, len: usize) -> Result<i32, io::Error> {
        while self.pending.is_empty() {
            if self.buf.is_empty() {
                let mut tmp = [0u8; 2048];
                let tmp_len = tmp.len();
                let n = self.inner.as_mut().unwrap().read(&mut tmp, 0, tmp_len);
                if n > 0 {
                    self.buf.extend_from_slice(&tmp[..n as usize]);
                }
            }
            if self.buf.is_empty() {
                return Ok(-1);
            }
            let (decoded, consumed) = decode_prefix(&self.encoding, &self.buf);
            if consumed == 0 {
                return Ok(0);
            }
            self.buf.drain(..consumed);
            self.pending.extend(decoded.chars());
        }
        let n = len.min(self.pending.len());
        for i in 0..n {
            buf[offset + i] = self.pending[i];
        }
        self.pending.drain(..n);
        Ok(n as i32)
    }
    pub fn close(&mut self) -> Result<(), io::Error> {
        Ok(())
    }
}

fn decode_prefix(encoding: &str, bytes: &[u8]) -> (String, usize) {
    if bytes.is_empty() {
        return (String::new(), 0);
    }
    if encoding.is_empty()
        || encoding.eq_ignore_ascii_case("UTF-8")
        || encoding.eq_ignore_ascii_case("utf8")
        || encoding.eq_ignore_ascii_case("UTF8")
    {
        let mut out = String::new();
        let mut consumed = 0usize;
        for (i, &b) in bytes.iter().enumerate() {
            let char_len = if b < 0x80 {
                1
            } else if b < 0xE0 {
                2
            } else if b < 0xF0 {
                3
            } else {
                4
            };
            if i + char_len > bytes.len() {
                break;
            }
            match std::str::from_utf8(&bytes[i..i + char_len]) {
                Ok(s) => {
                    out.push_str(s);
                    consumed = i + char_len;
                }
                Err(_) => break,
            }
        }
        (out, consumed)
    } else {
        let s = crate::io_legado_app_help_http_okhttputils::decode_bytes_with_charset(bytes, encoding);
        (s, bytes.len())
    }
}

struct BomDelegate(Rc<RefCell<BOMInputStream>>);

impl InputStream for BomDelegate {
    fn read_byte(&mut self) -> i32 {
        self.0.borrow_mut().read().unwrap_or(IOUtil::EOF)
    }
    fn read(&mut self, bts: &mut [u8]) -> i32 {
        self.0.borrow_mut().read_bytes(bts).unwrap_or(IOUtil::EOF)
    }
    fn read_off(&mut self, bts: &mut [u8], off: usize, len: usize) -> i32 {
        self.0.borrow_mut().read_off(bts, off, len).unwrap_or(IOUtil::EOF)
    }
    fn skip(&mut self, n: i64) -> Result<i64, io::Error> {
        self.0.borrow_mut().skip(n)
    }
    fn available(&mut self) -> Result<i32, io::Error> {
        Ok(0) // fix: 占位
    }
    fn close(&mut self) -> Result<(), io::Error> {
        Ok(()) // fix: 占位
    }
    fn mark(&mut self, readlimit: i32) {
        self.0.borrow_mut().mark(readlimit)
    }
    fn reset(&mut self) -> Result<(), io::Error> {
        self.0.borrow_mut().reset()
    }
    fn mark_supported(&self) -> bool {
        true // fix: 占位
    }
}
