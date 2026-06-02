use bumpalo::Bump;

use mago_database::file::FileId;
use mago_phpdoc_syntax::PHPDocParser;
use mago_phpdoc_syntax::cst::Document;
use mago_phpdoc_syntax::cst::Element;
use mago_phpdoc_syntax::cst::TagValue;
use mago_phpdoc_syntax::error::ParseError;
use mago_span::Position;
use mago_span::Span;

fn parse<'arena>(arena: &'arena Bump, source: &'arena [u8]) -> Document<'arena> {
    PHPDocParser::parse(arena, FileId::zero(), source)
}

fn span(start: u32, end: u32) -> Span {
    Span::new(FileId::zero(), Position::new(start), Position::new(end))
}

fn invalid_tag_count(document: &Document<'_>) -> usize {
    document
        .elements
        .iter()
        .filter(|element| matches!(element, Element::Tag(tag) if matches!(tag.value, TagValue::Invalid(_))))
        .count()
}

#[test]
fn valid_docblock_has_no_errors() {
    let arena = Bump::new();
    let document = parse(&arena, b"/** @return int */");

    assert!(!document.has_errors());
    assert_eq!(document.errors.as_slice(), [].as_slice());
}

#[test]
fn malformed_tag_value_records_an_error_and_recovers() {
    let arena = Bump::new();
    let document = parse(&arena, b"/** @param | */");

    assert!(document.has_errors());
    assert_eq!(document.errors.as_slice(), [ParseError::UnexpectedToken(span(11, 12))].as_slice());

    assert_eq!(invalid_tag_count(&document), 1);
}

#[test]
fn every_malformed_tag_is_recorded_in_order() {
    let arena = Bump::new();
    let document = parse(&arena, b"/**\n * @param |\n * @return |\n */");

    assert_eq!(
        document.errors.as_slice(),
        [ParseError::UnexpectedToken(span(14, 15)), ParseError::UnexpectedToken(span(27, 28))].as_slice()
    );

    assert_eq!(invalid_tag_count(&document), 2);
}

#[test]
fn unexpected_end_of_input_is_recorded() {
    let arena = Bump::new();
    let document = parse(&arena, b"/** @template */");

    assert_eq!(document.errors.as_slice(), [ParseError::UnexpectedEndOfInput(span(13, 13))].as_slice());
    assert_eq!(invalid_tag_count(&document), 1);
}

#[test]
fn import_type_without_from_is_recorded() {
    let arena = Bump::new();
    let document = parse(&arena, b"/** @import-type Foo */");

    assert_eq!(document.errors.as_slice(), [ParseError::UnexpectedEndOfInput(span(20, 20))].as_slice());
    assert_eq!(invalid_tag_count(&document), 1);
}
