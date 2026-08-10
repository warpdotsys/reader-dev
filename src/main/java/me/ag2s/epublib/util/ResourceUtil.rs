use std::io;

use crate::me::ag2s::epublib::Constants;
use crate::me::ag2s::epublib::domain::{MediaType, MediaTypes, Resource};
use crate::me::ag2s::epublib::epub::EpubProcessorSupport;
use crate::me::ag2s::epublib::util::{IOUtil, StringUtil};

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
        let html = model.replace("{name}", name)
            .replace("{author}", author)
            .replace("{kind}", if kind == null { "" } else { kind })
            .replace("{wordCount}", if word_count == null { "" } else { word_count })
            .replace("{intro}", &StringUtil::format_html(if intro == null { "" } else { intro }));
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
    pub fn create_resource_from_file(file: File) -> Result<Resource, io::Error> {
        if file == null {
            return Ok(None);
        }
        let media_type = MediaTypes::determine_media_type(&file.get_name());
        let data = IOUtil::to_byte_array(FileInputStream::new(file))?;
        Ok(Resource::new_data(data, media_type))
    }

    /**
     * 创建一个只带标题的HTMl类型的Resource,常用于封面页，大卷页
     *
     * @param title v
     * @param href  v
     * @return a resource with as contents a html page with the given title.
     */
    #[allow(dead_code)]
    pub fn create_resource(title: &str, href: &str) -> Resource {
        let content =
            "<html><head><title>".to_string() + title + "</title></head><body><h1>" + title
                + "</h1></body></html>";
        Resource::new_full(None, content.into_bytes(), href, MediaTypes::XHTML,
            Constants::CHARACTER_ENCODING)
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

    pub fn create_resource(zip_entry: &ZipEntry,
                           zip_input_stream: &InputStream) -> Result<Resource, io::Error> {
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
     * Gets the contents of the Resource as an InputSource in a null-safe manner.
     */
    #[allow(dead_code)]
    pub fn get_input_source(resource: &Resource) -> Result<Option<InputSource>, io::Error> {
        if resource == null {
            return Ok(None);
        }
        let reader = resource.get_reader();
        if reader == null {
            return Ok(None);
        }
        Ok(Some(InputSource::new(reader)))
    }

    /**
     * Reads parses the xml therein and returns the result as a Document
     */
    pub fn get_as_document(resource: &Resource) -> Result<Document, ParseError> {
        get_as_document_builder(resource, EpubProcessorSupport::create_document_builder())
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
        let input_source = get_input_source(resource)?;
        if input_source == null {
            return Ok(None);
        }
        document_builder.parse(input_source)
    }
}

pub struct File;
pub struct FileInputStream;
pub struct ZipEntry;
pub struct ZipInputStream;
pub struct InputStream;
pub struct Reader;
pub struct InputSource;
pub struct DocumentBuilder;
pub struct Document;
pub struct ParseError;

impl File {
    pub fn get_name(&self) -> String { todo!() }
}

impl FileInputStream {
    pub fn new(_file: File) -> Self { todo!() }
}

impl ZipEntry {
    pub fn get_name(&self) -> String { todo!() }
}

impl InputSource {
    pub fn new(_reader: Reader) -> Self { todo!() }
}

impl DocumentBuilder {
    pub fn parse(&self, _input_source: InputSource) -> Result<Document, ParseError> { todo!() }
}
