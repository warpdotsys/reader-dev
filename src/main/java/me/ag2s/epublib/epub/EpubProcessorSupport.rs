use crate::prelude::*;
use std::collections::VecDeque;

// fix: Constants 转录为 trait，关联常量无法跨模块以 `Constants::CHARACTER_ENCODING` 访问；镜像常量值（与 Java 一致）
const CHARACTER_ENCODING: &'static str = "UTF-8";

/**
 * Various low-level support methods for reading/writing epubs.
 *
 * @author paul.siegmann
 */
pub struct EpubProcessorSupport;

// fix: Java 的 `protected static DocumentBuilderFactory documentBuilderFactory` 字段
// fix: → 模块级 `static mut`（Rust 不允许 impl 内出现 associated static item）
pub static mut document_builder_factory: Option<DocumentBuilderFactory> = None;

impl EpubProcessorSupport {

    pub const TAG: &'static str = "me.ag2s.epublib.epub.EpubProcessorSupport";

    pub fn init() {
        // fix: 访问 static mut 需要 unsafe
        unsafe {
            document_builder_factory = Some(DocumentBuilderFactory::new_instance());
        }
        if let Some(ref mut dbf) = unsafe { document_builder_factory.as_mut() } {
            dbf.set_namespace_aware(true);
            dbf.set_validating(false);
        }
    }

    pub fn create_xml_serializer_stream(out: OutputStream) -> XmlSerializer {
        Self::create_xml_serializer_writer(OutputStreamWriter::new(out, CHARACTER_ENCODING))
    }

    pub fn create_xml_serializer_writer(out: Writer) -> XmlSerializer {
        /*
         * Disable XmlPullParserFactory here before it doesn't work when
         * building native image using GraalVM
         */
        let mut factory = XmlPullParserFactory::new_instance();
        factory.set_validating(true);
        // fix: 原 Java `var result: XmlSerializer` 误转录为 Option，直接持有 serializer
        let mut result = factory.new_serializer();

        //result = new KXmlSerializer();
        result.set_feature("http://xmlpull.org/v1/doc/features.html#indent-output", true);
        result.set_output(out);
        result
    }

    /**
     * Gets an EntityResolver that loads dtd's and such from the epub4j classpath.
     * In order to enable the loading of relative urls the given EntityResolver contains the previousLocation.
     * Because of a new EntityResolver is created every time this method is called.
     * Fortunately the EntityResolver created uses up very little memory per instance.
     *
     * @return an EntityResolver that loads dtd's and such from the epub4j classpath.
     */
    pub fn get_entity_resolver() -> EntityResolverImpl {
        EntityResolverImpl { previous_location: None }
    }

    #[allow(dead_code)]
    pub fn get_document_builder_factory(&self) -> Option<DocumentBuilderFactory> {
        // fix: static mut 已移到模块级，读取需要 unsafe；不可 move，用 as_ref().cloned()
        unsafe { document_builder_factory.as_ref().cloned() }
    }

    /**
     * Creates a DocumentBuilder that looks up dtd's and schema's from epub4j's classpath.
     *
     * @return a DocumentBuilder that looks up dtd's and schema's from epub4j's classpath.
     */
    pub fn create_document_builder() -> DocumentBuilder {
        let mut result = None;
        // fix: 对应 Java `documentBuilderFactory.newDocumentBuilder()`（static mut 访问需 unsafe；不可 move，用 as_ref()）
        result = Some(unsafe { document_builder_factory.as_ref() }.unwrap().new_document_builder());
        result.as_mut().unwrap().set_entity_resolver(Self::get_entity_resolver());
        result.unwrap()
    }

    // fix: 占位实现（原 Java ClassLoader.getResource / getResourceAsStream 语义未转录）
    pub fn get_resource(resource_path: &str) -> Option<()> {
        let _ = resource_path;
        None
    }
    pub fn get_resource_as_stream(resource_path: &str) -> Option<InputStream> {
        let _ = resource_path;
        None
    }
}

pub struct EntityResolverImpl {
    pub previous_location: Option<String>,
}

impl EntityResolverImpl {
    pub fn resolve_entity(&mut self, public_id: String, system_id: String) -> InputSource {
        let resource_path: String;
        if system_id.starts_with("http:") {
            let url = URL::new(system_id.clone());
            resource_path = "dtd/".to_string() + &url.get_host() + &url.get_path();
            self.previous_location = Some(resource_path.clone()[0..resource_path.rfind('/').unwrap()].to_string());
        } else {
            resource_path = self.previous_location.clone().unwrap() + &system_id[system_id.rfind('/').unwrap()..];
        }

        if EpubProcessorSupport::get_resource(resource_path.as_str()).is_none() {
            panic!("remote resource is not cached : [{}] cannot continue", system_id);
        }

        let in_stream = EpubProcessorSupport::get_resource_as_stream(resource_path.as_str()).unwrap();
        InputSource::new(in_stream)
    }
}

#[derive(Clone)]
pub struct DocumentBuilderFactory;
pub struct DocumentBuilder;
pub struct XmlSerializer;
pub struct XmlPullParserFactory;
pub struct Writer;
pub struct OutputStream;
pub struct OutputStreamWriter;
pub struct InputSource;
pub struct URL;

impl DocumentBuilderFactory {
    pub fn new_instance() -> Self { todo!() }
    pub fn set_namespace_aware(&mut self, _b: bool) { todo!() }
    pub fn set_validating(&mut self, _b: bool) { todo!() }
    pub fn new_document_builder(&self) -> DocumentBuilder { todo!() }
}

impl DocumentBuilder {
    pub fn set_entity_resolver(&mut self, _resolver: EntityResolverImpl) { todo!() }
}

impl XmlSerializer {
    pub fn set_feature(&mut self, _name: &str, _value: bool) { todo!() }
    pub fn set_output(&mut self, _out: Writer) { todo!() }
}

impl XmlPullParserFactory {
    pub fn new_instance() -> Self { todo!() }
    pub fn set_validating(&mut self, _b: bool) { todo!() }
    pub fn new_serializer(&self) -> XmlSerializer { todo!() }
}

impl OutputStreamWriter {
    pub fn new(out: OutputStream, _charset: &str) -> Writer { todo!() }
}

impl InputSource {
    pub fn new(_in_stream: InputStream) -> Self { todo!() }
}

impl URL {
    pub fn new(_s: String) -> Self { todo!() }
    pub fn get_host(&self) -> String { todo!() }
    pub fn get_path(&self) -> String { todo!() }
}

pub type InputStream = VecDeque<u8>;
