use bumpalo::Bump;

use mago_database::file::FileId;
use mago_phpdoc_syntax::PHPDocParser;
use mago_phpdoc_syntax::cst::Document;
use mago_phpdoc_syntax::cst::Element;
use mago_phpdoc_syntax::cst::TagValue;
use mago_phpdoc_syntax::cst::TagVendor;

fn parse<'arena>(arena: &'arena Bump, source: &'arena [u8]) -> Document<'arena> {
    PHPDocParser::parse(arena, FileId::zero(), source)
}

fn inherit_spans(document: &Document<'_>) -> Vec<(u32, u32)> {
    document.inherit_docs.iter().map(|inherit| (inherit.span.start.offset, inherit.span.end.offset)).collect()
}

#[test]
fn standalone_inheritdoc_tag_is_recognized_and_recorded() {
    let arena = Bump::new();
    let document = parse(&arena, b"/** @inheritdoc */");

    assert!(document.has_inherit_doc());
    assert_eq!(inherit_spans(&document), vec![(4, 15)]);

    let elements: Vec<&Element> = document.elements.iter().collect();
    assert_eq!(elements.len(), 1);
    let Element::Tag(tag) = elements[0] else { panic!("expected tag, got {:?}", elements[0]) };
    assert!(matches!(tag.value, TagValue::InheritDoc(_)));
}

#[test]
fn inheritdoc_tag_name_is_case_insensitive() {
    let arena = Bump::new();
    let document = parse(&arena, b"/** @inheritDoc */");

    assert!(document.has_inherit_doc());
    assert_eq!(inherit_spans(&document), vec![(4, 15)]);

    let tag = match document.elements.iter().next() {
        Some(Element::Tag(tag)) => tag,
        other => panic!("expected tag, got {other:?}"),
    };
    assert!(matches!(tag.value, TagValue::InheritDoc(_)));
}

#[test]
fn vendor_prefixed_inheritdoc_is_recognized() {
    let arena = Bump::new();
    let document = parse(&arena, b"/** @psalm-inheritdoc */");

    assert_eq!(inherit_spans(&document), vec![(4, 21)]);

    let tag = match document.elements.iter().next() {
        Some(Element::Tag(tag)) => tag,
        other => panic!("expected tag, got {other:?}"),
    };
    assert_eq!(tag.vendor, Some(TagVendor::Psalm));
    assert!(matches!(tag.value, TagValue::InheritDoc(_)));
}

#[test]
fn inline_inheritdoc_is_recorded_and_text_kept_raw() {
    let arena = Bump::new();
    let document = parse(&arena, b"/** Well, {@inheritDoc} i guess */");

    assert!(document.has_inherit_doc());
    assert_eq!(inherit_spans(&document), vec![(10, 23)]);

    let elements: Vec<&Element> = document.elements.iter().collect();
    assert_eq!(elements.len(), 1);
    let Element::Text(text) = elements[0] else { panic!("expected text, got {:?}", elements[0]) };
    assert_eq!(text.value, b"Well, {@inheritDoc} i guess");
}

#[test]
fn both_inline_and_standalone_are_recorded_in_order() {
    let arena = Bump::new();
    let document = parse(&arena, b"/**\n * {@inheritDoc} text\n * @inheritdoc\n */");

    assert!(document.has_inherit_doc());
    assert_eq!(inherit_spans(&document), vec![(7, 20), (29, 40)]);
}

#[test]
fn no_inheritdoc_means_empty() {
    let arena = Bump::new();
    let document = parse(&arena, b"/** @return int */");

    assert!(!document.has_inherit_doc());
    assert_eq!(inherit_spans(&document), Vec::new());
}
