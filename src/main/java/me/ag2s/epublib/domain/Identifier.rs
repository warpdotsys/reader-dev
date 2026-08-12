use crate::prelude::*;
// fix: E0404/E0659——prelude glob 的 `Any` 是 stubs 枚举，需 std::any::Any trait，显式导入覆盖
use std::any::Any;
// package me.ag2s.epublib.domain;

// import me.ag2s.epublib.util.StringUtil;
// import java.io.Serializable;
// import java.util.List;
// import java.util.UUID;

/**
 * A Book's identifier.
 *
 * Defaults to a random UUID and scheme "UUID"
 *
 * @author paul
 */
// fix: 克隆（get_book_id_identifier 使用 identifier.clone()），派生 Clone
#[derive(Clone)]
pub struct Identifier {

  book_id: bool,
  scheme: String,
  value: String,
}

// fix: Java 的嵌套接口 `Identifier.Scheme` 无法在 Rust impl 内声明 trait，已移到模块级
// fix: E0790——trait 关联常量不能以 `Scheme::UUID` 访问，改为单元 struct + 关联常量
pub struct Scheme;
impl Scheme {
  pub const UUID: &'static str = "UUID";
  pub const ISBN: &'static str = "ISBN";
  pub const URL: &'static str = "URL";
  pub const URI: &'static str = "URI";
}

impl Identifier {

  /**
   * Creates an Identifier with as value a random UUID and scheme "UUID"
   */
  pub fn new() -> Identifier {
    Identifier::with_value(Scheme::UUID.to_string(), Uuid::random_uuid().to_string())
  }


  pub fn with_value(scheme: String, value: String) -> Identifier {
    Identifier {
      scheme: scheme,
      value: value,
      book_id: false,
    }
  }

  /**
   * The first identifier for which the bookId is true is made the
   * bookId identifier.
   *
   * If no identifier has bookId == true then the first bookId identifier
   * is written as the primary.
   *
   * @param identifiers i
   * @return The first identifier for which the bookId is true is made
   * 		the bookId identifier.
   */
  pub fn get_book_id_identifier(identifiers: &Vec<Identifier>) -> Option<Identifier> {
    if identifiers.is_empty() {
      return None;
    }

    let mut result: Option<Identifier> = None;
    for identifier in identifiers {
      if identifier.is_book_id() {
        result = Some(identifier.clone());
        break;
      }
    }

    if result.is_none() {
      result = Some(identifiers.get(0).unwrap().clone());
    }

    return result;
  }

  pub fn get_scheme(&self) -> &String {
    &self.scheme
  }

  pub fn set_scheme(&mut self, scheme: String) {
    self.scheme = scheme;
  }

  pub fn get_value(&self) -> &String {
    &self.value
  }

  pub fn set_value(&mut self, value: String) {
    self.value = value;
  }


  pub fn set_book_id(&mut self, book_id: bool) {
    self.book_id = book_id;
  }


  /**
   * This bookId property allows the book creator to add multiple ids and
   * tell the epubwriter which one to write out as the bookId.
   *
   * The Dublin Core metadata spec allows multiple identifiers for a Book.
   * The epub spec requires exactly one identifier to be marked as the book id.
   *
   * @return whether this is the unique book id.
   */
  pub fn is_book_id(&self) -> bool {
    self.book_id
  }

  pub fn hash_code(&self) -> i32 {
    StringUtil::default_if_null(&self.scheme).hashCode() ^ StringUtil
        ::default_if_null(&self.value).hashCode()
  }

  pub fn equals(&self, other_identifier: &dyn Any) -> bool {
    if !(other_identifier.is::<Identifier>()) {
      return false;
    }
    return StringUtil::equals(&self.scheme, &other_identifier.downcast_ref::<Identifier>().unwrap().scheme)
        && StringUtil::equals(&self.value, &other_identifier.downcast_ref::<Identifier>().unwrap().value);
  }
  // @SuppressWarnings("NullableProblems")
  // @Override
  pub fn to_string(&self) -> String {
    if StringUtil::is_blank(&self.scheme) {
      return format!("{}", self.value);
    }
    return format!("{}:{}", self.scheme, self.value);
  }
}
