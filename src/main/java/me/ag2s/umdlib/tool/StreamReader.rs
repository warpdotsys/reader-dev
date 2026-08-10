// package me.ag2s.umdlib.tool;
//
// import java.io.FileInputStream;
// import java.io.IOException;
// import java.io.InputStream;

pub struct StreamReader<'a> {
    is: &'a mut dyn Read,
    pub offset: i64,
    pub size: i64,
}

impl<'a> StreamReader<'a> {

    pub fn get_offset(&self) -> i64 {
        return self.offset;
    }

    pub fn set_offset(&mut self, offset: i64) {
        self.offset = offset;
    }

    pub fn get_size(&self) -> i64 {
        return self.size;
    }

    pub fn set_size(&mut self, size: i64) {
        self.size = size;
    }

    fn inc_count(&mut self, value: i32) {
        let mut temp = (self.offset + value as i64) as i32;
        if temp < 0 {
            temp = i32::MAX;
        }
        self.offset = temp as i64;
    }

    pub fn new(input_stream: &'a mut dyn Read) -> StreamReader<'a> {
        let is = input_stream;
        StreamReader {
            is,
            offset: 0,
            size: 0,
            //this.size=inputStream.getChannel().size();
        }
    }

    pub fn read_uint8(&mut self) -> i16 {
        let mut b = vec![0u8; 1];
        self.is.read(&mut b);
        self.inc_count(1);
        return (b[0] & 0xFF) as i16;
    }

    pub fn read_byte(&mut self) -> u8 {
        let mut b = vec![0u8; 1];
        self.is.read(&mut b);
        self.inc_count(1);
        return b[0];
    }

    pub fn read_bytes(&mut self, len: usize) -> Vec<u8> {
        if len < 1 {
            println!("{}", len);
            panic!("Length must > 0: {}", len);
        }
        let mut b = vec![0u8; len];
        self.is.read(&mut b);
        self.inc_count(len as i32);
        return b;
    }

    pub fn read_hex(&mut self, len: usize) -> String {
        if len < 1 {
            println!("{}", len);
            panic!("Length must > 0: {}", len);
        }
        let mut b = vec![0u8; len];
        self.is.read(&mut b);
        self.inc_count(len as i32);
        return UmdUtils::to_hex(&b);
    }

    pub fn read_short(&mut self) -> i16 {
        let mut b = vec![0u8; 2];
        self.is.read(&mut b);
        self.inc_count(2);
        let x = (((b[0] & 0xFF) as i16) << 8) | (((b[1] & 0xFF) as i16) << 0);
        return x;
    }

    pub fn read_short_le(&mut self) -> i16 {
        let mut b = vec![0u8; 2];
        self.is.read(&mut b);
        self.inc_count(2);
        let x = (((b[1] & 0xFF) as i16) << 8) | (((b[0] & 0xFF) as i16) << 0);
        return x;
    }

    pub fn read_int(&mut self) -> i32 {
        let mut b = vec![0u8; 4];
        self.is.read(&mut b);
        self.inc_count(4);
        let x = ((b[0] & 0xFF) << 24) | ((b[1] & 0xFF) << 16) |
            ((b[2] & 0xFF) << 8) | ((b[3] & 0xFF) << 0);
        return x;
    }

    pub fn read_int_le(&mut self) -> i32 {
        let mut b = vec![0u8; 4];
        self.is.read(&mut b);
        self.inc_count(4);
        let x = ((b[3] & 0xFF) << 24) | ((b[2] & 0xFF) << 16) |
            ((b[1] & 0xFF) << 8) | ((b[0] & 0xFF) << 0);
        return x;
    }

    pub fn skip(&mut self, len: usize) {
        self.read_bytes(len);
    }

    pub fn read(&mut self, b: &mut [u8]) -> &[u8] {
        self.is.read(b);
        self.inc_count(b.len() as i32);
        return b;
    }

    pub fn read_range(&mut self, b: &mut [u8], off: usize, len: usize) -> &[u8] {
        self.is.read_range(b, off, len);
        self.inc_count(len as i32);
        return b;
    }
}
