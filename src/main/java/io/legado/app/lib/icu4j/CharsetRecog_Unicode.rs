// © 2016 and later: Unicode, Inc. and others.
// License & terms of use: http://www.unicode.org/copyright.html
/*
 *******************************************************************************
 * Copyright (C) 1996-2013, International Business Machines Corporation and    *
 * others. All Rights Reserved.                                                *
 *******************************************************************************
 *
 */

// package io.legado.app.lib.icu4j;

/**
 * This class matches UTF-16 and UTF-32, both big- and little-endian. The
 * BOM will be used if it is present.
 */
// abstract class CharsetRecog_Unicode extends CharsetRecognizer {
pub trait CharsetRecog_Unicode: CharsetRecognizer {}

// static int codeUnit16FromBytes(byte hi, byte lo) {
//     return ((hi & 0xff) << 8) | (lo & 0xff);
// }
fn code_unit16_from_bytes(hi: u8, lo: u8) -> i32 {
    ((hi as i32 & 0xff) << 8) | (lo as i32 & 0xff)
}

// UTF-16 confidence calculation. Very simple minded, but better than nothing.
//   Any 8 bit non-control characters bump the confidence up. These have a zero high byte,
//     and are very likely to be UTF-16, although they could also be part of a UTF-32 code.
//   NULs are a contra-indication, they will appear commonly if the actual encoding is UTF-32.
//   NULs should be rare in actual text.
// static int adjustConfidence(int codeUnit, int confidence) {
//     if (codeUnit == 0) {
//         confidence -= 10;
//     } else if ((codeUnit >= 0x20 && codeUnit <= 0xff) || codeUnit == 0x0a) {
//         confidence += 10;
//     }
//     if (confidence < 0) {
//         confidence = 0;
//     } else if (confidence > 100) {
//         confidence = 100;
//     }
//     return confidence;
// }
fn adjust_confidence(code_unit: i32, mut confidence: i32) -> i32 {
    if code_unit == 0 {
        confidence -= 10;
    } else if (code_unit >= 0x20 && code_unit <= 0xff) || code_unit == 0x0a {
        confidence += 10;
    }
    if confidence < 0 {
        confidence = 0;
    } else if confidence > 100 {
        confidence = 100;
    }
    confidence
}

// static class CharsetRecog_UTF_16_BE extends CharsetRecog_Unicode {
pub struct CharsetRecog_UTF_16_BE;

impl CharsetRecog_Unicode for CharsetRecog_UTF_16_BE {}

impl CharsetRecognizer for CharsetRecog_UTF_16_BE {
    // @Override
    // String getName() {
    //     return "UTF-16BE";
    // }
    fn get_name(&self) -> String {
        "UTF-16BE".to_string()
    }

    // @Override
    // CharsetMatch match(CharsetDetector det) {
    fn match_det(&self, det: &CharsetDetector) -> Option<CharsetMatch> {
        // byte[] input = det.fRawInput;
        let input = det.f_raw_input.clone().unwrap_or_default();
        let mut confidence = 10;

        // int bytesToCheck = Math.min(input.length, 30);
        let bytes_to_check = input.len().min(30);
        // for (int charIndex = 0; charIndex < bytesToCheck - 1; charIndex += 2) {
        //     int codeUnit = codeUnit16FromBytes(input[charIndex], input[charIndex + 1]);
        //     if (charIndex == 0 && codeUnit == 0xFEFF) {
        //         confidence = 100;
        //         break;
        //     }
        //     confidence = adjustConfidence(codeUnit, confidence);
        //     if (confidence == 0 || confidence == 100) {
        //         break;
        //     }
        // }
        let mut char_index = 0;
        while char_index < bytes_to_check.saturating_sub(1) {
            let code_unit = code_unit16_from_bytes(input[char_index], input[char_index + 1]);
            if char_index == 0 && code_unit == 0xFEFF {
                confidence = 100;
                break;
            }
            confidence = adjust_confidence(code_unit, confidence);
            if confidence == 0 || confidence == 100 {
                break;
            }
            char_index += 2;
        }
        // if (bytesToCheck < 4 && confidence < 100) {
        //     confidence = 0;
        // }
        if bytes_to_check < 4 && confidence < 100 {
            confidence = 0;
        }
        // if (confidence > 0) {
        //     return new CharsetMatch(det, this, confidence);
        // }
        // return null;
        if confidence > 0 {
            Some(CharsetMatch::new(det, self, confidence))
        } else {
            None
        }
    }
}

// static class CharsetRecog_UTF_16_LE extends CharsetRecog_Unicode {
pub struct CharsetRecog_UTF_16_LE;

impl CharsetRecog_Unicode for CharsetRecog_UTF_16_LE {}

impl CharsetRecognizer for CharsetRecog_UTF_16_LE {
    // @Override
    // String getName() {
    //     return "UTF-16LE";
    // }
    fn get_name(&self) -> String {
        "UTF-16LE".to_string()
    }

    // @Override
    // CharsetMatch match(CharsetDetector det) {
    fn match_det(&self, det: &CharsetDetector) -> Option<CharsetMatch> {
        // byte[] input = det.fRawInput;
        let input = det.f_raw_input.clone().unwrap_or_default();
        let mut confidence = 10;

        // int bytesToCheck = Math.min(input.length, 30);
        let bytes_to_check = input.len().min(30);
        // for (int charIndex = 0; charIndex < bytesToCheck - 1; charIndex += 2) {
        //     int codeUnit = codeUnit16FromBytes(input[charIndex + 1], input[charIndex]);
        //     if (charIndex == 0 && codeUnit == 0xFEFF) {
        //         confidence = 100;
        //         break;
        //     }
        //     confidence = adjustConfidence(codeUnit, confidence);
        //     if (confidence == 0 || confidence == 100) {
        //         break;
        //     }
        // }
        let mut char_index = 0;
        while char_index < bytes_to_check.saturating_sub(1) {
            let code_unit = code_unit16_from_bytes(input[char_index + 1], input[char_index]);
            if char_index == 0 && code_unit == 0xFEFF {
                confidence = 100;
                break;
            }
            confidence = adjust_confidence(code_unit, confidence);
            if confidence == 0 || confidence == 100 {
                break;
            }
            char_index += 2;
        }
        // if (bytesToCheck < 4 && confidence < 100) {
        //     confidence = 0;
        // }
        if bytes_to_check < 4 && confidence < 100 {
            confidence = 0;
        }
        // if (confidence > 0) {
        //     return new CharsetMatch(det, this, confidence);
        // }
        // return null;
        if confidence > 0 {
            Some(CharsetMatch::new(det, self, confidence))
        } else {
            None
        }
    }
}

// static abstract class CharsetRecog_UTF_32 extends CharsetRecog_Unicode {
//     abstract int getChar(byte[] input, int index);
//
//     @Override
//     abstract String getName();
//
//     @Override
//     CharsetMatch match(CharsetDetector det) {
//         byte[] input = det.fRawInput;
//         int limit = (det.fRawLength / 4) * 4;
//         int numValid = 0;
//         int numInvalid = 0;
//         boolean hasBOM = false;
//         int confidence = 0;
//
//         if (limit == 0) {
//             return null;
//         }
//         if (getChar(input, 0) == 0x0000FEFF) {
//             hasBOM = true;
//         }
//
//         for (int i = 0; i < limit; i += 4) {
//             int ch = getChar(input, i);
//
//             if (ch < 0 || ch >= 0x10FFFF || (ch >= 0xD800 && ch <= 0xDFFF)) {
//                 numInvalid += 1;
//             } else {
//                 numValid += 1;
//             }
//         }
//
//
//         // Cook up some sort of confidence score, based on presence of a BOM
//         //    and the existence of valid and/or invalid multi-byte sequences.
//         if (hasBOM && numInvalid == 0) {
//             confidence = 100;
//         } else if (hasBOM && numValid > numInvalid * 10) {
//             confidence = 80;
//         } else if (numValid > 3 && numInvalid == 0) {
//             confidence = 100;
//         } else if (numValid > 0 && numInvalid == 0) {
//             confidence = 80;
//         } else if (numValid > numInvalid * 10) {
//             // Probably corrupt UTF-32BE data.  Valid sequences aren't likely by chance.
//             confidence = 25;
//         }
//
//         return confidence == 0 ? null : new CharsetMatch(det, this, confidence);
//     }
// }
pub trait CharsetRecog_UTF_32: CharsetRecog_Unicode {
    fn get_char(&self, input: &[u8], index: usize) -> i32;

    fn match_utf32(&self, det: &CharsetDetector) -> Option<CharsetMatch> {
        // byte[] input = det.fRawInput;
        let input = det.f_raw_input.clone().unwrap_or_default();
        // int limit = (det.fRawLength / 4) * 4;
        let limit = (det.f_raw_length / 4) * 4;
        let mut num_valid = 0;
        let mut num_invalid = 0;
        let mut has_bom = false;
        let mut confidence = 0;

        // if (limit == 0) {
        //     return null;
        // }
        if limit == 0 {
            return None;
        }
        // if (getChar(input, 0) == 0x0000FEFF) {
        //     hasBOM = true;
        // }
        if self.get_char(&input, 0) == 0x0000FEFF {
            has_bom = true;
        }

        // for (int i = 0; i < limit; i += 4) {
        //     int ch = getChar(input, i);
        //
        //     if (ch < 0 || ch >= 0x10FFFF || (ch >= 0xD800 && ch <= 0xDFFF)) {
        //         numInvalid += 1;
        //     } else {
        //         numValid += 1;
        //     }
        // }
        let mut i = 0;
        while i < limit as usize {
            let ch = self.get_char(&input, i);

            if ch < 0 || ch >= 0x10FFFF || (ch >= 0xD800 && ch <= 0xDFFF) {
                num_invalid += 1;
            } else {
                num_valid += 1;
            }
            i += 4;
        }

        // Cook up some sort of confidence score, based on presence of a BOM
        //    and the existence of valid and/or invalid multi-byte sequences.
        // if (hasBOM && numInvalid == 0) {
        //     confidence = 100;
        // } else if (hasBOM && numValid > numInvalid * 10) {
        //     confidence = 80;
        // } else if (numValid > 3 && numInvalid == 0) {
        //     confidence = 100;
        // } else if (numValid > 0 && numInvalid == 0) {
        //     confidence = 80;
        // } else if (numValid > numInvalid * 10) {
        //     // Probably corrupt UTF-32BE data.  Valid sequences aren't likely by chance.
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
        } else if num_valid > num_invalid * 10 {
            // Probably corrupt UTF-32BE data.  Valid sequences aren't likely by chance.
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

// static class CharsetRecog_UTF_32_BE extends CharsetRecog_UTF_32 {
pub struct CharsetRecog_UTF_32_BE;

impl CharsetRecog_Unicode for CharsetRecog_UTF_32_BE {}

impl CharsetRecog_UTF_32 for CharsetRecog_UTF_32_BE {
    // @Override
    // int getChar(byte[] input, int index) {
    //     return (input[index + 0] & 0xFF) << 24 | (input[index + 1] & 0xFF) << 16 |
    //             (input[index + 2] & 0xFF) << 8 | (input[index + 3] & 0xFF);
    // }
    fn get_char(&self, input: &[u8], index: usize) -> i32 {
        (input[index + 0] as i32 & 0xFF) << 24 | (input[index + 1] as i32 & 0xFF) << 16
            | (input[index + 2] as i32 & 0xFF) << 8 | (input[index + 3] as i32 & 0xFF)
    }
}

impl CharsetRecognizer for CharsetRecog_UTF_32_BE {
    // @Override
    // String getName() {
    //     return "UTF-32BE";
    // }
    fn get_name(&self) -> String {
        "UTF-32BE".to_string()
    }

    // @Override
    // CharsetMatch match(CharsetDetector det) {
    fn match_det(&self, det: &CharsetDetector) -> Option<CharsetMatch> {
        self.match_utf32(det)
    }
}

// static class CharsetRecog_UTF_32_LE extends CharsetRecog_UTF_32 {
pub struct CharsetRecog_UTF_32_LE;

impl CharsetRecog_Unicode for CharsetRecog_UTF_32_LE {}

impl CharsetRecog_UTF_32 for CharsetRecog_UTF_32_LE {
    // @Override
    // int getChar(byte[] input, int index) {
    //     return (input[index + 3] & 0xFF) << 24 | (input[index + 2] & 0xFF) << 16 |
    //             (input[index + 1] & 0xFF) << 8 | (input[index + 0] & 0xFF);
    // }
    fn get_char(&self, input: &[u8], index: usize) -> i32 {
        (input[index + 3] as i32 & 0xFF) << 24 | (input[index + 2] as i32 & 0xFF) << 16
            | (input[index + 1] as i32 & 0xFF) << 8 | (input[index + 0] as i32 & 0xFF)
    }
}

impl CharsetRecognizer for CharsetRecog_UTF_32_LE {
    // @Override
    // String getName() {
    //     return "UTF-32LE";
    // }
    fn get_name(&self) -> String {
        "UTF-32LE".to_string()
    }

    // @Override
    // CharsetMatch match(CharsetDetector det) {
    fn match_det(&self, det: &CharsetDetector) -> Option<CharsetMatch> {
        self.match_utf32(det)
    }
}
