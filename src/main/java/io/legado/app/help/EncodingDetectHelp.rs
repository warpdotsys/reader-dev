// ============================================================================
// Rust translation of EncodingDetectHelp.java
// Pure transcription: no implementation changes.
//   class        -> struct
//   static fields/arrays -> associated consts (Encoding class translated below)
//   methods      -> fns (snake_case)
//   byte (signed)-> i8, int -> i32, long -> i64, float -> f32, boolean -> bool
//   byte[]       -> &[i8] / Vec<i8>   (i8 mirrors Java's signed byte exactly)
//   byte[][]     -> [[i32; ...]; ...]
//   Java `(byte) 0xNN` literal -> `(0xNNu8 as i8)`  (signed value preserved)
//   bare int literal vs byte   -> compared as i32 (Java int promotion preserved)
// ============================================================================

//package io.legado.app.help;

//import androidx.annotation.NonNull;
//import org.jsoup.Jsoup;
//import org.jsoup.nodes.Document;
//import org.jsoup.nodes.Element;
//import org.jsoup.select.Elements;

//import java.io.File;
//import java.io.FileInputStream;
//import java.io.InputStream;
//import java.net.URL;
//import java.nio.charset.StandardCharsets;

//import static io.legado.app.utils.TextUtils.isEmpty;

// helper: reinterpret &[i8] as &[u8] (same bytes, Java's new String(bytes, UTF_8))
fn as_u8<'a>(bytes: &'a [i8]) -> &'a [u8] {
    unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u8, bytes.len()) }
}

/**
 * <Detect encoding .> Copyright (C) <2009> <Fluck,ACC http://androidos.cc/dev>
 * <p>
 * This program is free software: you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or (at your option) any later
 * version.
 * <p>
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
 * details.
 * <p>
 * EncodingDetect.java<br>
 * 自动获取文件的编码
 *
 * @author Billows.Van
 * @version 1.0
 * @since Create on 2010-01-27 11:19:00
 */
// public class EncodingDetectHelp {
pub struct EncodingDetectHelp;

impl EncodingDetectHelp {

    // public static String getHtmlEncode( byte[] bytes) {
    pub fn get_html_encode(bytes: &[i8]) -> String {
        // try {
        //     Document doc = Jsoup.parse(new String(bytes, StandardCharsets.UTF_8));
        let doc = Jsoup::parse(String::from_utf8_lossy(as_u8(bytes)).to_string());
        //     Elements metaTags = doc.getElementsByTag("meta");
        let meta_tags = doc.get_elements_by_tag("meta");
        //     String charsetStr;
        let mut charset_str: String = String::new();
        //     for (Element metaTag : metaTags) {
        for meta_tag in meta_tags.iter() {
            //         charsetStr = metaTag.attr("charset");
            charset_str = meta_tag.attr("charset");
            //         if (!isEmpty(charsetStr)) {
            if !TextUtils::is_empty(&charset_str) {
                //             return charsetStr;
                return charset_str;
            }
            //         String content = metaTag.attr("content");
            let content = meta_tag.attr("content");
            //         String http_equiv = metaTag.attr("http-equiv");
            let http_equiv = meta_tag.attr("http-equiv");
            //         if (http_equiv.toLowerCase().equals("content-type")) {
            if http_equiv.to_lowercase() == "content-type" {
                //             if (content.toLowerCase().contains("charset")) {
                if content.to_lowercase().contains("charset") {
                    //                 charsetStr = content.substring(content.toLowerCase().indexOf("charset") + "charset=".length());
                    charset_str = content[content.to_lowercase().find("charset").unwrap() + "charset=".len()..].to_string();
                } else {
                    //                 charsetStr = content.substring(content.toLowerCase().indexOf(";") + 1);
                    charset_str = content[content.to_lowercase().find(';').map_or(0, |x| x + 1)..].to_string();
                }
                //             if (!isEmpty(charsetStr)) {
                if !TextUtils::is_empty(&charset_str) {
                    //                 return charsetStr;
                    return charset_str;
                }
            }
        }
        //     }
        // } catch (Exception ignored) {
        // }
        // return getJavaEncode(bytes);
        get_java_encode(bytes)
    }

    // public static String getJavaEncode(byte[] bytes) {
    pub fn get_java_encode(bytes: &[i8]) -> String {
        // int len = bytes.length > 2000 ? 2000 : bytes.length;
        let len = if bytes.len() > 2000 { 2000 } else { bytes.len() };
        // byte[] cBytes = new byte[len];
        let mut c_bytes: Vec<i8> = vec![0; len];
        // System.arraycopy(bytes, 0, cBytes, 0, len);
        c_bytes[..len].copy_from_slice(&bytes[..len]);
        // BytesEncodingDetect bytesEncodingDetect = new BytesEncodingDetect();
        let bytes_encoding_detect = BytesEncodingDetect::new();
        // String code = BytesEncodingDetect.javaname[bytesEncodingDetect.detectEncoding(cBytes)];
        let mut code = Encoding::JAVANAME[bytes_encoding_detect.detect_encoding(&c_bytes) as usize].to_string();
        // UTF-16LE 特殊处理
        // if ("Unicode".equals(code)) {
        if "Unicode" == code {
            //     if (cBytes[0] == -1) {
            if c_bytes[0] == -1 {
                //         code = "UTF-16LE";
                code = "UTF-16LE".to_string();
            }
        }
        // return code;
        code
    }

    /**
     * 得到文件的编码
     */
    // public static String getJavaEncode( String filePath) {
    pub fn get_java_encode_path(file_path: &str) -> String {
        // BytesEncodingDetect s = new BytesEncodingDetect();
        let s = BytesEncodingDetect::new();
        // String fileCode = BytesEncodingDetect.javaname[s
        //         .detectEncoding(new File(filePath))];
        let mut file_code = Encoding::JAVANAME[s
            .detect_encoding_file(&std::fs::File::open(file_path).unwrap()) as usize].to_string();

        // UTF-16LE 特殊处理
        // if ("Unicode".equals(fileCode)) {
        if "Unicode" == file_code {
            //     byte[] tempByte = BytesEncodingDetect.getFileBytes(new File(
            //             filePath));
            let temp_byte = BytesEncodingDetect::get_file_bytes(&std::fs::File::open(file_path).unwrap());
            //     if (tempByte[0] == -1) {
            if temp_byte[0] == -1 {
                //         fileCode = "UTF-16LE";
                file_code = "UTF-16LE".to_string();
            }
        }
        // return fileCode;
        file_code
    }

    /**
     * 得到文件的编码
     */
    // public static String getJavaEncode( File file) {
    pub fn get_java_encode_file(file: &std::fs::File) -> String {
        // BytesEncodingDetect s = new BytesEncodingDetect();
        let s = BytesEncodingDetect::new();
        // String fileCode = BytesEncodingDetect.javaname[s.detectEncoding(file)];
        let mut file_code = Encoding::JAVANAME[s.detect_encoding_file(file) as usize].to_string();
        // UTF-16LE 特殊处理
        // if ("Unicode".equals(fileCode)) {
        if "Unicode" == file_code {
            //     byte[] tempByte = BytesEncodingDetect.getFileBytes(file);
            let temp_byte = BytesEncodingDetect::get_file_bytes(file);
            //     if (tempByte[0] == -1) {
            if temp_byte[0] == -1 {
                //         fileCode = "UTF-16LE";
                file_code = "UTF-16LE".to_string();
            }
        }
        // return fileCode;
        file_code
    }

}

// class BytesEncodingDetect extends Encoding {
pub struct BytesEncodingDetect {
    // Frequency tables to hold the GB, Big5, and EUC-TW character
    // frequencies
    GBFreq: [[i32; 94]; 94],

    GBKFreq: [[i32; 191]; 126],

    Big5Freq: [[i32; 158]; 94],

    Big5PFreq: [[i32; 191]; 126],

    EUC_TWFreq: [[i32; 94]; 94],

    KRFreq: [[i32; 94]; 94],

    JPFreq: [[i32; 94]; 94],

    // int UnicodeFreq[94][128];
    // public static String[] nicename;
    // public static String[] codings;
    pub debug: bool,
}

impl BytesEncodingDetect {

    // public BytesEncodingDetect() {
    pub fn new() -> BytesEncodingDetect {
        // super();
        let mut s = BytesEncodingDetect {
            debug: false,
            // GBFreq = new int[94][94];
            GBFreq: [[0i32; 94]; 94],
            // GBKFreq = new int[126][191];
            GBKFreq: [[0i32; 191]; 126],
            // Big5Freq = new int[94][158];
            Big5Freq: [[0i32; 158]; 94],
            // Big5PFreq = new int[126][191];
            Big5PFreq: [[0i32; 191]; 126],
            // EUC_TWFreq = new int[94][94];
            EUC_TWFreq: [[0i32; 94]; 94],
            // KRFreq = new int[94][94];
            KRFreq: [[0i32; 94]; 94],
            // JPFreq = new int[94][94];
            JPFreq: [[0i32; 94]; 94],
        };
        // Initialize the Frequency Table for GB, GBK, Big5, EUC-TW, KR, JP
        s.initialize_frequencies();
        s
    }

    /**
     * Function : detectEncoding Aruguments: URL Returns : One of the encodings
     * from the Encoding enumeration (GB2312, HZ, BIG5, EUC_TW, ASCII, or OTHER)
     * Description: This function looks at the URL contents and assigns it a
     * probability score for each encoding type. The encoding type with the
     * highest probability is returned.
     */
    // public int detectEncoding(URL testurl) {
    pub fn detect_encoding_url(&self, testurl: &Url) -> i32 {
        // byte[] rawtext = new byte[10000];
        let mut rawtext: Vec<i8> = vec![0; 10000];
        // int bytesread = 0, byteoffset = 0;
        let mut bytesread: i32 = 0;
        let mut byteoffset: i32 = 0;
        // int guess = OTHER;
        let mut guess: i32 = Encoding::OTHER;
        // InputStream chinesestream;
        // try {
        //     chinesestream = testurl.openStream();
        let mut chinesestream = testurl.open_stream();
        //     while ((bytesread = chinesestream.read(rawtext, byteoffset,
        //             rawtext.length - byteoffset)) > 0) {
        loop {
            bytesread = chinesestream.read(&mut rawtext, byteoffset, rawtext.len() as i32 - byteoffset);
            if bytesread <= 0 {
                break;
            }
            //         byteoffset += bytesread;
            byteoffset += bytesread;
        }
        //     ;
        //     chinesestream.close();
        chinesestream.close();
        //     guess = detectEncoding(rawtext);
        guess = self.detect_encoding(&rawtext);
        // } catch (Exception e) {
        //     System.err.println("Error loading or using URL " + e.toString());
        //     guess = -1;
        // }
        // return guess;
        guess
    }

    /**
     * Function : detectEncoding Aruguments: File Returns : One of the encodings
     * from the Encoding enumeration (GB2312, HZ, BIG5, EUC_TW, ASCII, or OTHER)
     * Description: This function looks at the file and assigns it a probability
     * score for each encoding type. The encoding type with the highest
     * probability is returned.
     */
    // public int detectEncoding(File testfile) {
    pub fn detect_encoding_file(&self, testfile: &std::fs::File) -> i32 {
        // byte[] rawtext = getFileBytes(testfile);
        let rawtext: Vec<i8> = BytesEncodingDetect::get_file_bytes(testfile);
        // return detectEncoding(rawtext);
        self.detect_encoding(&rawtext)
    }

    // public static byte[] getFileBytes(File testfile) {
    pub fn get_file_bytes(testfile: &std::fs::File) -> Vec<i8> {
        // FileInputStream chinesefile;
        // byte[] rawtext;
        // rawtext = new byte[2000];
        let mut rawtext: Vec<i8> = vec![0; 2000];
        // try {
        //     chinesefile = new FileInputStream(testfile);
        //     chinesefile.read(rawtext);
        //     chinesefile.close();
        // } catch (Exception e) {
        //     System.err.println("Error: " + e);
        // }
        let mut chinesefile = std::io::BufReader::new(testfile);
        let mut buf = [0u8; 2000];
        let n = chinesefile.read(&mut buf).unwrap_or(0);
        for i in 0..n {
            rawtext[i] = buf[i] as i8;
        }
        // return rawtext;
        rawtext
    }


    /**
     * Function : detectEncoding Aruguments: byte array Returns : One of the
     * encodings from the Encoding enumeration (GB2312, HZ, BIG5, EUC_TW, ASCII,
     * or OTHER) Description: This function looks at the byte array and assigns
     * it a probability score for each encoding type. The encoding type with the
     * highest probability is returned.
     */
    // public int detectEncoding(byte[] rawtext) {
    pub fn detect_encoding(&self, rawtext: &[i8]) -> i32 {
        // int[] scores;
        let mut scores: Vec<i32>;
        // int index, maxscore = 0;
        let mut index: i32;
        let mut maxscore: i32 = 0;
        // int encoding_guess = OTHER;
        let mut encoding_guess: i32 = Encoding::OTHER;
        // scores = new int[TOTALTYPES];
        scores = vec![0; Encoding::TOTALTYPES as usize];
        // Assign Scores
        // scores[GB2312] = gb2312_probability(rawtext);
        scores[Encoding::GB2312 as usize] = self.gb2312_probability(rawtext);
        // scores[GBK] = gbk_probability(rawtext);
        scores[Encoding::GBK as usize] = self.gbk_probability(rawtext);
        // scores[GB18030] = gb18030_probability(rawtext);
        scores[Encoding::GB18030 as usize] = self.gb18030_probability(rawtext);
        // scores[HZ] = hz_probability(rawtext);
        scores[Encoding::HZ as usize] = self.hz_probability(rawtext);
        // scores[BIG5] = big5_probability(rawtext);
        scores[Encoding::BIG5 as usize] = self.big5_probability(rawtext);
        // scores[CNS11643] = euc_tw_probability(rawtext);
        scores[Encoding::CNS11643 as usize] = self.euc_tw_probability(rawtext);
        // scores[ISO2022CN] = iso_2022_cn_probability(rawtext);
        scores[Encoding::ISO2022CN as usize] = self.iso_2022_cn_probability(rawtext);
        // scores[UTF8] = utf8_probability(rawtext);
        scores[Encoding::UTF8 as usize] = self.utf8_probability(rawtext);
        // scores[UNICODE] = utf16_probability(rawtext);
        scores[Encoding::UNICODE as usize] = self.utf16_probability(rawtext);
        // scores[EUC_KR] = euc_kr_probability(rawtext);
        scores[Encoding::EUC_KR as usize] = self.euc_kr_probability(rawtext);
        // scores[CP949] = cp949_probability(rawtext);
        scores[Encoding::CP949 as usize] = self.cp949_probability(rawtext);
        // scores[JOHAB] = 0;
        scores[Encoding::JOHAB as usize] = 0;
        // scores[ISO2022KR] = iso_2022_kr_probability(rawtext);
        scores[Encoding::ISO2022KR as usize] = self.iso_2022_kr_probability(rawtext);
        // scores[ASCII] = ascii_probability(rawtext);
        scores[Encoding::ASCII as usize] = self.ascii_probability(rawtext);
        // scores[SJIS] = sjis_probability(rawtext);
        scores[Encoding::SJIS as usize] = self.sjis_probability(rawtext);
        // scores[EUC_JP] = euc_jp_probability(rawtext);
        scores[Encoding::EUC_JP as usize] = self.euc_jp_probability(rawtext);
        // scores[ISO2022JP] = iso_2022_jp_probability(rawtext);
        scores[Encoding::ISO2022JP as usize] = self.iso_2022_jp_probability(rawtext);
        // scores[UNICODET] = 0;
        scores[Encoding::UNICODET as usize] = 0;
        // scores[UNICODES] = 0;
        scores[Encoding::UNICODES as usize] = 0;
        // scores[ISO2022CN_GB] = 0;
        scores[Encoding::ISO2022CN_GB as usize] = 0;
        // scores[ISO2022CN_CNS] = 0;
        scores[Encoding::ISO2022CN_CNS as usize] = 0;
        // scores[OTHER] = 0;
        scores[Encoding::OTHER as usize] = 0;
        // Tabulate Scores
        // for (index = 0; index < TOTALTYPES; index++) {
        for index in 0..Encoding::TOTALTYPES {
            //     if (debug)
            if self.debug {
                //         System.err.println("Encoding " + nicename[index] + " score "
                //                 + scores[index]);
                eprintln!("Encoding {} score {}", Encoding::NICENAME[index as usize], scores[index as usize]);
            }
            //     if (scores[index] > maxscore) {
            if scores[index as usize] > maxscore {
                //         encoding_guess = index;
                encoding_guess = index;
                //         maxscore = scores[index];
                maxscore = scores[index as usize];
            }
        }
        // Return OTHER if nothing scored above 50
        // if (maxscore <= 50) {
        if maxscore <= 50 {
            //     encoding_guess = OTHER;
            encoding_guess = Encoding::OTHER;
        }
        // return encoding_guess;
        encoding_guess
    }

    /*
     * Function: gb2312_probability Argument: pointer to byte array Returns :
     * number from 0 to 100 representing probability text in array uses GB-2312
     * encoding
     */
    // int gb2312_probability(byte[] rawtext) {
    fn gb2312_probability(&self, rawtext: &[i8]) -> i32 {
        // int i, rawtextlen = 0;
        let mut i: i32 = 0;
        let mut rawtextlen: i32 = 0;
        // int dbchars = 1, gbchars = 1;
        let mut dbchars: i32 = 1;
        let mut gbchars: i32 = 1;
        // long gbfreq = 0, totalfreq = 1;
        let mut gbfreq: i64 = 0;
        let mut totalfreq: i64 = 1;
        // float rangeval = 0, freqval = 0;
        let mut rangeval: f32 = 0.0;
        let mut freqval: f32 = 0.0;
        // int row, column;
        let mut row: i32 = 0;
        let mut column: i32 = 0;
        // Stage 1: Check to see if characters fit into acceptable ranges
        // rawtextlen = rawtext.length;
        rawtextlen = rawtext.len() as i32;
        // for (i = 0; i < rawtextlen - 1; i++) {
        i = 0;
        while i < rawtextlen - 1 {
            // System.err.println(rawtext[i]);
            //     if (rawtext[i] >= 0) {
            if rawtext[i as usize] >= 0 {
                //         // asciichars++;
                //     } else {
            } else {
                //         dbchars++;
                dbchars += 1;
                //         if ((byte) 0xA1 <= rawtext[i] && rawtext[i] <= (byte) 0xF7
                //                 && (byte) 0xA1 <= rawtext[i + 1]
                //                 && rawtext[i + 1] <= (byte) 0xFE) {
                if (0xA1u8 as i8) <= rawtext[i as usize] && rawtext[i as usize] <= (0xF7u8 as i8)
                    && (0xA1u8 as i8) <= rawtext[i as usize + 1]
                    && rawtext[i as usize + 1] <= (0xFEu8 as i8)
                {
                    //             gbchars++;
                    gbchars += 1;
                    //             totalfreq += 500;
                    totalfreq += 500i64;
                    //             row = rawtext[i] + 256 - 0xA1;
                    row = rawtext[i as usize] as i32 + 256 - 0xA1;
                    //             column = rawtext[i + 1] + 256 - 0xA1;
                    column = rawtext[i as usize + 1] as i32 + 256 - 0xA1;
                    //             if (GBFreq[row][column] != 0) {
                    if self.GBFreq[row as usize][column as usize] != 0 {
                        //                 gbfreq += GBFreq[row][column];
                        gbfreq += self.GBFreq[row as usize][column as usize] as i64;
                    //             } else if (15 <= row && row < 55) {
                    } else if 15 <= row && row < 55 {
                        //                 // In GB high-freq character range
                        //                 gbfreq += 200;
                        gbfreq += 200i64;
                    }
                }
                //         }
                //         i++;
                i += 1;
            }
            //     }
            i += 1;
        }
        // rangeval = 50 * ((float) gbchars / (float) dbchars);
        rangeval = 50.0 * (gbchars as f32 / dbchars as f32);
        // freqval = 50 * ((float) gbfreq / (float) totalfreq);
        freqval = 50.0 * (gbfreq as f32 / totalfreq as f32);
        // return (int) (rangeval + freqval);
        (rangeval + freqval) as i32
    }

    /*
     * Function: gbk_probability Argument: pointer to byte array Returns :
     * number from 0 to 100 representing probability text in array uses GBK
     * encoding
     */
    // int gbk_probability(byte[] rawtext) {
    fn gbk_probability(&self, rawtext: &[i8]) -> i32 {
        // int i, rawtextlen = 0;
        let mut i: i32 = 0;
        let mut rawtextlen: i32 = 0;
        // int dbchars = 1, gbchars = 1;
        let mut dbchars: i32 = 1;
        let mut gbchars: i32 = 1;
        // long gbfreq = 0, totalfreq = 1;
        let mut gbfreq: i64 = 0;
        let mut totalfreq: i64 = 1;
        // float rangeval = 0, freqval = 0;
        let mut rangeval: f32 = 0.0;
        let mut freqval: f32 = 0.0;
        // int row, column;
        let mut row: i32 = 0;
        let mut column: i32 = 0;
        // Stage 1: Check to see if characters fit into acceptable ranges
        // rawtextlen = rawtext.length;
        rawtextlen = rawtext.len() as i32;
        // for (i = 0; i < rawtextlen - 1; i++) {
        i = 0;
        while i < rawtextlen - 1 {
            // System.err.println(rawtext[i]);
            //     if (rawtext[i] >= 0) {
            if rawtext[i as usize] >= 0 {
                //         // asciichars++;
                //     } else {
            } else {
                //         dbchars++;
                dbchars += 1;
                //         if ((byte) 0xA1 <= rawtext[i] && rawtext[i] <= (byte) 0xF7
                //                 && // Original GB range
                //                 (byte) 0xA1 <= rawtext[i + 1]
                //                 && rawtext[i + 1] <= (byte) 0xFE) {
                if (0xA1u8 as i8) <= rawtext[i as usize] && rawtext[i as usize] <= (0xF7u8 as i8)
                    && // Original GB range
                    (0xA1u8 as i8) <= rawtext[i as usize + 1]
                    && rawtext[i as usize + 1] <= (0xFEu8 as i8)
                {
                    //             gbchars++;
                    gbchars += 1;
                    //             totalfreq += 500;
                    totalfreq += 500i64;
                    //             row = rawtext[i] + 256 - 0xA1;
                    row = rawtext[i as usize] as i32 + 256 - 0xA1;
                    //             column = rawtext[i + 1] + 256 - 0xA1;
                    column = rawtext[i as usize + 1] as i32 + 256 - 0xA1;
                    //             // System.out.println("original row " + row + " column " +
                    //             // column);
                    //             if (GBFreq[row][column] != 0) {
                    if self.GBFreq[row as usize][column as usize] != 0 {
                        //                 gbfreq += GBFreq[row][column];
                        gbfreq += self.GBFreq[row as usize][column as usize] as i64;
                    //             } else if (15 <= row && row < 55) {
                    } else if 15 <= row && row < 55 {
                        //                 gbfreq += 200;
                        gbfreq += 200i64;
                    }
                //         } else if ((byte) 0x81 <= rawtext[i]
                //                 && rawtext[i] <= (byte) 0xFE && // Extended GB range
                //                 (((byte) 0x80 <= rawtext[i + 1] && rawtext[i + 1] <= (byte) 0xFE) || ((byte) 0x40 <= rawtext[i + 1] && rawtext[i + 1] <= (byte) 0x7E))) {
                } else if (0x81u8 as i8) <= rawtext[i as usize]
                    && rawtext[i as usize] <= (0xFEu8 as i8) && // Extended GB range
                    (((0x80u8 as i8) <= rawtext[i as usize + 1] && rawtext[i as usize + 1] <= (0xFEu8 as i8)) || ((0x40u8 as i8) <= rawtext[i as usize + 1] && rawtext[i as usize + 1] <= (0x7Eu8 as i8)))
                {
                    //             gbchars++;
                    gbchars += 1;
                    //             totalfreq += 500;
                    totalfreq += 500i64;
                    //             row = rawtext[i] + 256 - 0x81;
                    row = rawtext[i as usize] as i32 + 256 - 0x81;
                    //             if (0x40 <= rawtext[i + 1] && rawtext[i + 1] <= 0x7E) {
                    if 0x40 <= rawtext[i as usize + 1] as i32 && rawtext[i as usize + 1] as i32 <= 0x7E {
                        //                 column = rawtext[i + 1] - 0x40;
                        column = rawtext[i as usize + 1] as i32 - 0x40;
                    //             } else {
                    } else {
                        //                 column = rawtext[i + 1] + 256 - 0x40;
                        column = rawtext[i as usize + 1] as i32 + 256 - 0x40;
                    }
                    //             // System.out.println("extended row " + row + " column " +
                    //             // column + " rawtext[i] " + rawtext[i]);
                    //             if (GBKFreq[row][column] != 0) {
                    if self.GBKFreq[row as usize][column as usize] != 0 {
                        //                 gbfreq += GBKFreq[row][column];
                        gbfreq += self.GBKFreq[row as usize][column as usize] as i64;
                    }
                }
                //         }
                //         i++;
                i += 1;
            }
            //     }
            i += 1;
        }
        // rangeval = 50 * ((float) gbchars / (float) dbchars);
        rangeval = 50.0 * (gbchars as f32 / dbchars as f32);
        // freqval = 50 * ((float) gbfreq / (float) totalfreq);
        freqval = 50.0 * (gbfreq as f32 / totalfreq as f32);
        // For regular GB files, this would give the same score, so I handicap
        // it slightly
        // return (int) (rangeval + freqval) - 1;
        (rangeval + freqval) as i32 - 1
    }
    /*
     * Function: gb18030_probability Argument: pointer to byte array Returns :
     * number from 0 to 100 representing probability text in array uses GBK
     * encoding
     */
    // int gb18030_probability(byte[] rawtext) {
    fn gb18030_probability(&self, rawtext: &[i8]) -> i32 {
        // int i, rawtextlen = 0;
        let mut i: i32 = 0;
        let mut rawtextlen: i32 = 0;
        // int dbchars = 1, gbchars = 1;
        let mut dbchars: i32 = 1;
        let mut gbchars: i32 = 1;
        // long gbfreq = 0, totalfreq = 1;
        let mut gbfreq: i64 = 0;
        let mut totalfreq: i64 = 1;
        // float rangeval = 0, freqval = 0;
        let mut rangeval: f32 = 0.0;
        let mut freqval: f32 = 0.0;
        // int row, column;
        let mut row: i32 = 0;
        let mut column: i32 = 0;
        // Stage 1: Check to see if characters fit into acceptable ranges
        // rawtextlen = rawtext.length;
        rawtextlen = rawtext.len() as i32;
        // for (i = 0; i < rawtextlen - 1; i++) {
        i = 0;
        while i < rawtextlen - 1 {
            // System.err.println(rawtext[i]);
            //     if (rawtext[i] >= 0) {
            if rawtext[i as usize] >= 0 {
                //         // asciichars++;
                //     } else {
            } else {
                //         dbchars++;
                dbchars += 1;
                //         if ((byte) 0xA1 <= rawtext[i] && rawtext[i] <= (byte) 0xF7
                //                 && // Original GB range
                //                 i + 1 < rawtextlen && (byte) 0xA1 <= rawtext[i + 1]
                //                 && rawtext[i + 1] <= (byte) 0xFE) {
                if (0xA1u8 as i8) <= rawtext[i as usize] && rawtext[i as usize] <= (0xF7u8 as i8)
                    && // Original GB range
                    i + 1 < rawtextlen && (0xA1u8 as i8) <= rawtext[i as usize + 1]
                    && rawtext[i as usize + 1] <= (0xFEu8 as i8)
                {
                    //             gbchars++;
                    gbchars += 1;
                    //             totalfreq += 500;
                    totalfreq += 500i64;
                    //             row = rawtext[i] + 256 - 0xA1;
                    row = rawtext[i as usize] as i32 + 256 - 0xA1;
                    //             column = rawtext[i + 1] + 256 - 0xA1;
                    column = rawtext[i as usize + 1] as i32 + 256 - 0xA1;
                    //             // System.out.println("original row " + row + " column " +
                    //             // column);
                    //             if (GBFreq[row][column] != 0) {
                    if self.GBFreq[row as usize][column as usize] != 0 {
                        //                 gbfreq += GBFreq[row][column];
                        gbfreq += self.GBFreq[row as usize][column as usize] as i64;
                    //             } else if (15 <= row && row < 55) {
                    } else if 15 <= row && row < 55 {
                        //                 gbfreq += 200;
                        gbfreq += 200i64;
                    }
                //         } else if ((byte) 0x81 <= rawtext[i]
                //                 && rawtext[i] <= (byte) 0xFE
                //                 && // Extended GB range
                //                 i + 1 < rawtextlen
                //                 && (((byte) 0x80 <= rawtext[i + 1] && rawtext[i + 1] <= (byte) 0xFE) || ((byte) 0x40 <= rawtext[i + 1] && rawtext[i + 1] <= (byte) 0x7E))) {
                } else if (0x81u8 as i8) <= rawtext[i as usize]
                    && rawtext[i as usize] <= (0xFEu8 as i8)
                    && // Extended GB range
                    i + 1 < rawtextlen
                    && (((0x80u8 as i8) <= rawtext[i as usize + 1] && rawtext[i as usize + 1] <= (0xFEu8 as i8)) || ((0x40u8 as i8) <= rawtext[i as usize + 1] && rawtext[i as usize + 1] <= (0x7Eu8 as i8)))
                {
                    //             gbchars++;
                    gbchars += 1;
                    //             totalfreq += 500;
                    totalfreq += 500i64;
                    //             row = rawtext[i] + 256 - 0x81;
                    row = rawtext[i as usize] as i32 + 256 - 0x81;
                    //             if (0x40 <= rawtext[i + 1] && rawtext[i + 1] <= 0x7E) {
                    if 0x40 <= rawtext[i as usize + 1] as i32 && rawtext[i as usize + 1] as i32 <= 0x7E {
                        //                 column = rawtext[i + 1] - 0x40;
                        column = rawtext[i as usize + 1] as i32 - 0x40;
                    //             } else {
                    } else {
                        //                 column = rawtext[i + 1] + 256 - 0x40;
                        column = rawtext[i as usize + 1] as i32 + 256 - 0x40;
                    }
                    //             // System.out.println("extended row " + row + " column " +
                    //             // column + " rawtext[i] " + rawtext[i]);
                    //             if (GBKFreq[row][column] != 0) {
                    if self.GBKFreq[row as usize][column as usize] != 0 {
                        //                 gbfreq += GBKFreq[row][column];
                        gbfreq += self.GBKFreq[row as usize][column as usize] as i64;
                    }
                //         } else if ((byte) 0x81 <= rawtext[i]
                //                 && rawtext[i] <= (byte) 0xFE
                //                 && // Extended GB range
                //                 i + 3 < rawtextlen && (byte) 0x30 <= rawtext[i + 1]
                //                 && rawtext[i + 1] <= (byte) 0x39
                //                 && (byte) 0x81 <= rawtext[i + 2]
                //                 && rawtext[i + 2] <= (byte) 0xFE
                //                 && (byte) 0x30 <= rawtext[i + 3]
                //                 && rawtext[i + 3] <= (byte) 0x39) {
                } else if (0x81u8 as i8) <= rawtext[i as usize]
                    && rawtext[i as usize] <= (0xFEu8 as i8)
                    && // Extended GB range
                    i + 3 < rawtextlen && (0x30u8 as i8) <= rawtext[i as usize + 1]
                    && rawtext[i as usize + 1] <= (0x39u8 as i8)
                    && (0x81u8 as i8) <= rawtext[i as usize + 2]
                    && rawtext[i as usize + 2] <= (0xFEu8 as i8)
                    && (0x30u8 as i8) <= rawtext[i as usize + 3]
                    && rawtext[i as usize + 3] <= (0x39u8 as i8)
                {
                    //             gbchars++;
                    gbchars += 1;
                    /*
                     * totalfreq += 500; row = rawtext[i] + 256 - 0x81; if (0x40
                     * <= rawtext[i+1] && rawtext[i+1] <= 0x7E) { column =
                     * rawtext[i+1] - 0x40; } else { column = rawtext[i+1] + 256
                     * - 0x40; } //System.out.println("extended row " + row + "
                     * column " + column + " rawtext[i] " + rawtext[i]); if
                     * (GBKFreq[row][column] != 0) { gbfreq +=
                     * GBKFreq[row][column]; }
                     */
                }
                //         }
                //         i++;
                i += 1;
            }
            //     }
            i += 1;
        }
        // rangeval = 50 * ((float) gbchars / (float) dbchars);
        rangeval = 50.0 * (gbchars as f32 / dbchars as f32);
        // freqval = 50 * ((float) gbfreq / (float) totalfreq);
        freqval = 50.0 * (gbfreq as f32 / totalfreq as f32);
        // For regular GB files, this would give the same score, so I handicap
        // it slightly
        // return (int) (rangeval + freqval) - 1;
        (rangeval + freqval) as i32 - 1
    }

    /*
     * Function: hz_probability Argument: byte array Returns : number from 0 to
     * 100 representing probability text in array uses HZ encoding
     */
    // int hz_probability(byte[] rawtext) {
    fn hz_probability(&self, rawtext: &[i8]) -> i32 {
        // int i, rawtextlen;
        let mut i: i32 = 0;
        let mut rawtextlen: i32 = 0;
        // int hzchars = 0, dbchars = 1;
        let mut hzchars: i32 = 0;
        let mut dbchars: i32 = 1;
        // long hzfreq = 0, totalfreq = 1;
        let mut hzfreq: i64 = 0;
        let mut totalfreq: i64 = 1;
        // float rangeval = 0, freqval = 0;
        let mut rangeval: f32 = 0.0;
        let mut freqval: f32 = 0.0;
        // int hzstart = 0, hzend = 0;
        let mut hzstart: i32 = 0;
        let mut hzend: i32 = 0;
        // int row, column;
        let mut row: i32 = 0;
        let mut column: i32 = 0;
        // rawtextlen = rawtext.length;
        rawtextlen = rawtext.len() as i32;
        // for (i = 0; i < rawtextlen; i++) {
        i = 0;
        while i < rawtextlen {
            //     if (rawtext[i] == '~') {
            if rawtext[i as usize] == (b'~' as i8) {
                //         if (rawtext[i + 1] == '{') {
                if rawtext[i as usize + 1] == (b'{' as i8) {
                    //             hzstart++;
                    hzstart += 1;
                    //             i += 2;
                    i += 2;
                    //             while (i < rawtextlen - 1) {
                    while i < rawtextlen - 1 {
                        //                 if (rawtext[i] == 0x0A || rawtext[i] == 0x0D) {
                        if rawtext[i as usize] == 0x0A || rawtext[i as usize] == 0x0D {
                            //                     break;
                            break;
                        //                 } else if (rawtext[i] == '~' && rawtext[i + 1] == '}') {
                        } else if rawtext[i as usize] == (b'~' as i8) && rawtext[i as usize + 1] == (b'}' as i8) {
                            //                     hzend++;
                            hzend += 1;
                            //                     i++;
                            i += 1;
                            //                     break;
                            break;
                        //                 } else if ((0x21 <= rawtext[i] && rawtext[i] <= 0x77)
                        //                         && (0x21 <= rawtext[i + 1] && rawtext[i + 1] <= 0x77)) {
                        } else if (0x21 <= rawtext[i as usize] as i32 && rawtext[i as usize] as i32 <= 0x77)
                            && (0x21 <= rawtext[i as usize + 1] as i32 && rawtext[i as usize + 1] as i32 <= 0x77)
                        {
                            //                     hzchars += 2;
                            hzchars += 2;
                            //                     row = rawtext[i] - 0x21;
                            row = rawtext[i as usize] as i32 - 0x21;
                            //                     column = rawtext[i + 1] - 0x21;
                            column = rawtext[i as usize + 1] as i32 - 0x21;
                            //                     totalfreq += 500;
                            totalfreq += 500i64;
                            //                     if (GBFreq[row][column] != 0) {
                            if self.GBFreq[row as usize][column as usize] != 0 {
                                //                         hzfreq += GBFreq[row][column];
                                hzfreq += self.GBFreq[row as usize][column as usize] as i64;
                            //                     } else if (15 <= row && row < 55) {
                            } else if 15 <= row && row < 55 {
                                //                         hzfreq += 200;
                                hzfreq += 200i64;
                            }
                        //                 } else if ((0xA1 <= rawtext[i] && rawtext[i] <= 0xF7)
                        //                         && (0xA1 <= rawtext[i + 1] && rawtext[i + 1] <= 0xF7)) {
                        } else if (0xA1 <= rawtext[i as usize] as i32 && rawtext[i as usize] as i32 <= 0xF7)
                            && (0xA1 <= rawtext[i as usize + 1] as i32 && rawtext[i as usize + 1] as i32 <= 0xF7)
                        {
                            //                     hzchars += 2;
                            hzchars += 2;
                            //                     row = rawtext[i] + 256 - 0xA1;
                            row = rawtext[i as usize] as i32 + 256 - 0xA1;
                            //                     column = rawtext[i + 1] + 256 - 0xA1;
                            column = rawtext[i as usize + 1] as i32 + 256 - 0xA1;
                            //                     totalfreq += 500;
                            totalfreq += 500i64;
                            //                     if (GBFreq[row][column] != 0) {
                            if self.GBFreq[row as usize][column as usize] != 0 {
                                //                         hzfreq += GBFreq[row][column];
                                hzfreq += self.GBFreq[row as usize][column as usize] as i64;
                            //                     } else if (15 <= row && row < 55) {
                            } else if 15 <= row && row < 55 {
                                //                         hzfreq += 200;
                                hzfreq += 200i64;
                            }
                        }
                        //                 dbchars += 2;
                        dbchars += 2;
                        //                 i += 2;
                        i += 2;
                    }
                //         } else if (rawtext[i + 1] == '}') {
                } else if rawtext[i as usize + 1] == (b'}' as i8) {
                    //             hzend++;
                    hzend += 1;
                    //             i++;
                    i += 1;
                //         } else if (rawtext[i + 1] == '~') {
                } else if rawtext[i as usize + 1] == (b'~' as i8) {
                    //             i++;
                    i += 1;
                }
            }
            //     }
            i += 1;
        }
        // if (hzstart > 4) {
        if hzstart > 4 {
            //     rangeval = 50;
            rangeval = 50.0;
        // } else if (hzstart > 1) {
        } else if hzstart > 1 {
            //     rangeval = 41;
            rangeval = 41.0;
        // } else if (hzstart > 0) { // Only 39 in case the sequence happened to
        //     rangeval = 39; // in otherwise non-Hz text
        } else if hzstart > 0 {
            // Only 39 in case the sequence happened to
            // occur
            rangeval = 39.0; // in otherwise non-Hz text
        // } else {
        } else {
            //     rangeval = 0;
            rangeval = 0.0;
        }
        // freqval = 50 * ((float) hzfreq / (float) totalfreq);
        freqval = 50.0 * (hzfreq as f32 / totalfreq as f32);
        // return (int) (rangeval + freqval);
        (rangeval + freqval) as i32
    }

    /**
     * Function: big5_probability Argument: byte array Returns : number from 0
     * to 100 representing probability text in array uses Big5 encoding
     */
    // int big5_probability(byte[] rawtext) {
    fn big5_probability(&self, rawtext: &[i8]) -> i32 {
        // int i, rawtextlen = 0;
        let mut i: i32 = 0;
        let mut rawtextlen: i32 = 0;
        // int dbchars = 1, bfchars = 1;
        let mut dbchars: i32 = 1;
        let mut bfchars: i32 = 1;
        // float rangeval = 0, freqval = 0;
        let mut rangeval: f32 = 0.0;
        let mut freqval: f32 = 0.0;
        // long bffreq = 0, totalfreq = 1;
        let mut bffreq: i64 = 0;
        let mut totalfreq: i64 = 1;
        // int row, column;
        let mut row: i32 = 0;
        let mut column: i32 = 0;
        // Check to see if characters fit into acceptable ranges
        // rawtextlen = rawtext.length;
        rawtextlen = rawtext.len() as i32;
        // for (i = 0; i < rawtextlen - 1; i++) {
        i = 0;
        while i < rawtextlen - 1 {
            //     if (rawtext[i] >= 0) {
            if rawtext[i as usize] >= 0 {
                //         // asciichars++;
                //     } else {
            } else {
                //         dbchars++;
                dbchars += 1;
                //         if ((byte) 0xA1 <= rawtext[i]
                //                 && rawtext[i] <= (byte) 0xF9
                //                 && (((byte) 0x40 <= rawtext[i + 1] && rawtext[i + 1] <= (byte) 0x7E) || ((byte) 0xA1 <= rawtext[i + 1] && rawtext[i + 1] <= (byte) 0xFE))) {
                if (0xA1u8 as i8) <= rawtext[i as usize]
                    && rawtext[i as usize] <= (0xF9u8 as i8)
                    && (((0x40u8 as i8) <= rawtext[i as usize + 1] && rawtext[i as usize + 1] <= (0x7Eu8 as i8)) || ((0xA1u8 as i8) <= rawtext[i as usize + 1] && rawtext[i as usize + 1] <= (0xFEu8 as i8)))
                {
                    //             bfchars++;
                    bfchars += 1;
                    //             totalfreq += 500;
                    totalfreq += 500i64;
                    //             row = rawtext[i] + 256 - 0xA1;
                    row = rawtext[i as usize] as i32 + 256 - 0xA1;
                    //             if (0x40 <= rawtext[i + 1] && rawtext[i + 1] <= 0x7E) {
                    if 0x40 <= rawtext[i as usize + 1] as i32 && rawtext[i as usize + 1] as i32 <= 0x7E {
                        //                 column = rawtext[i + 1] - 0x40;
                        column = rawtext[i as usize + 1] as i32 - 0x40;
                    //             } else {
                    } else {
                        //                 column = rawtext[i + 1] + 256 - 0x61;
                        column = rawtext[i as usize + 1] as i32 + 256 - 0x61;
                    }
                    //             if (Big5Freq[row][column] != 0) {
                    if self.Big5Freq[row as usize][column as usize] != 0 {
                        //                 bffreq += Big5Freq[row][column];
                        bffreq += self.Big5Freq[row as usize][column as usize] as i64;
                    //             } else if (3 <= row && row <= 37) {
                    } else if 3 <= row && row <= 37 {
                        //                 bffreq += 200;
                        bffreq += 200i64;
                    }
                }
                //         }
                //         i++;
                i += 1;
            }
            //     }
            i += 1;
        }
        // rangeval = 50 * ((float) bfchars / (float) dbchars);
        rangeval = 50.0 * (bfchars as f32 / dbchars as f32);
        // freqval = 50 * ((float) bffreq / (float) totalfreq);
        freqval = 50.0 * (bffreq as f32 / totalfreq as f32);
        // return (int) (rangeval + freqval);
        (rangeval + freqval) as i32
    }

    /*
     * Function: big5plus_probability Argument: pointer to unsigned char array
     * Returns : number from 0 to 100 representing probability text in array
     * uses Big5+ encoding
     */
    // int big5plus_probability(byte[] rawtext) {
    fn big5plus_probability(&self, rawtext: &[i8]) -> i32 {
        // int i, rawtextlen = 0;
        let mut i: i32 = 0;
        let mut rawtextlen: i32 = 0;
        // int dbchars = 1, bfchars = 1;
        let mut dbchars: i32 = 1;
        let mut bfchars: i32 = 1;
        // long bffreq = 0, totalfreq = 1;
        let mut bffreq: i64 = 0;
        let mut totalfreq: i64 = 1;
        // float rangeval = 0, freqval = 0;
        let mut rangeval: f32 = 0.0;
        let mut freqval: f32 = 0.0;
        // int row, column;
        let mut row: i32 = 0;
        let mut column: i32 = 0;
        // Stage 1: Check to see if characters fit into acceptable ranges
        // rawtextlen = rawtext.length;
        rawtextlen = rawtext.len() as i32;
        // for (i = 0; i < rawtextlen - 1; i++) {
        i = 0;
        while i < rawtextlen - 1 {
            // System.err.println(rawtext[i]);
            //     if (rawtext[i] >= 128) {
            if rawtext[i as usize] as i32 >= 128 {
                //         // asciichars++;
                //     } else {
            } else {
                //         dbchars++;
                dbchars += 1;
                //         if (0xA1 <= rawtext[i]
                //                 && rawtext[i] <= 0xF9
                //                 && // Original Big5 range
                //                 ((0x40 <= rawtext[i + 1] && rawtext[i + 1] <= 0x7E) || (0xA1 <= rawtext[i + 1] && rawtext[i + 1] <= 0xFE))) {
                if 0xA1 <= rawtext[i as usize] as i32
                    && rawtext[i as usize] as i32 <= 0xF9
                    && // Original Big5 range
                    ((0x40 <= rawtext[i as usize + 1] as i32 && rawtext[i as usize + 1] as i32 <= 0x7E) || (0xA1 <= rawtext[i as usize + 1] as i32 && rawtext[i as usize + 1] as i32 <= 0xFE))
                {
                    //             bfchars++;
                    bfchars += 1;
                    //             totalfreq += 500;
                    totalfreq += 500i64;
                    //             row = rawtext[i] - 0xA1;
                    row = rawtext[i as usize] as i32 - 0xA1;
                    //             if (0x40 <= rawtext[i + 1] && rawtext[i + 1] <= 0x7E) {
                    if 0x40 <= rawtext[i as usize + 1] as i32 && rawtext[i as usize + 1] as i32 <= 0x7E {
                        //                 column = rawtext[i + 1] - 0x40;
                        column = rawtext[i as usize + 1] as i32 - 0x40;
                    //             } else {
                    } else {
                        //                 column = rawtext[i + 1] - 0x61;
                        column = rawtext[i as usize + 1] as i32 - 0x61;
                    }
                    //             // System.out.println("original row " + row + " column " +
                    //             // column);
                    //             if (Big5Freq[row][column] != 0) {
                    if self.Big5Freq[row as usize][column as usize] != 0 {
                        //                 bffreq += Big5Freq[row][column];
                        bffreq += self.Big5Freq[row as usize][column as usize] as i64;
                    //             } else if (3 <= row && row < 37) {
                    } else if 3 <= row && row < 37 {
                        //                 bffreq += 200;
                        bffreq += 200i64;
                    }
                //         } else if (0x81 <= rawtext[i]
                //                 && rawtext[i] <= 0xFE
                //                 && // Extended Big5 range
                //                 ((0x40 <= rawtext[i + 1] && rawtext[i + 1] <= 0x7E) || (0x80 <= rawtext[i + 1] && rawtext[i + 1] <= 0xFE))) {
                } else if 0x81 <= rawtext[i as usize] as i32
                    && rawtext[i as usize] as i32 <= 0xFE
                    && // Extended Big5 range
                    ((0x40 <= rawtext[i as usize + 1] as i32 && rawtext[i as usize + 1] as i32 <= 0x7E) || (0x80 <= rawtext[i as usize + 1] as i32 && rawtext[i as usize + 1] as i32 <= 0xFE))
                {
                    //             bfchars++;
                    bfchars += 1;
                    //             totalfreq += 500;
                    totalfreq += 500i64;
                    //             row = rawtext[i] - 0x81;
                    row = rawtext[i as usize] as i32 - 0x81;
                    //             if (0x40 <= rawtext[i + 1] && rawtext[i + 1] <= 0x7E) {
                    if 0x40 <= rawtext[i as usize + 1] as i32 && rawtext[i as usize + 1] as i32 <= 0x7E {
                        //                 column = rawtext[i + 1] - 0x40;
                        column = rawtext[i as usize + 1] as i32 - 0x40;
                    //             } else {
                    } else {
                        //                 column = rawtext[i + 1] - 0x40;
                        column = rawtext[i as usize + 1] as i32 - 0x40;
                    }
                    //             // System.out.println("extended row " + row + " column " +
                    //             // column + " rawtext[i] " + rawtext[i]);
                    //             if (Big5PFreq[row][column] != 0) {
                    if self.Big5PFreq[row as usize][column as usize] != 0 {
                        //                 bffreq += Big5PFreq[row][column];
                        bffreq += self.Big5PFreq[row as usize][column as usize] as i64;
                    }
                }
                //         }
                //         i++;
                i += 1;
            }
            //     }
            i += 1;
        }
        // rangeval = 50 * ((float) bfchars / (float) dbchars);
        rangeval = 50.0 * (bfchars as f32 / dbchars as f32);
        // freqval = 50 * ((float) bffreq / (float) totalfreq);
        freqval = 50.0 * (bffreq as f32 / totalfreq as f32);
        // For regular Big5 files, this would give the same score, so I handicap
        // it slightly
        // return (int) (rangeval + freqval) - 1;
        (rangeval + freqval) as i32 - 1
    }
    /*
     * Function: euc_tw_probability Argument: byte array Returns : number from 0
     * to 100 representing probability text in array uses EUC-TW (CNS 11643)
     * encoding
     */
    // int euc_tw_probability(byte[] rawtext) {
    fn euc_tw_probability(&self, rawtext: &[i8]) -> i32 {
        // int i, rawtextlen = 0;
        let mut i: i32 = 0;
        let mut rawtextlen: i32 = 0;
        // int dbchars = 1, cnschars = 1;
        let mut dbchars: i32 = 1;
        let mut cnschars: i32 = 1;
        // long cnsfreq = 0, totalfreq = 1;
        let mut cnsfreq: i64 = 0;
        let mut totalfreq: i64 = 1;
        // float rangeval = 0, freqval = 0;
        let mut rangeval: f32 = 0.0;
        let mut freqval: f32 = 0.0;
        // int row, column;
        let mut row: i32 = 0;
        let mut column: i32 = 0;
        // Check to see if characters fit into acceptable ranges
        // and have expected frequency of use
        // rawtextlen = rawtext.length;
        rawtextlen = rawtext.len() as i32;
        // for (i = 0; i < rawtextlen - 1; i++) {
        i = 0;
        while i < rawtextlen - 1 {
            //     if (rawtext[i] >= 0) { // in ASCII range
            if rawtext[i as usize] >= 0 {
                //         // asciichars++;
                //     } else { // high bit set
            } else {
                //         dbchars++;
                dbchars += 1;
                //         if (i + 3 < rawtextlen && (byte) 0x8E == rawtext[i]
                //                 && (byte) 0xA1 <= rawtext[i + 1]
                //                 && rawtext[i + 1] <= (byte) 0xB0
                //                 && (byte) 0xA1 <= rawtext[i + 2]
                //                 && rawtext[i + 2] <= (byte) 0xFE
                //                 && (byte) 0xA1 <= rawtext[i + 3]
                //                 && rawtext[i + 3] <= (byte) 0xFE) { // Planes 1 - 16
                if i + 3 < rawtextlen && (0x8Eu8 as i8) == rawtext[i as usize]
                    && (0xA1u8 as i8) <= rawtext[i as usize + 1]
                    && rawtext[i as usize + 1] <= (0xB0u8 as i8)
                    && (0xA1u8 as i8) <= rawtext[i as usize + 2]
                    && rawtext[i as usize + 2] <= (0xFEu8 as i8)
                    && (0xA1u8 as i8) <= rawtext[i as usize + 3]
                    && rawtext[i as usize + 3] <= (0xFEu8 as i8)
                {
                    //             cnschars++;
                    cnschars += 1;
                    //             // System.out.println("plane 2 or above CNS char");
                    //             // These are all less frequent chars so just ignore freq
                    //             i += 3;
                    i += 3;
                //         } else if ((byte) 0xA1 <= rawtext[i]
                //                 && rawtext[i] <= (byte) 0xFE
                //                 && // Plane 1
                //                 (byte) 0xA1 <= rawtext[i + 1]
                //                 && rawtext[i + 1] <= (byte) 0xFE) {
                } else if (0xA1u8 as i8) <= rawtext[i as usize]
                    && rawtext[i as usize] <= (0xFEu8 as i8)
                    && // Plane 1
                    (0xA1u8 as i8) <= rawtext[i as usize + 1]
                    && rawtext[i as usize + 1] <= (0xFEu8 as i8)
                {
                    //             cnschars++;
                    cnschars += 1;
                    //             totalfreq += 500;
                    totalfreq += 500i64;
                    //             row = rawtext[i] + 256 - 0xA1;
                    row = rawtext[i as usize] as i32 + 256 - 0xA1;
                    //             column = rawtext[i + 1] + 256 - 0xA1;
                    column = rawtext[i as usize + 1] as i32 + 256 - 0xA1;
                    //             if (EUC_TWFreq[row][column] != 0) {
                    if self.EUC_TWFreq[row as usize][column as usize] != 0 {
                        //                 cnsfreq += EUC_TWFreq[row][column];
                        cnsfreq += self.EUC_TWFreq[row as usize][column as usize] as i64;
                    //             } else if (35 <= row && row <= 92) {
                    } else if 35 <= row && row <= 92 {
                        //                 cnsfreq += 150;
                        cnsfreq += 150i64;
                    }
                    //             i++;
                    i += 1;
                }
            }
            //     }
            i += 1;
        }
        // rangeval = 50 * ((float) cnschars / (float) dbchars);
        rangeval = 50.0 * (cnschars as f32 / dbchars as f32);
        // freqval = 50 * ((float) cnsfreq / (float) totalfreq);
        freqval = 50.0 * (cnsfreq as f32 / totalfreq as f32);
        // return (int) (rangeval + freqval);
        (rangeval + freqval) as i32
    }

    /*
     * Function: iso_2022_cn_probability Argument: byte array Returns : number
     * from 0 to 100 representing probability text in array uses ISO 2022-CN
     * encoding WORKS FOR BASIC CASES, BUT STILL NEEDS MORE WORK
     */
    // int iso_2022_cn_probability(byte[] rawtext) {
    fn iso_2022_cn_probability(&self, rawtext: &[i8]) -> i32 {
        // int i, rawtextlen = 0;
        let mut i: i32 = 0;
        let mut rawtextlen: i32 = 0;
        // int dbchars = 1, isochars = 1;
        let mut dbchars: i32 = 1;
        let mut isochars: i32 = 1;
        // long isofreq = 0, totalfreq = 1;
        let mut isofreq: i64 = 0;
        let mut totalfreq: i64 = 1;
        // float rangeval = 0, freqval = 0;
        let mut rangeval: f32 = 0.0;
        let mut freqval: f32 = 0.0;
        // int row, column;
        let mut row: i32 = 0;
        let mut column: i32 = 0;
        // Check to see if characters fit into acceptable ranges
        // and have expected frequency of use
        // rawtextlen = rawtext.length;
        rawtextlen = rawtext.len() as i32;
        // for (i = 0; i < rawtextlen - 1; i++) {
        i = 0;
        while i < rawtextlen - 1 {
            //     if (rawtext[i] == (byte) 0x1B && i + 3 < rawtextlen) { // Escape
            if rawtext[i as usize] == 0x1B && i + 3 < rawtextlen {
            //         // char ESC
            //         if (rawtext[i + 1] == (byte) 0x24 && rawtext[i + 2] == 0x29
            //                 && rawtext[i + 3] == (byte) 0x41) { // GB Escape $ ) A
                if rawtext[i as usize + 1] == 0x24 && rawtext[i as usize + 2] == 0x29
                    && rawtext[i as usize + 3] == 0x41
                {
                    //             i += 4;
                    i += 4;
                    //             while (rawtext[i] != (byte) 0x1B) {
                    while rawtext[i as usize] != 0x1B {
                        //                 dbchars++;
                        dbchars += 1;
                        //                 if ((0x21 <= rawtext[i] && rawtext[i] <= 0x77)
                        //                         && (0x21 <= rawtext[i + 1] && rawtext[i + 1] <= 0x77)) {
                        if (0x21 <= rawtext[i as usize] as i32 && rawtext[i as usize] as i32 <= 0x77)
                            && (0x21 <= rawtext[i as usize + 1] as i32 && rawtext[i as usize + 1] as i32 <= 0x77)
                        {
                            //                     isochars++;
                            isochars += 1;
                            //                     row = rawtext[i] - 0x21;
                            row = rawtext[i as usize] as i32 - 0x21;
                            //                     column = rawtext[i + 1] - 0x21;
                            column = rawtext[i as usize + 1] as i32 - 0x21;
                            //                     totalfreq += 500;
                            totalfreq += 500i64;
                            //                     if (GBFreq[row][column] != 0) {
                            if self.GBFreq[row as usize][column as usize] != 0 {
                                //                         isofreq += GBFreq[row][column];
                                isofreq += self.GBFreq[row as usize][column as usize] as i64;
                            //                     } else if (15 <= row && row < 55) {
                            } else if 15 <= row && row < 55 {
                                //                         isofreq += 200;
                                isofreq += 200i64;
                            }
                            //                     i++;
                            i += 1;
                        }
                        //                 i++;
                        i += 1;
                    }
                //         } else if (i + 3 < rawtextlen && rawtext[i + 1] == (byte) 0x24
                //                 && rawtext[i + 2] == (byte) 0x29
                //                 && rawtext[i + 3] == (byte) 0x47) {
                } else if i + 3 < rawtextlen && rawtext[i as usize + 1] == 0x24
                    && rawtext[i as usize + 2] == 0x29
                    && rawtext[i as usize + 3] == 0x47
                {
                    //             // CNS Escape $ ) G
                    //             i += 4;
                    i += 4;
                    //             while (rawtext[i] != (byte) 0x1B) {
                    while rawtext[i as usize] != 0x1B {
                        //                 dbchars++;
                        dbchars += 1;
                        //                 if ((byte) 0x21 <= rawtext[i]
                        //                         && rawtext[i] <= (byte) 0x7E
                        //                         && (byte) 0x21 <= rawtext[i + 1]
                        //                         && rawtext[i + 1] <= (byte) 0x7E) {
                        if 0x21 <= rawtext[i as usize] as i32
                            && rawtext[i as usize] as i32 <= 0x7E
                            && 0x21 <= rawtext[i as usize + 1] as i32
                            && rawtext[i as usize + 1] as i32 <= 0x7E
                        {
                            //                     isochars++;
                            isochars += 1;
                            //                     totalfreq += 500;
                            totalfreq += 500i64;
                            //                     row = rawtext[i] - 0x21;
                            row = rawtext[i as usize] as i32 - 0x21;
                            //                     column = rawtext[i + 1] - 0x21;
                            column = rawtext[i as usize + 1] as i32 - 0x21;
                            //                     if (EUC_TWFreq[row][column] != 0) {
                            if self.EUC_TWFreq[row as usize][column as usize] != 0 {
                                //                         isofreq += EUC_TWFreq[row][column];
                                isofreq += self.EUC_TWFreq[row as usize][column as usize] as i64;
                            //                     } else if (35 <= row && row <= 92) {
                            } else if 35 <= row && row <= 92 {
                                //                         isofreq += 150;
                                isofreq += 150i64;
                            }
                            //                     i++;
                            i += 1;
                        }
                        //                 i++;
                        i += 1;
                    }
                }
                //         if (rawtext[i] == (byte) 0x1B && i + 2 < rawtextlen
                //                 && rawtext[i + 1] == (byte) 0x28
                //                 && rawtext[i + 2] == (byte) 0x42) { // ASCII:
                if rawtext[i as usize] == 0x1B && i + 2 < rawtextlen
                    && rawtext[i as usize + 1] == 0x28
                    && rawtext[i as usize + 2] == 0x42
                {
                //             // ESC
                //             // ( B
                //             i += 2;
                    i += 2;
                }
            }
            //     }
            i += 1;
        }
        // rangeval = 50 * ((float) isochars / (float) dbchars);
        rangeval = 50.0 * (isochars as f32 / dbchars as f32);
        // freqval = 50 * ((float) isofreq / (float) totalfreq);
        freqval = 50.0 * (isofreq as f32 / totalfreq as f32);
        // System.out.println("isochars dbchars isofreq totalfreq " + isochars +
        // " " + dbchars + " " + isofreq + " " + totalfreq + "
        // " + rangeval + " " + freqval);
        // return (int) (rangeval + freqval);
        (rangeval + freqval) as i32
        // return 0;
    }

    /*
     * Function: utf8_probability Argument: byte array Returns : number from 0
     * to 100 representing probability text in array uses UTF-8 encoding of
     * Unicode
     */
    // int utf8_probability(byte[] rawtext) {
    fn utf8_probability(&self, rawtext: &[i8]) -> i32 {
        // int score = 0;
        let mut score: i32 = 0;
        // int i, rawtextlen = 0;
        let mut i: i32 = 0;
        let mut rawtextlen: i32 = 0;
        // int goodbytes = 0, asciibytes = 0;
        let mut goodbytes: i32 = 0;
        let mut asciibytes: i32 = 0;
        // Maybe also use UTF8 Byte Order Mark: EF BB BF
        // Check to see if characters fit into acceptable ranges
        // rawtextlen = rawtext.length;
        rawtextlen = rawtext.len() as i32;
        // for (i = 0; i < rawtextlen; i++) {
        i = 0;
        while i < rawtextlen {
            //     if ((rawtext[i] & (byte) 0x7F) == rawtext[i]) { // One byte
            if (rawtext[i as usize] & (0x7Fu8 as i8)) == rawtext[i as usize] {
                //         asciibytes++;
                asciibytes += 1;
                //         // Ignore ASCII, can throw off count
                //     } else if (-64 <= rawtext[i] && rawtext[i] <= -33
                //             && // Two bytes
                //             i + 1 < rawtextlen && -128 <= rawtext[i + 1]
                //             && rawtext[i + 1] <= -65) {
            } else if -64 <= rawtext[i as usize] as i32 && rawtext[i as usize] as i32 <= -33
                && // Two bytes
                i + 1 < rawtextlen && -128 <= rawtext[i as usize + 1] as i32
                && rawtext[i as usize + 1] as i32 <= -65
            {
                //         goodbytes += 2;
                goodbytes += 2;
                //         i++;
                i += 1;
                //     } else if (-32 <= rawtext[i]
                //             && rawtext[i] <= -17
                //             && // Three bytes
                //             i + 2 < rawtextlen && -128 <= rawtext[i + 1]
                //             && rawtext[i + 1] <= -65 && -128 <= rawtext[i + 2]
                //             && rawtext[i + 2] <= -65) {
            } else if -32 <= rawtext[i as usize] as i32
                && rawtext[i as usize] as i32 <= -17
                && // Three bytes
                i + 2 < rawtextlen && -128 <= rawtext[i as usize + 1] as i32
                && rawtext[i as usize + 1] as i32 <= -65 && -128 <= rawtext[i as usize + 2] as i32
                && rawtext[i as usize + 2] as i32 <= -65
            {
                //         goodbytes += 3;
                goodbytes += 3;
                //         i += 2;
                i += 2;
            }
            //     }
            i += 1;
        }
        // if (asciibytes == rawtextlen) {
        if asciibytes == rawtextlen {
            //     return 0;
            return 0;
        }
        // score = (int) (100 * ((float) goodbytes / (float) (rawtextlen - asciibytes)));
        score = (100.0 * (goodbytes as f32 / (rawtextlen - asciibytes) as f32)) as i32;
        // System.out.println("rawtextlen " + rawtextlen + " goodbytes " +
        // goodbytes + " asciibytes " + asciibytes + " score " +
        // score);
        // If not above 98, reduce to zero to prevent coincidental matches
        // Allows for some (few) bad formed sequences
        // if (score > 98) {
        if score > 98 {
            //     return score;
            return score;
        // } else if (score > 95 && goodbytes > 30) {
        } else if score > 95 && goodbytes > 30 {
            //     return score;
            return score;
        // } else {
        } else {
            //     return 0;
            return 0;
        }
    }

    /*
     * Function: utf16_probability Argument: byte array Returns : number from 0
     * to 100 representing probability text in array uses UTF-16 encoding of
     * Unicode, guess based on BOM // NOT VERY GENERAL, NEEDS MUCH MORE WORK
     */
    // int utf16_probability(byte[] rawtext) {
    fn utf16_probability(&self, rawtext: &[i8]) -> i32 {
        // int score = 0;
        // int i, rawtextlen = 0;
        // int goodbytes = 0, asciibytes = 0;
        // if (rawtext.length > 1
        //         && ((byte) 0xFE == rawtext[0] && (byte) 0xFF == rawtext[1]) || // Big-endian
        //         ((byte) 0xFF == rawtext[0] && (byte) 0xFE == rawtext[1])) { // Little-endian
        if rawtext.len() > 1
            && ((0xFEu8 as i8) == rawtext[0] && (0xFFu8 as i8) == rawtext[1]) || // Big-endian
            ((0xFFu8 as i8) == rawtext[0] && (0xFEu8 as i8) == rawtext[1])
        {
            // Little-endian
            //     return 100;
            return 100;
        }
        // return 0;
        return 0;
        /*
         * // Check to see if characters fit into acceptable ranges rawtextlen =
         * rawtext.length; for (i = 0; i < rawtextlen; i++) { if ((rawtext[i] &
         * (byte)0x7F) == rawtext[i]) { // One byte goodbytes += 1;
         * asciibytes++; } else if ((rawtext[i] & (byte)0xDF) == rawtext[i]) {
         * // Two bytes if (i+1 < rawtextlen && (rawtext[i+1] & (byte)0xBF) ==
         * rawtext[i+1]) { goodbytes += 2; i++; } } else if ((rawtext[i] &
         * (byte)0xEF) == rawtext[i]) { // Three bytes if (i+2 < rawtextlen &&
         * (rawtext[i+1] & (byte)0xBF) == rawtext[i+1] && (rawtext[i+2] &
         * (byte)0xBF) == rawtext[i+2]) { goodbytes += 3; i+=2; } } }
         *
         * score = (int)(100 * ((float)goodbytes/(float)rawtext.length)); // An
         * all ASCII file is also a good UTF8 file, but I'd rather it // get
         * identified as ASCII. Can delete following 3 lines otherwise if
         * (goodbytes == asciibytes) { score = 0; } // If not above 90, reduce
         * to zero to prevent coincidental matches if (score > 90) { return
         * score; } else { return 0; }
         */
    }

    /*
     * Function: ascii_probability Argument: byte array Returns : number from 0
     * to 100 representing probability text in array uses all ASCII Description:
     * Sees if array has any characters not in ASCII range, if so, score is
     * reduced
     */
    // int ascii_probability(byte[] rawtext) {
    fn ascii_probability(&self, rawtext: &[i8]) -> i32 {
        // int score = 75;
        let mut score: i32 = 75;
        // int i, rawtextlen;
        let mut i: i32 = 0;
        let mut rawtextlen: i32 = 0;
        // rawtextlen = rawtext.length;
        rawtextlen = rawtext.len() as i32;
        // for (i = 0; i < rawtextlen; i++) {
        for i in 0..rawtextlen {
            //     if (rawtext[i] < 0) {
            if rawtext[i as usize] < 0 {
                //         score = score - 5;
                score = score - 5;
            //     } else if (rawtext[i] == (byte) 0x1B) { // ESC (used by ISO 2022)
            } else if rawtext[i as usize] == 0x1B {
                // ESC (used by ISO 2022)
                //         score = score - 5;
                score = score - 5;
            }
            //     if (score <= 0) {
            if score <= 0 {
                //         return 0;
                return 0;
            }
        }
        // return score;
        score
    }
    /*
     * Function: euc_kr__probability Argument: pointer to byte array Returns :
     * number from 0 to 100 representing probability text in array uses EUC-KR
     * encoding
     */
    // int euc_kr_probability(byte[] rawtext) {
    fn euc_kr_probability(&self, rawtext: &[i8]) -> i32 {
        // int i, rawtextlen = 0;
        let mut i: i32 = 0;
        let mut rawtextlen: i32 = 0;
        // int dbchars = 1, krchars = 1;
        let mut dbchars: i32 = 1;
        let mut krchars: i32 = 1;
        // long krfreq = 0, totalfreq = 1;
        let mut krfreq: i64 = 0;
        let mut totalfreq: i64 = 1;
        // float rangeval = 0, freqval = 0;
        let mut rangeval: f32 = 0.0;
        let mut freqval: f32 = 0.0;
        // int row, column;
        let mut row: i32 = 0;
        let mut column: i32 = 0;
        // Stage 1: Check to see if characters fit into acceptable ranges
        // rawtextlen = rawtext.length;
        rawtextlen = rawtext.len() as i32;
        // for (i = 0; i < rawtextlen - 1; i++) {
        i = 0;
        while i < rawtextlen - 1 {
            // System.err.println(rawtext[i]);
            //     if (rawtext[i] >= 0) {
            if rawtext[i as usize] >= 0 {
                //         // asciichars++;
                //     } else {
            } else {
                //         dbchars++;
                dbchars += 1;
                //         if ((byte) 0xA1 <= rawtext[i] && rawtext[i] <= (byte) 0xFE
                //                 && (byte) 0xA1 <= rawtext[i + 1]
                //                 && rawtext[i + 1] <= (byte) 0xFE) {
                if (0xA1u8 as i8) <= rawtext[i as usize] && rawtext[i as usize] <= (0xFEu8 as i8)
                    && (0xA1u8 as i8) <= rawtext[i as usize + 1]
                    && rawtext[i as usize + 1] <= (0xFEu8 as i8)
                {
                    //             krchars++;
                    krchars += 1;
                    //             totalfreq += 500;
                    totalfreq += 500i64;
                    //             row = rawtext[i] + 256 - 0xA1;
                    row = rawtext[i as usize] as i32 + 256 - 0xA1;
                    //             column = rawtext[i + 1] + 256 - 0xA1;
                    column = rawtext[i as usize + 1] as i32 + 256 - 0xA1;
                    //             if (KRFreq[row][column] != 0) {
                    if self.KRFreq[row as usize][column as usize] != 0 {
                        //                 krfreq += KRFreq[row][column];
                        krfreq += self.KRFreq[row as usize][column as usize] as i64;
                    //             } else if (15 <= row && row < 55) {
                    } else if 15 <= row && row < 55 {
                        //                 krfreq += 0;
                        krfreq += 0i64;
                    }
                }
                //         }
                //         i++;
                i += 1;
            }
            //     }
            i += 1;
        }
        // rangeval = 50 * ((float) krchars / (float) dbchars);
        rangeval = 50.0 * (krchars as f32 / dbchars as f32);
        // freqval = 50 * ((float) krfreq / (float) totalfreq);
        freqval = 50.0 * (krfreq as f32 / totalfreq as f32);
        // return (int) (rangeval + freqval);
        (rangeval + freqval) as i32
    }

    /*
     * Function: cp949__probability Argument: pointer to byte array Returns :
     * number from 0 to 100 representing probability text in array uses Cp949
     * encoding
     */
    // int cp949_probability(byte[] rawtext) {
    fn cp949_probability(&self, rawtext: &[i8]) -> i32 {
        // int i, rawtextlen = 0;
        let mut i: i32 = 0;
        let mut rawtextlen: i32 = 0;
        // int dbchars = 1, krchars = 1;
        let mut dbchars: i32 = 1;
        let mut krchars: i32 = 1;
        // long krfreq = 0, totalfreq = 1;
        let mut krfreq: i64 = 0;
        let mut totalfreq: i64 = 1;
        // float rangeval = 0, freqval = 0;
        let mut rangeval: f32 = 0.0;
        let mut freqval: f32 = 0.0;
        // int row, column;
        let mut row: i32 = 0;
        let mut column: i32 = 0;
        // Stage 1: Check to see if characters fit into acceptable ranges
        // rawtextlen = rawtext.length;
        rawtextlen = rawtext.len() as i32;
        // for (i = 0; i < rawtextlen - 1; i++) {
        i = 0;
        while i < rawtextlen - 1 {
            // System.err.println(rawtext[i]);
            //     if (rawtext[i] >= 0) {
            if rawtext[i as usize] >= 0 {
                //         // asciichars++;
                //     } else {
            } else {
                //         dbchars++;
                dbchars += 1;
                //         if ((byte) 0x81 <= rawtext[i]
                //                 && rawtext[i] <= (byte) 0xFE
                //                 && ((byte) 0x41 <= rawtext[i + 1]
                //                 && rawtext[i + 1] <= (byte) 0x5A
                //                 || (byte) 0x61 <= rawtext[i + 1]
                //                 && rawtext[i + 1] <= (byte) 0x7A || (byte) 0x81 <= rawtext[i + 1]
                //                 && rawtext[i + 1] <= (byte) 0xFE)) {
                if (0x81u8 as i8) <= rawtext[i as usize]
                    && rawtext[i as usize] <= (0xFEu8 as i8)
                    && ((0x41u8 as i8) <= rawtext[i as usize + 1]
                    && rawtext[i as usize + 1] <= (0x5Au8 as i8)
                    || (0x61u8 as i8) <= rawtext[i as usize + 1]
                    && rawtext[i as usize + 1] <= (0x7Au8 as i8) || (0x81u8 as i8) <= rawtext[i as usize + 1]
                    && rawtext[i as usize + 1] <= (0xFEu8 as i8))
                {
                    //             krchars++;
                    krchars += 1;
                    //             totalfreq += 500;
                    totalfreq += 500i64;
                    //             if ((byte) 0xA1 <= rawtext[i] && rawtext[i] <= (byte) 0xFE
                    //                     && (byte) 0xA1 <= rawtext[i + 1]
                    //                     && rawtext[i + 1] <= (byte) 0xFE) {
                    if (0xA1u8 as i8) <= rawtext[i as usize] && rawtext[i as usize] <= (0xFEu8 as i8)
                        && (0xA1u8 as i8) <= rawtext[i as usize + 1]
                        && rawtext[i as usize + 1] <= (0xFEu8 as i8)
                    {
                        //                 row = rawtext[i] + 256 - 0xA1;
                        row = rawtext[i as usize] as i32 + 256 - 0xA1;
                        //                 column = rawtext[i + 1] + 256 - 0xA1;
                        column = rawtext[i as usize + 1] as i32 + 256 - 0xA1;
                        //                 if (KRFreq[row][column] != 0) {
                        if self.KRFreq[row as usize][column as usize] != 0 {
                            //                     krfreq += KRFreq[row][column];
                            krfreq += self.KRFreq[row as usize][column as usize] as i64;
                        }
                    }
                }
                //         }
                //         i++;
                i += 1;
            }
            //     }
            i += 1;
        }
        // rangeval = 50 * ((float) krchars / (float) dbchars);
        rangeval = 50.0 * (krchars as f32 / dbchars as f32);
        // freqval = 50 * ((float) krfreq / (float) totalfreq);
        freqval = 50.0 * (krfreq as f32 / totalfreq as f32);
        // return (int) (rangeval + freqval);
        (rangeval + freqval) as i32
    }

    // int iso_2022_kr_probability(byte[] rawtext) {
    fn iso_2022_kr_probability(&self, rawtext: &[i8]) -> i32 {
        // int i;
        let mut i: i32 = 0;
        // for (i = 0; i < rawtext.length; i++) {
        for i in 0..rawtext.len() {
            //     if (i + 3 < rawtext.length && rawtext[i] == 0x1b
            //             && (char) rawtext[i + 1] == '$'
            //             && (char) rawtext[i + 2] == ')'
            //             && (char) rawtext[i + 3] == 'C') {
            if i + 3 < rawtext.len() && rawtext[i] == 0x1b
                && rawtext[i + 1] == (b'$' as i8)
                && rawtext[i + 2] == (b')' as i8)
                && rawtext[i + 3] == (b'C' as i8)
            {
                //         return 100;
                return 100;
            }
        }
        // return 0;
        0
    }

    /*
     * Function: euc_jp_probability Argument: pointer to byte array Returns :
     * number from 0 to 100 representing probability text in array uses EUC-JP
     * encoding
     */
    // int euc_jp_probability(byte[] rawtext) {
    fn euc_jp_probability(&self, rawtext: &[i8]) -> i32 {
        // int i, rawtextlen = 0;
        let mut i: i32 = 0;
        let mut rawtextlen: i32 = 0;
        // int dbchars = 1, jpchars = 1;
        let mut dbchars: i32 = 1;
        let mut jpchars: i32 = 1;
        // long jpfreq = 0, totalfreq = 1;
        let mut jpfreq: i64 = 0;
        let mut totalfreq: i64 = 1;
        // float rangeval = 0, freqval = 0;
        let mut rangeval: f32 = 0.0;
        let mut freqval: f32 = 0.0;
        // int row, column;
        let mut row: i32 = 0;
        let mut column: i32 = 0;
        // Stage 1: Check to see if characters fit into acceptable ranges
        // rawtextlen = rawtext.length;
        rawtextlen = rawtext.len() as i32;
        // for (i = 0; i < rawtextlen - 1; i++) {
        i = 0;
        while i < rawtextlen - 1 {
            // System.err.println(rawtext[i]);
            //     if (rawtext[i] >= 0) {
            if rawtext[i as usize] >= 0 {
                //         // asciichars++;
                //     } else {
            } else {
                //         dbchars++;
                dbchars += 1;
                //         if ((byte) 0xA1 <= rawtext[i] && rawtext[i] <= (byte) 0xFE
                //                 && (byte) 0xA1 <= rawtext[i + 1]
                //                 && rawtext[i + 1] <= (byte) 0xFE) {
                if (0xA1u8 as i8) <= rawtext[i as usize] && rawtext[i as usize] <= (0xFEu8 as i8)
                    && (0xA1u8 as i8) <= rawtext[i as usize + 1]
                    && rawtext[i as usize + 1] <= (0xFEu8 as i8)
                {
                    //             jpchars++;
                    jpchars += 1;
                    //             totalfreq += 500;
                    totalfreq += 500i64;
                    //             row = rawtext[i] + 256 - 0xA1;
                    row = rawtext[i as usize] as i32 + 256 - 0xA1;
                    //             column = rawtext[i + 1] + 256 - 0xA1;
                    column = rawtext[i as usize + 1] as i32 + 256 - 0xA1;
                    //             if (JPFreq[row][column] != 0) {
                    if self.JPFreq[row as usize][column as usize] != 0 {
                        //                 jpfreq += JPFreq[row][column];
                        jpfreq += self.JPFreq[row as usize][column as usize] as i64;
                    //             } else if (15 <= row && row < 55) {
                    } else if 15 <= row && row < 55 {
                        //                 jpfreq += 0;
                        jpfreq += 0i64;
                    }
                }
                //         }
                //         i++;
                i += 1;
            }
            //     }
            i += 1;
        }
        // rangeval = 50 * ((float) jpchars / (float) dbchars);
        rangeval = 50.0 * (jpchars as f32 / dbchars as f32);
        // freqval = 50 * ((float) jpfreq / (float) totalfreq);
        freqval = 50.0 * (jpfreq as f32 / totalfreq as f32);
        // return (int) (rangeval + freqval);
        (rangeval + freqval) as i32
    }

    // int iso_2022_jp_probability(byte[] rawtext) {
    fn iso_2022_jp_probability(&self, rawtext: &[i8]) -> i32 {
        // int i;
        let mut i: i32 = 0;
        // for (i = 0; i < rawtext.length; i++) {
        for i in 0..rawtext.len() {
            //     if (i + 2 < rawtext.length && rawtext[i] == 0x1b
            //             && (char) rawtext[i + 1] == '$'
            //             && (char) rawtext[i + 2] == 'B') {
            if i + 2 < rawtext.len() && rawtext[i] == 0x1b
                && rawtext[i + 1] == (b'$' as i8)
                && rawtext[i + 2] == (b'B' as i8)
            {
                //         return 100;
                return 100;
            }
        }
        // return 0;
        0
    }

    /*
     * Function: sjis_probability Argument: pointer to byte array Returns :
     * number from 0 to 100 representing probability text in array uses
     * Shift-JIS encoding
     */
    // int sjis_probability(byte[] rawtext) {
    fn sjis_probability(&self, rawtext: &[i8]) -> i32 {
        // int i, rawtextlen = 0;
        let mut i: i32 = 0;
        let mut rawtextlen: i32 = 0;
        // int dbchars = 1, jpchars = 1;
        let mut dbchars: i32 = 1;
        let mut jpchars: i32 = 1;
        // long jpfreq = 0, totalfreq = 1;
        let mut jpfreq: i64 = 0;
        let mut totalfreq: i64 = 1;
        // float rangeval = 0, freqval = 0;
        let mut rangeval: f32 = 0.0;
        let mut freqval: f32 = 0.0;
        // int row, column, adjust;
        let mut row: i32 = 0;
        let mut column: i32 = 0;
        let mut adjust: i32 = 0;
        // Stage 1: Check to see if characters fit into acceptable ranges
        // rawtextlen = rawtext.length;
        rawtextlen = rawtext.len() as i32;
        // for (i = 0; i < rawtextlen - 1; i++) {
        i = 0;
        while i < rawtextlen - 1 {
            // System.err.println(rawtext[i]);
            //     if (rawtext[i] >= 0) {
            if rawtext[i as usize] >= 0 {
                //         // asciichars++;
                //     } else {
            } else {
                //         dbchars++;
                dbchars += 1;
                //         if (i + 1 < rawtext.length
                //                 && (((byte) 0x81 <= rawtext[i] && rawtext[i] <= (byte) 0x9F) || ((byte) 0xE0 <= rawtext[i] && rawtext[i] <= (byte) 0xEF))
                //                 && (((byte) 0x40 <= rawtext[i + 1] && rawtext[i + 1] <= (byte) 0x7E) || ((byte) 0x80 <= rawtext[i + 1] && rawtext[i + 1] <= (byte) 0xFC))) {
                if i + 1 < rawtext.len() as i32
                    && (((0x81u8 as i8) <= rawtext[i as usize] && rawtext[i as usize] <= (0x9Fu8 as i8)) || ((0xE0u8 as i8) <= rawtext[i as usize] && rawtext[i as usize] <= (0xEFu8 as i8)))
                    && (((0x40u8 as i8) <= rawtext[i as usize + 1] && rawtext[i as usize + 1] <= (0x7Eu8 as i8)) || ((0x80u8 as i8) <= rawtext[i as usize + 1] && rawtext[i as usize + 1] <= (0xFCu8 as i8)))
                {
                    //             jpchars++;
                    jpchars += 1;
                    //             totalfreq += 500;
                    totalfreq += 500i64;
                    //             row = rawtext[i] + 256;
                    row = rawtext[i as usize] as i32 + 256;
                    //             column = rawtext[i + 1] + 256;
                    column = rawtext[i as usize + 1] as i32 + 256;
                    //             if (column < 0x9f) {
                    if column < 0x9f {
                        //                 adjust = 1;
                        adjust = 1;
                        //                 if (column > 0x7f) {
                        if column > 0x7f {
                            //                     column -= 0x20;
                            column -= 0x20;
                        //                 } else {
                        } else {
                            //                     column -= 0x19;
                            column -= 0x19;
                        }
                    //             } else {
                    } else {
                        //                 adjust = 0;
                        adjust = 0;
                        //                 column -= 0x7e;
                        column -= 0x7e;
                    }
                    //             if (row < 0xa0) {
                    if row < 0xa0 {
                        //                 row = ((row - 0x70) << 1) - adjust;
                        row = ((row - 0x70) << 1) - adjust;
                    //             } else {
                    } else {
                        //                 row = ((row - 0xb0) << 1) - adjust;
                        row = ((row - 0xb0) << 1) - adjust;
                    }
                    //             row -= 0x20;
                    row -= 0x20;
                    //             column = 0x20;
                    column = 0x20;
                    //             // System.out.println("original row " + row + " column " +
                    //             // column);
                    //             if (row < JPFreq.length && column < JPFreq[row].length
                    //                     && JPFreq[row][column] != 0) {
                    if (row as usize) < self.JPFreq.len() && (column as usize) < self.JPFreq[row as usize].len()
                        && self.JPFreq[row as usize][column as usize] != 0
                    {
                        //                 jpfreq += JPFreq[row][column];
                        jpfreq += self.JPFreq[row as usize][column as usize] as i64;
                    }
                    //             i++;
                    i += 1;
                //         } else if ((byte) 0xA1 <= rawtext[i]
                //                 && rawtext[i] <= (byte) 0xDF) {
                } else if (0xA1u8 as i8) <= rawtext[i as usize]
                    && rawtext[i as usize] <= (0xDFu8 as i8)
                {
                    //             // half-width katakana, convert to full-width
                }
            }
            //     }
            i += 1;
        }
        // rangeval = 50 * ((float) jpchars / (float) dbchars);
        rangeval = 50.0 * (jpchars as f32 / dbchars as f32);
        // freqval = 50 * ((float) jpfreq / (float) totalfreq);
        freqval = 50.0 * (jpfreq as f32 / totalfreq as f32);
        // For regular GB files, this would give the same score, so I handicap
        // it slightly
        // return (int) (rangeval + freqval) - 1;
        (rangeval + freqval) as i32 - 1
    }

    // void initialize_frequencies() {
    fn initialize_frequencies(&mut self) {
        // int i, j;
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        // for (i = 93; i >= 0; i--) {
        i = 93;
        while i >= 0 {
            //     for (j = 93; j >= 0; j--) {
            j = 93;
            while j >= 0 {
                //         GBFreq[i][j] = 0;
                self.GBFreq[i as usize][j as usize] = 0;
                j -= 1;
            }
            i -= 1;
        }
        // for (i = 125; i >= 0; i--) {
        i = 125;
        while i >= 0 {
            //     for (j = 190; j >= 0; j--) {
            j = 190;
            while j >= 0 {
                //         GBKFreq[i][j] = 0;
                self.GBKFreq[i as usize][j as usize] = 0;
                j -= 1;
            }
            i -= 1;
        }
        // // for (i = 0; i < 94; i++) {
        // // for (j = 0; j < 158; j++) {
        // for (i = 93; i >= 0; i--) {
        i = 93;
        while i >= 0 {
            //     for (j = 157; j >= 0; j--) {
            j = 157;
            while j >= 0 {
                //         Big5Freq[i][j] = 0;
                self.Big5Freq[i as usize][j as usize] = 0;
                j -= 1;
            }
            i -= 1;
        }
        // // for (i = 0; i < 126; i++) {
        // // for (j = 0; j < 191; j++) {
        // for (i = 125; i >= 0; i--) {
        i = 125;
        while i >= 0 {
            //     for (j = 190; j >= 0; j--) {
            j = 190;
            while j >= 0 {
                //         Big5PFreq[i][j] = 0;
                self.Big5PFreq[i as usize][j as usize] = 0;
                j -= 1;
            }
            i -= 1;
        }
        // // for (i = 0; i < 94; i++) {
        // // for (j = 0; j < 94; j++) {
        // for (i = 93; i >= 0; i--) {
        i = 93;
        while i >= 0 {
            //     for (j = 93; j >= 0; j--) {
            j = 93;
            while j >= 0 {
                //         EUC_TWFreq[i][j] = 0;
                self.EUC_TWFreq[i as usize][j as usize] = 0;
                j -= 1;
            }
            i -= 1;
        }
        // for (i = 93; i >= 0; i--) {
        i = 93;
        while i >= 0 {
            //     for (j = 93; j >= 0; j--) {
            j = 93;
            while j >= 0 {
                //         JPFreq[i][j] = 0;
                self.JPFreq[i as usize][j as usize] = 0;
                j -= 1;
            }
            i -= 1;
        }
        self.GBFreq[20][35] = 599;
        self.GBFreq[49][26] = 598;
        self.GBFreq[41][38] = 597;
        self.GBFreq[17][26] = 596;
        self.GBFreq[32][42] = 595;
        self.GBFreq[39][42] = 594;
        self.GBFreq[45][49] = 593;
        self.GBFreq[51][57] = 592;
        self.GBFreq[50][47] = 591;
        self.GBFreq[42][90] = 590;
        self.GBFreq[52][65] = 589;
        self.GBFreq[53][47] = 588;
        self.GBFreq[19][82] = 587;
        self.GBFreq[31][19] = 586;
        self.GBFreq[40][46] = 585;
        self.GBFreq[24][89] = 584;
        self.GBFreq[23][85] = 583;
        self.GBFreq[20][28] = 582;
        self.GBFreq[42][20] = 581;
        self.GBFreq[34][38] = 580;
        self.GBFreq[45][9] = 579;
        self.GBFreq[54][50] = 578;
        self.GBFreq[25][44] = 577;
        self.GBFreq[35][66] = 576;
        self.GBFreq[20][55] = 575;
        self.GBFreq[18][85] = 574;
        self.GBFreq[20][31] = 573;
        self.GBFreq[49][17] = 572;
        self.GBFreq[41][16] = 571;
        self.GBFreq[35][73] = 570;
        self.GBFreq[20][34] = 569;
        self.GBFreq[29][44] = 568;
        self.GBFreq[35][38] = 567;
        self.GBFreq[49][9] = 566;
        self.GBFreq[46][33] = 565;
        self.GBFreq[49][51] = 564;
        self.GBFreq[40][89] = 563;
        self.GBFreq[26][64] = 562;
        self.GBFreq[54][51] = 561;
        self.GBFreq[54][36] = 560;
        self.GBFreq[39][4] = 559;
        self.GBFreq[53][13] = 558;
        self.GBFreq[24][92] = 557;
        self.GBFreq[27][49] = 556;
        self.GBFreq[48][6] = 555;
        self.GBFreq[21][51] = 554;
        self.GBFreq[30][40] = 553;
        self.GBFreq[42][92] = 552;
        self.GBFreq[31][78] = 551;
        self.GBFreq[25][82] = 550;
        self.GBFreq[47][0] = 549;
        self.GBFreq[34][19] = 548;
        self.GBFreq[47][35] = 547;
        self.GBFreq[21][63] = 546;
        self.GBFreq[43][75] = 545;
        self.GBFreq[21][87] = 544;
        self.GBFreq[35][59] = 543;
        self.GBFreq[25][34] = 542;
        self.GBFreq[21][27] = 541;
        self.GBFreq[39][26] = 540;
        self.GBFreq[34][26] = 539;
        self.GBFreq[39][52] = 538;
        self.GBFreq[50][57] = 537;
        self.GBFreq[37][79] = 536;
        self.GBFreq[26][24] = 535;
        self.GBFreq[22][1] = 534;
        self.GBFreq[18][40] = 533;
        self.GBFreq[41][33] = 532;
        self.GBFreq[53][26] = 531;
        self.GBFreq[54][86] = 530;
        self.GBFreq[20][16] = 529;
        self.GBFreq[46][74] = 528;
        self.GBFreq[30][19] = 527;
        self.GBFreq[45][35] = 526;
        self.GBFreq[45][61] = 525;
        self.GBFreq[30][9] = 524;
        self.GBFreq[41][53] = 523;
        self.GBFreq[41][13] = 522;
        self.GBFreq[50][34] = 521;
        self.GBFreq[53][86] = 520;
        self.GBFreq[47][47] = 519;
        self.GBFreq[22][28] = 518;
        self.GBFreq[50][53] = 517;
        self.GBFreq[39][70] = 516;
        self.GBFreq[38][15] = 515;
        self.GBFreq[42][88] = 514;
        self.GBFreq[16][29] = 513;
        self.GBFreq[27][90] = 512;
        self.GBFreq[29][12] = 511;
        self.GBFreq[44][22] = 510;
        self.GBFreq[34][69] = 509;
        self.GBFreq[24][10] = 508;
        self.GBFreq[44][11] = 507;
        self.GBFreq[39][92] = 506;
        self.GBFreq[49][48] = 505;
        self.GBFreq[31][46] = 504;
        self.GBFreq[19][50] = 503;
        self.GBFreq[21][14] = 502;
        self.GBFreq[32][28] = 501;
        self.GBFreq[18][3] = 500;
        self.GBFreq[53][9] = 499;
        self.GBFreq[34][80] = 498;
        self.GBFreq[48][88] = 497;
        self.GBFreq[46][53] = 496;
        self.GBFreq[22][53] = 495;
        self.GBFreq[28][10] = 494;
        self.GBFreq[44][65] = 493;
        self.GBFreq[20][10] = 492;
        self.GBFreq[40][76] = 491;
        self.GBFreq[47][8] = 490;
        self.GBFreq[50][74] = 489;
        self.GBFreq[23][62] = 488;
        self.GBFreq[49][65] = 487;
        self.GBFreq[28][87] = 486;
        self.GBFreq[15][48] = 485;
        self.GBFreq[22][7] = 484;
        self.GBFreq[19][42] = 483;
        self.GBFreq[41][20] = 482;
        self.GBFreq[26][55] = 481;
        self.GBFreq[21][93] = 480;
        self.GBFreq[31][76] = 479;
        self.GBFreq[34][31] = 478;
        self.GBFreq[20][66] = 477;
        self.GBFreq[51][33] = 476;
        self.GBFreq[34][86] = 475;
        self.GBFreq[37][67] = 474;
        self.GBFreq[53][53] = 473;
        self.GBFreq[40][88] = 472;
        self.GBFreq[39][10] = 471;
        self.GBFreq[24][3] = 470;
        self.GBFreq[27][25] = 469;
        self.GBFreq[26][15] = 468;
        self.GBFreq[21][88] = 467;
        self.GBFreq[52][62] = 466;
        self.GBFreq[46][81] = 465;
        self.GBFreq[38][72] = 464;
        self.GBFreq[17][30] = 463;
        self.GBFreq[52][92] = 462;
        self.GBFreq[34][90] = 461;
        self.GBFreq[21][7] = 460;
        self.GBFreq[36][13] = 459;
        self.GBFreq[45][41] = 458;
        self.GBFreq[32][5] = 457;
        self.GBFreq[26][89] = 456;
        self.GBFreq[23][87] = 455;
        self.GBFreq[20][39] = 454;
        self.GBFreq[27][23] = 453;
        self.GBFreq[25][59] = 452;
        self.GBFreq[49][20] = 451;
        self.GBFreq[54][77] = 450;
        self.GBFreq[27][67] = 449;
        self.GBFreq[47][33] = 448;
        self.GBFreq[41][17] = 447;
        self.GBFreq[19][81] = 446;
        self.GBFreq[16][66] = 445;
        self.GBFreq[45][26] = 444;
        self.GBFreq[49][81] = 443;
        self.GBFreq[53][55] = 442;
        self.GBFreq[16][26] = 441;
        self.GBFreq[54][62] = 440;
        self.GBFreq[20][70] = 439;
        self.GBFreq[42][35] = 438;
        self.GBFreq[20][57] = 437;
        self.GBFreq[34][36] = 436;
        self.GBFreq[46][63] = 435;
        self.GBFreq[19][45] = 434;
        self.GBFreq[21][10] = 433;
        self.GBFreq[52][93] = 432;
        self.GBFreq[25][2] = 431;
        self.GBFreq[30][57] = 430;
        self.GBFreq[41][24] = 429;
        self.GBFreq[28][43] = 428;
        self.GBFreq[45][86] = 427;
        self.GBFreq[51][56] = 426;
        self.GBFreq[37][28] = 425;
        self.GBFreq[52][69] = 424;
        self.GBFreq[43][92] = 423;
        self.GBFreq[41][31] = 422;
        self.GBFreq[37][87] = 421;
        self.GBFreq[47][36] = 420;
        self.GBFreq[16][16] = 419;
        self.GBFreq[40][56] = 418;
        self.GBFreq[24][55] = 417;
        self.GBFreq[17][1] = 416;
        self.GBFreq[35][57] = 415;
        self.GBFreq[27][50] = 414;
        self.GBFreq[26][14] = 413;
        self.GBFreq[50][40] = 412;
        self.GBFreq[39][19] = 411;
        self.GBFreq[19][89] = 410;
        self.GBFreq[29][91] = 409;
        self.GBFreq[17][89] = 408;
        self.GBFreq[39][74] = 407;
        self.GBFreq[46][39] = 406;
        self.GBFreq[40][28] = 405;
        self.GBFreq[45][68] = 404;
        self.GBFreq[43][10] = 403;
        self.GBFreq[42][13] = 402;
        self.GBFreq[44][81] = 401;
        self.GBFreq[41][47] = 400;
        self.GBFreq[48][58] = 399;
        self.GBFreq[43][68] = 398;
        self.GBFreq[16][79] = 397;
        self.GBFreq[19][5] = 396;
        self.GBFreq[54][59] = 395;
        self.GBFreq[17][36] = 394;
        self.GBFreq[18][0] = 393;
        self.GBFreq[41][5] = 392;
        self.GBFreq[41][72] = 391;
        self.GBFreq[16][39] = 390;
        self.GBFreq[54][0] = 389;
        self.GBFreq[51][16] = 388;
        self.GBFreq[29][36] = 387;
        self.GBFreq[47][5] = 386;
        self.GBFreq[47][51] = 385;
        self.GBFreq[44][7] = 384;
        self.GBFreq[35][30] = 383;
        self.GBFreq[26][9] = 382;
        self.GBFreq[16][7] = 381;
        self.GBFreq[32][1] = 380;
        self.GBFreq[33][76] = 379;
        self.GBFreq[34][91] = 378;
        self.GBFreq[52][36] = 377;
        self.GBFreq[26][77] = 376;
        self.GBFreq[35][48] = 375;
        self.GBFreq[40][80] = 374;
        self.GBFreq[41][92] = 373;
        self.GBFreq[27][93] = 372;
        self.GBFreq[15][17] = 371;
        self.GBFreq[16][76] = 370;
        self.GBFreq[51][12] = 369;
        self.GBFreq[18][20] = 368;
        self.GBFreq[15][54] = 367;
        self.GBFreq[50][5] = 366;
        self.GBFreq[33][22] = 365;
        self.GBFreq[37][57] = 364;
        self.GBFreq[28][47] = 363;
        self.GBFreq[42][31] = 362;
        self.GBFreq[18][2] = 361;
        self.GBFreq[43][64] = 360;
        self.GBFreq[23][47] = 359;
        self.GBFreq[28][79] = 358;
        self.GBFreq[25][45] = 357;
        self.GBFreq[23][91] = 356;
        self.GBFreq[22][19] = 355;
        self.GBFreq[25][46] = 354;
        self.GBFreq[22][36] = 353;
        self.GBFreq[54][85] = 352;
        self.GBFreq[46][20] = 351;
        self.GBFreq[27][37] = 350;
        self.GBFreq[26][81] = 349;
        self.GBFreq[42][29] = 348;
        self.GBFreq[31][90] = 347;
        self.GBFreq[41][59] = 346;
        self.GBFreq[24][65] = 345;
        self.GBFreq[44][84] = 344;
        self.GBFreq[24][90] = 343;
        self.GBFreq[38][54] = 342;
        self.GBFreq[28][70] = 341;
        self.GBFreq[27][15] = 340;
        self.GBFreq[28][80] = 339;
        self.GBFreq[29][8] = 338;
        self.GBFreq[45][80] = 337;
        self.GBFreq[53][37] = 336;
        self.GBFreq[28][65] = 335;
        self.GBFreq[23][86] = 334;
        self.GBFreq[39][45] = 333;
        self.GBFreq[53][32] = 332;
        self.GBFreq[38][68] = 331;
        self.GBFreq[45][78] = 330;
        self.GBFreq[43][7] = 329;
        self.GBFreq[46][82] = 328;
        self.GBFreq[27][38] = 327;
        self.GBFreq[16][62] = 326;
        self.GBFreq[24][17] = 325;
        self.GBFreq[22][70] = 324;
        self.GBFreq[52][28] = 323;
        self.GBFreq[23][40] = 322;
        self.GBFreq[28][50] = 321;
        self.GBFreq[42][91] = 320;
        self.GBFreq[47][76] = 319;
        self.GBFreq[15][42] = 318;
        self.GBFreq[43][55] = 317;
        self.GBFreq[29][84] = 316;
        self.GBFreq[44][90] = 315;
        self.GBFreq[53][16] = 314;
        self.GBFreq[22][93] = 313;
        self.GBFreq[34][10] = 312;
        self.GBFreq[32][53] = 311;
        self.GBFreq[43][65] = 310;
        self.GBFreq[28][7] = 309;
        self.GBFreq[35][46] = 308;
        self.GBFreq[21][39] = 307;
        self.GBFreq[44][18] = 306;
        self.GBFreq[40][10] = 305;
        self.GBFreq[54][53] = 304;
        self.GBFreq[38][74] = 303;
        self.GBFreq[28][26] = 302;
        self.GBFreq[15][13] = 301;
        self.GBFreq[39][34] = 300;
        self.GBFreq[39][46] = 299;
        self.GBFreq[42][66] = 298;
        self.GBFreq[33][58] = 297;
        self.GBFreq[15][56] = 296;
        self.GBFreq[18][51] = 295;
        self.GBFreq[49][68] = 294;
        self.GBFreq[30][37] = 293;
        self.GBFreq[51][84] = 292;
        self.GBFreq[51][9] = 291;
        self.GBFreq[40][70] = 290;
        self.GBFreq[41][84] = 289;
        self.GBFreq[28][64] = 288;
        self.GBFreq[32][88] = 287;
        self.GBFreq[24][5] = 286;
        self.GBFreq[53][23] = 285;
        self.GBFreq[42][27] = 284;
        self.GBFreq[22][38] = 283;
        self.GBFreq[32][86] = 282;
        self.GBFreq[34][30] = 281;
        self.GBFreq[38][63] = 280;
        self.GBFreq[24][59] = 279;
        self.GBFreq[22][81] = 278;
        self.GBFreq[32][11] = 277;
        self.GBFreq[51][21] = 276;
        self.GBFreq[54][41] = 275;
        self.GBFreq[21][50] = 274;
        self.GBFreq[23][89] = 273;
        self.GBFreq[19][87] = 272;
        self.GBFreq[26][7] = 271;
        self.GBFreq[30][75] = 270;
        self.GBFreq[43][84] = 269;
        self.GBFreq[51][25] = 268;
        self.GBFreq[16][67] = 267;
        self.GBFreq[32][9] = 266;
        self.GBFreq[48][51] = 265;
        self.GBFreq[39][7] = 264;
        self.GBFreq[44][88] = 263;
        self.GBFreq[52][24] = 262;
        self.GBFreq[23][34] = 261;
        self.GBFreq[32][75] = 260;
        self.GBFreq[19][10] = 259;
        self.GBFreq[28][91] = 258;
        self.GBFreq[32][83] = 257;
        self.GBFreq[25][75] = 256;
        self.GBFreq[53][45] = 255;
        self.GBFreq[29][85] = 254;
        self.GBFreq[53][59] = 253;
        self.GBFreq[16][2] = 252;
        self.GBFreq[19][78] = 251;
        self.GBFreq[15][75] = 250;
        self.GBFreq[51][42] = 249;
        self.GBFreq[45][67] = 248;
        self.GBFreq[15][74] = 247;
        self.GBFreq[25][81] = 246;
        self.GBFreq[37][62] = 245;
        self.GBFreq[16][55] = 244;
        self.GBFreq[18][38] = 243;
        self.GBFreq[23][23] = 242;
        self.GBFreq[38][30] = 241;
        self.GBFreq[17][28] = 240;
        self.GBFreq[44][73] = 239;
        self.GBFreq[23][78] = 238;
        self.GBFreq[40][77] = 237;
        self.GBFreq[38][87] = 236;
        self.GBFreq[27][19] = 235;
        self.GBFreq[38][82] = 234;
        self.GBFreq[37][22] = 233;
        self.GBFreq[41][30] = 232;
        self.GBFreq[54][9] = 231;
        self.GBFreq[32][30] = 230;
        self.GBFreq[30][52] = 229;
        self.GBFreq[40][84] = 228;
        self.GBFreq[53][57] = 227;
        self.GBFreq[27][27] = 226;
        self.GBFreq[38][64] = 225;
        self.GBFreq[18][43] = 224;
        self.GBFreq[23][69] = 223;
        self.GBFreq[28][12] = 222;
        self.GBFreq[50][78] = 221;
        self.GBFreq[50][1] = 220;
        self.GBFreq[26][88] = 219;
        self.GBFreq[36][40] = 218;
        self.GBFreq[33][89] = 217;
        self.GBFreq[41][28] = 216;
        self.GBFreq[31][77] = 215;
        self.GBFreq[46][1] = 214;
        self.GBFreq[47][19] = 213;
        self.GBFreq[35][55] = 212;
        self.GBFreq[41][21] = 211;
        self.GBFreq[27][10] = 210;
        self.GBFreq[32][77] = 209;
        self.GBFreq[26][37] = 208;
        self.GBFreq[20][33] = 207;
        self.GBFreq[41][52] = 206;
        self.GBFreq[32][18] = 205;
        self.GBFreq[38][13] = 204;
        self.GBFreq[20][18] = 203;
        self.GBFreq[20][24] = 202;
        self.GBFreq[45][19] = 201;
        self.GBFreq[18][53] = 200;
        /*
         * GBFreq[39][0] = 199; GBFreq[40][71] = 198; GBFreq[41][27] = 197;
         * GBFreq[15][69] = 196; GBFreq[42][10] = 195; GBFreq[31][89] = 194;
         * GBFreq[51][28] = 193; GBFreq[41][22] = 192; GBFreq[40][43] = 191;
         * GBFreq[38][6] = 190; GBFreq[37][11] = 189; GBFreq[39][60] = 188;
         * GBFreq[48][47] = 187; GBFreq[46][80] = 186; GBFreq[52][49] = 185;
         * GBFreq[50][48] = 184; GBFreq[25][1] = 183; GBFreq[52][29] = 182;
         * GBFreq[24][66] = 181; GBFreq[23][35] = 180; GBFreq[49][72] = 179;
         * GBFreq[47][45] = 178; GBFreq[45][14] = 177; GBFreq[51][70] = 176;
         * GBFreq[22][30] = 175; GBFreq[49][83] = 174; GBFreq[26][79] = 173;
         * GBFreq[27][41] = 172; GBFreq[51][81] = 171; GBFreq[41][54] = 170;
         * GBFreq[20][4] = 169; GBFreq[29][60] = 168; GBFreq[20][27] = 167;
         * GBFreq[50][15] = 166; GBFreq[41][6] = 165; GBFreq[35][34] = 164;
         * GBFreq[44][87] = 163; GBFreq[46][66] = 162; GBFreq[42][37] = 161;
         * GBFreq[42][24] = 160; GBFreq[54][7] = 159; GBFreq[41][14] = 158;
         * GBFreq[39][83] = 157; GBFreq[16][87] = 156; GBFreq[20][59] = 155;
         * GBFreq[42][12] = 154; GBFreq[47][2] = 153; GBFreq[21][32] = 152;
         * GBFreq[53][29] = 151; GBFreq[22][40] = 150; GBFreq[24][58] = 149;
         * GBFreq[52][88] = 148; GBFreq[29][30] = 147; GBFreq[15][91] = 146;
         * GBFreq[54][72] = 145; GBFreq[51][75] = 144; GBFreq[33][67] = 143;
         * GBFreq[41][50] = 142; GBFreq[27][34] = 141; GBFreq[46][17] = 140;
         * GBFreq[31][74] = 139; GBFreq[42][67] = 138; GBFreq[54][87] = 137;
         * GBFreq[27][14] = 136; GBFreq[16][63] = 135; GBFreq[16][5] = 134;
         * GBFreq[43][23] = 133; GBFreq[23][13] = 132; GBFreq[31][12] = 131;
         * GBFreq[25][57] = 130; GBFreq[38][49] = 129; GBFreq[42][69] = 128;
         * GBFreq[23][80] = 127; GBFreq[29][0] = 126; GBFreq[28][2] = 125;
         * GBFreq[28][17] = 124; GBFreq[17][27] = 123; GBFreq[40][16] = 122;
         * GBFreq[45][1] = 121; GBFreq[36][33] = 120; GBFreq[35][23] = 119;
         * GBFreq[20][86] = 118; GBFreq[29][53] = 117; GBFreq[23][88] = 116;
         * GBFreq[51][87] = 115; GBFreq[54][27] = 114; GBFreq[44][36] = 113;
         * GBFreq[21][45] = 112; GBFreq[53][52] = 111; GBFreq[31][53] = 110;
         * GBFreq[38][47] = 109; GBFreq[27][21] = 108; GBFreq[30][42] = 107;
         * GBFreq[29][10] = 106; GBFreq[35][35] = 105; GBFreq[24][56] = 104;
         * GBFreq[41][29] = 103; GBFreq[18][68] = 102; GBFreq[29][24] = 101;
         * GBFreq[25][84] = 100; GBFreq[35][47] = 99; GBFreq[29][56] = 98;
         * GBFreq[30][44] = 97; GBFreq[53][3] = 96; GBFreq[30][63] = 95;
         * GBFreq[52][52] = 94; GBFreq[54][1] = 93; GBFreq[22][48] = 92;
         * GBFreq[54][66] = 91; GBFreq[21][90] = 90; GBFreq[52][47] = 89;
         * GBFreq[39][25] = 88; GBFreq[39][39] = 87; GBFreq[44][37] = 86;
         * GBFreq[44][76] = 85; GBFreq[46][75] = 84; GBFreq[18][37] = 83;
         * GBFreq[47][42] = 82; GBFreq[19][92] = 81; GBFreq[51][27] = 80;
         * GBFreq[48][83] = 79; GBFreq[23][70] = 78; GBFreq[29][9] = 77;
         * GBFreq[33][79] = 76; GBFreq[52][90] = 75; GBFreq[53][6] = 74;
         * GBFreq[24][36] = 73; GBFreq[25][25] = 72; GBFreq[44][26] = 71;
         * GBFreq[25][36] = 70; GBFreq[29][87] = 69; GBFreq[48][0] = 68;
         * GBFreq[15][40] = 67; GBFreq[17][45] = 66; GBFreq[30][14] = 65;
         * GBFreq[48][38] = 64; GBFreq[23][19] = 63; GBFreq[40][42] = 62;
         * GBFreq[31][63] = 61; GBFreq[16][23] = 60; GBFreq[26][21] = 59;
         * GBFreq[32][76] = 58; GBFreq[23][58] = 57; GBFreq[41][37] = 56;
         * GBFreq[30][43] = 55; GBFreq[47][38] = 54; GBFreq[21][46] = 53;
         * GBFreq[18][33] = 52; GBFreq[52][37] = 51; GBFreq[36][8] = 50;
         * GBFreq[49][24] = 49; GBFreq[15][66] = 48; GBFreq[35][77] = 47;
         * GBFreq[27][58] = 46; GBFreq[35][51] = 45; GBFreq[24][69] = 44;
         * GBFreq[20][54] = 43; GBFreq[24][41] = 42; GBFreq[41][0] = 41;
         * GBFreq[33][71] = 40; GBFreq[23][52] = 39; GBFreq[29][67] = 38;
         * GBFreq[46][51] = 37; GBFreq[46][90] = 36; GBFreq[49][33] = 35;
         * GBFreq[33][28] = 34; GBFreq[37][86] = 33; GBFreq[39][22] = 32;
         * GBFreq[37][37] = 31; GBFreq[29][62] = 30; GBFreq[29][50] = 29;
         * GBFreq[36][89] = 28; GBFreq[42][44] = 27; GBFreq[51][82] = 26;
         * GBFreq[28][83] = 25; GBFreq[15][78] = 24; GBFreq[46][62] = 23;
         * GBFreq[19][69] = 22; GBFreq[51][23] = 21; GBFreq[37][69] = 20;
         * GBFreq[25][5] = 19; GBFreq[51][85] = 18; GBFreq[48][77] = 17;
         * GBFreq[32][46] = 16; GBFreq[53][60] = 15; GBFreq[28][57] = 14;
         * GBFreq[54][82] = 13; GBFreq[54][15] = 12; GBFreq[49][54] = 11;
         * GBFreq[53][87] = 10; GBFreq[27][16] = 9; GBFreq[29][34] = 8;
         * GBFreq[20][44] = 7; GBFreq[42][73] = 6; GBFreq[47][71] = 5;
         * GBFreq[29][37] = 4; GBFreq[25][50] = 3; GBFreq[18][84] = 2;
         * GBFreq[50][45] = 1; GBFreq[48][46] = 0;
         */
        // GBFreq[43][89] = -1; GBFreq[54][68] = -2;
        self.Big5Freq[9][89] = 600;
        self.Big5Freq[11][15] = 599;
        self.Big5Freq[3][66] = 598;
        self.Big5Freq[6][121] = 597;
        self.Big5Freq[3][0] = 596;
        self.Big5Freq[5][82] = 595;
        self.Big5Freq[3][42] = 594;
        self.Big5Freq[5][34] = 593;
        self.Big5Freq[3][8] = 592;
        self.Big5Freq[3][6] = 591;
        self.Big5Freq[3][67] = 590;
        self.Big5Freq[7][139] = 589;
        self.Big5Freq[23][137] = 588;
        self.Big5Freq[12][46] = 587;
        self.Big5Freq[4][8] = 586;
        self.Big5Freq[4][41] = 585;
        self.Big5Freq[18][47] = 584;
        self.Big5Freq[12][114] = 583;
        self.Big5Freq[6][1] = 582;
        self.Big5Freq[22][60] = 581;
        self.Big5Freq[5][46] = 580;
        self.Big5Freq[11][79] = 579;
        self.Big5Freq[3][23] = 578;
        self.Big5Freq[7][114] = 577;
        self.Big5Freq[29][102] = 576;
        self.Big5Freq[19][14] = 575;
        self.Big5Freq[4][133] = 574;
        self.Big5Freq[3][29] = 573;
        self.Big5Freq[4][109] = 572;
        self.Big5Freq[14][127] = 571;
        self.Big5Freq[5][48] = 570;
        self.Big5Freq[13][104] = 569;
        self.Big5Freq[3][132] = 568;
        self.Big5Freq[26][64] = 567;
        self.Big5Freq[7][19] = 566;
        self.Big5Freq[4][12] = 565;
        self.Big5Freq[11][124] = 564;
        self.Big5Freq[7][89] = 563;
        self.Big5Freq[15][124] = 562;
        self.Big5Freq[4][108] = 561;
        self.Big5Freq[19][66] = 560;
        self.Big5Freq[3][21] = 559;
        self.Big5Freq[24][12] = 558;
        self.Big5Freq[28][111] = 557;
        self.Big5Freq[12][107] = 556;
        self.Big5Freq[3][112] = 555;
        self.Big5Freq[8][113] = 554;
        self.Big5Freq[5][40] = 553;
        self.Big5Freq[26][145] = 552;
        self.Big5Freq[3][48] = 551;
        self.Big5Freq[3][70] = 550;
        self.Big5Freq[22][17] = 549;
        self.Big5Freq[16][47] = 548;
        self.Big5Freq[3][53] = 547;
        self.Big5Freq[4][24] = 546;
        self.Big5Freq[32][120] = 545;
        self.Big5Freq[24][49] = 544;
        self.Big5Freq[24][142] = 543;
        self.Big5Freq[18][66] = 542;
        self.Big5Freq[29][150] = 541;
        self.Big5Freq[5][122] = 540;
        self.Big5Freq[5][114] = 539;
        self.Big5Freq[3][44] = 538;
        self.Big5Freq[10][128] = 537;
        self.Big5Freq[15][20] = 536;
        self.Big5Freq[13][33] = 535;
        self.Big5Freq[14][87] = 534;
        self.Big5Freq[3][126] = 533;
        self.Big5Freq[4][53] = 532;
        self.Big5Freq[4][40] = 531;
        self.Big5Freq[9][93] = 530;
        self.Big5Freq[15][137] = 529;
        self.Big5Freq[10][123] = 528;
        self.Big5Freq[4][56] = 527;
        self.Big5Freq[5][71] = 526;
        self.Big5Freq[10][8] = 525;
        self.Big5Freq[5][16] = 524;
        self.Big5Freq[5][146] = 523;
        self.Big5Freq[18][88] = 522;
        self.Big5Freq[24][4] = 521;
        self.Big5Freq[20][47] = 520;
        self.Big5Freq[5][33] = 519;
        self.Big5Freq[9][43] = 518;
        self.Big5Freq[20][12] = 517;
        self.Big5Freq[20][13] = 516;
        self.Big5Freq[5][156] = 515;
        self.Big5Freq[22][140] = 514;
        self.Big5Freq[8][146] = 513;
        self.Big5Freq[21][123] = 512;
        self.Big5Freq[4][90] = 511;
        self.Big5Freq[5][62] = 510;
        self.Big5Freq[17][59] = 509;
        self.Big5Freq[10][37] = 508;
        self.Big5Freq[18][107] = 507;
        self.Big5Freq[14][53] = 506;
        self.Big5Freq[22][51] = 505;
        self.Big5Freq[8][13] = 504;
        self.Big5Freq[5][29] = 503;
        self.Big5Freq[9][7] = 502;
        self.Big5Freq[22][14] = 501;
        self.Big5Freq[8][55] = 500;
        self.Big5Freq[33][9] = 499;
        self.Big5Freq[16][64] = 498;
        self.Big5Freq[7][131] = 497;
        self.Big5Freq[34][4] = 496;
        self.Big5Freq[7][101] = 495;
        self.Big5Freq[11][139] = 494;
        self.Big5Freq[3][135] = 493;
        self.Big5Freq[7][102] = 492;
        self.Big5Freq[17][13] = 491;
        self.Big5Freq[3][20] = 490;
        self.Big5Freq[27][106] = 489;
        self.Big5Freq[5][88] = 488;
        self.Big5Freq[6][33] = 487;
        self.Big5Freq[5][139] = 486;
        self.Big5Freq[6][0] = 485;
        self.Big5Freq[17][58] = 484;
        self.Big5Freq[5][133] = 483;
        self.Big5Freq[9][107] = 482;
        self.Big5Freq[23][39] = 481;
        self.Big5Freq[5][23] = 480;
        self.Big5Freq[3][79] = 479;
        self.Big5Freq[32][97] = 478;
        self.Big5Freq[3][136] = 477;
        self.Big5Freq[4][94] = 476;
        self.Big5Freq[21][61] = 475;
        self.Big5Freq[23][123] = 474;
        self.Big5Freq[26][16] = 473;
        self.Big5Freq[24][137] = 472;
        self.Big5Freq[22][18] = 471;
        self.Big5Freq[5][1] = 470;
        self.Big5Freq[20][119] = 469;
        self.Big5Freq[3][7] = 468;
        self.Big5Freq[10][79] = 467;
        self.Big5Freq[15][105] = 466;
        self.Big5Freq[3][144] = 465;
        self.Big5Freq[12][80] = 464;
        self.Big5Freq[15][73] = 463;
        self.Big5Freq[3][19] = 462;
        self.Big5Freq[8][109] = 461;
        self.Big5Freq[3][15] = 460;
        self.Big5Freq[31][82] = 459;
        self.Big5Freq[3][43] = 458;
        self.Big5Freq[25][119] = 457;
        self.Big5Freq[16][111] = 456;
        self.Big5Freq[7][77] = 455;
        self.Big5Freq[3][95] = 454;
        self.Big5Freq[24][82] = 453;
        self.Big5Freq[7][52] = 452;
        self.Big5Freq[9][151] = 451;
        self.Big5Freq[3][129] = 450;
        self.Big5Freq[5][87] = 449;
        self.Big5Freq[3][55] = 448;
        self.Big5Freq[8][153] = 447;
        self.Big5Freq[4][83] = 446;
        self.Big5Freq[3][114] = 445;
        self.Big5Freq[23][147] = 444;
        self.Big5Freq[15][31] = 443;
        self.Big5Freq[3][54] = 442;
        self.Big5Freq[11][122] = 441;
        self.Big5Freq[4][4] = 440;
        self.Big5Freq[34][149] = 439;
        self.Big5Freq[3][17] = 438;
        self.Big5Freq[21][64] = 437;
        self.Big5Freq[26][144] = 436;
        self.Big5Freq[4][62] = 435;
        self.Big5Freq[8][15] = 434;
        self.Big5Freq[35][80] = 433;
        self.Big5Freq[7][110] = 432;
        self.Big5Freq[23][114] = 431;
        self.Big5Freq[3][108] = 430;
        self.Big5Freq[3][62] = 429;
        self.Big5Freq[21][41] = 428;
        self.Big5Freq[15][99] = 427;
        self.Big5Freq[5][47] = 426;
        self.Big5Freq[4][96] = 425;
        self.Big5Freq[20][122] = 424;
        self.Big5Freq[5][21] = 423;
        self.Big5Freq[4][157] = 422;
        self.Big5Freq[16][14] = 421;
        self.Big5Freq[3][117] = 420;
        self.Big5Freq[7][129] = 419;
        self.Big5Freq[4][27] = 418;
        self.Big5Freq[5][30] = 417;
        self.Big5Freq[22][16] = 416;
        self.Big5Freq[5][64] = 415;
        self.Big5Freq[17][99] = 414;
        self.Big5Freq[17][57] = 413;
        self.Big5Freq[8][105] = 412;
        self.Big5Freq[5][112] = 411;
        self.Big5Freq[20][59] = 410;
        self.Big5Freq[6][129] = 409;
        self.Big5Freq[18][17] = 408;
        self.Big5Freq[3][92] = 407;
        self.Big5Freq[28][118] = 406;
        self.Big5Freq[3][109] = 405;
        self.Big5Freq[31][51] = 404;
        self.Big5Freq[13][116] = 403;
        self.Big5Freq[6][15] = 402;
        self.Big5Freq[36][136] = 401;
        self.Big5Freq[12][74] = 400;
        self.Big5Freq[20][88] = 399;
        self.Big5Freq[36][68] = 398;
        self.Big5Freq[3][147] = 397;
        self.Big5Freq[15][84] = 396;
        self.Big5Freq[16][32] = 395;
        self.Big5Freq[16][58] = 394;
        self.Big5Freq[7][66] = 393;
        self.Big5Freq[23][107] = 392;
        self.Big5Freq[9][6] = 391;
        self.Big5Freq[12][86] = 390;
        self.Big5Freq[23][112] = 389;
        self.Big5Freq[37][23] = 388;
        self.Big5Freq[3][138] = 387;
        self.Big5Freq[20][68] = 386;
        self.Big5Freq[15][116] = 385;
        self.Big5Freq[18][64] = 384;
        self.Big5Freq[12][139] = 383;
        self.Big5Freq[11][155] = 382;
        self.Big5Freq[4][156] = 381;
        self.Big5Freq[12][84] = 380;
        self.Big5Freq[18][49] = 379;
        self.Big5Freq[25][125] = 378;
        self.Big5Freq[25][147] = 377;
        self.Big5Freq[15][110] = 376;
        self.Big5Freq[19][96] = 375;
        self.Big5Freq[30][152] = 374;
        self.Big5Freq[6][31] = 373;
        self.Big5Freq[27][117] = 372;
        self.Big5Freq[3][10] = 371;
        self.Big5Freq[6][131] = 370;
        self.Big5Freq[13][112] = 369;
        self.Big5Freq[36][156] = 368;
        self.Big5Freq[4][60] = 367;
        self.Big5Freq[15][121] = 366;
        self.Big5Freq[4][112] = 365;
        self.Big5Freq[30][142] = 364;
        self.Big5Freq[23][154] = 363;
        self.Big5Freq[27][101] = 362;
        self.Big5Freq[9][140] = 361;
        self.Big5Freq[3][89] = 360;
        self.Big5Freq[18][148] = 359;
        self.Big5Freq[4][69] = 358;
        self.Big5Freq[16][49] = 357;
        self.Big5Freq[6][117] = 356;
        self.Big5Freq[36][55] = 355;
        self.Big5Freq[5][123] = 354;
        self.Big5Freq[4][126] = 353;
        self.Big5Freq[4][119] = 352;
        self.Big5Freq[9][95] = 351;
        self.Big5Freq[5][24] = 350;
        self.Big5Freq[16][133] = 349;
        self.Big5Freq[10][134] = 348;
        self.Big5Freq[26][59] = 347;
        self.Big5Freq[6][41] = 346;
        self.Big5Freq[6][146] = 345;
        self.Big5Freq[19][24] = 344;
        self.Big5Freq[5][113] = 343;
        self.Big5Freq[10][118] = 342;
        self.Big5Freq[34][151] = 341;
        self.Big5Freq[9][72] = 340;
        self.Big5Freq[31][25] = 339;
        self.Big5Freq[18][126] = 338;
        self.Big5Freq[18][28] = 337;
        self.Big5Freq[4][153] = 336;
        self.Big5Freq[3][84] = 335;
        self.Big5Freq[21][18] = 334;
        self.Big5Freq[25][129] = 333;
        self.Big5Freq[6][107] = 332;
        self.Big5Freq[12][25] = 331;
        self.Big5Freq[17][109] = 330;
        self.Big5Freq[7][76] = 329;
        self.Big5Freq[15][15] = 328;
        self.Big5Freq[4][14] = 327;
        self.Big5Freq[23][88] = 326;
        self.Big5Freq[18][2] = 325;
        self.Big5Freq[6][88] = 324;
        self.Big5Freq[16][84] = 323;
        self.Big5Freq[12][48] = 322;
        self.Big5Freq[7][68] = 321;
        self.Big5Freq[5][50] = 320;
        self.Big5Freq[13][54] = 319;
        self.Big5Freq[7][98] = 318;
        self.Big5Freq[11][6] = 317;
        self.Big5Freq[9][80] = 316;
        self.Big5Freq[16][41] = 315;
        self.Big5Freq[7][43] = 314;
        self.Big5Freq[28][117] = 313;
        self.Big5Freq[3][51] = 312;
        self.Big5Freq[7][3] = 311;
        self.Big5Freq[20][81] = 310;
        self.Big5Freq[4][2] = 309;
        self.Big5Freq[11][16] = 308;
        self.Big5Freq[10][4] = 307;
        self.Big5Freq[10][119] = 306;
        self.Big5Freq[6][142] = 305;
        self.Big5Freq[18][51] = 304;
        self.Big5Freq[8][144] = 303;
        self.Big5Freq[10][65] = 302;
        self.Big5Freq[11][64] = 301;
        self.Big5Freq[11][130] = 300;
        self.Big5Freq[9][92] = 299;
        self.Big5Freq[18][29] = 298;
        self.Big5Freq[18][78] = 297;
        self.Big5Freq[18][151] = 296;
        self.Big5Freq[33][127] = 295;
        self.Big5Freq[35][113] = 294;
        self.Big5Freq[10][155] = 293;
        self.Big5Freq[3][76] = 292;
        self.Big5Freq[36][123] = 291;
        self.Big5Freq[13][143] = 290;
        self.Big5Freq[5][135] = 289;
        self.Big5Freq[23][116] = 288;
        self.Big5Freq[6][101] = 287;
        self.Big5Freq[14][74] = 286;
        self.Big5Freq[7][153] = 285;
        self.Big5Freq[3][101] = 284;
        self.Big5Freq[9][74] = 283;
        self.Big5Freq[3][156] = 282;
        self.Big5Freq[4][147] = 281;
        self.Big5Freq[9][12] = 280;
        self.Big5Freq[18][133] = 279;
        self.Big5Freq[4][0] = 278;
        self.Big5Freq[7][155] = 277;
        self.Big5Freq[9][144] = 276;
        self.Big5Freq[23][49] = 275;
        self.Big5Freq[5][89] = 274;
        self.Big5Freq[10][11] = 273;
        self.Big5Freq[3][110] = 272;
        self.Big5Freq[3][40] = 271;
        self.Big5Freq[29][115] = 270;
        self.Big5Freq[9][100] = 269;
        self.Big5Freq[21][67] = 268;
        self.Big5Freq[23][145] = 267;
        self.Big5Freq[10][47] = 266;
        self.Big5Freq[4][31] = 265;
        self.Big5Freq[4][81] = 264;
        self.Big5Freq[22][62] = 263;
        self.Big5Freq[4][28] = 262;
        self.Big5Freq[27][39] = 261;
        self.Big5Freq[27][54] = 260;
        self.Big5Freq[32][46] = 259;
        self.Big5Freq[4][76] = 258;
        self.Big5Freq[26][15] = 257;
        self.Big5Freq[12][154] = 256;
        self.Big5Freq[9][150] = 255;
        self.Big5Freq[15][17] = 254;
        self.Big5Freq[5][129] = 253;
        self.Big5Freq[10][40] = 252;
        self.Big5Freq[13][37] = 251;
        self.Big5Freq[31][104] = 250;
        self.Big5Freq[3][152] = 249;
        self.Big5Freq[5][22] = 248;
        self.Big5Freq[8][48] = 247;
        self.Big5Freq[4][74] = 246;
        self.Big5Freq[6][17] = 245;
        self.Big5Freq[30][82] = 244;
        self.Big5Freq[4][116] = 243;
        self.Big5Freq[16][42] = 242;
        self.Big5Freq[5][55] = 241;
        self.Big5Freq[4][64] = 240;
        self.Big5Freq[14][19] = 239;
        self.Big5Freq[35][82] = 238;
        self.Big5Freq[30][139] = 237;
        self.Big5Freq[26][152] = 236;
        self.Big5Freq[32][32] = 235;
        self.Big5Freq[21][102] = 234;
        self.Big5Freq[10][131] = 233;
        self.Big5Freq[9][128] = 232;
        self.Big5Freq[3][87] = 231;
        self.Big5Freq[4][51] = 230;
        self.Big5Freq[10][15] = 229;
        self.Big5Freq[4][150] = 228;
        self.Big5Freq[7][4] = 227;
        self.Big5Freq[7][51] = 226;
        self.Big5Freq[7][157] = 225;
        self.Big5Freq[4][146] = 224;
        self.Big5Freq[4][91] = 223;
        self.Big5Freq[7][13] = 222;
        self.Big5Freq[17][116] = 221;
        self.Big5Freq[23][21] = 220;
        self.Big5Freq[5][106] = 219;
        self.Big5Freq[14][100] = 218;
        self.Big5Freq[10][152] = 217;
        self.Big5Freq[14][89] = 216;
        self.Big5Freq[6][138] = 215;
        self.Big5Freq[12][157] = 214;
        self.Big5Freq[10][102] = 213;
        self.Big5Freq[19][94] = 212;
        self.Big5Freq[7][74] = 211;
        self.Big5Freq[18][128] = 210;
        self.Big5Freq[27][111] = 209;
        self.Big5Freq[11][57] = 208;
        self.Big5Freq[3][131] = 207;
        self.Big5Freq[30][23] = 206;
        self.Big5Freq[30][126] = 205;
        self.Big5Freq[4][36] = 204;
        self.Big5Freq[26][124] = 203;
        self.Big5Freq[4][19] = 202;
        self.Big5Freq[9][152] = 201;
        /*
         * Big5Freq[5][0] = 200; Big5Freq[26][57] = 199; Big5Freq[13][155] =
         * 198; Big5Freq[3][38] = 197; Big5Freq[9][155] = 196; Big5Freq[28][53]
         * = 195; Big5Freq[15][71] = 194; Big5Freq[21][95] = 193;
         * Big5Freq[15][112] = 192; Big5Freq[14][138] = 191; Big5Freq[8][18] =
         * 190; Big5Freq[20][151] = 189; Big5Freq[37][27] = 188;
         * Big5Freq[32][48] = 187; Big5Freq[23][66] = 186; Big5Freq[9][2] = 185;
         * Big5Freq[13][133] = 184; Big5Freq[7][127] = 183; Big5Freq[3][11] =
         * 182; Big5Freq[12][118] = 181; Big5Freq[13][101] = 180;
         * Big5Freq[30][153] = 179; Big5Freq[4][65] = 178; Big5Freq[5][25] =
         * 177; Big5Freq[5][140] = 176; Big5Freq[6][25] = 175; Big5Freq[4][52] =
         * 174; Big5Freq[30][156] = 173; Big5Freq[16][13] = 172; Big5Freq[21][8]
         * = 171; Big5Freq[19][74] = 170; Big5Freq[15][145] = 169;
         * Big5Freq[9][15] = 168; Big5Freq[13][82] = 167; Big5Freq[26][86] =
         * 166; Big5Freq[18][52] = 165; Big5Freq[6][109] = 164; Big5Freq[10][99]
         * = 163; Big5Freq[18][101] = 162; Big5Freq[25][49] = 161;
         * Big5Freq[31][79] = 160; Big5Freq[28][20] = 159; Big5Freq[12][115] =
         * 158; Big5Freq[15][66] = 157; Big5Freq[11][104] = 156;
         * Big5Freq[23][106] = 155; Big5Freq[34][157] = 154; Big5Freq[32][94] =
         * 153; Big5Freq[29][88] = 152; Big5Freq[10][46] = 151;
         * Big5Freq[13][118] = 150; Big5Freq[20][37] = 149; Big5Freq[12][30] =
         * 148; Big5Freq[21][4] = 147; Big5Freq[16][33] = 146; Big5Freq[13][52]
         * = 145; Big5Freq[4][7] = 144; Big5Freq[21][49] = 143; Big5Freq[3][27]
         * = 142; Big5Freq[16][91] = 141; Big5Freq[5][155] = 140;
         * Big5Freq[29][130] = 139; Big5Freq[3][125] = 138; Big5Freq[14][26] =
         * 137; Big5Freq[15][39] = 136; Big5Freq[24][110] = 135;
         * Big5Freq[7][141] = 134; Big5Freq[21][15] = 133; Big5Freq[32][104] =
         * 132; Big5Freq[8][31] = 131; Big5Freq[34][112] = 130; Big5Freq[10][75]
         * = 129; Big5Freq[21][23] = 128; Big5Freq[34][131] = 127;
         * Big5Freq[12][3] = 126; Big5Freq[10][62] = 125; Big5Freq[9][120] =
         * 124; Big5Freq[32][149] = 123; Big5Freq[8][44] = 122; Big5Freq[24][2]
         * = 121; Big5Freq[6][148] = 120; Big5Freq[15][103] = 119;
         * Big5Freq[36][54] = 118; Big5Freq[36][134] = 117; Big5Freq[11][7] =
         * 116; Big5Freq[3][90] = 115; Big5Freq[36][73] = 114; Big5Freq[8][102]
         * = 113; Big5Freq[12][87] = 112; Big5Freq[25][64] = 111; Big5Freq[9][1]
         * = 110; Big5Freq[24][121] = 109; Big5Freq[5][75] = 108;
         * Big5Freq[17][83] = 107; Big5Freq[18][57] = 106; Big5Freq[8][95] =
         * 105; Big5Freq[14][36] = 104; Big5Freq[28][113] = 103;
         * Big5Freq[12][56] = 102; Big5Freq[14][61] = 101; Big5Freq[25][138] =
         * 100; Big5Freq[4][34] = 99; Big5Freq[11][152] = 98; Big5Freq[35][0] =
         * 97; Big5Freq[4][15] = 96; Big5Freq[8][82] = 95; Big5Freq[20][73] =
         * 94; Big5Freq[25][52] = 93; Big5Freq[24][6] = 92; Big5Freq[21][78] =
         * 91; Big5Freq[17][32] = 90; Big5Freq[17][91] = 89; Big5Freq[5][76] =
         * 88; Big5Freq[15][60] = 87; Big5Freq[15][150] = 86; Big5Freq[5][80] =
         * 85; Big5Freq[15][81] = 84; Big5Freq[28][108] = 83; Big5Freq[18][14] =
         * 82; Big5Freq[19][109] = 81; Big5Freq[28][133] = 80; Big5Freq[21][97]
         * = 79; Big5Freq[5][105] = 78; Big5Freq[18][114] = 77; Big5Freq[16][95]
         * = 76; Big5Freq[5][51] = 75; Big5Freq[3][148] = 74; Big5Freq[22][102]
         * = 73; Big5Freq[4][123] = 72; Big5Freq[8][88] = 71; Big5Freq[25][111]
         * = 70; Big5Freq[8][149] = 69; Big5Freq[9][48] = 68; Big5Freq[16][126]
         * = 67; Big5Freq[33][150] = 66; Big5Freq[9][54] = 65; Big5Freq[29][104]
         * = 64; Big5Freq[3][3] = 63; Big5Freq[11][49] = 62; Big5Freq[24][109] =
         * 61; Big5Freq[28][116] = 60; Big5Freq[34][113] = 59; Big5Freq[5][3] =
         * 58; Big5Freq[21][106] = 57; Big5Freq[4][98] = 56; Big5Freq[12][135] =
         * 55; Big5Freq[16][101] = 54; Big5Freq[12][147] = 53; Big5Freq[27][55]
         * = 52; Big5Freq[3][5] = 51; Big5Freq[11][101] = 50; Big5Freq[16][157]
         * = 49; Big5Freq[22][114] = 48; Big5Freq[18][46] = 47; Big5Freq[4][29]
         * = 46; Big5Freq[8][103] = 45; Big5Freq[16][151] = 44; Big5Freq[8][29]
         * = 43; Big5Freq[15][114] = 42; Big5Freq[22][70] = 41;
         * Big5Freq[13][121] = 40; Big5Freq[7][112] = 39; Big5Freq[20][83] = 38;
         * Big5Freq[3][36] = 37; Big5Freq[10][103] = 36; Big5Freq[3][96] = 35;
         * Big5Freq[21][79] = 34; Big5Freq[25][120] = 33; Big5Freq[29][121] =
         * 32; Big5Freq[23][71] = 31; Big5Freq[21][22] = 30; Big5Freq[18][89] =
         * 29; Big5Freq[25][104] = 28; Big5Freq[10][124] = 27; Big5Freq[26][4] =
         * 26; Big5Freq[21][136] = 25; Big5Freq[6][112] = 24; Big5Freq[12][103]
         * = 23; Big5Freq[17][66] = 22; Big5Freq[13][151] = 21;
         * Big5Freq[33][152] = 20; Big5Freq[11][148] = 19; Big5Freq[13][57] =
         * 18; Big5Freq[13][41] = 17; Big5Freq[7][60] = 16; Big5Freq[21][29] =
         * 15; Big5Freq[9][157] = 14; Big5Freq[24][95] = 13; Big5Freq[15][148] =
         * 12; Big5Freq[15][122] = 11; Big5Freq[6][125] = 10; Big5Freq[11][25] =
         * 9; Big5Freq[20][55] = 8; Big5Freq[19][84] = 7; Big5Freq[21][82] = 6;
         * Big5Freq[24][3] = 5; Big5Freq[13][70] = 4; Big5Freq[6][21] = 3;
         * Big5Freq[21][86] = 2; Big5Freq[12][23] = 1; Big5Freq[3][85] = 0;
         * EUC_TWFreq[45][90] = 600;
         */
        self.Big5PFreq[41][122] = 600;
        self.Big5PFreq[35][0] = 599;
        self.Big5PFreq[43][15] = 598;
        self.Big5PFreq[35][99] = 597;
        self.Big5PFreq[35][6] = 596;
        self.Big5PFreq[35][8] = 595;
        self.Big5PFreq[38][154] = 594;
        self.Big5PFreq[37][34] = 593;
        self.Big5PFreq[37][115] = 592;
        self.Big5PFreq[36][12] = 591;
        self.Big5PFreq[18][77] = 590;
        self.Big5PFreq[35][100] = 589;
        self.Big5PFreq[35][42] = 588;
        self.Big5PFreq[120][75] = 587;
        self.Big5PFreq[35][23] = 586;
        self.Big5PFreq[13][72] = 585;
        self.Big5PFreq[0][67] = 584;
        self.Big5PFreq[39][172] = 583;
        self.Big5PFreq[22][182] = 582;
        self.Big5PFreq[15][186] = 581;
        self.Big5PFreq[15][165] = 580;
        self.Big5PFreq[35][44] = 579;
        self.Big5PFreq[40][13] = 578;
        self.Big5PFreq[38][1] = 577;
        self.Big5PFreq[37][33] = 576;
        self.Big5PFreq[36][24] = 575;
        self.Big5PFreq[56][4] = 574;
        self.Big5PFreq[35][29] = 573;
        self.Big5PFreq[9][96] = 572;
        self.Big5PFreq[37][62] = 571;
        self.Big5PFreq[48][47] = 570;
        self.Big5PFreq[51][14] = 569;
        self.Big5PFreq[39][122] = 568;
        self.Big5PFreq[44][46] = 567;
        self.Big5PFreq[35][21] = 566;
        self.Big5PFreq[36][8] = 565;
        self.Big5PFreq[36][141] = 564;
        self.Big5PFreq[3][81] = 563;
        self.Big5PFreq[37][155] = 562;
        self.Big5PFreq[42][84] = 561;
        self.Big5PFreq[36][40] = 560;
        self.Big5PFreq[35][103] = 559;
        self.Big5PFreq[11][84] = 558;
        self.Big5PFreq[45][33] = 557;
        self.Big5PFreq[121][79] = 556;
        self.Big5PFreq[2][77] = 555;
        self.Big5PFreq[36][41] = 554;
        self.Big5PFreq[37][47] = 553;
        self.Big5PFreq[39][125] = 552;
        self.Big5PFreq[37][26] = 551;
        self.Big5PFreq[35][48] = 550;
        self.Big5PFreq[35][28] = 549;
        self.Big5PFreq[35][159] = 548;
        self.Big5PFreq[37][40] = 547;
        self.Big5PFreq[35][145] = 546;
        self.Big5PFreq[37][147] = 545;
        self.Big5PFreq[46][160] = 544;
        self.Big5PFreq[37][46] = 543;
        self.Big5PFreq[50][99] = 542;
        self.Big5PFreq[52][13] = 541;
        self.Big5PFreq[10][82] = 540;
        self.Big5PFreq[35][169] = 539;
        self.Big5PFreq[35][31] = 538;
        self.Big5PFreq[47][31] = 537;
        self.Big5PFreq[18][79] = 536;
        self.Big5PFreq[16][113] = 535;
        self.Big5PFreq[37][104] = 534;
        self.Big5PFreq[39][134] = 533;
        self.Big5PFreq[36][53] = 532;
        self.Big5PFreq[38][0] = 531;
        self.Big5PFreq[4][86] = 530;
        self.Big5PFreq[54][17] = 529;
        self.Big5PFreq[43][157] = 528;
        self.Big5PFreq[35][165] = 527;
        self.Big5PFreq[69][147] = 526;
        self.Big5PFreq[117][95] = 525;
        self.Big5PFreq[35][162] = 524;
        self.Big5PFreq[35][17] = 523;
        self.Big5PFreq[36][142] = 522;
        self.Big5PFreq[36][4] = 521;
        self.Big5PFreq[37][166] = 520;
        self.Big5PFreq[35][168] = 519;
        self.Big5PFreq[35][19] = 518;
        self.Big5PFreq[37][48] = 517;
        self.Big5PFreq[42][37] = 516;
        self.Big5PFreq[40][146] = 515;
        self.Big5PFreq[36][123] = 514;
        self.Big5PFreq[22][41] = 513;
        self.Big5PFreq[20][119] = 512;
        self.Big5PFreq[2][74] = 511;
        self.Big5PFreq[44][113] = 510;
        self.Big5PFreq[35][125] = 509;
        self.Big5PFreq[37][16] = 508;
        self.Big5PFreq[35][20] = 507;
        self.Big5PFreq[35][55] = 506;
        self.Big5PFreq[37][145] = 505;
        self.Big5PFreq[0][88] = 504;
        self.Big5PFreq[3][94] = 503;
        self.Big5PFreq[6][65] = 502;
        self.Big5PFreq[26][15] = 501;
        self.Big5PFreq[41][126] = 500;
        self.Big5PFreq[36][129] = 499;
        self.Big5PFreq[31][75] = 498;
        self.Big5PFreq[19][61] = 497;
        self.Big5PFreq[35][128] = 496;
        self.Big5PFreq[29][79] = 495;
        self.Big5PFreq[36][62] = 494;
        self.Big5PFreq[37][189] = 493;
        self.Big5PFreq[39][109] = 492;
        self.Big5PFreq[39][135] = 491;
        self.Big5PFreq[72][15] = 490;
        self.Big5PFreq[47][106] = 489;
        self.Big5PFreq[54][14] = 488;
        self.Big5PFreq[24][52] = 487;
        self.Big5PFreq[38][162] = 486;
        self.Big5PFreq[41][43] = 485;
        self.Big5PFreq[37][121] = 484;
        self.Big5PFreq[14][66] = 483;
        self.Big5PFreq[37][30] = 482;
        self.Big5PFreq[35][7] = 481;
        self.Big5PFreq[49][58] = 480;
        self.Big5PFreq[43][188] = 479;
        self.Big5PFreq[24][66] = 478;
        self.Big5PFreq[35][171] = 477;
        self.Big5PFreq[40][186] = 476;
        self.Big5PFreq[39][164] = 475;
        self.Big5PFreq[78][186] = 474;
        self.Big5PFreq[8][72] = 473;
        self.Big5PFreq[36][190] = 472;
        self.Big5PFreq[35][53] = 471;
        self.Big5PFreq[35][54] = 470;
        self.Big5PFreq[22][159] = 469;
        self.Big5PFreq[35][9] = 468;
        self.Big5PFreq[41][140] = 467;
        self.Big5PFreq[37][22] = 466;
        self.Big5PFreq[48][97] = 465;
        self.Big5PFreq[50][97] = 464;
        self.Big5PFreq[36][127] = 463;
        self.Big5PFreq[37][23] = 462;
        self.Big5PFreq[40][55] = 461;
        self.Big5PFreq[35][43] = 460;
        self.Big5PFreq[26][22] = 459;
        self.Big5PFreq[35][15] = 458;
        self.Big5PFreq[72][179] = 457;
        self.Big5PFreq[20][129] = 456;
        self.Big5PFreq[52][101] = 455;
        self.Big5PFreq[35][12] = 454;
        self.Big5PFreq[42][156] = 453;
        self.Big5PFreq[15][157] = 452;
        self.Big5PFreq[50][140] = 451;
        self.Big5PFreq[26][28] = 450;
        self.Big5PFreq[54][51] = 449;
        self.Big5PFreq[35][112] = 448;
        self.Big5PFreq[36][116] = 447;
        self.Big5PFreq[42][11] = 446;
        self.Big5PFreq[37][172] = 445;
        self.Big5PFreq[37][29] = 444;
        self.Big5PFreq[44][107] = 443;
        self.Big5PFreq[50][17] = 442;
        self.Big5PFreq[39][107] = 441;
        self.Big5PFreq[19][109] = 440;
        self.Big5PFreq[36][60] = 439;
        self.Big5PFreq[49][132] = 438;
        self.Big5PFreq[26][16] = 437;
        self.Big5PFreq[43][155] = 436;
        self.Big5PFreq[37][120] = 435;
        self.Big5PFreq[15][159] = 434;
        self.Big5PFreq[43][6] = 433;
        self.Big5PFreq[45][188] = 432;
        self.Big5PFreq[35][38] = 431;
        self.Big5PFreq[39][143] = 430;
        self.Big5PFreq[48][144] = 429;
        self.Big5PFreq[37][168] = 428;
        self.Big5PFreq[37][1] = 427;
        self.Big5PFreq[36][109] = 426;
        self.Big5PFreq[46][53] = 425;
        self.Big5PFreq[38][54] = 424;
        self.Big5PFreq[36][0] = 423;
        self.Big5PFreq[72][33] = 422;
        self.Big5PFreq[42][8] = 421;
        self.Big5PFreq[36][31] = 420;
        self.Big5PFreq[35][150] = 419;
        self.Big5PFreq[118][93] = 418;
        self.Big5PFreq[37][61] = 417;
        self.Big5PFreq[0][85] = 416;
        self.Big5PFreq[36][27] = 415;
        self.Big5PFreq[35][134] = 414;
        self.Big5PFreq[36][145] = 413;
        self.Big5PFreq[6][96] = 412;
        self.Big5PFreq[36][14] = 411;
        self.Big5PFreq[16][36] = 410;
        self.Big5PFreq[15][175] = 409;
        self.Big5PFreq[35][10] = 408;
        self.Big5PFreq[36][189] = 407;
        self.Big5PFreq[35][51] = 406;
        self.Big5PFreq[35][109] = 405;
        self.Big5PFreq[35][147] = 404;
        self.Big5PFreq[35][180] = 403;
        self.Big5PFreq[72][5] = 402;
        self.Big5PFreq[36][107] = 401;
        self.Big5PFreq[49][116] = 400;
        self.Big5PFreq[73][30] = 399;
        self.Big5PFreq[6][90] = 398;
        self.Big5PFreq[2][70] = 397;
        self.Big5PFreq[17][141] = 396;
        self.Big5PFreq[35][62] = 395;
        self.Big5PFreq[16][180] = 394;
        self.Big5PFreq[4][91] = 393;
        self.Big5PFreq[15][171] = 392;
        self.Big5PFreq[35][177] = 391;
        self.Big5PFreq[37][173] = 390;
        self.Big5PFreq[16][121] = 389;
        self.Big5PFreq[35][5] = 388;
        self.Big5PFreq[46][122] = 387;
        self.Big5PFreq[40][138] = 386;
        self.Big5PFreq[50][49] = 385;
        self.Big5PFreq[36][152] = 384;
        self.Big5PFreq[13][43] = 383;
        self.Big5PFreq[9][88] = 382;
        self.Big5PFreq[36][159] = 381;
        self.Big5PFreq[27][62] = 380;
        self.Big5PFreq[40][18] = 379;
        self.Big5PFreq[17][129] = 378;
        self.Big5PFreq[43][97] = 377;
        self.Big5PFreq[13][131] = 376;
        self.Big5PFreq[46][107] = 375;
        self.Big5PFreq[60][64] = 374;
        self.Big5PFreq[36][179] = 373;
        self.Big5PFreq[37][55] = 372;
        self.Big5PFreq[41][173] = 371;
        self.Big5PFreq[44][172] = 370;
        self.Big5PFreq[23][187] = 369;
        self.Big5PFreq[36][149] = 368;
        self.Big5PFreq[17][125] = 367;
        self.Big5PFreq[55][180] = 366;
        self.Big5PFreq[51][129] = 365;
        self.Big5PFreq[36][51] = 364;
        self.Big5PFreq[37][122] = 363;
        self.Big5PFreq[48][32] = 362;
        self.Big5PFreq[51][99] = 361;
        self.Big5PFreq[54][16] = 360;
        self.Big5PFreq[41][183] = 359;
        self.Big5PFreq[37][179] = 358;
        self.Big5PFreq[38][179] = 357;
        self.Big5PFreq[35][143] = 356;
        self.Big5PFreq[37][24] = 355;
        self.Big5PFreq[40][177] = 354;
        self.Big5PFreq[47][117] = 353;
        self.Big5PFreq[39][52] = 352;
        self.Big5PFreq[22][99] = 351;
        self.Big5PFreq[40][142] = 350;
        self.Big5PFreq[36][49] = 349;
        self.Big5PFreq[38][17] = 348;
        self.Big5PFreq[39][188] = 347;
        self.Big5PFreq[36][186] = 346;
        self.Big5PFreq[35][189] = 345;
        self.Big5PFreq[41][7] = 344;
        self.Big5PFreq[18][91] = 343;
        self.Big5PFreq[43][137] = 342;
        self.Big5PFreq[35][142] = 341;
        self.Big5PFreq[35][117] = 340;
        self.Big5PFreq[39][138] = 339;
        self.Big5PFreq[16][59] = 338;
        self.Big5PFreq[39][174] = 337;
        self.Big5PFreq[55][145] = 336;
        self.Big5PFreq[37][21] = 335;
        self.Big5PFreq[36][180] = 334;
        self.Big5PFreq[37][156] = 333;
        self.Big5PFreq[49][13] = 332;
        self.Big5PFreq[41][107] = 331;
        self.Big5PFreq[36][56] = 330;
        self.Big5PFreq[53][8] = 329;
        self.Big5PFreq[22][114] = 328;
        self.Big5PFreq[5][95] = 327;
        self.Big5PFreq[37][0] = 326;
        self.Big5PFreq[26][183] = 325;
        self.Big5PFreq[22][66] = 324;
        self.Big5PFreq[35][58] = 323;
        self.Big5PFreq[48][117] = 322;
        self.Big5PFreq[36][102] = 321;
        self.Big5PFreq[22][122] = 320;
        self.Big5PFreq[35][11] = 319;
        self.Big5PFreq[46][19] = 318;
        self.Big5PFreq[22][49] = 317;
        self.Big5PFreq[48][166] = 316;
        self.Big5PFreq[41][125] = 315;
        self.Big5PFreq[41][1] = 314;
        self.Big5PFreq[35][178] = 313;
        self.Big5PFreq[41][12] = 312;
        self.Big5PFreq[26][167] = 311;
        self.Big5PFreq[42][152] = 310;
        self.Big5PFreq[42][46] = 309;
        self.Big5PFreq[42][151] = 308;
        self.Big5PFreq[20][135] = 307;
        self.Big5PFreq[37][162] = 306;
        self.Big5PFreq[37][50] = 305;
        self.Big5PFreq[22][185] = 304;
        self.Big5PFreq[36][166] = 303;
        self.Big5PFreq[19][40] = 302;
        self.Big5PFreq[22][107] = 301;
        self.Big5PFreq[22][102] = 300;
        self.Big5PFreq[57][162] = 299;
        self.Big5PFreq[22][124] = 298;
        self.Big5PFreq[37][138] = 297;
        self.Big5PFreq[37][25] = 296;
        self.Big5PFreq[0][69] = 295;
        self.Big5PFreq[43][172] = 294;
        self.Big5PFreq[42][167] = 293;
        self.Big5PFreq[35][120] = 292;
        self.Big5PFreq[41][128] = 291;
        self.Big5PFreq[2][88] = 290;
        self.Big5PFreq[20][123] = 289;
        self.Big5PFreq[35][123] = 288;
        self.Big5PFreq[36][28] = 287;
        self.Big5PFreq[42][188] = 286;
        self.Big5PFreq[42][164] = 285;
        self.Big5PFreq[42][4] = 284;
        self.Big5PFreq[43][57] = 283;
        self.Big5PFreq[39][3] = 282;
        self.Big5PFreq[42][3] = 281;
        self.Big5PFreq[57][158] = 280;
        self.Big5PFreq[35][146] = 279;
        self.Big5PFreq[24][54] = 278;
        self.Big5PFreq[13][110] = 277;
        self.Big5PFreq[23][132] = 276;
        self.Big5PFreq[26][102] = 275;
        self.Big5PFreq[55][178] = 274;
        self.Big5PFreq[17][117] = 273;
        self.Big5PFreq[41][161] = 272;
        self.Big5PFreq[38][150] = 271;
        self.Big5PFreq[10][71] = 270;
        self.Big5PFreq[47][60] = 269;
        self.Big5PFreq[16][114] = 268;
        self.Big5PFreq[21][47] = 267;
        self.Big5PFreq[39][101] = 266;
        self.Big5PFreq[18][45] = 265;
        self.Big5PFreq[40][121] = 264;
        self.Big5PFreq[45][41] = 263;
        self.Big5PFreq[22][167] = 262;
        self.Big5PFreq[26][149] = 261;
        self.Big5PFreq[15][189] = 260;
        self.Big5PFreq[41][177] = 259;
        self.Big5PFreq[46][36] = 258;
        self.Big5PFreq[20][40] = 257;
        self.Big5PFreq[41][54] = 256;
        self.Big5PFreq[3][87] = 255;
        self.Big5PFreq[40][16] = 254;
        self.Big5PFreq[42][15] = 253;
        self.Big5PFreq[11][83] = 252;
        self.Big5PFreq[0][94] = 251;
        self.Big5PFreq[122][81] = 250;
        self.Big5PFreq[41][26] = 249;
        self.Big5PFreq[36][34] = 248;
        self.Big5PFreq[44][148] = 247;
        self.Big5PFreq[35][3] = 246;
        self.Big5PFreq[36][114] = 245;
        self.Big5PFreq[42][112] = 244;
        self.Big5PFreq[35][183] = 243;
        self.Big5PFreq[49][73] = 242;
        self.Big5PFreq[39][2] = 241;
        self.Big5PFreq[38][121] = 240;
        self.Big5PFreq[44][114] = 239;
        self.Big5PFreq[49][32] = 238;
        self.Big5PFreq[1][65] = 237;
        self.Big5PFreq[38][25] = 236;
        self.Big5PFreq[39][4] = 235;
        self.Big5PFreq[42][62] = 234;
        self.Big5PFreq[35][40] = 233;
        self.Big5PFreq[24][2] = 232;
        self.Big5PFreq[53][49] = 231;
        self.Big5PFreq[41][133] = 230;
        self.Big5PFreq[43][134] = 229;
        self.Big5PFreq[3][83] = 228;
        self.Big5PFreq[38][158] = 227;
        self.Big5PFreq[24][17] = 226;
        self.Big5PFreq[52][59] = 225;
        self.Big5PFreq[38][41] = 224;
        self.Big5PFreq[37][127] = 223;
        self.Big5PFreq[22][175] = 222;
        self.Big5PFreq[44][30] = 221;
        self.Big5PFreq[47][178] = 220;
        self.Big5PFreq[43][99] = 219;
        self.Big5PFreq[19][4] = 218;
        self.Big5PFreq[37][97] = 217;
        self.Big5PFreq[38][181] = 216;
        self.Big5PFreq[45][103] = 215;
        self.Big5PFreq[1][86] = 214;
        self.Big5PFreq[40][15] = 213;
        self.Big5PFreq[22][136] = 212;
        self.Big5PFreq[75][165] = 211;
        self.Big5PFreq[36][15] = 210;
        self.Big5PFreq[46][80] = 209;
        self.Big5PFreq[59][55] = 208;
        self.Big5PFreq[37][108] = 207;
        self.Big5PFreq[21][109] = 206;
        self.Big5PFreq[24][165] = 205;
        self.Big5PFreq[79][158] = 204;
        self.Big5PFreq[44][139] = 203;
        self.Big5PFreq[36][124] = 202;
        self.Big5PFreq[42][185] = 201;
        self.Big5PFreq[39][186] = 200;
        self.Big5PFreq[22][128] = 199;
        self.Big5PFreq[40][44] = 198;
        self.Big5PFreq[41][105] = 197;
        self.Big5PFreq[1][70] = 196;
        self.Big5PFreq[1][68] = 195;
        self.Big5PFreq[53][22] = 194;
        self.Big5PFreq[36][54] = 193;
        self.Big5PFreq[47][147] = 192;
        self.Big5PFreq[35][36] = 191;
        self.Big5PFreq[35][185] = 190;
        self.Big5PFreq[45][37] = 189;
        self.Big5PFreq[43][163] = 188;
        self.Big5PFreq[56][115] = 187;
        self.Big5PFreq[38][164] = 186;
        self.Big5PFreq[35][141] = 185;
        self.Big5PFreq[42][132] = 184;
        self.Big5PFreq[46][120] = 183;
        self.Big5PFreq[69][142] = 182;
        self.Big5PFreq[38][175] = 181;
        self.Big5PFreq[22][112] = 180;
        self.Big5PFreq[38][142] = 179;
        self.Big5PFreq[40][37] = 178;
        self.Big5PFreq[37][109] = 177;
        self.Big5PFreq[40][144] = 176;
        self.Big5PFreq[44][117] = 175;
        self.Big5PFreq[35][181] = 174;
        self.Big5PFreq[26][105] = 173;
        self.Big5PFreq[16][48] = 172;
        self.Big5PFreq[44][122] = 171;
        self.Big5PFreq[12][86] = 170;
        self.Big5PFreq[84][53] = 169;
        self.Big5PFreq[17][44] = 168;
        self.Big5PFreq[59][54] = 167;
        self.Big5PFreq[36][98] = 166;
        self.Big5PFreq[45][115] = 165;
        self.Big5PFreq[73][9] = 164;
        self.Big5PFreq[44][123] = 163;
        self.Big5PFreq[37][188] = 162;
        self.Big5PFreq[51][117] = 161;
        self.Big5PFreq[15][156] = 160;
        self.Big5PFreq[36][155] = 159;
        self.Big5PFreq[44][25] = 158;
        self.Big5PFreq[38][12] = 157;
        self.Big5PFreq[38][140] = 156;
        self.Big5PFreq[23][4] = 155;
        self.Big5PFreq[45][149] = 154;
        self.Big5PFreq[22][189] = 153;
        self.Big5PFreq[38][147] = 152;
        self.Big5PFreq[27][5] = 151;
        self.Big5PFreq[22][42] = 150;
        self.Big5PFreq[3][68] = 149;
        self.Big5PFreq[39][51] = 148;
        self.Big5PFreq[36][29] = 147;
        self.Big5PFreq[20][108] = 146;
        self.Big5PFreq[50][57] = 145;
        self.Big5PFreq[55][104] = 144;
        self.Big5PFreq[22][46] = 143;
        self.Big5PFreq[18][164] = 142;
        self.Big5PFreq[50][159] = 141;
        self.Big5PFreq[85][131] = 140;
        self.Big5PFreq[26][79] = 139;
        self.Big5PFreq[38][100] = 138;
        self.Big5PFreq[53][112] = 137;
        self.Big5PFreq[20][190] = 136;
        self.Big5PFreq[14][69] = 135;
        self.Big5PFreq[23][11] = 134;
        self.Big5PFreq[40][114] = 133;
        self.Big5PFreq[40][148] = 132;
        self.Big5PFreq[53][130] = 131;
        self.Big5PFreq[36][2] = 130;
        self.Big5PFreq[66][82] = 129;
        self.Big5PFreq[45][166] = 128;
        self.Big5PFreq[4][88] = 127;
        self.Big5PFreq[16][57] = 126;
        self.Big5PFreq[22][116] = 125;
        self.Big5PFreq[36][108] = 124;
        self.Big5PFreq[13][48] = 123;
        self.Big5PFreq[54][12] = 122;
        self.Big5PFreq[40][136] = 121;
        self.Big5PFreq[36][128] = 120;
        self.Big5PFreq[23][6] = 119;
        self.Big5PFreq[38][125] = 118;
        self.Big5PFreq[45][154] = 117;
        self.Big5PFreq[51][127] = 116;
        self.Big5PFreq[44][163] = 115;
        self.Big5PFreq[16][173] = 114;
        self.Big5PFreq[43][49] = 113;
        self.Big5PFreq[20][112] = 112;
        self.Big5PFreq[15][168] = 111;
        self.Big5PFreq[35][129] = 110;
        self.Big5PFreq[20][45] = 109;
        self.Big5PFreq[38][10] = 108;
        self.Big5PFreq[57][171] = 107;
        self.Big5PFreq[44][190] = 106;
        self.Big5PFreq[40][56] = 105;
        self.Big5PFreq[36][156] = 104;
        self.Big5PFreq[3][88] = 103;
        self.Big5PFreq[50][122] = 102;
        self.Big5PFreq[36][7] = 101;
        self.Big5PFreq[39][43] = 100;
        self.Big5PFreq[15][166] = 99;
        self.Big5PFreq[42][136] = 98;
        self.Big5PFreq[22][131] = 97;
        self.Big5PFreq[44][23] = 96;
        self.Big5PFreq[54][147] = 95;
        self.Big5PFreq[41][32] = 94;
        self.Big5PFreq[23][121] = 93;
        self.Big5PFreq[39][108] = 92;
        self.Big5PFreq[2][78] = 91;
        self.Big5PFreq[40][155] = 90;
        self.Big5PFreq[55][51] = 89;
        self.Big5PFreq[19][34] = 88;
        self.Big5PFreq[48][128] = 87;
        self.Big5PFreq[48][159] = 86;
        self.Big5PFreq[20][70] = 85;
        self.Big5PFreq[34][71] = 84;
        self.Big5PFreq[16][31] = 83;
        self.Big5PFreq[42][157] = 82;
        self.Big5PFreq[20][44] = 81;
        self.Big5PFreq[11][92] = 80;
        self.Big5PFreq[44][180] = 79;
        self.Big5PFreq[84][33] = 78;
        self.Big5PFreq[16][116] = 77;
        self.Big5PFreq[61][163] = 76;
        self.Big5PFreq[35][164] = 75;
        self.Big5PFreq[36][42] = 74;
        self.Big5PFreq[13][40] = 73;
        self.Big5PFreq[43][176] = 72;
        self.Big5PFreq[2][66] = 71;
        self.Big5PFreq[20][133] = 70;
        self.Big5PFreq[36][65] = 69;
        self.Big5PFreq[38][33] = 68;
        self.Big5PFreq[12][91] = 67;
        self.Big5PFreq[36][26] = 66;
        self.Big5PFreq[15][174] = 65;
        self.Big5PFreq[77][32] = 64;
        self.Big5PFreq[16][1] = 63;
        self.Big5PFreq[25][86] = 62;
        self.Big5PFreq[17][13] = 61;
        self.Big5PFreq[5][75] = 60;
        self.Big5PFreq[36][52] = 59;
        self.Big5PFreq[51][164] = 58;
        self.Big5PFreq[12][85] = 57;
        self.Big5PFreq[39][168] = 56;
        self.Big5PFreq[43][16] = 55;
        self.Big5PFreq[40][69] = 54;
        self.Big5PFreq[26][108] = 53;
        self.Big5PFreq[51][56] = 52;
        self.Big5PFreq[16][37] = 51;
        self.Big5PFreq[40][29] = 50;
        self.Big5PFreq[46][171] = 49;
        self.Big5PFreq[40][128] = 48;
        self.Big5PFreq[72][114] = 47;
        self.Big5PFreq[21][103] = 46;
        self.Big5PFreq[22][44] = 45;
        self.Big5PFreq[40][115] = 44;
        self.Big5PFreq[43][7] = 43;
        self.Big5PFreq[43][153] = 42;
        self.Big5PFreq[17][20] = 41;
        self.Big5PFreq[16][49] = 40;
        self.Big5PFreq[36][57] = 39;
        self.Big5PFreq[18][38] = 38;
        self.Big5PFreq[45][184] = 37;
        self.Big5PFreq[37][167] = 36;
        self.Big5PFreq[26][106] = 35;
        self.Big5PFreq[61][121] = 34;
        self.Big5PFreq[89][140] = 33;
        self.Big5PFreq[46][61] = 32;
        self.Big5PFreq[39][163] = 31;
        self.Big5PFreq[40][62] = 30;
        self.Big5PFreq[38][165] = 29;
        self.Big5PFreq[47][37] = 28;
        self.Big5PFreq[18][155] = 27;
        self.Big5PFreq[20][33] = 26;
        self.Big5PFreq[29][90] = 25;
        self.Big5PFreq[20][103] = 24;
        self.Big5PFreq[37][51] = 23;
        self.Big5PFreq[57][0] = 22;
        self.Big5PFreq[40][31] = 21;
        self.Big5PFreq[45][32] = 20;
        self.Big5PFreq[59][23] = 19;
        self.Big5PFreq[18][47] = 18;
        self.Big5PFreq[45][134] = 17;
        self.Big5PFreq[37][59] = 16;
        self.Big5PFreq[21][128] = 15;
        self.Big5PFreq[36][106] = 14;
        self.Big5PFreq[31][39] = 13;
        self.Big5PFreq[40][182] = 12;
        self.Big5PFreq[52][155] = 11;
        self.Big5PFreq[42][166] = 10;
        self.Big5PFreq[35][27] = 9;
        self.Big5PFreq[38][3] = 8;
        self.Big5PFreq[13][44] = 7;
        self.Big5PFreq[58][157] = 6;
        self.Big5PFreq[47][51] = 5;
        self.Big5PFreq[41][37] = 4;
        self.Big5PFreq[41][172] = 3;
        self.Big5PFreq[51][165] = 2;
        self.Big5PFreq[15][161] = 1;
        self.Big5PFreq[24][181] = 0;
        self.EUC_TWFreq[48][49] = 599;
        self.EUC_TWFreq[35][65] = 598;
        self.EUC_TWFreq[41][27] = 597;
        self.EUC_TWFreq[35][0] = 596;
        self.EUC_TWFreq[39][19] = 595;
        self.EUC_TWFreq[35][42] = 594;
        self.EUC_TWFreq[38][66] = 593;
        self.EUC_TWFreq[35][8] = 592;
        self.EUC_TWFreq[35][6] = 591;
        self.EUC_TWFreq[35][66] = 590;
        self.EUC_TWFreq[43][14] = 589;
        self.EUC_TWFreq[69][80] = 588;
        self.EUC_TWFreq[50][48] = 587;
        self.EUC_TWFreq[36][71] = 586;
        self.EUC_TWFreq[37][10] = 585;
        self.EUC_TWFreq[60][52] = 584;
        self.EUC_TWFreq[51][21] = 583;
        self.EUC_TWFreq[40][2] = 582;
        self.EUC_TWFreq[67][35] = 581;
        self.EUC_TWFreq[38][78] = 580;
        self.EUC_TWFreq[49][18] = 579;
        self.EUC_TWFreq[35][23] = 578;
        self.EUC_TWFreq[42][83] = 577;
        self.EUC_TWFreq[79][47] = 576;
        self.EUC_TWFreq[61][82] = 575;
        self.EUC_TWFreq[38][7] = 574;
        self.EUC_TWFreq[35][29] = 573;
        self.EUC_TWFreq[37][77] = 572;
        self.EUC_TWFreq[54][67] = 571;
        self.EUC_TWFreq[38][80] = 570;
        self.EUC_TWFreq[52][74] = 569;
        self.EUC_TWFreq[36][37] = 568;
        self.EUC_TWFreq[74][8] = 567;
        self.EUC_TWFreq[41][83] = 566;
        self.EUC_TWFreq[36][75] = 565;
        self.EUC_TWFreq[49][63] = 564;
        self.EUC_TWFreq[42][58] = 563;
        self.EUC_TWFreq[56][33] = 562;
        self.EUC_TWFreq[37][76] = 561;
        self.EUC_TWFreq[62][39] = 560;
        self.EUC_TWFreq[35][21] = 559;
        self.EUC_TWFreq[70][19] = 558;
        self.EUC_TWFreq[77][88] = 557;
        self.EUC_TWFreq[51][14] = 556;
        self.EUC_TWFreq[36][17] = 555;
        self.EUC_TWFreq[44][51] = 554;
        self.EUC_TWFreq[38][72] = 553;
        self.EUC_TWFreq[74][90] = 552;
        self.EUC_TWFreq[35][48] = 551;
        self.EUC_TWFreq[35][69] = 550;
        self.EUC_TWFreq[66][86] = 549;
        self.EUC_TWFreq[57][20] = 548;
        self.EUC_TWFreq[35][53] = 547;
        self.EUC_TWFreq[36][87] = 546;
        self.EUC_TWFreq[84][67] = 545;
        self.EUC_TWFreq[70][56] = 544;
        self.EUC_TWFreq[71][54] = 543;
        self.EUC_TWFreq[60][70] = 542;
        self.EUC_TWFreq[80][1] = 541;
        self.EUC_TWFreq[39][59] = 540;
        self.EUC_TWFreq[39][51] = 539;
        self.EUC_TWFreq[35][44] = 538;
        self.EUC_TWFreq[48][4] = 537;
        self.EUC_TWFreq[55][24] = 536;
        self.EUC_TWFreq[52][4] = 535;
        self.EUC_TWFreq[54][26] = 534;
        self.EUC_TWFreq[36][31] = 533;
        self.EUC_TWFreq[37][22] = 532;
        self.EUC_TWFreq[37][9] = 531;
        self.EUC_TWFreq[46][0] = 530;
        self.EUC_TWFreq[56][46] = 529;
        self.EUC_TWFreq[47][93] = 528;
        self.EUC_TWFreq[37][25] = 527;
        self.EUC_TWFreq[39][8] = 526;
        self.EUC_TWFreq[46][73] = 525;
        self.EUC_TWFreq[38][48] = 524;
        self.EUC_TWFreq[39][83] = 523;
        self.EUC_TWFreq[60][92] = 522;
        self.EUC_TWFreq[70][11] = 521;
        self.EUC_TWFreq[63][84] = 520;
        self.EUC_TWFreq[38][65] = 519;
        self.EUC_TWFreq[45][45] = 518;
        self.EUC_TWFreq[63][49] = 517;
        self.EUC_TWFreq[63][50] = 516;
        self.EUC_TWFreq[39][93] = 515;
        self.EUC_TWFreq[68][20] = 514;
        self.EUC_TWFreq[44][84] = 513;
        self.EUC_TWFreq[66][34] = 512;
        self.EUC_TWFreq[37][58] = 511;
        self.EUC_TWFreq[39][0] = 510;
        self.EUC_TWFreq[59][1] = 509;
        self.EUC_TWFreq[47][8] = 508;
        self.EUC_TWFreq[61][17] = 507;
        self.EUC_TWFreq[53][87] = 506;
        self.EUC_TWFreq[67][26] = 505;
        self.EUC_TWFreq[43][46] = 504;
        self.EUC_TWFreq[38][61] = 503;
        self.EUC_TWFreq[45][9] = 502;
        self.EUC_TWFreq[66][83] = 501;
        self.EUC_TWFreq[43][88] = 500;
        self.EUC_TWFreq[85][20] = 499;
        self.EUC_TWFreq[57][36] = 498;
        self.EUC_TWFreq[43][6] = 497;
        self.EUC_TWFreq[86][77] = 496;
        self.EUC_TWFreq[42][70] = 495;
        self.EUC_TWFreq[49][78] = 494;
        self.EUC_TWFreq[36][40] = 493;
        self.EUC_TWFreq[42][71] = 492;
        self.EUC_TWFreq[58][49] = 491;
        self.EUC_TWFreq[35][20] = 490;
        self.EUC_TWFreq[76][20] = 489;
        self.EUC_TWFreq[39][25] = 488;
        self.EUC_TWFreq[40][34] = 487;
        self.EUC_TWFreq[39][76] = 486;
        self.EUC_TWFreq[40][1] = 485;
        self.EUC_TWFreq[59][0] = 484;
        self.EUC_TWFreq[39][70] = 483;
        self.EUC_TWFreq[46][14] = 482;
        self.EUC_TWFreq[68][77] = 481;
        self.EUC_TWFreq[38][55] = 480;
        self.EUC_TWFreq[35][78] = 479;
        self.EUC_TWFreq[84][44] = 478;
        self.EUC_TWFreq[36][41] = 477;
        self.EUC_TWFreq[37][62] = 476;
        self.EUC_TWFreq[65][67] = 475;
        self.EUC_TWFreq[69][66] = 474;
        self.EUC_TWFreq[73][55] = 473;
        self.EUC_TWFreq[71][49] = 472;
        self.EUC_TWFreq[66][87] = 471;
        self.EUC_TWFreq[38][33] = 470;
        self.EUC_TWFreq[64][61] = 469;
        self.EUC_TWFreq[35][7] = 468;
        self.EUC_TWFreq[47][49] = 467;
        self.EUC_TWFreq[56][14] = 466;
        self.EUC_TWFreq[36][49] = 465;
        self.EUC_TWFreq[50][81] = 464;
        self.EUC_TWFreq[55][76] = 463;
        self.EUC_TWFreq[35][19] = 462;
        self.EUC_TWFreq[44][47] = 461;
        self.EUC_TWFreq[35][15] = 460;
        self.EUC_TWFreq[82][59] = 459;
        self.EUC_TWFreq[35][43] = 458;
        self.EUC_TWFreq[73][0] = 457;
        self.EUC_TWFreq[57][83] = 456;
        self.EUC_TWFreq[42][46] = 455;
        self.EUC_TWFreq[36][0] = 454;
        self.EUC_TWFreq[70][88] = 453;
        self.EUC_TWFreq[42][22] = 452;
        self.EUC_TWFreq[46][58] = 451;
        self.EUC_TWFreq[36][34] = 450;
        self.EUC_TWFreq[39][24] = 449;
        self.EUC_TWFreq[35][55] = 448;
        self.EUC_TWFreq[44][91] = 447;
        self.EUC_TWFreq[37][51] = 446;
        self.EUC_TWFreq[36][19] = 445;
        self.EUC_TWFreq[69][90] = 444;
        self.EUC_TWFreq[55][35] = 443;
        self.EUC_TWFreq[35][54] = 442;
        self.EUC_TWFreq[49][61] = 441;
        self.EUC_TWFreq[36][67] = 440;
        self.EUC_TWFreq[88][34] = 439;
        self.EUC_TWFreq[35][17] = 438;
        self.EUC_TWFreq[65][69] = 437;
        self.EUC_TWFreq[74][89] = 436;
        self.EUC_TWFreq[37][31] = 435;
        self.EUC_TWFreq[43][48] = 434;
        self.EUC_TWFreq[89][27] = 433;
        self.EUC_TWFreq[42][79] = 432;
        self.EUC_TWFreq[69][57] = 431;
        self.EUC_TWFreq[36][13] = 430;
        self.EUC_TWFreq[35][62] = 429;
        self.EUC_TWFreq[65][47] = 428;
        self.EUC_TWFreq[56][8] = 427;
        self.EUC_TWFreq[38][79] = 426;
        self.EUC_TWFreq[37][64] = 425;
        self.EUC_TWFreq[64][64] = 424;
        self.EUC_TWFreq[38][53] = 423;
        self.EUC_TWFreq[38][31] = 422;
        self.EUC_TWFreq[56][81] = 421;
        self.EUC_TWFreq[36][22] = 420;
        self.EUC_TWFreq[43][4] = 419;
        self.EUC_TWFreq[36][90] = 418;
        self.EUC_TWFreq[38][62] = 417;
        self.EUC_TWFreq[66][85] = 416;
        self.EUC_TWFreq[39][1] = 415;
        self.EUC_TWFreq[59][40] = 414;
        self.EUC_TWFreq[58][93] = 413;
        self.EUC_TWFreq[44][43] = 412;
        self.EUC_TWFreq[39][49] = 411;
        self.EUC_TWFreq[64][2] = 410;
        self.EUC_TWFreq[41][35] = 409;
        self.EUC_TWFreq[60][22] = 408;
        self.EUC_TWFreq[35][91] = 407;
        self.EUC_TWFreq[78][1] = 406;
        self.EUC_TWFreq[36][14] = 405;
        self.EUC_TWFreq[82][29] = 404;
        self.EUC_TWFreq[52][86] = 403;
        self.EUC_TWFreq[40][16] = 402;
        self.EUC_TWFreq[91][52] = 401;
        self.EUC_TWFreq[50][75] = 400;
        self.EUC_TWFreq[64][30] = 399;
        self.EUC_TWFreq[90][78] = 398;
        self.EUC_TWFreq[36][52] = 397;
        self.EUC_TWFreq[55][87] = 396;
        self.EUC_TWFreq[57][5] = 395;
        self.EUC_TWFreq[57][31] = 394;
        self.EUC_TWFreq[42][35] = 393;
        self.EUC_TWFreq[69][50] = 392;
        self.EUC_TWFreq[45][8] = 391;
        self.EUC_TWFreq[50][87] = 390;
        self.EUC_TWFreq[69][55] = 389;
        self.EUC_TWFreq[92][3] = 388;
        self.EUC_TWFreq[36][43] = 387;
        self.EUC_TWFreq[64][10] = 386;
        self.EUC_TWFreq[56][25] = 385;
        self.EUC_TWFreq[60][68] = 384;
        self.EUC_TWFreq[51][46] = 383;
        self.EUC_TWFreq[50][0] = 382;
        self.EUC_TWFreq[38][30] = 381;
        self.EUC_TWFreq[50][85] = 380;
        self.EUC_TWFreq[60][54] = 379;
        self.EUC_TWFreq[73][6] = 378;
        self.EUC_TWFreq[73][28] = 377;
        self.EUC_TWFreq[56][19] = 376;
        self.EUC_TWFreq[62][69] = 375;
        self.EUC_TWFreq[81][66] = 374;
        self.EUC_TWFreq[40][32] = 373;
        self.EUC_TWFreq[76][31] = 372;
        self.EUC_TWFreq[35][10] = 371;
        self.EUC_TWFreq[41][37] = 370;
        self.EUC_TWFreq[52][82] = 369;
        self.EUC_TWFreq[91][72] = 368;
        self.EUC_TWFreq[37][29] = 367;
        self.EUC_TWFreq[56][30] = 366;
        self.EUC_TWFreq[37][80] = 365;
        self.EUC_TWFreq[81][56] = 364;
        self.EUC_TWFreq[70][3] = 363;
        self.EUC_TWFreq[76][15] = 362;
        self.EUC_TWFreq[46][47] = 361;
        self.EUC_TWFreq[35][88] = 360;
        self.EUC_TWFreq[61][58] = 359;
        self.EUC_TWFreq[37][37] = 358;
        self.EUC_TWFreq[57][22] = 357;
        self.EUC_TWFreq[41][23] = 356;
        self.EUC_TWFreq[90][66] = 355;
        self.EUC_TWFreq[39][60] = 354;
        self.EUC_TWFreq[38][0] = 353;
        self.EUC_TWFreq[37][87] = 352;
        self.EUC_TWFreq[46][2] = 351;
        self.EUC_TWFreq[38][56] = 350;
        self.EUC_TWFreq[58][11] = 349;
        self.EUC_TWFreq[48][10] = 348;
        self.EUC_TWFreq[74][4] = 347;
        self.EUC_TWFreq[40][42] = 346;
        self.EUC_TWFreq[41][52] = 345;
        self.EUC_TWFreq[61][92] = 344;
        self.EUC_TWFreq[39][50] = 343;
        self.EUC_TWFreq[47][88] = 342;
        self.EUC_TWFreq[88][36] = 341;
        self.EUC_TWFreq[45][73] = 340;
        self.EUC_TWFreq[82][3] = 339;
        self.EUC_TWFreq[61][36] = 338;
        self.EUC_TWFreq[60][33] = 337;
        self.EUC_TWFreq[38][27] = 336;
        self.EUC_TWFreq[35][83] = 335;
        self.EUC_TWFreq[65][24] = 334;
        self.EUC_TWFreq[73][10] = 333;
        self.EUC_TWFreq[41][13] = 332;
        self.EUC_TWFreq[50][27] = 331;
        self.EUC_TWFreq[59][50] = 330;
        self.EUC_TWFreq[42][45] = 329;
        self.EUC_TWFreq[55][19] = 328;
        self.EUC_TWFreq[36][77] = 327;
        self.EUC_TWFreq[69][31] = 326;
        self.EUC_TWFreq[60][7] = 325;
        self.EUC_TWFreq[40][88] = 324;
        self.EUC_TWFreq[57][56] = 323;
        self.EUC_TWFreq[50][50] = 322;
        self.EUC_TWFreq[42][37] = 321;
        self.EUC_TWFreq[38][82] = 320;
        self.EUC_TWFreq[52][25] = 319;
        self.EUC_TWFreq[42][67] = 318;
        self.EUC_TWFreq[48][40] = 317;
        self.EUC_TWFreq[45][81] = 316;
        self.EUC_TWFreq[57][14] = 315;
        self.EUC_TWFreq[42][13] = 314;
        self.EUC_TWFreq[78][0] = 313;
        self.EUC_TWFreq[35][51] = 312;
        self.EUC_TWFreq[41][67] = 311;
        self.EUC_TWFreq[64][23] = 310;
        self.EUC_TWFreq[36][65] = 309;
        self.EUC_TWFreq[48][50] = 308;
        self.EUC_TWFreq[46][69] = 307;
        self.EUC_TWFreq[47][89] = 306;
        self.EUC_TWFreq[41][48] = 305;
        self.EUC_TWFreq[60][56] = 304;
        self.EUC_TWFreq[44][82] = 303;
        self.EUC_TWFreq[47][35] = 302;
        self.EUC_TWFreq[49][3] = 301;
        self.EUC_TWFreq[49][69] = 300;
        self.EUC_TWFreq[45][93] = 299;
        self.EUC_TWFreq[60][34] = 298;
        self.EUC_TWFreq[60][82] = 297;
        self.EUC_TWFreq[61][61] = 296;
        self.EUC_TWFreq[86][42] = 295;
        self.EUC_TWFreq[89][60] = 294;
        self.EUC_TWFreq[48][31] = 293;
        self.EUC_TWFreq[35][75] = 292;
        self.EUC_TWFreq[91][39] = 291;
        self.EUC_TWFreq[53][19] = 290;
        self.EUC_TWFreq[39][72] = 289;
        self.EUC_TWFreq[69][59] = 288;
        self.EUC_TWFreq[41][7] = 287;
        self.EUC_TWFreq[54][13] = 286;
        self.EUC_TWFreq[43][28] = 285;
        self.EUC_TWFreq[36][6] = 284;
        self.EUC_TWFreq[45][75] = 283;
        self.EUC_TWFreq[36][61] = 282;
        self.EUC_TWFreq[38][21] = 281;
        self.EUC_TWFreq[45][14] = 280;
        self.EUC_TWFreq[61][43] = 279;
        self.EUC_TWFreq[36][63] = 278;
        self.EUC_TWFreq[43][30] = 277;
        self.EUC_TWFreq[46][51] = 276;
        self.EUC_TWFreq[68][87] = 275;
        self.EUC_TWFreq[39][26] = 274;
        self.EUC_TWFreq[46][76] = 273;
        self.EUC_TWFreq[36][15] = 272;
        self.EUC_TWFreq[35][40] = 271;
        self.EUC_TWFreq[79][60] = 270;
        self.EUC_TWFreq[46][7] = 269;
        self.EUC_TWFreq[65][72] = 268;
        self.EUC_TWFreq[69][88] = 267;
        self.EUC_TWFreq[47][18] = 266;
        self.EUC_TWFreq[37][0] = 265;
        self.EUC_TWFreq[37][49] = 264;
        self.EUC_TWFreq[67][37] = 263;
        self.EUC_TWFreq[36][91] = 262;
        self.EUC_TWFreq[75][48] = 261;
        self.EUC_TWFreq[75][63] = 260;
        self.EUC_TWFreq[83][87] = 259;
        self.EUC_TWFreq[37][44] = 258;
        self.EUC_TWFreq[73][54] = 257;
        self.EUC_TWFreq[51][61] = 256;
        self.EUC_TWFreq[46][57] = 255;
        self.EUC_TWFreq[55][21] = 254;
        self.EUC_TWFreq[39][66] = 253;
        self.EUC_TWFreq[47][11] = 252;
        self.EUC_TWFreq[52][8] = 251;
        self.EUC_TWFreq[82][81] = 250;
        self.EUC_TWFreq[36][57] = 249;
        self.EUC_TWFreq[38][54] = 248;
        self.EUC_TWFreq[43][81] = 247;
        self.EUC_TWFreq[37][42] = 246;
        self.EUC_TWFreq[40][18] = 245;
        self.EUC_TWFreq[80][90] = 244;
        self.EUC_TWFreq[37][84] = 243;
        self.EUC_TWFreq[57][15] = 242;
        self.EUC_TWFreq[38][87] = 241;
        self.EUC_TWFreq[37][32] = 240;
        self.EUC_TWFreq[53][53] = 239;
        self.EUC_TWFreq[89][29] = 238;
        self.EUC_TWFreq[81][53] = 237;
        self.EUC_TWFreq[75][3] = 236;
        self.EUC_TWFreq[83][73] = 235;
        self.EUC_TWFreq[66][13] = 234;
        self.EUC_TWFreq[48][7] = 233;
        self.EUC_TWFreq[46][35] = 232;
        self.EUC_TWFreq[35][86] = 231;
        self.EUC_TWFreq[37][20] = 230;
        self.EUC_TWFreq[46][80] = 229;
        self.EUC_TWFreq[38][24] = 228;
        self.EUC_TWFreq[41][68] = 227;
        self.EUC_TWFreq[42][21] = 226;
        self.EUC_TWFreq[43][32] = 225;
        self.EUC_TWFreq[38][20] = 224;
        self.EUC_TWFreq[37][59] = 223;
        self.EUC_TWFreq[41][77] = 222;
        self.EUC_TWFreq[59][57] = 221;
        self.EUC_TWFreq[68][59] = 220;
        self.EUC_TWFreq[39][43] = 219;
        self.EUC_TWFreq[54][39] = 218;
        self.EUC_TWFreq[48][28] = 217;
        self.EUC_TWFreq[54][28] = 216;
        self.EUC_TWFreq[41][44] = 215;
        self.EUC_TWFreq[51][64] = 214;
        self.EUC_TWFreq[47][72] = 213;
        self.EUC_TWFreq[62][67] = 212;
        self.EUC_TWFreq[42][43] = 211;
        self.EUC_TWFreq[61][38] = 210;
        self.EUC_TWFreq[76][25] = 209;
        self.EUC_TWFreq[48][91] = 208;
        self.EUC_TWFreq[36][36] = 207;
        self.EUC_TWFreq[80][32] = 206;
        self.EUC_TWFreq[81][40] = 205;
        self.EUC_TWFreq[37][5] = 204;
        self.EUC_TWFreq[74][69] = 203;
        self.EUC_TWFreq[36][82] = 202;
        self.EUC_TWFreq[46][59] = 201;
        /*
         * EUC_TWFreq[38][32] = 200; EUC_TWFreq[74][2] = 199; EUC_TWFreq[53][31]
         * = 198; EUC_TWFreq[35][38] = 197; EUC_TWFreq[46][62] = 196;
         * EUC_TWFreq[77][31] = 195; EUC_TWFreq[55][74] = 194; EUC_TWFreq[66][6]
         * = 193; EUC_TWFreq[56][21] = 192; EUC_TWFreq[54][78] = 191;
         * EUC_TWFreq[43][51] = 190; EUC_TWFreq[64][93] = 189; EUC_TWFreq[92][7]
         * = 188; EUC_TWFreq[83][89] = 187; EUC_TWFreq[69][9] = 186;
         * EUC_TWFreq[45][4] = 185; EUC_TWFreq[53][9] = 184; EUC_TWFreq[43][2] =
         * 183; EUC_TWFreq[35][11] = 182; EUC_TWFreq[51][25] = 181;
         * EUC_TWFreq[52][71] = 180; EUC_TWFreq[81][67] = 179;
         * EUC_TWFreq[37][33] = 178; EUC_TWFreq[38][57] = 177;
         * EUC_TWFreq[39][77] = 176; EUC_TWFreq[40][26] = 175;
         * EUC_TWFreq[37][21] = 174; EUC_TWFreq[81][70] = 173;
         * EUC_TWFreq[56][80] = 172; EUC_TWFreq[65][14] = 171;
         * EUC_TWFreq[62][47] = 170; EUC_TWFreq[56][54] = 169;
         * EUC_TWFreq[45][17] = 168; EUC_TWFreq[52][52] = 167;
         * EUC_TWFreq[74][30] = 166; EUC_TWFreq[60][57] = 165;
         * EUC_TWFreq[41][15] = 164; EUC_TWFreq[47][69] = 163;
         * EUC_TWFreq[61][11] = 162; EUC_TWFreq[72][25] = 161;
         * EUC_TWFreq[82][56] = 160; EUC_TWFreq[76][92] = 159;
         * EUC_TWFreq[51][22] = 158; EUC_TWFreq[55][69] = 157;
         * EUC_TWFreq[49][43] = 156; EUC_TWFreq[69][49] = 155;
         * EUC_TWFreq[88][42] = 154; EUC_TWFreq[84][41] = 153;
         * EUC_TWFreq[79][33] = 152; EUC_TWFreq[47][17] = 151;
         * EUC_TWFreq[52][88] = 150; EUC_TWFreq[63][74] = 149;
         * EUC_TWFreq[50][32] = 148; EUC_TWFreq[65][10] = 147; EUC_TWFreq[57][6]
         * = 146; EUC_TWFreq[52][23] = 145; EUC_TWFreq[36][70] = 144;
         * EUC_TWFreq[65][55] = 143; EUC_TWFreq[35][27] = 142;
         * EUC_TWFreq[57][63] = 141; EUC_TWFreq[39][92] = 140;
         * EUC_TWFreq[79][75] = 139; EUC_TWFreq[36][30] = 138;
         * EUC_TWFreq[53][60] = 137; EUC_TWFreq[55][43] = 136;
         * EUC_TWFreq[71][22] = 135; EUC_TWFreq[43][16] = 134;
         * EUC_TWFreq[65][21] = 133; EUC_TWFreq[84][51] = 132;
         * EUC_TWFreq[43][64] = 131; EUC_TWFreq[87][91] = 130;
         * EUC_TWFreq[47][45] = 129; EUC_TWFreq[65][29] = 128;
         * EUC_TWFreq[88][16] = 127; EUC_TWFreq[50][5] = 126; EUC_TWFreq[47][33]
         * = 125; EUC_TWFreq[46][27] = 124; EUC_TWFreq[85][2] = 123;
         * EUC_TWFreq[43][77] = 122; EUC_TWFreq[70][9] = 121; EUC_TWFreq[41][54]
         * = 120; EUC_TWFreq[56][12] = 119; EUC_TWFreq[90][65] = 118;
         * EUC_TWFreq[91][50] = 117; EUC_TWFreq[48][41] = 116;
         * EUC_TWFreq[35][89] = 115; EUC_TWFreq[90][83] = 114;
         * EUC_TWFreq[44][40] = 113; EUC_TWFreq[50][88] = 112;
         * EUC_TWFreq[72][39] = 111; EUC_TWFreq[45][3] = 110; EUC_TWFreq[71][33]
         * = 109; EUC_TWFreq[39][12] = 108; EUC_TWFreq[59][24] = 107;
         * EUC_TWFreq[60][62] = 106; EUC_TWFreq[44][33] = 105;
         * EUC_TWFreq[53][70] = 104; EUC_TWFreq[77][90] = 103;
         * EUC_TWFreq[50][58] = 102; EUC_TWFreq[54][1] = 101; EUC_TWFreq[73][19]
         * = 100; EUC_TWFreq[37][3] = 99; EUC_TWFreq[49][91] = 98;
         * EUC_TWFreq[88][43] = 97; EUC_TWFreq[36][78] = 96; EUC_TWFreq[44][20]
         * = 95; EUC_TWFreq[64][15] = 94; EUC_TWFreq[72][28] = 93;
         * EUC_TWFreq[70][13] = 92; EUC_TWFreq[65][83] = 91; EUC_TWFreq[58][68]
         * = 90; EUC_TWFreq[59][32] = 89; EUC_TWFreq[39][13] = 88;
         * EUC_TWFreq[55][64] = 87; EUC_TWFreq[56][59] = 86; EUC_TWFreq[39][17]
         * = 85; EUC_TWFreq[55][84] = 84; EUC_TWFreq[77][85] = 83;
         * EUC_TWFreq[60][19] = 82; EUC_TWFreq[62][82] = 81; EUC_TWFreq[78][16]
         * = 80; EUC_TWFreq[66][8] = 79; EUC_TWFreq[39][42] = 78;
         * EUC_TWFreq[61][24] = 77; EUC_TWFreq[57][67] = 76; EUC_TWFreq[38][83]
         * = 75; EUC_TWFreq[36][53] = 74; EUC_TWFreq[67][76] = 73;
         * EUC_TWFreq[37][91] = 72; EUC_TWFreq[44][26] = 71; EUC_TWFreq[72][86]
         * = 70; EUC_TWFreq[44][87] = 69; EUC_TWFreq[45][50] = 68;
         * EUC_TWFreq[58][4] = 67; EUC_TWFreq[86][65] = 66; EUC_TWFreq[45][56] =
         * 65; EUC_TWFreq[79][49] = 64; EUC_TWFreq[35][3] = 63;
         * EUC_TWFreq[48][83] = 62; EUC_TWFreq[71][21] = 61; EUC_TWFreq[77][93]
         * = 60; EUC_TWFreq[87][92] = 59; EUC_TWFreq[38][35] = 58;
         * EUC_TWFreq[66][17] = 57; EUC_TWFreq[37][66] = 56; EUC_TWFreq[51][42]
         * = 55; EUC_TWFreq[57][73] = 54; EUC_TWFreq[51][54] = 53;
         * EUC_TWFreq[75][64] = 52; EUC_TWFreq[35][5] = 51; EUC_TWFreq[49][40] =
         * 50; EUC_TWFreq[58][35] = 49; EUC_TWFreq[67][88] = 48;
         * EUC_TWFreq[60][51] = 47; EUC_TWFreq[36][92] = 46; EUC_TWFreq[44][41]
         * = 45; EUC_TWFreq[58][29] = 44; EUC_TWFreq[43][62] = 43;
         * EUC_TWFreq[56][23] = 42; EUC_TWFreq[67][44] = 41; EUC_TWFreq[52][91]
         * = 40; EUC_TWFreq[42][81] = 39; EUC_TWFreq[64][25] = 38;
         * EUC_TWFreq[35][36] = 37; EUC_TWFreq[47][73] = 36; EUC_TWFreq[36][1] =
         * 35; EUC_TWFreq[65][84] = 34; EUC_TWFreq[73][1] = 33;
         * EUC_TWFreq[79][66] = 32; EUC_TWFreq[69][14] = 31; EUC_TWFreq[65][28]
         * = 30; EUC_TWFreq[60][93] = 29; EUC_TWFreq[72][79] = 28;
         * EUC_TWFreq[48][0] = 27; EUC_TWFreq[73][43] = 26; EUC_TWFreq[66][47] =
         * 25; EUC_TWFreq[41][18] = 24; EUC_TWFreq[51][10] = 23;
         * EUC_TWFreq[59][7] = 22; EUC_TWFreq[53][27] = 21; EUC_TWFreq[86][67] =
         * 20; EUC_TWFreq[49][87] = 19; EUC_TWFreq[52][28] = 18;
         * EUC_TWFreq[52][12] = 17; EUC_TWFreq[42][30] = 16; EUC_TWFreq[65][35]
         * = 15; EUC_TWFreq[46][64] = 14; EUC_TWFreq[71][7] = 13;
         * EUC_TWFreq[56][57] = 12; EUC_TWFreq[56][31] = 11; EUC_TWFreq[41][31]
         * = 10; EUC_TWFreq[48][59] = 9; EUC_TWFreq[63][92] = 8;
         * EUC_TWFreq[62][57] = 7; EUC_TWFreq[65][87] = 6; EUC_TWFreq[70][10] =
         * 5; EUC_TWFreq[52][40] = 4; EUC_TWFreq[40][22] = 3; EUC_TWFreq[65][91]
         * = 2; EUC_TWFreq[50][25] = 1; EUC_TWFreq[35][84] = 0;
         */
        self.GBKFreq[52][132] = 600;
        self.GBKFreq[73][135] = 599;
        self.GBKFreq[49][123] = 598;
        self.GBKFreq[77][146] = 597;
        self.GBKFreq[81][123] = 596;
        self.GBKFreq[82][144] = 595;
        self.GBKFreq[51][179] = 594;
        self.GBKFreq[83][154] = 593;
        self.GBKFreq[71][139] = 592;
        self.GBKFreq[64][139] = 591;
        self.GBKFreq[85][144] = 590;
        self.GBKFreq[52][125] = 589;
        self.GBKFreq[88][25] = 588;
        self.GBKFreq[81][106] = 587;
        self.GBKFreq[81][148] = 586;
        self.GBKFreq[62][137] = 585;
        self.GBKFreq[94][0] = 584;
        self.GBKFreq[1][64] = 583;
        self.GBKFreq[67][163] = 582;
        self.GBKFreq[20][190] = 581;
        self.GBKFreq[57][131] = 580;
        self.GBKFreq[29][169] = 579;
        self.GBKFreq[72][143] = 578;
        self.GBKFreq[0][173] = 577;
        self.GBKFreq[11][23] = 576;
        self.GBKFreq[61][141] = 575;
        self.GBKFreq[60][123] = 574;
        self.GBKFreq[81][114] = 573;
        self.GBKFreq[82][131] = 572;
        self.GBKFreq[67][156] = 571;
        self.GBKFreq[71][167] = 570;
        self.GBKFreq[20][50] = 569;
        self.GBKFreq[77][132] = 568;
        self.GBKFreq[84][38] = 567;
        self.GBKFreq[26][29] = 566;
        self.GBKFreq[74][187] = 565;
        self.GBKFreq[62][116] = 564;
        self.GBKFreq[67][135] = 563;
        self.GBKFreq[5][86] = 562;
        self.GBKFreq[72][186] = 561;
        self.GBKFreq[75][161] = 560;
        self.GBKFreq[78][130] = 559;
        self.GBKFreq[94][30] = 558;
        self.GBKFreq[84][72] = 557;
        self.GBKFreq[1][67] = 556;
        self.GBKFreq[75][172] = 555;
        self.GBKFreq[74][185] = 554;
        self.GBKFreq[53][160] = 553;
        self.GBKFreq[123][14] = 552;
        self.GBKFreq[79][97] = 551;
        self.GBKFreq[85][110] = 550;
        self.GBKFreq[78][171] = 549;
        self.GBKFreq[52][131] = 548;
        self.GBKFreq[56][100] = 547;
        self.GBKFreq[50][182] = 546;
        self.GBKFreq[94][64] = 545;
        self.GBKFreq[106][74] = 544;
        self.GBKFreq[11][102] = 543;
        self.GBKFreq[53][124] = 542;
        self.GBKFreq[24][3] = 541;
        self.GBKFreq[86][148] = 540;
        self.GBKFreq[53][184] = 539;
        self.GBKFreq[86][147] = 538;
        self.GBKFreq[96][161] = 537;
        self.GBKFreq[82][77] = 536;
        self.GBKFreq[59][146] = 535;
        self.GBKFreq[84][126] = 534;
        self.GBKFreq[79][132] = 533;
        self.GBKFreq[85][123] = 532;
        self.GBKFreq[71][101] = 531;
        self.GBKFreq[85][106] = 530;
        self.GBKFreq[6][184] = 529;
        self.GBKFreq[57][156] = 528;
        self.GBKFreq[75][104] = 527;
        self.GBKFreq[50][137] = 526;
        self.GBKFreq[79][133] = 525;
        self.GBKFreq[76][108] = 524;
        self.GBKFreq[57][142] = 523;
        self.GBKFreq[84][130] = 522;
        self.GBKFreq[52][128] = 521;
        self.GBKFreq[47][44] = 520;
        self.GBKFreq[52][152] = 519;
        self.GBKFreq[54][104] = 518;
        self.GBKFreq[30][47] = 517;
        self.GBKFreq[71][123] = 516;
        self.GBKFreq[52][107] = 515;
        self.GBKFreq[45][84] = 514;
        self.GBKFreq[107][118] = 513;
        self.GBKFreq[5][161] = 512;
        self.GBKFreq[48][126] = 511;
        self.GBKFreq[67][170] = 510;
        self.GBKFreq[43][6] = 509;
        self.GBKFreq[70][112] = 508;
        self.GBKFreq[86][174] = 507;
        self.GBKFreq[84][166] = 506;
        self.GBKFreq[79][130] = 505;
        self.GBKFreq[57][141] = 504;
        self.GBKFreq[81][178] = 503;
        self.GBKFreq[56][187] = 502;
        self.GBKFreq[81][162] = 501;
        self.GBKFreq[53][104] = 500;
        self.GBKFreq[123][35] = 499;
        self.GBKFreq[70][169] = 498;
        self.GBKFreq[69][164] = 497;
        self.GBKFreq[109][61] = 496;
        self.GBKFreq[73][130] = 495;
        self.GBKFreq[62][134] = 494;
        self.GBKFreq[54][125] = 493;
        self.GBKFreq[79][105] = 492;
        self.GBKFreq[70][165] = 491;
        self.GBKFreq[71][189] = 490;
        self.GBKFreq[23][147] = 489;
        self.GBKFreq[51][139] = 488;
        self.GBKFreq[47][137] = 487;
        self.GBKFreq[77][123] = 486;
        self.GBKFreq[86][183] = 485;
        self.GBKFreq[63][173] = 484;
        self.GBKFreq[79][144] = 483;
        self.GBKFreq[84][159] = 482;
        self.GBKFreq[60][91] = 481;
        self.GBKFreq[66][187] = 480;
        self.GBKFreq[73][114] = 479;
        self.GBKFreq[85][56] = 478;
        self.GBKFreq[71][149] = 477;
        self.GBKFreq[84][189] = 476;
        self.GBKFreq[104][31] = 475;
        self.GBKFreq[83][82] = 474;
        self.GBKFreq[68][35] = 473;
        self.GBKFreq[11][77] = 472;
        self.GBKFreq[15][155] = 471;
        self.GBKFreq[83][153] = 470;
        self.GBKFreq[71][1] = 469;
        self.GBKFreq[53][190] = 468;
        self.GBKFreq[50][135] = 467;
        self.GBKFreq[3][147] = 466;
        self.GBKFreq[48][136] = 465;
        self.GBKFreq[66][166] = 464;
        self.GBKFreq[55][159] = 463;
        self.GBKFreq[82][150] = 462;
        self.GBKFreq[58][178] = 461;
        self.GBKFreq[64][102] = 460;
        self.GBKFreq[16][106] = 459;
        self.GBKFreq[68][110] = 458;
        self.GBKFreq[54][14] = 457;
        self.GBKFreq[60][140] = 456;
        self.GBKFreq[91][71] = 455;
        self.GBKFreq[54][150] = 454;
        self.GBKFreq[78][177] = 453;
        self.GBKFreq[78][117] = 452;
        self.GBKFreq[104][12] = 451;
        self.GBKFreq[73][150] = 450;
        self.GBKFreq[51][142] = 449;
        self.GBKFreq[81][145] = 448;
        self.GBKFreq[66][183] = 447;
        self.GBKFreq[51][178] = 446;
        self.GBKFreq[75][107] = 445;
        self.GBKFreq[65][119] = 444;
        self.GBKFreq[69][176] = 443;
        self.GBKFreq[59][122] = 442;
        self.GBKFreq[78][160] = 441;
        self.GBKFreq[85][183] = 440;
        self.GBKFreq[105][16] = 439;
        self.GBKFreq[73][110] = 438;
        self.GBKFreq[104][39] = 437;
        self.GBKFreq[119][16] = 436;
        self.GBKFreq[76][162] = 435;
        self.GBKFreq[67][152] = 434;
        self.GBKFreq[82][24] = 433;
        self.GBKFreq[73][121] = 432;
        self.GBKFreq[83][83] = 431;
        self.GBKFreq[82][145] = 430;
        self.GBKFreq[49][133] = 429;
        self.GBKFreq[94][13] = 428;
        self.GBKFreq[58][139] = 427;
        self.GBKFreq[74][189] = 426;
        self.GBKFreq[66][177] = 425;
        self.GBKFreq[85][184] = 424;
        self.GBKFreq[55][183] = 423;
        self.GBKFreq[71][107] = 422;
        self.GBKFreq[11][98] = 421;
        self.GBKFreq[72][153] = 420;
        self.GBKFreq[2][137] = 419;
        self.GBKFreq[59][147] = 418;
        self.GBKFreq[58][152] = 417;
        self.GBKFreq[55][144] = 416;
        self.GBKFreq[73][125] = 415;
        self.GBKFreq[52][154] = 414;
        self.GBKFreq[70][178] = 413;
        self.GBKFreq[79][148] = 412;
        self.GBKFreq[63][143] = 411;
        self.GBKFreq[50][140] = 410;
        self.GBKFreq[47][145] = 409;
        self.GBKFreq[48][123] = 408;
        self.GBKFreq[56][107] = 407;
        self.GBKFreq[84][83] = 406;
        self.GBKFreq[59][112] = 405;
        self.GBKFreq[124][72] = 404;
        self.GBKFreq[79][99] = 403;
        self.GBKFreq[3][37] = 402;
        self.GBKFreq[114][55] = 401;
        self.GBKFreq[85][152] = 400;
        self.GBKFreq[60][47] = 399;
        self.GBKFreq[65][96] = 398;
        self.GBKFreq[74][110] = 397;
        self.GBKFreq[86][182] = 396;
        self.GBKFreq[50][99] = 395;
        self.GBKFreq[67][186] = 394;
        self.GBKFreq[81][74] = 393;
        self.GBKFreq[80][37] = 392;
        self.GBKFreq[21][60] = 391;
        self.GBKFreq[110][12] = 390;
        self.GBKFreq[60][162] = 389;
        self.GBKFreq[29][115] = 388;
        self.GBKFreq[83][130] = 387;
        self.GBKFreq[52][136] = 386;
        self.GBKFreq[63][114] = 385;
        self.GBKFreq[49][127] = 384;
        self.GBKFreq[83][109] = 383;
        self.GBKFreq[66][128] = 382;
        self.GBKFreq[78][136] = 381;
        self.GBKFreq[81][180] = 380;
        self.GBKFreq[76][104] = 379;
        self.GBKFreq[56][156] = 378;
        self.GBKFreq[61][23] = 377;
        self.GBKFreq[4][30] = 376;
        self.GBKFreq[69][154] = 375;
        self.GBKFreq[100][37] = 374;
        self.GBKFreq[54][177] = 373;
        self.GBKFreq[23][119] = 372;
        self.GBKFreq[71][171] = 371;
        self.GBKFreq[84][146] = 370;
        self.GBKFreq[20][184] = 369;
        self.GBKFreq[86][76] = 368;
        self.GBKFreq[74][132] = 367;
        self.GBKFreq[47][97] = 366;
        self.GBKFreq[82][137] = 365;
        self.GBKFreq[94][56] = 364;
        self.GBKFreq[92][30] = 363;
        self.GBKFreq[19][117] = 362;
        self.GBKFreq[48][173] = 361;
        self.GBKFreq[2][136] = 360;
        self.GBKFreq[7][182] = 359;
        self.GBKFreq[74][188] = 358;
        self.GBKFreq[14][132] = 357;
        self.GBKFreq[62][172] = 356;
        self.GBKFreq[25][39] = 355;
        self.GBKFreq[85][129] = 354;
        self.GBKFreq[64][98] = 353;
        self.GBKFreq[67][127] = 352;
        self.GBKFreq[72][167] = 351;
        self.GBKFreq[57][143] = 350;
        self.GBKFreq[76][187] = 349;
        self.GBKFreq[83][181] = 348;
        self.GBKFreq[84][10] = 347;
        self.GBKFreq[55][166] = 346;
        self.GBKFreq[55][188] = 345;
        self.GBKFreq[13][151] = 344;
        self.GBKFreq[62][124] = 343;
        self.GBKFreq[53][136] = 342;
        self.GBKFreq[106][57] = 341;
        self.GBKFreq[47][166] = 340;
        self.GBKFreq[109][30] = 339;
        self.GBKFreq[78][114] = 338;
        self.GBKFreq[83][19] = 337;
        self.GBKFreq[56][162] = 336;
        self.GBKFreq[60][177] = 335;
        self.GBKFreq[88][9] = 334;
        self.GBKFreq[74][163] = 333;
        self.GBKFreq[52][156] = 332;
        self.GBKFreq[71][180] = 331;
        self.GBKFreq[60][57] = 330;
        self.GBKFreq[72][173] = 329;
        self.GBKFreq[82][91] = 328;
        self.GBKFreq[51][186] = 327;
        self.GBKFreq[75][86] = 326;
        self.GBKFreq[75][78] = 325;
        self.GBKFreq[76][170] = 324;
        self.GBKFreq[60][147] = 323;
        self.GBKFreq[82][75] = 322;
        self.GBKFreq[80][148] = 321;
        self.GBKFreq[86][150] = 320;
        self.GBKFreq[13][95] = 319;
        self.GBKFreq[0][11] = 318;
        self.GBKFreq[84][190] = 317;
        self.GBKFreq[76][166] = 316;
        self.GBKFreq[14][72] = 315;
        self.GBKFreq[67][144] = 314;
        self.GBKFreq[84][44] = 313;
        self.GBKFreq[72][125] = 312;
        self.GBKFreq[66][127] = 311;
        self.GBKFreq[60][25] = 310;
        self.GBKFreq[70][146] = 309;
        self.GBKFreq[79][135] = 308;
        self.GBKFreq[54][135] = 307;
        self.GBKFreq[60][104] = 306;
        self.GBKFreq[55][132] = 305;
        self.GBKFreq[94][2] = 304;
        self.GBKFreq[54][133] = 303;
        self.GBKFreq[56][190] = 302;
        self.GBKFreq[58][174] = 301;
        self.GBKFreq[80][144] = 300;
        self.GBKFreq[85][113] = 299;
        /*
         * GBKFreq[83][15] = 298; GBKFreq[105][80] = 297; GBKFreq[7][179] = 296;
         * GBKFreq[93][4] = 295; GBKFreq[123][40] = 294; GBKFreq[85][120] = 293;
         * GBKFreq[77][165] = 292; GBKFreq[86][67] = 291; GBKFreq[25][162] =
         * 290; GBKFreq[77][183] = 289; GBKFreq[83][71] = 288; GBKFreq[78][99] =
         * 287; GBKFreq[72][177] = 286; GBKFreq[71][97] = 285; GBKFreq[58][111]
         * = 284; GBKFreq[77][175] = 283; GBKFreq[76][181] = 282;
         * GBKFreq[71][142] = 281; GBKFreq[64][150] = 280; GBKFreq[5][142] =
         * 279; GBKFreq[73][128] = 278; GBKFreq[73][156] = 277; GBKFreq[60][188]
         * = 276; GBKFreq[64][56] = 275; GBKFreq[74][128] = 274;
         * GBKFreq[48][163] = 273; GBKFreq[54][116] = 272; GBKFreq[73][127] =
         * 271; GBKFreq[16][176] = 270; GBKFreq[62][149] = 269; GBKFreq[105][96]
         * = 268; GBKFreq[55][186] = 267; GBKFreq[4][51] = 266; GBKFreq[48][113]
         * = 265; GBKFreq[48][152] = 264; GBKFreq[23][9] = 263; GBKFreq[56][102]
         * = 262; GBKFreq[11][81] = 261; GBKFreq[82][112] = 260; GBKFreq[65][85]
         * = 259; GBKFreq[69][125] = 258; GBKFreq[68][31] = 257; GBKFreq[5][20]
         * = 256; GBKFreq[60][176] = 255; GBKFreq[82][81] = 254;
         * GBKFreq[72][107] = 253; GBKFreq[3][52] = 252; GBKFreq[71][157] = 251;
         * GBKFreq[24][46] = 250; GBKFreq[69][108] = 249; GBKFreq[78][178] =
         * 248; GBKFreq[9][69] = 247; GBKFreq[73][144] = 246; GBKFreq[63][187] =
         * 245; GBKFreq[68][36] = 244; GBKFreq[47][151] = 243; GBKFreq[14][74] =
         * 242; GBKFreq[47][114] = 241; GBKFreq[80][171] = 240; GBKFreq[75][152]
         * = 239; GBKFreq[86][40] = 238; GBKFreq[93][43] = 237; GBKFreq[2][50] =
         * 236; GBKFreq[62][66] = 235; GBKFreq[1][183] = 234; GBKFreq[74][124] =
         * 233; GBKFreq[58][104] = 232; GBKFreq[83][106] = 231; GBKFreq[60][144]
         * = 230; GBKFreq[48][99] = 229; GBKFreq[54][157] = 228;
         * GBKFreq[70][179] = 227; GBKFreq[61][127] = 226; GBKFreq[57][135] =
         * 225; GBKFreq[59][190] = 224; GBKFreq[77][116] = 223; GBKFreq[26][17]
         * = 222; GBKFreq[60][13] = 221; GBKFreq[71][38] = 220; GBKFreq[85][177]
         * = 219; GBKFreq[59][73] = 218; GBKFreq[50][150] = 217;
         * GBKFreq[79][102] = 216; GBKFreq[76][118] = 215; GBKFreq[67][132] =
         * 214; GBKFreq[73][146] = 213; GBKFreq[83][184] = 212; GBKFreq[86][159]
         * = 211; GBKFreq[95][120] = 210; GBKFreq[23][139] = 209;
         * GBKFreq[64][183] = 208; GBKFreq[85][103] = 207; GBKFreq[41][90] =
         * 206; GBKFreq[87][72] = 205; GBKFreq[62][104] = 204; GBKFreq[79][168]
         * = 203; GBKFreq[79][150] = 202; GBKFreq[104][20] = 201;
         * GBKFreq[56][114] = 200; GBKFreq[84][26] = 199; GBKFreq[57][99] = 198;
         * GBKFreq[62][154] = 197; GBKFreq[47][98] = 196; GBKFreq[61][64] = 195;
         * GBKFreq[112][18] = 194; GBKFreq[123][19] = 193; GBKFreq[4][98] = 192;
         * GBKFreq[47][163] = 191; GBKFreq[66][188] = 190; GBKFreq[81][85] =
         * 189; GBKFreq[82][30] = 188; GBKFreq[65][83] = 187; GBKFreq[67][24] =
         * 186; GBKFreq[68][179] = 185; GBKFreq[55][177] = 184; GBKFreq[2][122]
         * = 183; GBKFreq[47][139] = 182; GBKFreq[79][158] = 181;
         * GBKFreq[64][143] = 180; GBKFreq[100][24] = 179; GBKFreq[73][103] =
         * 178; GBKFreq[50][148] = 177; GBKFreq[86][97] = 176; GBKFreq[59][116]
         * = 175; GBKFreq[64][173] = 174; GBKFreq[99][91] = 173; GBKFreq[11][99]
         * = 172; GBKFreq[78][179] = 171; GBKFreq[18][17] = 170;
         * GBKFreq[58][185] = 169; GBKFreq[47][165] = 168; GBKFreq[67][131] =
         * 167; GBKFreq[94][40] = 166; GBKFreq[74][153] = 165; GBKFreq[79][142]
         * = 164; GBKFreq[57][98] = 163; GBKFreq[1][164] = 162; GBKFreq[55][168]
         * = 161; GBKFreq[13][141] = 160; GBKFreq[51][31] = 159;
         * GBKFreq[57][178] = 158; GBKFreq[50][189] = 157; GBKFreq[60][167] =
         * 156; GBKFreq[80][34] = 155; GBKFreq[109][80] = 154; GBKFreq[85][54] =
         * 153; GBKFreq[69][183] = 152; GBKFreq[67][143] = 151; GBKFreq[47][120]
         * = 150; GBKFreq[45][75] = 149; GBKFreq[82][98] = 148; GBKFreq[83][22]
         * = 147; GBKFreq[13][103] = 146; GBKFreq[49][174] = 145;
         * GBKFreq[57][181] = 144; GBKFreq[64][127] = 143; GBKFreq[61][131] =
         * 142; GBKFreq[52][180] = 141; GBKFreq[74][134] = 140; GBKFreq[84][187]
         * = 139; GBKFreq[81][189] = 138; GBKFreq[47][160] = 137;
         * GBKFreq[66][148] = 136; GBKFreq[7][4] = 135; GBKFreq[85][134] = 134;
         * GBKFreq[88][13] = 133; GBKFreq[88][80] = 132; GBKFreq[69][166] = 131;
         * GBKFreq[86][18] = 130; GBKFreq[79][141] = 129; GBKFreq[50][108] =
         * 128; GBKFreq[94][69] = 127; GBKFreq[81][110] = 126; GBKFreq[69][119]
         * = 125; GBKFreq[72][161] = 124; GBKFreq[106][45] = 123;
         * GBKFreq[73][124] = 122; GBKFreq[94][28] = 121; GBKFreq[63][174] =
         * 120; GBKFreq[3][149] = 119; GBKFreq[24][160] = 118; GBKFreq[113][94]
         * = 117; GBKFreq[56][138] = 116; GBKFreq[64][185] = 115;
         * GBKFreq[86][56] = 114; GBKFreq[56][150] = 113; GBKFreq[110][55] =
         * 112; GBKFreq[28][13] = 111; GBKFreq[54][190] = 110; GBKFreq[8][180] =
         * 109; GBKFreq[73][149] = 108; GBKFreq[80][155] = 107; GBKFreq[83][172]
         * = 106; GBKFreq[67][174] = 105; GBKFreq[64][180] = 104;
         * GBKFreq[84][46] = 103; GBKFreq[91][74] = 102; GBKFreq[69][134] = 101;
         * GBKFreq[61][107] = 100; GBKFreq[47][171] = 99; GBKFreq[59][51] = 98;
         * GBKFreq[109][74] = 97; GBKFreq[64][174] = 96; GBKFreq[52][151] = 95;
         * GBKFreq[51][176] = 94; GBKFreq[80][157] = 93; GBKFreq[94][31] = 92;
         * GBKFreq[79][155] = 91; GBKFreq[72][174] = 90; GBKFreq[69][113] = 89;
         * GBKFreq[83][167] = 88; GBKFreq[83][122] = 87; GBKFreq[8][178] = 86;
         * GBKFreq[70][186] = 85; GBKFreq[59][153] = 84; GBKFreq[84][68] = 83;
         * GBKFreq[79][39] = 82; GBKFreq[47][180] = 81; GBKFreq[88][53] = 80;
         * GBKFreq[57][154] = 79; GBKFreq[47][153] = 78; GBKFreq[3][153] = 77;
         * GBKFreq[76][134] = 76; GBKFreq[51][166] = 75; GBKFreq[58][176] = 74;
         * GBKFreq[27][138] = 73; GBKFreq[73][126] = 72; GBKFreq[76][185] = 71;
         * GBKFreq[52][186] = 70; GBKFreq[81][151] = 69; GBKFreq[26][50] = 68;
         * GBKFreq[76][173] = 67; GBKFreq[106][56] = 66; GBKFreq[85][142] = 65;
         * GBKFreq[11][103] = 64; GBKFreq[69][159] = 63; GBKFreq[53][142] = 62;
         * GBKFreq[7][6] = 61; GBKFreq[84][59] = 60; GBKFreq[86][3] = 59;
         * GBKFreq[64][144] = 58; GBKFreq[1][187] = 57; GBKFreq[82][128] = 56;
         * GBKFreq[3][66] = 55; GBKFreq[68][133] = 54; GBKFreq[55][167] = 53;
         * GBKFreq[52][130] = 52; GBKFreq[61][133] = 51; GBKFreq[72][181] = 50;
         * GBKFreq[25][98] = 49; GBKFreq[84][149] = 48; GBKFreq[91][91] = 47;
         * GBKFreq[47][188] = 46; GBKFreq[68][130] = 45; GBKFreq[22][44] = 44;
         * GBKFreq[81][121] = 43; GBKFreq[72][140] = 42; GBKFreq[55][133] = 41;
         * GBKFreq[55][185] = 40; GBKFreq[56][105] = 39; GBKFreq[60][30] = 38;
         * GBKFreq[70][103] = 37; GBKFreq[62][141] = 36; GBKFreq[70][144] = 35;
         * GBKFreq[59][111] = 34; GBKFreq[54][17] = 33; GBKFreq[18][190] = 32;
         * GBKFreq[65][164] = 31; GBKFreq[83][125] = 30; GBKFreq[61][121] = 29;
         * GBKFreq[48][13] = 28; GBKFreq[51][189] = 27; GBKFreq[65][68] = 26;
         * GBKFreq[7][0] = 25; GBKFreq[76][188] = 24; GBKFreq[85][117] = 23;
         * GBKFreq[45][33] = 22; GBKFreq[78][187] = 21; GBKFreq[106][48] = 20;
         * GBKFreq[59][52] = 19; GBKFreq[86][185] = 18; GBKFreq[84][121] = 17;
         * GBKFreq[82][189] = 16; GBKFreq[68][156] = 15; GBKFreq[55][125] = 14;
         * GBKFreq[65][175] = 13; GBKFreq[7][140] = 12; GBKFreq[50][106] = 11;
         * GBKFreq[59][124] = 10; GBKFreq[67][115] = 9; GBKFreq[82][114] = 8;
         * GBKFreq[74][121] = 7; GBKFreq[106][69] = 6; GBKFreq[94][27] = 5;
         * GBKFreq[78][98] = 4; GBKFreq[85][186] = 3; GBKFreq[108][90] = 2;
         * GBKFreq[62][160] = 1; GBKFreq[60][169] = 0;
         */
        self.KRFreq[31][43] = 600;
        self.KRFreq[19][56] = 599;
        self.KRFreq[38][46] = 598;
        self.KRFreq[3][3] = 597;
        self.KRFreq[29][77] = 596;
        self.KRFreq[19][33] = 595;
        self.KRFreq[30][0] = 594;
        self.KRFreq[29][89] = 593;
        self.KRFreq[31][26] = 592;
        self.KRFreq[31][38] = 591;
        self.KRFreq[32][85] = 590;
        self.KRFreq[15][0] = 589;
        self.KRFreq[16][54] = 588;
        self.KRFreq[15][76] = 587;
        self.KRFreq[31][25] = 586;
        self.KRFreq[23][13] = 585;
        self.KRFreq[28][34] = 584;
        self.KRFreq[18][9] = 583;
        self.KRFreq[29][37] = 582;
        self.KRFreq[22][45] = 581;
        self.KRFreq[19][46] = 580;
        self.KRFreq[16][65] = 579;
        self.KRFreq[23][5] = 578;
        self.KRFreq[26][70] = 577;
        self.KRFreq[31][53] = 576;
        self.KRFreq[27][12] = 575;
        self.KRFreq[30][67] = 574;
        self.KRFreq[31][57] = 573;
        self.KRFreq[20][20] = 572;
        self.KRFreq[30][31] = 571;
        self.KRFreq[20][72] = 570;
        self.KRFreq[15][51] = 569;
        self.KRFreq[3][8] = 568;
        self.KRFreq[32][53] = 567;
        self.KRFreq[27][85] = 566;
        self.KRFreq[25][23] = 565;
        self.KRFreq[15][44] = 564;
        self.KRFreq[32][3] = 563;
        self.KRFreq[31][68] = 562;
        self.KRFreq[30][24] = 561;
        self.KRFreq[29][49] = 560;
        self.KRFreq[27][49] = 559;
        self.KRFreq[23][23] = 558;
        self.KRFreq[31][91] = 557;
        self.KRFreq[31][46] = 556;
        self.KRFreq[19][74] = 555;
        self.KRFreq[27][27] = 554;
        self.KRFreq[3][17] = 553;
        self.KRFreq[20][38] = 552;
        self.KRFreq[21][82] = 551;
        self.KRFreq[28][25] = 550;
        self.KRFreq[32][5] = 549;
        self.KRFreq[31][23] = 548;
        self.KRFreq[25][45] = 547;
        self.KRFreq[32][87] = 546;
        self.KRFreq[18][26] = 545;
        self.KRFreq[24][10] = 544;
        self.KRFreq[26][82] = 543;
        self.KRFreq[15][89] = 542;
        self.KRFreq[28][36] = 541;
        self.KRFreq[28][31] = 540;
        self.KRFreq[16][23] = 539;
        self.KRFreq[16][77] = 538;
        self.KRFreq[19][84] = 537;
        self.KRFreq[23][72] = 536;
        self.KRFreq[38][48] = 535;
        self.KRFreq[23][2] = 534;
        self.KRFreq[30][20] = 533;
        self.KRFreq[38][47] = 532;
        self.KRFreq[39][12] = 531;
        self.KRFreq[23][21] = 530;
        self.KRFreq[18][17] = 529;
        self.KRFreq[30][87] = 528;
        self.KRFreq[29][62] = 527;
        self.KRFreq[29][87] = 526;
        self.KRFreq[34][53] = 525;
        self.KRFreq[32][29] = 524;
        self.KRFreq[35][0] = 523;
        self.KRFreq[24][43] = 522;
        self.KRFreq[36][44] = 521;
        self.KRFreq[20][30] = 520;
        self.KRFreq[39][86] = 519;
        self.KRFreq[22][14] = 518;
        self.KRFreq[29][39] = 517;
        self.KRFreq[28][38] = 516;
        self.KRFreq[23][79] = 515;
        self.KRFreq[24][56] = 514;
        self.KRFreq[29][63] = 513;
        self.KRFreq[31][45] = 512;
        self.KRFreq[23][26] = 511;
        self.KRFreq[15][87] = 510;
        self.KRFreq[30][74] = 509;
        self.KRFreq[24][69] = 508;
        self.KRFreq[20][4] = 507;
        self.KRFreq[27][50] = 506;
        self.KRFreq[30][75] = 505;
        self.KRFreq[24][13] = 504;
        self.KRFreq[30][8] = 503;
        self.KRFreq[31][6] = 502;
        self.KRFreq[25][80] = 501;
        self.KRFreq[36][8] = 500;
        self.KRFreq[15][18] = 499;
        self.KRFreq[39][23] = 498;
        self.KRFreq[16][24] = 497;
        self.KRFreq[31][89] = 496;
        self.KRFreq[15][71] = 495;
        self.KRFreq[15][57] = 494;
        self.KRFreq[30][11] = 493;
        self.KRFreq[15][36] = 492;
        self.KRFreq[16][60] = 491;
        self.KRFreq[24][45] = 490;
        self.KRFreq[37][35] = 489;
        self.KRFreq[24][87] = 488;
        self.KRFreq[20][45] = 487;
        self.KRFreq[31][90] = 486;
        self.KRFreq[32][21] = 485;
        self.KRFreq[19][70] = 484;
        self.KRFreq[24][15] = 483;
        self.KRFreq[26][92] = 482;
        self.KRFreq[37][13] = 481;
        self.KRFreq[39][2] = 480;
        self.KRFreq[23][70] = 479;
        self.KRFreq[27][25] = 478;
        self.KRFreq[15][69] = 477;
        self.KRFreq[19][61] = 476;
        self.KRFreq[31][58] = 475;
        self.KRFreq[24][57] = 474;
        self.KRFreq[36][74] = 473;
        self.KRFreq[21][6] = 472;
        self.KRFreq[30][44] = 471;
        self.KRFreq[15][91] = 470;
        self.KRFreq[27][16] = 469;
        self.KRFreq[29][42] = 468;
        self.KRFreq[33][86] = 467;
        self.KRFreq[29][41] = 466;
        self.KRFreq[20][68] = 465;
        self.KRFreq[25][47] = 464;
        self.KRFreq[22][0] = 463;
        self.KRFreq[18][14] = 462;
        self.KRFreq[31][28] = 461;
        self.KRFreq[15][2] = 460;
        self.KRFreq[23][76] = 459;
        self.KRFreq[38][32] = 458;
        self.KRFreq[29][82] = 457;
        self.KRFreq[21][86] = 456;
        self.KRFreq[24][62] = 455;
        self.KRFreq[31][64] = 454;
        self.KRFreq[38][26] = 453;
        self.KRFreq[32][86] = 452;
        self.KRFreq[22][32] = 451;
        self.KRFreq[19][59] = 450;
        self.KRFreq[34][18] = 449;
        self.KRFreq[18][54] = 448;
        self.KRFreq[38][63] = 447;
        self.KRFreq[36][23] = 446;
        self.KRFreq[35][35] = 445;
        self.KRFreq[32][62] = 444;
        self.KRFreq[28][35] = 443;
        self.KRFreq[27][13] = 442;
        self.KRFreq[31][59] = 441;
        self.KRFreq[29][29] = 440;
        self.KRFreq[15][64] = 439;
        self.KRFreq[26][84] = 438;
        self.KRFreq[21][90] = 437;
        self.KRFreq[20][24] = 436;
        self.KRFreq[16][18] = 435;
        self.KRFreq[22][23] = 434;
        self.KRFreq[31][14] = 433;
        self.KRFreq[15][1] = 432;
        self.KRFreq[18][63] = 431;
        self.KRFreq[19][10] = 430;
        self.KRFreq[25][49] = 429;
        self.KRFreq[36][57] = 428;
        self.KRFreq[20][22] = 427;
        self.KRFreq[15][15] = 426;
        self.KRFreq[31][51] = 425;
        self.KRFreq[24][60] = 424;
        self.KRFreq[31][70] = 423;
        self.KRFreq[15][7] = 422;
        self.KRFreq[28][40] = 421;
        self.KRFreq[18][41] = 420;
        self.KRFreq[15][38] = 419;
        self.KRFreq[32][0] = 418;
        self.KRFreq[19][51] = 417;
        self.KRFreq[34][62] = 416;
        self.KRFreq[16][27] = 415;
        self.KRFreq[20][70] = 414;
        self.KRFreq[22][33] = 413;
        self.KRFreq[26][73] = 412;
        self.KRFreq[20][79] = 411;
        self.KRFreq[23][6] = 410;
        self.KRFreq[24][85] = 409;
        self.KRFreq[38][51] = 408;
        self.KRFreq[29][88] = 407;
        self.KRFreq[38][55] = 406;
        self.KRFreq[32][32] = 405;
        self.KRFreq[27][18] = 404;
        self.KRFreq[23][87] = 403;
        self.KRFreq[35][6] = 402;
        self.KRFreq[34][27] = 401;
        self.KRFreq[39][35] = 400;
        self.KRFreq[30][88] = 399;
        self.KRFreq[32][92] = 398;
        self.KRFreq[32][49] = 397;
        self.KRFreq[24][61] = 396;
        self.KRFreq[18][74] = 395;
        self.KRFreq[23][77] = 394;
        self.KRFreq[23][50] = 393;
        self.KRFreq[23][32] = 392;
        self.KRFreq[23][36] = 391;
        self.KRFreq[38][38] = 390;
        self.KRFreq[29][86] = 389;
        self.KRFreq[36][15] = 388;
        self.KRFreq[31][50] = 387;
        self.KRFreq[15][86] = 386;
        self.KRFreq[39][13] = 385;
        self.KRFreq[34][26] = 384;
        self.KRFreq[19][34] = 383;
        self.KRFreq[16][3] = 382;
        self.KRFreq[26][93] = 381;
        self.KRFreq[19][67] = 380;
        self.KRFreq[24][72] = 379;
        self.KRFreq[29][17] = 378;
        self.KRFreq[23][24] = 377;
        self.KRFreq[25][19] = 376;
        self.KRFreq[18][65] = 375;
        self.KRFreq[30][78] = 374;
        self.KRFreq[27][52] = 373;
        self.KRFreq[22][18] = 372;
        self.KRFreq[16][38] = 371;
        self.KRFreq[21][26] = 370;
        self.KRFreq[34][20] = 369;
        self.KRFreq[15][42] = 368;
        self.KRFreq[16][71] = 367;
        self.KRFreq[17][17] = 366;
        self.KRFreq[24][71] = 365;
        self.KRFreq[18][84] = 364;
        self.KRFreq[15][40] = 363;
        self.KRFreq[31][62] = 362;
        self.KRFreq[15][8] = 361;
        self.KRFreq[16][69] = 360;
        self.KRFreq[29][79] = 359;
        self.KRFreq[38][91] = 358;
        self.KRFreq[31][92] = 357;
        self.KRFreq[20][77] = 356;
        self.KRFreq[3][16] = 355;
        self.KRFreq[27][87] = 354;
        self.KRFreq[16][25] = 353;
        self.KRFreq[36][33] = 352;
        self.KRFreq[37][76] = 351;
        self.KRFreq[30][12] = 350;
        self.KRFreq[26][75] = 349;
        self.KRFreq[25][14] = 348;
        self.KRFreq[32][26] = 347;
        self.KRFreq[23][22] = 346;
        self.KRFreq[20][90] = 345;
        self.KRFreq[19][8] = 344;
        self.KRFreq[38][41] = 343;
        self.KRFreq[34][2] = 342;
        self.KRFreq[39][4] = 341;
        self.KRFreq[27][89] = 340;
        self.KRFreq[28][41] = 339;
        self.KRFreq[28][44] = 338;
        self.KRFreq[24][92] = 337;
        self.KRFreq[34][65] = 336;
        self.KRFreq[39][14] = 335;
        self.KRFreq[21][38] = 334;
        self.KRFreq[19][31] = 333;
        self.KRFreq[37][39] = 332;
        self.KRFreq[33][41] = 331;
        self.KRFreq[38][4] = 330;
        self.KRFreq[23][80] = 329;
        self.KRFreq[25][24] = 328;
        self.KRFreq[37][17] = 327;
        self.KRFreq[22][16] = 326;
        self.KRFreq[22][46] = 325;
        self.KRFreq[33][91] = 324;
        self.KRFreq[24][89] = 323;
        self.KRFreq[30][52] = 322;
        self.KRFreq[29][38] = 321;
        self.KRFreq[38][85] = 320;
        self.KRFreq[15][12] = 319;
        self.KRFreq[27][58] = 318;
        self.KRFreq[29][52] = 317;
        self.KRFreq[37][38] = 316;
        self.KRFreq[34][41] = 315;
        self.KRFreq[31][65] = 314;
        self.KRFreq[29][53] = 313;
        self.KRFreq[22][47] = 312;
        self.KRFreq[22][19] = 311;
        self.KRFreq[26][0] = 310;
        self.KRFreq[37][86] = 309;
        self.KRFreq[35][4] = 308;
        self.KRFreq[36][54] = 307;
        self.KRFreq[20][76] = 306;
        self.KRFreq[30][9] = 305;
        self.KRFreq[30][33] = 304;
        self.KRFreq[23][17] = 303;
        self.KRFreq[23][33] = 302;
        self.KRFreq[38][52] = 301;
        self.KRFreq[15][19] = 300;
        self.KRFreq[28][45] = 299;
        self.KRFreq[29][78] = 298;
        self.KRFreq[23][15] = 297;
        self.KRFreq[33][5] = 296;
        self.KRFreq[17][40] = 295;
        self.KRFreq[30][83] = 294;
        self.KRFreq[18][1] = 293;
        self.KRFreq[30][81] = 292;
        self.KRFreq[19][40] = 291;
        self.KRFreq[24][47] = 290;
        self.KRFreq[17][56] = 289;
        self.KRFreq[39][80] = 288;
        self.KRFreq[30][46] = 287;
        self.KRFreq[16][61] = 286;
        self.KRFreq[26][78] = 285;
        self.KRFreq[26][57] = 284;
        self.KRFreq[20][46] = 283;
        self.KRFreq[25][15] = 282;
        self.KRFreq[25][91] = 281;
        self.KRFreq[21][83] = 280;
        self.KRFreq[30][77] = 279;
        self.KRFreq[35][30] = 278;
        self.KRFreq[30][34] = 277;
        self.KRFreq[20][69] = 276;
        self.KRFreq[35][10] = 275;
        self.KRFreq[29][70] = 274;
        self.KRFreq[22][50] = 273;
        self.KRFreq[18][0] = 272;
        self.KRFreq[22][64] = 271;
        self.KRFreq[38][65] = 270;
        self.KRFreq[22][70] = 269;
        self.KRFreq[24][58] = 268;
        self.KRFreq[19][66] = 267;
        self.KRFreq[30][59] = 266;
        self.KRFreq[37][14] = 265;
        self.KRFreq[16][56] = 264;
        self.KRFreq[29][85] = 263;
        self.KRFreq[31][15] = 262;
        self.KRFreq[36][84] = 261;
        self.KRFreq[39][15] = 260;
        self.KRFreq[39][90] = 259;
        self.KRFreq[18][12] = 258;
        self.KRFreq[21][93] = 257;
        self.KRFreq[24][66] = 256;
        self.KRFreq[27][90] = 255;
        self.KRFreq[25][90] = 254;
        self.KRFreq[22][24] = 253;
        self.KRFreq[36][67] = 252;
        self.KRFreq[33][90] = 251;
        self.KRFreq[15][60] = 250;
        self.KRFreq[23][85] = 249;
        self.KRFreq[34][1] = 248;
        self.KRFreq[39][37] = 247;
        self.KRFreq[21][18] = 246;
        self.KRFreq[34][4] = 245;
        self.KRFreq[28][33] = 244;
        self.KRFreq[15][13] = 243;
        self.KRFreq[32][22] = 242;
        self.KRFreq[30][76] = 241;
        self.KRFreq[20][21] = 240;
        self.KRFreq[38][66] = 239;
        self.KRFreq[32][55] = 238;
        self.KRFreq[32][89] = 237;
        self.KRFreq[25][26] = 236;
        self.KRFreq[16][80] = 235;
        self.KRFreq[15][43] = 234;
        self.KRFreq[38][54] = 233;
        self.KRFreq[39][68] = 232;
        self.KRFreq[22][88] = 231;
        self.KRFreq[21][84] = 230;
        self.KRFreq[21][17] = 229;
        self.KRFreq[20][28] = 228;
        self.KRFreq[32][1] = 227;
        self.KRFreq[33][87] = 226;
        self.KRFreq[38][71] = 225;
        self.KRFreq[37][47] = 224;
        self.KRFreq[18][77] = 223;
        self.KRFreq[37][58] = 222;
        self.KRFreq[34][74] = 221;
        self.KRFreq[32][54] = 220;
        self.KRFreq[27][33] = 219;
        self.KRFreq[32][93] = 218;
        self.KRFreq[23][51] = 217;
        self.KRFreq[20][57] = 216;
        self.KRFreq[22][37] = 215;
        self.KRFreq[39][10] = 214;
        self.KRFreq[39][17] = 213;
        self.KRFreq[33][4] = 212;
        self.KRFreq[32][84] = 211;
        self.KRFreq[34][3] = 210;
        self.KRFreq[28][27] = 209;
        self.KRFreq[15][79] = 208;
        self.KRFreq[34][21] = 207;
        self.KRFreq[34][69] = 206;
        self.KRFreq[21][62] = 205;
        self.KRFreq[36][24] = 204;
        self.KRFreq[16][89] = 203;
        self.KRFreq[18][48] = 202;
        self.KRFreq[38][15] = 201;
        self.KRFreq[36][58] = 200;
        self.KRFreq[21][56] = 199;
        self.KRFreq[34][48] = 198;
        self.KRFreq[21][15] = 197;
        self.KRFreq[39][3] = 196;
        self.KRFreq[16][44] = 195;
        self.KRFreq[18][79] = 194;
        self.KRFreq[25][13] = 193;
        self.KRFreq[29][47] = 192;
        self.KRFreq[38][88] = 191;
        self.KRFreq[20][71] = 190;
        self.KRFreq[16][58] = 189;
        self.KRFreq[35][57] = 188;
        self.KRFreq[29][30] = 187;
        self.KRFreq[29][23] = 186;
        self.KRFreq[34][93] = 185;
        self.KRFreq[30][85] = 184;
        self.KRFreq[15][80] = 183;
        self.KRFreq[32][78] = 182;
        self.KRFreq[37][82] = 181;
        self.KRFreq[22][40] = 180;
        self.KRFreq[21][69] = 179;
        self.KRFreq[26][85] = 178;
        self.KRFreq[31][31] = 177;
        self.KRFreq[28][64] = 176;
        self.KRFreq[38][13] = 175;
        self.KRFreq[25][2] = 174;
        self.KRFreq[22][34] = 173;
        self.KRFreq[28][28] = 172;
        self.KRFreq[24][91] = 171;
        self.KRFreq[33][74] = 170;
        self.KRFreq[29][40] = 169;
        self.KRFreq[15][77] = 168;
        self.KRFreq[32][80] = 167;
        self.KRFreq[30][41] = 166;
        self.KRFreq[23][30] = 165;
        self.KRFreq[24][63] = 164;
        self.KRFreq[30][53] = 163;
        self.KRFreq[39][70] = 162;
        self.KRFreq[23][61] = 161;
        self.KRFreq[37][27] = 160;
        self.KRFreq[16][55] = 159;
        self.KRFreq[22][74] = 158;
        self.KRFreq[26][50] = 157;
        self.KRFreq[16][10] = 156;
        self.KRFreq[34][63] = 155;
        self.KRFreq[35][14] = 154;
        self.KRFreq[17][7] = 153;
        self.KRFreq[15][59] = 152;
        self.KRFreq[27][23] = 151;
        self.KRFreq[18][70] = 150;
        self.KRFreq[32][56] = 149;
        self.KRFreq[37][87] = 148;
        self.KRFreq[17][61] = 147;
        self.KRFreq[18][83] = 146;
        self.KRFreq[23][86] = 145;
        self.KRFreq[17][31] = 144;
        self.KRFreq[23][83] = 143;
        self.KRFreq[35][2] = 142;
        self.KRFreq[18][64] = 141;
        self.KRFreq[27][43] = 140;
        self.KRFreq[32][42] = 139;
        self.KRFreq[25][76] = 138;
        self.KRFreq[19][85] = 137;
        self.KRFreq[37][81] = 136;
        self.KRFreq[38][83] = 135;
        self.KRFreq[35][7] = 134;
        self.KRFreq[16][51] = 133;
        self.KRFreq[27][22] = 132;
        self.KRFreq[16][76] = 131;
        self.KRFreq[22][4] = 130;
        self.KRFreq[38][84] = 129;
        self.KRFreq[17][83] = 128;
        self.KRFreq[24][46] = 127;
        self.KRFreq[33][15] = 126;
        self.KRFreq[20][48] = 125;
        self.KRFreq[17][30] = 124;
        self.KRFreq[30][93] = 123;
        self.KRFreq[28][11] = 122;
        self.KRFreq[28][30] = 121;
        self.KRFreq[15][62] = 120;
        self.KRFreq[17][87] = 119;
        self.KRFreq[32][81] = 118;
        self.KRFreq[23][37] = 117;
        self.KRFreq[30][22] = 116;
        self.KRFreq[32][66] = 115;
        self.KRFreq[33][78] = 114;
        self.KRFreq[21][4] = 113;
        self.KRFreq[31][17] = 112;
        self.KRFreq[39][61] = 111;
        self.KRFreq[18][76] = 110;
        self.KRFreq[15][85] = 109;
        self.KRFreq[31][47] = 108;
        self.KRFreq[19][57] = 107;
        self.KRFreq[23][55] = 106;
        self.KRFreq[27][29] = 105;
        self.KRFreq[29][46] = 104;
        self.KRFreq[33][0] = 103;
        self.KRFreq[16][83] = 102;
        self.KRFreq[39][78] = 101;
        self.KRFreq[32][77] = 100;
        self.KRFreq[36][25] = 99;
        self.KRFreq[34][19] = 98;
        self.KRFreq[38][49] = 97;
        self.KRFreq[19][25] = 96;
        self.KRFreq[23][53] = 95;
        self.KRFreq[28][43] = 94;
        self.KRFreq[31][44] = 93;
        self.KRFreq[36][34] = 92;
        self.KRFreq[16][34] = 91;
        self.KRFreq[35][1] = 90;
        self.KRFreq[19][87] = 89;
        self.KRFreq[18][53] = 88;
        self.KRFreq[29][54] = 87;
        self.KRFreq[22][41] = 86;
        self.KRFreq[38][18] = 85;
        self.KRFreq[22][2] = 84;
        self.KRFreq[20][3] = 83;
        self.KRFreq[39][69] = 82;
        self.KRFreq[30][29] = 81;
        self.KRFreq[28][19] = 80;
        self.KRFreq[29][90] = 79;
        self.KRFreq[17][86] = 78;
        self.KRFreq[15][9] = 77;
        self.KRFreq[39][73] = 76;
        self.KRFreq[15][37] = 75;
        self.KRFreq[35][40] = 74;
        self.KRFreq[33][77] = 73;
        self.KRFreq[27][86] = 72;
        self.KRFreq[36][79] = 71;
        self.KRFreq[23][18] = 70;
        self.KRFreq[34][87] = 69;
        self.KRFreq[39][24] = 68;
        self.KRFreq[26][8] = 67;
        self.KRFreq[33][48] = 66;
        self.KRFreq[39][30] = 65;
        self.KRFreq[33][28] = 64;
        self.KRFreq[16][67] = 63;
        self.KRFreq[31][78] = 62;
        self.KRFreq[32][23] = 61;
        self.KRFreq[24][55] = 60;
        self.KRFreq[30][68] = 59;
        self.KRFreq[18][60] = 58;
        self.KRFreq[15][17] = 57;
        self.KRFreq[23][34] = 56;
        self.KRFreq[20][49] = 55;
        self.KRFreq[15][78] = 54;
        self.KRFreq[24][14] = 53;
        self.KRFreq[19][41] = 52;
        self.KRFreq[31][55] = 51;
        self.KRFreq[21][39] = 50;
        self.KRFreq[35][9] = 49;
        self.KRFreq[30][15] = 48;
        self.KRFreq[20][52] = 47;
        self.KRFreq[35][71] = 46;
        self.KRFreq[20][7] = 45;
        self.KRFreq[29][72] = 44;
        self.KRFreq[37][77] = 43;
        self.KRFreq[22][35] = 42;
        self.KRFreq[20][61] = 41;
        self.KRFreq[31][60] = 40;
        self.KRFreq[20][93] = 39;
        self.KRFreq[27][92] = 38;
        self.KRFreq[28][16] = 37;
        self.KRFreq[36][26] = 36;
        self.KRFreq[18][89] = 35;
        self.KRFreq[21][63] = 34;
        self.KRFreq[22][52] = 33;
        self.KRFreq[24][65] = 32;
        self.KRFreq[31][8] = 31;
        self.KRFreq[31][49] = 30;
        self.KRFreq[33][30] = 29;
        self.KRFreq[37][15] = 28;
        self.KRFreq[18][18] = 27;
        self.KRFreq[25][50] = 26;
        self.KRFreq[29][20] = 25;
        self.KRFreq[35][48] = 24;
        self.KRFreq[38][75] = 23;
        self.KRFreq[26][83] = 22;
        self.KRFreq[21][87] = 21;
        self.KRFreq[27][71] = 20;
        self.KRFreq[32][91] = 19;
        self.KRFreq[25][73] = 18;
        self.KRFreq[16][84] = 17;
        self.KRFreq[25][31] = 16;
        self.KRFreq[17][90] = 15;
        self.KRFreq[18][40] = 14;
        self.KRFreq[17][77] = 13;
        self.KRFreq[17][35] = 12;
        self.KRFreq[23][52] = 11;
        self.KRFreq[23][35] = 10;
        self.KRFreq[16][5] = 9;
        self.KRFreq[23][58] = 8;
        self.KRFreq[19][60] = 7;
        self.KRFreq[30][32] = 6;
        self.KRFreq[38][34] = 5;
        self.KRFreq[23][4] = 4;
        self.KRFreq[23][1] = 3;
        self.KRFreq[27][57] = 2;
        self.KRFreq[39][38] = 1;
        self.KRFreq[32][33] = 0;
        self.JPFreq[3][74] = 600;
        self.JPFreq[3][45] = 599;
        self.JPFreq[3][3] = 598;
        self.JPFreq[3][24] = 597;
        self.JPFreq[3][30] = 596;
        self.JPFreq[3][42] = 595;
        self.JPFreq[3][46] = 594;
        self.JPFreq[3][39] = 593;
        self.JPFreq[3][11] = 592;
        self.JPFreq[3][37] = 591;
        self.JPFreq[3][38] = 590;
        self.JPFreq[3][31] = 589;
        self.JPFreq[3][41] = 588;
        self.JPFreq[3][5] = 587;
        self.JPFreq[3][10] = 586;
        self.JPFreq[3][75] = 585;
        self.JPFreq[3][65] = 584;
        self.JPFreq[3][72] = 583;
        self.JPFreq[37][91] = 582;
        self.JPFreq[0][27] = 581;
        self.JPFreq[3][18] = 580;
        self.JPFreq[3][22] = 579;
        self.JPFreq[3][61] = 578;
        self.JPFreq[3][14] = 577;
        self.JPFreq[24][80] = 576;
        self.JPFreq[4][82] = 575;
        self.JPFreq[17][80] = 574;
        self.JPFreq[30][44] = 573;
        self.JPFreq[3][73] = 572;
        self.JPFreq[3][64] = 571;
        self.JPFreq[38][14] = 570;
        self.JPFreq[33][70] = 569;
        self.JPFreq[3][1] = 568;
        self.JPFreq[3][16] = 567;
        self.JPFreq[3][35] = 566;
        self.JPFreq[3][40] = 565;
        self.JPFreq[4][74] = 564;
        self.JPFreq[4][24] = 563;
        self.JPFreq[42][59] = 562;
        self.JPFreq[3][7] = 561;
        self.JPFreq[3][71] = 560;
        self.JPFreq[3][12] = 559;
        self.JPFreq[15][75] = 558;
        self.JPFreq[3][20] = 557;
        self.JPFreq[4][39] = 556;
        self.JPFreq[34][69] = 555;
        self.JPFreq[3][28] = 554;
        self.JPFreq[35][24] = 553;
        self.JPFreq[3][82] = 552;
        self.JPFreq[28][47] = 551;
        self.JPFreq[3][67] = 550;
        self.JPFreq[37][16] = 549;
        self.JPFreq[26][93] = 548;
        self.JPFreq[4][1] = 547;
        self.JPFreq[26][85] = 546;
        self.JPFreq[31][14] = 545;
        self.JPFreq[4][3] = 544;
        self.JPFreq[4][72] = 543;
        self.JPFreq[24][51] = 542;
        self.JPFreq[27][51] = 541;
        self.JPFreq[27][49] = 540;
        self.JPFreq[22][77] = 539;
        self.JPFreq[27][10] = 538;
        self.JPFreq[29][68] = 537;
        self.JPFreq[20][35] = 536;
        self.JPFreq[41][11] = 535;
        self.JPFreq[24][70] = 534;
        self.JPFreq[36][61] = 533;
        self.JPFreq[31][23] = 532;
        self.JPFreq[43][16] = 531;
        self.JPFreq[23][68] = 530;
        self.JPFreq[32][15] = 529;
        self.JPFreq[3][32] = 528;
        self.JPFreq[19][53] = 527;
        self.JPFreq[40][83] = 526;
        self.JPFreq[4][14] = 525;
        self.JPFreq[36][9] = 524;
        self.JPFreq[4][73] = 523;
        self.JPFreq[23][10] = 522;
        self.JPFreq[3][63] = 521;
        self.JPFreq[39][14] = 520;
        self.JPFreq[3][78] = 519;
        self.JPFreq[33][47] = 518;
        self.JPFreq[21][39] = 517;
        self.JPFreq[34][46] = 516;
        self.JPFreq[36][75] = 515;
        self.JPFreq[41][92] = 514;
        self.JPFreq[37][93] = 513;
        self.JPFreq[4][34] = 512;
        self.JPFreq[15][86] = 511;
        self.JPFreq[46][1] = 510;
        self.JPFreq[37][65] = 509;
        self.JPFreq[3][62] = 508;
        self.JPFreq[32][73] = 507;
        self.JPFreq[21][65] = 506;
        self.JPFreq[29][75] = 505;
        self.JPFreq[26][51] = 504;
        self.JPFreq[3][34] = 503;
        self.JPFreq[4][10] = 502;
        self.JPFreq[30][22] = 501;
        self.JPFreq[35][73] = 500;
        self.JPFreq[17][82] = 499;
        self.JPFreq[45][8] = 498;
        self.JPFreq[27][73] = 497;
        self.JPFreq[18][55] = 496;
        self.JPFreq[25][2] = 495;
        self.JPFreq[3][26] = 494;
        self.JPFreq[45][46] = 493;
        self.JPFreq[4][22] = 492;
        self.JPFreq[4][40] = 491;
        self.JPFreq[18][10] = 490;
        self.JPFreq[32][9] = 489;
        self.JPFreq[26][49] = 488;
        self.JPFreq[3][47] = 487;
        self.JPFreq[24][65] = 486;
        self.JPFreq[4][76] = 485;
        self.JPFreq[43][67] = 484;
        self.JPFreq[3][9] = 483;
        self.JPFreq[41][37] = 482;
        self.JPFreq[33][68] = 481;
        self.JPFreq[43][31] = 480;
        self.JPFreq[19][55] = 479;
        self.JPFreq[4][30] = 478;
        self.JPFreq[27][33] = 477;
        self.JPFreq[16][62] = 476;
        self.JPFreq[36][35] = 475;
        self.JPFreq[37][15] = 474;
        self.JPFreq[27][70] = 473;
        self.JPFreq[22][71] = 472;
        self.JPFreq[33][45] = 471;
        self.JPFreq[31][78] = 470;
        self.JPFreq[43][59] = 469;
        self.JPFreq[32][19] = 468;
        self.JPFreq[17][28] = 467;
        self.JPFreq[40][28] = 466;
        self.JPFreq[20][93] = 465;
        self.JPFreq[18][15] = 464;
        self.JPFreq[4][23] = 463;
        self.JPFreq[3][23] = 462;
        self.JPFreq[26][64] = 461;
        self.JPFreq[44][92] = 460;
        self.JPFreq[17][27] = 459;
        self.JPFreq[3][56] = 458;
        self.JPFreq[25][38] = 457;
        self.JPFreq[23][31] = 456;
        self.JPFreq[35][43] = 455;
        self.JPFreq[4][54] = 454;
        self.JPFreq[35][19] = 453;
        self.JPFreq[22][47] = 452;
        self.JPFreq[42][0] = 451;
        self.JPFreq[23][28] = 450;
        self.JPFreq[46][33] = 449;
        self.JPFreq[36][85] = 448;
        self.JPFreq[31][12] = 447;
        self.JPFreq[3][76] = 446;
        self.JPFreq[4][75] = 445;
        self.JPFreq[36][56] = 444;
        self.JPFreq[4][64] = 443;
        self.JPFreq[25][77] = 442;
        self.JPFreq[15][52] = 441;
        self.JPFreq[33][73] = 440;
        self.JPFreq[3][55] = 439;
        self.JPFreq[43][82] = 438;
        self.JPFreq[27][82] = 437;
        self.JPFreq[20][3] = 436;
        self.JPFreq[40][51] = 435;
        self.JPFreq[3][17] = 434;
        self.JPFreq[27][71] = 433;
        self.JPFreq[4][52] = 432;
        self.JPFreq[44][48] = 431;
        self.JPFreq[27][2] = 430;
        self.JPFreq[17][39] = 429;
        self.JPFreq[31][8] = 428;
        self.JPFreq[44][54] = 427;
        self.JPFreq[43][18] = 426;
        self.JPFreq[43][77] = 425;
        self.JPFreq[4][61] = 424;
        self.JPFreq[19][91] = 423;
        self.JPFreq[31][13] = 422;
        self.JPFreq[44][71] = 421;
        self.JPFreq[20][0] = 420;
        self.JPFreq[23][87] = 419;
        self.JPFreq[21][14] = 418;
        self.JPFreq[29][13] = 417;
        self.JPFreq[3][58] = 416;
        self.JPFreq[26][18] = 415;
        self.JPFreq[4][47] = 414;
        self.JPFreq[4][18] = 413;
        self.JPFreq[3][53] = 412;
        self.JPFreq[26][92] = 411;
        self.JPFreq[21][7] = 410;
        self.JPFreq[4][37] = 409;
        self.JPFreq[4][63] = 408;
        self.JPFreq[36][51] = 407;
        self.JPFreq[4][32] = 406;
        self.JPFreq[28][73] = 405;
        self.JPFreq[4][50] = 404;
        self.JPFreq[41][60] = 403;
        self.JPFreq[23][1] = 402;
        self.JPFreq[36][92] = 401;
        self.JPFreq[15][41] = 400;
        self.JPFreq[21][71] = 399;
        self.JPFreq[41][30] = 398;
        self.JPFreq[32][76] = 397;
        self.JPFreq[17][34] = 396;
        self.JPFreq[26][15] = 395;
        self.JPFreq[26][25] = 394;
        self.JPFreq[31][77] = 393;
        self.JPFreq[31][3] = 392;
        self.JPFreq[46][34] = 391;
        self.JPFreq[27][84] = 390;
        self.JPFreq[23][8] = 389;
        self.JPFreq[16][0] = 388;
        self.JPFreq[28][80] = 387;
        self.JPFreq[26][54] = 386;
        self.JPFreq[33][18] = 385;
        self.JPFreq[31][20] = 384;
        self.JPFreq[31][62] = 383;
        self.JPFreq[30][41] = 382;
        self.JPFreq[33][30] = 381;
        self.JPFreq[45][45] = 380;
        self.JPFreq[37][82] = 379;
        self.JPFreq[15][33] = 378;
        self.JPFreq[20][12] = 377;
        self.JPFreq[18][5] = 376;
        self.JPFreq[28][86] = 375;
        self.JPFreq[30][19] = 374;
        self.JPFreq[42][43] = 373;
        self.JPFreq[36][31] = 372;
        self.JPFreq[17][93] = 371;
        self.JPFreq[4][15] = 370;
        self.JPFreq[21][20] = 369;
        self.JPFreq[23][21] = 368;
        self.JPFreq[28][72] = 367;
        self.JPFreq[4][20] = 366;
        self.JPFreq[26][55] = 365;
        self.JPFreq[21][5] = 364;
        self.JPFreq[19][16] = 363;
        self.JPFreq[23][64] = 362;
        self.JPFreq[40][59] = 361;
        self.JPFreq[37][26] = 360;
        self.JPFreq[26][56] = 359;
        self.JPFreq[4][12] = 358;
        self.JPFreq[33][71] = 357;
        self.JPFreq[32][39] = 356;
        self.JPFreq[38][40] = 355;
        self.JPFreq[22][74] = 354;
        self.JPFreq[3][25] = 353;
        self.JPFreq[15][48] = 352;
        self.JPFreq[41][82] = 351;
        self.JPFreq[41][9] = 350;
        self.JPFreq[25][48] = 349;
        self.JPFreq[31][71] = 348;
        self.JPFreq[43][29] = 347;
        self.JPFreq[26][80] = 346;
        self.JPFreq[4][5] = 345;
        self.JPFreq[18][71] = 344;
        self.JPFreq[29][0] = 343;
        self.JPFreq[43][43] = 342;
        self.JPFreq[23][81] = 341;
        self.JPFreq[4][42] = 340;
        self.JPFreq[44][28] = 339;
        self.JPFreq[23][93] = 338;
        self.JPFreq[17][81] = 337;
        self.JPFreq[25][25] = 336;
        self.JPFreq[41][23] = 335;
        self.JPFreq[34][35] = 334;
        self.JPFreq[4][53] = 333;
        self.JPFreq[28][36] = 332;
        self.JPFreq[4][41] = 331;
        self.JPFreq[25][60] = 330;
        self.JPFreq[23][20] = 329;
        self.JPFreq[3][43] = 328;
        self.JPFreq[24][79] = 327;
        self.JPFreq[29][41] = 326;
        self.JPFreq[30][83] = 325;
        self.JPFreq[3][50] = 324;
        self.JPFreq[22][18] = 323;
        self.JPFreq[18][3] = 322;
        self.JPFreq[39][30] = 321;
        self.JPFreq[4][28] = 320;
        self.JPFreq[21][64] = 319;
        self.JPFreq[4][68] = 318;
        self.JPFreq[17][71] = 317;
        self.JPFreq[27][0] = 316;
        self.JPFreq[39][28] = 315;
        self.JPFreq[30][13] = 314;
        self.JPFreq[36][70] = 313;
        self.JPFreq[20][82] = 312;
        self.JPFreq[33][38] = 311;
        self.JPFreq[44][87] = 310;
        self.JPFreq[34][45] = 309;
        self.JPFreq[4][26] = 308;
        self.JPFreq[24][44] = 307;
        self.JPFreq[38][67] = 306;
        self.JPFreq[38][6] = 305;
        self.JPFreq[30][68] = 304;
        self.JPFreq[15][89] = 303;
        self.JPFreq[24][93] = 302;
        self.JPFreq[40][41] = 301;
        self.JPFreq[38][3] = 300;
        self.JPFreq[28][23] = 299;
        self.JPFreq[26][17] = 298;
        self.JPFreq[4][38] = 297;
        self.JPFreq[22][78] = 296;
        self.JPFreq[15][37] = 295;
        self.JPFreq[25][85] = 294;
        self.JPFreq[4][9] = 293;
        self.JPFreq[4][7] = 292;
        self.JPFreq[27][53] = 291;
        self.JPFreq[39][29] = 290;
        self.JPFreq[41][43] = 289;
        self.JPFreq[25][62] = 288;
        self.JPFreq[4][48] = 287;
        self.JPFreq[28][28] = 286;
        self.JPFreq[21][40] = 285;
        self.JPFreq[36][73] = 284;
        self.JPFreq[26][39] = 283;
        self.JPFreq[22][54] = 282;
        self.JPFreq[33][5] = 281;
        self.JPFreq[19][21] = 280;
        self.JPFreq[46][31] = 279;
        self.JPFreq[20][64] = 278;
        self.JPFreq[26][63] = 277;
        self.JPFreq[22][23] = 276;
        self.JPFreq[25][81] = 275;
        self.JPFreq[4][62] = 274;
        self.JPFreq[37][31] = 273;
        self.JPFreq[40][52] = 272;
        self.JPFreq[29][79] = 271;
        self.JPFreq[41][48] = 270;
        self.JPFreq[31][57] = 269;
        self.JPFreq[32][92] = 268;
        self.JPFreq[36][36] = 267;
        self.JPFreq[27][7] = 266;
        self.JPFreq[35][29] = 265;
        self.JPFreq[37][34] = 264;
        self.JPFreq[34][42] = 263;
        self.JPFreq[27][15] = 262;
        self.JPFreq[33][27] = 261;
        self.JPFreq[31][38] = 260;
        self.JPFreq[19][79] = 259;
        self.JPFreq[4][31] = 258;
        self.JPFreq[4][66] = 257;
        self.JPFreq[17][32] = 256;
        self.JPFreq[26][67] = 255;
        self.JPFreq[16][30] = 254;
        self.JPFreq[26][46] = 253;
        self.JPFreq[24][26] = 252;
        self.JPFreq[35][10] = 251;
        self.JPFreq[18][37] = 250;
        self.JPFreq[3][19] = 249;
        self.JPFreq[33][69] = 248;
        self.JPFreq[31][9] = 247;
        self.JPFreq[45][29] = 246;
        self.JPFreq[3][15] = 245;
        self.JPFreq[18][54] = 244;
        self.JPFreq[3][44] = 243;
        self.JPFreq[31][29] = 242;
        self.JPFreq[18][45] = 241;
        self.JPFreq[38][28] = 240;
        self.JPFreq[24][12] = 239;
        self.JPFreq[35][82] = 238;
        self.JPFreq[17][43] = 237;
        self.JPFreq[28][9] = 236;
        self.JPFreq[23][25] = 235;
        self.JPFreq[44][37] = 234;
        self.JPFreq[23][75] = 233;
        self.JPFreq[23][92] = 232;
        self.JPFreq[0][24] = 231;
        self.JPFreq[19][74] = 230;
        self.JPFreq[45][32] = 229;
        self.JPFreq[16][72] = 228;
        self.JPFreq[16][93] = 227;
        self.JPFreq[45][13] = 226;
        self.JPFreq[24][8] = 225;
        self.JPFreq[25][47] = 224;
        self.JPFreq[28][26] = 223;
        self.JPFreq[43][81] = 222;
        self.JPFreq[32][71] = 221;
        self.JPFreq[18][41] = 220;
        self.JPFreq[26][62] = 219;
        self.JPFreq[41][24] = 218;
        self.JPFreq[40][11] = 217;
        self.JPFreq[43][57] = 216;
        self.JPFreq[34][53] = 215;
        self.JPFreq[20][32] = 214;
        self.JPFreq[34][43] = 213;
        self.JPFreq[41][91] = 212;
        self.JPFreq[29][57] = 211;
        self.JPFreq[15][43] = 210;
        self.JPFreq[22][89] = 209;
        self.JPFreq[33][83] = 208;
        self.JPFreq[43][20] = 207;
        self.JPFreq[25][58] = 206;
        self.JPFreq[30][30] = 205;
        self.JPFreq[4][56] = 204;
        self.JPFreq[17][64] = 203;
        self.JPFreq[23][0] = 202;
        self.JPFreq[44][12] = 201;
        self.JPFreq[25][37] = 200;
        self.JPFreq[35][13] = 199;
        self.JPFreq[20][30] = 198;
        self.JPFreq[21][84] = 197;
        self.JPFreq[29][14] = 196;
        self.JPFreq[30][5] = 195;
        self.JPFreq[37][2] = 194;
        self.JPFreq[4][78] = 193;
        self.JPFreq[29][78] = 192;
        self.JPFreq[29][84] = 191;
        self.JPFreq[32][86] = 190;
        self.JPFreq[20][68] = 189;
        self.JPFreq[30][39] = 188;
        self.JPFreq[15][69] = 187;
        self.JPFreq[4][60] = 186;
        self.JPFreq[20][61] = 185;
        self.JPFreq[41][67] = 184;
        self.JPFreq[16][35] = 183;
        self.JPFreq[36][57] = 182;
        self.JPFreq[39][80] = 181;
        self.JPFreq[4][59] = 180;
        self.JPFreq[4][44] = 179;
        self.JPFreq[40][54] = 178;
        self.JPFreq[30][8] = 177;
        self.JPFreq[44][30] = 176;
        self.JPFreq[31][93] = 175;
        self.JPFreq[31][47] = 174;
        self.JPFreq[16][70] = 173;
        self.JPFreq[21][0] = 172;
        self.JPFreq[17][35] = 171;
        self.JPFreq[21][67] = 170;
        self.JPFreq[44][18] = 169;
        self.JPFreq[36][29] = 168;
        self.JPFreq[18][67] = 167;
        self.JPFreq[24][28] = 166;
        self.JPFreq[36][24] = 165;
        self.JPFreq[23][5] = 164;
        self.JPFreq[31][65] = 163;
        self.JPFreq[26][59] = 162;
        self.JPFreq[28][2] = 161;
        self.JPFreq[39][69] = 160;
        self.JPFreq[42][40] = 159;
        self.JPFreq[37][80] = 158;
        self.JPFreq[15][66] = 157;
        self.JPFreq[34][38] = 156;
        self.JPFreq[28][48] = 155;
        self.JPFreq[37][77] = 154;
        self.JPFreq[29][34] = 153;
        self.JPFreq[33][12] = 152;
        self.JPFreq[4][65] = 151;
        self.JPFreq[30][31] = 150;
        self.JPFreq[27][92] = 149;
        self.JPFreq[4][2] = 148;
        self.JPFreq[4][51] = 147;
        self.JPFreq[23][77] = 146;
        self.JPFreq[4][35] = 145;
        self.JPFreq[3][13] = 144;
        self.JPFreq[26][26] = 143;
        self.JPFreq[44][4] = 142;
        self.JPFreq[39][53] = 141;
        self.JPFreq[20][11] = 140;
        self.JPFreq[40][33] = 139;
        self.JPFreq[45][7] = 138;
        self.JPFreq[4][70] = 137;
        self.JPFreq[3][49] = 136;
        self.JPFreq[20][59] = 135;
        self.JPFreq[21][12] = 134;
        self.JPFreq[33][53] = 133;
        self.JPFreq[20][14] = 132;
        self.JPFreq[37][18] = 131;
        self.JPFreq[18][17] = 130;
        self.JPFreq[36][23] = 129;
        self.JPFreq[18][57] = 128;
        self.JPFreq[26][74] = 127;
        self.JPFreq[35][2] = 126;
        self.JPFreq[38][58] = 125;
        self.JPFreq[34][68] = 124;
        self.JPFreq[29][81] = 123;
        self.JPFreq[20][69] = 122;
        self.JPFreq[39][86] = 121;
        self.JPFreq[4][16] = 120;
        self.JPFreq[16][49] = 119;
        self.JPFreq[15][72] = 118;
        self.JPFreq[26][35] = 117;
        self.JPFreq[32][14] = 116;
        self.JPFreq[40][90] = 115;
        self.JPFreq[33][79] = 114;
        self.JPFreq[35][4] = 113;
        self.JPFreq[23][33] = 112;
        self.JPFreq[19][19] = 111;
        self.JPFreq[31][41] = 110;
        self.JPFreq[44][1] = 109;
        self.JPFreq[22][56] = 108;
        self.JPFreq[31][27] = 107;
        self.JPFreq[32][18] = 106;
        self.JPFreq[27][32] = 105;
        self.JPFreq[37][39] = 104;
        self.JPFreq[42][11] = 103;
        self.JPFreq[29][71] = 102;
        self.JPFreq[32][58] = 101;
        self.JPFreq[46][10] = 100;
        self.JPFreq[17][30] = 99;
        self.JPFreq[38][15] = 98;
        self.JPFreq[29][60] = 97;
        self.JPFreq[4][11] = 96;
        self.JPFreq[38][31] = 95;
        self.JPFreq[40][79] = 94;
        self.JPFreq[28][49] = 93;
        self.JPFreq[28][84] = 92;
        self.JPFreq[26][77] = 91;
        self.JPFreq[22][32] = 90;
        self.JPFreq[33][17] = 89;
        self.JPFreq[23][18] = 88;
        self.JPFreq[32][64] = 87;
        self.JPFreq[4][6] = 86;
        self.JPFreq[33][51] = 85;
        self.JPFreq[44][77] = 84;
        self.JPFreq[29][5] = 83;
        self.JPFreq[46][25] = 82;
        self.JPFreq[19][58] = 81;
        self.JPFreq[4][46] = 80;
        self.JPFreq[15][71] = 79;
        self.JPFreq[18][58] = 78;
        self.JPFreq[26][45] = 77;
        self.JPFreq[45][66] = 76;
        self.JPFreq[34][10] = 75;
        self.JPFreq[19][37] = 74;
        self.JPFreq[33][65] = 73;
        self.JPFreq[44][52] = 72;
        self.JPFreq[16][38] = 71;
        self.JPFreq[36][46] = 70;
        self.JPFreq[20][26] = 69;
        self.JPFreq[30][37] = 68;
        self.JPFreq[4][58] = 67;
        self.JPFreq[43][2] = 66;
        self.JPFreq[30][18] = 65;
        self.JPFreq[19][35] = 64;
        self.JPFreq[15][68] = 63;
        self.JPFreq[3][36] = 62;
        self.JPFreq[35][40] = 61;
        self.JPFreq[36][32] = 60;
        self.JPFreq[37][14] = 59;
        self.JPFreq[17][11] = 58;
        self.JPFreq[19][78] = 57;
        self.JPFreq[37][11] = 56;
        self.JPFreq[28][63] = 55;
        self.JPFreq[29][61] = 54;
        self.JPFreq[33][3] = 53;
        self.JPFreq[41][52] = 52;
        self.JPFreq[33][63] = 51;
        self.JPFreq[22][41] = 50;
        self.JPFreq[4][19] = 49;
        self.JPFreq[32][41] = 48;
        self.JPFreq[24][4] = 47;
        self.JPFreq[31][28] = 46;
        self.JPFreq[43][30] = 45;
        self.JPFreq[17][3] = 44;
        self.JPFreq[43][70] = 43;
        self.JPFreq[34][19] = 42;
        self.JPFreq[20][77] = 41;
        self.JPFreq[18][83] = 40;
        self.JPFreq[17][15] = 39;
        self.JPFreq[23][61] = 38;
        self.JPFreq[40][27] = 37;
        self.JPFreq[16][48] = 36;
        self.JPFreq[39][78] = 35;
        self.JPFreq[41][53] = 34;
        self.JPFreq[40][91] = 33;
        self.JPFreq[40][72] = 32;
        self.JPFreq[18][52] = 31;
        self.JPFreq[35][66] = 30;
        self.JPFreq[39][93] = 29;
        self.JPFreq[19][48] = 28;
        self.JPFreq[26][36] = 27;
        self.JPFreq[27][25] = 26;
        self.JPFreq[42][71] = 25;
        self.JPFreq[42][85] = 24;
        self.JPFreq[26][48] = 23;
        self.JPFreq[28][15] = 22;
        self.JPFreq[3][66] = 21;
        self.JPFreq[25][24] = 20;
        self.JPFreq[27][43] = 19;
        self.JPFreq[27][78] = 18;
        self.JPFreq[45][43] = 17;
        self.JPFreq[27][72] = 16;
        self.JPFreq[40][29] = 15;
        self.JPFreq[41][0] = 14;
        self.JPFreq[19][57] = 13;
        self.JPFreq[15][59] = 12;
        self.JPFreq[29][29] = 11;
        self.JPFreq[4][25] = 10;
        self.JPFreq[21][42] = 9;
        self.JPFreq[23][35] = 8;
        self.JPFreq[33][1] = 7;
        self.JPFreq[4][57] = 6;
        self.JPFreq[17][60] = 5;
        self.JPFreq[25][19] = 4;
        self.JPFreq[22][65] = 3;
        self.JPFreq[42][29] = 2;
        self.JPFreq[27][66] = 1;
        self.JPFreq[26][89] = 0;
    }
}

// class Encoding {
pub struct Encoding;

impl Encoding {
    // Supported Encoding Types
    // public static int GB2312 = 0;
    pub const GB2312: i32 = 0;

    // public static int GBK = 1;
    pub const GBK: i32 = 1;

    // public static int GB18030 = 2;
    pub const GB18030: i32 = 2;

    // public static int HZ = 3;
    pub const HZ: i32 = 3;

    // public static int BIG5 = 4;
    pub const BIG5: i32 = 4;

    // public static int CNS11643 = 5;
    pub const CNS11643: i32 = 5;

    // public static int UTF8 = 6;
    pub const UTF8: i32 = 6;

    // public static int UTF8T = 7;
    pub const UTF8T: i32 = 7;

    // public static int UTF8S = 8;
    pub const UTF8S: i32 = 8;

    // public static int UNICODE = 9;
    pub const UNICODE: i32 = 9;

    // public static int UNICODET = 10;
    pub const UNICODET: i32 = 10;

    // public static int UNICODES = 11;
    pub const UNICODES: i32 = 11;

    // public static int ISO2022CN = 12;
    pub const ISO2022CN: i32 = 12;

    // public static int ISO2022CN_CNS = 13;
    pub const ISO2022CN_CNS: i32 = 13;

    // public static int ISO2022CN_GB = 14;
    pub const ISO2022CN_GB: i32 = 14;

    // public static int EUC_KR = 15;
    pub const EUC_KR: i32 = 15;

    // public static int CP949 = 16;
    pub const CP949: i32 = 16;

    // public static int ISO2022KR = 17;
    pub const ISO2022KR: i32 = 17;

    // public static int JOHAB = 18;
    pub const JOHAB: i32 = 18;

    // public static int SJIS = 19;
    pub const SJIS: i32 = 19;

    // public static int EUC_JP = 20;
    pub const EUC_JP: i32 = 20;

    // public static int ISO2022JP = 21;
    pub const ISO2022JP: i32 = 21;

    // public static int ASCII = 22;
    pub const ASCII: i32 = 22;

    // public static int OTHER = 23;
    pub const OTHER: i32 = 23;

    // public static int TOTALTYPES = 24;
    pub const TOTALTYPES: i32 = 24;

    // public final static int SIMP = 0;
    pub const SIMP: i32 = 0;

    // public final static int TRAD = 1;
    pub const TRAD: i32 = 1;

    // Names of the encodings as understood by Java
    // public static String[] javaname;
    // javaname = new String[TOTALTYPES];
    // javaname[GB2312] = "GB2312"; javaname[GBK] = "GBK";
    // javaname[GB18030] = "GB18030"; javaname[HZ] = "ASCII"; // What to put here? Sun doesn't support HZ
    // javaname[ISO2022CN_GB] = "ISO2022CN_GB"; javaname[BIG5] = "BIG5";
    // javaname[CNS11643] = "EUC-TW"; javaname[ISO2022CN_CNS] = "ISO2022CN_CNS";
    // javaname[ISO2022CN] = "ISO2022CN"; javaname[UTF8] = "UTF-8";
    // javaname[UTF8T] = "UTF-8"; javaname[UTF8S] = "UTF-8";
    // javaname[UNICODE] = "Unicode"; javaname[UNICODET] = "Unicode";
    // javaname[UNICODES] = "Unicode"; javaname[EUC_KR] = "EUC_KR";
    // javaname[CP949] = "MS949"; javaname[ISO2022KR] = "ISO2022KR";
    // javaname[JOHAB] = "Johab"; javaname[SJIS] = "SJIS";
    // javaname[EUC_JP] = "EUC_JP"; javaname[ISO2022JP] = "ISO2022JP";
    // javaname[ASCII] = "ASCII"; javaname[OTHER] = "ISO8859_1";
    pub const JAVANAME: [&str; 24] = [
        "GB2312",   // [GB2312]
        "GBK",      // [GBK]
        "GB18030",  // [GB18030]
        "ASCII",    // [HZ] // What to put here? Sun doesn't support HZ
        "BIG5",     // [BIG5]
        "EUC-TW",   // [CNS11643]
        "UTF-8",    // [UTF8]
        "UTF-8",    // [UTF8T]
        "UTF-8",    // [UTF8S]
        "Unicode",  // [UNICODE]
        "Unicode",  // [UNICODET]
        "Unicode",  // [UNICODES]
        "ISO2022CN",    // [ISO2022CN]
        "ISO2022CN_CNS", // [ISO2022CN_CNS]
        "ISO2022CN_GB", // [ISO2022CN_GB]
        "EUC_KR",   // [EUC_KR]
        "MS949",    // [CP949]
        "ISO2022KR", // [ISO2022KR]
        "Johab",    // [JOHAB]
        "SJIS",     // [SJIS]
        "EUC_JP",   // [EUC_JP]
        "ISO2022JP", // [ISO2022JP]
        "ASCII",    // [ASCII]
        "ISO8859_1", // [OTHER]
    ];

    // Names of the encodings for human viewing
    // public static String[] nicename;
    // nicename = new String[TOTALTYPES];
    // nicename[GB2312] = "GB-2312"; nicename[GBK] = "GBK";
    // nicename[GB18030] = "GB18030"; nicename[HZ] = "HZ";
    // nicename[ISO2022CN_GB] = "ISO2022CN-GB"; nicename[BIG5] = "Big5";
    // nicename[CNS11643] = "CNS11643"; nicename[ISO2022CN_CNS] = "ISO2022CN-CNS";
    // nicename[ISO2022CN] = "ISO2022 CN"; nicename[UTF8] = "UTF-8";
    // nicename[UTF8T] = "UTF-8 (Trad)"; nicename[UTF8S] = "UTF-8 (Simp)";
    // nicename[UNICODE] = "Unicode"; nicename[UNICODET] = "Unicode (Trad)";
    // nicename[UNICODES] = "Unicode (Simp)"; nicename[EUC_KR] = "EUC-KR";
    // nicename[CP949] = "CP949"; nicename[ISO2022KR] = "ISO 2022 KR";
    // nicename[JOHAB] = "Johab"; nicename[SJIS] = "Shift-JIS";
    // nicename[EUC_JP] = "EUC-JP"; nicename[ISO2022JP] = "ISO 2022 JP";
    // nicename[ASCII] = "ASCII"; nicename[OTHER] = "OTHER";
    pub const NICENAME: [&str; 24] = [
        "GB-2312",  // [GB2312]
        "GBK",      // [GBK]
        "GB18030",  // [GB18030]
        "HZ",       // [HZ]
        "Big5",     // [BIG5]
        "CNS11643", // [CNS11643]
        "UTF-8",    // [UTF8]
        "UTF-8 (Trad)", // [UTF8T]
        "UTF-8 (Simp)", // [UTF8S]
        "Unicode",  // [UNICODE]
        "Unicode (Trad)", // [UNICODET]
        "Unicode (Simp)", // [UNICODES]
        "ISO2022 CN", // [ISO2022CN]
        "ISO2022CN-CNS", // [ISO2022CN_CNS]
        "ISO2022CN-GB", // [ISO2022CN_GB]
        "EUC-KR",   // [EUC_KR]
        "CP949",    // [CP949]
        "ISO 2022 KR", // [ISO2022KR]
        "Johab",    // [JOHAB]
        "Shift-JIS", // [SJIS]
        "EUC-JP",   // [EUC_JP]
        "ISO 2022 JP", // [ISO2022JP]
        "ASCII",    // [ASCII]
        "OTHER",    // [OTHER]
    ];

    // Names of charsets as used in charset parameter of HTML Meta tag
    // public static String[] htmlname;
    // htmlname = new String[TOTALTYPES];
    // htmlname[GB2312] = "GB2312"; htmlname[GBK] = "GBK";
    // htmlname[GB18030] = "GB18030"; htmlname[HZ] = "HZ-GB-2312";
    // htmlname[ISO2022CN_GB] = "ISO-2022-CN-EXT"; htmlname[BIG5] = "BIG5";
    // htmlname[CNS11643] = "EUC-TW"; htmlname[ISO2022CN_CNS] = "ISO-2022-CN-EXT";
    // htmlname[ISO2022CN] = "ISO-2022-CN"; htmlname[UTF8] = "UTF-8";
    // htmlname[UTF8T] = "UTF-8"; htmlname[UTF8S] = "UTF-8";
    // htmlname[UNICODE] = "UTF-16"; htmlname[UNICODET] = "UTF-16";
    // htmlname[UNICODES] = "UTF-16"; htmlname[EUC_KR] = "EUC-KR";
    // htmlname[CP949] = "x-windows-949"; htmlname[ISO2022KR] = "ISO-2022-KR";
    // htmlname[JOHAB] = "x-Johab"; htmlname[SJIS] = "Shift_JIS";
    // htmlname[EUC_JP] = "EUC-JP"; htmlname[ISO2022JP] = "ISO-2022-JP";
    // htmlname[ASCII] = "ASCII"; htmlname[OTHER] = "ISO8859-1";
    pub const HTMLNAME: [&str; 24] = [
        "GB2312",       // [GB2312]
        "GBK",          // [GBK]
        "GB18030",      // [GB18030]
        "HZ-GB-2312",   // [HZ]
        "BIG5",         // [BIG5]
        "EUC-TW",       // [CNS11643]
        "UTF-8",        // [UTF8]
        "UTF-8",        // [UTF8T]
        "UTF-8",        // [UTF8S]
        "UTF-16",       // [UNICODE]
        "UTF-16",       // [UNICODET]
        "UTF-16",       // [UNICODES]
        "ISO-2022-CN",  // [ISO2022CN]
        "ISO-2022-CN-EXT", // [ISO2022CN_CNS]
        "ISO-2022-CN-EXT", // [ISO2022CN_GB]
        "EUC-KR",       // [EUC_KR]
        "x-windows-949", // [CP949]
        "ISO-2022-KR",  // [ISO2022KR]
        "x-Johab",      // [JOHAB]
        "Shift_JIS",    // [SJIS]
        "EUC-JP",       // [EUC_JP]
        "ISO-2022-JP",  // [ISO2022JP]
        "ASCII",        // [ASCII]
        "ISO8859-1",    // [OTHER]
    ];

    // // Constructor
    // public Encoding() {
    //     javaname = new String[TOTALTYPES];
    //     nicename = new String[TOTALTYPES];
    //     htmlname = new String[TOTALTYPES];
    //     // Assign encoding names
    //     javaname[GB2312] = "GB2312";
    //     javaname[GBK] = "GBK";
    //     javaname[GB18030] = "GB18030";
    //     javaname[HZ] = "ASCII"; // What to put here? Sun doesn't support HZ
    //     javaname[ISO2022CN_GB] = "ISO2022CN_GB";
    //     javaname[BIG5] = "BIG5";
    //     javaname[CNS11643] = "EUC-TW";
    //     javaname[ISO2022CN_CNS] = "ISO2022CN_CNS";
    //     javaname[ISO2022CN] = "ISO2022CN";
    //     javaname[UTF8] = "UTF-8";
    //     javaname[UTF8T] = "UTF-8";
    //     javaname[UTF8S] = "UTF-8";
    //     javaname[UNICODE] = "Unicode";
    //     javaname[UNICODET] = "Unicode";
    //     javaname[UNICODES] = "Unicode";
    //     javaname[EUC_KR] = "EUC_KR";
    //     javaname[CP949] = "MS949";
    //     javaname[ISO2022KR] = "ISO2022KR";
    //     javaname[JOHAB] = "Johab";
    //     javaname[SJIS] = "SJIS";
    //     javaname[EUC_JP] = "EUC_JP";
    //     javaname[ISO2022JP] = "ISO2022JP";
    //     javaname[ASCII] = "ASCII";
    //     javaname[OTHER] = "ISO8859_1";
    //     // Assign encoding names
    //     htmlname[GB2312] = "GB2312";
    //     htmlname[GBK] = "GBK";
    //     htmlname[GB18030] = "GB18030";
    //     htmlname[HZ] = "HZ-GB-2312";
    //     htmlname[ISO2022CN_GB] = "ISO-2022-CN-EXT";
    //     htmlname[BIG5] = "BIG5";
    //     htmlname[CNS11643] = "EUC-TW";
    //     htmlname[ISO2022CN_CNS] = "ISO-2022-CN-EXT";
    //     htmlname[ISO2022CN] = "ISO-2022-CN";
    //     htmlname[UTF8] = "UTF-8";
    //     htmlname[UTF8T] = "UTF-8";
    //     htmlname[UTF8S] = "UTF-8";
    //     htmlname[UNICODE] = "UTF-16";
    //     htmlname[UNICODET] = "UTF-16";
    //     htmlname[UNICODES] = "UTF-16";
    //     htmlname[EUC_KR] = "EUC-KR";
    //     htmlname[CP949] = "x-windows-949";
    //     htmlname[ISO2022KR] = "ISO-2022-KR";
    //     htmlname[JOHAB] = "x-Johab";
    //     htmlname[SJIS] = "Shift_JIS";
    //     htmlname[EUC_JP] = "EUC-JP";
    //     htmlname[ISO2022JP] = "ISO-2022-JP";
    //     htmlname[ASCII] = "ASCII";
    //     htmlname[OTHER] = "ISO8859-1";
    //     // Assign Human readable names
    //     nicename[GB2312] = "GB-2312";
    //     nicename[GBK] = "GBK";
    //     nicename[GB18030] = "GB18030";
    //     nicename[HZ] = "HZ";
    //     nicename[ISO2022CN_GB] = "ISO2022CN-GB";
    //     nicename[BIG5] = "Big5";
    //     nicename[CNS11643] = "CNS11643";
    //     nicename[ISO2022CN_CNS] = "ISO2022CN-CNS";
    //     nicename[ISO2022CN] = "ISO2022 CN";
    //     nicename[UTF8] = "UTF-8";
    //     nicename[UTF8T] = "UTF-8 (Trad)";
    //     nicename[UTF8S] = "UTF-8 (Simp)";
    //     nicename[UNICODE] = "Unicode";
    //     nicename[UNICODET] = "Unicode (Trad)";
    //     nicename[UNICODES] = "Unicode (Simp)";
    //     nicename[EUC_KR] = "EUC-KR";
    //     nicename[CP949] = "CP949";
    //     nicename[ISO2022KR] = "ISO 2022 KR";
    //     nicename[JOHAB] = "Johab";
    //     nicename[SJIS] = "Shift-JIS";
    //     nicename[EUC_JP] = "EUC-JP";
    //     nicename[ISO2022JP] = "ISO 2022 JP";
    //     nicename[ASCII] = "ASCII";
    //     nicename[OTHER] = "OTHER";
    // }
}
