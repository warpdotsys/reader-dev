use crate::prelude::*;
use crate::me_ag2s_epublib_domain_date::Date;
use crate::me_ag2s_epublib_domain_mediatypes::MediaTypes;
// package me.ag2s.epublib.domain;

// import me.ag2s.epublib.util.StringUtil;

// import java.io.Serializable;
// import java.util.ArrayList;
// import java.util.HashMap;
// import java.util.List;
// import java.util.Map;

// import javax.xml.namespace.QName;

/**
 * A Book's collection of Metadata.
 * In the future it should contain all Dublin Core attributes, for now
 * it contains a set of often-used ones.
 *
 * @author paul
 */
pub struct Metadata {

    auto_generated_id: bool,
    authors: Vec<Author>,
    contributors: Vec<Author>,
    dates: Vec<Date>,
    language: String,
    other_properties: HashMap<QName, String>,
    rights: Vec<String>,
    titles: Vec<String>,
    identifiers: Vec<Identifier>,
    subjects: Vec<String>,
    format: String,
    types: Vec<String>,
    descriptions: Vec<String>,
    publishers: Vec<String>,
    meta_attributes: HashMap<String, String>,
}

impl Metadata {

    pub const DEFAULT_LANGUAGE: &'static str = "en";

    pub fn new() -> Metadata {
        let mut result = Metadata {
            auto_generated_id: false,
            authors: Vec::new(),
            contributors: Vec::new(),
            dates: Vec::new(),
            language: Metadata::DEFAULT_LANGUAGE.to_string(),
            other_properties: HashMap::new(),
            rights: Vec::new(),
            titles: Vec::new(),
            identifiers: Vec::new(),
            subjects: Vec::new(),
            format: MediaTypes::EPUB.get_name().clone(),
            types: Vec::new(),
            descriptions: Vec::new(),
            publishers: Vec::new(),
            meta_attributes: HashMap::new(),
        };
        result.identifiers.push(Identifier::new());
        result.auto_generated_id = true;
        result
    }

    // @SuppressWarnings("unused")
    pub fn is_auto_generated_id(&self) -> bool {
        return self.auto_generated_id;
    }

    /**
     * Metadata properties not hard-coded like the author, title, etc.
     *
     * @return Metadata properties not hard-coded like the author, title, etc.
     */
    pub fn get_other_properties(&self) -> &HashMap<QName, String> {
        &self.other_properties
    }

    pub fn set_other_properties(&mut self, other_properties: HashMap<QName, String>) {
        self.other_properties = other_properties;
    }

    // @SuppressWarnings("unused")
    pub fn add_date(&mut self, date: Date) -> Date {
        self.dates.push(date.clone());
        return date;
    }

    pub fn get_dates(&self) -> &Vec<Date> {
        &self.dates
    }

    pub fn set_dates(&mut self, dates: Vec<Date>) {
        self.dates = dates;
    }

    // @SuppressWarnings("UnusedReturnValue")
    pub fn add_author(&mut self, author: Author) -> Author {
        self.authors.push(author.clone());
        return author;
    }

    pub fn get_authors(&self) -> &Vec<Author> {
        &self.authors
    }

    pub fn set_authors(&mut self, authors: Vec<Author>) {
        self.authors = authors;
    }

    // @SuppressWarnings("UnusedReturnValue")
    pub fn add_contributor(&mut self, contributor: Author) -> Author {
        self.contributors.push(contributor.clone());
        return contributor;
    }

    pub fn get_contributors(&self) -> &Vec<Author> {
        &self.contributors
    }

    pub fn set_contributors(&mut self, contributors: Vec<Author>) {
        self.contributors = contributors;
    }

    pub fn get_language(&self) -> &String {
        &self.language
    }

    pub fn set_language(&mut self, language: String) {
        self.language = language;
    }

    pub fn get_subjects(&self) -> &Vec<String> {
        &self.subjects
    }

    pub fn set_subjects(&mut self, subjects: Vec<String>) {
        self.subjects = subjects;
    }

    pub fn set_rights(&mut self, rights: Vec<String>) {
        self.rights = rights;
    }

    pub fn get_rights(&self) -> &Vec<String> {
        &self.rights
    }


    /**
     * Gets the first non-blank title of the book.
     * Will return "" if no title found.
     *
     * @return the first non-blank title of the book.
     */
    pub fn get_first_title(&self) -> String {
        if self.titles.is_empty() {
            return "".to_string();
        }
        for title in &self.titles {
            if StringUtil::is_not_blank(title) {
                return title.clone();
            }
        }
        return "".to_string();
    }

    pub fn add_title(&mut self, title: String) -> String {
        self.titles.push(title.clone());
        return title;
    }

    pub fn set_titles(&mut self, titles: Vec<String>) {
        self.titles = titles;
    }

    pub fn get_titles(&self) -> &Vec<String> {
        &self.titles
    }

    // @SuppressWarnings("UnusedReturnValue")
    pub fn add_publisher(&mut self, publisher: String) -> String {
        self.publishers.push(publisher.clone());
        return publisher;
    }

    pub fn set_publishers(&mut self, publishers: Vec<String>) {
        self.publishers = publishers;
    }

    pub fn get_publishers(&self) -> &Vec<String> {
        &self.publishers
    }

    // @SuppressWarnings("UnusedReturnValue")
    pub fn add_description(&mut self, description: String) -> String {
        self.descriptions.push(description.clone());
        return description;
    }

    pub fn set_descriptions(&mut self, descriptions: Vec<String>) {
        self.descriptions = descriptions;
    }

    pub fn get_descriptions(&self) -> &Vec<String> {
        &self.descriptions
    }

    // @SuppressWarnings("unused")
    pub fn add_identifier(&mut self, identifier: Identifier) -> Identifier {
        if self.auto_generated_id && (!(self.identifiers.is_empty())) {
            self.identifiers[0] = identifier.clone();
        } else {
            self.identifiers.push(identifier.clone());
        }
        self.auto_generated_id = false;
        return identifier;
    }

    pub fn set_identifiers(&mut self, identifiers: Vec<Identifier>) {
        self.identifiers = identifiers;
        self.auto_generated_id = false;
    }

    pub fn get_identifiers(&self) -> &Vec<Identifier> {
        &self.identifiers
    }

    pub fn set_format(&mut self, format: String) {
        self.format = format;
    }

    pub fn get_format(&self) -> &String {
        &self.format
    }

    // @SuppressWarnings("UnusedReturnValue")
    pub fn add_type(&mut self, type_: String) -> String {
        self.types.push(type_.clone());
        return type_;
    }

    pub fn get_types(&self) -> &Vec<String> {
        &self.types
    }

    pub fn set_types(&mut self, types: Vec<String>) {
        self.types = types;
    }

    // @SuppressWarnings("unused")
    pub fn get_meta_attribute(&self, name: &String) -> Option<&String> {
        return self.meta_attributes.get(name);
    }

    pub fn set_meta_attributes(&mut self, meta_attributes: HashMap<String, String>) {
        self.meta_attributes = meta_attributes;
    }
}
