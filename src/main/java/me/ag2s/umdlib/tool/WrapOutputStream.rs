use crate::prelude::*;
// package me.ag2s.umdlib.tool;
//
// import java.io.IOException;
// import java.io.OutputStream;

pub struct WrapOutputStream<'a> {
    os: &'a mut dyn Write,
    written: i32,
}

impl<'a> WrapOutputStream<'a> {

    pub fn new(os: &'a mut dyn Write) -> Self {
        WrapOutputStream { os, written: 0 }
    }

    fn inc_count(&mut self, value: i32) {
        let mut temp = self.written + value;
        if temp < 0 {
            temp = i32::MAX;
        }
        self.written = temp;
    }

    // it is different from the writeInt of DataOutputStream
    pub fn write_int(&mut self, v: i32) {
        // fix: Java `>>>`（无符号右移）→ Rust `(v as u32) >> n`，`& 0xFF` 取低 8 位（小端顺序，与 UMD 格式一致）
        self.os.write(&[(((v as u32) >> 0) & 0xFF) as u8]);
        self.os.write(&[(((v as u32) >> 8) & 0xFF) as u8]);
        self.os.write(&[(((v as u32) >> 16) & 0xFF) as u8]);
        self.os.write(&[(((v as u32) >> 24) & 0xFF) as u8]);
        self.inc_count(4);
    }

    pub fn write_byte(&mut self, b: u8) {
        self.write(&[b]);
    }

    pub fn write_byte_int(&mut self, n: i32) {
        self.write(&[n as u8]);
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.write(bytes);
    }

    pub fn write_bytes_ints(&mut self, vals: &[i32]) {
        for v in vals {
            self.write(&[*v as u8]);
        }
    }

    pub fn write_range(&mut self, b: &[u8], off: usize, len: usize) {
        self.os.write_range(b, off, len);
        self.inc_count(len as i32);
    }

    pub fn write(&mut self, b: &[u8]) {
        self.os.write(b);
        self.inc_count(b.len() as i32);
    }

    pub fn write_byte_value(&mut self, b: i32) {
        self.os.write_byte(b);
        self.inc_count(1);
    }

    /////////////////////////////////////////////////

    pub fn close(&mut self) {
        self.os.close();
    }

    pub fn flush(&mut self) {
        self.os.flush();
    }

    pub fn get_written(&self) -> i32 {
        return self.written;
    }
}
