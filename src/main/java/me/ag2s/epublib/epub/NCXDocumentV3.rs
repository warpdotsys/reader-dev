use crate::prelude::*;
use crate::me::ag2s::epublib::domain::{Author, EpubBook, Identifier, Resource, TableOfContents, TOCReference};
use crate::me::ag2s::epublib::epub::{DOMUtil, EpubProcessorSupport, EpubReader, EpubWriter, NCXDocumentV2};
use crate::me::ag2s::epublib::util::{ResourceUtil, StringUtil};
use crate::me_ag2s_epublib_epub_domutil::{Document, Element, NodeList};
use crate::me_ag2s_epublib_epub_epubprocessorsupport::{OutputStream, XmlSerializer};
use crate::me_ag2s_epublib_util_resourceutil::ParseError;
use crate::stubs::ByteArrayOutputStream;

/**
 * Writes the ncx document as defined by namespace http://www.daisy.org/z3986/2005/ncx/
 *
 * @author Ag2S20150909
 */

pub struct NCXDocumentV3;

impl NCXDocumentV3 {
    pub const NAMESPACE_XHTML: &'static str = "http://www.w3.org/1999/xhtml";
    pub const NAMESPACE_EPUB: &'static str = "http://www.idpf.org/2007/ops";
    pub const LANGUAGE: &'static str = "en";
    #[allow(dead_code)]
    pub const PREFIX_XHTML: &'static str = "html";
    pub const NCX_ITEM_ID: &'static str = "htmltoc";
    pub const DEFAULT_NCX_HREF: &'static str = "toc.xhtml";
    pub const V3_NCX_PROPERTIES: &'static str = "nav";
    // fix: MediaType 带数据（application/xhtml+xml / xhtml）
    pub const V3_NCX_MEDIATYPE: MediaType = MediaType::with_extensions("application/xhtml+xml", "xhtml", &["xhtml", "html"]);

    const TAG: &'static str = "me.ag2s.epublib.epub.NCXDocumentV3";

    const XHTML_TAGS_HTML: &'static str = "html";
    const XHTML_TAGS_HEAD: &'static str = "head";
    const XHTML_TAGS_TITLE: &'static str = "title";
    const XHTML_TAGS_META: &'static str = "meta";
    const XHTML_TAGS_LINK: &'static str = "link";
    const XHTML_TAGS_BODY: &'static str = "body";
    const XHTML_TAGS_H1: &'static str = "h1";
    const XHTML_TAGS_H2: &'static str = "h2";
    const XHTML_TAGS_NAV: &'static str = "nav";
    const XHTML_TAGS_OL: &'static str = "ol";
    const XHTML_TAGS_LI: &'static str = "li";
    const XHTML_TAGS_A: &'static str = "a";
    const XHTML_TAGS_SPAN: &'static str = "span";

    const XHTML_ATTRIBUTES_XMLNS: &'static str = "xmlns";
    const XHTML_ATTRIBUTES_XMLNS_EPUB: &'static str = "xmlns:epub";
    const XHTML_ATTRIBUTES_LANG: &'static str = "lang";
    const XHTML_ATTRIBUTES_XML_LANG: &'static str = "xml:lang";
    const XHTML_ATTRIBUTES_REL: &'static str = "rel";
    const XHTML_ATTRIBUTES_TYPE: &'static str = "type";
    const XHTML_ATTRIBUTES_EPUB_TYPE: &'static str = "epub:type";//nav的必须属性
    const XHTML_ATTRIBUTES_ID: &'static str = "id";
    const XHTML_ATTRIBUTES_ROLE: &'static str = "role";
    const XHTML_ATTRIBUTES_HREF: &'static str = "href";
    const XHTML_ATTRIBUTES_HTTP_EQUIV: &'static str = "http-equiv";
    const XHTML_ATTRIBUTES_CONTENT: &'static str = "content";

    const XHTML_ATTRIBUTE_VALUES_CONTENT_TYPE: &'static str = "Content-Type";
    const XHTML_ATTRIBUTE_VALUES_HTML_UTF8: &'static str = "text/html; charset=utf-8";
    const XHTML_ATTRIBUTE_VALUES_LANG: &'static str = "en";
    const XHTML_ATTRIBUTE_VALUES_EPUB_TYPE: &'static str = "toc";
    const XHTML_ATTRIBUTE_VALUES_ROLE_TOC: &'static str = "doc-toc";

    /**
     * 解析epub的目录文件
     *
     * @param book       Book
     * @param epubReader epubreader
     * @return Resource
     */
    #[allow(dead_code)]
    pub fn read(book: &mut EpubBook, epub_reader: &EpubReader) -> Option<Resource> {
        let toc_resource = book.get_spine().get_toc_resource();
        if toc_resource.is_none() {
            // Log.e(TAG, "Book does not contain a table of contents file");
            eprintln!("{} Book does not contain a table of contents file", NCXDocumentV3::TAG);
            return None;
        }
        let ncx_resource = toc_resource.as_ref().unwrap();
        //一些epub 3 文件没有按照epub3的标准使用删除掉ncx目录文件
        if ncx_resource.get_href().ends_with(".ncx") {
            // Log.v(TAG,"该epub文件不标准，使用了epub2的目录文件");
            eprintln!("{} 该epub文件不标准，使用了epub2的目录文件", NCXDocumentV3::TAG);
            return NCXDocumentV2::read(book, epub_reader);
        }
        // Log.d(TAG, ncxResource.getHref());
        println!("{} {}", NCXDocumentV3::TAG, ncx_resource.get_href());
        // fix: Resource 无 Clone; Java 最后返回 ncxResource, 这里先按 get_data/get_href 重建占位资源, 避免借用冲突
        let ncx_resource_owned = Resource::with_id_data(
            None,
            Some(ncx_resource.get_data().unwrap().clone()),
            Some(ncx_resource.get_href().clone()),
            None);

        let result: Result<Option<()>, ParseError> = (|| {
            let ncx_document = ResourceUtil::get_as_document(&ncx_resource_owned)?;
            // Log.d(TAG, ncxDocument.getNodeName());
            println!("{} {}", NCXDocumentV3::TAG, ncx_document.get_node_name());

            let nav_map_elements = ncx_document.get_elements_by_tag_name(NCXDocumentV3::XHTML_TAGS_NAV);
            if nav_map_elements.get_length() == 0 {
                // Log.d(TAG,"epub3目录文件未发现nav节点，尝试使用epub2的规则解析");
                println!("{} {}", NCXDocumentV3::TAG, "epub3目录文件未发现nav节点，尝试使用epub2的规则解析");
                return Ok(None);
            }
            let nav_map_element = nav_map_elements.item(0);
            // Log.d(TAG, navMapElement.getTagName());
            println!("{} {}", NCXDocumentV3::TAG, nav_map_element.get_tag_name());
            let nav_map_element = nav_map_element.get_elements_by_tag_name(NCXDocumentV3::XHTML_TAGS_OL).item(0);

            let table_of_contents = TableOfContents::with_references(
                Self::read_toc_references(nav_map_element.get_child_nodes(), book));
            // Log.d(TAG, tableOfContents.toString());
            println!("{} {}", NCXDocumentV3::TAG, table_of_contents.calculate_depth());
            book.set_table_of_contents(table_of_contents);
            Ok(Some(()))
        })();
        match result {
            Ok(None) => return NCXDocumentV2::read(book, epub_reader),
            Ok(Some(_)) => {}
            Err(e) => {
                e.print_stack_trace();
                // Log.e(TAG, e.getMessage(), e);
            }
        }
        Some(ncx_resource_owned)
    }

    fn do_toc(n: &Element, book: &EpubBook) -> Vec<TOCReference> {
        let mut result = Vec::new();
        let el = n;
        let node_list = el.get_elements_by_tag_name(NCXDocumentV3::XHTML_TAGS_LI);
        for i in 0..node_list.get_length() {
            let element = node_list.item(i);
            result.push(Self::read_toc_reference(&element, book));
        }
        result
    }

    fn read_toc_references(navpoints: NodeList, book: &EpubBook) -> Vec<TOCReference> {
        // fix: Java `navpoints == null` 检查; NodeList 非 Option, 以长度 0 等价占位
        if navpoints.get_length() == 0 {
            return Vec::new();
        }
        //Log.d(TAG, "readTOCReferences:navpoints.getLength()" + navpoints.getLength());
        let mut result = Vec::with_capacity(navpoints.get_length());
        for i in 0..navpoints.get_length() {
            let node = navpoints.item(i);
            //如果该node是null,或者不是Element,跳出本次循环
            if node.get_node_type() != Document::ELEMENT_NODE {
                continue;
            }

            let el = node;
            //如果该Element的name为"li",将其添加到目录结果
            if el.get_tag_name() == NCXDocumentV3::XHTML_TAGS_LI {
                result.push(Self::read_toc_reference(&el, book));
            }

        }

        result
    }

    fn read_toc_reference(navpoint_element: &Element, book: &EpubBook) -> TOCReference {
        //章节的名称
        let label = Self::read_nav_label(navpoint_element);
        //Log.d(TAG, "label:" + label);
        let toc_resource = book.get_spine().get_toc_resource();
        let mut toc_resource_root = match toc_resource.as_ref() {
            Some(resource) => StringUtil::substring_before_last(resource.get_href(), '/'),
            None => String::new(),
        };
        if toc_resource_root.len() == toc_resource.as_ref().map_or(0, |r| r.get_href().len()) {
            toc_resource_root = "".to_string();
        } else {
            toc_resource_root = toc_resource_root + "/";
        }

        let reference = StringUtil::collapse_path_dots(&(toc_resource_root + &Self::read_nav_reference(navpoint_element)));
        // fix: Constants 为 trait 无法直接引用关联常量, 用 Java 原值 '#'
        let href = StringUtil::substring_before(&reference, '#');
        let fragment_id = StringUtil::substring_after(&reference, '#');
        let resource = book.get_resources().get_by_href(&href);
        if resource.is_none() {
            eprintln!("{} {} Resource with href {} in NCX document not found", NCXDocumentV3::TAG, " ", href);
            // Log.e(TAG, "Resource with href " + href + " in NCX document not found");
        }

        println!("{} label:{}", NCXDocumentV3::TAG, label);
        println!("{} href:{}", NCXDocumentV3::TAG, href);
        println!("{} fragmentId:{}", NCXDocumentV3::TAG, fragment_id);

        //父级目录
        let mut result = TOCReference::with_fragment(Some(label), resource, Some(fragment_id));
        //解析子级目录
        let child_toc_references = Self::do_toc(navpoint_element, book);
        //readTOCReferences(
        //navpointElement.getChildNodes(), book);
        result.set_children(child_toc_references);
        result
    }

    /**
     * 获取目录节点的href
     *
     * @param navpointElement navpointElement
     * @return String
     */
    fn read_nav_reference(navpoint_element: &Element) -> String {
        //https://www.w3.org/publishing/epub/epub-packages.html#sec-package-nav
        //父级节点必须是 "li"
        //Log.d(TAG, "readNavReference:" + navpointElement.getTagName());

        let content_element = DOMUtil::get_first_element_by_tag_name_ns(navpoint_element, "", NCXDocumentV3::XHTML_TAGS_A);
        if content_element.is_none() {
            // fix: Java 此处返回 null, Rust 用空串占位
            return String::new();
        }
        let mut result = DOMUtil::get_attribute(content_element.as_ref().unwrap(), "", NCXDocumentV3::XHTML_ATTRIBUTES_HREF);
        // fix: Constants 为 trait 无法直接引用关联常量, 用 Java 原值 "UTF-8"
        match decode_url(result.clone(), "UTF-8") {
            Ok(decoded) => result = decoded,
            Err(e) => {
                // Log.e(TAG, e.getMessage());
                let _ = e;
            }
        }

        result
    }

    /**
     * 获取目录节点里面的章节名
     *
     * @param navpointElement navpointElement
     * @return String
     */
    fn read_nav_label(navpoint_element: &Element) -> String {
        //https://www.w3.org/publishing/epub/epub-packages.html#sec-package-nav
        //父级节点必须是 "li"
        //Log.d(TAG, "readNavLabel:" + navpointElement.getTagName());
        let mut label = String::new();
        let mut label_element = DOMUtil::get_first_element_by_tag_name_ns(navpoint_element, "", "a");
        assert!(label_element.is_some());
        label = label_element.as_ref().unwrap().get_text_content();
        if StringUtil::is_not_blank(&label) {
            return label;
        } else {
            label_element = DOMUtil::get_first_element_by_tag_name_ns(navpoint_element, "", "span");
        }
        assert!(label_element.is_some());
        label = label_element.as_ref().unwrap().get_text_content();
        //如果通过 a 标签无法获取章节列表,则是无href章节名
        label
    }

    pub fn create_ncx_resource(book: &EpubBook) -> Result<Resource, NcxV3Error> {
        Self::create_ncx_resource_full(book.get_metadata().get_identifiers(),
            book.get_title(), book.get_metadata().get_authors(),
            book.get_table_of_contents())
    }

    pub fn create_ncx_resource_full(identifiers: &Vec<Identifier>,
                             title: String, authors: &Vec<Author>, table_of_contents: &TableOfContents) -> Result<Resource, NcxV3Error> {
        let data = ByteArrayOutputStream::new();
        // fix: Java 将 data 传入 createXmlSerializer; EPS::OutputStream 为空 stub, 用单元值占位
        let mut out = EpubProcessorSupport::create_xml_serializer_stream(OutputStream);
        Self::write_full(&mut out, identifiers, title, authors, table_of_contents)?;

        let mut resource = Resource::with_id_data(Some(NCXDocumentV3::NCX_ITEM_ID.to_string()), Some(data.to_byte_array()),
            Some(NCXDocumentV3::DEFAULT_NCX_HREF.to_string()), Some(NCXDocumentV3::V3_NCX_MEDIATYPE.clone()));
        resource.set_properties(NCXDocumentV3::V3_NCX_PROPERTIES.to_string());
        Ok(resource)
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
    pub fn write_serializer(xml_serializer: &mut XmlSerializer, book: &EpubBook) -> Result<(), NcxV3Error> {
        Self::write_full(xml_serializer, book.get_metadata().get_identifiers(), book.get_title(),
            book.get_metadata().get_authors(), book.get_table_of_contents())
    }

    /**
     * 写入
     *
     * @param serializer      serializer
     * @param identifiers     identifiers
     * @param title           title
     * @param authors         authors
     * @param tableOfContents tableOfContents
     */
    #[allow(dead_code)]
    pub fn write_full(serializer: &mut XmlSerializer,
               identifiers: &Vec<Identifier>, title: String, authors: &Vec<Author>,
               table_of_contents: &TableOfContents) -> Result<(), NcxV3Error> {
        let _ = (identifiers, authors);
        // fix: Constants 为 trait 无法直接引用关联常量, 用 Java 原值 "UTF-8"
        serializer.start_document("UTF-8", false);
        serializer.set_prefix(EpubWriter::EMPTY_NAMESPACE_PREFIX, NCXDocumentV3::NAMESPACE_XHTML);
        serializer.start_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_HTML);
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, NCXDocumentV3::XHTML_ATTRIBUTES_XMLNS_EPUB, NCXDocumentV3::NAMESPACE_EPUB.to_string());
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, NCXDocumentV3::XHTML_ATTRIBUTES_XML_LANG, NCXDocumentV3::XHTML_ATTRIBUTE_VALUES_LANG.to_string());
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, NCXDocumentV3::XHTML_ATTRIBUTES_LANG, NCXDocumentV3::LANGUAGE.to_string());
        //写入头部head标签
        Self::write_head(&title, serializer)?;
        //body开始
        serializer.start_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_BODY);
        //h1开始
        serializer.start_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_H1);
        serializer.text(&title);
        serializer.end_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_H1);
        //h1关闭
        //nav开始
        serializer.start_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_NAV);
        serializer.attribute("", NCXDocumentV3::XHTML_ATTRIBUTES_EPUB_TYPE, NCXDocumentV3::XHTML_ATTRIBUTE_VALUES_EPUB_TYPE.to_string());
        serializer.attribute("", NCXDocumentV3::XHTML_ATTRIBUTES_ID, NCXDocumentV3::XHTML_ATTRIBUTE_VALUES_EPUB_TYPE.to_string());
        serializer.attribute("", NCXDocumentV3::XHTML_ATTRIBUTES_ROLE, NCXDocumentV3::XHTML_ATTRIBUTE_VALUES_ROLE_TOC.to_string());
        //h2开始
        serializer.start_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_H2);
        serializer.text("目录");
        serializer.end_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_H2);

        Self::write_nav_points(table_of_contents.get_toc_references(), 1, serializer)?;

        serializer.end_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_NAV);

        //body关闭
        serializer.end_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_BODY);

        serializer.end_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_HTML);
        serializer.end_document();
        Ok(())
    }

    fn write_nav_points(toc_references: &Vec<TOCReference>,
                        mut play_order: i32,
                        serializer: &mut XmlSerializer) -> Result<i32, NcxV3Error> {
        Self::write_ol_start(serializer)?;
        for toc_reference in toc_references {
            if toc_reference.get_resource().is_none() {
                play_order = Self::write_nav_points(toc_reference.get_children(), play_order,
                    serializer)?;
                continue;
            }

            Self::write_nav_point_start(toc_reference, serializer)?;

            play_order += 1;
            if !toc_reference.get_children().is_empty() {
                play_order = Self::write_nav_points(toc_reference.get_children(), play_order,
                    serializer)?;
            }

            Self::write_nav_point_end(toc_reference, serializer)?;
        }
        Self::write_ol_s_end(serializer)?;
        Ok(play_order)
    }

    fn write_nav_point_start(toc_reference: &TOCReference, serializer: &mut XmlSerializer) -> Result<(), NcxV3Error> {
        Self::write_li_start(serializer)?;
        let title = toc_reference.get_title();
        let href = toc_reference.get_resource().as_ref().map(|r| r.get_href().clone()).unwrap_or_default();
        // fix: TOCReference 无 get_complete_href, 按 Java 语义 inline: href + "#" + fragmentId
        let complete_href = match toc_reference.get_fragment_id().as_ref() {
            Some(fragment_id) => format!("{}{}{}", href, '#', fragment_id),
            None => href,
        };
        if StringUtil::is_not_blank(&complete_href) {
            Self::write_label_href(title.as_ref().map(|s| s.as_str()).unwrap_or_default(), &complete_href, serializer)?;
        } else {
            Self::write_label(title.as_ref().map(|s| s.as_str()).unwrap_or_default(), serializer)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn write_nav_point_end(toc_reference: &TOCReference,
                           serializer: &mut XmlSerializer) -> Result<(), NcxV3Error> {
        let _ = toc_reference;
        Self::write_li_end(serializer)?;
        Ok(())
    }

    fn write_label_href(title: &str, href: &str, serializer: &mut XmlSerializer) -> Result<(), NcxV3Error> {
        serializer.start_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_A);
        serializer.attribute("", NCXDocumentV3::XHTML_ATTRIBUTES_HREF, href.to_string());
        //attribute必须在Text之前设置。
        serializer.text(title);
        //serializer.attribute(NAMESPACE_XHTML, XHTMLAttributes.href, href);
        serializer.end_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_A);
        Ok(())
    }

    fn write_label(title: &str, serializer: &mut XmlSerializer) -> Result<(), NcxV3Error> {
        serializer.start_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_SPAN);
        serializer.text(title);
        serializer.end_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_SPAN);
        Ok(())
    }

    fn write_li_start(serializer: &mut XmlSerializer) -> Result<(), NcxV3Error> {
        serializer.start_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_LI);
        // Log.d(TAG, "writeLiStart");
        println!("{} writeLiStart", NCXDocumentV3::TAG);
        Ok(())
    }

    fn write_li_end(serializer: &mut XmlSerializer) -> Result<(), NcxV3Error> {
        serializer.end_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_LI);
        // Log.d(TAG, "writeLiEND");
        println!("{} writeLiEND", NCXDocumentV3::TAG);
        Ok(())
    }

    fn write_ol_start(serializer: &mut XmlSerializer) -> Result<(), NcxV3Error> {
        serializer.start_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_OL);
        // Log.d(TAG, "writeOlStart");
        println!("{} writeOlStart", NCXDocumentV3::TAG);
        Ok(())
    }

    fn write_ol_s_end(serializer: &mut XmlSerializer) -> Result<(), NcxV3Error> {
        serializer.end_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_OL);
        // Log.d(TAG, "writeOlEnd");
        println!("{} writeOlEnd", NCXDocumentV3::TAG);
        Ok(())
    }

    fn write_head(title: &str, serializer: &mut XmlSerializer) -> Result<(), NcxV3Error> {
        serializer.start_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_HEAD);
        //title
        serializer.start_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_TITLE);
        serializer.text(&StringUtil::default_if_null(title));
        serializer.end_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_TITLE);
        //link
        serializer.start_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_LINK);
        serializer.attribute("", NCXDocumentV3::XHTML_ATTRIBUTES_REL, "stylesheet".to_string());
        serializer.attribute("", NCXDocumentV3::XHTML_ATTRIBUTES_TYPE, "text/css".to_string());
        serializer.attribute("", NCXDocumentV3::XHTML_ATTRIBUTES_HREF, "css/style.css".to_string());
        serializer.end_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_LINK);

        //meta
        serializer.start_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_META);
        serializer.attribute("", NCXDocumentV3::XHTML_ATTRIBUTES_HTTP_EQUIV, NCXDocumentV3::XHTML_ATTRIBUTE_VALUES_CONTENT_TYPE.to_string());
        serializer.attribute("", NCXDocumentV3::XHTML_ATTRIBUTES_CONTENT, NCXDocumentV3::XHTML_ATTRIBUTE_VALUES_HTML_UTF8.to_string());
        serializer.end_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_META);

        serializer.end_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_HEAD);
        Ok(())
    }
}

pub struct NcxV3Error;

impl crate::me_ag2s_epublib_util_resourceutil::Document {
    // fix: RU::Document 为空 stub; Java 语义 getNodeName/getElementsByTagName 占位
    pub fn get_node_name(&self) -> String {
        String::new()
    }

    pub fn get_elements_by_tag_name(&self, _name: &str) -> NodeList {
        NodeList { items: Vec::new() }
    }
}

impl Element {
    // fix: DOMUtil::Element 无 getTagName/getTextContent, 按 Java 语义占位
    pub fn get_tag_name(&self) -> String {
        String::new()
    }

    pub fn get_text_content(&self) -> String {
        String::new()
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
