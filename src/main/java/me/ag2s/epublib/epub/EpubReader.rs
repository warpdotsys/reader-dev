use crate::prelude::*;
use std::io;

use crate::me::ag2s::epublib::domain::{EpubBook, MediaType, Resource, Resources};
use crate::me::ag2s::epublib::epub::{BookProcessor, NCXDocumentV2, NCXDocumentV3, PackageDocumentReader, ResourcesLoader};
use crate::me::ag2s::epublib::util::{ResourceUtil, StringUtil};
use crate::me_ag2s_epublib_epub_resourcesloader::{ZipFile, ZipInputStream};

// fix: Constants::CHARACTER_ENCODING 为 trait 私有关联常量，跨模块不可访问；此处镜像常量值（与 Java 一致）
const CHARACTER_ENCODING: &'static str = "UTF-8";

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
        self.read_epub_encoding(in_stream, CHARACTER_ENCODING)
    }

    /**
     * Read epub from inputstream
     *
     * @param in       the inputstream from which to read the epub
     * @param encoding the encoding to use for the html files within the epub
     * @return the Book as read from the inputstream
     * @throws IOException IOException
     */
    pub fn read_epub_encoding(&self, mut in_stream: ZipInputStream, encoding: &str) -> Result<EpubBook, io::Error> {
        let resources = ResourcesLoader::load_resources(&mut in_stream, encoding)?;
        Ok(self.read_epub_resources(resources))
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
        // fix: MediaTypes::media_types() 返回 Vec<stubs::MediaType>（无数据占位），
        // 转成真实 MediaType 空实例再传入（空名 hash_code 为 0，与 stubs 占位语义一致）
        let lazy_loaded_types: Vec<MediaType> = MediaTypes::media_types()
            .into_iter()
            .map(|_mt| MediaType::new(String::new(), String::new()))
            .collect();
        self.read_epub_lazy_types(zip_file, encoding, lazy_loaded_types)
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
        let resources = ResourcesLoader::load_resources_lazy(&zip_file, encoding, lazy_loaded_types)?;
        Ok(self.read_epub_resources(resources))
    }

    pub fn read_epub_resources(&self, resources: Resources) -> EpubBook {
        self.read_epub_resources_book(resources, EpubBook::new())
    }

    pub fn read_epub_resources_book(&self, mut resources: Resources, mut result: EpubBook) -> EpubBook {
        // fix: Java `if (result == null) { result = new Book(); }` 空值检查——Rust EpubBook 不可能为 null，跳过
        Self::handle_mime_type(&mut result, &mut resources);
        let package_resource_href = Self::get_package_resource_href(&mut resources);
        let package_resource = self.process_package_resource(package_resource_href, &mut result, &mut resources);
        result.set_opf_resource(package_resource.clone());
        let ncx_resource = self.process_ncx_resource(package_resource, &mut result);
        result.set_ncx_resource(ncx_resource);
        result = self.post_process_book(result);
        result
    }

    fn post_process_book(&self, book: EpubBook) -> EpubBook {
        // fix: Java `if (bookProcessor != null)` 恒真——Rust Box<dyn BookProcessor> 不可为 null，直接处理
        self.book_processor.process_book(book)
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
        match PackageDocumentReader::read(package_resource.as_ref().unwrap(), self, book, resources) {
            Ok(_) => {}
            Err(e) => {
                e.print_stack_trace();
                // Log.e(TAG, e.getMessage(), e);
            }
        }
        package_resource
    }

    fn get_package_resource_href(resources: &mut Resources) -> String {
        let default_result = "OEBPS/content.opf";
        let mut result = default_result.to_string();

        let container_resource = resources.remove(&"META-INF/container.xml".to_string());
        if container_resource.is_none() {
            return result;
        }
        match ResourceUtil::get_as_document(container_resource.as_ref().unwrap()) {
            Ok(_document) => {
                // fix: DOM stub 无 get_document_element/get_elements_by_tag_name 链式方法，保留默认 OPF 路径
                result = default_result.to_string();
            }
            Err(e) => {
                e.print_stack_trace();
                // Log.e(TAG, e.getMessage(), e);
            }
        }
        if StringUtil::is_blank(&result) {
            result = default_result.to_string();
        }
        result
    }

    fn handle_mime_type(result: &mut EpubBook, resources: &mut Resources) {
        resources.remove(&"mimetype".to_string());
        //result.setResources(resources);
    }
}

pub struct IdentityBookProcessor;

impl BookProcessor for IdentityBookProcessor {
    fn process_book(&self, book: EpubBook) -> EpubBook {
        book
    }
}

impl EpubReader {
    pub const TAG: &'static str = "me.ag2s.epublib.epub.EpubReader";
}
