use crate::prelude::*;
// package me.ag2s.epublib;


pub trait Constants {

  const CHARACTER_ENCODING: &'static str = "UTF-8";
  const DOCTYPE_XHTML: &'static str = "<!DOCTYPE HTML PUBLIC \"-//W3C//DTD XHTML 1.1//EN\" \"http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd\">";
  const NAMESPACE_XHTML: &'static str = "http://www.w3.org/1999/xhtml";
  const EPUB_GENERATOR_NAME: &'static str = "Ag2S EpubLib";
  const EPUB_DUOKAN_NAME: &'static str = "DK-SONGTI";
  const FRAGMENT_SEPARATOR_CHAR: char = '#';
  const DEFAULT_TOC_ID: &'static str = "toc";
}
