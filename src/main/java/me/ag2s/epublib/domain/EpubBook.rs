use crate::prelude::*;
// package me.ag2s.epublib.domain;

// import java.io.Serializable;
// import java.util.ArrayList;
// import java.util.LinkedHashMap;
// import java.util.List;
// import java.util.Map;

/**
 * Representation of a Book.
 * <p>
 * All resources of a Book (html, css, xml, fonts, images) are represented
 * as Resources. See getResources() for access to these.<br/>
 * A Book as 3 indexes into these Resources, as per the epub specification.<br/>
 * <dl>
 * <dt>Spine</dt>
 * <dd>these are the Resources to be shown when a user reads the book from
 * start to finish.</dd>
 * <dt>Table of Contents<dt>
 * <dd>The table of contents. Table of Contents references may be in a
 * different order and contain different Resources than the spine, and often do.
 * <dt>Guide</dt>
 * <dd>The Guide has references to a set of special Resources like the
 * cover page, the Glossary, the copyright page, etc.
 * </dl>
 * <p/>
 * The complication is that these 3 indexes may and usually do point to
 * different pages.
 * A chapter may be split up in 2 pieces to fit it in to memory. Then the
 * spine will contain both pieces, but the Table of Contents only the first.
 * <p>
 * The Content page may be in the Table of Contents, the Guide, but not
 * in the Spine.
 * Etc.
 * <p/>
 * <p>
 * Please see the illustration at: doc/schema.svg
 *
 * @author paul
 * @author jake
 */
pub struct EpubBook {

    resources: Resources,
    metadata: Metadata,
    spine: Spine,
    table_of_contents: TableOfContents,
    guide: Guide,
    opf_resource: Option<Resource>,
    ncx_resource: Option<Resource>,
    cover_image: Option<Resource>,

    version: String,
}

impl EpubBook {

    pub fn new() -> EpubBook {
        EpubBook {
            resources: Resources::new(),
            metadata: Metadata::new(),
            spine: Spine::new(),
            table_of_contents: TableOfContents::new(),
            guide: Guide::new(),
            opf_resource: None,
            ncx_resource: None,
            cover_image: None,
            version: "2.0".to_string(),
        }
    }

    pub fn get_version(&self) -> &String {
        &self.version
    }

    pub fn set_version(&mut self, version: String) {
        self.version = version;
    }

    pub fn is_epub3(&self) -> bool {
        self.version.starts_with("3.")
    }

    // @SuppressWarnings("UnusedReturnValue")
    pub fn add_section(
            &mut self, parent_section: &mut TOCReference, section_title: String, resource: Resource) -> TOCReference {
        self.add_section_with_fragment(parent_section, section_title, resource, None)
    }

    // fix: Java 重载 addSection(String, Resource) 转录改名，避免与父节点版重名
    pub fn add_section_at_root(&mut self, title: String, resource: Resource) -> TOCReference {
        self.add_section_with_fragment_at_root(title, resource, None)
    }

    /**
     * Adds the resource to the table of contents of the book as a child
     * section of the given parentSection
     *
     * @param parentSection parentSection
     * @param sectionTitle  sectionTitle
     * @param resource      resource
     * @param fragmentId    fragmentId
     * @return The table of contents
     */
    pub fn add_section_with_fragment(
            &mut self, parent_section: &mut TOCReference, section_title: String, resource: Resource,
            fragment_id: Option<String>) -> TOCReference {
        self.resources.add(resource.clone());
        if self.spine.find_first_resource_by_id(&resource.get_id()) < 0 {
            self.spine.add_spine_reference(SpineReference::new(Some(resource.clone())));
        }
        return parent_section.add_child_section(
                TOCReference::with_fragment(Some(section_title), Some(resource), fragment_id));
    }

    // fix: Java 重载 addSectionWithFragment(String, Resource, String) 转录改名，避免与父节点版重名
    pub fn add_section_with_fragment_at_root(
            &mut self, title: String, resource: Resource, fragment_id: Option<String>) -> TOCReference {
        self.resources.add(resource.clone());
        let toc_reference = self.table_of_contents
                .add_toc_reference(TOCReference::with_fragment(Some(title), Some(resource.clone()), fragment_id));
        if self.spine.find_first_resource_by_id(&resource.get_id()) < 0 {
            self.spine.add_spine_reference(SpineReference::new(Some(resource)));
        }
        return toc_reference;
    }

    // @SuppressWarnings("unused")
    pub fn generate_spine_from_table_of_contents(&mut self) {
        let mut spine = Spine::from_toc(&self.table_of_contents);

        // in case the tocResource was already found and assigned
        spine.set_toc_resource(self.spine.get_toc_resource().clone());

        self.spine = spine;
    }

    /**
     * The Book's metadata (titles, authors, etc)
     *
     * @return The Book's metadata (titles, authors, etc)
     */
    pub fn get_metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn set_metadata(&mut self, metadata: Metadata) {
        self.metadata = metadata;
    }


    pub fn set_resources(&mut self, resources: Resources) {
        self.resources = resources;
    }

    // @SuppressWarnings("unused")
    pub fn add_resource(&mut self, resource: Resource) -> Resource {
        return self.resources.add(resource);
    }

    /**
     * The collection of all images, chapters, sections, xhtml files,
     * stylesheets, etc that make up the book.
     *
     * @return The collection of all images, chapters, sections, xhtml files,
     * stylesheets, etc that make up the book.
     */
    pub fn get_resources(&self) -> &Resources {
        &self.resources
    }


    /**
     * The sections of the book that should be shown if a user reads the book
     * from start to finish.
     *
     * @return The Spine
     */
    pub fn get_spine(&self) -> &Spine {
        &self.spine
    }


    pub fn set_spine(&mut self, spine: Spine) {
        self.spine = spine;
    }


    /**
     * The Table of Contents of the book.
     *
     * @return The Table of Contents of the book.
     */
    pub fn get_table_of_contents(&self) -> &TableOfContents {
        &self.table_of_contents
    }


    pub fn set_table_of_contents(&mut self, table_of_contents: TableOfContents) {
        self.table_of_contents = table_of_contents;
    }

    /**
     * The book's cover page as a Resource.
     * An XHTML document containing a link to the cover image.
     *
     * @return The book's cover page as a Resource
     */
    pub fn get_cover_page(&self) -> Option<Resource> {
        // fix: Guide::get_cover_page() 需要 &mut self，改为只读扫描引用列表（等价逻辑）
        let mut cover_page = self.guide.get_references().iter()
                .find(|guide_reference| guide_reference.get_type().eq(GuideReference::COVER))
                .and_then(|guide_reference| guide_reference.get_resource().clone());
        if cover_page.is_none() {
            cover_page = self.spine.get_resource(0);
        }
        return cover_page;
    }


    pub fn set_cover_page(&mut self, cover_page: Option<Resource>) {
        if cover_page.is_none() {
            return;
        }
        if self.resources.not_contains_by_href(&cover_page.as_ref().unwrap().get_href()) {
            self.resources.add(cover_page.as_ref().unwrap().clone());
        }
        self.guide.set_cover_page(cover_page);
    }

    /**
     * Gets the first non-blank title from the book's metadata.
     *
     * @return the first non-blank title from the book's metadata.
     */
    pub fn get_title(&self) -> String {
        self.get_metadata().get_first_title()
    }


    /**
     * The book's cover image.
     *
     * @return The book's cover image.
     */
    pub fn get_cover_image(&self) -> &Option<Resource> {
        &self.cover_image
    }

    pub fn set_cover_image(&mut self, cover_image: Option<Resource>) {
        if cover_image.is_none() {
            return;
        }
        if self.resources.not_contains_by_href(&cover_image.as_ref().unwrap().get_href()) {
            self.resources.add(cover_image.as_ref().unwrap().clone());
        }
        self.cover_image = cover_image;
    }

    /**
     * The guide; contains references to special sections of the book like
     * colophon, glossary, etc.
     *
     * @return The guide; contains references to special sections of the book
     * like colophon, glossary, etc.
     */
    pub fn get_guide(&self) -> &Guide {
        &self.guide
    }

    /**
     * All Resources of the Book that can be reached via the Spine, the
     * TableOfContents or the Guide.
     * <p/>
     * Consists of a list of "reachable" resources:
     * <ul>
     * <li>The coverpage</li>
     * <li>The resources of the Spine that are not already in the result</li>
     * <li>The resources of the Table of Contents that are not already in the
     * result</li>
     * <li>The resources of the Guide that are not already in the result</li>
     * </ul>
     * To get all html files that make up the epub file use
     * {@link #getResources()}
     *
     * @return All Resources of the Book that can be reached via the Spine,
     * the TableOfContents or the Guide.
     */
    pub fn get_contents(&self) -> Vec<Resource> {
        let mut result: Vec<(String, Resource)> = Vec::new();
        EpubBook::add_to_contents_result(&self.get_cover_page(), &mut result);

        for spine_reference in self.get_spine().get_spine_references() {
            EpubBook::add_to_contents_result(&spine_reference.get_resource().clone(), &mut result);
        }

        for resource in self.get_table_of_contents().get_all_unique_resources() {
            EpubBook::add_to_contents_result(&Some(resource), &mut result);
        }

        for guide_reference in self.get_guide().get_references() {
            EpubBook::add_to_contents_result(&guide_reference.get_resource().clone(), &mut result);
        }

        let mut result_resources: Vec<Resource> = Vec::new();
        for (_, resource) in result {
            result_resources.push(resource);
        }
        return result_resources;
    }

    fn add_to_contents_result(resource: &Option<Resource>,
                                            all_reachable_resources: &mut Vec<(String, Resource)>) {
        if resource.is_some() && (!all_reachable_resources.iter()
                .any(|(href, _)| href.eq(resource.as_ref().unwrap().get_href()))) {
            all_reachable_resources.push((resource.as_ref().unwrap().get_href().clone(), resource.as_ref().unwrap().clone()));
        }
    }

    pub fn get_opf_resource(&self) -> &Option<Resource> {
        &self.opf_resource
    }

    pub fn set_opf_resource(&mut self, opf_resource: Option<Resource>) {
        self.opf_resource = opf_resource;
    }

    pub fn set_ncx_resource(&mut self, ncx_resource: Option<Resource>) {
        self.ncx_resource = ncx_resource;
    }

    pub fn get_ncx_resource(&self) -> &Option<Resource> {
        &self.ncx_resource
    }
}
