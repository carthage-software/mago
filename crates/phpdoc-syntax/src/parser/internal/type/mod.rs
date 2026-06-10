use mago_allocator::Arena;

use crate::cst::r#type::CallableTypeKind;
use crate::cst::r#type::PropertiesOfFilter;
use crate::cst::r#type::ReferenceKind;
use crate::cst::r#type::Type;
use crate::cst::variable::Variable;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::parser::internal::r#type::keyword::TypeKeyword;
use crate::parser::internal::r#type::keyword::lookup_keyword;
use crate::token::Token;
use crate::token::TokenKind;

pub(crate) mod alias_reference;
pub(crate) mod array;
pub(crate) mod callable;
pub(crate) mod class_like_string;
pub(crate) mod composite;
pub(crate) mod conditional;
pub(crate) mod generics;
pub(crate) mod index_access;
pub(crate) mod int_mask;
pub(crate) mod int_range;
pub(crate) mod iterable;
pub(crate) mod key_of;
pub(crate) mod keyword;
pub(crate) mod literal;
pub(crate) mod new;
pub(crate) mod object;
pub(crate) mod properties_of;
pub(crate) mod reference;
pub(crate) mod shape;
pub(crate) mod slice;
pub(crate) mod template_type;
pub(crate) mod unary;
pub(crate) mod value_of;
pub(crate) mod wildcard;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypePrecedence {
    Lowest,
    Conditional,
    Union,
    Intersection,
    Postfix,
    Callable,
}

#[inline]
pub(super) fn is_keyword(token: &Token<'_>, keyword: TypeKeyword) -> bool {
    token.kind == TokenKind::Identifier && lookup_keyword(token.value) == Some(keyword)
}

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    #[inline]
    pub(crate) fn parse_type(&mut self) -> Result<Type<'arena>, ParseError> {
        self.parse_type_with_precedence(TypePrecedence::Lowest)
    }

    #[inline]
    pub(crate) fn parse_type_without_conditional(&mut self) -> Result<Type<'arena>, ParseError> {
        self.parse_type_with_precedence(TypePrecedence::Union)
    }

    #[inline]
    pub(crate) fn is_at_member_identifier_at(&mut self, offset: usize) -> bool {
        self.stream.lookahead(offset).is_some_and(|token| token.kind == TokenKind::Identifier)
    }

    #[inline]
    pub(crate) fn is_at_member_identifier(&mut self) -> bool {
        self.is_at_member_identifier_at(0)
    }

    #[inline]
    pub(crate) fn eat_member_identifier(&mut self) -> Result<Token<'arena>, ParseError> {
        self.stream.eat(TokenKind::Identifier)
    }

    pub(crate) fn parse_primary_type(&mut self) -> Result<Type<'arena>, ParseError> {
        let next = self.stream.peek()?;
        let file_id = self.file_id();

        let inner = match next.kind {
            TokenKind::Variable => Type::Variable(Variable::from_token(self.stream.consume()?, file_id)),
            TokenKind::ThisVariable => Type::ThisVariable(Variable::from_token(self.stream.consume()?, file_id)),
            TokenKind::Question => self.parse_nullable_type()?,
            TokenKind::LeftParenthesis => self.parse_parenthesized_type()?,
            TokenKind::Asterisk => self.parse_wildcard_type()?,
            TokenKind::Minus => self.parse_negated_type()?,
            TokenKind::Plus => self.parse_posited_type()?,
            TokenKind::Bang => self.parse_alias_reference_type()?,
            TokenKind::LiteralInteger
            | TokenKind::LiteralFloat
            | TokenKind::SingleQuotedString
            | TokenKind::DoubleQuotedString => self.parse_literal_type()?,
            TokenKind::PartialString => return Err(ParseError::UnclosedLiteralString(next.span_for(file_id))),
            TokenKind::Identifier => match lookup_keyword(next.value) {
                Some(keyword) => self.parse_keyword_type(keyword)?,
                None => self.parse_reference_type()?,
            },
            _ => return Err(ParseError::UnexpectedToken(next.span_for(file_id))),
        };

        Ok(inner)
    }

    fn parse_keyword_type(&mut self, keyword: TypeKeyword) -> Result<Type<'arena>, ParseError> {
        let ty = match keyword {
            TypeKeyword::Int | TypeKeyword::Integer => {
                let keyword = self.parse_keyword()?;
                if self.stream.is_at(TokenKind::LeftAngleBracket) {
                    self.parse_int_range_type(keyword)?
                } else {
                    Type::Int(keyword)
                }
            }
            TypeKeyword::Array
            | TypeKeyword::NonEmptyArray
            | TypeKeyword::AssociativeArray
            | TypeKeyword::List
            | TypeKeyword::NonEmptyList => self.parse_array_like_type()?,
            TypeKeyword::Object => self.parse_object_type()?,
            TypeKeyword::Iterable => self.parse_iterable_type()?,
            TypeKeyword::KeyOf => self.parse_key_of_type()?,
            TypeKeyword::ValueOf => self.parse_value_of_type()?,
            TypeKeyword::IntMaskOf => self.parse_int_mask_of_type()?,
            TypeKeyword::IntMask => self.parse_int_mask_type()?,
            TypeKeyword::New => self.parse_new_type()?,
            TypeKeyword::TemplateType => self.parse_template_type()?,
            TypeKeyword::PropertiesOf => self.parse_properties_of_type(PropertiesOfFilter::All)?,
            TypeKeyword::PublicPropertiesOf => self.parse_properties_of_type(PropertiesOfFilter::Public)?,
            TypeKeyword::PrivatePropertiesOf => self.parse_properties_of_type(PropertiesOfFilter::Private)?,
            TypeKeyword::ProtectedPropertiesOf => self.parse_properties_of_type(PropertiesOfFilter::Protected)?,
            TypeKeyword::ClassString => self.parse_class_string_type()?,
            TypeKeyword::ClassLikeString => self.parse_class_like_string_type()?,
            TypeKeyword::InterfaceString => self.parse_interface_string_type()?,
            TypeKeyword::EnumString => self.parse_enum_string_type()?,
            TypeKeyword::TraitString => self.parse_trait_string_type()?,
            TypeKeyword::Callable => self.parse_callable_type(CallableTypeKind::Callable)?,
            TypeKeyword::PureCallable => self.parse_callable_type(CallableTypeKind::PureCallable)?,
            TypeKeyword::PureClosure => self.parse_callable_type(CallableTypeKind::PureClosure)?,
            TypeKeyword::Mixed => Type::Mixed(self.parse_keyword()?),
            TypeKeyword::NonEmptyMixed => Type::NonEmptyMixed(self.parse_keyword()?),
            TypeKeyword::Null => Type::Null(self.parse_keyword()?),
            TypeKeyword::Void => Type::Void(self.parse_keyword()?),
            TypeKeyword::Never
            | TypeKeyword::NoReturn
            | TypeKeyword::NeverReturn
            | TypeKeyword::NeverReturns
            | TypeKeyword::Nothing => Type::Never(self.parse_keyword()?),
            TypeKeyword::Resource => Type::Resource(self.parse_keyword()?),
            TypeKeyword::OpenResource => Type::OpenResource(self.parse_keyword()?),
            TypeKeyword::ClosedResource => Type::ClosedResource(self.parse_keyword()?),
            TypeKeyword::True => Type::True(self.parse_keyword()?),
            TypeKeyword::False => Type::False(self.parse_keyword()?),
            TypeKeyword::Bool | TypeKeyword::Boolean => Type::Bool(self.parse_keyword()?),
            TypeKeyword::Float | TypeKeyword::Real | TypeKeyword::Double => Type::Float(self.parse_keyword()?),
            TypeKeyword::String => Type::String(self.parse_keyword()?),
            TypeKeyword::Scalar => Type::Scalar(self.parse_keyword()?),
            TypeKeyword::Numeric => Type::Numeric(self.parse_keyword()?),
            TypeKeyword::ArrayKey => Type::ArrayKey(self.parse_keyword()?),
            TypeKeyword::StringableObject => Type::StringableObject(self.parse_keyword()?),
            TypeKeyword::CallableString => Type::CallableString(self.parse_keyword()?),
            TypeKeyword::LowercaseCallableString => Type::LowercaseCallableString(self.parse_keyword()?),
            TypeKeyword::UppercaseCallableString => Type::UppercaseCallableString(self.parse_keyword()?),
            TypeKeyword::NumericString => Type::NumericString(self.parse_keyword()?),
            TypeKeyword::NonEmptyString => Type::NonEmptyString(self.parse_keyword()?),
            TypeKeyword::NonEmptyLowercaseString => Type::NonEmptyLowercaseString(self.parse_keyword()?),
            TypeKeyword::LowercaseString => Type::LowercaseString(self.parse_keyword()?),
            TypeKeyword::NonEmptyUppercaseString => Type::NonEmptyUppercaseString(self.parse_keyword()?),
            TypeKeyword::UppercaseString => Type::UppercaseString(self.parse_keyword()?),
            TypeKeyword::TruthyString => Type::TruthyString(self.parse_keyword()?),
            TypeKeyword::NonFalsyString => Type::NonFalsyString(self.parse_keyword()?),
            TypeKeyword::PositiveInt => Type::PositiveInt(self.parse_keyword()?),
            TypeKeyword::NegativeInt => Type::NegativeInt(self.parse_keyword()?),
            TypeKeyword::NonPositiveInt => Type::NonPositiveInt(self.parse_keyword()?),
            TypeKeyword::NonNegativeInt => Type::NonNegativeInt(self.parse_keyword()?),
            TypeKeyword::NonZeroInt => Type::NonZeroInt(self.parse_keyword()?),
            TypeKeyword::UnspecifiedLiteralInt => Type::UnspecifiedLiteralInt(self.parse_keyword()?),
            TypeKeyword::UnspecifiedLiteralString => Type::UnspecifiedLiteralString(self.parse_keyword()?),
            TypeKeyword::UnspecifiedLiteralFloat => Type::UnspecifiedLiteralFloat(self.parse_keyword()?),
            TypeKeyword::Empty => Type::Empty(self.parse_keyword()?),
            TypeKeyword::EmptyScalar => Type::EmptyScalar(self.parse_keyword()?),
            TypeKeyword::NonEmptyUnspecifiedLiteralString => {
                Type::NonEmptyUnspecifiedLiteralString(self.parse_keyword()?)
            }
            TypeKeyword::Self_ => {
                let keyword = self.parse_keyword()?;

                self.parse_named_reference(ReferenceKind::Self_(keyword))?
            }
            TypeKeyword::Static => {
                let keyword = self.parse_keyword()?;

                self.parse_named_reference(ReferenceKind::Static(keyword))?
            }
            TypeKeyword::Parent => {
                let keyword = self.parse_keyword()?;

                self.parse_named_reference(ReferenceKind::Parent(keyword))?
            }
            TypeKeyword::As | TypeKeyword::Is | TypeKeyword::Not | TypeKeyword::Min | TypeKeyword::Max => {
                self.parse_reference_type()?
            }
        };

        Ok(ty)
    }
}

#[cfg(test)]
mod tests {

    use mago_allocator::LocalArena;
    use mago_database::file::FileId;
    use mago_span::Position;
    use mago_span::Span;

    use crate::cst::r#type::CallableTypeKind;
    use crate::cst::r#type::ReferenceKind;
    use crate::cst::r#type::Type;
    use crate::error::ParseError;
    use crate::parser::PHPDocParser;

    fn parse<'arena>(arena: &'arena LocalArena, source: &'arena [u8]) -> Type<'arena> {
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(source.len() as u32));
        let mut parser = PHPDocParser::new(arena, source, span);

        match parser.parse_type() {
            Ok(ty) => ty,
            Err(error) => panic!("failed to parse {:?}: {error:?}", String::from_utf8_lossy(source)),
        }
    }

    #[test]
    fn deeply_nested_type_does_not_overflow() {
        let spawned = std::thread::Builder::new().stack_size(128 * 1024 * 1024).spawn(|| {
            let arena = LocalArena::new();
            let input = "(".repeat(5000);
            let span = Span::new(FileId::zero(), Position::new(0), Position::new(input.len() as u32));
            let mut parser = PHPDocParser::new(&arena, input.as_bytes(), span);

            assert!(
                matches!(parser.parse_type(), Err(ParseError::RecursionLimitExceeded(_))),
                "expected RecursionLimitExceeded for deeply nested parentheses"
            );
        });

        let Ok(handle) = spawned else {
            panic!("failed to spawn parser thread");
        };

        if handle.join().is_err() {
            panic!("parser thread aborted (stack overflow)");
        }
    }

    #[test]
    fn parses_int_keyword() {
        let arena = LocalArena::new();
        assert!(matches!(parse(&arena, b"int"), Type::Int(_)));
        assert!(matches!(parse(&arena, b"INT"), Type::Int(_)));
        assert!(matches!(parse(&arena, b"non-empty-string"), Type::NonEmptyString(_)));
    }

    #[test]
    fn parses_reference_fallback() {
        let arena = LocalArena::new();
        let Type::Reference(reference) = parse(&arena, b"\\Foo\\Bar") else { panic!() };
        let ReferenceKind::Identifier(identifier) = reference.kind else { panic!() };
        assert_eq!(identifier.value, b"\\Foo\\Bar");
        assert!(reference.parameters.is_none());
    }

    #[test]
    fn parses_union_and_intersection() {
        let arena = LocalArena::new();
        let Type::Union(union) = parse(&arena, b"int|string") else { panic!() };
        assert!(matches!(union.left, Type::Int(_)));
        assert!(matches!(union.right, Type::String(_)));

        let Type::Intersection(intersection) = parse(&arena, b"A&B") else { panic!() };
        assert!(matches!(intersection.left, Type::Reference(_)));
        assert!(matches!(intersection.right, Type::Reference(_)));
    }

    #[test]
    fn parses_nullable() {
        let arena = LocalArena::new();
        let Type::Nullable(nullable) = parse(&arena, b"?Foo") else { panic!() };
        assert!(matches!(nullable.inner, Type::Reference(_)));
    }

    #[test]
    fn parses_generics() {
        let arena = LocalArena::new();
        let Type::Array(array) = parse(&arena, b"array<int, string>") else { panic!() };
        let Some(parameters) = array.parameters else { panic!() };
        assert_eq!(parameters.entries.len(), 2);
    }

    #[test]
    fn parses_shape() {
        let arena = LocalArena::new();
        let Type::Shape(shape) = parse(&arena, b"array{a: int, b?: string}") else { panic!() };
        assert_eq!(shape.fields.len(), 2);
    }

    #[test]
    fn parses_negated_and_posited() {
        let arena = LocalArena::new();
        let Type::Negated(negated) = parse(&arena, b"-1") else { panic!() };
        assert!(matches!(negated.operand, Type::LiteralInt(_)));

        let Type::Posited(posited) = parse(&arena, b"+2") else { panic!() };
        assert!(matches!(posited.operand, Type::LiteralInt(_)));

        let Type::Negated(negated) = parse(&arena, b"-Foo") else { panic!() };
        assert!(matches!(negated.operand, Type::Reference(_)));
    }

    #[test]
    fn parses_int_range() {
        let arena = LocalArena::new();
        assert!(matches!(parse(&arena, b"int<-5, 10>"), Type::IntRange(_)));
        assert!(matches!(parse(&arena, b"int<min, max>"), Type::IntRange(_)));
    }

    #[test]
    fn parses_key_of_and_class_string() {
        let arena = LocalArena::new();
        assert!(matches!(parse(&arena, b"key-of<Foo>"), Type::KeyOf(_)));
        assert!(matches!(parse(&arena, b"class-string"), Type::ClassString(_)));
        assert!(matches!(parse(&arena, b"class-string<Foo>"), Type::ClassString(_)));
    }

    #[test]
    fn parses_single_generic_parameter_with_trailing_comma() {
        let arena = LocalArena::new();
        let Type::ClassString(class_string) = parse(&arena, b"class-string<Foo,>") else { panic!() };
        let Some(parameter) = &class_string.parameter else { panic!("expected a generic parameter") };
        assert!(parameter.entry.comma.is_some());
    }

    #[test]
    fn parses_callable_and_closure() {
        let arena = LocalArena::new();
        let Type::Callable(callable) = parse(&arena, b"callable(int, string): bool") else { panic!() };
        assert_eq!(callable.kind, CallableTypeKind::Callable);

        let Type::Callable(closure) = parse(&arena, b"Closure(): void") else { panic!() };
        assert_eq!(closure.kind, CallableTypeKind::Closure);
    }

    #[test]
    fn parses_conditional() {
        let arena = LocalArena::new();
        assert!(matches!(parse(&arena, b"T is int ? string : float"), Type::Conditional(_)));
    }

    #[test]
    fn parses_member_reference() {
        let arena = LocalArena::new();
        assert!(matches!(parse(&arena, b"Foo::BAR"), Type::MemberReference(_)));
        assert!(matches!(parse(&arena, b"Foo::*"), Type::MemberReference(_)));
    }

    #[test]
    fn parses_literal_string() {
        let arena = LocalArena::new();
        let Type::LiteralString(literal) = parse(&arena, b"'hello'") else { panic!() };
        assert_eq!(literal.value, b"hello");
    }

    #[test]
    fn does_not_emit_invalid_float_for_dangling_exponent() {
        let arena = LocalArena::new();

        let Type::LiteralFloat(literal) = parse(&arena, b".1") else { panic!("expected .1 to be a float") };
        assert_eq!(*literal.value, 0.1);
        assert_eq!(literal.raw, b".1");

        assert!(matches!(parse(&arena, b".1E"), Type::LiteralFloat(_)));
        assert!(matches!(parse(&arena, b".1e+"), Type::LiteralFloat(_)));
        assert!(matches!(parse(&arena, b"1e"), Type::LiteralInt(_)));
        assert!(matches!(parse(&arena, b"1e+"), Type::LiteralInt(_)));
        assert!(matches!(parse(&arena, b"3.e"), Type::LiteralInt(_)));

        let _ = parse(&arena, b".1E.111.12E1ra");
    }

    #[test]
    fn errors_on_empty() {
        let arena = LocalArena::new();
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(0));
        let mut parser = PHPDocParser::new(&arena, b"", span);
        if parser.parse_type().is_ok() {
            panic!("expected an error for empty input");
        }
    }
}
