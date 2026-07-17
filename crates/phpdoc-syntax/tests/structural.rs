use mago_allocator::LocalArena;

use mago_database::file::FileId;
use mago_phpdoc_syntax::PHPDocParser;
use mago_phpdoc_syntax::cst::AssertPattern;
use mago_phpdoc_syntax::cst::AssertSubject;
use mago_phpdoc_syntax::cst::ConstantExpression;
use mago_phpdoc_syntax::cst::Document;
use mago_phpdoc_syntax::cst::Element;
use mago_phpdoc_syntax::cst::Tag;
use mago_phpdoc_syntax::cst::TagValue;
use mago_phpdoc_syntax::cst::TagVendor;
use mago_phpdoc_syntax::cst::Text;
use mago_phpdoc_syntax::cst::TextSegment;
use mago_phpdoc_syntax::cst::WhereTagValueModifier;
use mago_phpdoc_syntax::cst::r#type::Type;

fn parse<'arena>(arena: &'arena LocalArena, source: &'arena [u8]) -> Document<'arena> {
    PHPDocParser::parse(arena, FileId::zero(), source)
}

fn first_tag<'arena>(document: &'arena Document<'arena>) -> &'arena Tag<'arena> {
    for element in document.elements.iter() {
        if let Element::Tag(tag) = element {
            return tag;
        }
    }

    panic!("expected a tag element");
}

#[track_caller]
fn plain<'arena>(text: &Text<'arena>) -> &'arena [u8] {
    match text.segments {
        [] => &[],
        [TextSegment::PlainText(segment)] => segment.value,
        other => panic!("expected a single plain-text segment, got {other:?}"),
    }
}

#[test]
fn parses_param_with_type_and_variable() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @param string $bar */");
    let tag = first_tag(&document);

    assert_eq!(tag.name.value, b"param");
    assert_eq!(tag.vendor, None);

    let TagValue::Param(param) = &tag.value else { panic!("expected param, got {:?}", tag.value) };
    assert!(matches!(param.r#type, Type::String(_)));
    assert!(param.ampersand.is_none());
    assert!(param.ellipsis.is_none());
    assert_eq!(param.parameter.map(|parameter| parameter.value), Some(&b"$bar"[..]));
    assert!(param.description.is_none());
}

#[test]
fn parses_param_with_type_and_no_variable() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @param list<string> */");
    let tag = first_tag(&document);

    assert_eq!(tag.name.value, b"param");

    let TagValue::Param(param) = &tag.value else { panic!("expected param, got {:?}", tag.value) };
    assert!(matches!(param.r#type, Type::List(_)));
    assert!(param.parameter.is_none());
    assert!(param.description.is_none());
}

#[test]
fn parses_param_with_type_and_no_variable_but_description() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @param list<string> the items */");
    let tag = first_tag(&document);

    let TagValue::Param(param) = &tag.value else { panic!("expected param, got {:?}", tag.value) };
    assert!(matches!(param.r#type, Type::List(_)));
    assert!(param.parameter.is_none());
    let Some(description) = &param.description else { panic!("expected a description") };
    assert_eq!(plain(description), b"the items");
}

#[test]
fn parses_param_by_reference_variadic_with_description() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @param int &...$values the values */");
    let tag = first_tag(&document);

    let TagValue::Param(param) = &tag.value else { panic!() };
    assert!(param.ampersand.is_some());
    assert!(param.ellipsis.is_some());
    assert_eq!(param.parameter.map(|parameter| parameter.value), Some(&b"$values"[..]));
    let Some(description) = &param.description else { panic!() };
    assert_eq!(plain(description), b"the values");
}

#[test]
fn parses_typeless_param() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @param $foo description */");
    let tag = first_tag(&document);

    assert!(matches!(tag.value, TagValue::TypelessParam(_)));
}

#[test]
fn parses_parameter_dependent_param_type() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @param $prototype $value */");
    let tag = first_tag(&document);

    let TagValue::Param(param) = &tag.value else { panic!("expected typed param, got {:?}", tag.value) };
    let Type::Variable(parameter_type) = param.r#type else {
        panic!("expected parameter-dependent type, got {:?}", param.r#type);
    };

    assert_eq!(parameter_type.value, b"$prototype");
    assert_eq!(param.parameter.map(|parameter| parameter.value), Some(&b"$value"[..]));
    assert!(param.description.is_none());
}

#[test]
fn parses_indexed_parameter_dependent_param_type() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @param $objects[$key] $value */");
    let tag = first_tag(&document);

    let TagValue::Param(param) = &tag.value else { panic!("expected typed param, got {:?}", tag.value) };
    assert!(matches!(param.r#type, Type::IndexAccess(_)));
    assert_eq!(param.parameter.map(|parameter| parameter.value), Some(&b"$value"[..]));
    assert!(param.description.is_none());
}

#[test]
fn parses_return_with_union() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @return int|string the result */");
    let tag = first_tag(&document);

    let TagValue::Return(value) = &tag.value else { panic!() };
    assert!(matches!(value.r#type, Type::Union(_)));
    let Some(description) = &value.description else { panic!() };
    assert_eq!(plain(description), b"the result");
}

#[test]
fn parses_var_with_name() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @var array<int, string> $map */");
    let tag = first_tag(&document);

    let TagValue::Var(value) = &tag.value else { panic!() };
    assert!(matches!(value.r#type, Type::Array(_)));
    let Some(variable) = &value.variable else { panic!() };
    assert_eq!(variable.value, b"$map");
}

#[test]
fn parses_template_with_bound_and_default() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @template T of object = stdClass */");
    let tag = first_tag(&document);

    let TagValue::Template(template) = &tag.value else { panic!() };
    assert_eq!(template.name.value, b"T");
    let Some(bound) = &template.bound else { panic!("expected a bound") };
    assert_eq!(bound.keyword.value, b"of");
    assert!(matches!(bound.r#type, Type::Object(_)));
    let Some(default) = &template.default else { panic!("expected a default") };
    assert!(matches!(default.r#type, Type::Reference(_)));
}

#[test]
fn parses_method_with_static_return_templates_and_parameters() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @method static T find<T>(int $id, ?string $name = null) by id */");
    let tag = first_tag(&document);

    let TagValue::Method(method) = &tag.value else { panic!() };
    assert!(method.r#static.is_some());
    assert!(method.return_type.is_some());
    assert_eq!(method.name.value, b"find");
    assert!(method.templates.is_some());
    assert_eq!(method.parameters.entries.len(), 2);
}

#[test]
fn parses_method_with_static_as_return_type_only() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @method static create() */");
    let tag = first_tag(&document);

    let TagValue::Method(method) = &tag.value else { panic!() };
    assert!(method.r#static.is_none());
    assert!(method.return_type.is_some());
    assert_eq!(method.name.value, b"create");
    assert_eq!(method.parameters.entries.len(), 0);
}

#[test]
fn parses_method_without_return_type() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @method getName() */");
    let tag = first_tag(&document);

    let TagValue::Method(method) = &tag.value else { panic!() };
    assert!(method.return_type.is_none());
    assert_eq!(method.name.value, b"getName");
    assert_eq!(method.parameters.entries.len(), 0);
}

#[test]
fn parses_method_with_literal_parameter_type_and_grouped_default() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @method foo(int $a = 1, 'x'|'y' $b) */");
    let tag = first_tag(&document);

    let TagValue::Method(method) = &tag.value else { panic!("got {:?}", tag.value) };
    assert_eq!(method.name.value, b"foo");

    let entries: Vec<_> = method.parameters.entries.iter().collect();
    assert_eq!(entries.len(), 2);

    let Some(a_type) = entries[0].r#type else { panic!("expected a type") };
    assert!(matches!(a_type, Type::Int(_)));
    assert_eq!(entries[0].parameter.value, b"$a");
    let Some(a_default) = &entries[0].default else { panic!("expected a default") };
    assert!(matches!(a_default.value, ConstantExpression::Integer(_)));

    let Some(b_type) = entries[1].r#type else { panic!("expected a type") };
    assert!(matches!(b_type, Type::Union(_)));
    assert_eq!(entries[1].parameter.value, b"$b");
    assert!(entries[1].default.is_none());
}

#[test]
fn parses_assert_property() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @phpstan-assert !null $this->value */");
    let tag = first_tag(&document);

    assert_eq!(tag.vendor, Some(TagVendor::PhpStan));
    let TagValue::Assert(assert) = &tag.value else { panic!("got {:?}", tag.value) };
    assert!(assert.bang.is_some());
    let AssertSubject::Property { parameter, property, .. } = &assert.subject else {
        panic!("got {:?}", assert.subject)
    };
    assert_eq!(parameter.value, b"$this");
    assert_eq!(property.value, b"value");
}

#[test]
fn parses_assert_method() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @psalm-assert Foo $this->bar() */");
    let tag = first_tag(&document);

    assert_eq!(tag.vendor, Some(TagVendor::Psalm));
    let TagValue::Assert(assert) = &tag.value else { panic!() };
    let AssertSubject::Method { method, .. } = &assert.subject else { panic!("got {:?}", assert.subject) };
    assert_eq!(method.value, b"bar");
}

#[test]
fn parses_assert_if_true_with_keyword_pattern() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @assert-if-true truthy $foo */");
    let tag = first_tag(&document);

    let TagValue::AssertIfTrue(assert) = &tag.value else { panic!("got {:?}", tag.value) };
    assert!(matches!(assert.pattern, AssertPattern::Truthy(_)));
    let AssertSubject::Parameter { variable } = &assert.subject else { panic!("got {:?}", assert.subject) };
    assert_eq!(variable.value, b"$foo");
}

#[test]
fn parses_assert_keyword_named_type_when_not_followed_by_variable() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @assert truthy|string $foo */");
    let tag = first_tag(&document);

    let TagValue::Assert(assert) = &tag.value else { panic!("got {:?}", tag.value) };
    assert!(matches!(assert.pattern, AssertPattern::Type(Type::Union(_))));
}

#[test]
fn parses_extends_with_generics() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @extends Collection<int, User> */");
    let tag = first_tag(&document);

    let TagValue::Extends(extends) = &tag.value else { panic!() };
    assert!(matches!(extends.r#type, Type::Reference(_)));
}

#[test]
fn parses_type_alias() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @phpstan-type UserId = int */");
    let tag = first_tag(&document);

    assert_eq!(tag.vendor, Some(TagVendor::PhpStan));
    let TagValue::TypeAlias(alias) = &tag.value else { panic!() };
    assert_eq!(alias.alias.value, b"UserId");
    assert!(alias.equals.is_some());
    assert!(matches!(alias.r#type, Type::Int(_)));
}

#[test]
fn parses_type_alias_import() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @phpstan-import-type UserId from User as Id */");
    let tag = first_tag(&document);

    let TagValue::TypeAliasImport(import) = &tag.value else { panic!() };
    assert_eq!(import.imported_alias.value, b"UserId");
    assert_eq!(import.from_keyword.value, b"from");
    assert_eq!(import.imported_from.value, b"User");
    let Some(imported_as) = &import.imported_as else { panic!() };
    assert_eq!(imported_as.keyword.value, b"as");
    assert_eq!(imported_as.local.value, b"Id");
}

#[test]
fn parses_inheritors_as_union_type() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @inheritors A|B|C */");
    let tag = first_tag(&document);

    let TagValue::Inheritors(inheritors) = &tag.value else { panic!() };
    assert!(matches!(inheritors.r#type, Type::Union(_)));
}

#[test]
fn parses_inheritors_with_generic_parameters() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @inheritors None<T>|Some<T> */");
    let tag = first_tag(&document);

    let TagValue::Inheritors(inheritors) = &tag.value else { panic!() };
    let Type::Union(union) = inheritors.r#type else { panic!() };
    assert!(matches!(union.left, Type::Reference(_)));
    assert!(matches!(union.right, Type::Reference(_)));
}

#[test]
fn parses_property_with_type() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @property string $name */");
    let tag = first_tag(&document);

    let TagValue::Property(property) = &tag.value else { panic!() };
    assert!(property.r#type.is_some());
    assert_eq!(property.variable.value, b"$name");
}

#[test]
fn parses_property_without_type() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @property $name */");
    let tag = first_tag(&document);

    let TagValue::Property(property) = &tag.value else { panic!() };
    assert!(property.r#type.is_none());
    assert_eq!(property.variable.value, b"$name");
}

#[test]
fn parses_phan_vendor() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @phan-param int $x */");
    let tag = first_tag(&document);

    assert_eq!(tag.vendor, Some(TagVendor::Phan));
    assert!(matches!(tag.value, TagValue::Param(_)));
}

#[test]
fn parses_unknown_tag_as_generic() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @author Jane Doe <jane@example.com> */");
    let tag = first_tag(&document);

    assert_eq!(tag.name.value, b"author");
    assert!(matches!(tag.value, TagValue::Generic(_)));
}

#[test]
fn parses_deprecated_description_only() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @deprecated use bar() instead */");
    let tag = first_tag(&document);

    let TagValue::Deprecated(deprecated) = &tag.value else { panic!() };
    assert_eq!(plain(&deprecated.description), b"use bar() instead");
}

#[test]
fn parses_multiline_docblock_with_multiple_tags() {
    let arena = LocalArena::new();
    let source = b"/**\n * Summary line.\n *\n * @param int $a first\n * @param string $b second\n * @return bool\n */";
    let document = parse(&arena, source);

    let elements: Vec<&Element> = document.elements.iter().collect();
    assert_eq!(elements.len(), 4);

    let Element::Text(summary) = elements[0] else { panic!("expected text, got {:?}", elements[0]) };
    assert_eq!(plain(summary), b"Summary line.");

    let Element::Tag(a) = elements[1] else { panic!("expected tag, got {:?}", elements[1]) };
    let TagValue::Param(a) = &a.value else { panic!("expected param, got {:?}", a.value) };
    assert!(matches!(a.r#type, Type::Int(_)));
    assert_eq!(a.parameter.map(|parameter| parameter.value), Some(&b"$a"[..]));
    let Some(a_description) = a.description else { panic!("expected description") };
    assert_eq!(plain(&a_description), b"first");

    let Element::Tag(b) = elements[2] else { panic!("expected tag, got {:?}", elements[2]) };
    let TagValue::Param(b) = &b.value else { panic!("expected param, got {:?}", b.value) };
    assert!(matches!(b.r#type, Type::String(_)));
    assert_eq!(b.parameter.map(|parameter| parameter.value), Some(&b"$b"[..]));
    let Some(b_description) = b.description else { panic!("expected description") };
    assert_eq!(plain(&b_description), b"second");

    let Element::Tag(ret) = elements[3] else { panic!("expected tag, got {:?}", elements[3]) };
    let TagValue::Return(ret) = &ret.value else { panic!("expected return, got {:?}", ret.value) };
    assert!(matches!(ret.r#type, Type::Bool(_)));
    assert!(ret.description.is_none());
}

#[test]
fn parses_where_tag_with_is_modifier() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @where T is int */");
    let tag = first_tag(&document);

    let TagValue::Where(where_clause) = &tag.value else { panic!("got {:?}", tag.value) };
    assert_eq!(where_clause.name.value, b"T");
    let WhereTagValueModifier::Is(keyword) = &where_clause.modifier else { panic!("expected is modifier") };
    assert_eq!(keyword.value, b"is");
    assert!(matches!(where_clause.r#type, Type::Int(_)));
}

#[test]
fn parses_where_tag_with_colon_modifier() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @where T: string */");
    let tag = first_tag(&document);

    let TagValue::Where(where_clause) = &tag.value else { panic!("got {:?}", tag.value) };
    assert_eq!(where_clause.name.value, b"T");
    assert!(matches!(where_clause.modifier, WhereTagValueModifier::Colon(_)));
    assert!(matches!(where_clause.r#type, Type::String(_)));
}

#[test]
fn recovers_invalid_tag_value() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @param | */");
    let tag = first_tag(&document);

    assert!(matches!(tag.value, TagValue::Invalid(_)));
}

#[test]
fn parses_method_named_is_with_variadic_parameter() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @method static bool is(mixed ...$patterns) */");
    let tag = first_tag(&document);

    assert!(!document.has_errors());
    let TagValue::Method(method) = &tag.value else { panic!("got {:?}", tag.value) };
    assert!(method.r#static.is_some());
    assert_eq!(method.name.value, b"is");

    let parameters: Vec<_> = method.parameters.entries.iter().collect();
    assert_eq!(parameters.len(), 1);
    assert!(parameters[0].ellipsis.is_some());
    assert_eq!(parameters[0].parameter.value, b"$patterns");
}

#[test]
fn parses_bare_int_mask_as_int() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @param int-mask $flags */");
    let tag = first_tag(&document);

    assert!(!document.has_errors());
    let TagValue::Param(param) = &tag.value else { panic!("got {:?}", tag.value) };
    assert!(matches!(param.r#type, Type::Int(_)));
    assert_eq!(param.parameter.map(|parameter| parameter.value), Some(&b"$flags"[..]));
}

#[test]
fn parses_var_with_description_starting_with_is() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @var mixed Is null for add operations */");
    let tag = first_tag(&document);

    assert!(!document.has_errors());
    let TagValue::Var(var) = &tag.value else { panic!("got {:?}", tag.value) };
    assert!(matches!(var.r#type, Type::Mixed(_)));
    assert!(var.variable.is_none());
    let Some(description) = &var.description else { panic!("expected a description") };
    assert_eq!(plain(description), b"Is null for add operations");
}

#[test]
fn return_description_starting_with_bracket_is_not_index_access() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** @return array{int<0, max>, int<0, max>} [bytes_a_to_b, bytes_b_to_a] */");

    assert!(!document.has_errors(), "expected no errors, got {:?}", document.errors);

    let tag = first_tag(&document);
    let TagValue::Return(return_tag) = &tag.value else { panic!("got {:?}", tag.value) };
    assert!(matches!(return_tag.r#type, Type::Shape(_)));

    let Some(description) = &return_tag.description else { panic!("expected a description") };
    assert_eq!(plain(description), b"[bytes_a_to_b, bytes_b_to_a]");
}

#[test]
fn adjacent_bracket_still_parses_as_slice_and_index_access() {
    let arena = LocalArena::new();

    let document = parse(&arena, b"/** @return string[] */");
    let TagValue::Return(return_tag) = &first_tag(&document).value else { panic!() };
    assert!(matches!(return_tag.r#type, Type::Slice(_)));

    let document = parse(&arena, b"/** @return Foo[Bar] */");
    let TagValue::Return(return_tag) = &first_tag(&document).value else { panic!() };
    assert!(matches!(return_tag.r#type, Type::IndexAccess(_)));
}

#[test]
fn escaped_backtick_does_not_open_inline_code() {
    let arena = LocalArena::new();
    let document = parse(
        &arena,
        b"/**\n * Processes escape sequences:\n * - Punctuation: \\\\, \\?, \\\", \\', \\`\n * - Whitespace: \\a, \\b\n */",
    );

    assert!(!document.has_errors(), "expected no errors, got {:?}", document.errors);

    let Some(Element::Text(text)) = document.elements.first() else {
        panic!("expected a text element");
    };
    assert!(
        text.segments.iter().all(|segment| matches!(segment, TextSegment::PlainText(_))),
        "expected plain text only, got {:?}",
        text.segments
    );
}

#[test]
fn backtick_after_whitespace_still_opens_inline_code() {
    let arena = LocalArena::new();
    let document = parse(&arena, b"/** some `code` here */");

    assert!(!document.has_errors(), "expected no errors, got {:?}", document.errors);

    let Some(Element::Text(text)) = document.elements.first() else {
        panic!("expected a text element");
    };
    assert!(
        text.segments.iter().any(|segment| matches!(segment, TextSegment::InlineCode(_))),
        "expected an inline code segment, got {:?}",
        text.segments
    );
}
