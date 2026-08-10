use crate::me::ag2s::epublib::Constants;
use crate::me::ag2s::epublib::domain::{Author, EpubBook, Identifier, MediaType, MediaTypes, Resource, TableOfContents, TOCReference};
use crate::me::ag2s::epublib::epub::{DOMUtil, EpubProcessorSupport, EpubReader, EpubWriter, NCXDocumentV2};
use crate::me::ag2s::epublib::util::{ResourceUtil, StringUtil};

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
    pub const V3_NCX_MEDIATYPE: MediaType = MediaTypes::XHTML;

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
        let ncx_resource;
        if book.get_spine().get_toc_resource() == null {
            // Log.e(TAG, "Book does not contain a table of contents file");
            eprintln!("{} Book does not contain a table of contents file", NCXDocumentV3::TAG);
            return None;
        }
        match {
            ncx_resource = book.get_spine().get_toc_resource();
            if ncx_resource == null {
                return None;
            }
            //一些epub 3 文件没有按照epub3的标准使用删除掉ncx目录文件
            if ncx_resource.get_href().ends_with(".ncx") {
                // Log.v(TAG,"该epub文件不标准，使用了epub2的目录文件");
                eprintln!("{} 该epub文件不标准，使用了epub2的目录文件", NCXDocumentV3::TAG);
                return NCXDocumentV2::read(book, epub_reader);
            }
            // Log.d(TAG, ncxResource.getHref());
            println!("{} {}", NCXDocumentV3::TAG, ncx_resource.get_href());

            let ncx_document = ResourceUtil::get_as_document(&ncx_resource);
            // Log.d(TAG, ncxDocument.getNodeName());
            println!("{} {}", NCXDocumentV3::TAG, ncx_document.get_node_name());

            let nav_map_element = ncx_document.get_elements_by_tag_name(NCXDocumentV3::XHTML_TAGS_NAV).item(0);
            if nav_map_element == null {
                // Log.d(TAG,"epub3目录文件未发现nav节点，尝试使用epub2的规则解析");
                println!("{} {}", NCXDocumentV3::TAG, "epub3目录文件未发现nav节点，尝试使用epub2的规则解析");
                return NCXDocumentV2::read(book, epub_reader);
            }
            let nav_map_element = nav_map_element.get_elements_by_tag_name(NCXDocumentV3::XHTML_TAGS_OL).item(0);
            // Log.d(TAG, navMapElement.getTagName());
            println!("{} {}", NCXDocumentV3::TAG, nav_map_element.get_tag_name());

            let table_of_contents = TableOfContents::new(
                read_toc_references(nav_map_element.get_child_nodes(), book));
            // Log.d(TAG, tableOfContents.toString());
            println!("{} {}", NCXDocumentV3::TAG, table_of_contents.to_string());
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

    fn do_toc(n: &Node, book: &EpubBook) -> Vec<TOCReference> {
        let mut result = Vec::new();

        if n == null || n.get_node_type() != Document::ELEMENT_NODE {
            return result;
        } else {
            let el = n;
            let node_list = el.get_elements_by_tag_name(NCXDocumentV3::XHTML_TAGS_LI);
            for i in 0..node_list.get_length() {
                result.push(read_toc_reference(node_list.item(i), book));
            }
        }
        result
    }

    fn read_toc_references(navpoints: &NodeList, book: &EpubBook) -> Vec<TOCReference> {
        if navpoints == null {
            return Vec::new();
        }
        //Log.d(TAG, "readTOCReferences:navpoints.getLength()" + navpoints.getLength());
        let mut result = Vec::with_capacity(navpoints.get_length());
        for i in 0..navpoints.get_length() {
            let node = navpoints.item(i);
            //如果该node是null,或者不是Element,跳出本次循环
            if node == null || node.get_node_type() != Document::ELEMENT_NODE {
                continue;
            }

            let el = node;
            //如果该Element的name为”li“,将其添加到目录结果
            if el.get_tag_name() == NCXDocumentV3::XHTML_TAGS_LI {
                result.push(read_toc_reference(el, book));
            }

        }

        result
    }

    fn read_toc_reference(navpoint_element: &Element, book: &EpubBook) -> TOCReference {
        //章节的名称
        let label = read_nav_label(navpoint_element);
        //Log.d(TAG, "label:" + label);
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
            eprintln!("{} {} Resource with href {} in NCX document not found", NCXDocumentV3::TAG, " ", href);
            // Log.e(TAG, "Resource with href " + href + " in NCX document not found");
        }

        println!("{} label:{}", NCXDocumentV3::TAG, label);
        println!("{} href:{}", NCXDocumentV3::TAG, href);
        println!("{} fragmentId:{}", NCXDocumentV3::TAG, fragment_id);

        //父级目录
        let mut result = TOCReference::new(label, resource, fragment_id);
        //解析子级目录
        let child_toc_references = do_toc(navpoint_element, book);
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
        if content_element == null {
            return null;
        }
        let mut result = DOMUtil::get_attribute(content_element, "", NCXDocumentV3::XHTML_ATTRIBUTES_HREF);
        match decode_url(result.clone(), Constants::CHARACTER_ENCODING) {
            Ok(decoded) => result = decoded,
            Err(e) => {
                // Log.e(TAG, e.getMessage());
                e.printStackTrace();
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
        let mut label;
        let mut label_element = DOMUtil::get_first_element_by_tag_name_ns(navpoint_element, "", "a");
        assert!(label_element != null);
        label = label_element.get_text_content();
        if StringUtil::is_not_blank(&label) {
            return label;
        } else {
            label_element = DOMUtil::get_first_element_by_tag_name_ns(navpoint_element, "", "span");
        }
        assert!(label_element != null);
        label = label_element.get_text_content();
        //如果通过 a 标签无法获取章节列表,则是无href章节名
        label
    }

    pub fn create_ncx_resource(book: &EpubBook) -> Result<Resource, NcxV3Error> {
        create_ncx_resource_full(book.get_metadata().get_identifiers().clone(),
            book.get_title(), book.get_metadata().get_authors().clone(),
            book.get_table_of_contents())
    }

    pub fn create_ncx_resource_full(identifiers: Vec<Identifier>,
                             title: String, authors: Vec<Author>, table_of_contents: TableOfContents) -> Result<Resource, NcxV3Error> {
        let mut data = ByteArrayOutputStream::new();
        let mut out = EpubProcessorSupport::create_xml_serializer_stream(data.clone());
        write_full(&mut out, identifiers, title, authors, table_of_contents)?;

        let mut resource = Resource::new(NCXDocumentV3::NCX_ITEM_ID.to_string(), data.to_byte_array(),
            NCXDocumentV3::DEFAULT_NCX_HREF.to_string(), NCXDocumentV3::V3_NCX_MEDIATYPE);
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
        write_full(xml_serializer, book.get_metadata().get_identifiers().clone(), book.get_title(),
            book.get_metadata().get_authors().clone(), book.get_table_of_contents())
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
               identifiers: Vec<Identifier>, title: String, authors: Vec<Author>,
               table_of_contents: TableOfContents) -> Result<(), NcxV3Error> {
        serializer.start_document(Constants::CHARACTER_ENCODING, false);
        serializer.set_prefix(EpubWriter::EMPTY_NAMESPACE_PREFIX, NCXDocumentV3::NAMESPACE_XHTML);
        serializer.start_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_HTML);
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, NCXDocumentV3::XHTML_ATTRIBUTES_XMLNS_EPUB, NCXDocumentV3::NAMESPACE_EPUB);
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, NCXDocumentV3::XHTML_ATTRIBUTES_XML_LANG, NCXDocumentV3::XHTML_ATTRIBUTE_VALUES_LANG);
        serializer.attribute(EpubWriter::EMPTY_NAMESPACE_PREFIX, NCXDocumentV3::XHTML_ATTRIBUTES_LANG, NCXDocumentV3::LANGUAGE);
        //写入头部head标签
        write_head(&title, serializer)?;
        //body开始
        serializer.start_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_BODY);
        //h1开始
        serializer.start_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_H1);
        serializer.text(&title);
        serializer.end_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_H1);
        //h1关闭
        //nav开始
        serializer.start_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_NAV);
        serializer.attribute("", NCXDocumentV3::XHTML_ATTRIBUTES_EPUB_TYPE, NCXDocumentV3::XHTML_ATTRIBUTE_VALUES_EPUB_TYPE);
        serializer.attribute("", NCXDocumentV3::XHTML_ATTRIBUTES_ID, NCXDocumentV3::XHTML_ATTRIBUTE_VALUES_EPUB_TYPE);
        serializer.attribute("", NCXDocumentV3::XHTML_ATTRIBUTES_ROLE, NCXDocumentV3::XHTML_ATTRIBUTE_VALUES_ROLE_TOC);
        //h2开始
        serializer.start_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_H2);
        serializer.text("目录");
        serializer.end_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_H2);

        write_nav_points(table_of_contents.get_toc_references(), 1, serializer)?;

        serializer.end_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_NAV);

        //body关闭
        serializer.end_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_BODY);

        serializer.end_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_HTML);
        serializer.end_document();
        Ok(())
    }

    fn write_nav_points(toc_references: Vec<TOCReference>,
                        mut play_order: i32,
                        serializer: &mut XmlSerializer) -> Result<i32, NcxV3Error> {
        write_ol_start(serializer)?;
        for toc_reference in toc_references {
            if toc_reference.get_resource() == null {
                play_order = write_nav_points(toc_reference.get_children(), play_order,
                    serializer)?;
                continue;
            }

            write_nav_point_start(&toc_reference, serializer)?;

            play_order += 1;
            if !toc_reference.get_children().is_empty() {
                play_order = write_nav_points(toc_reference.get_children(), play_order,
                    serializer)?;
            }

            write_nav_point_end(&toc_reference, serializer)?;
        }
        write_ol_s_end(serializer)?;
        Ok(play_order)
    }

    fn write_nav_point_start(toc_reference: &TOCReference, serializer: &mut XmlSerializer) -> Result<(), NcxV3Error> {
        write_li_start(serializer)?;
        let title = toc_reference.get_title();
        let href = toc_reference.get_complete_href();
        if StringUtil::is_not_blank(&href) {
            write_label_href(&title, &href, serializer)?;
        } else {
            write_label(&title, serializer)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn write_nav_point_end(toc_reference: &TOCReference,
                           serializer: &mut XmlSerializer) -> Result<(), NcxV3Error> {
        write_li_end(serializer)?;
        Ok(())
    }

    fn write_label_href(title: &str, href: &str, serializer: &mut XmlSerializer) -> Result<(), NcxV3Error> {
        serializer.start_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_A);
        serializer.attribute("", NCXDocumentV3::XHTML_ATTRIBUTES_HREF, href);
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
        serializer.attribute("", NCXDocumentV3::XHTML_ATTRIBUTES_REL, "stylesheet");
        serializer.attribute("", NCXDocumentV3::XHTML_ATTRIBUTES_TYPE, "text/css");
        serializer.attribute("", NCXDocumentV3::XHTML_ATTRIBUTES_HREF, "css/style.css");
        serializer.end_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_LINK);

        //meta
        serializer.start_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_META);
        serializer.attribute("", NCXDocumentV3::XHTML_ATTRIBUTES_HTTP_EQUIV, NCXDocumentV3::XHTML_ATTRIBUTE_VALUES_CONTENT_TYPE);
        serializer.attribute("", NCXDocumentV3::XHTML_ATTRIBUTES_CONTENT, NCXDocumentV3::XHTML_ATTRIBUTE_VALUES_HTML_UTF8);
        serializer.end_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_META);

        serializer.end_tag(NCXDocumentV3::NAMESPACE_XHTML, NCXDocumentV3::XHTML_TAGS_HEAD);
        Ok(())
    }
}

pub struct Document;
pub struct Element;
pub struct Node;
pub struct NodeList;
pub struct XmlSerializer;
pub struct ByteArrayOutputStream;
pub struct NcxV3Error;

impl Document {
    pub const ELEMENT_NODE: u16 = 1;
    pub fn get_node_name(&self) -> String { todo!() }
    pub fn get_elements_by_tag_name(&self, _name: &str) -> NodeList { todo!() }
}

impl Node {
    pub fn get_node_type(&self) -> u16 { todo!() }
}

impl Element {
    pub fn get_elements_by_tag_name(&self, _name: &str) -> NodeList { todo!() }
    pub fn get_tag_name(&self) -> String { todo!() }
    pub fn get_child_nodes(&self) -> NodeList { todo!() }
    pub fn get_text_content(&self) -> String { todo!() }
}

impl NodeList {
    pub fn get_length(&self) -> usize { todo!() }
    pub fn item(&self, _i: usize) -> Element { todo!() }
}

impl XmlSerializer {
    pub fn start_document(&mut self, _encoding: &str, _standalone: bool) { todo!() }
    pub fn set_prefix(&mut self, _prefix: &str, _namespace: &str) { todo!() }
    pub fn start_tag(&mut self, _namespace: &str, _name: &str) { todo!() }
    pub fn attribute(&mut self, _namespace: &str, _name: &str, _value: &str) { todo!() }
    pub fn end_tag(&mut self, _namespace: &str, _name: &str) { todo!() }
    pub fn text(&mut self, _text: &str) { todo!() }
    pub fn end_document(&mut self) { todo!() }
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
