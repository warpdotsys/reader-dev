use crate::prelude::*;
use std::io::Read;
// package me.ag2s.umdlib.umd;
//
// import java.io.IOException;
// import java.io.InputStream;
//
//
// import me.ag2s.umdlib.domain.UmdBook;
// import me.ag2s.umdlib.domain.UmdCover;
// import me.ag2s.umdlib.domain.UmdHeader;
// import me.ag2s.umdlib.tool.StreamReader;
// import me.ag2s.umdlib.tool.UmdUtils;

/**
 * UMD格式的电子书解析
 * 格式规范参考：
 * http://blog.sina.com.cn/s/blog_7c8dc2d501018o5d.html
 * http://blog.sina.com.cn/s/blog_7c8dc2d501018o5l.html
 *
 */

pub struct UmdReader<'a> {
    pub book: UmdBook,
    // fix: new() 无法为 &'a mut dyn Read 提供空初值，改为 Option，read() 时再填入
    pub input_stream: Option<&'a mut dyn Read>,
    pub _additional_check_number: i32,
    pub _total_content_len: i32,
    pub end: bool,
}

impl<'a> UmdReader<'a> {

    pub fn new() -> Self {
        UmdReader {
            book: UmdBook::new(),
            input_stream: None,
            _additional_check_number: 0,
            _total_content_len: 0,
            end: false,
        }
    }

    pub fn read(&mut self, input_stream: &'a mut dyn Read) -> UmdBook {
        self.book = UmdBook::new();
        let mut reader = StreamReader::new(input_stream);
        let mut umd_header = UmdHeader::new();
        if reader.read_int_le() != (0xde9a9b89u32 as i32) {
            panic!("Wrong header");
        }
        let mut num1: i16 = -1;
        let mut ch = reader.read_byte();
        while ch == 35 {
            //int num2=reader.readByte();
            let mut seg_type = reader.read_short_le();
            let seg_flag = reader.read_byte();
            let len = (reader.read_uint8() - 5) as i16;

            println!("块标识:{}", seg_type);
            //short length1 = reader.readByte();
            self.read_section(seg_type, seg_flag, len, &mut reader, &mut umd_header);

            if seg_type as i32 == 241 || seg_type as i32 == 10 {
                seg_type = num1;
            }
            loop {
                ch = reader.read_byte();
                if !(ch == 36) {
                    break;
                }
                //int num3 = reader.readByte();
                println!("{}", ch);
                let additional_check_number = reader.read_int_le();
                let length2 = (reader.read_int_le() - 9);
                self.read_additional_section(seg_type, additional_check_number, length2, &mut reader);
            }
            num1 = seg_type;
        }
        // fix: Java 中 book 与 umdHeader 为同一引用，解析完成后一次性写入，语义等价
        self.book.set_header(umd_header);
        println!("{}", self.book.get_header().to_string());
        // fix: UmdBook 无 Clone，解析结果用 mem::replace 取出（self.book 位于 &mut self 后）
        return std::mem::replace(&mut self.book, UmdBook::new());
    }

    fn read_additional_section(&mut self, seg_type: i16, additional_check_number: i32, length: i32, reader: &mut StreamReader) {
        match seg_type {
            14 => {
                //this._TotalImageList.Add((object) Image.FromStream((Stream) new MemoryStream(reader.ReadBytes((int) length))));
            }
            15 => {
                //this._TotalImageList.Add((object) Image.FromStream((Stream) new MemoryStream(reader.ReadBytes((int) length))));
            }
            129 => {
                reader.read_bytes(length as usize);
            }
            130 => {
                //byte[] covers = reader.readBytes(length);
                self.book.set_cover(UmdCover::with_data(reader.read_bytes(length as usize)));
                //this._Book.Cover = BitmapImage.FromStream((Stream) new MemoryStream(reader.ReadBytes((int) length)));
            }
            131 => {
                println!("{}", length / 4);
                self.book.set_num(length / 4);
                for _i in 0..length / 4 {
                    self.book.chapters.add_content_length(reader.read_int_le());
                }
            }
            132 => {
                //System.out.println(length/4);
                println!("{}", self._additional_check_number);
                println!("{}", additional_check_number);
                if self._additional_check_number != additional_check_number {
                    println!("{}", length);
                    self.book.chapters.contents.write(&UmdUtils::decompress(&reader.read_bytes(length as usize)));
                    self.book.chapters.contents.flush();
                } else {
                    for _i in 0..self.book.get_num() {
                        let len = reader.read_uint8();
                        let title = reader.read_bytes(len as usize);
                        //System.out.println(UmdUtils.unicodeBytesToString(title));
                        self.book.chapters.add_title_bytes(title);
                    }
                }
            }
            _ => {
                /*Console.WriteLine("未知内容");
                Console.WriteLine("Seg Type = " + (object) segType);
                Console.WriteLine("Seg Len = " + (object) length);
                Console.WriteLine("content = " + (object) reader.ReadBytes((int) length));*/
            }
        }
    }

    pub fn read_section(&mut self, seg_type: i16, seg_flag: u8, length: i16, reader: &mut StreamReader, header: &mut UmdHeader) {
        match seg_type {
            1 => {
                //umd文件头 DCTS_CMD_ID_VERSION
                header.set_umd_type(reader.read_byte());
                reader.read_bytes(2); //Random 2
                println!("UMD文件类型:{}", header.get_umd_type());
            }
            2 => {
                //文件标题 DCTS_CMD_ID_TITLE
                header.set_title(UmdUtils::unicode_bytes_to_string(&reader.read_bytes(length as usize)));
                println!("文件标题:{}", header.get_title());
            }
            3 => {
                //作者
                header.set_author(UmdUtils::unicode_bytes_to_string(&reader.read_bytes(length as usize)));
                println!("作者:{}", header.get_author());
            }
            4 => {
                //年
                header.set_year(UmdUtils::unicode_bytes_to_string(&reader.read_bytes(length as usize)));
                println!("年:{}", header.get_year());
            }
            5 => {
                //月
                header.set_month(UmdUtils::unicode_bytes_to_string(&reader.read_bytes(length as usize)));
                println!("月:{}", header.get_month());
            }
            6 => {
                //日
                header.set_day(UmdUtils::unicode_bytes_to_string(&reader.read_bytes(length as usize)));
                println!("日:{}", header.get_day());
            }
            7 => {
                //小说类型
                header.set_book_type(UmdUtils::unicode_bytes_to_string(&reader.read_bytes(length as usize)));
                println!("小说类型:{}", header.get_book_type());
            }
            8 => {
                //出版商
                header.set_book_man(UmdUtils::unicode_bytes_to_string(&reader.read_bytes(length as usize)));
                println!("出版商:{}", header.get_book_man());
            }
            9 => {
                // 零售商
                header.set_shop_keeper(UmdUtils::unicode_bytes_to_string(&reader.read_bytes(length as usize)));
                println!("零售商:{}", header.get_shop_keeper());
            }
            10 => {
                //CONTENT ID
                println!("CONTENT ID:{}", reader.read_hex(length as usize));
            }
            11 => {
                //内容长度 DCTS_CMD_ID_FILE_LENGTH
                self._total_content_len = reader.read_int_le();
                self.book.chapters.set_total_content_len(self._total_content_len);
                println!("内容长度:{}", self._total_content_len);
            }
            12 => {
                //UMD文件结束
                self.end = true;
                let num2 = reader.read_int_le();
                println!("整个文件长度{}", num2);
            }
            13 => {
            }
            14 => {
                let num3 = reader.read_byte() as i32;
                let _ = num3;
            }
            15 => {
                reader.read_bytes(length as usize);
            }
            129 | 131 => {
                //正文
                //章节偏移
                self._additional_check_number = reader.read_int_le();
                println!("章节偏移:{}", self._additional_check_number);
            }
            132 => {
                //章节标题，正文
                self._additional_check_number = reader.read_int_le();
                println!("章节标题，正文:{}", self._additional_check_number);
            }
            130 => {
                //封面（jpg）
                let num4 = reader.read_byte() as i32;
                let _ = num4;
                self._additional_check_number = reader.read_int_le();
            }
            135 => {
                //页面偏移（Page Offset）
                reader.read_uint8(); //fontSize 一字节 字体大小
                reader.read_uint8(); //screenWidth 屏幕宽度
                reader.read_bytes(4); //BlockRandom 指向一个页面偏移数据块
            }
            240 => {
                //CDS KEY
            }
            241 => {
                //许可证(LICENCE KEY)
                //System.out.println("整个文件长度" + length);
                println!("许可证(LICENCE KEY):{}", reader.read_hex(16));
            }
            _ => {
                if length > 0 {
                    let num_array = reader.read_bytes(length as usize);
                    let _ = num_array;
                }
            }
        }
    }
}

impl std::fmt::Display for UmdReader<'_> {
    // @Override
    // public String toString() {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // fix: UmdBook 无 Display 实现，改用其 header 展示（UmdHeader 已实现 Display）
        return write!(f, "UmdReader{{book={}}}", self.book.get_header());
    }
}
