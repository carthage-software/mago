#![allow(clippy::pub_use)]
#![allow(clippy::exhaustive_enums)]

pub mod cst;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod token;

pub use crate::parser::PHPDocParser;
pub use crate::parser::parse_type;
