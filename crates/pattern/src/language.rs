//! Mago's language binding for the grit-pattern-matcher engine.
//!
//! Implements [`grit_util::Language`] for Mago's PHP AST. The language tells the engine how
//! to:
//!
//! * Recognize metavariables (we use `^name` as the surface sigil and `µname` as the
//!   substituted form the PHP parser sees). This matches the grit ecosystem convention
//!   for PHP, since `$` is already a real PHP variable sigil.
//! * Wrap a user-written snippet in a context that makes it parse as valid PHP.
//! * Distinguish comments and statements from other nodes.
//! * Pad snippets when inserting them (no-op here, since PHP isn't whitespace-sensitive).

use std::borrow::Cow;
use std::sync::OnceLock;

use grit_util::AstNode;
use grit_util::ByteRange;
use grit_util::CodeRange;
use grit_util::EffectRange;
use grit_util::Language;
use regex::Regex;

use crate::node::MagoNode;

/// The PHP language binding.
///
/// Stateless and constructed on demand. All configuration (regexes, snippet contexts) lives
/// in `static` tables.
#[derive(Clone, Debug, Default)]
pub struct MagoLanguage;

impl MagoLanguage {
    pub const fn new() -> Self {
        Self
    }
}

/// Regex that finds metavariables in a snippet source (`^name`, `^...`, or `^_`).
fn metavariable_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\^(\.\.\.|[A-Za-z_][A-Za-z0-9_]*)").expect("invalid metavariable regex"))
}

/// Regex matching the substituted form (`µname`) left behind in the parsed PHP source.
fn replaced_variable_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\bµ[A-Za-z_][A-Za-z0-9_]*").expect("invalid replaced variable regex"))
}

/// Regex for bracketed metavariables: `^[name]`.
fn bracket_variable_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\^\[([A-Za-z_][A-Za-z0-9_]*)\]").expect("invalid bracket variable regex"))
}

/// Regex matching a string that is *exactly* a metavariable (no surrounding chars).
fn exact_variable_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\^([A-Za-z_][A-Za-z0-9_]*)$").expect("invalid exact variable regex"))
}

/// Regex matching a string that is exactly a substituted metavariable (`µname` alone).
fn exact_replaced_variable_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^µ[A-Za-z_][A-Za-z0-9_]*$").expect("invalid exact replaced variable regex"))
}

/// Snippet context wrappers. The engine tries each `(prefix, suffix)` pair around the
/// user's snippet until one parses. Listed from most specific (expression statement) to
/// most permissive (whole file).
const PHP_SNIPPET_CONTEXTS: &[(&str, &str)] = &[
    ("<?php ", ";"),
    ("<?php $__grit_tmp = ", ";"),
    ("<?php class __Grit { ", " }"),
    ("<?php function __grit_fn() { ", " }"),
    ("<?php ", ""),
    ("", ""),
];

impl Language for MagoLanguage {
    type Node<'a> = MagoNode<'a>;

    fn language_name(&self) -> &'static str {
        "PHP"
    }

    fn snippet_context_strings(&self) -> &[(&'static str, &'static str)] {
        PHP_SNIPPET_CONTEXTS
    }

    fn metavariable_prefix(&self) -> &'static str {
        "^"
    }

    fn comment_prefix(&self) -> &'static str {
        "//"
    }

    fn metavariable_regex(&self) -> &'static Regex {
        metavariable_regex()
    }

    fn substitute_metavariable_prefix(&self, src: &str) -> String {
        substitute_metavariable_prefix(src)
    }

    fn replaced_metavariable_regex(&self) -> &'static Regex {
        replaced_variable_regex()
    }

    fn metavariable_bracket_regex(&self) -> &'static Regex {
        bracket_variable_regex()
    }

    fn exact_variable_regex(&self) -> &'static Regex {
        exact_variable_regex()
    }

    fn exact_replaced_variable_regex(&self) -> &'static Regex {
        exact_replaced_variable_regex()
    }

    fn is_comment(&self, node: &Self::Node<'_>) -> bool {
        node.is_comment()
    }

    fn is_metavariable(&self, node: &Self::Node<'_>) -> bool {
        node.is_metavariable()
    }

    fn is_statement(&self, node: &Self::Node<'_>) -> bool {
        node.is_statement_kind()
    }

    fn align_padding<'a>(
        &self,
        node: &Self::Node<'a>,
        range: &CodeRange,
        _skip_ranges: &[CodeRange],
        _new_padding: Option<usize>,
        _offset: usize,
        _substitutions: &mut [(EffectRange, String)],
    ) -> Cow<'a, str> {
        let src = node.source();
        Cow::Borrowed(&src[range.start as usize..range.end as usize])
    }

    fn pad_snippet<'a>(&self, snippet: &'a str, _padding: &str) -> Cow<'a, str> {
        Cow::Borrowed(snippet)
    }

    fn comment_text_range(&self, node: &Self::Node<'_>) -> Option<ByteRange> {
        Some(node.byte_range())
    }

    fn get_skip_padding_ranges(&self, _node: &Self::Node<'_>) -> Vec<CodeRange> {
        Vec::new()
    }
}

/// Metavariable substitution with PHP-aware position detection.
///
/// For each `^name` occurrence in `src`, peek ahead past the name's identifier bytes at
/// the first non-whitespace byte. If that byte opens an access chain (`->`, `?->`, `[`)
/// or is a variable-position assignment LHS, we emit `$µname` so the host parser treats
/// the slot as a variable. Otherwise we emit `µname` (bare identifier).
///
/// `::` and `(` keep the bare-identifier form because they require class-name or
/// function-name positions, both of which accept identifiers.
fn substitute_metavariable_prefix(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$'
            && bytes.get(i + 1) == Some(&b'^')
            && bytes.get(i + 2).map(|&b| is_ident_start(b)).unwrap_or(false)
        {
            let name_start = i + 2;
            let mut j = name_start;
            while j < bytes.len() && is_ident_continue(bytes[j]) {
                j += 1;
            }
            out.push('$');
            out.push('µ');
            // SAFETY: bytes[name_start..j] was scanned with is_ident_start + is_ident_continue,
            // which only accept ASCII alphanumerics and `_`; the slice is therefore valid UTF-8.
            out.push_str(unsafe { std::str::from_utf8_unchecked(&bytes[name_start..j]) });
            i = j;
            continue;
        }

        if bytes[i] == b'^' {
            if bytes.get(i + 1) == Some(&b'.') && bytes.get(i + 2) == Some(&b'.') && bytes.get(i + 3) == Some(&b'.') {
                let name_start = i + 4;
                let mut j = name_start;
                while j < bytes.len() && is_ident_continue(bytes[j]) {
                    j += 1;
                }
                out.push_str("__MAGO_DOTS_");
                if j > name_start {
                    // SAFETY: see above; the scanned range is ASCII ident bytes only.
                    out.push_str(unsafe { std::str::from_utf8_unchecked(&bytes[name_start..j]) });
                } else {
                    out.push('_');
                }
                i = j;
                continue;
            }

            if let Some(&next) = bytes.get(i + 1)
                && is_ident_start(next)
            {
                let name_start = i + 1;
                let mut j = name_start;
                while j < bytes.len() && is_ident_continue(bytes[j]) {
                    j += 1;
                }
                let mut k = j;
                while k < bytes.len() && matches!(bytes[k], b' ' | b'\t' | b'\n' | b'\r') {
                    k += 1;
                }
                let wants_variable = matches!(
                    (bytes.get(k), bytes.get(k + 1), bytes.get(k + 2)),
                    (Some(&b'-'), Some(&b'>'), _) | (Some(&b'?'), Some(&b'-'), Some(&b'>')) | (Some(&b'['), _, _)
                );
                if wants_variable {
                    out.push('$');
                }
                out.push('µ');
                // SAFETY: see above; the scanned range is ASCII ident bytes only.
                out.push_str(unsafe { std::str::from_utf8_unchecked(&bytes[name_start..j]) });
                i = j;
                continue;
            }
        }
        let width = utf8_char_width(bytes[i]);
        // SAFETY: `src` was passed in as `&str`, so `bytes` is valid UTF-8. `width` is
        // computed from the leading byte, so `bytes[i..i + width]` is a single UTF-8 char.
        out.push_str(unsafe { std::str::from_utf8_unchecked(&bytes[i..i + width]) });
        i += width;
    }
    out
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_ident_continue(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn utf8_char_width(first_byte: u8) -> usize {
    match first_byte {
        0b0000_0000..=0b0111_1111 => 1,
        0b1100_0000..=0b1101_1111 => 2,
        0b1110_0000..=0b1110_1111 => 3,
        0b1111_0000..=0b1111_0111 => 4,
        _ => 1,
    }
}
