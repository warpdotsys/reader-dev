// © 2016 and later: Unicode, Inc. and others.
// License & terms of use: http://www.unicode.org/copyright.html
/**
 * ******************************************************************************
 * Copyright (C) 2005 - 2014, International Business Machines Corporation and  *
 * others. All Rights Reserved.                                                *
 * ******************************************************************************
 */
// package io.legado.app.lib.icu4j;

/**
 * Charset recognizer for UTF-8
 */
// class CharsetRecog_UTF8 extends CharsetRecognizer {
pub struct CharsetRecog_UTF8;

impl CharsetRecognizer for CharsetRecog_UTF8 {
    // @Override
    // String getName() {
    //     return "UTF-8";
    // }
    fn get_name(&self) -> String {
        "UTF-8".to_string()
    }

    /* (non-Javadoc)
     * @see com.ibm.icu.text.CharsetRecognizer#match(com.ibm.icu.text.CharsetDetector)
     */
    // @Override
    // CharsetMatch match(CharsetDetector det) {
    fn match_det(&self, det: &CharsetDetector) -> Option<CharsetMatch> {
        let mut has_bom = false;
        let mut num_valid = 0;
        let mut num_invalid = 0;
        // byte[] input = det.fRawInput;
        let input = det.f_raw_input.clone().unwrap_or_default();
        let mut i;
        let mut trail_bytes = 0;
        let confidence: i32;

        // if (det.fRawLength >= 3 &&
        //         (input[0] & 0xFF) == 0xef && (input[1] & 0xFF) == 0xbb && (input[2] & 0xFF) == 0xbf) {
        //     hasBOM = true;
        // }
        if det.f_raw_length >= 3
            && (input[0] & 0xFF) == 0xef && (input[1] & 0xFF) == 0xbb && (input[2] & 0xFF) == 0xbf
        {
            has_bom = true;
        }

        // Scan for multi-byte sequences
        // for (i = 0; i < det.fRawLength; i++) {
        //     int b = input[i];
        //     if ((b & 0x80) == 0) {
        //         continue;   // ASCII
        //     }
        //
        //     // Hi bit on char found.  Figure out how long the sequence should be
        //     if ((b & 0x0e0) == 0x0c0) {
        //         trailBytes = 1;
        //     } else if ((b & 0x0f0) == 0x0e0) {
        //         trailBytes = 2;
        //     } else if ((b & 0x0f8) == 0xf0) {
        //         trailBytes = 3;
        //     } else {
        //         numInvalid++;
        //         continue;
        //     }
        //
        //     // Verify that we've got the right number of trail bytes in the sequence
        //     for (; ; ) {
        //         i++;
        //         if (i >= det.fRawLength) {
        //             break;
        //         }
        //         b = input[i];
        //         if ((b & 0xc0) != 0x080) {
        //             numInvalid++;
        //             break;
        //         }
        //         if (--trailBytes == 0) {
        //             numValid++;
        //             break;
        //         }
        //     }
        // }
        i = 0;
        while i < det.f_raw_length as usize {
            let mut b = input[i];
            if (b & 0x80) == 0 {
                i += 1;
                continue; // ASCII
            }

            // Hi bit on char found.  Figure out how long the sequence should be
            if (b & 0x0e0) == 0x0c0 {
                trail_bytes = 1;
            } else if (b & 0x0f0) == 0x0e0 {
                trail_bytes = 2;
            } else if (b & 0x0f8) == 0xf0 {
                trail_bytes = 3;
            } else {
                num_invalid += 1;
                i += 1;
                continue;
            }

            // Verify that we've got the right number of trail bytes in the sequence
            loop {
                i += 1;
                if i >= det.f_raw_length as usize {
                    break;
                }
                b = input[i];
                if (b & 0xc0) != 0x080 {
                    num_invalid += 1;
                    break;
                }
                trail_bytes -= 1;
                if trail_bytes == 0 {
                    num_valid += 1;
                    break;
                }
            }
        }

        // Cook up some sort of confidence score, based on presense of a BOM
        //    and the existence of valid and/or invalid multi-byte sequences.
        confidence = 0;
        // if (hasBOM && numInvalid == 0) {
        //     confidence = 100;
        // } else if (hasBOM && numValid > numInvalid * 10) {
        //     confidence = 80;
        // } else if (numValid > 3 && numInvalid == 0) {
        //     confidence = 100;
        // } else if (numValid > 0 && numInvalid == 0) {
        //     confidence = 80;
        // } else if (numValid == 0 && numInvalid == 0) {
        //     // Plain ASCII. Confidence must be > 10, it's more likely than UTF-16, which
        //     //              accepts ASCII with confidence = 10.
        //     // TODO: add plain ASCII as an explicitly detected type.
        //     confidence = 15;
        // } else if (numValid > numInvalid * 10) {
        //     // Probably corruput utf-8 data.  Valid sequences aren't likely by chance.
        //     confidence = 25;
        // }
        if has_bom && num_invalid == 0 {
            confidence = 100;
        } else if has_bom && num_valid > num_invalid * 10 {
            confidence = 80;
        } else if num_valid > 3 && num_invalid == 0 {
            confidence = 100;
        } else if num_valid > 0 && num_invalid == 0 {
            confidence = 80;
        } else if num_valid == 0 && num_invalid == 0 {
            // Plain ASCII. Confidence must be > 10, it's more likely than UTF-16, which
            //              accepts ASCII with confidence = 10.
            // TODO: add plain ASCII as an explicitly detected type.
            confidence = 15;
        } else if num_valid > num_invalid * 10 {
            // Probably corruput utf-8 data.  Valid sequences aren't likely by chance.
            confidence = 25;
        }
        // return confidence == 0 ? null : new CharsetMatch(det, this, confidence);
        if confidence == 0 {
            None
        } else {
            Some(CharsetMatch::new(det, self, confidence))
        }
    }
}
