use ordered_float::OrderedFloat;

use mago_syntax_core::utils::parse_literal_float;
use mago_syntax_core::utils::parse_literal_integer;

use crate::ast::*;
use crate::error::ParseError;
use crate::parser::internal::array_like::parse_array_like_type;
use crate::parser::internal::callable::parse_callable_type_specifications;
use crate::parser::internal::callable::parse_optional_callable_type_specifications;
use crate::parser::internal::generic::parse_generic_parameters_or_none;
use crate::parser::internal::generic::parse_single_generic_parameter;
use crate::parser::internal::generic::parse_single_generic_parameter_or_none;
use crate::parser::internal::object::parse_object_type;
use crate::parser::internal::stream::TypeTokenStream;
use crate::token::TypeTokenKind;

pub mod array_like;
pub mod callable;
pub mod generic;
pub mod object;
pub mod stream;

#[inline]
pub fn parse_type<'input>(stream: &mut TypeTokenStream<'input>) -> Result<Type<'input>, ParseError> {
    let next = stream.peek()?;
    let mut inner = match next.kind {
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
        TypeTokenKind::Bool | TypeTokenKind::Boolean => Type::Bool(Keyword::from(stream.consume()?)),
        TypeTokenKind::Float | TypeTokenKind::Real | TypeTokenKind::Double => {
            Type::Float(Keyword::from(stream.consume()?))
        }
        TypeTokenKind::Int | TypeTokenKind::Integer => {
            let keyword = Keyword::from(stream.consume()?);

            if stream.is_at(TypeTokenKind::LessThan)? {
                Type::IntRange(IntRangeType {
                    keyword,
                    less_than: stream.consume()?.span,
                    min: if stream.is_at(TypeTokenKind::Minus)? {
                        let minus = stream.consume()?.span;
                        let token = stream.eat(TypeTokenKind::LiteralInteger)?;
                        let value = parse_literal_integer(token.value).unwrap_or_else(|| {
                            unreachable!("lexer generated invalid integer `{}`; this should never happen.", token.value)
                        });

                        IntOrKeyword::NegativeInt {
                            minus,
                            int: LiteralIntType { span: token.span, value, raw: token.value },
                        }
                    } else if stream.is_at(TypeTokenKind::LiteralInteger)? {
                        let token = stream.consume()?;
                        let value = parse_literal_integer(token.value).unwrap_or_else(|| {
                            unreachable!("lexer generated invalid integer `{}`; this should never happen.", token.value)
                        });

                        IntOrKeyword::Int(LiteralIntType { span: token.span, value, raw: token.value })
                    } else {
                        IntOrKeyword::Keyword(Keyword::from(stream.eat(TypeTokenKind::Min)?))
                    },
                    comma: stream.eat(TypeTokenKind::Comma)?.span,
                    max: if stream.is_at(TypeTokenKind::Minus)? {
                        let minus = stream.consume()?.span;
                        let token = stream.eat(TypeTokenKind::LiteralInteger)?;
                        let value = parse_literal_integer(token.value).unwrap_or_else(|| {
                            unreachable!("lexer generated invalid integer `{}`; this should never happen.", token.value)
                        });

                        IntOrKeyword::NegativeInt {
                            minus,
                            int: LiteralIntType { span: token.span, value, raw: token.value },
                        }
                    } else if stream.is_at(TypeTokenKind::LiteralInteger)? {
                        let token = stream.consume()?;
                        let value = parse_literal_integer(token.value).unwrap_or_else(|| {
                            unreachable!("lexer generated invalid integer `{}`; this should never happen.", token.value)
                        });

                        IntOrKeyword::Int(LiteralIntType { span: token.span, value, raw: token.value })
                    } else {
                        IntOrKeyword::Keyword(Keyword::from(stream.eat(TypeTokenKind::Max)?))
                    },
                    greater_than: stream.eat(TypeTokenKind::GreaterThan)?.span,
                })
            } else {
                Type::Int(keyword)
            }
        }
        TypeTokenKind::PositiveInt => Type::PositiveInt(Keyword::from(stream.consume()?)),
        TypeTokenKind::NegativeInt => Type::NegativeInt(Keyword::from(stream.consume()?)),
        TypeTokenKind::NonPositiveInt => Type::NonPositiveInt(Keyword::from(stream.consume()?)),
        TypeTokenKind::NonNegativeInt => Type::NonNegativeInt(Keyword::from(stream.consume()?)),
        TypeTokenKind::String => Type::String(Keyword::from(stream.consume()?)),
        TypeTokenKind::NumericString => Type::NumericString(Keyword::from(stream.consume()?)),
        TypeTokenKind::NonEmptyString => Type::NonEmptyString(Keyword::from(stream.consume()?)),
        TypeTokenKind::NonEmptyLowercaseString => Type::NonEmptyLowercaseString(Keyword::from(stream.consume()?)),
        TypeTokenKind::LowercaseString => Type::LowercaseString(Keyword::from(stream.consume()?)),
        TypeTokenKind::TruthyString => Type::TruthyString(Keyword::from(stream.consume()?)),
        TypeTokenKind::NonFalsyString => Type::NonFalsyString(Keyword::from(stream.consume()?)),
        TypeTokenKind::Object => parse_object_type(stream)?,
        TypeTokenKind::NoReturn | TypeTokenKind::NeverReturn | TypeTokenKind::NeverReturns | TypeTokenKind::Nothing => {
            Type::Never(Keyword::from(stream.consume()?))
        }
        TypeTokenKind::KeyOf => Type::KeyOf(KeyOfType {
            keyword: Keyword::from(stream.consume()?),
            parameter: parse_single_generic_parameter(stream)?,
        }),
        TypeTokenKind::ValueOf => Type::ValueOf(ValueOfType {
            keyword: Keyword::from(stream.consume()?),
            parameter: parse_single_generic_parameter(stream)?,
        }),
        TypeTokenKind::Scalar => Type::Scalar(Keyword::from(stream.consume()?)),
        TypeTokenKind::Numeric => Type::Numeric(Keyword::from(stream.consume()?)),
        TypeTokenKind::ArrayKey => Type::ArrayKey(Keyword::from(stream.consume()?)),
        TypeTokenKind::StringableObject => Type::StringableObject(Keyword::from(stream.consume()?)),
        TypeTokenKind::UnspecifiedLiteralInt => Type::UnspecifiedLiteralInt(Keyword::from(stream.consume()?)),
        TypeTokenKind::UnspecifiedLiteralString => Type::UnspecifiedLiteralString(Keyword::from(stream.consume()?)),
        TypeTokenKind::NonEmptyUnspecifiedLiteralString => {
            Type::NonEmptyUnspecifiedLiteralString(Keyword::from(stream.consume()?))
        }
        TypeTokenKind::PropertiesOf => Type::PropertiesOf(PropertiesOfType {
            filter: PropertiesOfFilter::All,
            keyword: Keyword::from(stream.consume()?),
            parameter: parse_single_generic_parameter(stream)?,
        }),
        TypeTokenKind::PublicPropertiesOf => Type::PropertiesOf(PropertiesOfType {
            filter: PropertiesOfFilter::Public,
            keyword: Keyword::from(stream.consume()?),
            parameter: parse_single_generic_parameter(stream)?,
        }),
        TypeTokenKind::PrivatePropertiesOf => Type::PropertiesOf(PropertiesOfType {
            filter: PropertiesOfFilter::Private,
            keyword: Keyword::from(stream.consume()?),
            parameter: parse_single_generic_parameter(stream)?,
        }),
        TypeTokenKind::ProtectedPropertiesOf => Type::PropertiesOf(PropertiesOfType {
            filter: PropertiesOfFilter::Protected,
            keyword: Keyword::from(stream.consume()?),
            parameter: parse_single_generic_parameter(stream)?,
        }),
        TypeTokenKind::Array
        | TypeTokenKind::NonEmptyArray
        | TypeTokenKind::AssociativeArray
        | TypeTokenKind::List
        | TypeTokenKind::NonEmptyList => parse_array_like_type(stream)?,
        TypeTokenKind::Iterable => Type::Iterable(IterableType {
            keyword: Keyword::from(stream.consume()?),
            parameters: parse_generic_parameters_or_none(stream)?,
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
            Type::Negated(NegatedType { minus: stream.consume()?.span, number: parse_literal_number_type(stream)? })
        }
        TypeTokenKind::Plus => {
            Type::Posited(PositedType { plus: stream.consume()?.span, number: parse_literal_number_type(stream)? })
        }
        TypeTokenKind::EnumString => Type::EnumString(EnumStringType {
            keyword: Keyword::from(stream.consume()?),
            parameter: parse_single_generic_parameter_or_none(stream)?,
        }),
        TypeTokenKind::TraitString => Type::TraitString(TraitStringType {
            keyword: Keyword::from(stream.consume()?),
            parameter: parse_single_generic_parameter_or_none(stream)?,
        }),
        TypeTokenKind::ClassString => Type::ClassString(ClassStringType {
            keyword: Keyword::from(stream.consume()?),
            parameter: parse_single_generic_parameter_or_none(stream)?,
        }),
        TypeTokenKind::InterfaceString => Type::InterfaceString(InterfaceStringType {
            keyword: Keyword::from(stream.consume()?),
            parameter: parse_single_generic_parameter_or_none(stream)?,
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
                let double_colon = stream.consume()?.span;

                if stream.is_at(TypeTokenKind::Asterisk)? {
                    let asterisk = stream.consume()?.span;

                    Type::MemberReference(MemberReferenceType {
                        class: identifier,
                        double_colon,
                        member: if stream.is_at(TypeTokenKind::Identifier)? {
                            MemberReferenceSelector::EndsWith(
                                asterisk,
                                Identifier::from(stream.eat(TypeTokenKind::Identifier)?),
                            )
                        } else {
                            MemberReferenceSelector::Wildcard(asterisk)
                        },
                    })
                } else {
                    let identifier = Identifier::from(stream.eat(TypeTokenKind::Identifier)?);

                    Type::MemberReference(MemberReferenceType {
                        class: identifier,
                        double_colon,
                        member: if stream.is_at(TypeTokenKind::Asterisk)? {
                            MemberReferenceSelector::StartsWith(identifier, stream.consume()?.span)
                        } else {
                            MemberReferenceSelector::Identifier(identifier)
                        },
                    })
                }
            } else {
                Type::Reference(ReferenceType { identifier, parameters: parse_generic_parameters_or_none(stream)? })
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
                    let double_colon = stream.consume()?.span;

                    if stream.is_at(TypeTokenKind::Asterisk)? {
                        let asterisk = stream.consume()?.span;

                        Type::MemberReference(MemberReferenceType {
                            class: identifier,
                            double_colon,
                            member: if stream.is_at(TypeTokenKind::Identifier)? {
                                MemberReferenceSelector::EndsWith(
                                    asterisk,
                                    Identifier::from(stream.eat(TypeTokenKind::Identifier)?),
                                )
                            } else {
                                MemberReferenceSelector::Wildcard(asterisk)
                            },
                        })
                    } else {
                        let member_identifier = Identifier::from(stream.eat(TypeTokenKind::Identifier)?);

                        Type::MemberReference(MemberReferenceType {
                            class: identifier,
                            double_colon,
                            member: if stream.is_at(TypeTokenKind::Asterisk)? {
                                MemberReferenceSelector::StartsWith(member_identifier, stream.consume()?.span)
                            } else {
                                MemberReferenceSelector::Identifier(member_identifier)
                            },
                        })
                    }
                } else {
                    Type::Reference(ReferenceType { identifier, parameters: parse_generic_parameters_or_none(stream)? })
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
                    let double_colon = stream.consume()?.span;

                    Type::MemberReference(MemberReferenceType {
                        class: identifier,
                        double_colon,
                        member: if stream.is_at(TypeTokenKind::Asterisk)? {
                            let asterisk = stream.consume()?.span;

                            if stream.is_at(TypeTokenKind::Identifier)? {
                                MemberReferenceSelector::EndsWith(
                                    asterisk,
                                    Identifier::from(stream.eat(TypeTokenKind::Identifier)?),
                                )
                            } else {
                                MemberReferenceSelector::Wildcard(asterisk)
                            }
                        } else {
                            let identifier = Identifier::from(stream.eat(TypeTokenKind::Identifier)?);

                            if stream.is_at(TypeTokenKind::Asterisk)? {
                                MemberReferenceSelector::StartsWith(identifier, stream.consume()?.span)
                            } else {
                                MemberReferenceSelector::Identifier(identifier)
                            }
                        },
                    })
                } else {
                    Type::Reference(ReferenceType { identifier, parameters: parse_generic_parameters_or_none(stream)? })
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

    loop {
        let is_inner_nullable = matches!(inner, Type::Nullable(_));

        inner = match stream.lookahead(0)?.map(|t| t.kind) {
            Some(TypeTokenKind::Pipe) if !is_inner_nullable => Type::Union(UnionType {
                left: Box::new(inner),
                pipe: stream.consume()?.span,
                right: Box::new(parse_type(stream)?),
            }),
            Some(TypeTokenKind::Ampersand) if !is_inner_nullable => Type::Intersection(IntersectionType {
                left: Box::new(inner),
                ampersand: stream.consume()?.span,
                right: Box::new(parse_type(stream)?),
            }),
            Some(TypeTokenKind::Is) if !is_inner_nullable => Type::Conditional(ConditionalType {
                subject: Box::new(inner),
                is: Keyword::from(stream.consume()?),
                not: if stream.is_at(TypeTokenKind::Not)? { Some(Keyword::from(stream.consume()?)) } else { None },
                target: Box::new(parse_type(stream)?),
                question_mark: stream.eat(TypeTokenKind::Question)?.span,
                then: Box::new(parse_type(stream)?),
                colon: stream.eat(TypeTokenKind::Colon)?.span,
                otherwise: Box::new(parse_type(stream)?),
            }),
            Some(TypeTokenKind::LeftBracket) => {
                let left_bracket = stream.consume()?.span;

                if stream.is_at(TypeTokenKind::RightBracket)? {
                    Type::Slice(SliceType {
                        inner: Box::new(inner),
                        left_bracket,
                        right_bracket: stream.consume()?.span,
                    })
                } else {
                    Type::IndexAccess(IndexAccessType {
                        target: Box::new(inner),
                        left_bracket,
                        index: Box::new(parse_type(stream)?),
                        right_bracket: stream.eat(TypeTokenKind::RightBracket)?.span,
                    })
                }
            }
            _ => {
                return Ok(inner);
            }
        };
    }
}

pub fn parse_literal_number_type<'input>(
    stream: &mut TypeTokenStream<'input>,
) -> Result<LiteralIntOrFloatType<'input>, ParseError> {
    let next = stream.peek()?;

    match next.kind {
        TypeTokenKind::LiteralInteger => {
            let token = stream.consume()?;
            let value = parse_literal_integer(token.value).unwrap_or_else(|| {
                unreachable!("lexer generated invalid integer `{}`; this should never happen.", token.value)
            });

            Ok(LiteralIntOrFloatType::Int(LiteralIntType { span: token.span, value, raw: token.value }))
        }
        TypeTokenKind::LiteralFloat => {
            let token = stream.consume()?;
            let value = parse_literal_float(token.value).unwrap_or_else(|| {
                unreachable!("lexer generated invalid float `{}`; this should never happen.", token.value)
            });

            Ok(LiteralIntOrFloatType::Float(LiteralFloatType {
                span: token.span,
                value: OrderedFloat(value),
                raw: token.value,
            }))
        }
        _ => Err(ParseError::UnexpectedToken(vec![], next.kind, next.span)),
    }
}
