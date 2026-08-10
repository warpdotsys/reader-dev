use std::io;

use crate::me::ag2s::epublib::domain::{EpubBook, MediaTypes, Resource};
use crate::me::ag2s::epublib::epub::{BookProcessor, EpubProcessorSupport, NCXDocumentV2, NCXDocumentV3, PackageDocumentWriter};
use crate::me::ag2s::epublib::util::IOUtil;

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
        EpubWriter::new_with_processor(Box::new(crate::me::ag2s::epublib::epub::EpubReader::IdentityBookProcessor))
    }

    pub fn new_with_processor(book_processor: Box<dyn BookProcessor>) -> Self {
        EpubWriter {
            book_processor,
        }
    }

    pub fn write(&self, book: EpubBook, out: OutputStream) -> Result<(), io::Error> {
        let book = self.process_book(book);
        let mut result_stream = ZipOutputStream::new(out);
        write_mime_type(&mut result_stream);
        write_container(&mut result_stream);
        init_toc_resource(&book);
        write_resources(&book, &mut result_stream);
        write_package_document(&book, &mut result_stream);
        result_stream.close();
        Ok(())
    }

    fn process_book(&self, book: EpubBook) -> EpubBook {
        if self.book_processor != null {
            self.book_processor.process_book(book).unwrap()
        } else {
            book
        }
    }

    fn init_toc_resource(book: &EpubBook) {
        let toc_resource;
        match {
            if book.is_epub3() {
                NCXDocumentV3::create_ncx_resource(book)
            } else {
                NCXDocumentV2::create_ncx_resource(book)
            }
        } {
            Ok(resource) => {
                toc_resource = resource;
                let current_toc_resource = book.get_spine().get_toc_resource();
                if current_toc_resource != null {
                    book.get_resources().remove(current_toc_resource.get_href());
                }
                book.get_spine().set_toc_resource(toc_resource);
                book.get_resources().add(toc_resource);
            }
            Err(e) => {
                e.printStackTrace();
                // Log.e(TAG,
                //     "Error writing table of contents: "
                //         + ex.getClass().getName() + ": " + ex.getMessage(), ex);
            }
        }
    }

    fn write_resources(book: &EpubBook, result_stream: &mut ZipOutputStream) {
        for resource in book.get_resources().get_all() {
            write_resource(resource, result_stream);
        }
    }

    /**
     * Writes the resource to the resultStream.
     *
     * @param resource resource
     * @param  resultStream resultStream
     */
    fn write_resource(resource: &Resource, result_stream: &mut ZipOutputStream) {
        if resource == null {
            return;
        }
        match {
            result_stream.put_next_entry(ZipEntry::new("OEBPS/".to_string() + resource.get_href()));
            let input_stream = resource.get_input_stream();
            IOUtil::copy(input_stream, result_stream);
            input_stream.close();
        } {
            Ok(_) => {}
            Err(e) => {
                e.printStackTrace();
                // Log.e(TAG,e.getMessage(), e);
            }
        }
    }

    fn write_package_document(book: &EpubBook, result_stream: &mut ZipOutputStream) -> Result<(), io::Error> {
        result_stream.put_next_entry(ZipEntry::new("OEBPS/content.opf"));
        let xml_serializer = EpubProcessorSupport::create_xml_serializer_stream(result_stream.clone());
        PackageDocumentWriter::write(self, xml_serializer, book);
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
        result_stream.put_next_entry(ZipEntry::new("META-INF/container.xml"));
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
        let mut mimetype_zip_entry = ZipEntry::new("mimetype");
        mimetype_zip_entry.set_method(ZipEntry::STORED);
        let mimetype_bytes = MediaTypes::EPUB.get_name().as_bytes().to_vec();
        mimetype_zip_entry.set_size(mimetype_bytes.len() as u64);
        mimetype_zip_entry.set_crc(calculate_crc(&mimetype_bytes));
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
        MediaTypes::NCX.get_name()
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

pub struct OutputStream;
pub struct ZipOutputStream;
pub struct ZipEntry;
pub struct CRC32;
pub struct Writer;
pub struct OutputStreamWriter;

impl ZipOutputStream {
    pub fn new(_out: OutputStream) -> Self { todo!() }
    pub fn put_next_entry(&mut self, _entry: ZipEntry) { todo!() }
    pub fn write(&mut self, _bytes: Vec<u8>) { todo!() }
    pub fn close(&mut self) { todo!() }
}

impl Clone for ZipOutputStream {
    fn clone(&self) -> Self { todo!() }
}

impl ZipEntry {
    pub const STORED: u16 = 0;
    pub fn new(_name: String) -> Self { todo!() }
    pub fn set_method(&mut self, _method: u16) { todo!() }
    pub fn set_size(&mut self, _size: u64) { todo!() }
    pub fn set_crc(&mut self, _crc: u64) { todo!() }
}

impl CRC32 {
    pub fn new() -> Self { todo!() }
    pub fn update(&mut self, _data: &[u8]) { todo!() }
    pub fn get_value(&self) -> u64 { todo!() }
}

impl OutputStreamWriter {
    pub fn new(_out: ZipOutputStream) -> Self { todo!() }
    pub fn write(&mut self, _s: &str) { todo!() }
    pub fn flush(&mut self) { todo!() }
}
