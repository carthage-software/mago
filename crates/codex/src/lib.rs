#![allow(clippy::wildcard_imports)]
#![allow(clippy::exhaustive_enums)]
#![allow(clippy::float_arithmetic)]
#![allow(clippy::pub_use)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::match_wildcard_for_single_variants)]

use mago_span::Span;
use mago_word::Word;
use mago_word::word;

pub mod assertion;
pub mod consts;
pub mod context;
pub mod diff;
pub mod differ;
pub mod flags;
pub mod identifier;
pub mod issue;
pub mod metadata;
pub mod misc;
pub mod populator;
pub mod reference;
pub mod scanner;
pub mod signature;
pub mod signature_builder;
pub mod symbol;
pub mod ttype;
pub mod visibility;

mod utils;

#[must_use]
pub fn get_anonymous_class_name(span: Span) -> Word {
    use std::io::Write;

    let mut buffer = [0u8; 64];
    let mut writer = &mut buffer[..];

    // SAFETY: 64 bytes is ample for the prefix plus three integers; the write cannot exceed it.
    unsafe {
        write!(writer, "class@anonymous:{}-{}:{}", span.file_id, span.start.offset, span.end.offset).unwrap_unchecked();
    };

    let written_len = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());

    word(&buffer[..written_len])
}
