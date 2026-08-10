// package me.ag2s.umdlib.domain;
//
// import java.io.IOException;
// import java.io.OutputStream;
//
// import me.ag2s.umdlib.tool.WrapOutputStream;

pub struct UmdBook {
    pub num: i32,

    /** Header Part of UMD book */
    pub header: UmdHeader,
    /**
     * Detail chapters Part of UMD book
     * (include Titles & Contents of each chapter)
     */
    pub chapters: UmdChapters,

    /** Cover Part of UMD book (for example, and JPEG file) */
    pub cover: UmdCover,

    /** End Part of UMD book */
    pub end: UmdEnd,
}

impl UmdBook {

    pub fn new() -> Self {
        UmdBook {
            num: 0,
            header: UmdHeader::new(),
            chapters: UmdChapters::new(),
            cover: UmdCover::new(),
            end: UmdEnd::new(),
        }
    }

    pub fn get_num(&self) -> i32 {
        return self.num;
    }

    pub fn set_num(&mut self, num: i32) {
        self.num = num;
    }

    /**
     * Build the UMD file.
     * @param os
     * @throws IOException
     */
    pub fn build_umd(&self, os: &mut dyn Write) {
        let mut wos = WrapOutputStream::new(os);

        self.header.build_header(&mut wos);
        self.chapters.build_chapters(&mut wos);
        self.cover.build_cover(&mut wos);
        self.end.build_end(&mut wos);
    }

    pub fn get_header(&self) -> &UmdHeader {
        return &self.header;
    }

    pub fn set_header(&mut self, header: UmdHeader) {
        self.header = header;
    }

    pub fn get_chapters(&self) -> &UmdChapters {
        return &self.chapters;
    }

    pub fn set_chapters(&mut self, chapters: UmdChapters) {
        self.chapters = chapters;
    }

    pub fn get_cover(&self) -> &UmdCover {
        return &self.cover;
    }

    pub fn set_cover(&mut self, cover: UmdCover) {
        self.cover = cover;
    }

    pub fn get_end(&self) -> &UmdEnd {
        return &self.end;
    }

    pub fn set_end(&mut self, end: UmdEnd) {
        self.end = end;
    }
}
