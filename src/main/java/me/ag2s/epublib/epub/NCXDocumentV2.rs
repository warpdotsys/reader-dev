use crate::prelude::*;
use crate::me::ag2s::epublib::Constants;
use crate::me::ag2s::epublib::domain::{Author, EpubBook, Identifier, MediaTypes, Resource, TableOfContents, TOCReference};
use crate::me::ag2s::epublib::epub::{DOMUtil, EpubProcessorSupport, EpubReader, EpubWriter};
use crate::me::ag2s::epublib::util::{ResourceUtil, StringUtil};
use crate::me_ag2s_epublib_epub_domutil::{Document, Element, NodeList};
use crate::me_ag2s_epublib_epub_epubprocessorsupport::{OutputStream, XmlSerializer};
use crate::stubs::{ByteArrayOutputStream, NcxError, ZipEntry, ZipOutputStream};

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
    pub fn read(book: &mut EpubBook, _epub_reader: &EpubReader) -> Option<Resource> {
        let toc_resource = book.get_spine().get_toc_resource();
        if toc_resource.is_none() {
            // Log.e(TAG, "Book does not contain a table of contents file");
            eprintln!("{} Book does not contain a table of contents file", NCXDocumentV2::TAG);
            return None;
        }
        let ncx_resource = toc_resource.as_ref().unwrap();
        let result: Result<Option<TableOfContents>, NcxError> = (|| {
            // Log.d(TAG, ncxResource.getHref());
            println!("{} ncxResource.getHref(){}", NCXDocumentV2::TAG, ncx_resource.get_href());
            // fix: RU::Document 为空 stub; get_document_element 为本文件扩展方法, 返回占位根元素
            let ncx_document = ResourceUtil::get_as_document(ncx_resource).map_err(|_| NcxError)?;
            let nav_map_element = DOMUtil::get_first_element_by_tag_name_ns(
                &ncx_document.get_document_element(), NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_NAV_MAP);
            if nav_map_element.is_none() {
                return Ok(None);
            }
            Ok(Some(TableOfContents::with_references(
                Self::read_toc_references(nav_map_element.unwrap().get_child_nodes(), book))))
        })();
        // fix: Resource 无 Clone; Java 直接返回 ncxResource, 按 get_data/get_href 重建占位资源
        let result_resource = Resource::with_id_data(
            None,
            Some(ncx_resource.get_data().unwrap().clone()),
            Some(ncx_resource.get_href().clone()),
            None);
        match result {
            Ok(None) => return None,
            Ok(Some(table_of_contents)) => book.set_table_of_contents(table_of_contents),
            Err(e) => {
                e.printStackTrace();
                // Log.e(TAG, e.getMessage(), e);
            }
        }
        Some(result_resource)
    }

    fn read_toc_references(navpoints: NodeList, book: &EpubBook) -> Vec<TOCReference> {
        // fix: Java `navpoints == null` 检查; NodeList 非 Option, 以长度 0 等价占位
        if navpoints.get_length() == 0 {
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
            let toc_reference = Self::read_toc_reference(&node, book);
            result.push(toc_reference);
        }
        result
    }

    fn read_toc_reference(navpoint_element: &Element, book: &EpubBook) -> TOCReference {
        let label = Self::read_nav_label(navpoint_element);
        //Log.d(TAG,"label:"+label);
        let mut toc_resource_root = StringUtil::substring_before_last(book.get_spine().get_toc_resource().as_ref().unwrap().get_href(), '/');
        if toc_resource_root.len() == book.get_spine().get_toc_resource().as_ref().unwrap().get_href().len() {
            toc_resource_root = "".to_string();
        } else {
            toc_resource_root = toc_resource_root + "/";
        }
        let reference = StringUtil::collapse_path_dots(&(toc_resource_root + &Self::read_nav_reference(navpoint_element)));
        // fix: E0790——Constants 是 trait，其关联常量不能以 `Constants::X` 引用，改用字面量（值即 '#'）
        let href = StringUtil::substring_before(&reference, '#');
        let fragment_id = StringUtil::substring_after(&reference, '#');
        let resource = book.get_resources().get_by_href(&href);
        if resource.is_none() {
            eprintln!("{} Resource with href {} in NCX document not found", NCXDocumentV2::TAG, href);
            // Log.e(TAG, "Resource with href " + href + " in NCX document not found");
        }
        println!("{} label:{}", NCXDocumentV2::TAG, label);
        println!("{} href:{}", NCXDocumentV2::TAG, href);
        println!("{} fragmentId:{}", NCXDocumentV2::TAG, fragment_id);
        let mut result = TOCReference::with_fragment(Some(label), resource, Some(fragment_id));
        let child_toc_references = Self::read_toc_references(navpoint_element.get_child_nodes(), book);
        result.set_children(child_toc_references);
        result
    }

    fn read_nav_reference(navpoint_element: &Element) -> String {
        let content_element = DOMUtil::get_first_element_by_tag_name_ns(navpoint_element, NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_CONTENT);
        if content_element.is_none() {
            // fix: Java 此处返回 null, Rust 用空串占位
            return String::new();
        }
        let mut result = DOMUtil::get_attribute(content_element.as_ref().unwrap(), NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_ATTRIBUTES_SRC);
        // fix: E0790——Constants 是 trait，其关联常量不能以 `Constants::X` 引用，改用字面量（值即 "UTF-8"）
        match decode_url(result.clone(), "UTF-8") {
            Ok(decoded) => result = decoded,
            Err(_) => {
                // Log.e(TAG, e.getMessage());
            }
        }
        result
    }

    fn read_nav_label(navpoint_element: &Element) -> String {
        //Log.d(TAG,navpointElement.getTagName());
        let nav_label = DOMUtil::get_first_element_by_tag_name_ns(navpoint_element, NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_NAV_LABEL);
        assert!(nav_label.is_some());
        // fix: Java 文本子节点可能为 null, 以空串占位
        DOMUtil::get_text_children_content(
            DOMUtil::get_first_element_by_tag_name_ns(nav_label.as_ref().unwrap(), NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_TEXT).as_ref().unwrap()).unwrap_or_default()
    }

    #[allow(dead_code)]
    pub fn write(_epub_writer: &EpubWriter, book: &EpubBook, result_stream: &mut ZipOutputStream) -> Result<(), std::io::Error> {
        result_stream.put_next_entry(&ZipEntry::new(book.get_spine().get_toc_resource().as_ref().unwrap().get_href().clone()));
        // fix: Java 将 resultStream 传入 createXmlSerializer; EPS::OutputStream 为空 stub, 用单元占位
        let mut out = EpubProcessorSupport::create_xml_serializer_stream(OutputStream);
        let _ = Self::write_serializer(&mut out, book);
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
        Self::write_full(xml_serializer, book.get_metadata().get_identifiers(), book.get_title(),
            book.get_metadata().get_authors(), book.get_table_of_contents())
    }

    pub fn create_ncx_resource(book: &EpubBook) -> Result<Resource, NcxError> {
        Self::create_ncx_resource_full(book.get_metadata().get_identifiers(),
            book.get_title(), book.get_metadata().get_authors(),
            book.get_table_of_contents())
    }

    pub fn create_ncx_resource_full(identifiers: &Vec<Identifier>,
                             title: String, authors: &Vec<Author>, table_of_contents: &TableOfContents) -> Result<Resource, NcxError> {
        let data = ByteArrayOutputStream::new();
        // fix: Java 将 data 传入 createXmlSerializer; EPS::OutputStream 为空 stub, 用单元占位
        let mut out = EpubProcessorSupport::create_xml_serializer_stream(OutputStream);
        let _ = Self::write_full(&mut out, identifiers, title, authors, table_of_contents);
        Ok(Resource::with_id_data(Some(NCXDocumentV2::NCX_ITEM_ID.to_string()), Some(data.to_byte_array()),
            Some(NCXDocumentV2::DEFAULT_NCX_HREF.to_string()), Some(MediaTypes::NCX)))
    }

    pub fn write_full(serializer: &mut XmlSerializer,
               identifiers: &Vec<Identifier>, title: String, authors: &Vec<Author>,
               table_of_contents: &TableOfContents) -> Result<(), NcxError> {
        // fix: E0790——Constants 是 trait，其关联常量不能以 `Constants::X` 引用，改用字面量（值即 "UTF-8"）
        serializer.start_document("UTF-8", false);
        serializer.set_prefix(EpubWriter::EMPTY_NAMESPACE_PREFIX, NCXDocumentV2::NAMESPACE_NCX);
        serializer.start_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_NCX);
        //		serializer.writeNamespace("ncx", NAMESPACE_NCX);
        //		serializer.attribute("xmlns", NAMESPACE_NCX);
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, NCXDocumentV2::NCX_ATTRIBUTES_VERSION,
            NCXDocumentV2::NCX_ATTRIBUTE_VALUES_VERSION.to_string());
        serializer.start_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_HEAD);

        for identifier in identifiers {
            Self::write_meta_element(identifier.get_scheme(), identifier.get_value(),
                serializer);
        }

        // fix: E0790——Constants 是 trait，其关联常量不能以 `Constants::X` 引用，改用字面量（值即 "Ag2S EpubLib"）
        Self::write_meta_element("generator", "Ag2S EpubLib", serializer);
        Self::write_meta_element("depth", &table_of_contents.calculate_depth().to_string(),
            serializer);
        Self::write_meta_element("totalPageCount", "0", serializer);
        Self::write_meta_element("maxPageNumber", "0", serializer);

        serializer.end_tag(NCXDocumentV2::NAMESPACE_NCX, "head");

        serializer.start_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_DOC_TITLE);
        serializer.start_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_TEXT);
        // write the first title
        serializer.text(&StringUtil::default_if_null(&title));
        serializer.end_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_TEXT);
        serializer.end_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_DOC_TITLE);

        for author in authors {
            serializer.start_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_DOC_AUTHOR);
            serializer.start_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_TEXT);
            serializer.text(&format!("{}, {}", author.get_lastname(), author.get_firstname()));
            serializer.end_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_TEXT);
            serializer.end_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_DOC_AUTHOR);
        }

        serializer.start_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_NAV_MAP);
        let _ = Self::write_nav_points(table_of_contents.get_toc_references(), 1, serializer);
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
            content.to_string());
        serializer.end_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_META);
        Ok(())
    }

    fn write_nav_points(toc_references: &Vec<TOCReference>,
                        mut play_order: i32,
                        serializer: &mut XmlSerializer) -> Result<i32, NcxError> {
        for toc_reference in toc_references {
            if toc_reference.get_resource().is_none() {
                play_order = Self::write_nav_points(toc_reference.get_children(), play_order,
                    serializer)?;
                continue;
            }
            Self::write_nav_point_start(toc_reference, play_order, serializer)?;
            play_order += 1;
            if !toc_reference.get_children().is_empty() {
                play_order = Self::write_nav_points(toc_reference.get_children(), play_order,
                    serializer)?;
            }
            Self::write_nav_point_end(toc_reference, serializer)?;
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
            NCXDocumentV2::NCX_ATTRIBUTE_VALUES_CHAPTER.to_string());
        serializer.start_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_NAV_LABEL);
        serializer.start_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_TEXT);
        serializer.text(toc_reference.get_title().as_ref().map(|s| s.as_str()).unwrap_or_default());
        serializer.end_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_TEXT);
        serializer.end_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_NAV_LABEL);
        serializer.start_tag(NCXDocumentV2::NAMESPACE_NCX, NCXDocumentV2::NCX_TAGS_CONTENT);
        // fix: unwrap_or(&String::new()) 临时值语句末即析构（E0716），默认值提为具名变量
        let default_href = String::new();
        let href = toc_reference.get_resource().as_ref().map(|r| r.get_href()).unwrap_or(&default_href);
        // fix: TOCReference 无 get_complete_href, 按 Java 语义 inline: href + "#" + fragmentId
        let complete_href = match toc_reference.get_fragment_id().as_ref() {
            // fix: E0790——Constants 是 trait，其关联常量不能以 `Constants::X` 引用，改用字面量（值即 '#'）
            Some(fragment_id) => format!("{}{}{}", href, '#', fragment_id),
            None => href.clone(),
        };
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, NCXDocumentV2::NCX_ATTRIBUTES_SRC,
            complete_href);
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

fn decode_url(s: String, _encoding: &str) -> Result<String, ()> {
    let bytes = s.as_bytes();
    let mut result: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(high), Some(low)) = (hex_value(bytes[i + 1]), hex_value(bytes[i + 2])) {
                result.push(high * 16 + low);
                i += 3;
                continue;
            }
        }
        result.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(result).map_err(|_| ())
}

fn hex_value(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

// ---- DOM / XML 序列化 stub 扩展（NCXDocumentV2 专用; 避免改动其他转录文件） ----

impl Document {
    pub const ELEMENT_NODE: u16 = 1;
}

impl crate::me_ag2s_epublib_util_resourceutil::Document {
    // fix: 真实解析（原占位恒 null 根 → NCX 目录解析失败）
    pub fn get_document_element(&self) -> Element {
        if self.html.is_empty() {
            Element::null()
        } else {
            crate::me_ag2s_epublib_epub_domutil::Document::parse(&self.html)
                .root
                .clone()
                .unwrap_or_else(Element::null)
        }
    }
}

impl XmlSerializer {
    // fix: EPS::XmlSerializer 为空 stub, 补齐 NCX 写入所需方法
    pub fn start_document(&mut self, _encoding: &str, _standalone: bool) {}
    pub fn set_prefix(&mut self, _prefix: &str, _namespace: &str) {}
    pub fn start_tag(&mut self, _namespace: &str, _name: &str) {}
    pub fn attribute(&mut self, _namespace: &str, _name: &str, _value: String) {}
    pub fn end_tag(&mut self, _namespace: &str, _name: &str) {}
    pub fn text(&mut self, _text: &str) {}
    pub fn end_document(&mut self) {}
}
