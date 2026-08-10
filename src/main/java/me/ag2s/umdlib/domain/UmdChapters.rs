// package me.ag2s.umdlib.domain;
//
// import java.io.ByteArrayOutputStream;
// import java.io.File;
// import java.io.IOException;
// import java.util.ArrayList;
// import java.util.Arrays;
// import java.util.List;
// import java.util.zip.DeflaterOutputStream;
//
// import me.ag2s.umdlib.tool.UmdUtils;
// import me.ag2s.umdlib.tool.WrapOutputStream;

/**
 * It includes all titles and contents of each chapter in the UMD file.
 * And the content has been compressed by zlib.
 * 
 * @author Ray Liang (liangguanhui@qq.com)
 * 2009-12-20
 */
pub struct UmdChapters {
    // private static final int DEFAULT_CHUNK_INIT_SIZE = 32768;
    default_chunk_init_size: i32,

    total_content_len: i32,

    pub titles: Vec<Vec<u8>>,
    pub content_lengths: Vec<i32>,
    pub contents: ByteArrayOutputStream,
}

impl UmdChapters {

    pub fn new() -> Self {
        UmdChapters {
            default_chunk_init_size: 32768,
            total_content_len: 0,
            titles: Vec::new(),
            content_lengths: Vec::new(),
            contents: ByteArrayOutputStream::new(),
        }
    }

    pub fn get_titles(&self) -> &Vec<Vec<u8>> {
        return &self.titles;
    }

    pub fn add_title(&mut self, s: &str) {
        self.titles.push(UmdUtils::string_to_unicode_bytes(s));
    }

    pub fn add_title_bytes(&mut self, s: Vec<u8>) {
        self.titles.push(s);
    }

    pub fn add_content_length(&mut self, integer: i32) {
        self.content_lengths.push(integer);
    }

    pub fn get_content_length(&self, index: usize) -> i32 {
        return self.content_lengths[index];
    }

    pub fn get_content(&self, index: usize) -> Vec<u8> {
        let st = self.content_lengths[index];
        let b = self.contents.to_byte_array();
        let end = if index + 1 < self.content_lengths.len() {
            self.content_lengths[index + 1]
        } else {
            self.get_total_content_len()
        };
        println!("总长度:{}", self.contents.size());
        println!("起始值:{}", st);
        println!("结束值:{}", end);
        let mut b_ar = vec![0u8; (end - st) as usize];
        b_ar.copy_from_slice(&b[st as usize..end as usize]);
        return b_ar;
    }

    pub fn get_content_string(&self, index: usize) -> String {
        return UmdUtils::unicode_bytes_to_string(self.get_content(index)).replace(0x2029 as char, '\n');
    }

    pub fn get_title(&self, index: usize) -> String {
        return UmdUtils::unicode_bytes_to_string(self.titles[index].clone());
    }

    pub fn build_chapters(&self, wos: &mut WrapOutputStream) {
        self.write_chapters_head(wos);
        self.write_chapters_content_offset(wos);
        self.write_chapters_titles(wos);
        self.write_chapters_chunks(wos);
    }

    fn write_chapters_head(&self, wos: &mut WrapOutputStream) {
        wos.write_bytes(&[b'#', 0x0b, 0, 0, 0x09]);
        wos.write_int(self.contents.size() as i32);
    }

    fn write_chapters_content_offset(&self, wos: &mut WrapOutputStream) {
        wos.write_bytes(&[b'#', 0x83, 0, 0, 0x09]);
        let rb = UmdUtils::gen_random_bytes(4);
        wos.write_bytes(&rb); //random numbers
        wos.write(b'$');
        wos.write_bytes(&rb); //random numbers

        wos.write_int(self.content_lengths.len() as i32 * 4 + 9);  // about the count of chapters
        let mut offset = 0;
        for n in &self.content_lengths {
            wos.write_int(offset);
            offset += n;
        }
    }

    fn write_chapters_titles(&self, wos: &mut WrapOutputStream) {
        wos.write_bytes(&[b'#', 0x84, 0, 0x01, 0x09]);
        let rb = UmdUtils::gen_random_bytes(4);
        wos.write_bytes(&rb); //random numbers
        wos.write(b'$');
        wos.write_bytes(&rb); //random numbers

        let mut total_titles_len = 0;
        for t in &self.titles {
            total_titles_len += t.len();
        }

        // about the length of the titles
        wos.write_int((total_titles_len + self.titles.len() + 9) as i32);

        for t in &self.titles {
            wos.write_byte(t.len() as u8);
            wos.write(t);
        }
    }

    fn write_chapters_chunks(&self, wos: &mut WrapOutputStream) {
        let all_contents = self.contents.to_byte_array();

        let zero16 = vec![0u8; 16];

        // write each package of content
        let mut start_pos = 0;
        let mut len = 0;
        let mut left = 0;
        let mut chunk_cnt = 0;
        let mut bos = ByteArrayOutputStream::with_capacity(self.default_chunk_init_size + 256);
        let mut chunk_rb_list = Vec::<Vec<u8>>::new();

        while start_pos < all_contents.len() {
            left = all_contents.len() - start_pos;
            len = if self.default_chunk_init_size < left as i32 {
                self.default_chunk_init_size
            } else {
                left as i32
            };

            bos.reset();
            let mut zos = DeflaterOutputStream::new(&mut bos);
            zos.write(&all_contents[start_pos..start_pos + len as usize]);
            zos.close();
            let chunk = bos.to_byte_array();

            let rb = UmdUtils::gen_random_bytes(4);
            wos.write_byte(b'$');
            wos.write_bytes(&rb);  // 4 random
            chunk_rb_list.push(rb);
            wos.write_int(chunk.len() as i32 + 9);
            wos.write(&chunk);

            // end of each chunk
            wos.write_bytes(&[b'#', 0xF1, 0, 0, 0x15]);
            wos.write(&zero16);

            start_pos += len as usize;
            chunk_cnt += 1;
        }

        // end of all chunks
        wos.write_bytes(&[b'#', 0x81, 0, 0x01, 0x09]);
        wos.write_bytes(&[0, 0, 0, 0]); //random numbers
        wos.write(b'$');
        wos.write_bytes(&[0, 0, 0, 0]); //random numbers
        wos.write_int(chunk_cnt * 4 + 9);
        let mut i = chunk_cnt - 1;
        loop {
            // random. They are as the same as random numbers in the begin of each chunk
            // use desc order to output these random
            wos.write_bytes(&chunk_rb_list[i]);
            if i == 0 {
                break;
            }
            i -= 1;
        }
    }

    pub fn add_chapter(&mut self, title: &str, content: &str) {
        self.titles.push(UmdUtils::string_to_unicode_bytes(title));
        let b = UmdUtils::string_to_unicode_bytes(content);
        self.content_lengths.push(b.len() as i32);
        self.contents.write(&b);
    }

    pub fn add_file(&mut self, f: &File, title: &str) {
        let temp = UmdUtils::read_file(f);
        let s = String::from_utf8_lossy(&temp).to_string();
        self.add_chapter(title, &s);
    }

    pub fn add_file_auto(&mut self, f: &File) {
        let mut s = f.get_name();
        let idx = s.rfind('.').map(|i| i as i32).unwrap_or(-1);
        if idx >= 0 {
            s = s[..idx as usize].to_string();
        }
        self.add_file(f, &s);
    }

    pub fn clear_chapters(&mut self) {
        self.titles.clear();
        self.content_lengths.clear();
        self.contents.reset();
    }

    pub fn get_total_content_len(&self) -> i32 {
        return self.total_content_len;
    }

    pub fn set_total_content_len(&mut self, total_content_len: i32) {
        self.total_content_len = total_content_len;
    }
}
