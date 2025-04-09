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
                panic!("Failed to parse union type: {:?}", err);
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
            res => panic!("Expected Ok(Type::Array), got {:?}", res),
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
            res => panic!("Expected Ok(Type::List), got {:?}", res),
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
            res => panic!("Expected Ok(Type::NonEmptyArray), got {:?}", res),
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
            res => panic!("Expected Ok(Type::List), got {:?}", res),
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
            field.key.as_ref(),
            Type::LiteralString(LiteralStringType { raw: "'name'", value: "name", .. })
        ));
        assert!(matches!(field.value.as_ref(), Type::String(_)));
    }

    #[test]
    fn test_parse_int_key_shape() {
        match do_parse("array{0: string, 1: bool}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 2);
                let first_field = &shape.fields[0];
                assert!(matches!(first_field.key.as_ref(), Type::LiteralInt(LiteralIntType { value: 0, .. })));
                assert!(matches!(first_field.value.as_ref(), Type::String(_)));
                let second_field = &shape.fields[1];
                assert!(matches!(second_field.key.as_ref(), Type::LiteralInt(LiteralIntType { value: 1, .. })));
                assert!(matches!(second_field.value.as_ref(), Type::Bool(_)));
            }
            res => panic!("Expected Ok(Type::Shape), got {:?}", res),
        }
    }

    #[test]
    fn test_parse_optional_field_shape() {
        match do_parse("array{name: string, age?: int, address: string}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 3);
                assert!(shape.fields[0].question_mark.is_none());
                assert!(shape.fields[1].question_mark.is_some());
                assert!(shape.fields[2].question_mark.is_none());
            }
            res => panic!("Expected Ok(Type::Shape), got {:?}", res),
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
            res => panic!("Expected Ok(Type::Shape), got {:?}", res),
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
            res => panic!("Expected Ok(Type::Shape), got {:?}", res),
        }
    }

    #[test]
    fn test_parse_empty_shape() {
        match do_parse("array{}") {
            Ok(Type::Shape(shape)) => {
                assert_eq!(shape.fields.len(), 0);
                assert!(shape.additional_fields.is_none());
            }
            res => panic!("Expected Ok(Type::Shape), got {:?}", res),
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
            res => panic!("Expected Ok(Type::Intersection), got {:?}", res),
        }
    }

    #[test]
    fn test_parse_member_ref() {
        match do_parse("MyClass::MY_CONST") {
            Ok(Type::MemberReference(m)) => {
                assert_eq!(m.class.value, "MyClass");
                assert_eq!(m.member.value, "MY_CONST");
            }
            res => panic!("Expected Ok(Type::MemberReference), got {:?}", res),
        }

        match do_parse("\\Fully\\Qualified::class") {
            Ok(Type::MemberReference(m)) => {
                assert_eq!(m.class.value, "\\Fully\\Qualified"); // Check if lexer keeps leading \
                assert_eq!(m.member.value, "class");
            }
            res => panic!("Expected Ok(Type::MemberReference), got {:?}", res),
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
            res => panic!("Expected Ok(Type::Iterable), got {:?}", res),
        }

        match do_parse("iterable<bool>") {
            // Test single param case
            Ok(Type::Iterable(i)) => {
                let params = i.parameters.expect("Expected generic parameters");
                assert_eq!(params.entries.len(), 1);
                assert!(matches!(params.entries[0].inner, Type::Bool(_)));
            }
            res => panic!("Expected Ok(Type::Iterable), got {:?}", res),
        }

        match do_parse("iterable") {
            Ok(Type::Iterable(i)) => {
                assert!(i.parameters.is_none());
            }
            res => panic!("Expected Ok(Type::Iterable), got {:?}", res),
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
            res => panic!("Expected Ok(Type::Callable), got {:?}", res),
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
            res => panic!("Expected Ok(Type::Callable), got {:?}", res),
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
            res => panic!("Expected Ok(Type::Callable), got {:?}", res),
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
            res => panic!("Expected Ok(Type::Callable), got {:?}", res),
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
            res => panic!("Expected Ok(Type::Callable) for Closure, got {:?}", res),
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
            res => panic!("Expected Ok(Type::Callable), got {:?}", res),
        }
    }
}
