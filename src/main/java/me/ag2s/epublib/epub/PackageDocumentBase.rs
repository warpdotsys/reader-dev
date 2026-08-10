/**
 * Functionality shared by the PackageDocumentReader and the PackageDocumentWriter
 *
 * @author paul
 *
 */
pub struct PackageDocumentBase;

impl PackageDocumentBase {

    pub const BOOK_ID_ID: &'static str = "duokan-book-id";
    pub const NAMESPACE_OPF: &'static str = "http://www.idpf.org/2007/opf";
    pub const NAMESPACE_DUBLIN_CORE: &'static str = "http://purl.org/dc/elements/1.1/";
    pub const PREFIX_DUBLIN_CORE: &'static str = "dc";
    //public static final String PREFIX_OPF = "opf";
    //在EPUB3标准中，packge前面没有opf头，一些epub阅读器也不支持opf头。
    //Some Epub Reader not reconize op:packge,So just let it empty;
    pub const PREFIX_OPF: &'static str = "";
    //添加 version 变量来区分Epub文件的版本
    //Add the version field to distinguish the version of EPUB file
    pub const VERSION: &'static str = "version";
    pub const DATE_FORMAT: &'static str = "yyyy-MM-dd";

    const DC_TAGS_TITLE: &'static str = "title";
    const DC_TAGS_CREATOR: &'static str = "creator";
    const DC_TAGS_SUBJECT: &'static str = "subject";
    const DC_TAGS_DESCRIPTION: &'static str = "description";
    const DC_TAGS_PUBLISHER: &'static str = "publisher";
    const DC_TAGS_CONTRIBUTOR: &'static str = "contributor";
    const DC_TAGS_DATE: &'static str = "date";
    const DC_TAGS_TYPE: &'static str = "type";
    const DC_TAGS_FORMAT: &'static str = "format";
    const DC_TAGS_IDENTIFIER: &'static str = "identifier";
    const DC_TAGS_SOURCE: &'static str = "source";
    const DC_TAGS_LANGUAGE: &'static str = "language";
    const DC_TAGS_RELATION: &'static str = "relation";
    const DC_TAGS_COVERAGE: &'static str = "coverage";
    const DC_TAGS_RIGHTS: &'static str = "rights";

    const DC_ATTRIBUTES_SCHEME: &'static str = "scheme";
    const DC_ATTRIBUTES_ID: &'static str = "id";

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
    const OPF_ATTRIBUTES_EVENT: &'static str = "event";
    const OPF_ATTRIBUTES_ROLE: &'static str = "role";
    const OPF_ATTRIBUTES_FILE_AS: &'static str = "file-as";
    const OPF_ATTRIBUTES_ID: &'static str = "id";
    const OPF_ATTRIBUTES_MEDIA_TYPE: &'static str = "media-type";
    const OPF_ATTRIBUTES_TITLE: &'static str = "title";
    const OPF_ATTRIBUTES_TOC: &'static str = "toc";
    const OPF_ATTRIBUTES_VERSION: &'static str = "version";
    const OPF_ATTRIBUTES_SCHEME: &'static str = "scheme";
    const OPF_ATTRIBUTES_PROPERTY: &'static str = "property";
    //add for epub3
    /**
     * add for epub3
     */
    const OPF_ATTRIBUTES_PROPERTIES: &'static str = "properties";

    const OPF_VALUES_META_COVER: &'static str = "cover";
    const OPF_VALUES_REFERENCE_COVER: &'static str = "cover";
    const OPF_VALUES_NO: &'static str = "no";
    const OPF_VALUES_GENERATOR: &'static str = "generator";
    const OPF_VALUES_DUOKAN: &'static str = "duokan-body-font";
}
