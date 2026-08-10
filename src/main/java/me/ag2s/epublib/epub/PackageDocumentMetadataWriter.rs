use std::collections::HashMap;

use crate::me::ag2s::epublib::Constants;
use crate::me::ag2s::epublib::domain::{Author, Date, EpubBook, Identifier};
use crate::me::ag2s::epublib::epub::{EpubWriter, PackageDocumentBase};
use crate::me::ag2s::epublib::util::StringUtil;

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
        serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_METADATA);
        serializer.set_prefix(PackageDocumentBase::PREFIX_DUBLIN_CORE, PackageDocumentBase::NAMESPACE_DUBLIN_CORE);
        serializer.set_prefix(PackageDocumentBase::PREFIX_OPF, PackageDocumentBase::NAMESPACE_OPF);

        write_identifiers(book.get_metadata().get_identifiers(), serializer)?;
        write_simple_metdata_elements(PackageDocumentBase::DC_TAGS_TITLE, book.get_metadata().get_titles(),
            serializer)?;
        write_simple_metdata_elements(PackageDocumentBase::DC_TAGS_SUBJECT, book.get_metadata().get_subjects(),
            serializer)?;
        write_simple_metdata_elements(PackageDocumentBase::DC_TAGS_DESCRIPTION,
            book.get_metadata().get_descriptions(), serializer)?;
        write_simple_metdata_elements(PackageDocumentBase::DC_TAGS_PUBLISHER,
            book.get_metadata().get_publishers(), serializer)?;
        write_simple_metdata_elements(PackageDocumentBase::DC_TAGS_TYPE, book.get_metadata().get_types(),
            serializer)?;
        write_simple_metdata_elements(PackageDocumentBase::DC_TAGS_RIGHTS, book.get_metadata().get_rights(),
            serializer)?;

        // write authors
        for author in book.get_metadata().get_authors() {
            serializer.start_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, PackageDocumentBase::DC_TAGS_CREATOR);
            serializer.attribute(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_ATTRIBUTES_ROLE,
                author.get_relator().get_code());
            serializer.attribute(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_ATTRIBUTES_FILE_AS,
                author.get_lastname() + ", " + author.get_firstname());
            serializer.text(&(author.get_firstname() + " " + author.get_lastname()));
            serializer.end_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, PackageDocumentBase::DC_TAGS_CREATOR);
        }

        // write contributors
        for author in book.get_metadata().get_contributors() {
            serializer.start_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, PackageDocumentBase::DC_TAGS_CONTRIBUTOR);
            serializer.attribute(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_ATTRIBUTES_ROLE,
                author.get_relator().get_code());
            serializer.attribute(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_ATTRIBUTES_FILE_AS,
                author.get_lastname() + ", " + author.get_firstname());
            serializer.text(&(author.get_firstname() + " " + author.get_lastname()));
            serializer.end_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, PackageDocumentBase::DC_TAGS_CONTRIBUTOR);
        }

        // write dates
        for date in book.get_metadata().get_dates() {
            serializer.start_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, PackageDocumentBase::DC_TAGS_DATE);
            if date.get_event() != null {
                serializer.attribute(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_ATTRIBUTES_EVENT,
                    date.get_event().to_string());
            }
            serializer.text(&date.get_value());
            serializer.end_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, PackageDocumentBase::DC_TAGS_DATE);
        }

        // write language
        if StringUtil::is_not_blank(&book.get_metadata().get_language()) {
            serializer.start_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, "language");
            serializer.text(&book.get_metadata().get_language());
            serializer.end_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, "language");
        }

        // write other properties
        if book.get_metadata().get_other_properties() != null {
            for (map_entry) in book.get_metadata().get_other_properties().iter() {
                serializer.start_tag(map_entry.0.namespace_uri(), PackageDocumentBase::OPF_TAGS_META);
                serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX,
                    PackageDocumentBase::OPF_ATTRIBUTES_PROPERTY, map_entry.0.local_part());
                serializer.text(map_entry.1);
                serializer.end_tag(map_entry.0.namespace_uri(), PackageDocumentBase::OPF_TAGS_META);
            }
        }

        // write coverimage
        if book.get_cover_image() != null { // write the cover image
            serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_META);
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_NAME,
                PackageDocumentBase::OPF_VALUES_META_COVER);
            serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_CONTENT,
                book.get_cover_image().get_id());
            serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_META);
        }

        // write generator
        serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_META);
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_NAME,
            PackageDocumentBase::OPF_VALUES_GENERATOR);
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_CONTENT,
            Constants::EPUB_GENERATOR_NAME);
        serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_META);

        // write duokan
        serializer.start_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_META);
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_NAME,
            PackageDocumentBase::OPF_VALUES_DUOKAN);
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::OPF_ATTRIBUTES_CONTENT,
            Constants::EPUB_DUOKAN_NAME);
        serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_META);

        serializer.end_tag(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_METADATA);
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
    fn write_identifiers(identifiers: Vec<Identifier>, serializer: &mut XmlSerializer) -> Result<(), SerError> {
        let book_id_identifier = Identifier::get_book_id_identifier(&identifiers);
        if book_id_identifier == null {
            return Ok(());
        }

        serializer.start_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, PackageDocumentBase::DC_TAGS_IDENTIFIER);
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, PackageDocumentBase::DC_ATTRIBUTES_ID,
            PackageDocumentBase::BOOK_ID_ID);
        serializer.attribute(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_ATTRIBUTES_SCHEME,
            book_id_identifier.get_scheme());
        serializer.text(&book_id_identifier.get_value());
        serializer.end_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, PackageDocumentBase::DC_TAGS_IDENTIFIER);

        for identifier in &identifiers[1..identifiers.len()] {
            if identifier == book_id_identifier {
                continue;
            }
            serializer.start_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, PackageDocumentBase::DC_TAGS_IDENTIFIER);
            serializer.attribute(PackageDocumentBase::NAMESPACE_OPF, "scheme", identifier.get_scheme());
            serializer.text(&identifier.get_value());
            serializer.end_tag(PackageDocumentBase::NAMESPACE_DUBLIN_CORE, PackageDocumentBase::DC_TAGS_IDENTIFIER);
        }
        Ok(())
    }
}

pub struct XmlSerializer;
pub struct SerError;

impl XmlSerializer {
    pub fn start_tag(&mut self, _namespace: &str, _name: &str) { todo!() }
    pub fn set_prefix(&mut self, _prefix: &str, _namespace: &str) { todo!() }
    pub fn attribute(&mut self, _namespace: &str, _name: &str, _value: String) { todo!() }
    pub fn text(&mut self, _text: &str) { todo!() }
    pub fn end_tag(&mut self, _namespace: &str, _name: &str) { todo!() }
}
