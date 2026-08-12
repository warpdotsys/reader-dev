use crate::prelude::*;
use crate::stubs::Charset;
// package io.legado.app.model.localBook
//
// import io.legado.app.data.entities.Book
// import io.legado.app.data.entities.BookChapter
// import io.legado.app.data.entities.TxtTocRule
// import io.legado.app.help.DefaultData
// import io.legado.app.utils.EncodingDetect
// import io.legado.app.utils.MD5Utils
// import io.legado.app.utils.StringUtils
// import io.legado.app.utils.Utf8BomUtils
// import java.io.FileNotFoundException
// import java.nio.charset.Charset
// import java.util.regex.Matcher
// import java.util.regex.Pattern
// import kotlin.math.min
// import mu.KotlinLogging
//
// private val logger = KotlinLogging.logger {}

pub struct TextFile {
    book: Book,

    // private val blank: Byte = 0x0a
    blank: u8,

    //默认从文件中获取数据的长度
    // private val bufferSize = 512000
    buffer_size: usize,

    //没有标题的时候，每个章节的最大长度
    // private val maxLengthWithNoToc = 10 * 1024
    max_length_with_no_toc: usize,

    //使用正则划分目录，每个章节的最大允许长度
    // private val maxLengthWithToc = 102400
    max_length_with_toc: usize,

    charset: Charset,
}

impl TextFile {

    pub fn new(book: Book) -> Self {
        let charset = book.file_charset();
        TextFile {
            book,
            blank: 0x0a,
            buffer_size: 512000,
            max_length_with_no_toc: 10 * 1024,
            max_length_with_toc: 102400,
            charset,
        }
    }

    // companion object {

    // @Throws(FileNotFoundException::class)
    // fix: Kotlin companion `TextFile(book).getChapterList()` —— Book 无 Clone，临时取走处理后写回
    pub fn get_chapter_list(book: &mut Book) -> Vec<BookChapter> {
        let mut taken = std::mem::take(book);
        let mut tf = TextFile::new(taken);
        let toc = tf.get_chapter_list_inner();
        *book = tf.book;
        return toc;
    }

    // @Throws(FileNotFoundException::class)
    pub fn get_content(book: &mut Book, book_chapter: &BookChapter) -> Option<String> {
        let mut tf = TextFile::new(std::mem::take(book));
        let count = (book_chapter.end.unwrap() - book_chapter.start.unwrap()) as usize;
        let mut buffer = vec![0u8; count];
        let mut bis = LocalBook::get_book_input_stream(&mut tf.book);
        bis.skip(book_chapter.start.unwrap());
        bis.read(&mut buffer, 0, count);
        if tf.book.charset.is_none() {
            tf.book.charset = Some(EncodingDetect::get_encode(tf.book.get_local_file()));
        }
        let result = String::from_utf8_lossy(&buffer[..]).to_string()
            .substring_after(&book_chapter.title)
            .replace_with_regex("^[\\n\\s]+", "　　");
        *book = tf.book;
        return Some(result);
    }

    // }

    /**
     * 获取目录
     */
    // @Throws(FileNotFoundException::class)
    // fix: 与 companion 静态 get_chapter_list(book) 重名（E0592），实例方法改名
    pub fn get_chapter_list_inner(&mut self) -> Vec<BookChapter> {
        if self.book.charset.is_none() || self.book.toc_url.is_blank() {
            let mut bis = LocalBook::get_book_input_stream(&mut self.book);
            let mut buffer = vec![0u8; self.buffer_size];
            let length = bis.read(&mut buffer, 0, self.buffer_size);
            if self.book.charset.is_none() {
                self.book.charset = Some(EncodingDetect::get_encode_from_bytes(buffer[..length as usize].to_vec()));
            }
            self.charset = self.book.file_charset();
            if self.book.toc_url.is_blank() {
                let block_content = String::from_utf8_lossy(&buffer[0..length as usize]).to_string();
                self.book.toc_url = self.get_toc_rule(&block_content).map(|p| p.pattern()).unwrap_or(String::new());
            }
        }
        let mut toc = self.analyze_with_pattern(Some(self.book.toc_url.to_pattern_with_flags(Pattern::MULTILINE)));
        for (index, book_chapter) in toc.iter_mut().enumerate() {
            book_chapter.index = index as i32;
            book_chapter.book_url = self.book.book_url.clone();
            book_chapter.url = md5_encode16(self.book.origin_name.clone() + &index.to_string() + &book_chapter.title);
        }
        self.book.latest_chapter_title = toc.last().map(|c| c.title.clone());
        self.book.total_chapter_num = toc.len() as i32;
        return toc;
    }

    /**
     * 按规则解析目录
     */
    fn analyze_with_pattern(&mut self, pattern: Option<Pattern>) -> Vec<BookChapter> {
        if pattern.is_none() || pattern.as_ref().unwrap().pattern().is_empty() {
            return self.analyze(0, i64::MAX);
        }
        let pattern = pattern.unwrap();
        let mut toc = Vec::new();
        let mut bis = LocalBook::get_book_input_stream(&mut self.book);
        let mut block_content: String;
        //加载章节
        let mut cur_offset: i64 = 0;
        //读取的长度
        let mut length: usize;
        let mut buffer = vec![0u8; self.buffer_size];
        let mut buffer_start = 3;
        bis.read_range(&mut buffer, 0, 3);
        if Utf8BomUtils::hasBom(&buffer) {
            buffer_start = 0;
            cur_offset = 3;
        }
        //获取文件中的数据到buffer，直到没有数据为止
        loop {
            length = bis.read_range(&mut buffer, buffer_start, self.buffer_size - buffer_start);
            if !(length > 0) {
                break;
            }
            let mut end = buffer_start + length;
            if end == self.buffer_size {
                let mut i = buffer_start + length - 1;
                loop {
                    if buffer[i] == self.blank {
                        end = i;
                        break;
                    }
                    if i == 0 {
                        break;
                    }
                    i -= 1;
                }
            }
            //将数据转换成String, 不能超过length
            block_content = String::from_utf8_lossy(&buffer[0..end]).to_string();
            buffer.copy_within(end..buffer_start + length, 0);
            buffer_start = buffer_start + length - end;
            length = end;
            //当前Block下使过的String的指针
            let mut seek_pos: usize = 0;
            //进行正则匹配
            let mut matcher = pattern.matcher(block_content.clone());
            //如果存在相应章节
            while matcher.find() {
                //获取匹配到的字符在字符串中的起始位置
                let chapter_start = matcher.start();
                //获取章节内容
                let chapter_content = block_content[seek_pos..chapter_start].to_string();
                let chapter_length = chapter_content.as_bytes().len();
                let last_start = toc.last().map(|c: &BookChapter| c.start).flatten().unwrap_or(cur_offset);
                if self.book.get_split_long_chapter()
                    && cur_offset + chapter_length as i64 - last_start > self.max_length_with_toc as i64
                {
                    if let Some(last) = toc.last_mut() {
                        last.end = last.start;
                    }
                    //章节字数太多进行拆分
                    let last_title = toc.last().map(|c| c.title.clone());
                    let last_title_length = last_title.as_ref().map(|t| t.as_bytes().len()).unwrap_or(0);
                    let mut chapters = self.analyze(
                        last_start + last_title_length as i64,
                        cur_offset + chapter_length as i64
                    );
                    if let Some(lt) = last_title {
                        for (index, book_chapter) in chapters.iter_mut().enumerate() {
                            book_chapter.title = format!("{}({})", lt, index + 1);
                        }
                    }
                    toc.extend(chapters);
                    //创建当前章节
                    let mut cur_chapter = BookChapter::default();
                    cur_chapter.title = matcher.group();
                    cur_chapter.start = Some(cur_offset + chapter_length as i64);
                    toc.push(cur_chapter);
                } else if seek_pos == 0 && chapter_start != 0 {
                    /*
                     * 如果 seekPos == 0 && chapterStart != 0 表示当前block处前面有一段内容
                     * 第一种情况一定是序章 第二种情况是上一个章节的内容
                     */
                    if toc.is_empty() {
                        //如果当前没有章节，那么就是序章
                        //加入简介
                        if StringUtils::trim(&chapter_content).is_not_blank() {
                            let mut qy_chapter = BookChapter::default();
                            qy_chapter.title = "前言".to_string();
                            qy_chapter.start = Some(cur_offset);
                            qy_chapter.end = Some(chapter_length as i64);
                            toc.push(qy_chapter);
                        }
                        //创建当前章节
                        let mut cur_chapter = BookChapter::default();
                        cur_chapter.title = matcher.group();
                        cur_chapter.start = Some(chapter_length as i64);
                        toc.push(cur_chapter);
                    } else {
                        //否则就block分割之后，上一个章节的剩余内容
                        //获取上一章节
                        let last_chapter = toc.last_mut().unwrap();
                        last_chapter.is_volume =
                            chapter_content.substring_after(&last_chapter.title).is_blank();
                        //将当前段落添加上一章去
                        last_chapter.end = Some(last_chapter.end.unwrap() + chapter_length as i64);
                        //创建当前章节
                        let mut cur_chapter = BookChapter::default();
                        cur_chapter.title = matcher.group();
                        cur_chapter.start = last_chapter.end;
                        toc.push(cur_chapter);
                    }
                } else {
                    if !toc.is_empty() {
                        //获取章节内容
                        //获取上一章节
                        let last_chapter = toc.last_mut().unwrap();
                        last_chapter.is_volume =
                            chapter_content.substring_after(&last_chapter.title).is_blank();
                        last_chapter.end = Some(
                            last_chapter.start.unwrap() + chapter_content.as_bytes().len() as i64
                        );
                        //创建当前章节
                        let mut cur_chapter = BookChapter::default();
                        cur_chapter.title = matcher.group();
                        cur_chapter.start = last_chapter.end;
                        toc.push(cur_chapter);
                    } else {
                        //如果章节不存在则创建章节
                        let mut cur_chapter = BookChapter::default();
                        cur_chapter.title = matcher.group();
                        cur_chapter.start = Some(cur_offset);
                        cur_chapter.end = Some(cur_offset);
                        toc.push(cur_chapter);
                    }
                }
                //设置指针偏移
                seek_pos += chapter_content.len();
            }
            //block的偏移点
            cur_offset += length as i64;
            //设置上一章的结尾
            if let Some(last) = toc.last_mut() {
                last.end = Some(cur_offset);
            }
        }
        System::gc();
        System::run_finalization();
        return toc;
    }

    /**
     * 无规则拆分目录
     */
    fn analyze(
        &mut self,
        file_start: i64,
        file_end: i64
    ) -> Vec<BookChapter> {
        let mut toc = Vec::new();
        let mut bis = LocalBook::get_book_input_stream(&mut self.book);
        //block的个数
        let mut block_pos = 0;
        //加载章节
        let mut cur_offset: i64 = 0;
        let mut chapter_pos = 0;
        //读取的长度
        let mut length = 0;
        let mut buffer = vec![0u8; self.buffer_size];
        let mut buffer_start = 3;
        if file_start == 0 {
            bis.read_range(&mut buffer, 0, 3);
            if Utf8BomUtils::hasBom(&buffer) {
                buffer_start = 0;
                cur_offset = 3;
            }
        } else {
            bis.skip(file_start);
            cur_offset = file_start;
            buffer_start = 0;
        }
        //获取文件中的数据到buffer，直到没有数据为止
        loop {
            let read_len = (self.buffer_size - buffer_start).min(
                (file_end - cur_offset - buffer_start as i64) as usize
            );
            if !(file_end - cur_offset - buffer_start as i64 > 0) {
                break;
            }
            length = bis.read_range(&mut buffer, buffer_start, read_len);
            if !(length > 0) {
                break;
            }
            block_pos += 1;
            //章节在buffer的偏移量
            let mut chapter_offset = 0;
            //当前剩余可分配的长度
            length += buffer_start;
            let mut str_length = length;
            //分章的位置
            chapter_pos = 0;
            while str_length > 0 {
                chapter_pos += 1;
                //是否长度超过一章
                if str_length > self.max_length_with_no_toc {
                    //在buffer中一章的终止点
                    let mut end = length;
                    //寻找换行符作为终止点
                    for i in (chapter_offset + self.max_length_with_no_toc)..length {
                        if buffer[i] == self.blank {
                            end = i;
                            break;
                        }
                    }
                    let mut chapter = BookChapter::default();
                    chapter.title = format!("第{}章({})", block_pos, chapter_pos);
                    chapter.start = Some(toc.last().map(|c: &BookChapter| c.end).flatten().unwrap_or(cur_offset));
                    chapter.end = Some(chapter.start.unwrap() + (end - chapter_offset) as i64);
                    toc.push(chapter);
                    //减去已经被分配的长度
                    str_length -= (end - chapter_offset);
                    //设置偏移的位置
                    chapter_offset = end;
                } else {
                    buffer.copy_within((length - str_length)..length, 0);
                    length -= str_length;
                    buffer_start = str_length;
                    str_length = 0;
                }
            }
            //block的偏移点
            cur_offset += length as i64;
        }
        //设置结尾章节
        if buffer_start > 100 || toc.is_empty() {
            let mut chapter = BookChapter::default();
            chapter.title = format!("第{}章({})", block_pos, chapter_pos);
            chapter.start = Some(toc.last().map(|c| c.end).flatten().unwrap_or(cur_offset));
            chapter.end = Some(chapter.start.unwrap() + buffer_start as i64);
            toc.push(chapter);
        } else {
            if let Some(last) = toc.last_mut() {
                last.end = Some(last.end.unwrap() + buffer_start as i64);
            }
        }
        return toc;
    }

    /**
     * 获取所有匹配次数大于1的目录规则
     */
    fn get_toc_rule(&self, content: &str) -> Option<Pattern> {
        let mut rules = self.get_toc_rules();
        rules.reverse();
        let mut max_cs = 1;
        let mut toc_pattern: Option<Pattern> = None;
        for toc_rule in rules {
            let pattern = toc_rule.rule.to_pattern_with_flags(Pattern::MULTILINE);
            let mut matcher = pattern.matcher(content.to_string());
            let mut cs = 0;
            while matcher.find() {
                cs += 1;
            }
            if cs >= max_cs {
                max_cs = cs;
                toc_pattern = Some(pattern);
            }
        }
        return toc_pattern;
    }

    /**
     * 获取启用的目录规则
     */
    fn get_toc_rules(&self) -> Vec<&'static TxtTocRule> {
        return DefaultData::txt_toc_rules()
            .iter()
            .filter(|it| it.enable)
            .collect();
    }
}
