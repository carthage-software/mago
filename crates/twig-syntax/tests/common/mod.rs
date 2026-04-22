//! Shared helpers for the Twig syntax integration tests.
//!
//! Included via `#[path = "common/mod.rs"] mod common;` at the top of each
//! integration binary because Cargo compiles each file in `tests/` as a
//! separate crate.

#![allow(dead_code)]

use bumpalo::Bump;

use mago_database::file::FileId;
use mago_syntax_core::input::Input;
use mago_twig_syntax::ast::Template;
use mago_twig_syntax::error::ParseError;
use mago_twig_syntax::error::SyntaxError;
use mago_twig_syntax::lexer::TwigLexer;
use mago_twig_syntax::parser::parse_file_content;
use mago_twig_syntax::settings::LexerSettings;
use mago_twig_syntax::token::TwigToken;
use mago_twig_syntax::token::TwigTokenKind;

/// Parse `source` under [`FileId::zero`] with default settings.
pub fn parse<'arena>(arena: &'arena Bump, source: &'arena str) -> &'arena Template<'arena> {
    parse_file_content(arena, FileId::zero(), source)
}

/// Tokenise the source and collect every token (including trivia).
pub fn tokenize(source: &str) -> Result<Vec<TwigToken<'_>>, SyntaxError> {
    let input = Input::new(FileId::zero(), source.as_bytes());
    let mut lexer = TwigLexer::new(input, LexerSettings::default());
    let mut out = Vec::new();
    while let Some(result) = lexer.advance() {
        out.push(result?);
    }

    Ok(out)
}

/// Tokenise `source` and stitch the token values back together. The
/// result must equal the input byte-for-byte; any divergence indicates a
/// non-lossless lexer.
pub fn roundtrip_tokens(source: &str) -> Result<String, SyntaxError> {
    let tokens = tokenize(source)?;
    let mut out = String::with_capacity(source.len());
    for tok in &tokens {
        out.push_str(tok.value);
    }

    Ok(out)
}

pub fn parse_ok<'a>(arena: &'a Bump, src: &'a str) -> &'a Template<'a> {
    let tpl = parse(arena, src);
    if tpl.has_errors() {
        let joined: Vec<String> = tpl.errors.iter().map(|e| e.to_string()).collect();
        panic!("expected {src:?} to parse but got: {}", joined.join("; "));
    }

    tpl
}

pub fn parses(src: &str) {
    let arena = Bump::new();
    let tpl = parse(&arena, src);
    if tpl.has_errors() {
        let joined: Vec<String> = tpl.errors.iter().map(|e| e.to_string()).collect();
        panic!("expected {src:?} to parse but got: {}", joined.join("; "));
    }
}

pub fn rejects(src: &str) {
    let arena = Bump::new();
    let tpl = parse(&arena, src);
    if !tpl.has_errors() {
        panic!("expected {src:?} to be rejected");
    }
}

pub fn rejects_with<F: Fn(&ParseError) -> bool>(src: &str, matcher: F) {
    let arena = Bump::new();
    let tpl = parse(&arena, src);
    if !tpl.has_errors() {
        panic!("expected {src:?} to be rejected");
    }
    assert!(tpl.errors.iter().any(matcher), "error did not match expectation for {src:?}: {:?}", tpl.errors,);
}

pub fn roundtrip(src: &str) {
    let out = roundtrip_tokens(src).unwrap_or_else(|e| panic!("tokenize failed for {src:?}: {e}"));
    assert_eq!(out, src, "round-trip mismatch for input {src:?}");
}

pub fn parse_and_roundtrip(src: &str) {
    roundtrip(src);
    parses(src);
}

/// Tokenise (no parse) and return a `Vec` for easy structural assertions.
pub fn lex(src: &str) -> Vec<TwigToken<'_>> {
    tokenize(src).unwrap_or_else(|e| panic!("tokenize failed for {src:?}: {e}"))
}

/// Kinds of all tokens produced by lexing `src`.
pub fn kinds(src: &str) -> Vec<TwigTokenKind> {
    lex(src).into_iter().map(|t| t.kind).collect()
}

/// `(kind, value)` pairs for all tokens produced by lexing `src`.
pub fn kinds_and_values(src: &str) -> Vec<(TwigTokenKind, String)> {
    lex(src).into_iter().map(|t| (t.kind, t.value.to_string())).collect()
}

/// Return the first token matching `pred` (non-trivia by default if caller
/// passes the appropriate predicate).
pub fn find_first_token<'a, F: Fn(&TwigToken<'a>) -> bool>(src: &'a str, pred: F) -> Option<TwigToken<'a>> {
    let toks = tokenize(src).ok()?;
    toks.into_iter().find(|t| pred(t))
}
