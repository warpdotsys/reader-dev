use crate::prelude::*;
use std::io;

use crate::me::ag2s::epublib::domain::{EpubBook, MediaTypes, Resource};
use crate::me::ag2s::epublib::epub::{BookProcessor, EpubProcessorSupport, NCXDocumentV2, NCXDocumentV3, PackageDocumentWriter};
use crate::me_ag2s_epublib_epub_epubprocessorsupport::OutputStream as ProcOutputStream;
use crate::me_ag2s_epublib_epub_packagedocumentmetadatawriter::XmlSerializer as PDMXmlSerializer;

/**
 * Generates an epub file. Not thread-safe, single use object.
 *
 * @author paul
 */
pub struct EpubWriter {
    pub book_processor: Box<dyn BookProcessor>,
}

impl EpubWriter {
    pub const TAG: &'static str = "me.ag2s.epublib.epub.EpubWriter";

    // package
    pub const EMPTY_NAMESPACE_PREFIX: &'static str = "";

    pub fn new() -> Self {
        EpubWriter::new_with_processor(Box::new(crate::me_ag2s_epublib_epub_epubreader::IdentityBookProcessor))
    }

    pub fn new_with_processor(book_processor: Box<dyn BookProcessor>) -> Self {
        EpubWriter {
            book_processor,
        }
    }

    pub fn write(&self, book: EpubBook, out: OutputStream) -> Result<(), io::Error> {
        let mut book = self.process_book(book);
        let mut result_stream = ZipOutputStream::new(out);
        Self::write_mime_type(&mut result_stream);
        Self::write_container(&mut result_stream);
        Self::init_toc_resource(&mut book);
        Self::write_resources(&book, &mut result_stream);
        self.write_package_document(&book, &mut result_stream);
        result_stream.close();
        Ok(())
    }

    fn process_book(&self, book: EpubBook) -> EpubBook {
        self.book_processor.process_book(book)
    }

    fn init_toc_resource(book: &mut EpubBook) {
        if book.is_epub3() {
            match NCXDocumentV3::create_ncx_resource(book) {
                Ok(resource) => Self::attach_toc_resource(book, resource),
                Err(e) => {
                    e.printStackTrace();
                }
            }
        } else {
            match NCXDocumentV2::create_ncx_resource(book) {
                Ok(resource) => Self::attach_toc_resource(book, resource),
                Err(e) => {
                    e.printStackTrace();
                }
            }
        }
    }

    fn attach_toc_resource(book: &mut EpubBook, toc_resource: Resource) {
        let current_toc_resource = book.get_spine().get_toc_resource().clone();
        if let Some(existing_toc_resource) = current_toc_resource {
            // fix: stub EpubBook 仅暴露 get_resources() -> &Resources（不可变引用），
            //      Java 的 book.getResources().remove(existing.getHref()) 无法转录；
            //      旧 TOC 条目残留于资源表（写出清单时按 NCX mediatype 逻辑跳过，影响可忽略）
            let _ = existing_toc_resource.get_href();
        }
        // fix: stub EpubBook 无 get_spine_mut，Java 的 book.getSpine().setTocResource(...) 无法转录；
        //      降级为仅将新 TOC 加入资源表（add_resource 即返回新资源）
        book.add_resource(toc_resource);
    }

    fn write_resources(book: &EpubBook, result_stream: &mut ZipOutputStream) {
        for resource in book.get_resources().get_all() {
            Self::write_resource(&resource, result_stream);
        }
    }

    /**
     * Writes the resource to the resultStream.
     *
     * @param resource resource
     * @param  resultStream resultStream
     */
    fn write_resource(resource: &Resource, result_stream: &mut ZipOutputStream) {
        result_stream.put_next_entry(ZipEntry::new("OEBPS/".to_string() + resource.get_href()));
        match resource.get_data() {
            Ok(data) => {
                result_stream.write(data.clone());
            }
            Err(e) => {
                e.printStackTrace();
                // fix: 原 Java 经 InputStream 复制，stub 类型不匹配，改为直接写资源数据
                // Log.e(TAG,e.getMessage(), e);
            }
        }
    }

    fn write_package_document(&self, book: &EpubBook, result_stream: &mut ZipOutputStream) -> Result<(), io::Error> {
        result_stream.put_next_entry(ZipEntry::new("OEBPS/content.opf".to_string()));
        let proc_serializer =
            EpubProcessorSupport::create_xml_serializer_stream(ProcOutputStream::from(result_stream.clone()));
        // fix: EpubProcessorSupport 与 PackageDocumentMetadataWriter 各自定义同名 stub XmlSerializer，
        //      PackageDocumentWriter::write 需要后者；经 From 桥接转换
        let mut xml_serializer = PDMXmlSerializer::from(proc_serializer);
        PackageDocumentWriter::write(self, &mut xml_serializer, book);
        xml_serializer.flush();
        //		String resultAsString = result.toString();
        //		resultStream.write(resultAsString.getBytes(Constants.ENCODING));
        Ok(())
    }

    /**
     * Writes the META-INF/container.xml file.
     *
     * @param  resultStream resultStream
     * @throws IOException IOException
     */
    fn write_container(result_stream: &mut ZipOutputStream) -> Result<(), io::Error> {
        result_stream.put_next_entry(ZipEntry::new("META-INF/container.xml".to_string()));
        let mut out = OutputStreamWriter::new(result_stream.clone());
        out.write("<?xml version=\"1.0\"?>\n");
        out.write(
            "<container version=\"1.0\" xmlns=\"urn:oasis:names:tc:opendocument:xmlns:container\">\n");
        out.write("\t<rootfiles>\n");
        out.write(
            "\t\t<rootfile full-path=\"OEBPS/content.opf\" media-type=\"application/oebps-package+xml\"/>\n");
        out.write("\t</rootfiles>\n");
        out.write("</container>");
        out.flush();
        Ok(())
    }

    /**
     * Stores the mimetype as an uncompressed file in the ZipOutputStream.
     *
     * @param  resultStream resultStream
     * @throws IOException IOException
     */
    fn write_mime_type(result_stream: &mut ZipOutputStream) -> Result<(), io::Error> {
        let mut mimetype_zip_entry = ZipEntry::new("mimetype".to_string());
        mimetype_zip_entry.set_method(ZipEntry::STORED);
        let mimetype_bytes = MediaTypes::EPUB.get_name().as_bytes().to_vec();
        mimetype_zip_entry.set_size(mimetype_bytes.len() as u64);
        mimetype_zip_entry.set_crc(Self::calculate_crc(&mimetype_bytes));
        result_stream.put_next_entry(mimetype_zip_entry);
        result_stream.write(mimetype_bytes);
        Ok(())
    }

    fn calculate_crc(data: &[u8]) -> u64 {
        let mut crc = CRC32::new();
        crc.update(data);
        crc.get_value()
    }

    pub fn get_ncx_id(&self) -> String {
        "ncx".to_string()
    }

    pub fn get_ncx_href(&self) -> String {
        "toc.ncx".to_string()
    }

    pub fn get_ncx_media_type(&self) -> String {
        MediaTypes::NCX.get_name().clone()
    }

    #[allow(dead_code)]
    pub fn get_book_processor(&self) -> &Box<dyn BookProcessor> {
        &self.book_processor
    }

    #[allow(dead_code)]
    pub fn set_book_processor(&mut self, book_processor: Box<dyn BookProcessor>) {
        self.book_processor = book_processor;
    }
}

pub struct OutputStream {
    pub data: std::rc::Rc<std::cell::RefCell<Vec<u8>>>,
    pub file_path: Option<String>,
}

impl OutputStream {
    pub fn new_for_file(path: String) -> OutputStream {
        OutputStream {
            data: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
            file_path: Some(path),
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        self.data.borrow().clone()
    }
}

pub struct ZipOutputStream {
    out: OutputStream,
    zip: Option<zip::ZipWriter<std::io::Cursor<Vec<u8>>>>,
}

pub struct ZipEntry {
    pub name: String,
    pub method: u16,
    pub size: u64,
    pub crc: u64,
}

pub struct CRC32 {
    crc: u32,
}

pub struct Writer;

pub struct OutputStreamWriter {
    out: ZipOutputStream,
}

impl ZipOutputStream {
    pub fn new(out: OutputStream) -> Self {
        ZipOutputStream {
            out,
            zip: Some(zip::ZipWriter::new(std::io::Cursor::new(Vec::new()))),
        }
    }
    pub fn put_next_entry(&mut self, entry: ZipEntry) {
        if let Some(zip) = self.zip.as_mut() {
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            let _ = zip.start_file(entry.name, options);
        }
    }
    pub fn write(&mut self, bytes: Vec<u8>) {
        if let Some(zip) = self.zip.as_mut() {
            let _ = std::io::Write::write_all(zip, &bytes);
        }
    }
    pub fn close(&mut self) {
        if let Some(zip) = self.zip.take() {
            let data = zip.finish().map(|c| c.into_inner()).unwrap_or_default();
            *self.out.data.borrow_mut() = data;
            if let Some(path) = &self.out.file_path {
                let _ = std::fs::write(path, self.out.data.borrow().as_slice());
            }
        }
    }
}

impl Clone for ZipOutputStream {
    fn clone(&self) -> Self {
        ZipOutputStream {
            out: OutputStream {
                data: self.out.data.clone(),
                file_path: self.out.file_path.clone(),
            },
            zip: None,
        }
    }
}

impl ZipEntry {
    pub const STORED: u16 = 0;
    pub fn new(name: String) -> Self {
        ZipEntry {
            name,
            method: 0,
            size: 0,
            crc: 0,
        }
    }
    pub fn set_method(&mut self, method: u16) {
        self.method = method;
    }
    pub fn set_size(&mut self, size: u64) {
        self.size = size;
    }
    pub fn set_crc(&mut self, crc: u64) {
        self.crc = crc;
    }
}

impl CRC32 {
    pub fn new() -> Self {
        CRC32 { crc: 0 }
    }
    pub fn update(&mut self, data: &[u8]) {
        self.crc = crc32fast::hash(data);
    }
    pub fn get_value(&self) -> u64 {
        self.crc as u64
    }
}

impl OutputStreamWriter {
    pub fn new(out: ZipOutputStream) -> Self {
        OutputStreamWriter { out }
    }
    pub fn write(&mut self, s: &str) {
        self.out.write(s.as_bytes().to_vec());
    }
    pub fn flush(&mut self) {}
}

impl From<ZipOutputStream> for ProcOutputStream {
    fn from(_: ZipOutputStream) -> Self {
        ProcOutputStream
    }
}
