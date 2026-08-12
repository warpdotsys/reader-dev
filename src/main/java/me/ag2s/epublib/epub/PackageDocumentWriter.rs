use crate::prelude::*;
use crate::me::ag2s::epublib::domain::{EpubBook, Guide, GuideReference, MediaTypes, Resource, Spine, SpineReference};
use crate::me::ag2s::epublib::epub::{EpubWriter, NCXDocumentV3, PackageDocumentBase, PackageDocumentMetadataWriter};
use crate::me::ag2s::epublib::util::StringUtil;
use crate::me_ag2s_epublib_epub_packagedocumentmetadatawriter::{SerError, XmlSerializer};

// fix: Constants::CHARACTER_ENCODING 未 pub，跨模块不可访问；此处镜像常量值（与 Java 一致）
const CHARACTER_ENCODING: &'static str = "UTF-8";
// fix: PackageDocumentBase 中相关 OPF_* 常量未 pub，跨模块不可访问；此处镜像常量值（与 Java 一致）
const OPF_TAGS_MANIFEST: &'static str = "manifest";
const OPF_TAGS_PACKAGE_TAG: &'static str = "package";
const OPF_TAGS_ITEMREF: &'static str = "itemref";
const OPF_TAGS_SPINE: &'static str = "spine";
const OPF_TAGS_REFERENCE: &'static str = "reference";
const OPF_TAGS_GUIDE: &'static str = "guide";
const OPF_TAGS_ITEM: &'static str = "item";
const OPF_ATTRIBUTES_UNIQUE_IDENTIFIER: &'static str = "unique-identifier";
const OPF_ATTRIBUTES_IDREF: &'static str = "idref";
const OPF_ATTRIBUTES_TYPE: &'static str = "type";
const OPF_ATTRIBUTES_HREF: &'static str = "href";
const OPF_ATTRIBUTES_LINEAR: &'static str = "linear";
const OPF_ATTRIBUTES_ID: &'static str = "id";
const OPF_ATTRIBUTES_MEDIA_TYPE: &'static str = "media-type";
const OPF_ATTRIBUTES_TITLE: &'static str = "title";
const OPF_ATTRIBUTES_TOC: &'static str = "toc";
const OPF_ATTRIBUTES_VERSION: &'static str = "version";
const OPF_ATTRIBUTES_PROPERTIES: &'static str = "properties";
const OPF_VALUES_NO: &'static str = "no";

/**
 * Writes the opf package document as defined by namespace http://www.idpf.org/2007/opf
 *
 * @author paul
 */
pub struct PackageDocumentWriter;

impl PackageDocumentWriter {

    const TAG: &'static str = "me.ag2s.epublib.epub.PackageDocumentWriter";

    pub fn write(epub_writer: &EpubWriter, serializer: &mut XmlSerializer,
                 book: &EpubBook) {
        match (|| -> Result<(), SerError> {
            serializer.start_document(CHARACTER_ENCODING, false);
            serializer.set_prefix(PackageDocumentBase::PREFIX_OPF, PackageDocumentBase::NAMESPACE_OPF);
            serializer.set_prefix(PackageDocumentBase::PREFIX_DUBLIN_CORE, PackageDocumentBase::NAMESPACE_DUBLIN_CORE);
            serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_PACKAGE_TAG);
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_VERSION,
                book.get_version().clone());
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX,
                OPF_ATTRIBUTES_UNIQUE_IDENTIFIER, PackageDocumentBase::BOOK_ID_ID.to_string());

            PackageDocumentMetadataWriter::write_meta_data(book, serializer)?;

            Self::write_manifest(book, epub_writer, serializer)?;
            Self::write_spine(book, epub_writer, serializer)?;
            Self::write_guide(book, epub_writer, serializer)?;

            serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_PACKAGE_TAG);
            serializer.end_document();
            serializer.flush();
            Ok(())
        })() {
            Ok(_) => {}
            Err(e) => {
                e.print_stack_trace();
            }
        }
    }

    /**
     * Writes the package's spine.
     *
     * @param book e
     * @param epubWriter g
     * @param serializer g
     * @throws IOException g
     * @throws IllegalStateException g
     * @throws IllegalArgumentException 1@throws XMLStreamException
     */
    #[allow(dead_code)]
    fn write_spine(book: &EpubBook, epub_writer: &EpubWriter,
                   serializer: &mut XmlSerializer) -> Result<(), SerError> {
        serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_SPINE);
        let toc_resource = book.get_spine().get_toc_resource();
        // fix: Java 的 getTocResource() 返回可空 Resource；Rust 版返回 &Option<Resource>，空时降级为空串
        let toc_resource_id = toc_resource.as_ref().map(|r| r.get_id().clone()).unwrap_or_default();
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_TOC,
            toc_resource_id);

        let cover_page = book.get_cover_page();
        if cover_page.is_some() // there is a cover page
            && book.get_spine().find_first_resource_by_id(cover_page.as_ref().unwrap().get_id())
            < 0 { // cover page is not already in the spine
            // write the cover html file
            serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_ITEMREF);
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_IDREF,
                cover_page.as_ref().unwrap().get_id().clone());
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_LINEAR,
                "no".to_string());
            serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_ITEMREF);
        }
        Self::write_spine_items(book.get_spine(), serializer)?;
        serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_SPINE);
        Ok(())
    }

    fn write_manifest(book: &EpubBook, epub_writer: &EpubWriter,
                      serializer: &mut XmlSerializer) -> Result<(), SerError> {
        serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_MANIFEST);

        serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_ITEM);

        //For EPUB3
        if book.is_epub3() {
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_PROPERTIES, NCXDocumentV3::V3_NCX_PROPERTIES.to_string());
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_ID, NCXDocumentV3::NCX_ITEM_ID.to_string());
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_HREF, NCXDocumentV3::DEFAULT_NCX_HREF.to_string());
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_MEDIA_TYPE, NCXDocumentV3::V3_NCX_MEDIATYPE.get_name().clone());
        } else {
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_ID,
                epub_writer.get_ncx_id());
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_HREF, epub_writer.get_ncx_href());
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_MEDIA_TYPE, epub_writer.get_ncx_media_type());
        }

        serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_ITEM);

        //		writeCoverResources(book, serializer);

        for resource in Self::get_all_resources_sort_by_id(book) {
            Self::write_item(book, &resource, serializer)?;
        }

        serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_MANIFEST);
        Ok(())
    }

    fn get_all_resources_sort_by_id(book: &EpubBook) -> Vec<Resource> {
        let mut all_resources: Vec<Resource> = book.get_resources().get_all();
        all_resources.sort_by(|resource1, resource2| resource1.get_id().to_lowercase().cmp(&resource2.get_id().to_lowercase()));
        all_resources
    }

    /**
     * Writes a resources as an item element
     *
     * @param resource   g
     * @param serializer g
     * @throws IOException              g
     * @throws IllegalStateException    g
     * @throws IllegalArgumentException 1@throws XMLStreamException
     */
    fn write_item(book: &EpubBook, resource: &Resource,
                  serializer: &mut XmlSerializer) -> Result<(), SerError> {
        // fix: get_all() 返回 Vec<Resource>，元素不可能是 null；Java 的 resource == null 恒假，故省略该条件
        if (resource.get_media_type().as_ref().map_or(false, |mt| mt.get_name() == MediaTypes::NCX.get_name())
            && book.get_spine().get_toc_resource().is_some()) {
            return Ok(());
        }
        if StringUtil::is_blank(&resource.get_id()) {
            //      log.error("resource id must not be empty (href: " + resource.getHref()
            //          + ", mediatype:" + resource.getMediaType() + ")");
            eprintln!("{} {} resource id must not be empty (href: {}, mediatype: {})", PackageDocumentWriter::TAG, " ", resource.get_href(), resource.get_media_type().as_ref().map(|mt| mt.to_string()).unwrap_or_default());
            return Ok(());
        }
        if StringUtil::is_blank(&resource.get_href()) {
            //      log.error("resource href must not be empty (id: " + resource.getId()
            //          + ", mediatype:" + resource.getMediaType() + ")");
            eprintln!("{} {} resource href must not be empty (id: {}, mediatype: {})", PackageDocumentWriter::TAG, " ", resource.get_id(), resource.get_media_type().as_ref().map(|mt| mt.to_string()).unwrap_or_default());
            return Ok(());
        }
        // fix: Java 的 getMediaType() 返回可空 MediaType；Rust 版返回 &Option<MediaType>
        if resource.get_media_type().is_none() {
            //      log.error("resource mediatype must not be empty (id: " + resource.getId()
            //          + ", href:" + resource.getHref() + ")");
            eprintln!("{} {} resource mediatype must not be empty (id: {}, href: {})", PackageDocumentWriter::TAG, " ", resource.get_id(), resource.get_href());
            return Ok(());
        }
        serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_ITEM);
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_ID,
            resource.get_id().clone());
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_HREF,
            resource.get_href().clone());
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_MEDIA_TYPE,
            resource.get_media_type().as_ref().unwrap().get_name().clone());
        serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_ITEM);
        Ok(())
    }

    /**
     * List all spine references
     *
     * @throws IOException f
     * @throws IllegalStateException f
     * @throws IllegalArgumentException f
     */
    #[allow(dead_code)]
    fn write_spine_items(spine: &Spine, serializer: &mut XmlSerializer) -> Result<(), SerError> {
        for spine_reference in spine.get_spine_references() {
            serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_ITEMREF);
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_IDREF,
                spine_reference.get_resource_id().unwrap_or_default());
            if !spine_reference.is_linear() {
                serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_LINEAR,
                    OPF_VALUES_NO.to_string());
            }
            serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_ITEMREF);
        }
        Ok(())
    }

    fn write_guide(book: &EpubBook, epub_writer: &EpubWriter,
                   serializer: &mut XmlSerializer) -> Result<(), SerError> {
        serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_GUIDE);
        Self::ensure_cover_page_guide_reference_written(book.get_guide(), epub_writer,
            serializer)?;
        for reference in book.get_guide().get_references() {
            Self::write_guide_reference(reference, serializer)?;
        }
        serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_GUIDE);
        Ok(())
    }

    #[allow(dead_code)]
    fn ensure_cover_page_guide_reference_written(guide: &Guide,
                                                 epub_writer: &EpubWriter, serializer: &mut XmlSerializer) -> Result<(), SerError> {
        // fix: Rust 版 get_guide_references_by_type 参数为 &String
        if !(guide.get_guide_references_by_type(&GuideReference::COVER.to_string()).is_empty()) {
            return Ok(());
        }
        // fix: Java 的 guide.getCoverPage() 需 &mut Guide，此处无法获取可变借用；
        // 等价地直接取首个 COVER 类型引用的资源（与 getCoverPage 的 lazy 缓存语义一致）
        let cover_page: Option<Resource> = guide.get_guide_references_by_type(&GuideReference::COVER.to_string())
            .first()
            .and_then(|r| r.get_resource().clone());
        if let Some(cover_page) = cover_page {
            // fix: GuideReference 无 new_full，Java 构造函数 new GuideReference(resource, type, title) 对应 with_type
            Self::write_guide_reference(
                &GuideReference::with_type(Some(cover_page), GuideReference::COVER.to_string(),
                    GuideReference::COVER.to_string()), serializer)?;
        }
        Ok(())
    }

    fn write_guide_reference(reference: &GuideReference,
                             serializer: &mut XmlSerializer) -> Result<(), SerError> {
        // fix: 转录后为 &GuideReference（不存在 null），Java 的 reference == null 恒假，故省略该检查
        serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_REFERENCE);
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_TYPE,
            reference.get_type().clone());
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_HREF,
            reference.get_complete_href());
        // fix: Java 的 getTitle() 返回可空 String；Rust 版返回 &Option<String>
        if reference.get_title().as_ref().map_or(false, |t| StringUtil::is_not_blank(t)) {
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_TITLE,
                reference.get_title().clone().unwrap_or_default());
        }
        serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_REFERENCE);
        Ok(())
    }
}

// fix: XmlSerializer/SerError 复用 PackageDocumentMetadataWriter 的本地 stub 类型，
// 以保证 write_meta_data(book, serializer) 的类型一致；此处仅补充本文件用到的缺失方法。
impl XmlSerializer {
    pub fn start_document(&mut self, _encoding: &str, _standalone: bool) {
        // fix: 占位 stub（降级实现，不做实际输出）
    }
    pub fn end_document(&mut self) {
        // fix: 占位 stub（降级实现，不做实际输出）
    }
    pub fn flush(&mut self) {
        // fix: 占位 stub（降级实现，不做实际输出）
    }
}

impl ThrowableExt for SerError {
    fn localized_message(&self) -> String {
        String::new()
    }
    fn stack_trace_to_string(&self) -> String {
        String::new()
    }
    fn msg(&self) -> Option<String> {
        None
    }
}
