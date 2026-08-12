use crate::prelude::*;
// package me.ag2s.epublib.domain;
// @SuppressWarnings("unused")
pub enum ManifestItemRefProperties {
	PAGE_SPREAD_LEFT,
	PAGE_SPREAD_RIGHT,
}

impl ManifestProperties for ManifestItemRefProperties {

	fn get_name(&self) -> &'static str {
		match self {
			ManifestItemRefProperties::PAGE_SPREAD_LEFT => "page-spread-left",
			ManifestItemRefProperties::PAGE_SPREAD_RIGHT => "page-spread-right",
		}
	}
}
