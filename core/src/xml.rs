//! Garbage-collectable XML DOM impl

mod document;
mod error;
mod namespace;
mod tree;

#[cfg(test)]
mod tests;

pub use document::XMLDocument;
pub use error::Error;
pub use error::ParseError;
pub use namespace::XMLName;
pub use tree::XMLNode;

pub const ELEMENT_NODE: u8 = 1;
pub const TEXT_NODE: u8 = 3;
pub const COMMENT_NODE: u8 = 8;
pub const DOCUMENT_NODE: u8 = 9;
pub const DOCUMENT_TYPE_NODE: u8 = 10;
