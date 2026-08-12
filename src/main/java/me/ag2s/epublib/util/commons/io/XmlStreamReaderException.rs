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

use std::io;

/**
 * The XmlStreamReaderException is thrown by the XmlStreamReader constructors if
 * the charset encoding can not be determined according to the XML 1.0
 * specification and RFC 3023.
 * <p>
 * The exception returns the unconsumed InputStream to allow the application to
 * do an alternate processing with the stream. Note that the original
 * InputStream given to the XmlStreamReader cannot be used as that one has been
 * already read.
 * </p>
 *
 * @since 2.0
 */
pub struct XmlStreamReaderException {
    source: io::Error,
    serial_version_uid: i64,
    bom_encoding: Option<String>,
    xml_guess_encoding: Option<String>,
    xml_encoding: Option<String>,
    content_type_mime: Option<String>,
    content_type_encoding: Option<String>,
}

impl XmlStreamReaderException {

    const SERIAL_VERSION_UID: i64 = 1;

    /**
     * Creates an exception instance if the charset encoding could not be
     * determined.
     * <p>
     * Instances of this exception are thrown by the XmlStreamReader.
     * </p>
     *
     * @param msg message describing the reason for the exception.
     * @param bomEnc BOM encoding.
     * @param xmlGuessEnc XML guess encoding.
     * @param xmlEnc XML prolog encoding.
     */
    pub fn new(msg: &str, bom_enc: String, xml_guess_enc: String, xml_enc: String) -> Self {
        XmlStreamReaderException::new_full(msg, None, None, bom_enc, xml_guess_enc, xml_enc)
    }

    /**
     * Creates an exception instance if the charset encoding could not be
     * determined.
     * <p>
     * Instances of this exception are thrown by the XmlStreamReader.
     * </p>
     *
     * @param msg message describing the reason for the exception.
     * @param ctMime MIME type in the content-type.
     * @param ctEnc encoding in the content-type.
     * @param bomEnc BOM encoding.
     * @param xmlGuessEnc XML guess encoding.
     * @param xmlEnc XML prolog encoding.
     */
    pub fn new_full(msg: &str, ct_mime: Option<String>, ct_enc: Option<String>,
                    bom_enc: String, xml_guess_enc: String, xml_enc: String) -> Self {
        XmlStreamReaderException {
            source: io::Error::new(io::ErrorKind::Other, msg),
            serial_version_uid: 1,
            content_type_mime: ct_mime,
            content_type_encoding: ct_enc,
            bom_encoding: Some(bom_enc),
            xml_guess_encoding: Some(xml_guess_enc),
            xml_encoding: Some(xml_enc),
        }
    }

    /**
     * Returns the BOM encoding found in the InputStream.
     *
     * @return the BOM encoding, None if none.
     */
    pub fn get_bom_encoding(&self) -> &Option<String> {
        &self.bom_encoding
    }

    /**
     * Returns the encoding guess based on the first bytes of the InputStream.
     *
     * @return the encoding guess, None if it couldn't be guessed.
     */
    pub fn get_xml_guess_encoding(&self) -> &Option<String> {
        &self.xml_guess_encoding
    }

    /**
     * Returns the encoding found in the XML prolog of the InputStream.
     *
     * @return the encoding of the XML prolog, None if none.
     */
    pub fn get_xml_encoding(&self) -> &Option<String> {
        &self.xml_encoding
    }

    /**
     * Returns the MIME type in the content-type used to attempt determining the
     * encoding.
     *
     * @return the MIME type in the content-type, None if there was not
     *         content-type or the encoding detection did not involve HTTP.
     */
    pub fn get_content_type_mime(&self) -> &Option<String> {
        &self.content_type_mime
    }

    /**
     * Returns the encoding in the content-type used to attempt determining the
     * encoding.
     *
     * @return the encoding in the content-type, None if there was not
     *         content-type, no encoding in it or the encoding detection did not
     *         involve HTTP.
     */
    pub fn get_content_type_encoding(&self) -> &Option<String> {
        &self.content_type_encoding
    }
}
