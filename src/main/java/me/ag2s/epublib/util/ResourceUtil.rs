use crate::prelude::*;
use std::io;

use crate::me::ag2s::epublib::domain::{MediaTypes, Resource};
use crate::me::ag2s::epublib::util::{IOUtil, StringUtil};
// fix: 显式导入 IOUtil 的 InputStream（本文件原占位 struct 与 IOUtil::to_byte_array 参数类型不一致）
use crate::me_ag2s_epublib_util_ioutil::InputStream;

/**
 * Various resource utility methods
 *
 * @author paul
 */
pub struct ResourceUtil;

impl ResourceUtil {
    /**
     * 快速创建HTML类型的Resource
     *
     * @param title 章节的标题
     * @param txt   章节的正文
     * @param model html模板
     * @return 返回Resource
     */
    pub fn create_chapter_resource(mut title: String, txt: &str, model: &str, href: &str) -> Resource {
        if title.contains("\n") {
            title = "<span class=\"chapter-sequence-number\">".to_string() + &title.replacen("\\s*\\n\\s*", "</span><br />", 1);
        } else {
            title = title.replacen("\\s+", "</span><br />", 1);
            if title.contains("</span>") {
                title = "<span class=\"chapter-sequence-number\">".to_string() + &title;
            }
        }
        let html = model.replace("{title}", &title)
            .replace("{content}", &StringUtil::format_html(txt));
        Resource::new_bytes(html.into_bytes(), href)
    }

    pub fn create_public_resource(name: &str, author: &str, intro: &str, kind: &str, word_count: &str, model: &str, href: &str) -> Resource {
        // fix: Java `kind == null` 在 Rust `&str` 下恒不成立，改为空串判断（&str 不可为 None）
        let html = model.replace("{name}", name)
            .replace("{author}", author)
            .replace("{kind}", if kind.is_empty() { "" } else { kind })
            .replace("{wordCount}", if word_count.is_empty() { "" } else { word_count })
            .replace("{intro}", &StringUtil::format_html(if intro.is_empty() { "" } else { intro }));
        Resource::new_bytes(html.into_bytes(), href)
    }

    /**
     * 快速从File创建Resource
     *
     * @param file File
     * @return Resource
     * @throws IOException IOException
     */

    #[allow(dead_code)]
    // fix: Java `createResourceFromFile(File)` 的 `file == null` 分支在 Rust `File` 占位结构按值传入下恒不成立，已省略；
    // 返回类型改为 Option<Resource> 以承载原 Java 的 null 返回语义
    pub fn create_resource_from_file(file: File) -> Result<Option<Resource>, io::Error> {
        let media_type = MediaTypes::determine_media_type(&file.get_name());
        let data = IOUtil::to_byte_array(&FileInputStream::new(file))?;
        Ok(Some(Resource::new_data(data, media_type)))
    }

    /**
     * 创建一个只带标题的HTMl类型的Resource,常用于封面页，大卷页
     *
     * @param title v
     * @param href  v
     * @return a resource with as contents a html page with the given title.
     */
    #[allow(dead_code)]
    // fix: Java 同名重载 createResource(String,String) 与 createResource(ZipEntry,InputStream) 无法在 Rust 共存，
    // 此版本（仅标题+href）改名 create_resource_html；ZipEntry 版本保留原名（ResourcesLoader 依赖）
    pub fn create_resource_html(title: &str, href: &str) -> Resource {
        let content =
            "<html><head><title>".to_string() + title + "</title></head><body><h1>" + title
                + "</h1></body></html>";
        Resource::new_full(None, content.into_bytes(), href, MediaTypes::XHTML,
            // fix: E0790 Constants 转录为 trait，关联常量不能直接访问；字面量值即 "UTF-8"
            "UTF-8")
    }

    /**
     * Creates a resource out of the given zipEntry and zipInputStream.
     *
     * @param zipEntry       v
     * @param zipInputStream v
     * @return a resource created out of the given zipEntry and zipInputStream.
     * @throws IOException v
     */
    pub fn create_resource_zip_in(zip_entry: &ZipEntry,
                                  zip_input_stream: &mut ZipInputStream) -> Result<Resource, io::Error> {
        Ok(Resource::new_stream(zip_input_stream, zip_entry.get_name()))

    }

    // fix: E0308 调用方（ResourcesLoader）传入其本模块的 ZipEntry 与两种不同的流类型
    //（&Vec<u8> 与 &mut ZipInputStream，&mut 会自动重借用为 &）；流参数泛型化（Resource::new_stream 本身即泛型），
    // ZipEntry 改用调用方模块的类型（其 get_name 已实现）
    pub fn create_resource<S: crate::stubs::InputStream>(zip_entry: &crate::me_ag2s_epublib_epub_resourcesloader::ZipEntry,
                              zip_input_stream: &mut S) -> Result<Resource, io::Error> {
        Ok(Resource::new_stream(zip_input_stream, zip_entry.get_name()))

    }

    /**
     * Converts a given string from given input character encoding to the requested output character encoding.
     *
     * @param inputEncoding  v
     * @param outputEncoding v
     * @param input          v
     * @return the string from given input character encoding converted to the requested output character encoding.
     * @throws UnsupportedEncodingException v
     */
    #[allow(dead_code)]
    pub fn recode(input_encoding: &str, output_encoding: &str,
                  input: &[u8]) -> Vec<u8> {
        String::from_utf8_lossy(input).as_bytes().to_vec()
    }

    /**
     * Gets the contents of the Resource as an InputSource in a None-safe manner.
     */
    #[allow(dead_code)]
    pub fn get_input_source(resource: &Resource) -> Result<Option<InputSource>, io::Error> {
        // fix: Java `resource == null` 检查在 Rust `&Resource` 下恒不成立，已省略
        let reader = resource.get_reader().ok();
        if reader.is_none() {
            // fix: Java `reader == null` 返回 null InputSource
            return Ok(None);
        }
        Ok(Some(InputSource::new(reader.unwrap())))
    }

    /**
     * Reads parses the xml therein and returns the result as a Document
     */
    pub fn get_as_document(resource: &Resource) -> Result<Document, ParseError> {
        // fix: EpubProcessorSupport::create_document_builder() 返回另一模块的占位 DocumentBuilder（无 parse 方法），
        // 改用本文件 DocumentBuilder 占位（其 parse 由本文件实现）
        Self::get_as_document_builder(resource, &DocumentBuilder)
    }

    /**
     * Reads the given resources inputstream, parses the xml therein and returns the result as a Document
     *
     * @param resource        v
     * @param documentBuilder v
     * @return the document created from the given resource
     * @throws UnsupportedEncodingException v
     * @throws SAXException                 v
     * @throws IOException                  v
     */
    pub fn get_as_document_builder(resource: &Resource,
                                   document_builder: &DocumentBuilder) -> Result<Document, ParseError> {
        let input_source = Self::get_input_source(resource).map_err(|_| ParseError)?;
        if input_source.is_none() {
            // fix: Java 返回 null Document，Rust 占位返回空 Document
            return Ok(Document::new(String::new()));
        }
        document_builder.parse(input_source.unwrap())
    }
}

pub struct File;
pub struct FileInputStream;
pub struct ZipEntry;
pub struct ZipInputStream;

// fix: 流接口（create_resource_zip_in 调用路径；ZipInputStream 由调用方以真实流填充）
impl crate::stubs::InputStream for ZipInputStream {
    fn read(&mut self, b: &mut [u8], off: usize, len: usize) -> i32 {
        let _ = (b, off, len);
        -1
    }
    fn close(&mut self) {}
}
pub struct InputSource {
    pub data: String,
}
pub struct DocumentBuilder;
pub struct Document {
    pub html: String,
}
pub struct ParseError;

impl Document {
    pub fn new(html: String) -> Document {
        Document { html }
    }
}

impl File {
    // fix: 占位（Java `file.getName()`）；File 为单元结构体，返回空串
    pub fn get_name(&self) -> String { String::new() }
}

impl FileInputStream {
    // fix: 返回 IOUtil::InputStream 以便与 IOUtil::to_byte_array(&InputStream) 衔接（原占位返回 FileInputStream）
    pub fn new(_file: File) -> InputStream { InputStream }
}

impl ZipEntry {
    // fix: 占位（Java `zipEntry.getName()`）；ZipEntry 为单元结构体，返回空串
    pub fn get_name(&self) -> String { String::new() }
}

impl InputSource {
    // fix: Java `Reader` 参数转录为 XmlStreamReader（Resource::get_reader 的返回类型）
    //      读取 reader 中全部文本作为 XML 数据
    pub fn new(mut reader: XmlStreamReader) -> Self {
        InputSource {
            data: read_reader_all(&mut reader),
        }
    }
}

fn read_reader_all(reader: &mut XmlStreamReader) -> String {
    // XmlStreamReader.read(&mut [char]) 读取字符
    let mut text = String::new();
    loop {
        let mut buf = ['\0'; 2048];
        let len = buf.len();
        match reader.read(&mut buf, 0, len) {
            Ok(n) if n > 0 => {
                text.push_str(&buf[..n as usize].iter().collect::<String>());
            }
            _ => break,
        }
    }
    text
}

impl DocumentBuilder {
    // 真实解析：InputSource 数据 → Document（XML 文本载体，DOM 树由 to_dom_document 构建）
    pub fn parse(&self, input_source: InputSource) -> Result<Document, ParseError> {
        Ok(Document::new(input_source.data))
    }
}
