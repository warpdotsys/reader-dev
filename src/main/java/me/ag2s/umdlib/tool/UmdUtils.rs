// package me.ag2s.umdlib.tool;
//
// import java.io.BufferedInputStream;
// import java.io.BufferedOutputStream;
// import java.io.ByteArrayInputStream;
// import java.io.ByteArrayOutputStream;
// import java.io.File;
// import java.io.FileInputStream;
// import java.io.FileOutputStream;
// import java.io.IOException;
// import java.util.Random;
// import java.util.zip.InflaterInputStream;

pub struct UmdUtils;

impl UmdUtils {

    const EOF: i32 = -1;
    const BUFFER_SIZE: usize = 8 * 1024;

    /**
     * 将字符串编码成Unicode形式的byte[]
     * @param s 要编码的字符串
     * @return 编码好的byte[]
     */
    pub fn string_to_unicode_bytes(s: &str) -> Vec<u8> {
        // if (s == null) {
        //     throw new NullPointerException();
        // }

        let len = s.chars().count();
        let mut ret = vec![0u8; len * 2];
        let mut a: i32;
        let mut b: i32;
        let mut c: i32;
        let chars: Vec<char> = s.chars().collect();
        for i in 0..len {
            c = chars[i] as i32;
            a = c >> 8;
            b = c & 0xFF;
            if a < 0 {
                a += 0xFF;
            }
            if b < 0 {
                b += 0xFF;
            }
            ret[i * 2] = b as u8;
            ret[i * 2 + 1] = a as u8;
        }
        return ret;
    }

    /**
     * 将编码成Unicode形式的byte[]解码成原始字符串
     * @param bytes 编码成Unicode形式的byte[]
     * @return 原始字符串
     */
    pub fn unicode_bytes_to_string(bytes: &[u8]) -> String {
        let s_len = bytes.len() / 2;
        let mut sb = String::new();
        let mut a: i32;
        let mut b: i32;
        let mut c: i32;
        for i in 0..s_len {
            a = bytes[i * 2 + 1] as i32;
            b = bytes[i * 2] as i32;
            c = (a & 0xff) << 8 | (b & 0xff);
            if c < 0 {
                c += 0xffff;
            }
            // char[] c1 = Character.toChars(c)
            sb.push(char::from_u32(c as u32).unwrap());
        }
        return sb;
    }

    /**
     * 将byte[]转化成Hex形式
     * @param bArr byte[]
     * @return 目标HEX字符串
     */
    pub fn to_hex(b_arr: &[u8]) -> String {
        let mut sb = String::new();
        let mut s_tmp: String;

        for i in 0..b_arr.len() {
            s_tmp = format!("{:x}", 0xFF & b_arr[i]);
            if s_tmp.len() < 2 {
                sb.push('0');
            }
            sb.push_str(&s_tmp.to_uppercase());
        }

        return sb;
    }

    /**
     * 解压缩zip的byte[]
     * @param compress zippered byte[]
     * @return decompressed byte[]
     * @throws Exception 解码时失败时
     */
    pub fn decompress(compress: &[u8]) -> Vec<u8> {
        let bais = ByteArrayInputStream::new(compress);
        let mut iis = InflaterInputStream::new(bais);
        let mut baos = ByteArrayOutputStream::new();
        let mut c = 0;
        let mut buf = vec![0u8; Self::BUFFER_SIZE];
        loop {
            c = iis.read(&mut buf);

            if c == Self::EOF {
                break;
            }
            baos.write_range(&buf, 0, c as usize);
        }
        baos.flush();
        return baos.to_byte_array();
    }

    pub fn save_file(f: &File, content: &[u8]) {
        let mut fos = FileOutputStream::new(f);
        // try {
        let mut bos = BufferedOutputStream::new(&mut fos);
        bos.write(content);
        bos.flush();
        // } finally {
        //     fos.close();
        // }
    }

    pub fn read_file(f: &File) -> Vec<u8> {
        let mut fis = FileInputStream::new(f);
        // try {
        let mut baos = ByteArrayOutputStream::new();
        let mut bis = BufferedInputStream::new(&mut fis);
        let mut ch: i32;
        loop {
            ch = bis.read();
            if !(ch >= 0) {
                break;
            }
            baos.write_byte(ch as u8);
        }
        baos.flush();
        return baos.to_byte_array();
        // } finally {
        //     fis.close();
        // }
    }

    pub fn gen_random_bytes(len: usize) -> Vec<u8> {
        if len <= 0 {
            panic!("Length must > 0: {}", len);
        }
        let mut ret = vec![0u8; len];
        for i in 0..ret.len() {
            ret[i] = RANDOM.next_u32() as u8;
        }
        return ret;
    }
}

// private static Random random = new Random();
static RANDOM: std::sync::Mutex<Random> = std::sync::Mutex::new(Random::new());
