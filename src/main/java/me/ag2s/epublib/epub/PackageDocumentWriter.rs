use crate::me::ag2s::epublib::Constants;
use crate::me::ag2s::epublib::domain::{EpubBook, Guide, GuideReference, MediaTypes, Resource, Spine, SpineReference};
use crate::me::ag2s::epublib::epub::{EpubWriter, NCXDocumentV3, PackageDocumentBase, PackageDocumentMetadataWriter};
use crate::me::ag2s::epublib::util::StringUtil;

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
        match {
            serializer.start_document(Constants::CHARACTER_ENCODING, false);
            serializer.set_prefix(PackageDocumentBase::PREFIX_OPF, PackageDocumentBase::NAMESPACE_OPF);
            serializer.set_prefix(PackageDocumentBase::PREFIX_DUBLIN_CORE, PackageDocumentBase::NAMESPACE_DUBLIN_CORE);
            serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_PACKAGE_TAG);
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_VERSION,
                book.get_version());
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX,
                PackageDocumentBase::OPF_ATTRIBUTES_UNIQUE_IDENTIFIER, PackageDocumentBase::BOOK_ID_ID);

            PackageDocumentMetadataWriter::write_meta_data(book, serializer)?;

            write_manifest(book, epub_writer, serializer)?;
            write_spine(book, epub_writer, serializer)?;
            write_guide(book, epub_writer, serializer)?;

            serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_PACKAGE_TAG);
            serializer.end_document();
            serializer.flush();
            Ok(())
        } {
            Ok(_) => {}
            Err(e) => {
                e.printStackTrace();
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
        serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_SPINE);
        let toc_resource = book.get_spine().get_toc_resource();
        let toc_resource_id = toc_resource.get_id();
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_TOC,
            toc_resource_id);

        if book.get_cover_page() != null // there is a cover page
            && book.get_spine().find_first_resource_by_id(&book.get_cover_page().get_id())
            < 0 { // cover page is not already in the spine
            // write the cover html file
            serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_ITEMREF);
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_IDREF,
                book.get_cover_page().get_id());
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_LINEAR,
                "no");
            serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_ITEMREF);
        }
        write_spine_items(&book.get_spine(), serializer)?;
        serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_SPINE);
        Ok(())
    }

    fn write_manifest(book: &EpubBook, epub_writer: &EpubWriter,
                      serializer: &mut XmlSerializer) -> Result<(), SerError> {
        serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_MANIFEST);

        serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_ITEM);

        //For EPUB3
        if book.is_epub3() {
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_PROPERTIES, NCXDocumentV3::V3_NCX_PROPERTIES);
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_ID, NCXDocumentV3::NCX_ITEM_ID);
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_HREF, NCXDocumentV3::DEFAULT_NCX_HREF);
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_MEDIA_TYPE, NCXDocumentV3::V3_NCX_MEDIATYPE.get_name());
        } else {
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_ID,
                epub_writer.get_ncx_id());
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_HREF, epub_writer.get_ncx_href());
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_MEDIA_TYPE, epub_writer.get_ncx_media_type());
        }

        serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_ITEM);

        //		writeCoverResources(book, serializer);

        for resource in get_all_resources_sort_by_id(book) {
            write_item(book, resource, serializer)?;
        }

        serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_MANIFEST);
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
        if resource == null ||
            (resource.get_media_type() == MediaTypes::NCX
                && book.get_spine().get_toc_resource() != null) {
            return Ok(());
        }
        if StringUtil::is_blank(&resource.get_id()) {
            //      log.error("resource id must not be empty (href: " + resource.getHref()
            //          + ", mediatype:" + resource.getMediaType() + ")");
            eprintln!("{} {} resource id must not be empty (href: {}, mediatype: {})", PackageDocumentWriter::TAG, " ", resource.get_href(), resource.get_media_type());
            return Ok(());
        }
        if StringUtil::is_blank(&resource.get_href()) {
            //      log.error("resource href must not be empty (id: " + resource.getId()
            //          + ", mediatype:" + resource.getMediaType() + ")");
            eprintln!("{} {} resource href must not be empty (id: {}, mediatype: {})", PackageDocumentWriter::TAG, " ", resource.get_id(), resource.get_media_type());
            return Ok(());
        }
        if resource.get_media_type() == null {
            //      log.error("resource mediatype must not be empty (id: " + resource.getId()
            //          + ", href:" + resource.getHref() + ")");
            eprintln!("{} {} resource mediatype must not be empty (id: {}, href: {})", PackageDocumentWriter::TAG, " ", resource.get_id(), resource.get_href());
            return Ok(());
        }
        serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_ITEM);
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_ID,
            resource.get_id());
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_HREF,
            resource.get_href());
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_MEDIA_TYPE,
            resource.get_media_type().get_name());
        serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_ITEM);
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
            serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_ITEMREF);
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_IDREF,
                spine_reference.get_resource_id());
            if !spine_reference.is_linear() {
                serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_LINEAR,
                    PackageDocumentBase::OPF_VALUES_NO);
            }
            serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_ITEMREF);
        }
        Ok(())
    }

    fn write_guide(book: &EpubBook, epub_writer: &EpubWriter,
                   serializer: &mut XmlSerializer) -> Result<(), SerError> {
        serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_GUIDE);
        ensure_cover_page_guide_reference_written(&book.get_guide(), epub_writer,
            serializer)?;
        for reference in book.get_guide().get_references() {
            write_guide_reference(reference, serializer)?;
        }
        serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_GUIDE);
        Ok(())
    }

    #[allow(dead_code)]
    fn ensure_cover_page_guide_reference_written(guide: &Guide,
                                                 epub_writer: &EpubWriter, serializer: &mut XmlSerializer) -> Result<(), SerError> {
        if !(guide.get_guide_references_by_type(GuideReference::COVER).is_empty()) {
            return Ok(());
        }
        let cover_page = guide.get_cover_page();
        if cover_page != null {
            write_guide_reference(
                &GuideReference::new_full(guide.get_cover_page(), GuideReference::COVER,
                    GuideReference::COVER), serializer)?;
        }
        Ok(())
    }

    fn write_guide_reference(reference: &GuideReference,
                             serializer: &mut XmlSerializer) -> Result<(), SerError> {
        if reference == null {
            return Ok(());
        }
        serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_REFERENCE);
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_TYPE,
            reference.get_type());
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_HREF,
            reference.get_complete_href());
        if StringUtil::is_not_blank(&reference.get_title()) {
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_TITLE,
                reference.get_title());
        }
        serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_REFERENCE);
        Ok(())
    }
}

pub struct XmlSerializer;
pub struct SerError;

impl XmlSerializer {
    pub fn start_document(&mut self, _encoding: &str, _standalone: bool) { todo!() }
    pub fn set_prefix(&mut self, _prefix: &str, _namespace: &str) { todo!() }
    pub fn start_tag(&mut self, _namespace: &str, _name: &str) { todo!() }
    pub fn attribute(&mut self, _namespace: &str, _name: &str, _value: String) { todo!() }
    pub fn end_tag(&mut self, _namespace: &str, _name: &str) { todo!() }
    pub fn end_document(&mut self) { todo!() }
    pub fn flush(&mut self) { todo!() }
}
