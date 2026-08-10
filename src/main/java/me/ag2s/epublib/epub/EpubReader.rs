use std::io;

use crate::me::ag2s::epublib::Constants;
use crate::me::ag2s::epublib::domain::{EpubBook, MediaType, Resource, Resources};
use crate::me::ag2s::epublib::epub::{BookProcessor, NCXDocumentV2, NCXDocumentV3, PackageDocumentReader, ResourcesLoader};
use crate::me::ag2s::epublib::util::{ResourceUtil, StringUtil};

/**
 * Reads an epub file.
 *
 * @author paul
 */
#[allow(dead_code)]
pub struct EpubReader {
    pub book_processor: Box<dyn BookProcessor>,
}

impl EpubReader {
    pub fn new() -> Self {
        EpubReader {
            book_processor: Box::new(IdentityBookProcessor),
        }
    }

    pub fn read_epub(&self, in_stream: ZipInputStream) -> Result<EpubBook, io::Error> {
        self.read_epub(in_stream, Constants::CHARACTER_ENCODING)
    }

    /**
     * Read epub from inputstream
     *
     * @param in       the inputstream from which to read the epub
     * @param encoding the encoding to use for the html files within the epub
     * @return the Book as read from the inputstream
     * @throws IOException IOException
     */
    pub fn read_epub_encoding(&self, in_stream: ZipInputStream, encoding: &str) -> Result<EpubBook, io::Error> {
        self.read_epub_resources(ResourcesLoader::load_resources(in_stream, encoding))
    }

    /**
     * Reads this EPUB without loading all resources into memory.
     *
     * @param zipFile  the file to load
     * @param encoding the encoding for XHTML files
     * @return this Book without loading all resources into memory.
     * @throws IOException IOException
     */
    pub fn read_epub_lazy(&self, zip_file: ZipFile, encoding: &str) -> Result<EpubBook, io::Error> {
        self.read_epub_lazy_types(zip_file, encoding, MediaTypes::media_types.to_vec())
    }

    /**
     * Reads this EPUB without loading all resources into memory.
     *
     * @param zipFile         the file to load
     * @param encoding        the encoding for XHTML files
     * @param lazyLoadedTypes a list of the MediaType to load lazily
     * @return this Book without loading all resources into memory.
     * @throws IOException IOException
     */
    pub fn read_epub_lazy_types(&self, zip_file: ZipFile, encoding: &str, lazy_loaded_types: Vec<MediaType>) -> Result<EpubBook, io::Error> {
        let resources = ResourcesLoader::load_resources_lazy(zip_file, encoding, lazy_loaded_types);
        self.read_epub_resources(resources)
    }

    pub fn read_epub_resources(&self, resources: Resources) -> EpubBook {
        self.read_epub_resources_book(resources, EpubBook::new())
    }

    pub fn read_epub_resources_book(&self, mut resources: Resources, mut result: EpubBook) -> EpubBook {
        if result == null {
            result = EpubBook::new();
        }
        handle_mime_type(&mut result, &mut resources);
        let package_resource_href = get_package_resource_href(&mut resources);
        let package_resource = process_package_resource(package_resource_href, &mut result, &mut resources);
        result.set_opf_resource(package_resource);
        let ncx_resource = process_ncx_resource(package_resource, &mut result);
        result.set_ncx_resource(ncx_resource);
        result = post_process_book(result);
        result
    }

    fn post_process_book(&self, book: EpubBook) -> EpubBook {
        if self.book_processor != null {
            book.process_book(book)
        } else {
            book
        }
    }

    fn process_ncx_resource(&self, package_resource: Option<Resource>, book: &mut EpubBook) -> Option<Resource> {
        println!("{} OPF:getHref(){}", EpubReader::TAG, package_resource.as_ref().unwrap().get_href());
        if book.is_epub3() {
            NCXDocumentV3::read(book, self)
        } else {
            NCXDocumentV2::read(book, self)
        }
    }

    fn process_package_resource(&self, package_resource_href: String, book: &mut EpubBook, resources: &mut Resources) -> Option<Resource> {
        let package_resource = resources.remove(&package_resource_href);
        match PackageDocumentReader::read(&package_resource, self, book, resources) {
            Ok(_) => {}
            Err(e) => {
                e.printStackTrace();
                // Log.e(TAG, e.getMessage(), e);
            }
        }
        package_resource
    }

    fn get_package_resource_href(resources: &mut Resources) -> String {
        let default_result = "OEBPS/content.opf";
        let mut result = default_result.to_string();

        let container_resource = resources.remove("META-INF/container.xml");
        if container_resource == null {
            return result;
        }
        match ResourceUtil::get_as_document(&container_resource) {
            Ok(document) => {
                let root_file_element = document.get_document_element().get_elements_by_tag_name("rootfiles").item(0).get_elements_by_tag_name("rootfile").item(0);
                result = root_file_element.get_attribute("full-path");
            }
            Err(e) => {
                e.printStackTrace();
                // Log.e(TAG, e.getMessage(), e);
            }
        }
        if StringUtil::is_blank(&result) {
            result = default_result.to_string();
        }
        result
    }

    fn handle_mime_type(result: &mut EpubBook, resources: &mut Resources) {
        resources.remove("mimetype");
        //result.setResources(resources);
    }
}

pub struct IdentityBookProcessor;

impl BookProcessor for IdentityBookProcessor {
    fn process_book(&self, book: EpubBook) -> Result<EpubBook, Box<dyn std::error::Error>> {
        Ok(book)
    }
}

pub struct ZipInputStream;
pub struct ZipFile;
pub struct MediaTypes;
pub struct NullType;

impl MediaTypes {
    pub const media_types: [MediaType; 0] = [];
}

impl EpubReader {
    pub const TAG: &'static str = "me.ag2s.epublib.epub.EpubReader";
}
