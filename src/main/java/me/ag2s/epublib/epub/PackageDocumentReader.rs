use std::collections::{HashMap, HashSet};

use crate::me::ag2s::epublib::Constants;
use crate::me::ag2s::epublib::domain::{EpubBook, Guide, GuideReference, MediaTypes, Resource, Resources, Spine, SpineReference};
use crate::me::ag2s::epublib::epub::{DOMUtil, EpubReader, PackageDocumentBase, PackageDocumentMetadataReader};
use crate::me::ag2s::epublib::util::{ResourceUtil, StringUtil};

/**
 * Reads the opf package document as defined by namespace http://www.idpf.org/2007/opf
 *
 * @author paul
 */
pub struct PackageDocumentReader;

impl PackageDocumentReader {

    const TAG: &'static str = "me.ag2s.epublib.epub.PackageDocumentReader";
    const POSSIBLE_NCX_ITEM_IDS: [&'static str; 4] = ["toc", "ncx", "ncxtoc", "htmltoc"];

    pub fn read(package_resource: &Resource, epub_reader: &EpubReader, book: &mut EpubBook,
        resources: &mut Resources) -> Result<(), ReaderError> {
        let package_document = ResourceUtil::get_as_document(package_resource);
        let package_href = package_resource.get_href();
        let mut resources = fix_hrefs(package_href, resources);
        read_guide(&package_document, epub_reader, book, &resources);

        // Books sometimes use non-identifier ids. We map these here to legal ones
        let mut id_mapping = HashMap::new();
        let version = DOMUtil::get_attribute(package_document.get_document_element(), PackageDocumentBase::PREFIX_OPF, PackageDocumentBase::VERSION);

        resources = read_manifest(&package_document, &package_href, epub_reader,
            resources, &mut id_mapping);
        book.set_resources(resources);
        book.set_version(version);
        read_cover(&package_document, book);
        book.set_metadata(
            PackageDocumentMetadataReader::read_metadata(&package_document));
        book.set_spine(read_spine(&package_document, &book.get_resources(), &id_mapping));

        // if we did not find a cover page then we make the first page of the book the cover page
        if book.get_cover_page() == null && book.get_spine().size() > 0 {
            book.set_cover_page(book.get_spine().get_resource(0));
        }
        Ok(())
    }

    //	private static Resource readCoverImage(Element metadataElement, Resources resources) {
    //		String coverResourceId = DOMUtil.getFindAttributeValue(metadataElement.getOwnerDocument(), NAMESPACE_OPF, OPFTags.meta, OPFAttributes.name, OPFValues.meta_cover, OPFAttributes.content);
    //		if (StringUtil.isBlank(coverResourceId)) {
    //			return null;
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
            package_document.get_document_element(), PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_MANIFEST);
        let mut result = Resources::new();
        if manifest_element == null {
            // Log.e(TAG,
            //         "Package document does not contain element " + OPFTags.manifest);
            eprintln!("{} {} Package does not contain element {}", PackageDocumentReader::TAG, " ", PackageDocumentBase::OPF_TAGS_MANIFEST);
            return result;
        }
        let item_elements = manifest_element.get_elements_by_tag_name_ns(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_ITEM);
        for i in 0..item_elements.get_length() {
            let item_element = item_elements.item(i);
            let id = DOMUtil::get_attribute(item_element, PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_ATTRIBUTES_ID);
            let mut href = DOMUtil::get_attribute(item_element, PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_ATTRIBUTES_HREF);

            match decode_url(href.clone(), Constants::CHARACTER_ENCODING) {
                Ok(decoded) => href = decoded,
                Err(e) => {
                    // Log.e(TAG, e.getMessage());
                    e.printStackTrace();
                }
            }
            let media_type_name = DOMUtil::get_attribute(item_element, PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_ATTRIBUTES_MEDIA_TYPE);
            let mut resource = resources.remove(&href);
            if resource == null {
                // Log.e(TAG, "resource with href '" + href + "' not found");
                eprintln!("{} {} resource with href '{}' not found", PackageDocumentReader::TAG, " ", href);
                continue;
            }
            resource.set_id(id);
            //for epub3
            let properties = DOMUtil::get_attribute(item_element, PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_ATTRIBUTES_PROPERTIES);
            resource.set_properties(properties);

            let media_type = MediaTypes::get_media_type_by_name(&media_type_name);
            if media_type != null {
                resource.set_media_type(media_type);
            }
            result.add(resource);
            id_mapping.insert(id, resource.get_id());
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
            package_document.get_document_element(), PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_GUIDE);
        if guide_element == null {
            return;
        }
        let mut guide = book.get_guide();
        let guide_references = guide_element.get_elements_by_tag_name_ns(PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_REFERENCE);
        for i in 0..guide_references.get_length() {
            let reference_element = guide_references.item(i);
            let resource_href = DOMUtil::get_attribute(reference_element, PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_ATTRIBUTES_HREF);
            if StringUtil::is_blank(&resource_href) {
                continue;
            }
            let resource = resources.get_by_href(&StringUtil::substring_before(resource_href.clone(), Constants::FRAGMENT_SEPARATOR_CHAR));
            if resource == null {
                // Log.e(TAG, "Guide is referencing resource with href " + resourceHref
                //         + " which could not be found");
                eprintln!("{} {} Guide is referencing resource with href {} which could not be found", PackageDocumentReader::TAG, " ", resource_href);
                continue;
            }
            let type_name = DOMUtil::get_attribute(reference_element, PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_ATTRIBUTES_TYPE);
            if StringUtil::is_blank(&type_name) {
                // Log.e(TAG, "Guide is referencing resource with href " + resourceHref
                //         + " which is missing the 'type' attribute");
                eprintln!("{} {} Guide is referencing resource with href {} which is missing the 'type' attribute", PackageDocumentReader::TAG, " ", resource_href);
                continue;
            }
            let title = DOMUtil::get_attribute(reference_element, PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_ATTRIBUTES_TITLE);
            if GuideReference::COVER.eq_ignore_ascii_case(&type_name) {
                continue; // cover is handled elsewhere
            }
            let reference = GuideReference::new(resource, type_name, title,
                StringUtil::substring_after(resource_href.clone(), Constants::FRAGMENT_SEPARATOR_CHAR));
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
        if last_slash_pos < 0 {
            return resources_by_href;
        }
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
            package_document.get_document_element(), PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_SPINE);
        if spine_element == null {
            // Log.e(TAG, "Element " + OPFTags.spine
            //         + " not found in package document, generating one automatically");
            eprintln!("{} {} Element {} not found in package document, generating one automatically", PackageDocumentReader::TAG, " ", PackageDocumentBase::OPF_TAGS_SPINE);
            return generate_spine_from_resources(resources);
        }
        let mut result = Spine::new();
        let toc_resource_id = DOMUtil::get_attribute(spine_element, PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_ATTRIBUTES_TOC);
        // Log.v(TAG,tocResourceId);
        println!("{} {}", PackageDocumentReader::TAG, toc_resource_id);
        result.set_toc_resource(find_table_of_contents_resource(&toc_resource_id, resources));
        let spine_nodes = DOMUtil::get_elements_by_tag_name_ns_doc(package_document, PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_TAGS_ITEMREF);
        if spine_nodes == null {
            // Log.e(TAG,"spineNodes is null");
            eprintln!("{} {} spineNodes is null", PackageDocumentReader::TAG, " ");
            return result;
        }
        let mut spine_references = Vec::with_capacity(spine_nodes.get_length());
        for i in 0..spine_nodes.get_length() {
            let spine_item = spine_nodes.item(i);
            let itemref = DOMUtil::get_attribute(spine_item, PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_ATTRIBUTES_IDREF);
            if StringUtil::is_blank(&itemref) {
                // Log.e(TAG, "itemref with missing or empty idref"); // XXX
                eprintln!("{} {} itemref with missing or empty idref", PackageDocumentReader::TAG, " ");
                continue;
            }
            let mut id = id_mapping.get(&itemref).cloned();
            if id == null {
                id = itemref;
            }

            let resource = resources.get_by_id_or_href(&id);
            if resource == null {
                // Log.e(TAG, "resource with id '" + id + "' not found");
                eprintln!("{} {} resource with id '{}' not found", PackageDocumentReader::TAG, " ", id);
                continue;
            }

            let mut spine_reference = SpineReference::new(resource);
            if PackageDocumentBase::OPF_VALUES_NO.eq_ignore_ascii_case(&DOMUtil::get_attribute(spine_item, PackageDocumentBase::NAMESPACE_OPF, PackageDocumentBase::OPF_ATTRIBUTES_LINEAR)) {
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
            let resource = resources.get_by_href(&resource_href);
            if resource.get_media_type() == MediaTypes::NCX {
                result.set_toc_resource(resource);
            } else if resource.get_media_type() == MediaTypes::XHTML {
                result.add_spine_reference(SpineReference::new(resource));
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
    fn find_table_of_contents_resource(toc_resource_id: &str, resources: &Resources) -> Resource {
        let mut toc_resource;
        //一些epub3的文件为了兼容epub2,保留的epub2的目录文件，这里优先选择epub3的xml目录
        toc_resource = resources.get_by_properties("nav");
        if toc_resource != null {
            return toc_resource;
        }

        if StringUtil::is_not_blank(toc_resource_id) {
            toc_resource = resources.get_by_id_or_href(toc_resource_id);
        }

        if toc_resource != null {
            return toc_resource;
        }

        // get the first resource with the NCX mediatype
        toc_resource = resources.find_first_resource_by_media_type(MediaTypes::NCX);

        if toc_resource == null {
            for possible_ncx_item_id in PackageDocumentReader::POSSIBLE_NCX_ITEM_IDS {
                toc_resource = resources.get_by_id_or_href(possible_ncx_item_id);
                if toc_resource != null {
                    break;
                }
                toc_resource = resources.get_by_id_or_href(&possible_ncx_item_id.to_uppercase());
                if toc_resource != null {
                    break;
                }
            }
        }

        if toc_resource == null {
            eprintln!("{} {} Could not find table of contents resource. Tried resource with id '{}', {} , {} and any NCX resource.",
                PackageDocumentReader::TAG, " ", toc_resource_id, Constants::DEFAULT_TOC_ID, Constants::DEFAULT_TOC_ID.to_uppercase());
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
        let cover_resource_id = DOMUtil::get_find_attribute_value(package_document, PackageDocumentBase::NAMESPACE_OPF,
            PackageDocumentBase::OPF_TAGS_META, PackageDocumentBase::OPF_ATTRIBUTES_NAME, PackageDocumentBase::OPF_VALUES_META_COVER,
            PackageDocumentBase::OPF_ATTRIBUTES_CONTENT);

        if StringUtil::is_not_blank(&cover_resource_id) {
            let cover_href = DOMUtil::get_find_attribute_value(package_document, PackageDocumentBase::NAMESPACE_OPF,
                PackageDocumentBase::OPF_TAGS_ITEM, PackageDocumentBase::OPF_ATTRIBUTES_ID, &cover_resource_id,
                PackageDocumentBase::OPF_ATTRIBUTES_HREF);
            if StringUtil::is_not_blank(&cover_href) {
                result.insert(cover_href);
            } else {
                result.insert(cover_resource_id); // maybe there was a cover href put in the cover id attribute
            }
        }
        // try and find a reference tag with type is 'cover' and reference is not blank
        let cover_href = DOMUtil::get_find_attribute_value(package_document, PackageDocumentBase::NAMESPACE_OPF,
            PackageDocumentBase::OPF_TAGS_REFERENCE, PackageDocumentBase::OPF_ATTRIBUTES_TYPE, PackageDocumentBase::OPF_VALUES_REFERENCE_COVER,
            PackageDocumentBase::OPF_ATTRIBUTES_HREF);
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

        let cover_hrefs = find_cover_hrefs(package_document);
        for cover_href in cover_hrefs {
            let resource = book.get_resources().get_by_href(&cover_href);
            if resource == null {
                eprintln!("{} {} Cover resource {} not found", PackageDocumentReader::TAG, " ", cover_href);
                continue;
            }
            if resource.get_media_type() == MediaTypes::XHTML {
                book.set_cover_page(resource);
            } else if MediaTypes::is_bitmap_image(resource.get_media_type()) {
                book.set_cover_image(resource);
            }
        }
    }
}

pub struct Document;
pub struct Element;
pub struct NodeList;
pub struct ReaderError;

impl Document {
    pub fn get_document_element(&self) -> Element { todo!() }
}

impl Element {
    pub fn get_elements_by_tag_name_ns(&self, _namespace: &str, _name: &str) -> NodeList { todo!() }
}

impl NodeList {
    pub fn get_length(&self) -> usize { todo!() }
    pub fn item(&self, _i: usize) -> Element { todo!() }
}

fn decode_url(_s: String, _encoding: &str) -> Result<String, ()> {
    todo!()
}
