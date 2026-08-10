// © 2016 and later: Unicode, Inc. and others.
// License & terms of use: http://www.unicode.org/copyright.html
/**
 * class CharsetRecog_2022  part of the ICU charset detection imlementation.
 * This is a superclass for the individual detectors for
 * each of the detectable members of the ISO 2022 family
 * of encodings.
 * <p>
 * The separate classes are nested within this class.
 */
// abstract class CharsetRecog_2022 extends CharsetRecognizer {
pub trait CharsetRecog_2022: CharsetRecognizer {
    /**
     * Matching function shared among the 2022 detectors JP, CN and KR
     * Counts up the number of legal an unrecognized escape sequences in
     * the sample of text, and computes a score based on the total number &
     * the proportion that fit the encoding.
     *
     * @param text            the byte buffer containing text to analyse
     * @param textLen         the size of the text in the byte.
     * @param escapeSequences the byte escape sequences to test for.
     * @return match quality, in the range of 0-100.
     */
    // int match(byte[] text, int textLen, byte[][] escapeSequences) {
    fn match_escape_sequences(&self, text: &[u8], text_len: usize, escape_sequences: &[&[u8]]) -> i32 {
        let mut i;
        let mut j;
        let mut esc_n;
        let mut hits = 0;
        let mut misses = 0;
        let mut shifts = 0;
        // int quality;
        // scanInput:
        // for (i = 0; i < textLen; i++) {
        i = 0;
        'scan_input: while i < text_len {
            if text[i] == 0x1b {
                // checkEscapes:
                // for (escN = 0; escN < escapeSequences.length; escN++) {
                //     byte[] seq = escapeSequences[escN];
                //
                //     if ((textLen - i) < seq.length) {
                //         continue;
                //     }
                //
                //     for (j = 1; j < seq.length; j++) {
                //         if (seq[j] != text[i + j]) {
                //             continue checkEscapes;
                //         }
                //     }
                //
                //     hits++;
                //     i += seq.length - 1;
                //     continue scanInput;
                // }
                //
                // misses++;
                esc_n = 0;
                'check_escapes: loop {
                    if esc_n >= escape_sequences.len() {
                        misses += 1;
                        break 'check_escapes;
                    }
                    let seq = escape_sequences[esc_n];

                    if (text_len - i) < seq.len() {
                        esc_n += 1;
                        continue 'check_escapes;
                    }

                    j = 1;
                    while j < seq.len() {
                        if seq[j] != text[i + j] {
                            // continue checkEscapes;
                            esc_n += 1;
                            continue 'check_escapes;
                        }
                        j += 1;
                    }

                    hits += 1;
                    i += seq.len() - 1;
                    continue 'scan_input;
                }
            }

            // if (text[i] == 0x0e || text[i] == 0x0f) {
            //     // Shift in/out
            //     shifts++;
            // }
            if text[i] == 0x0e || text[i] == 0x0f {
                // Shift in/out
                shifts += 1;
            }
            i += 1;
        }

        // if (hits == 0) {
        //     return 0;
        // }
        if hits == 0 {
            return 0;
        }

        //
        // Initial quality is based on relative proportion of recongized vs.
        //   unrecognized escape sequences.
        //   All good:  quality = 100;
        //   half or less good: quality = 0;
        //   linear inbetween.
        // quality = (100 * hits - 100 * misses) / (hits + misses);
        let mut quality = (100 * hits - 100 * misses) / (hits + misses);

        // Back off quality if there were too few escape sequences seen.
        //   Include shifts in this computation, so that KR does not get penalized
        //   for having only a single Escape sequence, but many shifts.
        // if (hits + shifts < 5) {
        //     quality -= (5 - (hits + shifts)) * 10;
        // }
        if hits + shifts < 5 {
            quality -= (5 - (hits + shifts)) * 10;
        }

        // if (quality < 0) {
        //     quality = 0;
        // }
        // return quality;
        if quality < 0 {
            quality = 0;
        }
        quality
    }
}

// static class CharsetRecog_2022JP extends CharsetRecog_2022 {
pub struct CharsetRecog_2022JP;

impl CharsetRecog_2022 for CharsetRecog_2022JP {}

impl CharsetRecognizer for CharsetRecog_2022JP {
    // private final byte[][] escapeSequences = {
    //         {0x1b, 0x24, 0x28, 0x43},   // KS X 1001:1992
    //         {0x1b, 0x24, 0x28, 0x44},   // JIS X 212-1990
    //         {0x1b, 0x24, 0x40},         // JIS C 6226-1978
    //         {0x1b, 0x24, 0x41},         // GB 2312-80
    //         {0x1b, 0x24, 0x42},         // JIS X 208-1983
    //         {0x1b, 0x26, 0x40},         // JIS X 208 1990, 1997
    //         {0x1b, 0x28, 0x42},         // ASCII
    //         {0x1b, 0x28, 0x48},         // JIS-Roman
    //         {0x1b, 0x28, 0x49},         // Half-width katakana
    //         {0x1b, 0x28, 0x4a},         // JIS-Roman
    //         {0x1b, 0x2e, 0x41},         // ISO 8859-1
    //         {0x1b, 0x2e, 0x46}          // ISO 8859-7
    // };
    const ESCAPE_SEQUENCES: &'static [&'static [u8]] = &[
        &[0x1b, 0x24, 0x28, 0x43],   // KS X 1001:1992
        &[0x1b, 0x24, 0x28, 0x44],   // JIS X 212-1990
        &[0x1b, 0x24, 0x40],         // JIS C 6226-1978
        &[0x1b, 0x24, 0x41],         // GB 2312-80
        &[0x1b, 0x24, 0x42],         // JIS X 208-1983
        &[0x1b, 0x26, 0x40],         // JIS X 208 1990, 1997
        &[0x1b, 0x28, 0x42],         // ASCII
        &[0x1b, 0x28, 0x48],         // JIS-Roman
        &[0x1b, 0x28, 0x49],         // Half-width katakana
        &[0x1b, 0x28, 0x4a],         // JIS-Roman
        &[0x1b, 0x2e, 0x41],         // ISO 8859-1
        &[0x1b, 0x2e, 0x46],         // ISO 8859-7
    ];

    // @Override
    // String getName() {
    //     return "ISO-2022-JP";
    // }
    fn get_name(&self) -> String {
        "ISO-2022-JP".to_string()
    }

    // @Override
    // CharsetMatch match(CharsetDetector det) {
    //     int confidence = match(det.fInputBytes, det.fInputLen, escapeSequences);
    //     return confidence == 0 ? null : new CharsetMatch(det, this, confidence);
    // }
    fn match_det(&self, det: &CharsetDetector) -> Option<CharsetMatch> {
        let confidence = self.match_escape_sequences(
            &det.f_input_bytes,
            det.f_input_len as usize,
            Self::ESCAPE_SEQUENCES,
        );
        if confidence == 0 {
            None
        } else {
            Some(CharsetMatch::new(det, self, confidence))
        }
    }
}

// static class CharsetRecog_2022KR extends CharsetRecog_2022 {
pub struct CharsetRecog_2022KR;

impl CharsetRecog_2022 for CharsetRecog_2022KR {}

impl CharsetRecognizer for CharsetRecog_2022KR {
    // private final byte[][] escapeSequences = {
    //         {0x1b, 0x24, 0x29, 0x43}
    // };
    const ESCAPE_SEQUENCES: &'static [&'static [u8]] = &[&[0x1b, 0x24, 0x29, 0x43]];

    // @Override
    // String getName() {
    //     return "ISO-2022-KR";
    // }
    fn get_name(&self) -> String {
        "ISO-2022-KR".to_string()
    }

    // @Override
    // CharsetMatch match(CharsetDetector det) {
    //     int confidence = match(det.fInputBytes, det.fInputLen, escapeSequences);
    //     return confidence == 0 ? null : new CharsetMatch(det, this, confidence);
    // }
    fn match_det(&self, det: &CharsetDetector) -> Option<CharsetMatch> {
        let confidence = self.match_escape_sequences(
            &det.f_input_bytes,
            det.f_input_len as usize,
            Self::ESCAPE_SEQUENCES,
        );
        if confidence == 0 {
            None
        } else {
            Some(CharsetMatch::new(det, self, confidence))
        }
    }
}

// static class CharsetRecog_2022CN extends CharsetRecog_2022 {
pub struct CharsetRecog_2022CN;

impl CharsetRecog_2022 for CharsetRecog_2022CN {}

impl CharsetRecognizer for CharsetRecog_2022CN {
    // private final byte[][] escapeSequences = {
    //         {0x1b, 0x24, 0x29, 0x41},   // GB 2312-80
    //         {0x1b, 0x24, 0x29, 0x47},   // CNS 11643-1992 Plane 1
    //         {0x1b, 0x24, 0x2A, 0x48},   // CNS 11643-1992 Plane 2
    //         {0x1b, 0x24, 0x29, 0x45},   // ISO-IR-165
    //         {0x1b, 0x24, 0x2B, 0x49},   // CNS 11643-1992 Plane 3
    //         {0x1b, 0x24, 0x2B, 0x4A},   // CNS 11643-1992 Plane 4
    //         {0x1b, 0x24, 0x2B, 0x4B},   // CNS 11643-1992 Plane 5
    //         {0x1b, 0x24, 0x2B, 0x4C},   // CNS 11643-1992 Plane 6
    //         {0x1b, 0x24, 0x2B, 0x4D},   // CNS 11643-1992 Plane 7
    //         {0x1b, 0x4e},               // SS2
    //         {0x1b, 0x4f},               // SS3
    // };
    const ESCAPE_SEQUENCES: &'static [&'static [u8]] = &[
        &[0x1b, 0x24, 0x29, 0x41],   // GB 2312-80
        &[0x1b, 0x24, 0x29, 0x47],   // CNS 11643-1992 Plane 1
        &[0x1b, 0x24, 0x2A, 0x48],   // CNS 11643-1992 Plane 2
        &[0x1b, 0x24, 0x29, 0x45],   // ISO-IR-165
        &[0x1b, 0x24, 0x2B, 0x49],   // CNS 11643-1992 Plane 3
        &[0x1b, 0x24, 0x2B, 0x4A],   // CNS 11643-1992 Plane 4
        &[0x1b, 0x24, 0x2B, 0x4B],   // CNS 11643-1992 Plane 5
        &[0x1b, 0x24, 0x2B, 0x4C],   // CNS 11643-1992 Plane 6
        &[0x1b, 0x24, 0x2B, 0x4D],   // CNS 11643-1992 Plane 7
        &[0x1b, 0x4e],               // SS2
        &[0x1b, 0x4f],               // SS3
    ];

    // @Override
    // String getName() {
    //     return "ISO-2022-CN";
    // }
    fn get_name(&self) -> String {
        "ISO-2022-CN".to_string()
    }

    // @Override
    // CharsetMatch match(CharsetDetector det) {
    //     int confidence = match(det.fInputBytes, det.fInputLen, escapeSequences);
    //     return confidence == 0 ? null : new CharsetMatch(det, this, confidence);
    // }
    fn match_det(&self, det: &CharsetDetector) -> Option<CharsetMatch> {
        let confidence = self.match_escape_sequences(
            &det.f_input_bytes,
            det.f_input_len as usize,
            Self::ESCAPE_SEQUENCES,
        );
        if confidence == 0 {
            None
        } else {
            Some(CharsetMatch::new(det, self, confidence))
        }
    }
}
