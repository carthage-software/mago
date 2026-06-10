#![cfg_attr(doc, doc = include_str!("./../README.md"))]
#![allow(clippy::pub_use)]
#![allow(clippy::exhaustive_enums)]

use mago_allocator::Arena;

use mago_span::Position;
use mago_span::Span;
use mago_syntax_core::input::Input;

use crate::ast::Type;
use crate::error::ParseError;
use crate::lexer::TypeLexer;

pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod token;

/// Parses a string representation of a PHPDoc type into an arena-allocated
/// Abstract Syntax Tree.
///
/// All AST nodes are allocated in the caller-supplied arena, so
/// no per-node heap allocation happens during parsing. The resulting
/// [`Type`] borrows from `input` (for textual slices) and from `arena`
/// (for nested sub-trees) for the duration of `'arena`.
///
/// # Arguments
///
/// * `arena` - The arena that will own every AST node.
/// * `span` - The original `Span` of the `input` string slice within its
///   source file; used to anchor every produced span.
/// * `input` - The type string to parse (e.g. `"int|string"`,
///   `"array<int, MyClass>"`).
///
/// # Errors
///
/// Returns a [`ParseError`] if any lexing or parsing error occurs.
pub fn parse_str<'arena, A>(arena: &'arena A, span: Span, input: &'arena [u8]) -> Result<Type<'arena>, ParseError>
where
    A: Arena,
{
    let input = Input::anchored_at(span.file_id, input, span.start);
    let lexer = TypeLexer::new(input);
    parser::construct(arena, lexer)
}

/// Parses the **longest valid type prefix** of `input` and reports the
/// absolute position just past the consumed bytes.
///
/// Unlike [`parse_str`], this does not require the entire input to be a
/// single type. It is the handoff point for embedding callers (e.g. the
/// phpdoc-syntax parser): they parse one type, fast-forward their own
/// scanner to the returned position, and keep going with their own
/// tokens from there.
///
/// # Arguments
///
/// * `arena` - The arena that will own every AST node.
/// * `span` - The absolute span covering `input` within its source file.
/// * `input` - The slice to parse; only the prefix that forms a complete
///   type expression is consumed.
///
/// # Errors
///
/// Returns a [`ParseError`] if the prefix does not start with a valid
/// type.
pub fn parse_prefix<'arena, A>(
    arena: &'arena A,
    span: Span,
    input: &'arena [u8],
) -> Result<(Type<'arena>, Position), ParseError>
where
    A: Arena,
{
    let input_obj = Input::anchored_at(span.file_id, input, span.start);
    let lexer = TypeLexer::new(input_obj);
    parser::construct_prefix(arena, lexer)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use mago_allocator::LocalArena;

    use mago_database::file::FileId;
    use mago_span::HasSpan;
    use mago_span::Position;
    use mago_span::Span;

    use crate::ast::*;

    use super::*;

    /// Test helper: parses `input` against a fresh leaked arena so the
    /// resulting `Type<'static>` can be freely inspected without plumbing
    /// lifetimes through every test. The arena is intentionally leaked;
    /// tests run once and exit, so the cost is bounded and the ergonomics
    /// match the pre-arena API.
    fn do_parse(input: &str) -> Result<Type<'static>, ParseError> {
        let arena: &'static LocalArena = Box::leak(Box::new(LocalArena::new()));
        let owned: &'static [u8] = arena.alloc_slice_copy(input.as_bytes());
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(owned.len() as u32));
        parse_str(arena, span, owned)
    }

    #[test]
    fn test_parse_simple_keyword() {
        let result = do_parse("int");
        assert!(result.is_ok());
        match result.unwrap() {
            Type::Int(k) => assert_eq!(k.value, b"int".as_slice()),
            _ => panic!("Expected Type::Int"),
        }
    }

    #[test]
    fn test_parse_composite_keyword() {
        let result = do_parse("non-empty-string");
        assert!(result.is_ok());
        match result.unwrap() {
            Type::NonEmptyString(k) => assert_eq!(k.value, b"non-empty-string".as_slice()),
            _ => panic!("Expected Type::NonEmptyString"),
        }
    }

    #[test]
    fn test_parse_empty_keywords() {
        match do_parse("empty") {
            Ok(Type::Empty(k)) => assert_eq!(k.value, b"empty".as_slice()),
            other => panic!("Expected Type::Empty, got: {other:?}"),
        }

        match do_parse("empty-scalar") {
            Ok(Type::EmptyScalar(k)) => assert_eq!(k.value, b"empty-scalar".as_slice()),
            other => panic!("Expected Type::EmptyScalar, got: {other:?}"),
        }

        match do_parse("empty|object") {
            Ok(Type::Union(u)) => {
                assert!(matches!(u.left, Type::Empty(_)));
                assert!(matches!(u.right, Type::Object(_)));
            }
            other => panic!("Expected Type::Union, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_literal_ints() {
        let assert_parsed_literal_int = |input: &str, expected_value: u64| {
            let result = do_parse(input);
            assert!(result.is_ok());
            match result.unwrap() {
                Type::LiteralInt(LiteralIntType { value, .. }) => assert_eq!(
                    value, expected_value,
                    "Expected value to be {expected_value} for input {input}, but got {value}"
                ),
                _ => panic!("Expected Type::LiteralInt"),
            }
        };

        assert_parsed_literal_int("0", 0);
        assert_parsed_literal_int("1", 1);
        assert_parsed_literal_int("123_345", 123_345);
        assert_parsed_literal_int("0b1", 1);
        assert_parsed_literal_int("0o10", 8);
        assert_parsed_literal_int("0x1", 1);
        assert_parsed_literal_int("0x10", 16);
        assert_parsed_literal_int("0xFF", 255);
    }

    #[test]
    fn test_parse_literal_floats() {
        let assert_parsed_literal_float = |input: &str, expected_value: f64| {
            let result = do_parse(input);
            assert!(result.is_ok());
            match result.unwrap() {
                Type::LiteralFloat(LiteralFloatType { value, .. }) => assert_eq!(
                    value, expected_value,
                    "Expected value to be {expected_value} for input {input}, but got {value}"
                ),
                _ => panic!("Expected Type::LiteralInt"),
            }
        };

        assert_parsed_literal_float("0.0", 0.0);
        assert_parsed_literal_float("1.0", 1.0);
        assert_parsed_literal_float("0.1e1", 1.0);
        assert_parsed_literal_float("0.1e-1", 0.01);
        assert_parsed_literal_float("0.1E1", 1.0);
        assert_parsed_literal_float("0.1E-1", 0.01);
        assert_parsed_literal_float("0.1e+1", 1.0);
        assert_parsed_literal_float(".1e+1", 1.0);
    }

    #[test]
    fn test_float_with_dangling_exponent_does_not_panic() {
        match do_parse("3.") {
            Ok(Type::LiteralFloat(LiteralFloatType { value, raw, .. })) => {
                assert_eq!(*value, 3.0);
                assert_eq!(raw, b"3.".as_slice());
            }
            other => panic!("expected `3.` to parse as LiteralFloat 3.0, got: {other:?}"),
        }

        let _ = do_parse("3.eint");
        let _ = do_parse("3.e");

        match do_parse(".1") {
            Ok(Type::LiteralFloat(LiteralFloatType { value, raw, .. })) => {
                assert_eq!(*value, 0.1);
                assert_eq!(raw, b".1".as_slice());
            }
            other => panic!("expected `.1` to parse as LiteralFloat 0.1, got: {other:?}"),
        }

        let _ = do_parse(".1E");
        let _ = do_parse(".1e");
        let _ = do_parse(".1e+");
        let _ = do_parse(".1E.111.12E1ra");
    }

    #[test]
    fn test_deeply_nested_type_does_not_overflow() {
        std::thread::Builder::new()
            .stack_size(128 * 1024 * 1024)
            .spawn(|| {
                let input = "(".repeat(5000);
                assert!(
                    matches!(do_parse(&input), Err(ParseError::RecursionLimitExceeded(_))),
                    "expected RecursionLimitExceeded for deeply nested parentheses"
                );

                let _ = do_parse("44[899[inT is(((((((((((((((((((((((((((((((((");
            })
            .expect("spawn parser thread")
            .join()
            .expect("parser thread must not abort (no stack overflow)");
    }

    #[test]
    fn test_parse_simple_union() {
        match do_parse("int|string") {
            Ok(ty) => match ty {
                Type::Union(u) => {
                    assert!(matches!(u.left, Type::Int(_)));
                    assert!(matches!(u.right, Type::String(_)));
                }
                _ => panic!("Expected Type::Union"),
            },
            Err(err) => {
                panic!("Failed to parse union type: {err:?}");
            }
        }
    }

    #[test]
    fn test_parse_variable_union() {
        match do_parse("$a|$b") {
            Ok(ty) => match ty {
                Type::Union(u) => {
                    assert!(matches!(u.left, Type::Variable(_)));
                    assert!(matches!(u.right, Type::Variable(_)));
                }
                _ => panic!("Expected Type::Union"),
            },
            Err(err) => {
                panic!("Failed to parse union type: {err:?}");
            }
        }
    }

    #[test]
    fn test_parse_nullable() {
        let result = do_parse("?string");
        assert!(result.is_ok());
        match result.unwrap() {
            Type::Nullable(n) => {
                assert!(matches!(n.inner, Type::String(_)));
            }
            _ => panic!("Expected Type::Nullable"),
        }
    }

    #[test]
    fn test_parse_generic_array() {
        let result = do_parse("array<int, bool>");
        assert!(result.is_ok());
        match result.unwrap() {
            Type::Array(a) => {
                assert!(a.parameters.is_some());
                let params = a.parameters.unwrap();
                assert_eq!(params.entries.len(), 2);
                assert!(matches!(params.entries[0].inner, Type::Int(_)));
                assert!(matches!(params.entries[1].inner, Type::Bool(_)));
            }
            _ => panic!("Expected Type::Array"),
        }
    }

    #[test]
    fn test_parse_generic_array_one_param() {
        match do_parse("array<string>") {
            Ok(Type::Array(a)) => {
                let params = a.parameters.expect("Expected generic parameters");
                assert_eq!(params.entries.len(), 1);
                assert!(matches!(params.entries[0].inner, Type::String(_)));
            }
            res => panic!("Expected Ok(Type::Array), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_generic_list() {
        match do_parse("list<string>") {
            Ok(Type::List(l)) => {
                let params = l.parameters.expect("Expected generic parameters");
                assert_eq!(params.entries.len(), 1);
                assert!(matches!(params.entries[0].inner, Type::String(_)));
            }
            res => panic!("Expected Ok(Type::List), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_non_empty_array() {
        match do_parse("non-empty-array<int, bool>") {
            Ok(Type::NonEmptyArray(a)) => {
                let params = a.parameters.expect("Expected generic parameters");
                assert_eq!(params.entries.len(), 2);
                assert!(matches!(params.entries[0].inner, Type::Int(_)));
                assert!(matches!(params.entries[1].inner, Type::Bool(_)));
            }
            res => panic!("Expected Ok(Type::NonEmptyArray), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_nested_generics() {
        match do_parse("list<array<int, string>>") {
            Ok(Type::List(l)) => {
                let params = l.parameters.expect("Expected generic parameters");
                assert_eq!(params.entries.len(), 1);
                match &params.entries[0].inner {
                    Type::Array(inner_array) => {
                        let inner_params = inner_array.parameters.as_ref().expect("Inner array needs params");
                        assert_eq!(inner_params.entries.len(), 2);
                        assert!(matches!(inner_params.entries[0].inner, Type::Int(_)));
                        assert!(matches!(inner_params.entries[1].inner, Type::String(_)));
                    }
                    _ => panic!("Expected inner type to be Type::Array"),
                }
            }
            res => panic!("Expected Ok(Type::List), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_simple_shape() {
        let result = do_parse("array{'name': string}");
        assert!(matches!(result, Ok(Type::Shape(_))));
        let Ok(Type::Shape(shape)) = result else {
            panic!("Expected Type::Shape");
        };

        assert_eq!(shape.kind, ShapeTypeKind::Array);
        assert_eq!(shape.keyword.value, b"array".as_slice());
        assert_eq!(shape.fields.len(), 1);
        assert!(shape.additional_fields.is_none());

        let field = &shape.fields[0];
        assert!(matches!(field.key.as_ref().map(|k| &k.key), Some(ShapeKey::String { value: b"name", .. })));
        assert!(matches!(field.value, Type::String(_)));
    }

    #[test]
    fn test_parse_int_key_shape() {
        match do_parse("array{0: string, 1: bool}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 2);
                let first_field = &shape.fields[0];
                assert!(matches!(first_field.key.as_ref().map(|k| &k.key), Some(ShapeKey::Integer { value: 0, .. })));
                assert!(matches!(first_field.value, Type::String(_)));
                let second_field = &shape.fields[1];
                assert!(matches!(second_field.key.as_ref().map(|k| &k.key), Some(ShapeKey::Integer { value: 1, .. })));
                assert!(matches!(second_field.value, Type::Bool(_)));
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_optional_field_shape() {
        match do_parse("array{name: string, age?: int, address: string}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 3);
                assert!(!shape.fields[0].is_optional());
                assert!(shape.fields[1].is_optional());
                assert!(!shape.fields[2].is_optional());
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_unsealed_shape() {
        match do_parse("array{name: string, ...}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 1);
                assert!(shape.additional_fields.is_some());
                assert!(shape.additional_fields.unwrap().parameters.is_none()); // No fallback specified
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_shape_with_keys_containing_special_chars() {
        match do_parse("array{key-with-dash: int, key-with---multiple-dashes?: int}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 2);

                if let Some(ShapeKey::String { value: s, .. }) = shape.fields[0].key.as_ref().map(|k| &k.key) {
                    assert_eq!(*s, b"key-with-dash".as_slice());
                } else {
                    panic!("Expected key to be a ShapeKey::String");
                }

                if let Some(ShapeKey::String { value: s, .. }) = shape.fields[1].key.as_ref().map(|k| &k.key) {
                    assert_eq!(*s, b"key-with---multiple-dashes".as_slice());
                } else {
                    panic!("Expected key to be a ShapeKey::String");
                }
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_shape_with_keys_after_types() {
        match do_parse("array{list: list<int>, int?: int, string: string, bool: bool}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 4);

                if let Some(ShapeKey::String { value: s, .. }) = shape.fields[0].key.as_ref().map(|k| &k.key) {
                    assert_eq!(*s, b"list".as_slice());
                } else {
                    panic!("Expected key to be a ShapeKey::String");
                }

                if let Some(ShapeKey::String { value: s, .. }) = shape.fields[1].key.as_ref().map(|k| &k.key) {
                    assert_eq!(*s, b"int".as_slice());
                } else {
                    panic!("Expected key to be a ShapeKey::String");
                }

                if let Some(ShapeKey::String { value: s, .. }) = shape.fields[2].key.as_ref().map(|k| &k.key) {
                    assert_eq!(*s, b"string".as_slice());
                } else {
                    panic!("Expected key to be a ShapeKey::String");
                }

                if let Some(ShapeKey::String { value: s, .. }) = shape.fields[3].key.as_ref().map(|k| &k.key) {
                    assert_eq!(*s, b"bool".as_slice());
                } else {
                    panic!("Expected key to be a ShapeKey::String");
                }
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_shape_keyless_entry_with_commas_inside_generics() {
        // Regression: the shape-field-key scan used to bail on any top-level
        // comma without tracking bracket depth. A `,` inside `<...>` must
        // be skipped over, not mistaken for the field terminator.
        match do_parse("array{array<int, string>}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 1);
                assert!(shape.fields[0].key.is_none(), "expected a keyless (positional) field");
                match shape.fields[0].value {
                    Type::Array(_) => {}
                    v => panic!("expected value to be a generic array type, got {v:?}"),
                }
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_shape_keyed_entry_with_commas_inside_value_generics() {
        // `foo: array<int, string>` must be recognized as a keyed field.
        // The scan has to see the `:` at top level despite the `,` nested
        // inside `<...>` in the value.
        match do_parse("array{foo: array<int, string>}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 1);
                let key = shape.fields[0].key.as_ref().expect("expected a keyed field");
                match &key.key {
                    ShapeKey::String { value, .. } => assert_eq!(*value, b"foo".as_slice()),
                    other => panic!("expected identifier key, got {other:?}"),
                }
                match shape.fields[0].value {
                    Type::Array(_) => {}
                    v => panic!("expected value to be a generic array type, got {v:?}"),
                }
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_shape_with_large_union_value_does_not_overflow() {
        // Regression: a single keyless field whose value is a long union
        // containing nested generics used to scan past the stream's
        // lookahead capacity (16 slots previously, now 64) looking for a
        // phantom `:`. With bracket-depth tracking and the
        // SHAPE_KEY_SCAN_LIMIT cap the scan stays bounded.
        let input = "array{\
            int | string | float | bool | null | \
            array<int, string> | array<string, int> | \
            callable(int, string): bool | \
            list<int> | iterable<string, mixed>\
        }";
        match do_parse(input) {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 1, "expected a single keyless field");
                assert!(shape.fields[0].key.is_none(), "value is a union, not a keyed field");
                match shape.fields[0].value {
                    Type::Union(_) => {}
                    v => panic!("expected a union value type, got {v:?}"),
                }
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_shape_many_fields_with_nested_generics() {
        // Stress test: many fields, each with a value containing top-level
        // commas inside `<>`. Previously the scan would overflow the fixed
        // lookahead buffer on some fields because it couldn't distinguish
        // `,` inside a generic from the field separator.
        let input = "array{\
            a: list<int, string>, \
            b: array<int, string>, \
            c: iterable<int, string>, \
            d: callable(int, string): void, \
            e: array<string, array<int, string>>, \
            f: string\
        }";
        match do_parse(input) {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 6);
                for (i, expected_key) in [b"a".as_slice(), b"b", b"c", b"d", b"e", b"f"].iter().enumerate() {
                    let key = shape.fields[i].key.as_ref().expect("expected a keyed field");
                    match &key.key {
                        ShapeKey::String { value, .. } => assert_eq!(value, expected_key),
                        other => panic!("field {i}: expected identifier key, got {other:?}"),
                    }
                }
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_unsealed_shape_with_fallback() {
        match do_parse(
            "array{
                name: string, // This is a comment
                ...<string, string>
            }",
        ) {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 1);
                assert!(shape.additional_fields.as_ref().is_some_and(|a| a.parameters.is_some()));
                let params = shape.additional_fields.unwrap().parameters.unwrap();
                assert_eq!(params.entries.len(), 2);
                assert!(matches!(params.entries[0].inner, Type::String(_)));
                assert!(matches!(params.entries[1].inner, Type::String(_)));
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_empty_shape() {
        match do_parse("array{}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 0);
                assert!(shape.additional_fields.is_none());
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_nested_spread_singleline() {
        // Test nested spreads on single line - this should work
        match do_parse("array{a?: int, ...<string, array{b?: int, ...<string, int>}>}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 1);
                assert!(shape.additional_fields.is_some());
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_nested_spread_multiline() {
        match do_parse(
            "array{
                a?: int,
                ...<string, array{
                    b?: int,
                    ...<string, int>,
                }>
            }",
        ) {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 1);
                assert!(shape.additional_fields.is_some());
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_spread_with_trailing_comma() {
        match do_parse("array{a?: int, ...<string, int>,}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 1);
                assert!(shape.additional_fields.is_some());
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_error_unexpected_token() {
        let result = do_parse("int|>");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParseError::UnexpectedToken { .. }));
    }

    #[test]
    fn test_parse_error_eof() {
        let result = do_parse("array<int");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParseError::UnexpectedEndOfFile { .. }));
    }

    #[test]
    fn test_parse_error_trailing_token() {
        let result = do_parse("int|string&");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParseError::UnexpectedEndOfFile { .. }));
    }

    #[test]
    fn test_parse_intersection() {
        match do_parse("Countable&Traversable") {
            Ok(Type::Intersection(i)) => {
                assert!(matches!(i.left, Type::Reference(_)));
                assert!(matches!(i.right, Type::Reference(_)));

                if let Type::Reference(r) = i.left {
                    assert_eq!(r.identifier.value, b"Countable".as_slice());
                } else {
                    panic!();
                }

                if let Type::Reference(r) = i.right {
                    assert_eq!(r.identifier.value, b"Traversable".as_slice());
                } else {
                    panic!();
                }
            }
            res => panic!("Expected Ok(Type::Intersection), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_member_ref() {
        match do_parse("MyClass::MY_CONST") {
            Ok(Type::MemberReference(m)) => {
                assert_eq!(m.class.value, b"MyClass".as_slice());
                assert_eq!(m.member.to_string(), "MY_CONST");
            }
            res => panic!("Expected Ok(Type::MemberReference), got {res:?}"),
        }

        match do_parse("\\Fully\\Qualified::class") {
            Ok(Type::MemberReference(m)) => {
                assert_eq!(m.class.value, b"\\Fully\\Qualified".as_slice()); // Check if lexer keeps leading \
                assert_eq!(m.member.to_string(), "class");
            }
            res => panic!("Expected Ok(Type::MemberReference), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_member_ref_named_new() {
        match do_parse("Action::NEW") {
            Ok(Type::MemberReference(m)) => {
                assert_eq!(m.class.value, b"Action".as_slice());
                assert_eq!(m.member.to_string(), "NEW");
            }
            res => panic!("Expected Ok(Type::MemberReference) for Action::NEW, got {res:?}"),
        }

        match do_parse("Action::new") {
            Ok(Type::MemberReference(m)) => {
                assert_eq!(m.class.value, b"Action".as_slice());
                assert_eq!(m.member.to_string(), "new");
            }
            res => panic!("Expected Ok(Type::MemberReference) for Action::new, got {res:?}"),
        }

        match do_parse("Action::DELETE|Action::NEW") {
            Ok(Type::Union(u)) => match (&u.left, &u.right) {
                (Type::MemberReference(lhs), Type::MemberReference(rhs)) => {
                    assert_eq!(lhs.member.to_string(), "DELETE");
                    assert_eq!(rhs.member.to_string(), "NEW");
                }
                other => panic!("Expected two member references, got {other:?}"),
            },
            res => panic!("Expected Ok(Type::Union), got {res:?}"),
        }

        match do_parse("\\App\\Action::NEW") {
            Ok(Type::MemberReference(m)) => {
                assert_eq!(m.member.to_string(), "NEW");
            }
            res => panic!("Expected Ok(Type::MemberReference), got {res:?}"),
        }

        match do_parse("App\\Action::NEW") {
            Ok(Type::MemberReference(m)) => {
                assert_eq!(m.member.to_string(), "NEW");
            }
            res => panic!("Expected Ok(Type::MemberReference), got {res:?}"),
        }

        match do_parse("Action::new*") {
            Ok(Type::MemberReference(m)) => {
                assert_eq!(m.class.value, b"Action".as_slice());
                assert!(matches!(m.member, MemberReferenceSelector::StartsWith(..)));
            }
            res => panic!("Expected Ok(Type::MemberReference) for Action::new*, got {res:?}"),
        }

        match do_parse("Action::*new") {
            Ok(Type::MemberReference(m)) => {
                assert_eq!(m.class.value, b"Action".as_slice());
                assert!(matches!(m.member, MemberReferenceSelector::EndsWith(..)));
            }
            res => panic!("Expected Ok(Type::MemberReference) for Action::*new, got {res:?}"),
        }

        match do_parse("new<Foo>") {
            Ok(Type::New(_)) => {}
            res => panic!("Expected Ok(Type::New), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_new_in_other_identifier_contexts() {
        match do_parse("array{new: int}") {
            Ok(Type::Shape(_)) => {}
            res => panic!("Expected Ok(Type::Shape) for array{{new: int}}, got {res:?}"),
        }

        match do_parse("array{new?: int}") {
            Ok(Type::Shape(_)) => {}
            res => panic!("Expected Ok(Type::Shape) for array{{new?: int}}, got {res:?}"),
        }

        match do_parse("array{Foo::NEW: int}") {
            Ok(Type::Shape(_)) => {}
            res => panic!("Expected Ok(Type::Shape) for array{{Foo::NEW: int}}, got {res:?}"),
        }

        match do_parse("object{new: int}") {
            Ok(Type::Object(_)) => {}
            res => panic!("Expected Ok(Type::Object) for object{{new: int}}, got {res:?}"),
        }

        match do_parse("!Foo::new") {
            Ok(Type::AliasReference(_)) => {}
            res => panic!("Expected Ok(Type::AliasReference) for !Foo::new, got {res:?}"),
        }
    }

    #[test]
    fn test_parse_iterable() {
        match do_parse("iterable<int, string>") {
            Ok(Type::Iterable(i)) => {
                let params = i.parameters.expect("Expected generic parameters");
                assert_eq!(params.entries.len(), 2);
                assert!(matches!(params.entries[0].inner, Type::Int(_)));
                assert!(matches!(params.entries[1].inner, Type::String(_)));
            }
            res => panic!("Expected Ok(Type::Iterable), got {res:?}"),
        }

        match do_parse("iterable<bool>") {
            // Test single param case
            Ok(Type::Iterable(i)) => {
                let params = i.parameters.expect("Expected generic parameters");
                assert_eq!(params.entries.len(), 1);
                assert!(matches!(params.entries[0].inner, Type::Bool(_)));
            }
            res => panic!("Expected Ok(Type::Iterable), got {res:?}"),
        }

        match do_parse("iterable") {
            Ok(Type::Iterable(i)) => {
                assert!(i.parameters.is_none());
            }
            res => panic!("Expected Ok(Type::Iterable), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_negated_int() {
        let assert_negated_int = |input: &str, expected_value: u64| {
            let result = do_parse(input);
            assert!(result.is_ok());
            match result.unwrap() {
                Type::Negated(n) => {
                    assert!(matches!(n.number, LiteralIntOrFloatType::Int(_)));
                    if let LiteralIntOrFloatType::Int(lit) = n.number {
                        assert_eq!(lit.value, expected_value);
                    } else {
                        panic!()
                    }
                }
                _ => panic!("Expected Type::Negated"),
            }
        };

        assert_negated_int("-0", 0);
        assert_negated_int("-1", 1);
        assert_negated_int(
            "-
            // This is a comment
            123_345",
            123_345,
        );
        assert_negated_int("-0b1", 1);
    }

    #[test]
    fn test_parse_negated_float() {
        let assert_negated_float = |input: &str, expected_value: f64| {
            let result = do_parse(input);
            assert!(result.is_ok());
            match result.unwrap() {
                Type::Negated(n) => {
                    assert!(matches!(n.number, LiteralIntOrFloatType::Float(_)));
                    if let LiteralIntOrFloatType::Float(lit) = n.number {
                        assert_eq!(lit.value, expected_value);
                    } else {
                        panic!()
                    }
                }
                _ => panic!("Expected Type::Negated"),
            }
        };

        assert_negated_float("-0.0", 0.0);
        assert_negated_float("-1.0", 1.0);
        assert_negated_float("-0.1e1", 1.0);
        assert_negated_float("-0.1e-1", 0.01);
    }

    #[test]
    fn test_parse_negated_union() {
        match do_parse("-1|-2.0|string") {
            Ok(Type::Union(n)) => {
                assert!(matches!(n.left, Type::Negated(_)));
                assert!(matches!(n.right, Type::Union(_)));

                if let Type::Negated(neg) = n.left {
                    assert!(matches!(neg.number, LiteralIntOrFloatType::Int(_)));
                    if let LiteralIntOrFloatType::Int(lit) = neg.number {
                        assert_eq!(lit.value, 1);
                    } else {
                        panic!()
                    }
                } else {
                    panic!("Expected left side to be Type::Negated");
                }

                if let Type::Union(inner_union) = n.right {
                    assert!(matches!(inner_union.left, Type::Negated(_)));
                    assert!(matches!(inner_union.right, Type::String(_)));

                    if let Type::Negated(neg) = inner_union.left {
                        assert!(matches!(neg.number, LiteralIntOrFloatType::Float(_)));
                        if let LiteralIntOrFloatType::Float(lit) = neg.number {
                            assert_eq!(lit.value, 2.0);
                        } else {
                            panic!()
                        }
                    } else {
                        panic!("Expected left side of inner union to be Type::Negated");
                    }

                    if let Type::String(s) = inner_union.right {
                        assert_eq!(s.value, b"string".as_slice());
                    } else {
                        panic!("Expected right side of inner union to be Type::String");
                    }
                } else {
                    panic!("Expected right side to be Type::Union");
                }
            }
            res => panic!("Expected Ok(Type::Negated), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_callable_no_spec() {
        match do_parse("callable") {
            Ok(Type::Callable(c)) => {
                assert!(c.specification.is_none());
                assert_eq!(c.kind, CallableTypeKind::Callable);
            }
            res => panic!("Expected Ok(Type::Callable), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_callable_params_only() {
        match do_parse("callable(int, ?string)") {
            Ok(Type::Callable(c)) => {
                let spec = c.specification.expect("Expected callable specification");
                assert!(spec.return_type.is_none());
                assert_eq!(spec.parameters.entries.len(), 2);
                assert!(matches!(spec.parameters.entries[0].parameter_type, Some(Type::Int(_))));
                assert!(matches!(spec.parameters.entries[1].parameter_type, Some(Type::Nullable(_))));
                assert!(spec.parameters.entries[0].ellipsis.is_none());
                assert!(spec.parameters.entries[0].equals.is_none());
            }
            res => panic!("Expected Ok(Type::Callable), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_callable_return_only() {
        match do_parse("callable(): void") {
            Ok(Type::Callable(c)) => {
                let spec = c.specification.expect("Expected callable specification");
                assert!(spec.parameters.entries.is_empty());
                assert!(spec.return_type.is_some());
                assert!(matches!(spec.return_type.unwrap().return_type, Type::Void(_)));
            }
            res => panic!("Expected Ok(Type::Callable), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_pure_callable_full() {
        match do_parse("pure-callable(bool): int") {
            Ok(Type::Callable(c)) => {
                assert_eq!(c.kind, CallableTypeKind::PureCallable);
                let spec = c.specification.expect("Expected callable specification");
                assert_eq!(spec.parameters.entries.len(), 1);
                assert!(matches!(spec.parameters.entries[0].parameter_type, Some(Type::Bool(_))));
                assert!(spec.return_type.is_some());
                assert!(matches!(spec.return_type.unwrap().return_type, Type::Int(_)));
            }
            res => panic!("Expected Ok(Type::Callable), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_closure_via_identifier() {
        match do_parse("Closure(string): bool") {
            Ok(Type::Callable(c)) => {
                assert_eq!(c.kind, CallableTypeKind::Closure);
                assert_eq!(c.keyword.value, b"Closure".as_slice());
                let spec = c.specification.expect("Expected callable specification");
                assert_eq!(spec.parameters.entries.len(), 1);
                assert!(matches!(spec.parameters.entries[0].parameter_type, Some(Type::String(_))));
                assert!(spec.return_type.is_some());
                assert!(matches!(spec.return_type.unwrap().return_type, Type::Bool(_)));
            }
            res => panic!("Expected Ok(Type::Callable) for Closure, got {res:?}"),
        }
    }

    #[test]
    fn test_parse_complex_pure_callable() {
        match do_parse("pure-callable(list<int>, ?Closure(): void=, int...): ((Simple&Iter<T>)|null)") {
            Ok(Type::Callable(c)) => {
                assert_eq!(c.kind, CallableTypeKind::PureCallable);
                let spec = c.specification.expect("Expected callable specification");
                assert_eq!(spec.parameters.entries.len(), 3);
                assert!(spec.return_type.is_some());

                let first_param = &spec.parameters.entries[0];
                assert!(matches!(first_param.parameter_type, Some(Type::List(_))));
                assert!(first_param.ellipsis.is_none());
                assert!(first_param.equals.is_none());

                let second_param = &spec.parameters.entries[1];
                assert!(matches!(second_param.parameter_type, Some(Type::Nullable(_))));
                assert!(second_param.ellipsis.is_none());
                assert!(second_param.equals.is_some());

                let third_param = &spec.parameters.entries[2];
                assert!(matches!(third_param.parameter_type, Some(Type::Int(_))));
                assert!(third_param.ellipsis.is_some());
                assert!(third_param.equals.is_none());

                if let Type::Parenthesized(p) = spec.return_type.unwrap().return_type {
                    assert!(matches!(p.inner, Type::Union(_)));
                    if let Type::Union(u) = p.inner {
                        assert!(matches!(u.left, Type::Parenthesized(_)));
                        assert!(matches!(u.right, Type::Null(_)));
                    }
                } else {
                    panic!("Expected Type::CallableReturnType");
                }
            }
            res => panic!("Expected Ok(Type::Callable), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_callable_by_reference_parameter() {
        match do_parse("callable(bool &$ret): void") {
            Ok(Type::Callable(c)) => {
                let spec = c.specification.expect("Expected callable specification");
                assert_eq!(spec.parameters.entries.len(), 1);

                let param = &spec.parameters.entries[0];
                assert!(matches!(param.parameter_type, Some(Type::Bool(_))));
                assert!(param.is_by_reference(), "expected `&` to be parsed as by-reference marker");
                assert!(!param.is_variadic());
                assert!(!param.is_optional());
                assert!(param.variable.is_some());
            }
            res => panic!("Expected Ok(Type::Callable), got {res:?}"),
        }

        match do_parse("callable(string &...$rest): int") {
            Ok(Type::Callable(c)) => {
                let spec = c.specification.expect("Expected callable specification");
                assert_eq!(spec.parameters.entries.len(), 1);

                let param = &spec.parameters.entries[0];
                assert!(matches!(param.parameter_type, Some(Type::String(_))));
                assert!(param.is_by_reference());
                assert!(param.is_variadic());
            }
            res => panic!("Expected Ok(Type::Callable), got {res:?}"),
        }

        match do_parse("callable(Foo&Bar $x): void") {
            Ok(Type::Callable(c)) => {
                let spec = c.specification.expect("Expected callable specification");
                assert_eq!(spec.parameters.entries.len(), 1);

                let param = &spec.parameters.entries[0];
                assert!(matches!(param.parameter_type, Some(Type::Intersection(_))));
                assert!(!param.is_by_reference());
                assert!(param.variable.is_some());
            }
            res => panic!("Expected Ok(Type::Callable), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_conditional_type() {
        match do_parse("int is not string ? array : int") {
            Ok(Type::Conditional(c)) => {
                assert!(matches!(c.subject, Type::Int(_)));
                assert!(c.not.is_some());
                assert!(matches!(c.target, Type::String(_)));
                assert!(matches!(c.then, Type::Array(_)));
                assert!(matches!(c.otherwise, Type::Int(_)));
            }
            res => panic!("Expected Ok(Type::Conditional), got {res:?}"),
        }

        match do_parse("$input is string ? array : int") {
            Ok(Type::Conditional(c)) => {
                assert!(matches!(c.subject, Type::Variable(_)));
                assert!(c.not.is_none());
                assert!(matches!(c.target, Type::String(_)));
                assert!(matches!(c.then, Type::Array(_)));
                assert!(matches!(c.otherwise, Type::Int(_)));
            }
            res => panic!("Expected Ok(Type::Conditional), got {res:?}"),
        }

        match do_parse("int is string ? array : (int is not $bar ? string : $baz)") {
            Ok(Type::Conditional(c)) => {
                assert!(matches!(c.subject, Type::Int(_)));
                assert!(c.not.is_none());
                assert!(matches!(c.target, Type::String(_)));
                assert!(matches!(c.then, Type::Array(_)));

                let Type::Parenthesized(p) = c.otherwise else {
                    panic!("Expected Type::Parenthesized");
                };

                if let Type::Conditional(inner_conditional) = p.inner {
                    assert!(matches!(inner_conditional.subject, Type::Int(_)));
                    assert!(inner_conditional.not.is_some());
                    assert!(matches!(inner_conditional.target, Type::Variable(_)));
                    assert!(matches!(inner_conditional.then, Type::String(_)));
                    assert!(matches!(inner_conditional.otherwise, Type::Variable(_)));
                } else {
                    panic!("Expected Type::Conditional");
                }
            }
            res => panic!("Expected Ok(Type::Conditional), got {res:?}"),
        }
    }

    #[test]
    fn test_keyof() {
        match do_parse("key-of<MyArray>") {
            Ok(Type::KeyOf(k)) => {
                assert_eq!(k.keyword.value, b"key-of".as_slice());
                match &k.parameter.entry.inner {
                    Type::Reference(r) => assert_eq!(r.identifier.value, b"MyArray".as_slice()),
                    _ => panic!("Expected Type::Reference"),
                }
            }
            res => panic!("Expected Ok(Type::KeyOf), got {res:?}"),
        }
    }

    #[test]
    fn test_valueof() {
        match do_parse("value-of<MyArray>") {
            Ok(Type::ValueOf(v)) => {
                assert_eq!(v.keyword.value, b"value-of".as_slice());
                match &v.parameter.entry.inner {
                    Type::Reference(r) => assert_eq!(r.identifier.value, b"MyArray".as_slice()),
                    _ => panic!("Expected Type::Reference"),
                }
            }
            res => panic!("Expected Ok(Type::ValueOf), got {res:?}"),
        }
    }

    #[test]
    fn test_indexed_access() {
        match do_parse("MyArray[MyKey]") {
            Ok(Type::IndexAccess(i)) => {
                match i.target {
                    Type::Reference(r) => assert_eq!(r.identifier.value, b"MyArray".as_slice()),
                    _ => panic!("Expected Type::Reference"),
                }
                match i.index {
                    Type::Reference(r) => assert_eq!(r.identifier.value, b"MyKey".as_slice()),
                    _ => panic!("Expected Type::Reference"),
                }
            }
            res => panic!("Expected Ok(Type::IndexAccess), got {res:?}"),
        }
    }

    #[test]
    fn test_slice_type() {
        match do_parse("string[]") {
            Ok(Type::Slice(s)) => {
                assert!(matches!(s.inner, Type::String(_)));
            }
            res => panic!("Expected Ok(Type::Slice), got {res:?}"),
        }
    }

    #[test]
    fn test_slice_of_slice_of_slice_type() {
        match do_parse("string[][][]") {
            Ok(Type::Slice(s)) => {
                assert!(matches!(s.inner, Type::Slice(_)));
                if let Type::Slice(inner_slice) = s.inner {
                    assert!(matches!(inner_slice.inner, Type::Slice(_)));
                    if let Type::Slice(inner_inner_slice) = inner_slice.inner {
                        assert!(matches!(inner_inner_slice.inner, Type::String(_)));
                    } else {
                        panic!("Expected inner slice to be a Slice");
                    }
                } else {
                    panic!("Expected outer slice to be a Slice");
                }
            }
            res => panic!("Expected Ok(Type::Slice), got {res:?}"),
        }
    }

    #[test]
    fn test_int_range() {
        match do_parse("int<0, 100>") {
            Ok(Type::IntRange(r)) => {
                assert_eq!(r.keyword.value, b"int".as_slice());

                match r.min {
                    IntOrKeyword::Int(literal_int_type) => {
                        assert_eq!(literal_int_type.value, 0);
                    }
                    _ => {
                        panic!("Expected min to be a LiteralIntType, got `{}`", r.min)
                    }
                }

                match r.max {
                    IntOrKeyword::Int(literal_int_type) => {
                        assert_eq!(literal_int_type.value, 100);
                    }
                    _ => {
                        panic!("Expected max to be a LiteralIntType, got `{}`", r.max)
                    }
                }
            }
            res => panic!("Expected Ok(Type::IntRange), got {res:?}"),
        }

        match do_parse("int<min, 0>") {
            Ok(Type::IntRange(r)) => {
                match r.min {
                    IntOrKeyword::Keyword(keyword) => {
                        assert_eq!(keyword.value, b"min".as_slice());
                    }
                    _ => {
                        panic!("Expected min to be a Keyword, got `{}`", r.min)
                    }
                }

                match r.max {
                    IntOrKeyword::Int(literal_int_type) => {
                        assert_eq!(literal_int_type.value, 0);
                    }
                    _ => {
                        panic!("Expected max to be a LiteralIntType, got `{}`", r.max)
                    }
                }
            }
            res => panic!("Expected Ok(Type::IntRange), got {res:?}"),
        }

        match do_parse("int<min, max>") {
            Ok(Type::IntRange(r)) => {
                match r.min {
                    IntOrKeyword::Keyword(keyword) => {
                        assert_eq!(keyword.value, b"min".as_slice());
                    }
                    _ => {
                        panic!("Expected min to be a Keyword, got `{}`", r.min)
                    }
                }

                match r.max {
                    IntOrKeyword::Keyword(keyword) => {
                        assert_eq!(keyword.value, b"max".as_slice());
                    }
                    _ => {
                        panic!("Expected max to be a Keyword, got `{}`", r.max)
                    }
                }
            }
            res => panic!("Expected Ok(Type::IntRange), got {res:?}"),
        }
    }

    #[test]
    fn test_properties_of() {
        match do_parse("properties-of<MyClass>") {
            Ok(Type::PropertiesOf(p)) => {
                assert_eq!(p.keyword.value, b"properties-of".as_slice());
                assert_eq!(p.filter, PropertiesOfFilter::All);
                match &p.parameter.entry.inner {
                    Type::Reference(r) => assert_eq!(r.identifier.value, b"MyClass".as_slice()),
                    _ => panic!(),
                }
            }
            res => panic!("Expected Ok(Type::PropertiesOf), got {res:?}"),
        }

        match do_parse("protected-properties-of<T>") {
            Ok(Type::PropertiesOf(p)) => {
                assert_eq!(p.keyword.value, b"protected-properties-of".as_slice());
                assert_eq!(p.filter, PropertiesOfFilter::Protected);
                match &p.parameter.entry.inner {
                    Type::Reference(r) => assert_eq!(r.identifier.value, b"T".as_slice()),
                    _ => panic!(),
                }
            }
            res => panic!("Expected Ok(Type::PropertiesOf), got {res:?}"),
        }

        match do_parse("private-properties-of<T>") {
            Ok(Type::PropertiesOf(p)) => {
                assert_eq!(p.keyword.value, b"private-properties-of".as_slice());
                assert_eq!(p.filter, PropertiesOfFilter::Private);
                match &p.parameter.entry.inner {
                    Type::Reference(r) => assert_eq!(r.identifier.value, b"T".as_slice()),
                    _ => panic!(),
                }
            }
            res => panic!("Expected Ok(Type::PropertiesOf), got {res:?}"),
        }

        match do_parse("public-properties-of<T>") {
            Ok(Type::PropertiesOf(p)) => {
                assert_eq!(p.keyword.value, b"public-properties-of".as_slice());
                assert_eq!(p.filter, PropertiesOfFilter::Public);
                match &p.parameter.entry.inner {
                    Type::Reference(r) => assert_eq!(r.identifier.value, b"T".as_slice()),
                    _ => panic!(),
                }
            }
            res => panic!("Expected Ok(Type::PropertiesOf), got {res:?}"),
        }
    }

    #[test]
    fn test_variable() {
        match do_parse("$myVar") {
            Ok(Type::Variable(v)) => {
                assert_eq!(v.value, b"$myVar".as_slice());
            }
            res => panic!("Expected Ok(Type::Variable), got {res:?}"),
        }
    }

    #[test]
    fn test_nullable_intersection() {
        // Nullable applies only to the rightmost element of an intersection before parens
        match do_parse("Countable&?Traversable") {
            Ok(Type::Intersection(i)) => {
                assert!(matches!(i.left, Type::Reference(r) if r.identifier.value == b"Countable".as_slice()));
                assert!(matches!(i.right, Type::Nullable(_)));
                if let Type::Nullable(n) = i.right {
                    assert!(matches!(n.inner, Type::Reference(r) if r.identifier.value == b"Traversable".as_slice()));
                } else {
                    panic!();
                }
            }
            res => panic!("Expected Ok(Type::Intersection), got {res:?}"),
        }
    }

    #[test]
    fn test_parenthesized_nullable() {
        match do_parse("?(Countable&Traversable)") {
            Ok(Type::Nullable(n)) => {
                assert!(matches!(n.inner, Type::Parenthesized(_)));
                if let Type::Parenthesized(p) = n.inner {
                    assert!(matches!(p.inner, Type::Intersection(_)));
                } else {
                    panic!()
                }
            }
            res => panic!("Expected Ok(Type::Nullable), got {res:?}"),
        }
    }

    #[test]
    fn test_positive_negative_int() {
        match do_parse("positive-int|negative-int") {
            Ok(Type::Union(u)) => {
                assert!(matches!(u.left, Type::PositiveInt(_)));
                assert!(matches!(u.right, Type::NegativeInt(_)));
            }
            res => panic!("Expected Ok(Type::Union), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_float_alias() {
        match do_parse("double") {
            Ok(Type::Float(f)) => {
                assert_eq!(f.value, b"double".as_slice());
            }
            res => panic!("Expected Ok(Type::Float), got {res:?}"),
        }

        match do_parse("real") {
            Ok(Type::Float(f)) => {
                assert_eq!(f.value, b"real".as_slice());
            }
            res => panic!("Expected Ok(Type::Float), got {res:?}"),
        }

        match do_parse("float") {
            Ok(Type::Float(f)) => {
                assert_eq!(f.value, b"float".as_slice());
            }
            res => panic!("Expected Ok(Type::Float), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_bool_alias() {
        match do_parse("boolean") {
            Ok(Type::Bool(b)) => {
                assert_eq!(b.value, b"boolean".as_slice());
            }
            res => panic!("Expected Ok(Type::Bool), got {res:?}"),
        }

        match do_parse("bool") {
            Ok(Type::Bool(b)) => {
                assert_eq!(b.value, b"bool".as_slice());
            }
            res => panic!("Expected Ok(Type::Bool), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_integer_alias() {
        match do_parse("integer") {
            Ok(Type::Int(i)) => {
                assert_eq!(i.value, b"integer".as_slice());
            }
            res => panic!("Expected Ok(Type::Int), got {res:?}"),
        }

        match do_parse("int") {
            Ok(Type::Int(i)) => {
                assert_eq!(i.value, b"int".as_slice());
            }
            res => panic!("Expected Ok(Type::Int), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_callable_with_variables() {
        match do_parse("callable(string ...$names)") {
            Ok(Type::Callable(callable)) => {
                assert_eq!(callable.keyword.value, b"callable".as_slice());
                assert!(callable.specification.is_some());

                let specification = callable.specification.unwrap();

                assert!(specification.return_type.is_none());
                assert_eq!(specification.parameters.entries.len(), 1);

                let first_parameter = specification.parameters.entries.first().unwrap();
                assert!(first_parameter.variable.is_some());
                assert!(first_parameter.ellipsis.is_some());

                let variable = first_parameter.variable.unwrap();
                assert_eq!(variable.value, b"$names".as_slice());
            }
            res => panic!("Expected Ok(Type::Callable), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_string_or_lowercase_string_union() {
        match do_parse("string|lowercase-string") {
            Ok(Type::Union(u)) => {
                assert!(matches!(u.left, Type::String(_)));
                assert!(matches!(u.right, Type::LowercaseString(_)));
            }
            res => panic!("Expected Ok(Type::Union), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_optional_literal_string_shape_field() {
        match do_parse("array{'salt'?: int, 'cost'?: int, ...}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 2);
                assert!(shape.additional_fields.is_some());

                let first_field = &shape.fields[0];
                assert!(first_field.is_optional());
                assert!(matches!(
                    first_field.key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"salt", .. })
                ));
                assert!(matches!(first_field.value, Type::Int(_)));

                let second_field = &shape.fields[1];
                assert!(second_field.is_optional());
                assert!(matches!(
                    second_field.key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"cost", .. })
                ));
                assert!(matches!(second_field.value, Type::Int(_)));
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_keyword_keys() {
        match do_parse("array{string: int, bool: string, int: float, mixed: object}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 4);

                assert!(matches!(
                    shape.fields[0].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"string", .. })
                ));
                assert!(matches!(
                    shape.fields[1].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"bool", .. })
                ));
                assert!(matches!(
                    shape.fields[2].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"int", .. })
                ));
                assert!(matches!(
                    shape.fields[3].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"mixed", .. })
                ));
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_negated_integer_keys() {
        match do_parse("array{-1: string, -42: int, +5: bool}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 3);

                assert!(matches!(
                    shape.fields[0].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::Integer { value: -1, .. })
                ));
                assert!(matches!(
                    shape.fields[1].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::Integer { value: -42, .. })
                ));
                assert!(matches!(
                    shape.fields[2].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::Integer { value: 5, .. })
                ));
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_float_keys() {
        match do_parse("array{123.4: string, -1.2: int, +0.5: bool}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 3);

                assert!(matches!(
                    shape.fields[0].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"123.4", .. })
                ));
                assert!(matches!(
                    shape.fields[1].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"-1.2", .. })
                ));
                assert!(matches!(
                    shape.fields[2].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"+0.5", .. })
                ));
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_complex_identifier_keys() {
        match do_parse(
            "array{key_with_underscore: int, key-with-dash: string, key\\with\\backslash: bool, +key: mixed, -key: object, \\leading_backslash: int}",
        ) {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 6);

                assert!(matches!(
                    shape.fields[0].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"key_with_underscore", .. })
                ));
                assert!(matches!(
                    shape.fields[1].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"key-with-dash", .. })
                ));
                assert!(matches!(
                    shape.fields[2].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"key\\with\\backslash", .. })
                ));
                assert!(matches!(
                    shape.fields[3].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"+key", .. })
                ));
                assert!(matches!(
                    shape.fields[4].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"-key", .. })
                ));
                assert!(matches!(
                    shape.fields[5].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"\\leading_backslash", .. })
                ));
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_optional_keys_with_question_mark_in_name() {
        match do_parse("array{key?name: int, regular?: string}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 2);

                assert!(!shape.fields[0].is_optional());
                assert!(matches!(
                    shape.fields[0].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"key?name", .. })
                ));

                assert!(shape.fields[1].is_optional());
                assert!(matches!(
                    shape.fields[1].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"regular", .. })
                ));
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_integer_formats() {
        match do_parse("array{42: string, 0x2A: int, 0b101010: bool, 0o52: mixed}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 4);

                assert!(matches!(
                    shape.fields[0].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::Integer { value: 42, .. })
                ));
                assert!(matches!(
                    shape.fields[1].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::Integer { value: 42, .. })
                ));
                assert!(matches!(
                    shape.fields[2].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::Integer { value: 42, .. })
                ));
                assert!(matches!(
                    shape.fields[3].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::Integer { value: 42, .. })
                ));
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_quoted_vs_unquoted_keys() {
        match do_parse("array{'string': int, \"double\": bool, unquoted: mixed}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 3);

                assert!(matches!(
                    shape.fields[0].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"string", .. })
                ));
                assert!(matches!(
                    shape.fields[1].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"double", .. })
                ));
                assert!(matches!(
                    shape.fields[2].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"unquoted", .. })
                ));
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_all_keyword_types() {
        let keywords = vec![
            "list", "int", "integer", "string", "float", "double", "real", "bool", "boolean", "false", "true",
            "object", "callable", "array", "iterable", "null", "mixed", "resource", "void", "scalar", "numeric",
            "never", "nothing", "as", "is", "not", "min", "max",
        ];

        for keyword in keywords {
            let input = format!("array{{{keyword}: string}}");
            match do_parse(&input) {
                Ok(Type::Shape(shape)) => {
                    assert_eq!(shape.fields.len(), 1);
                    assert!(
                        matches!(
                            shape.fields[0].key.as_ref().map(|k| &k.key),
                            Some(ShapeKey::String { value, .. }) if *value == keyword.as_bytes()
                        ),
                        "Failed for keyword: {keyword}"
                    );
                }
                res => panic!("Expected Ok(Type::Shape) for keyword '{keyword}', got {res:?}"),
            }
        }
    }

    #[test]
    fn test_parse_php_specific_keywords() {
        match do_parse("array{self: string, static: int, parent: bool, class: mixed, __CLASS__: object}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 5);

                assert!(matches!(
                    shape.fields[0].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"self", .. })
                ));
                assert!(matches!(
                    shape.fields[1].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"static", .. })
                ));
                assert!(matches!(
                    shape.fields[2].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"parent", .. })
                ));
                assert!(matches!(
                    shape.fields[3].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"class", .. })
                ));
                assert!(matches!(
                    shape.fields[4].key.as_ref().map(|k| &k.key),
                    Some(ShapeKey::String { value: b"__CLASS__", .. })
                ));
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_shape_key_spans() {
        match do_parse("array{test: string}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 1);
                let field = &shape.fields[0];

                if let Some(key) = &field.key {
                    let span = key.key.span();
                    assert!(span.start.offset < span.end.offset, "Span should have valid start/end");

                    assert_eq!(span.end.offset - span.start.offset, 4, "Span should cover 'test' (4 characters)");
                }
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_shape_key_spans_quoted() {
        match do_parse("array{'hello': string}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 1);
                let field = &shape.fields[0];

                if let Some(key) = &field.key {
                    let span = key.key.span();
                    assert_eq!(span.end.offset - span.start.offset, 7, "Span should cover 'hello' including quotes");

                    assert!(matches!(&key.key, ShapeKey::String { value: b"hello", .. }));
                }
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_shape_key_spans_integer() {
        match do_parse("array{42: string}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 1);
                let field = &shape.fields[0];

                if let Some(key) = &field.key {
                    let span = key.key.span();
                    assert_eq!(span.end.offset - span.start.offset, 2, "Span should cover '42' (2 characters)");

                    assert!(matches!(&key.key, ShapeKey::Integer { value: 42, .. }));
                }
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_shape_key_spans_negated_integer() {
        match do_parse("array{-123: string}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 1);
                let field = &shape.fields[0];

                if let Some(key) = &field.key {
                    let span = key.key.span();
                    assert_eq!(span.end.offset - span.start.offset, 4, "Span should cover '-123' (4 characters)");

                    assert!(matches!(&key.key, ShapeKey::Integer { value: -123, .. }));
                }
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_shape_key_spans_complex_identifiers() {
        match do_parse("array{complex-key_name: string}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 1);
                let field = &shape.fields[0];

                if let Some(key) = &field.key {
                    let span = key.key.span();
                    assert_eq!(
                        span.end.offset - span.start.offset,
                        16,
                        "Span should cover 'complex-key_name' (16 characters)"
                    );

                    assert!(matches!(&key.key, ShapeKey::String { value: b"complex-key_name", .. }));
                }
            }
            res => panic!("Expected Ok(Type::Shape), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_shape_key_overflow_unsigned() {
        let result = do_parse("array{9223372036854775808: string}");
        assert!(result.is_err(), "Expected parse error for shape key > i64::MAX, got: {result:?}");
    }

    #[test]
    fn test_parse_shape_key_overflow_negated() {
        let result = do_parse("array{-9223372036854775808: string}");
        assert!(result.is_err(), "Expected parse error for negated shape key overflow, got: {result:?}");
    }

    #[test]
    fn test_parse_wildcard_asterisk() {
        let result = do_parse("*");
        assert!(result.is_ok(), "Expected successful parse for wildcard, got: {result:?}");
        match result.unwrap() {
            Type::Wildcard(w) => assert_eq!(w.kind, WildcardKind::Asterisk),
            other => panic!("Expected Type::Wildcard, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_wildcard_underscore() {
        let result = do_parse("_");
        assert!(result.is_ok(), "Expected successful parse for underscore wildcard, got: {result:?}");
        match result.unwrap() {
            Type::Wildcard(w) => assert_eq!(w.kind, WildcardKind::Underscore),
            other => panic!("Expected Type::Wildcard, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_wildcard_in_generic() {
        let result = do_parse("array<string, *>");
        assert!(result.is_ok(), "Expected successful parse for wildcard in generic, got: {result:?}");

        let result = do_parse("array<string, _>");
        assert!(result.is_ok(), "Expected successful parse for underscore wildcard in generic, got: {result:?}");
    }

    #[test]
    fn test_parse_wildcard_display() {
        assert_eq!(do_parse("*").unwrap().to_string(), "*");
        assert_eq!(do_parse("_").unwrap().to_string(), "_");
    }

    #[test]
    fn test_parse_non_zero_int() {
        match do_parse("non-zero-int") {
            Ok(Type::NonZeroInt(k)) => assert_eq!(k.value, b"non-zero-int".as_slice()),
            other => panic!("Expected Type::NonZeroInt, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_int_range_int_keyword_max() {
        match do_parse("int<0, int>") {
            Ok(Type::IntRange(range)) => {
                assert!(matches!(range.min, IntOrKeyword::Int(LiteralIntType { value: 0, .. })));
                match range.max {
                    IntOrKeyword::Keyword(keyword) => assert!(keyword.value.eq_ignore_ascii_case(b"int")),
                    other => panic!("Expected IntOrKeyword::Keyword, got: {other:?}"),
                }
            }
            other => panic!("Expected Type::IntRange, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_int_range_int_keyword_min() {
        match do_parse("int<int, 0>") {
            Ok(Type::IntRange(range)) => {
                match range.min {
                    IntOrKeyword::Keyword(keyword) => assert!(keyword.value.eq_ignore_ascii_case(b"int")),
                    other => panic!("Expected IntOrKeyword::Keyword, got: {other:?}"),
                }
                assert!(matches!(range.max, IntOrKeyword::Int(LiteralIntType { value: 0, .. })));
            }
            other => panic!("Expected Type::IntRange, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_member_reference_reserved_keywords() {
        for name in [
            "NULL", "ARRAY", "INT", "STRING", "FLOAT", "TRUE", "FALSE", "MIXED", "CALLABLE", "ITERABLE", "RESOURCE",
            "BOOL", "OBJECT", "NEVER", "VOID", "NUMERIC", "SCALAR",
        ] {
            let input = format!("TypeIdentifier::{name}");
            match do_parse(&input) {
                Ok(Type::MemberReference(r)) => match r.member {
                    MemberReferenceSelector::Identifier(ident) => {
                        assert!(
                            ident.value.eq_ignore_ascii_case(name.as_bytes()),
                            "Expected member name {name}, got {}",
                            String::from_utf8_lossy(ident.value),
                        );
                    }
                    other => panic!("Expected Identifier selector for {input}, got {other:?}"),
                },
                other => panic!("Expected Type::MemberReference for {input}, got: {other:?}"),
            }
        }
    }

    #[test]
    fn test_parse_member_reference_reserved_prefix_wildcard() {
        match do_parse("Foo::INT*") {
            Ok(Type::MemberReference(r)) => match r.member {
                MemberReferenceSelector::StartsWith(ident, _) => {
                    assert!(
                        ident.value.eq_ignore_ascii_case(b"INT"),
                        "expected INT prefix, got {}",
                        String::from_utf8_lossy(ident.value)
                    );
                }
                other => panic!("Expected StartsWith selector, got {other:?}"),
            },
            other => panic!("Expected Type::MemberReference, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_nested_generic_with_reserved_const() {
        match do_parse("UnionType<T|Foo::NULL>") {
            Ok(Type::Reference(r)) => {
                let params = r.parameters.expect("Expected generic parameters");
                assert_eq!(params.entries.len(), 1);
                match &params.entries[0].inner {
                    Type::Union(u) => {
                        assert!(matches!(u.left, Type::Reference(_)));
                        assert!(matches!(u.right, Type::MemberReference(_)));
                    }
                    other => panic!("Expected inner Union, got {other:?}"),
                }
            }
            other => panic!("Expected Type::Reference, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_builtin_type_identifier_union() {
        let input = "BuiltinType<TypeIdentifier::ARRAY>|BuiltinType<TypeIdentifier::ITERABLE>|ObjectType|GenericType";
        assert!(do_parse(input).is_ok(), "expected successful parse for {input}");
    }

    #[test]
    fn test_parse_collection_type_with_reserved_identifier() {
        let input = "CollectionType<BuiltinType<TypeIdentifier::ITERABLE>>";
        assert!(do_parse(input).is_ok(), "expected successful parse for {input}");
    }

    #[test]
    fn test_parse_trailing_pipe() {
        match do_parse("int|string|") {
            Ok(Type::TrailingPipe(trailing)) => assert!(matches!(trailing.inner, Type::Union(_))),
            other => panic!("Expected Type::TrailingPipe, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_trailing_pipe_single() {
        match do_parse("int|") {
            Ok(Type::TrailingPipe(trailing)) => assert!(matches!(trailing.inner, Type::Int(_))),
            other => panic!("Expected Type::TrailingPipe, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_trailing_pipe_in_shape_value() {
        match do_parse("array{0: int|string|}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 1);
                assert!(matches!(shape.fields[0].value, Type::TrailingPipe(_)));
            }
            other => panic!("Expected Type::Shape, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_trailing_pipe_in_generic_shape_value() {
        let input = "iterable<array{0: int|array<string, mixed>|}>";
        match do_parse(input) {
            Ok(Type::Iterable(iter)) => {
                let params = iter.parameters.expect("expected generic parameters");
                assert_eq!(params.entries.len(), 1);
                match &params.entries[0].inner {
                    Type::Shape(shape) => {
                        assert_eq!(shape.fields.len(), 1);
                        assert!(matches!(shape.fields[0].value, Type::TrailingPipe(_)));
                    }
                    other => panic!("Expected Type::Shape, got {other:?}"),
                }
            }
            other => panic!("Expected Type::Iterable, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_global_wildcard_starts_with() {
        match do_parse("FILTER_FLAG_*") {
            Ok(Type::GlobalWildcardReference(g)) => match g.selector {
                GlobalWildcardSelector::StartsWith(identifier, _) => {
                    assert_eq!(identifier.value, b"FILTER_FLAG_".as_slice());
                }
                other @ GlobalWildcardSelector::EndsWith(..) => {
                    panic!("Expected StartsWith selector, got {other:?}")
                }
            },
            other => panic!("Expected Type::GlobalWildcardReference, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_global_wildcard_ends_with() {
        match do_parse("*_SUFFIX") {
            Ok(Type::GlobalWildcardReference(g)) => match g.selector {
                GlobalWildcardSelector::EndsWith(_, identifier) => {
                    assert_eq!(identifier.value, b"_SUFFIX".as_slice());
                }
                other @ GlobalWildcardSelector::StartsWith(..) => {
                    panic!("Expected EndsWith selector, got {other:?}")
                }
            },
            other => panic!("Expected Type::GlobalWildcardReference, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_global_wildcard_in_int_mask_of() {
        let input = "int-mask-of<FILTER_FLAG_*>";
        match do_parse(input) {
            Ok(Type::IntMaskOf(mask)) => {
                assert!(matches!(mask.parameter.entry.inner, Type::GlobalWildcardReference(_)));
            }
            other => panic!("Expected Type::IntMaskOf, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_int_mask_of_class_wildcard_regression() {
        let input = "int-mask-of<Ulid::FORMAT_*>";
        match do_parse(input) {
            Ok(Type::IntMaskOf(mask)) => match &mask.parameter.entry.inner {
                Type::MemberReference(r) => {
                    assert!(matches!(r.member, MemberReferenceSelector::StartsWith(_, _)));
                }
                other => panic!("Expected MemberReference, got {other:?}"),
            },
            other => panic!("Expected Type::IntMaskOf, got: {other:?}"),
        }
    }
}
