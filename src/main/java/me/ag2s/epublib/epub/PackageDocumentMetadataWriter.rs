use crate::prelude::*;
use std::collections::HashMap;

use crate::me::ag2s::epublib::domain::{Author, Date, EpubBook, Identifier};
use crate::me::ag2s::epublib::epub::{EpubWriter, PackageDocumentBase};
use crate::me::ag2s::epublib::util::StringUtil;

// fix: PackageDocumentBase 的这些关联常量是私有的（E0624），此处按原值以模块级常量镜像，保持原逻辑不变
const OPF_TAGS_METADATA: &'static str = "metadata";
const OPF_TAGS_META: &'static str = "meta";
const DC_TAGS_TITLE: &'static str = "title";
const DC_TAGS_CREATOR: &'static str = "creator";
const DC_TAGS_SUBJECT: &'static str = "subject";
const DC_TAGS_DESCRIPTION: &'static str = "description";
const DC_TAGS_PUBLISHER: &'static str = "publisher";
const DC_TAGS_CONTRIBUTOR: &'static str = "contributor";
const DC_TAGS_DATE: &'static str = "date";
const DC_TAGS_TYPE: &'static str = "type";
const DC_TAGS_IDENTIFIER: &'static str = "identifier";
const DC_TAGS_RIGHTS: &'static str = "rights";
const DC_ATTRIBUTES_ID: &'static str = "id";
const OPF_ATTRIBUTES_ROLE: &'static str = "role";
const OPF_ATTRIBUTES_FILE_AS: &'static str = "file-as";
const OPF_ATTRIBUTES_EVENT: &'static str = "event";
const OPF_ATTRIBUTES_SCHEME: &'static str = "scheme";
const OPF_ATTRIBUTES_NAME: &'static str = "name";
const OPF_ATTRIBUTES_CONTENT: &'static str = "content";
const OPF_ATTRIBUTES_PROPERTY: &'static str = "property";
const OPF_VALUES_META_COVER: &'static str = "cover";
const OPF_VALUES_GENERATOR: &'static str = "generator";
const OPF_VALUES_DUOKAN: &'static str = "duokan-body-font";
// fix: E0790——Constants 是 trait，其关联常量不能以 `Constants::X` 引用，按原值镜像为模块常量
const EPUB_GENERATOR_NAME: &'static str = "Ag2S EpubLib";
const EPUB_DUOKAN_NAME: &'static str = "DK-SONGTI";

pub struct PackageDocumentMetadataWriter;

impl PackageDocumentMetadataWriter {

    /**
     * Writes the book's metadata.
     *
     * @param book       book
     * @param serializer serializer
     * @throws IOException              IOException
     * @throws IllegalStateException    IllegalStateException
     * @throws IllegalArgumentException IllegalArgumentException
     */
    pub fn write_meta_data(book: &EpubBook, serializer: &mut XmlSerializer) -> Result<(), SerError> {
        serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_METADATA);
        serializer.set_prefix(PackageDocumentBase::PREFIX_DUBLIN_CORE, PackageDocumentBase::NAMESPACE_DUBLIN_CORE);
        serializer.set_prefix(PackageDocumentBase::PREFIX_OPF, PackageDocumentBase::NAMESPACE_OPF);

        Self::write_identifiers(book.get_metadata().get_identifiers(), serializer)?;
        Self::write_simple_metdata_elements(DC_TAGS_TITLE, book.get_metadata().get_titles(),
            serializer)?;
        Self::write_simple_metdata_elements(DC_TAGS_SUBJECT, book.get_metadata().get_subjects(),
            serializer)?;
        Self::write_simple_metdata_elements(DC_TAGS_DESCRIPTION,
            book.get_metadata().get_descriptions(), serializer)?;
        Self::write_simple_metdata_elements(DC_TAGS_PUBLISHER,
            book.get_metadata().get_publishers(), serializer)?;
        Self::write_simple_metdata_elements(DC_TAGS_TYPE, book.get_metadata().get_types(),
            serializer)?;
        Self::write_simple_metdata_elements(DC_TAGS_RIGHTS, book.get_metadata().get_rights(),
            serializer)?;

        // write authors
        for author in book.get_metadata().get_authors() {
            serializer.start_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, DC_TAGS_CREATOR);
            serializer.attribute(PackageDocumentBase::NAMESPACE_OPF, OPF_ATTRIBUTES_ROLE,
                author.get_relator().get_code().to_string());
            serializer.attribute(PackageDocumentBase::NAMESPACE_OPF, OPF_ATTRIBUTES_FILE_AS,
                format!("{}, {}", author.get_lastname(), author.get_firstname()));
            serializer.text(&format!("{} {}", author.get_firstname(), author.get_lastname()));
            serializer.end_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, DC_TAGS_CREATOR);
        }

        // write contributors
        for author in book.get_metadata().get_contributors() {
            serializer.start_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, DC_TAGS_CONTRIBUTOR);
            serializer.attribute(PackageDocumentBase::NAMESPACE_OPF, OPF_ATTRIBUTES_ROLE,
                author.get_relator().get_code().to_string());
            serializer.attribute(PackageDocumentBase::NAMESPACE_OPF, OPF_ATTRIBUTES_FILE_AS,
                format!("{}, {}", author.get_lastname(), author.get_firstname()));
            serializer.text(&format!("{} {}", author.get_firstname(), author.get_lastname()));
            serializer.end_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, DC_TAGS_CONTRIBUTOR);
        }

        // write dates
        for date in book.get_metadata().get_dates() {
            serializer.start_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, DC_TAGS_DATE);
            if date.get_event().is_some() {
                serializer.attribute(PackageDocumentBase::NAMESPACE_OPF, OPF_ATTRIBUTES_EVENT,
                    date.get_event().as_ref().unwrap().to_string());
            }
            serializer.text(&date.get_value());
            serializer.end_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, DC_TAGS_DATE);
        }

        // write language
        if StringUtil::is_not_blank(&book.get_metadata().get_language()) {
            serializer.start_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, "language");
            serializer.text(&book.get_metadata().get_language());
            serializer.end_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, "language");
        }

        // write other properties
        if !book.get_metadata().get_other_properties().is_empty() {
            for (map_entry) in book.get_metadata().get_other_properties().iter() {
                serializer.start_tag(&map_entry.0.namespace_uri(), OPF_TAGS_META);
                serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX,
                    OPF_ATTRIBUTES_PROPERTY, map_entry.0.local_part());
                serializer.text(map_entry.1);
                serializer.end_tag(&map_entry.0.namespace_uri(), OPF_TAGS_META);
            }
        }

        // write coverimage
        if book.get_cover_image().is_some() { // write the cover image
            serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_META);
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_NAME,
                OPF_VALUES_META_COVER.to_string());
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_CONTENT,
                book.get_cover_image().as_ref().unwrap().get_id().to_string());
            serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_META);
        }

        // write generator
        serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_META);
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_NAME,
            OPF_VALUES_GENERATOR.to_string());
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_CONTENT,
            EPUB_GENERATOR_NAME.to_string());
        serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_META);

        // write duokan
        serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_META);
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_NAME,
            OPF_VALUES_DUOKAN.to_string());
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, OPF_ATTRIBUTES_CONTENT,
            EPUB_DUOKAN_NAME.to_string());
        serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_META);

        serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, OPF_TAGS_METADATA);
        Ok(())
    }

    fn write_simple_metdata_elements(tag_name: &str, values: &Vec<String>, serializer: &mut XmlSerializer) -> Result<(), SerError> {
        for value in values {
            if StringUtil::is_blank(value) {
                continue;
            }
            serializer.start_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, tag_name);
            serializer.text(value);
            serializer.end_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, tag_name);
        }
        Ok(())
    }

    /**
     * Writes out the complete list of Identifiers to the package document.
     * The first identifier for which the bookId is true is made the bookId identifier.
     * If no identifier has bookId == true then the first bookId identifier is written as the primary.
     *
     * @param identifiers identifiers
     * @param serializer serializer
     * @throws IllegalStateException e
     * @throws IllegalArgumentException e
     * @
     */
    fn write_identifiers(identifiers: &Vec<Identifier>, serializer: &mut XmlSerializer) -> Result<(), SerError> {
        let book_id_identifier = Identifier::get_book_id_identifier(identifiers);
        if book_id_identifier.is_none() {
            return Ok(());
        }
        let book_id_identifier = book_id_identifier.unwrap();

        serializer.start_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, DC_TAGS_IDENTIFIER);
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, DC_ATTRIBUTES_ID,
            PackageDocumentBase::BOOK_ID_ID.to_string());
        serializer.attribute(PackageDocumentBase::NAMESPACE_OPF, OPF_ATTRIBUTES_SCHEME,
            book_id_identifier.get_scheme().to_string());
        serializer.text(&book_id_identifier.get_value());
        serializer.end_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, DC_TAGS_IDENTIFIER);

        for identifier in &identifiers[1..identifiers.len()] {
            // fix: Identifier 无 PartialEq，按 Identifier::equals 语义（scheme 与 value 均相等）比较
            if identifier.get_scheme() == book_id_identifier.get_scheme()
                && identifier.get_value() == book_id_identifier.get_value() {
                continue;
            }
            serializer.start_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, DC_TAGS_IDENTIFIER);
            serializer.attribute(PackageDocumentBase::NAMESPACE_OPF, "scheme", identifier.get_scheme().to_string());
            serializer.text(&identifier.get_value());
            serializer.end_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, DC_TAGS_IDENTIFIER);
        }
        Ok(())
    }
}

// fix: 真实 XML 序列化（原全空实现——EPUB 导出 OPF 空文件；buf 拼接 + take_output 由写入方取走）
pub struct XmlSerializer {
    buf: String,
    open_tag: Option<String>,
}
pub struct SerError;

impl XmlSerializer {
    pub fn new() -> Self {
        XmlSerializer { buf: String::new(), open_tag: None }
    }
    fn close_open_tag(&mut self) {
        if self.open_tag.take().is_some() {
            self.buf.push('>');
        }
    }
    pub fn start_document(&mut self, _encoding: &str, _standalone: bool) {
        self.buf = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        self.open_tag = None;
    }
    pub fn start_tag(&mut self, _namespace: &str, name: &str) {
        self.close_open_tag();
        self.buf.push('<');
        self.buf.push_str(name);
        self.open_tag = Some(name.to_string());
    }
    pub fn set_prefix(&mut self, _prefix: &str, _namespace: &str) {}
    pub fn attribute(&mut self, _namespace: &str, name: &str, value: String) {
        let v = value.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;").replace('>', "&gt;");
        self.buf.push(' ');
        self.buf.push_str(name);
        self.buf.push_str("=\"");
        self.buf.push_str(&v);
        self.buf.push('"');
    }
    pub fn text(&mut self, text: &str) {
        self.close_open_tag();
        let t = text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
        self.buf.push_str(&t);
    }
    pub fn end_tag(&mut self, _namespace: &str, name: &str) {
        self.close_open_tag();
        self.buf.push_str("</");
        self.buf.push_str(name);
        self.buf.push('>');
    }
    pub fn end_document(&mut self) {
        self.close_open_tag();
    }
    pub fn flush(&mut self) {}
    // fix: 写入方取走序列化结果（原 out 为占位 Writer，内容丢失）
    pub fn take_output(&mut self) -> String {
        self.close_open_tag();
        std::mem::take(&mut self.buf)
    }
}
