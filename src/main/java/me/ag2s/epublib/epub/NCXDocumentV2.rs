use crate::me::ag2s::epublib::Constants;
use crate::me::ag2s::epublib::domain::{Author, EpubBook, Identifier, MediaTypes, Resource, TableOfContents, TOCReference};
use crate::me::ag2s::epublib::epub::{DOMUtil, EpubProcessorSupport, EpubReader, EpubWriter};
use crate::me::ag2s::epublib::util::{ResourceUtil, StringUtil};

/**
 * Writes the ncx document as defined by namespace http://www.daisy.org/z3986/2005/ncx/
 *
 * @author paul
 */
pub struct NCXDocumentV2;

impl NCXDocumentV2 {

    pub const NAMESPACE_NCX: &'static str = "http://www.daisy.org/z3986/2005/ncx/";
    #[allow(dead_code)]
    pub const PREFIX_NCX: &'static str = "ncx";
    pub const NCX_ITEM_ID: &'static str = "ncx";
    pub const DEFAULT_NCX_HREF: &'static str = "toc.ncx";
    pub const PREFIX_DTB: &'static str = "dtb";

    const TAG: &'static str = "me.ag2s.epublib.epub.NCXDocumentV2";

    const NCX_TAGS_NCX: &'static str = "ncx";
    const NCX_TAGS_META: &'static str = "meta";
    const NCX_TAGS_NAV_POINT: &'static str = "navPoint";
    const NCX_TAGS_NAV_MAP: &'static str = "navMap";
    const NCX_TAGS_NAV_LABEL: &'static str = "navLabel";
    const NCX_TAGS_CONTENT: &'static str = "content";
    const NCX_TAGS_TEXT: &'static str = "text";
    const NCX_TAGS_DOC_TITLE: &'static str = "docTitle";
    const NCX_TAGS_DOC_AUTHOR: &'static str = "docAuthor";
    const NCX_TAGS_HEAD: &'static str = "head";

    const NCX_ATTRIBUTES_SRC: &'static str = "src";
    const NCX_ATTRIBUTES_NAME: &'static str = "name";
    const NCX_ATTRIBUTES_CONTENT: &'static str = "content";
    const NCX_ATTRIBUTES_ID: &'static str = "id";
    const NCX_ATTRIBUTES_PLAY_ORDER: &'static str = "playOrder";
    const NCX_ATTRIBUTES_CLAZZ: &'static str = "class";
    const NCX_ATTRIBUTES_VERSION: &'static str = "version";

    const NCX_ATTRIBUTE_VALUES_CHAPTER: &'static str = "chapter";
    const NCX_ATTRIBUTE_VALUES_VERSION: &'static str = "2005-1";

    #[allow(dead_code)]
    pub fn read(book: &mut EpubBook, epub_reader: &EpubReader) -> Option<Resource> {
        let ncx_resource;
        if book.get_spine().get_toc_resource() == null {
            // Log.e(TAG, "Book does not contain a table of contents file");
            eprintln!("{} Book does not contain a table of contents file", NCXDocumentV2::TAG);
            return None;
        }
        match {
            ncx_resource = book.get_spine().get_toc_resource();
            if ncx_resource == null {
                return None;
            }
            // Log.d(TAG, ncxResource.getHref());
            println!("{} ncxResource.getHref(){}", NCXDocumentV2::TAG, ncx_resource.get_href());
            let ncx_document = ResourceUtil::get_as_document(&ncx_resource);
            let nav_map_element = DOMUtil::get_first_element_by_tag_name_ns(
                ncx_document.get_document_element(), NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_NAV_MAP);
            if nav_map_element == null {
                return None;
            }

            let table_of_contents = TableOfContents::new(
                read_toc_references(nav_map_element.get_child_nodes(), book));
            book.set_table_of_contents(table_of_contents);
            Ok(())
        } {
            Ok(_) => {}
            Err(e) => {
                e.printStackTrace();
                // Log.e(TAG, e.getMessage(), e);
            }
        }
        ncx_resource
    }

    fn read_toc_references(navpoints: NodeList, book: &EpubBook) -> Vec<TOCReference> {
        if navpoints == null {
            return Vec::new();
        }
        let mut result = Vec::with_capacity(navpoints.get_length());
        for i in 0..navpoints.get_length() {
            let node = navpoints.item(i);
            if node.get_node_type() != Document::ELEMENT_NODE {
                continue;
            }
            if !(node.get_local_name() == NCXDocumentV2::NCX_TAGS_NAV_POINT) {
                continue;
            }
            let toc_reference = read_toc_reference(node, book);
            result.push(toc_reference);
        }
        result
    }

    fn read_toc_reference(navpoint_element: &Element, book: &EpubBook) -> TOCReference {
        let label = read_nav_label(navpoint_element);
        //Log.d(TAG,"label:"+label);
        let mut toc_resource_root = StringUtil::substring_before_last(book.get_spine().get_toc_resource().get_href(), '/');
        if toc_resource_root.len() == book.get_spine().get_toc_resource().get_href().len() {
            toc_resource_root = "".to_string();
        } else {
            toc_resource_root = toc_resource_root + "/";
        }
        let reference = StringUtil::collapse_path_dots(toc_resource_root + read_nav_reference(navpoint_element));
        let href = StringUtil::substring_before(reference, Constants::FRAGMENT_SEPARATOR_CHAR);
        let fragment_id = StringUtil::substring_after(reference, Constants::FRAGMENT_SEPARATOR_CHAR);
        let resource = book.get_resources().get_by_href(&href);
        if resource == null {
            eprintln!("{} Resource with href {} in NCX document not found", NCXDocumentV2::TAG, href);
            // Log.e(TAG, "Resource with href " + href + " in NCX document not found");
        }
        println!("{} label:{}", NCXDocumentV2::TAG, label);
        println!("{} href:{}", NCXDocumentV2::TAG, href);
        println!("{} fragmentId:{}", NCXDocumentV2::TAG, fragment_id);
        let mut result = TOCReference::new(label, resource, fragment_id);
        let child_toc_references = read_toc_references(navpoint_element.get_child_nodes(), book);
        result.set_children(child_toc_references);
        result
    }

    fn read_nav_reference(navpoint_element: &Element) -> String {
        let content_element = DOMUtil::get_first_element_by_tag_name_ns(navpoint_element, NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_CONTENT);
        if content_element == null {
            return null;
        }
        let mut result = DOMUtil::get_attribute(content_element, NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_ATTRIBUTES_SRC);
        match decode_url(result.clone(), Constants::CHARACTER_ENCODING) {
            Ok(decoded) => result = decoded,
            Err(e) => {
                e.printStackTrace();
                // Log.e(TAG, e.getMessage());
            }
        }
        result
    }

    fn read_nav_label(navpoint_element: &Element) -> String {
        //Log.d(TAG,navpointElement.getTagName());
        let nav_label = DOMUtil::get_first_element_by_tag_name_ns(navpoint_element, NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_NAV_LABEL);
        assert!(nav_label != null);
        DOMUtil::get_text_children_content(DOMUtil::get_first_element_by_tag_name_ns(nav_label, NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_TEXT))
    }

    #[allow(dead_code)]
    pub fn write(epub_writer: &EpubWriter, book: &EpubBook, result_stream: &mut ZipOutputStream) -> Result<(), std::io::Error> {
        result_stream.put_next_entry(ZipEntry::new(book.get_spine().get_toc_resource().get_href()));
        let mut out = EpubProcessorSupport::create_xml_serializer_stream(result_stream.clone());
        write_serializer(&mut out, book);
        out.flush();
        Ok(())
    }

    /**
     * Generates a resource containing an xml document containing the table of contents of the book in ncx format.
     *
     * @param xmlSerializer the serializer used
     * @param book          the book to serialize
     * @throws IOException              IOException
     * @throws IllegalStateException    IllegalStateException
     * @throws IllegalArgumentException IllegalArgumentException
     */
    pub fn write_serializer(xml_serializer: &mut XmlSerializer, book: &EpubBook) -> Result<(), NcxError> {
        write_full(xml_serializer, book.get_metadata().get_identifiers().clone(), book.get_title(),
            book.get_metadata().get_authors().clone(), book.get_table_of_contents())
    }

    pub fn create_ncx_resource(book: &EpubBook) -> Result<Resource, NcxError> {
        create_ncx_resource_full(book.get_metadata().get_identifiers().clone(),
            book.get_title(), book.get_metadata().get_authors().clone(),
            book.get_table_of_contents())
    }

    pub fn create_ncx_resource_full(identifiers: Vec<Identifier>,
                             title: String, authors: Vec<Author>, table_of_contents: TableOfContents) -> Result<Resource, NcxError> {
        let mut data = ByteArrayOutputStream::new();
        let mut out = EpubProcessorSupport::create_xml_serializer_stream(data.clone());
        write_full(&mut out, identifiers, title, authors, table_of_contents);
        Resource::new(NCXDocumentV2::NCX_ITEM_ID.to_string(), data.to_byte_array(),
            NCXDocumentV2::DEFAULT_NCX_HREF.to_string(), MediaTypes::NCX)
    }

    pub fn write_full(serializer: &mut XmlSerializer,
               identifiers: Vec<Identifier>, title: String, authors: Vec<Author>,
               table_of_contents: TableOfContents) -> Result<(), NcxError> {
        serializer.start_document(Constants::CHARACTER_ENCODING, false);
        serializer.set_prefix(EpubWriter::EMPTY_NAMESPACE_PREFIX, NCXDocumentV2::NAMESPACE_NCX);
        serializer.start_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_NCX);
        //		serializer.writeNamespace("ncx", NAMESPACE_NCX);
        //		serializer.attribute("xmlns", NAMESPACE_NCX);
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, NCXDocumentV2::NCX_ATTRIBUTES_VERSION,
            NCXDocumentV2::NCX_ATTRIBUTE_VALUES_VERSION);
        serializer.start_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_HEAD);

        for identifier in &identifiers {
            write_meta_element(identifier.get_scheme(), identifier.get_value(),
                serializer);
        }

        write_meta_element("generator", Constants::EPUB_GENERATOR_NAME, serializer);
        write_meta_element("depth", table_of_contents.calculate_depth().to_string(),
            serializer);
        write_meta_element("totalPageCount", "0", serializer);
        write_meta_element("maxPageNumber", "0", serializer);

        serializer.end_tag(NCXDocumentV2::NAMESPACE_NCX, "head");

        serializer.start_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_DOC_TITLE);
        serializer.start_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_TEXT);
        // write the first title
        serializer.text(StringUtil::default_if_null(&title));
        serializer.end_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_TEXT);
        serializer.end_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_DOC_TITLE);

        for author in &authors {
            serializer.start_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_DOC_AUTHOR);
            serializer.start_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_TEXT);
            serializer.text(author.get_lastname() + ", " + author.get_firstname());
            serializer.end_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_TEXT);
            serializer.end_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_DOC_AUTHOR);
        }

        serializer.start_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_NAV_MAP);
        write_nav_points(table_of_contents.get_toc_references(), 1, serializer);
        serializer.end_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_NAV_MAP);

        serializer.end_tag(NCXDocumentV2::NAMESPACE_NCX, "ncx");
        serializer.end_document();
        Ok(())
    }

    fn write_meta_element(dtb_name: &str, content: &str,
                          serializer: &mut XmlSerializer) -> Result<(), NcxError> {
        serializer.start_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_META);
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, NCXDocumentV2::NCX_ATTRIBUTES_NAME,
            NCXDocumentV2::PREFIX_DTB.to_string() + ":" + dtb_name);
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, NCXDocumentV2::NCX_ATTRIBUTES_CONTENT,
            content);
        serializer.end_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_META);
        Ok(())
    }

    fn write_nav_points(toc_references: Vec<TOCReference>,
                        mut play_order: i32,
                        serializer: &mut XmlSerializer) -> Result<i32, NcxError> {
        for toc_reference in toc_references {
            if toc_reference.get_resource() == null {
                play_order = write_nav_points(toc_reference.get_children(), play_order,
                    serializer)?;
                continue;
            }
            write_nav_point_start(&toc_reference, play_order, serializer)?;
            play_order += 1;
            if !toc_reference.get_children().is_empty() {
                play_order = write_nav_points(toc_reference.get_children(), play_order,
                    serializer)?;
            }
            write_nav_point_end(&toc_reference, serializer)?;
        }
        Ok(play_order)
    }

    fn write_nav_point_start(toc_reference: &TOCReference,
                             play_order: i32, serializer: &mut XmlSerializer) -> Result<(), NcxError> {
        serializer.start_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_NAV_POINT);
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, NCXDocumentV2::NCX_ATTRIBUTES_ID,
            "navPoint-".to_string() + &play_order.to_string());
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, NCXDocumentV2::NCX_ATTRIBUTES_PLAY_ORDER,
            play_order.to_string());
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, NCXDocumentV2::NCX_ATTRIBUTES_CLAZZ,
            NCXDocumentV2::NCX_ATTRIBUTE_VALUES_CHAPTER);
        serializer.start_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_NAV_LABEL);
        serializer.start_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_TEXT);
        serializer.text(&toc_reference.get_title());
        serializer.end_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_TEXT);
        serializer.end_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_NAV_LABEL);
        serializer.start_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_CONTENT);
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, NCXDocumentV2::NCX_ATTRIBUTES_SRC,
            toc_reference.get_complete_href());
        serializer.end_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_CONTENT);
        Ok(())
    }

    #[allow(dead_code)]
    fn write_nav_point_end(toc_reference: &TOCReference,
                           serializer: &mut XmlSerializer) -> Result<(), NcxError> {
        serializer.end_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_NAV_POINT);
        Ok(())
    }
}

pub struct Document;
pub struct Element;
pub struct Node;
pub struct NodeList;
pub struct ZipOutputStream;
pub struct ZipEntry;
pub struct XmlSerializer;
pub struct ByteArrayOutputStream;
pub struct NcxError;

impl Document {
    pub const ELEMENT_NODE: u16 = 1;
}

impl Node {
    pub fn get_node_type(&self) -> u16 { todo!() }
    pub fn get_local_name(&self) -> String { todo!() }
}

impl NodeList {
    pub fn get_length(&self) -> usize { todo!() }
    pub fn item(&self, _i: usize) -> Element { todo!() }
}

impl Element {
    pub fn get_child_nodes(&self) -> NodeList { todo!() }
}

impl ZipOutputStream {
    pub fn put_next_entry(&mut self, _entry: ZipEntry) { todo!() }
}

impl Clone for ZipOutputStream {
    fn clone(&self) -> Self { todo!() }
}

impl ZipEntry {
    pub fn new(_name: String) -> Self { todo!() }
}

impl XmlSerializer {
    pub fn start_document(&mut self, _encoding: &str, _standalone: bool) { todo!() }
    pub fn set_prefix(&mut self, _prefix: &str, _namespace: &str) { todo!() }
    pub fn start_tag(&mut self, _namespace: &str, _name: &str) { todo!() }
    pub fn attribute(&mut self, _namespace: &str, _name: &str, _value: String) { todo!() }
    pub fn end_tag(&mut self, _namespace: &str, _name: &str) { todo!() }
    pub fn text(&mut self, _text: &str) { todo!() }
    pub fn end_document(&mut self) { todo!() }
    pub fn flush(&mut self) { todo!() }
}

impl ByteArrayOutputStream {
    pub fn new() -> Self { todo!() }
    pub fn to_byte_array(&self) -> Vec<u8> { todo!() }
}

impl Clone for ByteArrayOutputStream {
    fn clone(&self) -> Self { todo!() }
}

fn decode_url(_s: String, _encoding: &str) -> Result<String, ()> {
    todo!()
}

pub type InputStream = Vec<u8>;
