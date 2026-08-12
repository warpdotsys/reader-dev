#![allow(unused, dead_code, non_snake_case, non_camel_case_types, non_upper_case_globals, clippy::all, deprecated)]

#[path = "../src/main/java/org/kxml2/io/KXmlParser.rs"]
pub mod org_kxml2_io_kxmlparser;

#[path = "../src/main/java/org/kxml2/io/KXmlSerializer.rs"]
pub mod org_kxml2_io_kxmlserializer;

#[path = "../src/main/java/org/kxml2/kdom/Document.rs"]
pub mod org_kxml2_kdom_document;

#[path = "../src/main/java/org/kxml2/kdom/Element.rs"]
pub mod org_kxml2_kdom_element;

#[path = "../src/main/java/org/kxml2/kdom/Node.rs"]
pub mod org_kxml2_kdom_node;

#[path = "../src/main/java/org/kxml2/wap/Wbxml.rs"]
pub mod org_kxml2_wap_wbxml;

#[path = "../src/main/java/org/kxml2/wap/WbxmlParser.rs"]
pub mod org_kxml2_wap_wbxmlparser;

#[path = "../src/main/java/org/kxml2/wap/WbxmlSerializer.rs"]
pub mod org_kxml2_wap_wbxmlserializer;

#[path = "../src/main/java/org/kxml2/wap/syncml/SyncML.rs"]
pub mod org_kxml2_wap_syncml_syncml;

#[path = "../src/main/java/org/kxml2/wap/wml/Wml.rs"]
pub mod org_kxml2_wap_wml_wml;

#[path = "../src/main/java/org/kxml2/wap/wv/WV.rs"]
pub mod org_kxml2_wap_wv_wv;

pub mod stubs { include!("stubs.rs"); }

pub mod prelude {
    pub use crate::stubs::*;
    pub use crate::org_kxml2_io_kxmlparser::*;
    pub use crate::org_kxml2_io_kxmlserializer::*;
    pub use crate::org_kxml2_kdom_document::*;
    pub use crate::org_kxml2_kdom_element::*;
    pub use crate::org_kxml2_kdom_node::*;
    pub use crate::org_kxml2_wap_wbxml::*;
    pub use crate::org_kxml2_wap_wbxmlparser::*;
    pub use crate::org_kxml2_wap_wbxmlserializer::*;
    pub use crate::org_kxml2_wap_syncml_syncml::*;
    pub use crate::org_kxml2_wap_wml_wml::*;
    pub use crate::org_kxml2_wap_wv_wv::*;
}
