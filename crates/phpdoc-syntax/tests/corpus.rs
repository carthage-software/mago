use bumpalo::Bump;

use mago_database::file::FileId;
use mago_phpdoc_syntax::PHPDocParser;
use mago_phpdoc_syntax::cst::Document;
use mago_phpdoc_syntax::cst::Element;
use mago_phpdoc_syntax::cst::Tag;
use mago_phpdoc_syntax::cst::TagValue;
use mago_phpdoc_syntax::cst::Text;
use mago_phpdoc_syntax::cst::r#type::ReferenceKind;
use mago_phpdoc_syntax::cst::r#type::Type;

fn parse<'arena>(arena: &'arena Bump, source: &'arena [u8]) -> Document<'arena> {
    PHPDocParser::parse(arena, FileId::zero(), source)
}

fn tags<'arena>(document: &'arena Document<'arena>) -> Vec<&'arena Tag<'arena>> {
    document
        .elements
        .iter()
        .filter_map(|element| if let Element::Tag(tag) = element { Some(*tag) } else { None })
        .collect()
}

fn texts<'arena>(document: &'arena Document<'arena>) -> Vec<&'arena Text<'arena>> {
    document
        .elements
        .iter()
        .filter_map(|element| if let Element::Text(text) = element { Some(text) } else { None })
        .collect()
}

#[test]
fn parses_description_text_and_typed_tags() {
    let arena = Bump::new();
    let source = b"/**\n * Summary.\n *\n * @param string $foo\n * @return void\n */";
    let document = parse(&arena, source);

    let elements: Vec<&Element> = document.elements.iter().collect();
    assert_eq!(elements.len(), 3);

    let Element::Text(summary) = elements[0] else { panic!("expected text, got {:?}", elements[0]) };
    assert_eq!(summary.value, b"Summary.");

    let Element::Tag(foo) = elements[1] else { panic!("expected tag, got {:?}", elements[1]) };
    assert_eq!(foo.name.value, b"param");
    let TagValue::Param(foo) = &foo.value else { panic!("expected param, got {:?}", foo.value) };
    assert!(matches!(foo.r#type, Type::String(_)));
    assert_eq!(foo.parameter.value, b"$foo");
    assert!(foo.description.is_none());

    let Element::Tag(ret) = elements[2] else { panic!("expected tag, got {:?}", elements[2]) };
    assert_eq!(ret.name.value, b"return");
    let TagValue::Return(ret) = &ret.value else { panic!("expected return, got {:?}", ret.value) };
    assert!(matches!(ret.r#type, Type::Void(_)));
    assert!(ret.description.is_none());
}

#[test]
fn underscore_in_tag_name_is_preserved() {
    let arena = Bump::new();
    let document = parse(&arena, b"/** @some_tag Description */");
    let tags = tags(&document);

    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name.value, b"some_tag");

    let TagValue::Generic(generic) = &tags[0].value else { panic!("expected generic, got {:?}", tags[0].value) };
    assert_eq!(generic.value.value, b"Description");
}

#[test]
fn utf8_variable_name_and_ascii_description() {
    let arena = Bump::new();
    let source = "/** @param string $مثال A parameter with an Arabic variable name. */".as_bytes();
    let document = parse(&arena, source);
    let tags = tags(&document);

    assert_eq!(tags.len(), 1);
    let TagValue::Param(param) = &tags[0].value else { panic!("expected param, got {:?}", tags[0].value) };
    assert!(matches!(param.r#type, Type::String(_)));
    assert_eq!(param.parameter.value, "$مثال".as_bytes());
    let Some(description) = param.description else { panic!("expected a description") };
    assert_eq!(description.value, b"A parameter with an Arabic variable name.");
}

#[test]
fn utf8_return_description_is_kept_verbatim() {
    let arena = Bump::new();
    let source = "/** @return int 返回值是整数类型。 */".as_bytes();
    let document = parse(&arena, source);
    let tags = tags(&document);

    assert_eq!(tags.len(), 1);
    let TagValue::Return(ret) = &tags[0].value else { panic!("expected return, got {:?}", tags[0].value) };
    assert!(matches!(ret.r#type, Type::Int(_)));
    let Some(description) = ret.description else { panic!("expected a description") };
    assert_eq!(description.value, "返回值是整数类型。".as_bytes());
}

#[test]
fn ideographic_space_is_part_of_identifiers() {
    let arena = Bump::new();
    let source = "/** @param\u{3000}string $foo 中文描述 */".as_bytes();
    let document = parse(&arena, source);
    let tags = tags(&document);

    assert_eq!(tags.len(), 1);
    let TagValue::Param(param) = &tags[0].value else { panic!("expected param, got {:?}", tags[0].value) };
    let Type::Reference(reference) = param.r#type else { panic!("expected reference, got {:?}", param.r#type) };
    let ReferenceKind::Identifier(identifier) = reference.kind else {
        panic!("expected identifier, got {:?}", reference.kind)
    };
    assert_eq!(identifier.value, "\u{3000}string".as_bytes());
    assert_eq!(param.parameter.value, b"$foo");
    let Some(description) = param.description else { panic!("expected a description") };
    assert_eq!(description.value, "中文描述".as_bytes());
}

#[test]
fn ideographic_space_return_and_throws() {
    let arena = Bump::new();
    let source = "/**\n * @return\u{3000}int\n * @throws\u{3000}Exception\n */".as_bytes();
    let document = parse(&arena, source);
    let tags = tags(&document);
    assert_eq!(tags.len(), 2);

    let TagValue::Return(ret) = &tags[0].value else { panic!("expected return, got {:?}", tags[0].value) };
    let Type::Reference(ret_type) = ret.r#type else { panic!("expected reference, got {:?}", ret.r#type) };
    let ReferenceKind::Identifier(ret_identifier) = ret_type.kind else {
        panic!("expected identifier, got {:?}", ret_type.kind)
    };
    assert_eq!(ret_identifier.value, "\u{3000}int".as_bytes());

    let TagValue::Throws(throws) = &tags[1].value else { panic!("expected throws, got {:?}", tags[1].value) };
    let Type::Reference(throws_type) = throws.r#type else { panic!("expected reference, got {:?}", throws.r#type) };
    let ReferenceKind::Identifier(throws_identifier) = throws_type.kind else {
        panic!("expected identifier, got {:?}", throws_type.kind)
    };
    assert_eq!(throws_identifier.value, "\u{3000}Exception".as_bytes());
}

#[test]
fn multiline_description_is_kept_verbatim() {
    let arena = Bump::new();
    let source = b"/** @var string[] line one\nline two*/";
    let document = parse(&arena, source);
    let tags = tags(&document);

    assert_eq!(tags.len(), 1);
    let TagValue::Var(var) = &tags[0].value else { panic!("expected var, got {:?}", tags[0].value) };
    let Type::Slice(slice) = var.r#type else { panic!("expected slice, got {:?}", var.r#type) };
    assert!(matches!(slice.inner, Type::String(_)));
    assert!(var.variable.is_none());

    let Some(description) = var.description else { panic!("expected a description") };
    assert_eq!(description.value, b"line one\nline two");
}

#[test]
fn doctrine_annotations_become_generic_tags() {
    let arena = Bump::new();
    let source = b"/**\n * @Event(\"X\")\n * @SimpleAnnotation\n */";
    let document = parse(&arena, source);
    let tags = tags(&document);
    assert_eq!(tags.len(), 2);

    assert_eq!(tags[0].name.value, b"Event");
    let TagValue::Generic(event) = &tags[0].value else { panic!("expected generic, got {:?}", tags[0].value) };
    assert_eq!(event.value.value, b"(\"X\")");

    assert_eq!(tags[1].name.value, b"SimpleAnnotation");
    let TagValue::Generic(simple) = &tags[1].value else { panic!("expected generic, got {:?}", tags[1].value) };
    assert_eq!(simple.value.value, b"");
}

#[test]
fn inline_tags_are_kept_as_raw_text() {
    let arena = Bump::new();
    let document = parse(&arena, br#"/** See {@see \Some\Class} for details. */"#);
    let texts = texts(&document);

    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0].value, br#"See {@see \Some\Class} for details."#);
}

#[test]
fn code_blocks_are_kept_as_raw_text() {
    let arena = Bump::new();
    let document = parse(&arena, b"/**\n * ```php\n * echo 1;\n * ```\n */");
    let texts = texts(&document);

    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0].value, b"```php\n * echo 1;\n * ```");
}

#[test]
fn utf8_paragraph_text_is_kept_verbatim() {
    let arena = Bump::new();
    let source = "/**\n * 中文段落\n */".as_bytes();
    let document = parse(&arena, source);
    let texts = texts(&document);

    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0].value, "中文段落".as_bytes());
}
