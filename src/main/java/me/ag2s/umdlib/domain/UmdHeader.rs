// package me.ag2s.umdlib.domain;
//
//
// import java.io.IOException;
//
// import me.ag2s.umdlib.tool.UmdUtils;
// import me.ag2s.umdlib.tool.WrapOutputStream;

/**
 * Header of UMD file.
 * It includes a lot of properties of header.
 * All the properties are String type.
 * 
 * @author Ray Liang (liangguanhui@qq.com)
 * 2009-12-20
 */
pub struct UmdHeader {
    pub umd_type: u8,
    pub title: String,
    pub author: String,
    pub year: String,
    pub month: String,
    pub day: String,
    pub book_type: String,
    pub book_man: String,
    pub shop_keeper: String,
}

impl UmdHeader {

    // private final static byte B_type_umd = (byte) 0x01;
    const B_TYPE_UMD: u8 = 0x01;
    // private final static byte B_type_title = (byte) 0x02;
    const B_TYPE_TITLE: u8 = 0x02;
    // private final static byte B_type_author = (byte) 0x03;
    const B_TYPE_AUTHOR: u8 = 0x03;
    // private final static byte B_type_year = (byte) 0x04;
    const B_TYPE_YEAR: u8 = 0x04;
    // private final static byte B_type_month = (byte) 0x05;
    const B_TYPE_MONTH: u8 = 0x05;
    // private final static byte B_type_day = (byte) 0x06;
    const B_TYPE_DAY: u8 = 0x06;
    // private final static byte B_type_bookType = (byte) 0x07;
    const B_TYPE_BOOK_TYPE: u8 = 0x07;
    // private final static byte B_type_bookMan = (byte) 0x08;
    const B_TYPE_BOOK_MAN: u8 = 0x08;
    // private final static byte B_type_shopKeeper = (byte) 0x09;
    const B_TYPE_SHOP_KEEPER: u8 = 0x09;

    pub fn new() -> Self {
        UmdHeader {
            umd_type: 0,
            title: String::new(),
            author: String::new(),
            year: String::new(),
            month: String::new(),
            day: String::new(),
            book_type: String::new(),
            book_man: String::new(),
            shop_keeper: String::new(),
        }
    }

    pub fn get_umd_type(&self) -> u8 {
        return self.umd_type;
    }

    pub fn set_umd_type(&mut self, umd_type: u8) {
        self.umd_type = umd_type;
    }

    pub fn build_header(&self, wos: &mut WrapOutputStream) {
        wos.write_bytes(&[0x89, 0x9b, 0x9a, 0xde]); // UMD file type flags
        wos.write_byte(b'#');
        wos.write_bytes(&[0x01, 0x00, 0x00, 0x08]); // Unknown
        wos.write_byte(0x01); //0x01 is text type; while 0x02 is Image type.
        wos.write_bytes(&UmdUtils::gen_random_bytes(2)); //random number

        // start properties output
        self.build_type(wos, Self::B_TYPE_TITLE, &self.title);
        self.build_type(wos, Self::B_TYPE_AUTHOR, &self.author);
        self.build_type(wos, Self::B_TYPE_YEAR, &self.year);
        self.build_type(wos, Self::B_TYPE_MONTH, &self.month);
        self.build_type(wos, Self::B_TYPE_DAY, &self.day);
        self.build_type(wos, Self::B_TYPE_BOOK_TYPE, &self.book_type);
        self.build_type(wos, Self::B_TYPE_BOOK_MAN, &self.book_man);
        self.build_type(wos, Self::B_TYPE_SHOP_KEEPER, &self.shop_keeper);
    }

    pub fn build_type(&self, wos: &mut WrapOutputStream, type_: u8, content: &str) {
        if content.is_empty() {
            return;
        }

        wos.write_bytes(&[b'#', type_, 0, 0]);

        let temp = UmdUtils::string_to_unicode_bytes(content);
        wos.write_byte((temp.len() + 5) as u8);
        wos.write(&temp);
    }

    pub fn get_title(&self) -> &str {
        return &self.title;
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn get_author(&self) -> &str {
        return &self.author;
    }

    pub fn set_author(&mut self, author: String) {
        self.author = author;
    }

    pub fn get_book_man(&self) -> &str {
        return &self.book_man;
    }

    pub fn set_book_man(&mut self, book_man: String) {
        self.book_man = book_man;
    }

    pub fn get_shop_keeper(&self) -> &str {
        return &self.shop_keeper;
    }

    pub fn set_shop_keeper(&mut self, shop_keeper: String) {
        self.shop_keeper = shop_keeper;
    }

    pub fn get_year(&self) -> &str {
        return &self.year;
    }

    pub fn set_year(&mut self, year: String) {
        self.year = year;
    }

    pub fn get_month(&self) -> &str {
        return &self.month;
    }

    pub fn set_month(&mut self, month: String) {
        self.month = month;
    }

    pub fn get_day(&self) -> &str {
        return &self.day;
    }

    pub fn set_day(&mut self, day: String) {
        self.day = day;
    }

    pub fn get_book_type(&self) -> &str {
        return &self.book_type;
    }

    pub fn set_book_type(&mut self, book_type: String) {
        self.book_type = book_type;
    }
}

impl std::fmt::Display for UmdHeader {
    // @Override
    // public String toString() {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        return write!(f, "UmdHeader{{\
            umdType={}, title='{}', author='{}', year='{}', month='{}', day='{}', bookType='{}', bookMan='{}', shopKeeper='{}'\
        }}",
            self.umd_type, self.title, self.author, self.year, self.month, self.day,
            self.book_type, self.book_man, self.shop_keeper);
    }
}
