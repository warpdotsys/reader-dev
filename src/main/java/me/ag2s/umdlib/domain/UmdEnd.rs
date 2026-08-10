// package me.ag2s.umdlib.domain;
//
// import java.io.IOException;
//
// import me.ag2s.umdlib.tool.WrapOutputStream;

/**
 * End part of UMD book, nothing to be special
 * 
 * @author Ray Liang (liangguanhui@qq.com)
 * 2009-12-20
 */
pub struct UmdEnd;

impl UmdEnd {

    pub fn new() -> Self {
        UmdEnd
    }

    pub fn build_end(&self, wos: &mut WrapOutputStream) {
        wos.write_bytes(&[b'#', 0x0C, 0, 0x01, 0x09]);
        wos.write_int(wos.get_written() + 4);
    }
}
