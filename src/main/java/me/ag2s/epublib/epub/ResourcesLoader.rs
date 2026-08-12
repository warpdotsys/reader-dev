use crate::prelude::*;
use std::io;

use crate::me::ag2s::epublib::domain::{EpubResourceProvider, LazyResource, LazyResourceProvider, MediaType, MediaTypes, Resource, Resources};
use crate::me::ag2s::epublib::util::{CollectionUtil, ResourceUtil};

// fix: LazyResource::with_size 要求 `impl LazyResourceProvider`（该 trait 定义处签名已损坏 E0782，
// 此处补实现并委托给 EpubResourceProvider 的固有方法，返回类型以实际固有方法签名 Box<dyn InputStream> 为准）
impl LazyResourceProvider for EpubResourceProvider {
    fn get_resource_stream(&self, href: &String) -> Result<Box<dyn crate::stubs::InputStream>, IOException> {
        EpubResourceProvider::get_resource_stream(self, href)
    }
}

/**
 * Loads Resources from inputStreams, ZipFiles, etc
 *
 * @author paul
 */
pub struct ResourcesLoader;

impl ResourcesLoader {

    const TAG: &'static str = "me.ag2s.epublib.epub.ResourcesLoader";

    /**
     * Loads the entries of the zipFile as resources.
     * <p>
     * The MediaTypes that are in the lazyLoadedTypes will not get their
     * contents loaded, but are stored as references to entries into the
     * ZipFile and are loaded on demand by the Resource system.
     *
     * @param zipFile             import epub zipfile
     * @param defaultHtmlEncoding epub xhtml default encoding
     * @param lazyLoadedTypes     lazyLoadedTypes
     * @return Resources
     * @throws IOException IOException
     */
    pub fn load_resources_lazy(zip_file: &ZipFile,
                               default_html_encoding: &str,
                               lazy_loaded_types: Vec<MediaType>) -> Result<Resources, io::Error> {

        let mut result = Resources::new();
        let entries = zip_file.entries();

        for zip_entry in entries {
            // fix: ZipEntry 为非空结构体占位，省略 `== None` 判断
            if zip_entry.is_directory() {
                continue;
            }

            let href = zip_entry.get_name();

            let mut resource: Resource;

            if Self::should_load_lazy(&href, &lazy_loaded_types) {
                // fix: Rust 无继承——Java 中 LazyResource 是 Resource 子类可直接 add；
                // 转录后保留 LazyResource 构造语义（provider 接线），入库改用同 href 的占位 Resource
                let resource_provider = EpubResourceProvider::new(zip_file.get_name());
                let _lazy = LazyResource::with_size(resource_provider, zip_entry.get_size() as i64, href.clone());
                resource = Resource::new_bytes(Vec::new(), &href);
            } else {
                // fix: create_resource 已泛型化（ResourceUtil 侧修复），直接传本模块 ZipEntry 与真实输入流
                let mut resource_tmp = ResourceUtil::create_resource(&zip_entry, &zip_file.get_input_stream(&zip_entry))?;
                /*掌上书苑有很多自制书OPF的nameSpace格式不标准，强制修复成正确的格式*/
                if href.ends_with("opf") {
                    // fix: get_data 现返回 Result，先解包再读取
                    let data = resource_tmp.get_data().ok().cloned().unwrap_or_default();
                    let string = String::from_utf8_lossy(&data).replace("smlns=\"", "xmlns=\"");
                    resource_tmp.set_data(string.into_bytes());
                }
                resource = resource_tmp;
            }

            // fix: get_media_type 现返回 &Option<MediaType>，且 MediaType 无 PartialEq，用 hash_code 比较
            if resource.get_media_type().as_ref().map(|mt| mt.hash_code()) == Some(MediaTypes::XHTML.hash_code()) {
                resource.set_input_encoding(default_html_encoding.to_string());
            }
            result.add(resource);
        }

        Ok(result)
    }

    /**
     * Whether the given href will load a mediaType that is in the
     * collection of lazilyLoadedMediaTypes.
     *
     * @param href                   href
     * @param lazilyLoadedMediaTypes lazilyLoadedMediaTypes
     * @return Whether the given href will load a mediaType that is
     * in the collection of lazilyLoadedMediaTypes.
     */
    fn should_load_lazy(href: &str,
                        lazily_loaded_media_types: &Vec<MediaType>) -> bool {
        if lazily_loaded_media_types.is_empty() {
            return false;
        }
        let media_type = MediaTypes::determine_media_type(&href.to_string());
        // fix: determine_media_type 现返回 Option<MediaType>，且 MediaType 无 PartialEq，用 hash_code 比较
        media_type.map_or(false, |mt| {
            lazily_loaded_media_types.iter().any(|t| t.hash_code() == mt.hash_code())
        })
    }

    /**
     * Loads all entries from the ZipInputStream as Resources.
     * <p>
     * Loads the contents of all ZipEntries into memory.
     * Is fast, but may lead to memory problems when reading large books
     * on devices with small amounts of memory.
     *
     * @param zipInputStream      zipInputStream
     * @param defaultHtmlEncoding defaultHtmlEncoding
     * @return Resources
     * @throws IOException IOException
     */
    pub fn load_resources(zip_input_stream: &mut ZipInputStream,
                          default_html_encoding: &str) -> Result<Resources, io::Error> {
        let mut result = Resources::new();
        let mut zip_entry;
        loop {
            // get next valid zipEntry
            zip_entry = Self::get_next_zip_entry(zip_input_stream)?;
            // fix: ZipEntry 为非空结构体占位，省略 `== None` 判断（None 不可能出现）
            if zip_entry.is_directory() {
                continue;
            }
            let href = zip_entry.get_name();

            // store resource
            // fix: create_resource 已泛型化（ResourceUtil 侧修复），直接传本模块 ZipEntry 与输入流（&mut 自动重借用为 &）
            let mut resource = ResourceUtil::create_resource(&zip_entry, zip_input_stream)?;
            ///*掌上书苑有很多自制书OPF的nameSpace格式不标准，强制修复成正确的格式*/
            if href.ends_with("opf") {
                // fix: get_data 现返回 Result，先解包再读取
                let data = resource.get_data().ok().cloned().unwrap_or_default();
                let string = String::from_utf8_lossy(&data).replace("smlns=\"", "xmlns=\"");
                resource.set_data(string.into_bytes());
            }
            // fix: get_media_type 现返回 &Option<MediaType>，且 MediaType 无 PartialEq，用 hash_code 比较
            if resource.get_media_type().as_ref().map(|mt| mt.hash_code()) == Some(MediaTypes::XHTML.hash_code()) {
                resource.set_input_encoding(default_html_encoding.to_string());
            }
            result.add(resource);
        }

        Ok(result)
    }

    fn get_next_zip_entry(zip_input_stream: &mut ZipInputStream) -> Result<ZipEntry, io::Error> {
        match zip_input_stream.get_next_entry() {
            Ok(entry) => Ok(entry),
            Err(e) => {
                //see <a href="https://github.com/psiegman/epublib/issues/122">Issue #122 Infinite loop</a>.
                //when reading a file that is not a real zip archive or a zero length file, zipInputStream.getNextEntry()
                //throws an exception and does not advance, so loadResources enters an infinite loop
                //log.error("Invalid or damaged zip file.", e);
                // Log.e(TAG, e.getLocalizedMessage());
                // fix: ZipException 占位无 printStackTrace 方法，省略
                match zip_input_stream.close_entry() {
                    Ok(_) => {}
                    Err(_ignored) => {}
                }
                Err(io::Error::new(io::ErrorKind::InvalidData, "invalid zip entry"))
            }
        }
    }

    /**
     * Loads all entries from the ZipInputStream as Resources.
     * <p>
     * Loads the contents of all ZipEntries into memory.
     * Is fast, but may lead to memory problems when reading large books
     * on devices with small amounts of memory.
     *
     * @param zipFile             zipFile
     * @param defaultHtmlEncoding defaultHtmlEncoding
     * @return Resources
     * @throws IOException IOException
     */
    pub fn load_resources_from_zip_file(zip_file: &ZipFile, default_html_encoding: &str) -> Result<Resources, io::Error> {
        let ls: Vec<MediaType> = Vec::new();
        Self::load_resources_lazy(zip_file, default_html_encoding, ls)
    }
}

pub struct ZipFile;
pub struct ZipEntry;
pub struct ZipInputStream;
pub struct ZipException;

impl ZipFile {
    pub fn get_name(&self) -> String { todo!() }
    pub fn entries(&self) -> ZipEntryIter { todo!() }
    pub fn get_input_stream(&self, _entry: &ZipEntry) -> InputStream { todo!() }
}

pub struct ZipEntryIter;

impl Iterator for ZipEntryIter {
    type Item = ZipEntry;
    fn next(&mut self) -> Option<ZipEntry> { todo!() }
}

impl ZipEntry {
    pub fn is_directory(&self) -> bool { todo!() }
    pub fn get_name(&self) -> String { todo!() }
    pub fn get_size(&self) -> u64 { todo!() }
}

impl ZipInputStream {
    pub fn get_next_entry(&mut self) -> Result<ZipEntry, ZipException> { todo!() }
    pub fn close_entry(&mut self) -> Result<(), ZipException> { todo!() }
}

pub type InputStream = Vec<u8>;
