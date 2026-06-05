#![allow(clippy::wildcard_imports)]
#![allow(clippy::exhaustive_enums)]
#![allow(clippy::float_arithmetic)]
#![allow(clippy::pub_use)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::match_wildcard_for_single_variants)]

use mago_database::file::File;
use mago_span::Span;
use mago_word::Word;
use mago_word::concat_word;
use mago_word::u32_word;

pub mod assertion;
pub mod consts;
pub mod context;
pub mod diff;
pub mod differ;
pub mod flags;
pub mod identifier;
pub mod ir_scanner;
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

/// Builds the synthetic display name for an anonymous construct (closure,
/// arrow function, or anonymous class).
///
/// Format: `{<prefix>:<workspace-relative path>:<line>:<column>}`, where the
/// line and column are 1-based and computed from the span's start offset.
/// Matches PHP's own `{closure:...}` stringification.
#[must_use]
pub fn build_synthetic_name(prefix: &str, file: &File, span: Span) -> Word {
    let line = file.line_number(span.start.offset).saturating_add(1);
    let column = file.column_number(span.start.offset).saturating_add(1);

    concat_word!(b"{", prefix.as_bytes(), b":", file.name.as_ref(), b":", u32_word(line), b":", u32_word(column), b"}",)
}

#[must_use]
pub fn get_anonymous_class_name(file: &File, span: Span) -> Word {
    build_synthetic_name("anonymous-class", file, span)
}
