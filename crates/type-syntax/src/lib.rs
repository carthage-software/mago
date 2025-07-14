#![doc = include_str!("./../README.md")]

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

/// Parses a string representation of a PHPDoc type into an Abstract Syntax Tree (AST).
///
/// This is the main entry point for the type parser. It takes the type string
/// and its original `Span` (representing its location within the source file)
/// and returns the parsed `Type` AST or a `ParseError`.
///
/// # Arguments
///
/// * `span` - The original `Span` of the `input` string slice within its source file.
///   This is crucial for ensuring all AST nodes have correct, absolute positioning.
/// * `input` - The `&str` containing the type string to parse (e.g., `"int|string"`, `"array<int, MyClass>"`).
///
/// # Returns
///
/// * `Ok(Type)` containing the root of the parsed AST on success.
/// * `Err(ParseError)` if any lexing or parsing error occurs.
pub fn parse_str(span: Span, input: &str) -> Result<Type<'_>, ParseError> {
    // Create an Input anchored at the type string's original starting position.
    let input = Input::anchored_at(input.as_bytes(), span.start);
    // Create the type-specific lexer.
    let lexer = TypeLexer::new(input);
    // Construct the type AST using the lexer.
    parser::construct(lexer)
}

#[cfg(test)]
mod tests {
    use mago_source::SourceIdentifier;
    use mago_span::Position;
    use mago_span::Span;

    use crate::ast::*;

    use super::*;

    fn do_parse(input: &str) -> Result<Type<'_>, ParseError> {
        let source = SourceIdentifier::dummy();
        let span = Span::new(Position::new(source, 0), Position::new(source, input.len()));
        parse_str(span, input)
    }

    #[test]
    fn test_parse_simple_keyword() {
        let result = do_parse("int");
        assert!(result.is_ok());
        match result.unwrap() {
            Type::Int(k) => assert_eq!(k.value, "int"),
            _ => panic!("Expected Type::Int"),
        }
    }

    #[test]
    fn test_parse_composite_keyword() {
        let result = do_parse("non-empty-string");
        assert!(result.is_ok());
        match result.unwrap() {
            Type::NonEmptyString(k) => assert_eq!(k.value, "non-empty-string"),
            _ => panic!("Expected Type::NonEmptyString"),
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
        assert_parsed_literal_int("123_345", 123345);
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
    fn test_parse_simple_union() {
        match do_parse("int|string") {
            Ok(ty) => match ty {
                Type::Union(u) => {
                    assert!(matches!(*u.left, Type::Int(_)));
                    assert!(matches!(*u.right, Type::String(_)));
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
                    assert!(matches!(*u.left, Type::Variable(_)));
                    assert!(matches!(*u.right, Type::Variable(_)));
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
                assert!(matches!(*n.inner, Type::String(_)));
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
        assert_eq!(shape.keyword.value, "array");
        assert_eq!(shape.fields.len(), 1);
        assert!(shape.additional_fields.is_none());

        let field = &shape.fields[0];
        assert!(matches!(
            field.key.as_ref().map(|k| k.name.as_ref()),
            Some(Type::LiteralString(LiteralStringType { raw: "'name'", value: "name", .. }))
        ));
        assert!(matches!(field.value.as_ref(), Type::String(_)));
    }

    #[test]
    fn test_parse_int_key_shape() {
        match do_parse("array{0: string, 1: bool}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 2);
                let first_field = &shape.fields[0];
                assert!(matches!(
                    first_field.key.as_ref().map(|k| k.name.as_ref()),
                    Some(Type::LiteralInt(LiteralIntType { value: 0, .. }))
                ));
                assert!(matches!(first_field.value.as_ref(), Type::String(_)));
                let second_field = &shape.fields[1];
                assert!(matches!(
                    second_field.key.as_ref().map(|k| k.name.as_ref()),
                    Some(Type::LiteralInt(LiteralIntType { value: 1, .. }))
                ));
                assert!(matches!(second_field.value.as_ref(), Type::Bool(_)));
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
                assert!(matches!(*i.left, Type::Reference(_)));
                assert!(matches!(*i.right, Type::Reference(_)));

                if let Type::Reference(r) = *i.left {
                    assert_eq!(r.identifier.value, "Countable");
                } else {
                    panic!();
                }

                if let Type::Reference(r) = *i.right {
                    assert_eq!(r.identifier.value, "Traversable");
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
                assert_eq!(m.class.value, "MyClass");
                assert_eq!(m.member.to_string(), "MY_CONST");
            }
            res => panic!("Expected Ok(Type::MemberReference), got {res:?}"),
        }

        match do_parse("\\Fully\\Qualified::class") {
            Ok(Type::MemberReference(m)) => {
                assert_eq!(m.class.value, "\\Fully\\Qualified"); // Check if lexer keeps leading \
                assert_eq!(m.member.to_string(), "class");
            }
            res => panic!("Expected Ok(Type::MemberReference), got {res:?}"),
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
                    assert!(matches!(*n.inner, Type::LiteralInt(_)));
                    if let Type::LiteralInt(lit) = *n.inner {
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
            123345,
        );
        assert_negated_int("-0b1", 1);
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
                assert!(matches!(*spec.parameters.entries[0].parameter_type, Type::Int(_)));
                assert!(matches!(*spec.parameters.entries[1].parameter_type, Type::Nullable(_)));
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
                assert!(matches!(*spec.return_type.unwrap().return_type, Type::Void(_)));
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
                assert!(matches!(*spec.parameters.entries[0].parameter_type, Type::Bool(_)));
                assert!(spec.return_type.is_some());
                assert!(matches!(*spec.return_type.unwrap().return_type, Type::Int(_)));
            }
            res => panic!("Expected Ok(Type::Callable), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_closure_via_identifier() {
        match do_parse("Closure(string): bool") {
            Ok(Type::Callable(c)) => {
                assert_eq!(c.kind, CallableTypeKind::Closure);
                assert_eq!(c.keyword.value, "Closure");
                let spec = c.specification.expect("Expected callable specification");
                assert_eq!(spec.parameters.entries.len(), 1);
                assert!(matches!(*spec.parameters.entries[0].parameter_type, Type::String(_)));
                assert!(spec.return_type.is_some());
                assert!(matches!(*spec.return_type.unwrap().return_type, Type::Bool(_)));
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
                assert!(matches!(*first_param.parameter_type, Type::List(_)));
                assert!(first_param.ellipsis.is_none());
                assert!(first_param.equals.is_none());

                let second_param = &spec.parameters.entries[1];
                assert!(matches!(*second_param.parameter_type, Type::Nullable(_)));
                assert!(second_param.ellipsis.is_none());
                assert!(second_param.equals.is_some());

                let third_param = &spec.parameters.entries[2];
                assert!(matches!(*third_param.parameter_type, Type::Int(_)));
                assert!(third_param.ellipsis.is_some());
                assert!(third_param.equals.is_none());

                if let Type::Parenthesized(p) = *spec.return_type.unwrap().return_type {
                    assert!(matches!(*p.inner, Type::Union(_)));
                    if let Type::Union(u) = *p.inner {
                        assert!(matches!(u.left.as_ref(), Type::Parenthesized(_)));
                        assert!(matches!(u.right.as_ref(), Type::Null(_)));
                    }
                } else {
                    panic!("Expected Type::CallableReturnType");
                }
            }
            res => panic!("Expected Ok(Type::Callable), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_conditional_type() {
        match do_parse("int is not string ? array : int") {
            Ok(Type::Conditional(c)) => {
                assert!(matches!(*c.subject, Type::Int(_)));
                assert!(c.not.is_some());
                assert!(matches!(*c.target, Type::String(_)));
                assert!(matches!(*c.then, Type::Array(_)));
                assert!(matches!(*c.otherwise, Type::Int(_)));
            }
            res => panic!("Expected Ok(Type::Conditional), got {res:?}"),
        }

        match do_parse("$input is string ? array : int") {
            Ok(Type::Conditional(c)) => {
                assert!(matches!(*c.subject, Type::Variable(_)));
                assert!(c.not.is_none());
                assert!(matches!(*c.target, Type::String(_)));
                assert!(matches!(*c.then, Type::Array(_)));
                assert!(matches!(*c.otherwise, Type::Int(_)));
            }
            res => panic!("Expected Ok(Type::Conditional), got {res:?}"),
        }

        match do_parse("int is string ? array : (int is not $bar ? string : $baz)") {
            Ok(Type::Conditional(c)) => {
                assert!(matches!(*c.subject, Type::Int(_)));
                assert!(c.not.is_none());
                assert!(matches!(*c.target, Type::String(_)));
                assert!(matches!(*c.then, Type::Array(_)));

                let Type::Parenthesized(p) = *c.otherwise else {
                    panic!("Expected Type::Parenthesized");
                };

                if let Type::Conditional(inner_conditional) = *p.inner {
                    assert!(matches!(*inner_conditional.subject, Type::Int(_)));
                    assert!(inner_conditional.not.is_some());
                    assert!(matches!(*inner_conditional.target, Type::Variable(_)));
                    assert!(matches!(*inner_conditional.then, Type::String(_)));
                    assert!(matches!(*inner_conditional.otherwise, Type::Variable(_)));
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
                assert_eq!(k.keyword.value, "key-of");
                match &k.parameter.entry.inner {
                    Type::Reference(r) => assert_eq!(r.identifier.value, "MyArray"),
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
                assert_eq!(v.keyword.value, "value-of");
                match &v.parameter.entry.inner {
                    Type::Reference(r) => assert_eq!(r.identifier.value, "MyArray"),
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
                match *i.target {
                    Type::Reference(r) => assert_eq!(r.identifier.value, "MyArray"),
                    _ => panic!("Expected Type::Reference"),
                }
                match *i.index {
                    Type::Reference(r) => assert_eq!(r.identifier.value, "MyKey"),
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
                assert!(matches!(*s.inner, Type::String(_)));
            }
            res => panic!("Expected Ok(Type::Slice), got {res:?}"),
        }
    }

    #[test]
    fn test_slice_of_slice_of_slice_type() {
        match do_parse("string[][][]") {
            Ok(Type::Slice(s)) => {
                assert!(matches!(*s.inner, Type::Slice(_)));
                if let Type::Slice(inner_slice) = *s.inner {
                    assert!(matches!(*inner_slice.inner, Type::Slice(_)));
                    if let Type::Slice(inner_inner_slice) = *inner_slice.inner {
                        assert!(matches!(*inner_inner_slice.inner, Type::String(_)));
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
                assert_eq!(r.keyword.value, "int");

                match r.min {
                    IntOrKeyword::Int(literal_int_type) => {
                        assert_eq!(literal_int_type.value, 0);
                    }
                    _ => {
                        panic!("Expected min to be a LiteralIntType, got `{}`", r.min)
                    }
                };

                match r.max {
                    IntOrKeyword::Int(literal_int_type) => {
                        assert_eq!(literal_int_type.value, 100);
                    }
                    _ => {
                        panic!("Expected max to be a LiteralIntType, got `{}`", r.max)
                    }
                };
            }
            res => panic!("Expected Ok(Type::IntRange), got {res:?}"),
        }

        match do_parse("int<min, 0>") {
            Ok(Type::IntRange(r)) => {
                match r.min {
                    IntOrKeyword::Keyword(keyword) => {
                        assert_eq!(keyword.value, "min");
                    }
                    _ => {
                        panic!("Expected min to be a Keyword, got `{}`", r.min)
                    }
                };

                match r.max {
                    IntOrKeyword::Int(literal_int_type) => {
                        assert_eq!(literal_int_type.value, 0);
                    }
                    _ => {
                        panic!("Expected max to be a LiteralIntType, got `{}`", r.max)
                    }
                };
            }
            res => panic!("Expected Ok(Type::IntRange), got {res:?}"),
        }

        match do_parse("int<min, max>") {
            Ok(Type::IntRange(r)) => {
                match r.min {
                    IntOrKeyword::Keyword(keyword) => {
                        assert_eq!(keyword.value, "min");
                    }
                    _ => {
                        panic!("Expected min to be a Keyword, got `{}`", r.min)
                    }
                };

                match r.max {
                    IntOrKeyword::Keyword(keyword) => {
                        assert_eq!(keyword.value, "max");
                    }
                    _ => {
                        panic!("Expected max to be a Keyword, got `{}`", r.max)
                    }
                };
            }
            res => panic!("Expected Ok(Type::IntRange), got {res:?}"),
        }
    }

    #[test]
    fn test_properties_of() {
        match do_parse("properties-of<MyClass>") {
            Ok(Type::PropertiesOf(p)) => {
                assert_eq!(p.keyword.value, "properties-of");
                assert_eq!(p.filter, PropertiesOfFilter::All);
                match &p.parameter.entry.inner {
                    Type::Reference(r) => assert_eq!(r.identifier.value, "MyClass"),
                    _ => panic!(),
                }
            }
            res => panic!("Expected Ok(Type::PropertiesOf), got {res:?}"),
        }

        match do_parse("protected-properties-of<T>") {
            Ok(Type::PropertiesOf(p)) => {
                assert_eq!(p.keyword.value, "protected-properties-of");
                assert_eq!(p.filter, PropertiesOfFilter::Protected);
                match &p.parameter.entry.inner {
                    Type::Reference(r) => assert_eq!(r.identifier.value, "T"),
                    _ => panic!(),
                }
            }
            res => panic!("Expected Ok(Type::PropertiesOf), got {res:?}"),
        }

        match do_parse("private-properties-of<T>") {
            Ok(Type::PropertiesOf(p)) => {
                assert_eq!(p.keyword.value, "private-properties-of");
                assert_eq!(p.filter, PropertiesOfFilter::Private);
                match &p.parameter.entry.inner {
                    Type::Reference(r) => assert_eq!(r.identifier.value, "T"),
                    _ => panic!(),
                }
            }
            res => panic!("Expected Ok(Type::PropertiesOf), got {res:?}"),
        }

        match do_parse("public-properties-of<T>") {
            Ok(Type::PropertiesOf(p)) => {
                assert_eq!(p.keyword.value, "public-properties-of");
                assert_eq!(p.filter, PropertiesOfFilter::Public);
                match &p.parameter.entry.inner {
                    Type::Reference(r) => assert_eq!(r.identifier.value, "T"),
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
                assert_eq!(v.value, "$myVar");
            }
            res => panic!("Expected Ok(Type::Variable), got {res:?}"),
        }
    }

    #[test]
    fn test_nullable_intersection() {
        // Nullable applies only to the rightmost element of an intersection before parens
        match do_parse("Countable&?Traversable") {
            Ok(Type::Intersection(i)) => {
                assert!(matches!(*i.left, Type::Reference(r) if r.identifier.value == "Countable"));
                assert!(matches!(*i.right, Type::Nullable(_)));
                if let Type::Nullable(n) = *i.right {
                    assert!(matches!(*n.inner, Type::Reference(r) if r.identifier.value == "Traversable"));
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
                assert!(matches!(*n.inner, Type::Parenthesized(_)));
                if let Type::Parenthesized(p) = *n.inner {
                    assert!(matches!(*p.inner, Type::Intersection(_)));
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
                assert!(matches!(*u.left, Type::PositiveInt(_)));
                assert!(matches!(*u.right, Type::NegativeInt(_)));
            }
            res => panic!("Expected Ok(Type::Union), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_float_alias() {
        match do_parse("double") {
            Ok(Type::Float(f)) => {
                assert_eq!(f.value, "double");
            }
            res => panic!("Expected Ok(Type::Float), got {res:?}"),
        }

        match do_parse("real") {
            Ok(Type::Float(f)) => {
                assert_eq!(f.value, "real");
            }
            res => panic!("Expected Ok(Type::Float), got {res:?}"),
        }

        match do_parse("float") {
            Ok(Type::Float(f)) => {
                assert_eq!(f.value, "float");
            }
            res => panic!("Expected Ok(Type::Float), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_bool_alias() {
        match do_parse("boolean") {
            Ok(Type::Bool(b)) => {
                assert_eq!(b.value, "boolean");
            }
            res => panic!("Expected Ok(Type::Bool), got {res:?}"),
        }

        match do_parse("bool") {
            Ok(Type::Bool(b)) => {
                assert_eq!(b.value, "bool");
            }
            res => panic!("Expected Ok(Type::Bool), got {res:?}"),
        }
    }

    #[test]
    fn test_parse_integer_alias() {
        match do_parse("integer") {
            Ok(Type::Int(i)) => {
                assert_eq!(i.value, "integer");
            }
            res => panic!("Expected Ok(Type::Int), got {res:?}"),
        }

        match do_parse("int") {
            Ok(Type::Int(i)) => {
                assert_eq!(i.value, "int");
            }
            res => panic!("Expected Ok(Type::Int), got {res:?}"),
        }
    }
}
