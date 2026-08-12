use crate::prelude::*;
pub struct Base64;

impl Base64 {
    /**
     * Default values for encoder/decoder flags.
     */
    pub const DEFAULT: i32 = 0;
    /**
     * Encoder flag bit to omit the padding '=' characters at the end
     * of the output (if any).
     */
    pub const NO_PADDING: i32 = 1;
    /**
     * Encoder flag bit to omit all line terminators (i.e., the output
     * will be on one long line).
     */
    pub const NO_WRAP: i32 = 2;
    /**
     * Encoder flag bit to indicate lines should be terminated with a
     * CRLF pair instead of just an LF.  Has no effect if {@code
     * NO_WRAP} is specified as well.
     */
    pub const CRLF: i32 = 4;
    /**
     * Encoder/decoder flag bit to indicate using the "URL and
     * filename safe" variant of Base64 (see RFC 3548 section 4) where
     * {@code -} and {@code _} are used in place of {@code +} and
     * {@code /}.
     */
    pub const URL_SAFE: i32 = 8;
    /**
     * Flag to pass to {@link Base64OutputStream} to indicate that it
     * should not close the output stream it is wrapping when it
     * itself is closed.
     */
    pub const NO_CLOSE: i32 = 16;

    //  --------------------------------------------------------
    //  shared code
    //  --------------------------------------------------------

    //  --------------------------------------------------------
    //  decoding
    //  --------------------------------------------------------

    /**
     * Decode the Base64-encoded data in input and return the data in
     * a new byte array.
     *
     * <p>The padding '=' characters at the end are considered optional, but
     * if any are present, there must be the correct number of them.
     *
     * @param str   the input String to decode, which is converted to
     *              bytes using the default charset
     * @param flags controls certain features of the decoded output.
     *              Pass {@code DEFAULT} to decode standard Base64.
     * @throws IllegalArgumentException if the input contains
     *                                  incorrect padding
     */
    pub fn decode_str(str: &str, flags: i32) -> Vec<u8> {
        Self::decode(str.as_bytes(), flags)
    }

    /**
     * Decode the Base64-encoded data in input and return the data in
     * a new byte array.
     *
     * <p>The padding '=' characters at the end are considered optional, but
     * if any are present, there must be the correct number of them.
     *
     * @param input the input array to decode
     * @param flags controls certain features of the decoded output.
     *              Pass {@code DEFAULT} to decode standard Base64.
     * @throws IllegalArgumentException if the input contains
     *                                  incorrect padding
     */
    pub fn decode(input: &[u8], flags: i32) -> Vec<u8> {
        Self::decode_range(input, 0, input.len(), flags)
    }

    /**
     * Decode the Base64-encoded data in input and return the data in
     * a new byte array.
     *
     * <p>The padding '=' characters at the end are considered optional, but
     * if any are present, there must be the correct number of them.
     *
     * @param input  the data to decode
     * @param offset the position within the input array at which to start
     * @param len    the number of bytes of input to decode
     * @param flags  controls certain features of the decoded output.
     *               Pass {@code DEFAULT} to decode standard Base64.
     * @throws IllegalArgumentException if the input contains
     *                                  incorrect padding
     */
    pub fn decode_range(input: &[u8], offset: usize, len: usize, flags: i32) -> Vec<u8> {
        // Allocate space for the most data the input could represent.
        // (It could contain less if it contains whitespace, etc.)
        let mut decoder = Decoder::new(flags, vec![0u8; len * 3 / 4]);
        if !decoder.process(input, offset, len, true) {
            panic!("bad base-64");
        }
        // Maybe we got lucky and allocated exactly enough output space.
        if decoder.op == decoder.output.len() {
            return decoder.output;
        }
        // Need to shorten the array, so allocate a new one of the
        // right size and copy.
        let mut temp = vec![0u8; decoder.op];
        System::arraycopy(&decoder.output, 0, &mut temp, 0, decoder.op);
        temp
    }

    //  --------------------------------------------------------
    //  encoding
    //  --------------------------------------------------------

    /**
     * Base64-encode the given data and return a newly allocated
     * String with the result.
     *
     * @param input the data to encode
     * @param flags controls certain features of the encoded output.
     *              Passing {@code DEFAULT} results in output that
     *              adheres to RFC 2045.
     */
    pub fn encodeToString(input: &[u8], flags: i32) -> String {
        String::from_utf8_lossy(&Self::encode(input, flags)).to_string()
    }

    /**
     * Base64-encode the given data and return a newly allocated
     * String with the result.
     *
     * @param input  the data to encode
     * @param offset the position within the input array at which to
     *               start
     * @param len    the number of bytes of input to encode
     * @param flags  controls certain features of the encoded output.
     *               Passing {@code DEFAULT} results in output that
     *               adheres to RFC 2045.
     */
    pub fn encodeToString_range(input: &[u8], offset: usize, len: usize, flags: i32) -> String {
        String::from_utf8_lossy(&Self::encode_range(input, offset, len, flags)).to_string()
    }

    /**
     * Base64-encode the given data and return a newly allocated
     * byte[] with the result.
     *
     * @param input the data to encode
     * @param flags controls certain features of the encoded output.
     *              Passing {@code DEFAULT} results in output that
     *              adheres to RFC 2045.
     */
    pub fn encode(input: &[u8], flags: i32) -> Vec<u8> {
        Self::encode_range(input, 0, input.len(), flags)
    }

    /**
     * Base64-encode the given data and return a newly allocated
     * byte[] with the result.
     *
     * @param input  the data to encode
     * @param offset the position within the input array at which to
     *               start
     * @param len    the number of bytes of input to encode
     * @param flags  controls certain features of the encoded output.
     *               Passing {@code DEFAULT} results in output that
     *               adheres to RFC 2045.
     */
    pub fn encode_range(input: &[u8], offset: usize, len: usize, flags: i32) -> Vec<u8> {
        let mut encoder = Encoder::new(flags, None);
        // Compute the exact length of the array we will produce.
        let mut output_len = len / 3 * 4;
        // Account for the tail of the data and the padding bytes, if any.
        if encoder.do_padding {
            if len % 3 > 0 {
                output_len += 4;
            }
        } else {
            match len % 3 {
                0 => {}
                1 => output_len += 2,
                2 => output_len += 3,
                _ => {}
            }
        }
        // Account for the newlines, if any.
        if encoder.do_newline && len > 0 {
            output_len += (((len - 1) / (3 * Encoder::LINE_GROUPS)) + 1)
                * if encoder.do_cr { 2 } else { 1 };
        }
        encoder.output = Some(vec![0u8; output_len]);
        encoder.process(input, offset, len, true);
        assert!(encoder.op == output_len);
        encoder.output.unwrap()
    }

    pub fn new() -> Base64 {
        Base64
    }   // don't instantiate
}

/* package */ pub trait Coder {
    /**
     * Encode/decode another block of input data.  this.output is
     * provided by the caller, and must be big enough to hold all
     * the coded data.  On exit, this.opwill be set to the length
     * of the coded data.
     *
     * @param finish true if this is the final call to process for
     *               this object.  Will finalize the coder state and
     *               include any final bytes in the output.
     * @return true if the input so far is good; false if some
     * error has been detected in the input stream..
     */
    fn process(&mut self, input: &[u8], offset: usize, len: usize, finish: bool) -> bool;

    /**
     * @return the maximum number of bytes a call to process()
     * could produce for the given number of input bytes.  This may
     * be an overestimate.
     */
    fn maxOutputSize(&self, len: usize) -> usize;
}

/* package */ pub struct Decoder {
    /**
     * States 0-3 are reading through the next input tuple.
     * State 4 is having read one '=' and expecting exactly
     * one more.
     * State 5 is expecting no more data or padding characters
     * in the input.
     * State 6 is the error state; an error has been detected
     * in the input and no future input can "fix" it.
     */
    pub output: Vec<u8>,
    pub op: usize,
    state: i32,   // state number (0 to 6)
    value: i32,
    alphabet: &'static [i32],
}

impl Decoder {
    /**
     * Lookup table for turning bytes into their position in the
     * Base64 alphabet.
     */
    const DECODE: [i32; 256] = [
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 62, -1, -1, -1, 63,
        52, 53, 54, 55, 56, 57, 58, 59, 60, 61, -1, -1, -1, -2, -1, -1,
        -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
        15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, -1, -1, -1, -1, -1,
        -1, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
        41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    ];
    /**
     * Decode lookup table for the "web safe" variant (RFC 3548
     * sec. 4) where - and _ replace + and /.
     */
    const DECODE_WEBSAFE: [i32; 256] = [
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 62, -1, -1,
        52, 53, 54, 55, 56, 57, 58, 59, 60, 61, -1, -1, -1, -2, -1, -1,
        -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
        15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, -1, -1, -1, -1, 63,
        -1, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
        41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    ];
    /**
     * Non-data values in the DECODE arrays.
     */
    const SKIP: i32 = -1;
    const EQUALS: i32 = -2;

    pub fn new(flags: i32, output: Vec<u8>) -> Decoder {
        Decoder {
            output,
            op: 0,
            state: 0,
            value: 0,
            alphabet: if (flags & Base64::URL_SAFE) == 0 { &Self::DECODE } else { &Self::DECODE_WEBSAFE },
        }
    }

    /**
     * @return an overestimate for the number of bytes {@code
     * len} bytes could decode to.
     */
    pub fn maxOutputSize(&self, len: usize) -> usize {
        len * 3 / 4 + 10
    }

    /**
     * Decode another block of input data.
     *
     * @return true if the state machine is still healthy.  false if
     * bad base-64 data has been detected in the input stream.
     */
    pub fn process(&mut self, input: &[u8], offset: usize, len: usize, finish: bool) -> bool {
        if self.state == 6 {
            return false;
        }
        let mut p = offset;
        let end = len + offset;
        // Using local variables makes the decoder about 12%
        // faster than if we manipulate the member variables in
        // the loop.  (Even alphabet makes a measurable
        // difference, which is somewhat surprising to me since
        // the member variable is final.)
        let mut state = self.state;
        let mut value = self.value;
        let mut op = 0usize;
        let output = &mut self.output;
        let alphabet = &self.alphabet;
        let mut d;
        while p < end {
            // Try the fast path:  we're starting a new tuple and the
            // next four bytes of the input stream are all data
            // bytes.  This corresponds to going through states
            // 0-1-2-3-0.  We expect to use this method for most of
            // the data.
            //
            // If any of the next four bytes of input are non-data
            // (whitespace, etc.), value will end up negative.  (All
            // the non-data values in decode are small negative
            // numbers, so shifting any of them up and or'ing them
            // together will result in a value with its top bit set.)
            //
            // You can remove this whole block and the output should
            // be the same, just slower.
            if state == 0 {
                while p + 4 <= end &&
                    {
                        value = (alphabet[input[p] as usize & 0xff] << 18)
                            | (alphabet[input[p + 1] as usize & 0xff] << 12)
                            | (alphabet[input[p + 2] as usize & 0xff] << 6)
                            | (alphabet[input[p + 3] as usize & 0xff]);
                        value >= 0
                    } {
                    output[op + 2] = value as u8;
                    output[op + 1] = (value >> 8) as u8;
                    output[op] = (value >> 16) as u8;
                    op += 3;
                    p += 4;
                }
                if p >= end {
                    break;
                }
            }
            // The fast path isn't available -- either we've read a
            // partial tuple, or the next four input bytes aren't all
            // data, or whatever.  Fall back to the slower state
            // machine implementation.
            d = alphabet[input[p] as usize & 0xff];
            p += 1;
            match state {
                0 => {
                    if d >= 0 {
                        value = d;
                        state += 1;
                    } else if d != Self::SKIP {
                        self.state = 6;
                        return false;
                    }
                }
                1 => {
                    if d >= 0 {
                        value = (value << 6) | d;
                        state += 1;
                    } else if d != Self::SKIP {
                        self.state = 6;
                        return false;
                    }
                }
                2 => {
                    if d >= 0 {
                        value = (value << 6) | d;
                        state += 1;
                    } else if d == Self::EQUALS {
                        // Emit the last (partial) output tuple;
                        // expect exactly one more padding character.
                        output[op] = (value >> 4) as u8;
                        op += 1;
                        state = 4;
                    } else if d != Self::SKIP {
                        self.state = 6;
                        return false;
                    }
                }
                3 => {
                    if d >= 0 {
                        // Emit the output triple and return to state 0.
                        value = (value << 6) | d;
                        output[op + 2] = value as u8;
                        output[op + 1] = (value >> 8) as u8;
                        output[op] = (value >> 16) as u8;
                        op += 3;
                        state = 0;
                    } else if d == Self::EQUALS {
                        // Emit the last (partial) output tuple;
                        // expect no further data or padding characters.
                        output[op + 1] = (value >> 2) as u8;
                        output[op] = (value >> 10) as u8;
                        op += 2;
                        state = 5;
                    } else if d != Self::SKIP {
                        self.state = 6;
                        return false;
                    }
                }
                4 => {
                    if d == Self::EQUALS {
                        state += 1;
                    } else if d != Self::SKIP {
                        self.state = 6;
                        return false;
                    }
                }
                5 => {
                    if d != Self::SKIP {
                        self.state = 6;
                        return false;
                    }
                }
                _ => {}
            }
        }
        if !finish {
            // We're out of input, but a future call could provide
            // more.
            self.state = state;
            self.value = value;
            self.op = op;
            return true;
        }
        // Done reading input.  Now figure out where we are left in
        // the state machine and finish up.
        match state {
            0 => {
                // Output length is a multiple of three.  Fine.
            }
            1 => {
                // Read one extra input byte, which isn't enough to
                // make another output byte.  Illegal.
                self.state = 6;
                return false;
            }
            2 => {
                // Read two extra input bytes, enough to emit 1 more
                // output byte.  Fine.
                output[op] = (value >> 4) as u8;
                op += 1;
            }
            3 => {
                // Read three extra input bytes, enough to emit 2 more
                // output bytes.  Fine.
                output[op] = (value >> 10) as u8;
                op += 1;
                output[op] = (value >> 2) as u8;
                op += 1;
            }
            4 => {
                // Read one padding '=' when we expected 2.  Illegal.
                self.state = 6;
                return false;
            }
            5 => {
                // Read all the padding '='s we expected and no more.
                // Fine.
            }
            _ => {}
        }
        self.state = state;
        self.op = op;
        true
    }
}

/* package */ pub struct Encoder {
    pub output: Option<Vec<u8>>,
    pub op: usize,
    tail: [u8; 2],
    /* package */ tailLen: usize,
    count: i64,
    pub do_padding: bool,
    pub do_newline: bool,
    pub do_cr: bool,
    alphabet: &'static [u8],
}

impl Encoder {
    /**
     * Emit a new line every this many output tuples.  Corresponds to
     * a 76-character line length (the maximum allowable according to
     * <a href="http://www.ietf.org/rfc/rfc2045.txt">RFC 2045</a>).
     */
    pub const LINE_GROUPS: usize = 19;
    /**
     * Lookup table for turning Base64 alphabet positions (6 bits)
     * into output bytes.
     */
    const ENCODE: [u8; 64] = [
        'A' as u8, 'B' as u8, 'C' as u8, 'D' as u8, 'E' as u8, 'F' as u8, 'G' as u8, 'H' as u8, 'I' as u8, 'J' as u8, 'K' as u8, 'L' as u8, 'M' as u8, 'N' as u8, 'O' as u8, 'P' as u8,
        'Q' as u8, 'R' as u8, 'S' as u8, 'T' as u8, 'U' as u8, 'V' as u8, 'W' as u8, 'X' as u8, 'Y' as u8, 'Z' as u8, 'a' as u8, 'b' as u8, 'c' as u8, 'd' as u8, 'e' as u8, 'f' as u8,
        'g' as u8, 'h' as u8, 'i' as u8, 'j' as u8, 'k' as u8, 'l' as u8, 'm' as u8, 'n' as u8, 'o' as u8, 'p' as u8, 'q' as u8, 'r' as u8, 's' as u8, 't' as u8, 'u' as u8, 'v' as u8,
        'w' as u8, 'x' as u8, 'y' as u8, 'z' as u8, '0' as u8, '1' as u8, '2' as u8, '3' as u8, '4' as u8, '5' as u8, '6' as u8, '7' as u8, '8' as u8, '9' as u8, '+' as u8, '/' as u8,
    ];
    /**
     * Lookup table for turning Base64 alphabet positions (6 bits)
     * into output bytes.
     */
    const ENCODE_WEBSAFE: [u8; 64] = [
        'A' as u8, 'B' as u8, 'C' as u8, 'D' as u8, 'E' as u8, 'F' as u8, 'G' as u8, 'H' as u8, 'I' as u8, 'J' as u8, 'K' as u8, 'L' as u8, 'M' as u8, 'N' as u8, 'O' as u8, 'P' as u8,
        'Q' as u8, 'R' as u8, 'S' as u8, 'T' as u8, 'U' as u8, 'V' as u8, 'W' as u8, 'X' as u8, 'Y' as u8, 'Z' as u8, 'a' as u8, 'b' as u8, 'c' as u8, 'd' as u8, 'e' as u8, 'f' as u8,
        'g' as u8, 'h' as u8, 'i' as u8, 'j' as u8, 'k' as u8, 'l' as u8, 'm' as u8, 'n' as u8, 'o' as u8, 'p' as u8, 'q' as u8, 'r' as u8, 's' as u8, 't' as u8, 'u' as u8, 'v' as u8,
        'w' as u8, 'x' as u8, 'y' as u8, 'z' as u8, '0' as u8, '1' as u8, '2' as u8, '3' as u8, '4' as u8, '5' as u8, '6' as u8, '7' as u8, '8' as u8, '9' as u8, '-' as u8, '_' as u8,
    ];

    pub fn new(flags: i32, output: Option<Vec<u8>>) -> Encoder {
        let do_padding = (flags & Base64::NO_PADDING) == 0;
        let do_newline = (flags & Base64::NO_WRAP) == 0;
        let do_cr = (flags & Base64::CRLF) != 0;
        let count = if do_newline { Self::LINE_GROUPS as i64 } else { -1 };
        Encoder {
            output,
            op: 0,
            tail: [0u8; 2],
            tailLen: 0,
            count,
            do_padding,
            do_newline,
            do_cr,
            alphabet: if (flags & Base64::URL_SAFE) == 0 { &Self::ENCODE } else { &Self::ENCODE_WEBSAFE },
        }
    }

    /**
     * @return an overestimate for the number of bytes {@code
     * len} bytes could encode to.
     */
    pub fn maxOutputSize(&self, len: usize) -> usize {
        len * 8 / 5 + 10
    }

    pub fn process(&mut self, input: &[u8], offset: usize, len: usize, finish: bool) -> bool {
        // Using local variables makes the encoder about 9% faster.
        let alphabet = self.alphabet;
        let output = self.output.as_mut().unwrap();
        let mut op = 0usize;
        let mut count = self.count;
        let mut p = offset;
        let end = len + offset;
        let mut v: i64 = -1;
        // First we need to concatenate the tail of the previous call
        // with any input bytes available now and see if we can empty
        // the tail.
        match self.tailLen {
            0 => {
                // There was no tail.
            }
            1 => {
                if p + 2 <= end {
                    // A 1-byte tail with at least 2 bytes of
                    // input available now.
                    v = ((self.tail[0] as i64 & 0xff) << 16)
                        | ((input[p] as i64 & 0xff) << 8)
                        | (input[p + 1] as i64 & 0xff);
                    p += 2;
                    self.tailLen = 0;
                }
            }
            2 => {
                if p + 1 <= end {
                    // A 2-byte tail with at least 1 byte of input.
                    v = ((self.tail[0] as i64 & 0xff) << 16)
                        | ((self.tail[1] as i64 & 0xff) << 8)
                        | (input[p] as i64 & 0xff);
                    p += 1;
                    self.tailLen = 0;
                }
            }
            _ => {}
        }
        if v != -1 {
            output[op] = alphabet[(v >> 18) as usize & 0x3f];
            op += 1;
            output[op] = alphabet[(v >> 12) as usize & 0x3f];
            op += 1;
            output[op] = alphabet[(v >> 6) as usize & 0x3f];
            op += 1;
            output[op] = alphabet[v as usize & 0x3f];
            op += 1;
            count -= 1;
            if count == 0 {
                if self.do_cr {
                    output[op] = '\r' as u8;
                    op += 1;
                }
                output[op] = '\n' as u8;
                op += 1;
                count = Self::LINE_GROUPS as i64;
            }
        }
        // At this point either there is no tail, or there are fewer
        // than 3 bytes of input available.
        // The main loop, turning 3 input bytes into 4 output bytes on
        // each iteration.
        while p + 3 <= end {
            v = ((input[p] as i64 & 0xff) << 16)
                | ((input[p + 1] as i64 & 0xff) << 8)
                | (input[p + 2] as i64 & 0xff);
            output[op] = alphabet[(v >> 18) as usize & 0x3f];
            output[op + 1] = alphabet[(v >> 12) as usize & 0x3f];
            output[op + 2] = alphabet[(v >> 6) as usize & 0x3f];
            output[op + 3] = alphabet[v as usize & 0x3f];
            p += 3;
            op += 4;
            count -= 1;
            if count == 0 {
                if self.do_cr {
                    output[op] = '\r' as u8;
                    op += 1;
                }
                output[op] = '\n' as u8;
                op += 1;
                count = Self::LINE_GROUPS as i64;
            }
        }
        if finish {
            // Finish up the tail of the input.  Note that we need to
            // consume any bytes in tail before any bytes
            // remaining in input; there should be at most two bytes
            // total.
            if p - self.tailLen == end - 1 {
                let mut t = 0usize;
                v = ((if self.tailLen > 0 { self.tail[t] } else { input[p] } as i64 & 0xff)) << 4;
                if self.tailLen > 0 {
                    t += 1;
                } else {
                    p += 1;
                }
                self.tailLen -= t;
                output[op] = alphabet[(v >> 6) as usize & 0x3f];
                op += 1;
                output[op] = alphabet[v as usize & 0x3f];
                op += 1;
                if self.do_padding {
                    output[op] = '=' as u8;
                    op += 1;
                    output[op] = '=' as u8;
                    op += 1;
                }
                if self.do_newline {
                    if self.do_cr {
                        output[op] = '\r' as u8;
                        op += 1;
                    }
                    output[op] = '\n' as u8;
                    op += 1;
                }
            } else if p - self.tailLen == end - 2 {
                let mut t = 0usize;
                v = ((if self.tailLen > 1 { self.tail[t] } else { input[p] } as i64 & 0xff) << 10)
                    | ((if self.tailLen > 0 { self.tail[t + 1] } else { input[p + 1] } as i64 & 0xff) << 2);
                if self.tailLen > 1 {
                    t += 1;
                } else {
                    p += 1;
                }
                if self.tailLen > 1 {
                    t += 1;
                } else {
                    p += 1;
                }
                self.tailLen -= t;
                output[op] = alphabet[(v >> 12) as usize & 0x3f];
                op += 1;
                output[op] = alphabet[(v >> 6) as usize & 0x3f];
                op += 1;
                output[op] = alphabet[v as usize & 0x3f];
                op += 1;
                if self.do_padding {
                    output[op] = '=' as u8;
                    op += 1;
                }
                if self.do_newline {
                    if self.do_cr {
                        output[op] = '\r' as u8;
                        op += 1;
                    }
                    output[op] = '\n' as u8;
                    op += 1;
                }
            } else if self.do_newline && op > 0 && count != Self::LINE_GROUPS as i64 {
                if self.do_cr {
                    output[op] = '\r' as u8;
                    op += 1;
                }
                output[op] = '\n' as u8;
                op += 1;
            }
            assert!(self.tailLen == 0);
            assert!(p == end);
        } else {
            // Save the leftovers in tail to be consumed on the next
            // call to encodeInternal.
            if p == end - 1 {
                self.tail[self.tailLen] = input[p];
                self.tailLen += 1;
            } else if p == end - 2 {
                self.tail[self.tailLen] = input[p];
                self.tailLen += 1;
                self.tail[self.tailLen] = input[p + 1];
                self.tailLen += 1;
            }
        }
        self.op = op;
        self.count = count;
        true
    }
}
