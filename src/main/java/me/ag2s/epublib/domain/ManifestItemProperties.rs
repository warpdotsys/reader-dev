use crate::prelude::*;
// package me.ag2s.epublib.domain;
// @SuppressWarnings("unused")
pub enum ManifestItemProperties {
  COVER_IMAGE,
  MATHML,
  NAV,
  REMOTE_RESOURCES,
  SCRIPTED,
  SVG,
  SWITCH,
}

impl ManifestProperties for ManifestItemProperties {

  fn get_name(&self) -> &'static str {
    match self {
      ManifestItemProperties::COVER_IMAGE => "cover-image",
      ManifestItemProperties::MATHML => "mathml",
      ManifestItemProperties::NAV => "nav",
      ManifestItemProperties::REMOTE_RESOURCES => "remote-resources",
      ManifestItemProperties::SCRIPTED => "scripted",
      ManifestItemProperties::SVG => "svg",
      ManifestItemProperties::SWITCH => "switch",
    }
  }
}
