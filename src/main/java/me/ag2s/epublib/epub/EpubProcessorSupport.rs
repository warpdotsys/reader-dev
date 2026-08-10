use std::collections::VecDeque;

use crate::me::ag2s::epublib::Constants;

/**
 * Various low-level support methods for reading/writing epubs.
 *
 * @author paul.siegmann
 */
pub struct EpubProcessorSupport;

impl EpubProcessorSupport {

    pub const TAG: &'static str = "me.ag2s.epublib.epub.EpubProcessorSupport";

    pub static mut document_builder_factory: Option<DocumentBuilderFactory> = None;

    pub fn init() {
        EpubProcessorSupport::document_builder_factory = Some(DocumentBuilderFactory::new_instance());
        if let Some(ref mut dbf) = EpubProcessorSupport::document_builder_factory {
            dbf.set_namespace_aware(true);
            dbf.set_validating(false);
        }
    }

    pub fn create_xml_serializer_stream(out: OutputStream) -> XmlSerializer {
        create_xml_serializer_writer(OutputStreamWriter::new(out, Constants::CHARACTER_ENCODING))
    }

    pub fn create_xml_serializer_writer(out: Writer) -> XmlSerializer {
        let mut result = None;
        /*
         * Disable XmlPullParserFactory here before it doesn't work when
         * building native image using GraalVM
         */
        let factory = XmlPullParserFactory::new_instance();
        factory.set_validating(true);
        result = Some(factory.new_serializer());

        //result = new KXmlSerializer();
        result.set_feature("http://xmlpull.org/v1/doc/features.html#indent-output", true);
        result.set_output(out);
        result.unwrap()
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
        EpubProcessorSupport::document_builder_factory
    }

    /**
     * Creates a DocumentBuilder that looks up dtd's and schema's from epub4j's classpath.
     *
     * @return a DocumentBuilder that looks up dtd's and schema's from epub4j's classpath.
     */
    pub fn create_document_builder() -> DocumentBuilder {
        let mut result = None;
        result = Some(DocumentBuilderFactory::new_document_builder());
        result.set_entity_resolver(get_entity_resolver());
        result.unwrap()
    }
}

pub struct EntityResolverImpl {
    pub previous_location: Option<String>,
}

impl EntityResolverImpl {
    pub fn resolve_entity(&mut self, public_id: String, system_id: String) -> InputSource {
        let resource_path: String;
        if system_id.starts_with("http:") {
            let url = URL::new(system_id);
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
