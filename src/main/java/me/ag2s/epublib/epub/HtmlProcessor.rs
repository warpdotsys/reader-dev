use crate::prelude::*;
use crate::me::ag2s::epublib::domain::Resource;
use crate::stubs::OutputStream;

#[allow(dead_code)]
pub trait HtmlProcessor {

    fn process_html_resource(&self, resource: &Resource, out: &mut OutputStream);
}
