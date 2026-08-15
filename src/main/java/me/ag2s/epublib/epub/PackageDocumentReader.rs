use crate::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::me::ag2s::epublib::domain::{EpubBook, GuideReference, MediaTypes, Resource, Resources, Spine, SpineReference};
use crate::me::ag2s::epublib::epub::{DOMUtil, EpubReader, PackageDocumentBase, PackageDocumentMetadataReader};
use crate::me::ag2s::epublib::util::{ResourceUtil, StringUtil};
use crate::me_ag2s_epublib_domain_mediatype::MediaType;
use crate::me_ag2s_epublib_util_resourceutil::Document;

/**
 * Reads the opf package document as defined by namespace http://www.idpf.org/2007/opf
 *
 * @author paul
 */
pub struct PackageDocumentReader;

impl PackageDocumentReader {

    const TAG: &'static str = "me.ag2s.epublib.epub.PackageDocumentReader";
    const POSSIBLE_NCX_ITEM_IDS: [&'static str; 4] = ["toc", "ncx", "ncxtoc", "htmltoc"];

    // fix: PackageDocumentBase 同名常量私有，此处镜像常量值（与 Java 一致）
    const OPF_TAGS_METADATA: &'static str = "metadata";
    const OPF_TAGS_META: &'static str = "meta";
    const OPF_TAGS_MANIFEST: &'static str = "manifest";
    const OPF_TAGS_PACKAGE_TAG: &'static str = "package";
    const OPF_TAGS_ITEMREF: &'static str = "itemref";
    const OPF_TAGS_SPINE: &'static str = "spine";
    const OPF_TAGS_REFERENCE: &'static str = "reference";
    const OPF_TAGS_GUIDE: &'static str = "guide";
    const OPF_TAGS_ITEM: &'static str = "item";
    const OPF_ATTRIBUTES_UNIQUE_IDENTIFIER: &'static str = "unique-identifier";
    const OPF_ATTRIBUTES_IDREF: &'static str = "idref";
    const OPF_ATTRIBUTES_NAME: &'static str = "name";
    const OPF_ATTRIBUTES_CONTENT: &'static str = "content";
    const OPF_ATTRIBUTES_TYPE: &'static str = "type";
    const OPF_ATTRIBUTES_HREF: &'static str = "href";
    const OPF_ATTRIBUTES_LINEAR: &'static str = "linear";
    const OPF_ATTRIBUTES_ID: &'static str = "id";
    const OPF_ATTRIBUTES_MEDIA_TYPE: &'static str = "media-type";
    const OPF_ATTRIBUTES_TITLE: &'static str = "title";
    const OPF_ATTRIBUTES_TOC: &'static str = "toc";
    const OPF_ATTRIBUTES_PROPERTIES: &'static str = "properties";
    const OPF_VALUES_META_COVER: &'static str = "cover";
    const OPF_VALUES_REFERENCE_COVER: &'static str = "cover";
    const OPF_VALUES_NO: &'static str = "no";

    // fix: Constants 为 trait 私有关联常量，跨模块不可直接引用；此处镜像常量值（与 Java 一致）
    const CHARACTER_ENCODING: &'static str = "UTF-8";
    const FRAGMENT_SEPARATOR_CHAR: char = '#';
    const DEFAULT_TOC_ID: &'static str = "toc";

    pub fn read(package_resource: &Resource, epub_reader: &EpubReader, book: &mut EpubBook,
        resources: &mut Resources) -> Result<(), ReaderError> {
        let package_document = ResourceUtil::get_as_document(package_resource).unwrap_or_else(|_| Document::new(String::new()));
        let package_href = package_resource.get_href();
        let mut resources = Self::fix_hrefs(package_href, resources);
        Self::read_guide(&package_document, epub_reader, book, &resources);

        // Books sometimes use non-identifier ids. We map these here to legal ones
        let mut id_mapping = HashMap::new();
        let version = DOMUtil::get_attribute(&package_document.get_document_element(), PackageDocumentBase::PREFIX_OPF, PackageDocumentBase::VERSION);

        resources = Self::read_manifest(&package_document, &package_href, epub_reader,
            resources, &mut id_mapping);
        book.set_resources(resources);
        book.set_version(version);
        Self::read_cover(&package_document, book);
        book.set_metadata(
            PackageDocumentMetadataReader::read_metadata(&package_document.to_dom_document()));
        book.set_spine(Self::read_spine(&package_document, &book.get_resources(), &id_mapping));

        // if we did not find a cover page then we make the first page of the book the cover page
        if book.get_cover_page().is_none() && book.get_spine().size() > 0 {
            book.set_cover_page(book.get_spine().get_resource(0));
        }
        Ok(())
    }

    //	private static Resource readCoverImage(Element metadataElement, Resources resources) {
    //		String coverResourceId = DOMUtil.getFindAttributeValue(metadataElement.getOwnerDocument(), NAMESPACE_OPF, OPFTags.meta, OPFAttributes.name, OPFValues.meta_cover, OPFAttributes.content);
    //		if (StringUtil.isBlank(coverResourceId)) {
    //			return None;
    //		}
    //		Resource coverResource = resources.getByIdOrHref(coverResourceId);
    //		return coverResource;
    //	}

    /**
     * Reads the manifest containing the resource ids, hrefs and mediatypes.
     *
     * @param packageDocument e
     * @param packageHref     e
     * @param epubReader      e
     * @param resources       e
     * @param idMapping       e
     * @return a Map with resources, with their id's as key.
     */
    #[allow(dead_code)]
    fn read_manifest(package_document: &Document,
                     package_href: &str,
                     epub_reader: &EpubReader, mut resources: Resources,
                     id_mapping: &mut HashMap<String, String>) -> Resources {
        let manifest_element = DOMUtil::get_first_element_by_tag_name_ns(
            &package_document.get_document_element(), PackageDocumentBase::NAMESPACE_OPF, Self::OPF_TAGS_MANIFEST);
        let mut result = Resources::new();
        if manifest_element.is_none() {
            // Log.e(TAG,
            //         "Package document does not contain element " + OPFTags.manifest);
            eprintln!("{} {} Package does not contain element {}", PackageDocumentReader::TAG, " ", Self::OPF_TAGS_MANIFEST);
            return result;
        }
        let item_elements = manifest_element.unwrap().get_elements_by_tag_name_ns(PackageDocumentBase::NAMESPACE_OPF, Self::OPF_TAGS_ITEM);
        for i in 0..item_elements.get_length() {
            let item_element = item_elements.item(i);
            let id = DOMUtil::get_attribute(&item_element, PackageDocumentBase::NAMESPACE_OPF, Self::OPF_ATTRIBUTES_ID);
            let mut href = DOMUtil::get_attribute(&item_element, PackageDocumentBase::NAMESPACE_OPF, Self::OPF_ATTRIBUTES_HREF);

            match decode_url(href.clone(), Self::CHARACTER_ENCODING) {
                Ok(decoded) => href = decoded,
                Err(e) => {
                    // Log.e(TAG, e.getMessage());
                    e.print_stack_trace();
                }
            }
            let media_type_name = DOMUtil::get_attribute(&item_element, PackageDocumentBase::NAMESPACE_OPF, Self::OPF_ATTRIBUTES_MEDIA_TYPE);
            let resource = resources.remove(&href);
            if resource.is_none() {
                // Log.e(TAG, "resource with href '" + href + "' not found");
                eprintln!("{} {} resource with href '{}' not found", PackageDocumentReader::TAG, " ", href);
                continue;
            }
            let mut resource = resource.unwrap();
            resource.set_id(id.clone());
            //for epub3
            let properties = DOMUtil::get_attribute(&item_element, PackageDocumentBase::NAMESPACE_OPF, Self::OPF_ATTRIBUTES_PROPERTIES);
            resource.set_properties(properties);

            let media_type = MediaTypes::get_media_type_by_name(&media_type_name);
            if media_type.is_some() {
                resource.set_media_type(media_type);
            }
            id_mapping.insert(id, resource.get_id().clone());
            result.add(resource);
        }
        result
    }

    /**
     * Reads the book's guide.
     * Here some more attempts are made at finding the cover page.
     *
     * @param packageDocument r
     * @param epubReader      r
     * @param book            r
     * @param resources       g
     */
    #[allow(dead_code)]
    fn read_guide(package_document: &Document,
                  epub_reader: &EpubReader, book: &mut EpubBook, resources: &Resources) {
        let guide_element = DOMUtil::get_first_element_by_tag_name_ns(
            &package_document.get_document_element(), PackageDocumentBase::NAMESPACE_OPF, Self::OPF_TAGS_GUIDE);
        if guide_element.is_none() {
            return;
        }
        let mut guide = book.get_guide().clone();
        let guide_references = guide_element.unwrap().get_elements_by_tag_name_ns(PackageDocumentBase::NAMESPACE_OPF, Self::OPF_TAGS_REFERENCE);
        for i in 0..guide_references.get_length() {
            let reference_element = guide_references.item(i);
            let resource_href = DOMUtil::get_attribute(&reference_element, PackageDocumentBase::NAMESPACE_OPF, Self::OPF_ATTRIBUTES_HREF);
            if StringUtil::is_blank(&resource_href) {
                continue;
            }
            let resource = resources.get_by_href(&StringUtil::substring_before(&resource_href, Self::FRAGMENT_SEPARATOR_CHAR));
            if resource.is_none() {
                // Log.e(TAG, "Guide is referencing resource with href " + resourceHref
                //         + " which could not be found");
                eprintln!("{} {} Guide is referencing resource with href {} which could not be found", PackageDocumentReader::TAG, " ", resource_href);
                continue;
            }
            let type_name = DOMUtil::get_attribute(&reference_element, PackageDocumentBase::NAMESPACE_OPF, Self::OPF_ATTRIBUTES_TYPE);
            if StringUtil::is_blank(&type_name) {
                // Log.e(TAG, "Guide is referencing resource with href " + resourceHref
                //         + " which is missing the 'type' attribute");
                eprintln!("{} {} Guide is referencing resource with href {} which is missing the 'type' attribute", PackageDocumentReader::TAG, " ", resource_href);
                continue;
            }
            let title = DOMUtil::get_attribute(&reference_element, PackageDocumentBase::NAMESPACE_OPF, Self::OPF_ATTRIBUTES_TITLE);
            if GuideReference::COVER.eq_ignore_ascii_case(&type_name) {
                continue; // cover is handled elsewhere
            }
            let reference = GuideReference::with_fragment(resource, Some(type_name), Some(title),
                Some(StringUtil::substring_after(&resource_href, Self::FRAGMENT_SEPARATOR_CHAR)));
            guide.add_reference(reference);
        }
    }

    /**
     * Strips off the package prefixes up to the href of the packageHref.
     * <p>
     * Example:
     * If the packageHref is "OEBPS/content.opf" then a resource href like "OEBPS/foo/bar.html" will be turned into "foo/bar.html"
     *
     * @param packageHref     f
     * @param resourcesByHref g
     * @return The stripped package href
     */
    fn fix_hrefs(package_href: &str, resources_by_href: &mut Resources) -> Resources {
        let last_slash_pos = package_href.rfind('/');
        if last_slash_pos.is_none() {
            return std::mem::replace(resources_by_href, Resources::new());
        }
        let last_slash_pos = last_slash_pos.unwrap();
        let mut result = Resources::new();
        for mut resource in resources_by_href.get_all() {
            if StringUtil::is_not_blank(&resource.get_href())
                && resource.get_href().len() > last_slash_pos {
                resource.set_href(resource.get_href()[last_slash_pos + 1..].to_string());
            }
            result.add(resource);
        }
        result
    }

    /**
     * Reads the document's spine, containing all sections in reading order.
     *
     * @param packageDocument b
     * @param resources       b
     * @param idMapping       b
     * @return the document's spine, containing all sections in reading order.
     */
    fn read_spine(package_document: &Document, resources: &Resources,
                  id_mapping: &HashMap<String, String>) -> Spine {

        let spine_element = DOMUtil::get_first_element_by_tag_name_ns(
            &package_document.get_document_element(), PackageDocumentBase::NAMESPACE_OPF, Self::OPF_TAGS_SPINE);
        if spine_element.is_none() {
            // Log.e(TAG, "Element " + OPFTags.spine
            //         + " not found in package document, generating one automatically");
            eprintln!("{} {} Element {} not found in package document, generating one automatically", PackageDocumentReader::TAG, " ", Self::OPF_TAGS_SPINE);
            return Self::generate_spine_from_resources(resources);
        }
        let mut result = Spine::new();
        let toc_resource_id = DOMUtil::get_attribute(&spine_element.unwrap(), PackageDocumentBase::NAMESPACE_OPF, Self::OPF_ATTRIBUTES_TOC);
        // Log.v(TAG,tocResourceId);
        println!("{} {}", PackageDocumentReader::TAG, toc_resource_id);
        result.set_toc_resource(Self::find_table_of_contents_resource(&toc_resource_id, resources));
        let spine_nodes = DOMUtil::get_elements_by_tag_name_ns_doc(&package_document.to_dom_document(), PackageDocumentBase::NAMESPACE_OPF, Self::OPF_TAGS_ITEMREF);
        if spine_nodes.is_none() {
            // Log.e(TAG,"spineNodes is null");
            eprintln!("{} {} spineNodes is null", PackageDocumentReader::TAG, " ");
            return result;
        }
        let spine_nodes = spine_nodes.unwrap();
        let mut spine_references = Vec::with_capacity(spine_nodes.get_length());
        for i in 0..spine_nodes.get_length() {
            let spine_item = spine_nodes.item(i);
            let itemref = DOMUtil::get_attribute(&spine_item, PackageDocumentBase::NAMESPACE_OPF, Self::OPF_ATTRIBUTES_IDREF);
            if StringUtil::is_blank(&itemref) {
                // Log.e(TAG, "itemref with missing or empty idref"); // XXX
                eprintln!("{} {} itemref with missing or empty idref", PackageDocumentReader::TAG, " ");
                continue;
            }
            let id = id_mapping.get(&itemref).cloned().unwrap_or(itemref);

            let resource = resources.get_by_id_or_href(&id);
            if resource.is_none() {
                // Log.e(TAG, "resource with id '" + id + "' not found");
                eprintln!("{} {} resource with id '{}' not found", PackageDocumentReader::TAG, " ", id);
                continue;
            }

            let mut spine_reference = SpineReference::new(resource);
            if Self::OPF_VALUES_NO.eq_ignore_ascii_case(&DOMUtil::get_attribute(&spine_item, PackageDocumentBase::NAMESPACE_OPF, Self::OPF_ATTRIBUTES_LINEAR)) {
                spine_reference.set_linear(false);
            }
            spine_references.push(spine_reference);
        }
        result.set_spine_references(spine_references);
        result
    }

    /**
     * Creates a spine out of all resources in the resources.
     * The generated spine consists of all XHTML pages in order of their href.
     *
     * @param resources f
     * @return a spine created out of all resources in the resources.
     */
    fn generate_spine_from_resources(resources: &Resources) -> Spine {
        let mut result = Spine::new();
        let mut resource_hrefs: Vec<String> = resources.get_all_hrefs();
        resource_hrefs.sort_by_key(|s| s.to_lowercase());
        for resource_href in resource_hrefs {
            let resource = resources.get_by_href(&resource_href).unwrap();
            if resource.get_media_type().as_ref() == Some(&MediaTypes::NCX) {
                result.set_toc_resource(Some(resource));
            } else if resource.get_media_type().as_ref() == Some(&MediaTypes::XHTML) {
                result.add_spine_reference(SpineReference::new(Some(resource)));
            }
        }
        result
    }

    /**
     * The spine tag should contain a 'toc' attribute with as value the resource id of the table of contents resource.
     * <p>
     * Here we try several ways of finding this table of contents resource.
     * We try the given attribute value, some often-used ones and finally look through all resources for the first resource with the table of contents mimetype.
     *
     * @param tocResourceId g
     * @param resources     g
     * @return the Resource containing the table of contents
     */
    fn find_table_of_contents_resource(toc_resource_id: &str, resources: &Resources) -> Option<Resource> {
        let mut toc_resource;
        //一些epub3的文件为了兼容epub2,保留的epub2的目录文件，这里优先选择epub3的xml目录
        toc_resource = resources.get_by_properties(&String::from("nav"));
        if toc_resource.is_some() {
            return toc_resource;
        }

        if StringUtil::is_not_blank(toc_resource_id) {
            toc_resource = resources.get_by_id_or_href(&toc_resource_id.to_string());
        }

        if toc_resource.is_some() {
            return toc_resource;
        }

        // get the first resource with the NCX mediatype
        toc_resource = resources.find_first_resource_by_media_type(&MediaType::new(MediaTypes::NCX_NAME.to_string(), String::new()));

        if toc_resource.is_none() {
            for possible_ncx_item_id in PackageDocumentReader::POSSIBLE_NCX_ITEM_IDS {
                toc_resource = resources.get_by_id_or_href(&possible_ncx_item_id.to_string());
                if toc_resource.is_some() {
                    break;
                }
                toc_resource = resources.get_by_id_or_href(&possible_ncx_item_id.to_uppercase());
                if toc_resource.is_some() {
                    break;
                }
            }
        }

        if toc_resource.is_none() {
            eprintln!("{} {} Could not find table of contents resource. Tried resource with id '{}', {} , {} and any NCX resource.",
                PackageDocumentReader::TAG, " ", toc_resource_id, Self::DEFAULT_TOC_ID, Self::DEFAULT_TOC_ID.to_uppercase());
        }
        toc_resource
    }

    /**
     * Find all resources that have something to do with the coverpage and the cover image.
     * Search the meta tags and the guide references
     *
     * @param packageDocument s
     * @return all resources that have something to do with the coverpage and the cover image.
     */
    // package
    fn find_cover_hrefs(package_document: &Document) -> HashSet<String> {

        let mut result = HashSet::new();

        // try and find a meta tag with name = 'cover' and a non-blank id
        let cover_resource_id = DOMUtil::get_find_attribute_value(&package_document.to_dom_document(), PackageDocumentBase::NAMESPACE_OPF,
            Self::OPF_TAGS_META, Self::OPF_ATTRIBUTES_NAME, Self::OPF_VALUES_META_COVER,
            Self::OPF_ATTRIBUTES_CONTENT).unwrap_or_default();

        if StringUtil::is_not_blank(&cover_resource_id) {
            let cover_href = DOMUtil::get_find_attribute_value(&package_document.to_dom_document(), PackageDocumentBase::NAMESPACE_OPF,
                Self::OPF_TAGS_ITEM, Self::OPF_ATTRIBUTES_ID, &cover_resource_id,
                Self::OPF_ATTRIBUTES_HREF).unwrap_or_default();
            if StringUtil::is_not_blank(&cover_href) {
                result.insert(cover_href);
            } else {
                result.insert(cover_resource_id); // maybe there was a cover href put in the cover id attribute
            }
        }
        // try and find a reference tag with type is 'cover' and reference is not blank
        let cover_href = DOMUtil::get_find_attribute_value(&package_document.to_dom_document(), PackageDocumentBase::NAMESPACE_OPF,
            Self::OPF_TAGS_REFERENCE, Self::OPF_ATTRIBUTES_TYPE, Self::OPF_VALUES_REFERENCE_COVER,
            Self::OPF_ATTRIBUTES_HREF).unwrap_or_default();
        if StringUtil::is_not_blank(&cover_href) {
            result.insert(cover_href);
        }
        result
    }

    /**
     * Finds the cover resource in the packageDocument and adds it to the book if found.
     * Keeps the cover resource in the resources map
     *
     * @param packageDocument s
     * @param book            x
     */
    fn read_cover(package_document: &Document, book: &mut EpubBook) {

        let cover_hrefs = Self::find_cover_hrefs(package_document);
        for cover_href in cover_hrefs {
            let resource = book.get_resources().get_by_href(&cover_href);
            if resource.is_none() {
                eprintln!("{} {} Cover resource {} not found", PackageDocumentReader::TAG, " ", cover_href);
                continue;
            }
            let resource = resource.unwrap();
            if resource.get_media_type().as_ref() == Some(&MediaTypes::XHTML) {
                book.set_cover_page(Some(resource));
            } else if MediaTypes::is_bitmap_image(resource.get_media_type().as_ref().unwrap()) {
                book.set_cover_image(Some(resource));
            }
        }
    }
}

pub struct ReaderError;

fn decode_url(s: String, _encoding: &str) -> Result<String, ()> {
    // fix: 真实 percent 解码（原原样返回——manifest 中 %20/中文 href 与 zip 条目名不匹配，资源被丢弃）
    match percent_encoding::percent_decode_str(&s).decode_utf8() {
        Ok(decoded) => Ok(decoded.into_owned()),
        Err(_) => Ok(s),
    }
}
