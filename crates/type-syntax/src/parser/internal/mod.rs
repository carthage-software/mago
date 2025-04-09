use ordered_float::OrderedFloat;

use mago_syntax_core::utils::parse_literal_float;
use mago_syntax_core::utils::parse_literal_integer;

use crate::ast::*;
use crate::error::ParseError;
use crate::parser::internal::array_like::parse_array_like_type;
use crate::parser::internal::callable::parse_callable_type_specifications;
use crate::parser::internal::callable::parse_optional_callable_type_specifications;
use crate::parser::internal::generic::parse_optional_generic_parameters;
use crate::parser::internal::stream::TypeTokenStream;
use crate::token::TypeTokenKind;

pub mod array_like;
pub mod callable;
pub mod generic;
pub mod stream;

#[inline]
pub fn parse_type<'input>(stream: &mut TypeTokenStream<'input>) -> Result<Type<'input>, ParseError> {
    let next = stream.peek()?;
    let inner = match next.kind {
        TypeTokenKind::Variable => Type::Variable(VariableType::from(stream.consume()?)),
        TypeTokenKind::Question => {
            Type::Nullable(NullableType { question_mark: stream.consume()?.span, inner: Box::new(parse_type(stream)?) })
        }
        TypeTokenKind::LeftParenthesis => Type::Parenthesized(ParenthesizedType {
            left_parenthesis: stream.consume()?.span,
            inner: Box::new(parse_type(stream)?),
            right_parenthesis: stream.eat(TypeTokenKind::RightParenthesis)?.span,
        }),
        TypeTokenKind::Mixed => Type::Mixed(Keyword::from(stream.consume()?)),
        TypeTokenKind::Null => Type::Null(Keyword::from(stream.consume()?)),
        TypeTokenKind::Void => Type::Void(Keyword::from(stream.consume()?)),
        TypeTokenKind::Never => Type::Never(Keyword::from(stream.consume()?)),
        TypeTokenKind::Resource => Type::Resource(Keyword::from(stream.consume()?)),
        TypeTokenKind::ClosedResource => Type::ClosedResource(Keyword::from(stream.consume()?)),
        TypeTokenKind::OpenResource => Type::OpenResource(Keyword::from(stream.consume()?)),
        TypeTokenKind::True => Type::True(Keyword::from(stream.consume()?)),
        TypeTokenKind::False => Type::False(Keyword::from(stream.consume()?)),
        TypeTokenKind::Bool => Type::Bool(Keyword::from(stream.consume()?)),
        TypeTokenKind::Float => Type::Float(Keyword::from(stream.consume()?)),
        TypeTokenKind::Int => Type::Int(Keyword::from(stream.consume()?)),
        TypeTokenKind::String => Type::String(Keyword::from(stream.consume()?)),
        TypeTokenKind::NumericString => Type::NumericString(Keyword::from(stream.consume()?)),
        TypeTokenKind::NonEmptyString => Type::NonEmptyString(Keyword::from(stream.consume()?)),
        TypeTokenKind::TruthyString => Type::TruthyString(Keyword::from(stream.consume()?)),
        TypeTokenKind::Object => Type::Object(Keyword::from(stream.consume()?)),
        TypeTokenKind::NoReturn | TypeTokenKind::NeverReturn | TypeTokenKind::NeverReturns | TypeTokenKind::Nothing => {
            Type::Never(Keyword::from(stream.consume()?))
        }
        TypeTokenKind::Scalar => Type::Scalar(Keyword::from(stream.consume()?)),
        TypeTokenKind::Numeric => Type::Numeric(Keyword::from(stream.consume()?)),
        TypeTokenKind::ArrayKey => Type::ArrayKey(Keyword::from(stream.consume()?)),
        TypeTokenKind::StringableObject => Type::StringableObject(Keyword::from(stream.consume()?)),
        TypeTokenKind::UnspecifiedLiteralString => Type::UnspecifiedLiteralString(Keyword::from(stream.consume()?)),
        TypeTokenKind::NonEmptyUnspecifiedLiteralString => {
            Type::NonEmptyUnspecifiedLiteralString(Keyword::from(stream.consume()?))
        }
        TypeTokenKind::Array
        | TypeTokenKind::NonEmptyArray
        | TypeTokenKind::AssociativeArray
        | TypeTokenKind::List
        | TypeTokenKind::NonEmptyList => parse_array_like_type(stream)?,
        TypeTokenKind::Iterable => Type::Iterable(IterableType {
            keyword: Keyword::from(stream.consume()?),
            parameters: parse_optional_generic_parameters(stream)?,
        }),
        TypeTokenKind::LiteralFloat => {
            let token = stream.consume()?;
            let value = parse_literal_float(token.value).unwrap_or_else(|| {
                unreachable!("lexer generated invalid float `{}`; this should never happen.", token.value)
            });

            Type::LiteralFloat(LiteralFloatType { span: token.span, value: OrderedFloat(value), raw: token.value })
        }
        TypeTokenKind::LiteralInteger => {
            let token = stream.consume()?;
            let value = parse_literal_integer(token.value).unwrap_or_else(|| {
                unreachable!("lexer generated invalid integer `{}`; this should never happen.", token.value)
            });

            Type::LiteralInt(LiteralIntType { span: token.span, value, raw: token.value })
        }
        TypeTokenKind::LiteralString => {
            let token = stream.consume()?;
            let value = &token.value[1..token.value.len() - 1];

            Type::LiteralString(LiteralStringType { span: token.span, value, raw: token.value })
        }
        TypeTokenKind::Minus => {
            Type::Negated(NegatedType { minus: stream.consume()?.span, inner: Box::new(parse_type(stream)?) })
        }
        TypeTokenKind::Plus => {
            Type::Posited(PositedType { plus: stream.consume()?.span, inner: Box::new(parse_type(stream)?) })
        }
        TypeTokenKind::EnumString => Type::EnumString(EnumStringType {
            keyword: Keyword::from(stream.consume()?),
            parameters: parse_optional_generic_parameters(stream)?,
        }),
        TypeTokenKind::TraitString => Type::TraitString(TraitStringType {
            keyword: Keyword::from(stream.consume()?),
            parameters: parse_optional_generic_parameters(stream)?,
        }),
        TypeTokenKind::ClassString => Type::ClassString(ClassStringType {
            keyword: Keyword::from(stream.consume()?),
            parameters: parse_optional_generic_parameters(stream)?,
        }),
        TypeTokenKind::InterfaceString => Type::InterfaceString(InterfaceStringType {
            keyword: Keyword::from(stream.consume()?),
            parameters: parse_optional_generic_parameters(stream)?,
        }),
        TypeTokenKind::Callable => Type::Callable(CallableType {
            kind: CallableTypeKind::Callable,
            keyword: Keyword::from(stream.consume()?),
            specification: parse_optional_callable_type_specifications(stream)?,
        }),
        TypeTokenKind::PureCallable => Type::Callable(CallableType {
            kind: CallableTypeKind::PureCallable,
            keyword: Keyword::from(stream.consume()?),
            specification: parse_optional_callable_type_specifications(stream)?,
        }),
        TypeTokenKind::PureClosure => Type::Callable(CallableType {
            kind: CallableTypeKind::PureClosure,
            keyword: Keyword::from(stream.consume()?),
            specification: parse_optional_callable_type_specifications(stream)?,
        }),
        TypeTokenKind::QualifiedIdentifier => {
            let identifier = Identifier::from(stream.consume()?);
            if stream.is_at(TypeTokenKind::ColonColon)? {
                Type::MemberReference(MemberReferenceType {
                    class: identifier,
                    double_colon: stream.consume()?.span,
                    member: Identifier::from(stream.eat(TypeTokenKind::Identifier)?),
                })
            } else {
                Type::Reference(ReferenceType { identifier, parameters: parse_optional_generic_parameters(stream)? })
            }
        }
        TypeTokenKind::Identifier => {
            if next.value.eq_ignore_ascii_case("Closure")
                && matches!(stream.lookahead(1)?.map(|t| t.kind), Some(TypeTokenKind::LeftParenthesis))
            {
                Type::Callable(CallableType {
                    kind: CallableTypeKind::Closure,
                    keyword: Keyword::from(stream.consume()?),
                    specification: Some(parse_callable_type_specifications(stream)?),
                })
            } else {
                let identifier = Identifier::from(stream.consume()?);
                if stream.is_at(TypeTokenKind::ColonColon)? {
                    Type::MemberReference(MemberReferenceType {
                        class: identifier,
                        double_colon: stream.consume()?.span,
                        member: Identifier::from(stream.eat(TypeTokenKind::Identifier)?),
                    })
                } else {
                    Type::Reference(ReferenceType {
                        identifier,
                        parameters: parse_optional_generic_parameters(stream)?,
                    })
                }
            }
        }
        TypeTokenKind::FullyQualifiedIdentifier => {
            if next.value.eq_ignore_ascii_case("\\Closure")
                && matches!(stream.lookahead(1)?.map(|t| t.kind), Some(TypeTokenKind::LeftParenthesis))
            {
                Type::Callable(CallableType {
                    kind: CallableTypeKind::Closure,
                    keyword: Keyword::from(stream.consume()?),
                    specification: Some(parse_callable_type_specifications(stream)?),
                })
            } else {
                let identifier = Identifier::from(stream.consume()?);

                if stream.is_at(TypeTokenKind::ColonColon)? {
                    Type::MemberReference(MemberReferenceType {
                        class: identifier,
                        double_colon: stream.consume()?.span,
                        member: Identifier::from(stream.eat(TypeTokenKind::Identifier)?),
                    })
                } else {
                    Type::Reference(ReferenceType {
                        identifier,
                        parameters: parse_optional_generic_parameters(stream)?,
                    })
                }
            }
        }
        TypeTokenKind::Whitespace | TypeTokenKind::SingleLineComment => {
            unreachable!("trivia tokens are skipped by the stream.")
        }
        TypeTokenKind::PartialLiteralString => {
            return Err(ParseError::UnclosedLiteralString(next.span));
        }
        _ => {
            return Err(ParseError::UnexpectedToken(vec![], next.kind, next.span));
        }
    };

    // Nullable types can't be used in unions or intersections
    if let Type::Nullable(_) = inner {
        return Ok(inner);
    }

    Ok(match stream.lookahead(0)?.map(|t| t.kind) {
        Some(TypeTokenKind::Pipe) => Type::Union(UnionType {
            left: Box::new(inner),
            pipe: stream.consume()?.span,
            right: Box::new(parse_type(stream)?),
        }),
        Some(TypeTokenKind::Ampersand) => Type::Intersection(IntersectionType {
            left: Box::new(inner),
            ampersand: stream.consume()?.span,
            right: Box::new(parse_type(stream)?),
        }),
        Some(TypeTokenKind::Is) => Type::Conditional(ConditionalType {
            subject: Box::new(inner),
            is: Keyword::from(stream.consume()?),
            not: if stream.is_at(TypeTokenKind::Not)? { Some(Keyword::from(stream.consume()?)) } else { None },
            target: Box::new(parse_type(stream)?),
            question_mark: stream.eat(TypeTokenKind::Question)?.span,
            then: Box::new(parse_type(stream)?),
            colon: stream.eat(TypeTokenKind::Colon)?.span,
            otherwise: Box::new(parse_type(stream)?),
        }),
        _ => inner,
    })
}
