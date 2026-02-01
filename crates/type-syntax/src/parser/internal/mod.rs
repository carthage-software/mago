use ordered_float::OrderedFloat;

use mago_database::file::HasFileId;
use mago_syntax_core::utils::parse_literal_float;
use mago_syntax_core::utils::parse_literal_integer;

use crate::ast::AliasName;
use crate::ast::AliasReferenceType;
use crate::ast::CallableType;
use crate::ast::CallableTypeKind;
use crate::ast::ClassStringType;
use crate::ast::ConditionalType;
use crate::ast::EnumStringType;
use crate::ast::Identifier;
use crate::ast::IndexAccessType;
use crate::ast::IntMaskOfType;
use crate::ast::IntMaskType;
use crate::ast::IntOrKeyword;
use crate::ast::IntRangeType;
use crate::ast::InterfaceStringType;
use crate::ast::IntersectionType;
use crate::ast::IterableType;
use crate::ast::KeyOfType;
use crate::ast::Keyword;
use crate::ast::LiteralFloatType;
use crate::ast::LiteralIntOrFloatType;
use crate::ast::LiteralIntType;
use crate::ast::LiteralStringType;
use crate::ast::MemberReferenceSelector;
use crate::ast::MemberReferenceType;
use crate::ast::NegatedType;
use crate::ast::NullableType;
use crate::ast::ParenthesizedType;
use crate::ast::PositedType;
use crate::ast::PropertiesOfFilter;
use crate::ast::PropertiesOfType;
use crate::ast::ReferenceType;
use crate::ast::SliceType;
use crate::ast::TraitStringType;
use crate::ast::Type;
use crate::ast::UnionType;
use crate::ast::ValueOfType;
use crate::ast::VariableType;
use crate::error::ParseError;
use crate::parser::internal::array_like::parse_array_like_type;
use crate::parser::internal::callable::parse_callable_type_specifications;
use crate::parser::internal::callable::parse_optional_callable_type_specifications;
use crate::parser::internal::generic::parse_generic_parameters;
use crate::parser::internal::generic::parse_generic_parameters_or_none;
use crate::parser::internal::generic::parse_single_generic_parameter;
use crate::parser::internal::generic::parse_single_generic_parameter_or_none;
use crate::parser::internal::object::parse_object_type;
use crate::parser::internal::stream::TypeTokenStream;
use crate::token::TypePrecedence;
use crate::token::TypeTokenKind;

pub mod array_like;
pub mod callable;
pub mod generic;
pub mod object;
pub mod stream;

/// Parses a complete type expression, including unions, intersections, and conditionals.
#[inline]
pub fn parse_type<'input>(stream: &mut TypeTokenStream<'input>) -> Result<Type<'input>, ParseError> {
    parse_type_with_precedence(stream, TypePrecedence::Lowest)
}

/// Parses a type expression with the given minimum precedence.
///
/// This function controls what operators are consumed based on precedence:
/// - `Lowest`: Parses everything including unions, intersections, and conditionals
/// - `Callable`: Used for callable return types; stops before `|`, `&`, and `is`
///
/// For example, `Closure(): int|string` with:
/// - `Lowest` precedence: parses as `Union(Closure(): int, string)` (correct PHPStan/Psalm behavior)
/// - `Callable` precedence: parses only `int`, leaving `|string` for the parent to handle
pub fn parse_type_with_precedence<'input>(
    stream: &mut TypeTokenStream<'input>,
    min_precedence: TypePrecedence,
) -> Result<Type<'input>, ParseError> {
    let mut inner = parse_primary_type(stream)?;

    loop {
        let is_inner_nullable = matches!(inner, Type::Nullable(_));

        inner = match stream.lookahead(0)?.map(|t| t.kind) {
            // Union types: T|U
            Some(TypeTokenKind::Pipe) if !is_inner_nullable && min_precedence <= TypePrecedence::Union => {
                Type::Union(UnionType {
                    left: Box::new(inner),
                    pipe: stream.consume()?.span_for(stream.file_id()),
                    right: Box::new(parse_type_with_precedence(stream, TypePrecedence::Union)?),
                })
            }
            // Intersection types: T&U
            Some(TypeTokenKind::Ampersand) if !is_inner_nullable && min_precedence <= TypePrecedence::Intersection => {
                Type::Intersection(IntersectionType {
                    left: Box::new(inner),
                    ampersand: stream.consume()?.span_for(stream.file_id()),
                    right: Box::new(parse_type_with_precedence(stream, TypePrecedence::Intersection)?),
                })
            }
            // Conditional types: T is U ? V : W
            Some(TypeTokenKind::Is) if !is_inner_nullable && min_precedence <= TypePrecedence::Conditional => {
                Type::Conditional(ConditionalType {
                    subject: Box::new(inner),
                    is: Keyword::from_token(stream.consume()?, stream.file_id()),
                    not: if stream.is_at(TypeTokenKind::Not)? {
                        Some(Keyword::from_token(stream.consume()?, stream.file_id()))
                    } else {
                        None
                    },
                    target: Box::new(parse_type_with_precedence(stream, TypePrecedence::Conditional)?),
                    question_mark: stream.eat(TypeTokenKind::Question)?.span_for(stream.file_id()),
                    then: Box::new(parse_type_with_precedence(stream, TypePrecedence::Conditional)?),
                    colon: stream.eat(TypeTokenKind::Colon)?.span_for(stream.file_id()),
                    otherwise: Box::new(parse_type_with_precedence(stream, TypePrecedence::Conditional)?),
                })
            }
            // Postfix operations: T[], T[K]
            Some(TypeTokenKind::LeftBracket) if min_precedence <= TypePrecedence::Postfix => {
                let left_bracket = stream.consume()?.span_for(stream.file_id());

                if stream.is_at(TypeTokenKind::RightBracket)? {
                    Type::Slice(SliceType {
                        inner: Box::new(inner),
                        left_bracket,
                        right_bracket: stream.consume()?.span_for(stream.file_id()),
                    })
                } else {
                    Type::IndexAccess(IndexAccessType {
                        target: Box::new(inner),
                        left_bracket,
                        index: Box::new(parse_type(stream)?),
                        right_bracket: stream.eat(TypeTokenKind::RightBracket)?.span_for(stream.file_id()),
                    })
                }
            }
            _ => {
                return Ok(inner);
            }
        };
    }
}

/// Parses a primary (atomic) type without consuming any infix operators.
#[inline]
fn parse_primary_type<'input>(stream: &mut TypeTokenStream<'input>) -> Result<Type<'input>, ParseError> {
    let next = stream.peek()?;
    let inner = match next.kind {
        TypeTokenKind::Variable => Type::Variable(VariableType::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::Question => Type::Nullable(NullableType {
            question_mark: stream.consume()?.span_for(stream.file_id()),
            inner: Box::new(parse_type(stream)?),
        }),
        TypeTokenKind::LeftParenthesis => Type::Parenthesized(ParenthesizedType {
            left_parenthesis: stream.consume()?.span_for(stream.file_id()),
            inner: Box::new(parse_type(stream)?),
            right_parenthesis: stream.eat(TypeTokenKind::RightParenthesis)?.span_for(stream.file_id()),
        }),
        TypeTokenKind::Mixed => Type::Mixed(Keyword::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::NonEmptyMixed => Type::NonEmptyMixed(Keyword::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::Null => Type::Null(Keyword::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::Void => Type::Void(Keyword::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::Never => Type::Never(Keyword::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::Resource => Type::Resource(Keyword::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::ClosedResource => Type::ClosedResource(Keyword::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::OpenResource => Type::OpenResource(Keyword::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::True => Type::True(Keyword::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::False => Type::False(Keyword::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::Bool | TypeTokenKind::Boolean => {
            Type::Bool(Keyword::from_token(stream.consume()?, stream.file_id()))
        }
        TypeTokenKind::Float | TypeTokenKind::Real | TypeTokenKind::Double => {
            Type::Float(Keyword::from_token(stream.consume()?, stream.file_id()))
        }
        TypeTokenKind::Int | TypeTokenKind::Integer => {
            let keyword = Keyword::from_token(stream.consume()?, stream.file_id());

            if stream.is_at(TypeTokenKind::LessThan)? {
                Type::IntRange(IntRangeType {
                    keyword,
                    less_than: stream.consume()?.span_for(stream.file_id()),
                    min: if stream.is_at(TypeTokenKind::Minus)? {
                        let minus = stream.consume()?.span_for(stream.file_id());
                        let token = stream.eat(TypeTokenKind::LiteralInteger)?;
                        let value = parse_literal_integer(token.value).unwrap_or_else(|| {
                            unreachable!("lexer generated invalid integer `{}`; this should never happen.", token.value)
                        });

                        IntOrKeyword::NegativeInt {
                            minus,
                            int: LiteralIntType { span: token.span_for(stream.file_id()), value, raw: token.value },
                        }
                    } else if stream.is_at(TypeTokenKind::LiteralInteger)? {
                        let token = stream.consume()?;
                        let value = parse_literal_integer(token.value).unwrap_or_else(|| {
                            unreachable!("lexer generated invalid integer `{}`; this should never happen.", token.value)
                        });

                        IntOrKeyword::Int(LiteralIntType {
                            span: token.span_for(stream.file_id()),
                            value,
                            raw: token.value,
                        })
                    } else {
                        IntOrKeyword::Keyword(Keyword::from_token(stream.eat(TypeTokenKind::Min)?, stream.file_id()))
                    },
                    comma: stream.eat(TypeTokenKind::Comma)?.span_for(stream.file_id()),
                    max: if stream.is_at(TypeTokenKind::Minus)? {
                        let minus = stream.consume()?.span_for(stream.file_id());
                        let token = stream.eat(TypeTokenKind::LiteralInteger)?;
                        let value = parse_literal_integer(token.value).unwrap_or_else(|| {
                            unreachable!("lexer generated invalid integer `{}`; this should never happen.", token.value)
                        });

                        IntOrKeyword::NegativeInt {
                            minus,
                            int: LiteralIntType { span: token.span_for(stream.file_id()), value, raw: token.value },
                        }
                    } else if stream.is_at(TypeTokenKind::LiteralInteger)? {
                        let token = stream.consume()?;
                        let value = parse_literal_integer(token.value).unwrap_or_else(|| {
                            unreachable!("lexer generated invalid integer `{}`; this should never happen.", token.value)
                        });

                        IntOrKeyword::Int(LiteralIntType {
                            span: token.span_for(stream.file_id()),
                            value,
                            raw: token.value,
                        })
                    } else {
                        IntOrKeyword::Keyword(Keyword::from_token(stream.eat(TypeTokenKind::Max)?, stream.file_id()))
                    },
                    greater_than: stream.eat(TypeTokenKind::GreaterThan)?.span_for(stream.file_id()),
                })
            } else {
                Type::Int(keyword)
            }
        }
        TypeTokenKind::PositiveInt => Type::PositiveInt(Keyword::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::NegativeInt => Type::NegativeInt(Keyword::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::NonPositiveInt => Type::NonPositiveInt(Keyword::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::NonNegativeInt => Type::NonNegativeInt(Keyword::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::String => Type::String(Keyword::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::NumericString => Type::NumericString(Keyword::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::NonEmptyString => Type::NonEmptyString(Keyword::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::NonEmptyLowercaseString => {
            Type::NonEmptyLowercaseString(Keyword::from_token(stream.consume()?, stream.file_id()))
        }
        TypeTokenKind::LowercaseString => {
            Type::LowercaseString(Keyword::from_token(stream.consume()?, stream.file_id()))
        }
        TypeTokenKind::TruthyString => Type::TruthyString(Keyword::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::NonFalsyString => Type::NonFalsyString(Keyword::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::Object => parse_object_type(stream)?,
        TypeTokenKind::NoReturn | TypeTokenKind::NeverReturn | TypeTokenKind::NeverReturns | TypeTokenKind::Nothing => {
            Type::Never(Keyword::from_token(stream.consume()?, stream.file_id()))
        }
        TypeTokenKind::KeyOf => Type::KeyOf(KeyOfType {
            keyword: Keyword::from_token(stream.consume()?, stream.file_id()),
            parameter: parse_single_generic_parameter(stream)?,
        }),
        TypeTokenKind::ValueOf => Type::ValueOf(ValueOfType {
            keyword: Keyword::from_token(stream.consume()?, stream.file_id()),
            parameter: parse_single_generic_parameter(stream)?,
        }),
        TypeTokenKind::IntMask => Type::IntMask(IntMaskType {
            keyword: Keyword::from_token(stream.consume()?, stream.file_id()),
            parameters: parse_generic_parameters(stream)?,
        }),
        TypeTokenKind::IntMaskOf => Type::IntMaskOf(IntMaskOfType {
            keyword: Keyword::from_token(stream.consume()?, stream.file_id()),
            parameter: parse_single_generic_parameter(stream)?,
        }),
        TypeTokenKind::Scalar => Type::Scalar(Keyword::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::Numeric => Type::Numeric(Keyword::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::ArrayKey => Type::ArrayKey(Keyword::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::StringableObject => {
            Type::StringableObject(Keyword::from_token(stream.consume()?, stream.file_id()))
        }
        TypeTokenKind::UnspecifiedLiteralInt => {
            Type::UnspecifiedLiteralInt(Keyword::from_token(stream.consume()?, stream.file_id()))
        }
        TypeTokenKind::UnspecifiedLiteralString => {
            Type::UnspecifiedLiteralString(Keyword::from_token(stream.consume()?, stream.file_id()))
        }
        TypeTokenKind::UnspecifiedLiteralFloat => {
            Type::UnspecifiedLiteralFloat(Keyword::from_token(stream.consume()?, stream.file_id()))
        }
        TypeTokenKind::NonEmptyUnspecifiedLiteralString => {
            Type::NonEmptyUnspecifiedLiteralString(Keyword::from_token(stream.consume()?, stream.file_id()))
        }
        TypeTokenKind::PropertiesOf => Type::PropertiesOf(PropertiesOfType {
            filter: PropertiesOfFilter::All,
            keyword: Keyword::from_token(stream.consume()?, stream.file_id()),
            parameter: parse_single_generic_parameter(stream)?,
        }),
        TypeTokenKind::PublicPropertiesOf => Type::PropertiesOf(PropertiesOfType {
            filter: PropertiesOfFilter::Public,
            keyword: Keyword::from_token(stream.consume()?, stream.file_id()),
            parameter: parse_single_generic_parameter(stream)?,
        }),
        TypeTokenKind::PrivatePropertiesOf => Type::PropertiesOf(PropertiesOfType {
            filter: PropertiesOfFilter::Private,
            keyword: Keyword::from_token(stream.consume()?, stream.file_id()),
            parameter: parse_single_generic_parameter(stream)?,
        }),
        TypeTokenKind::ProtectedPropertiesOf => Type::PropertiesOf(PropertiesOfType {
            filter: PropertiesOfFilter::Protected,
            keyword: Keyword::from_token(stream.consume()?, stream.file_id()),
            parameter: parse_single_generic_parameter(stream)?,
        }),
        TypeTokenKind::Array
        | TypeTokenKind::NonEmptyArray
        | TypeTokenKind::AssociativeArray
        | TypeTokenKind::List
        | TypeTokenKind::NonEmptyList => parse_array_like_type(stream)?,
        TypeTokenKind::Iterable => Type::Iterable(IterableType {
            keyword: Keyword::from_token(stream.consume()?, stream.file_id()),
            parameters: parse_generic_parameters_or_none(stream)?,
        }),
        TypeTokenKind::LiteralFloat => {
            let token = stream.consume()?;
            let value = parse_literal_float(token.value).unwrap_or_else(|| {
                unreachable!("lexer generated invalid float `{}`; this should never happen.", token.value)
            });

            Type::LiteralFloat(LiteralFloatType {
                span: token.span_for(stream.file_id()),
                value: OrderedFloat(value),
                raw: token.value,
            })
        }
        TypeTokenKind::LiteralInteger => {
            let token = stream.consume()?;
            let value = parse_literal_integer(token.value).unwrap_or_else(|| {
                unreachable!("lexer generated invalid integer `{}`; this should never happen.", token.value)
            });

            Type::LiteralInt(LiteralIntType { span: token.span_for(stream.file_id()), value, raw: token.value })
        }
        TypeTokenKind::LiteralString => {
            let token = stream.consume()?;
            let value = &token.value[1..token.value.len() - 1];

            Type::LiteralString(LiteralStringType { span: token.span_for(stream.file_id()), value, raw: token.value })
        }
        TypeTokenKind::Minus => Type::Negated(NegatedType {
            minus: stream.consume()?.span_for(stream.file_id()),
            number: parse_literal_number_type(stream)?,
        }),
        TypeTokenKind::Plus => Type::Posited(PositedType {
            plus: stream.consume()?.span_for(stream.file_id()),
            number: parse_literal_number_type(stream)?,
        }),
        TypeTokenKind::EnumString => Type::EnumString(EnumStringType {
            keyword: Keyword::from_token(stream.consume()?, stream.file_id()),
            parameter: parse_single_generic_parameter_or_none(stream)?,
        }),
        TypeTokenKind::TraitString => Type::TraitString(TraitStringType {
            keyword: Keyword::from_token(stream.consume()?, stream.file_id()),
            parameter: parse_single_generic_parameter_or_none(stream)?,
        }),
        TypeTokenKind::ClassString => Type::ClassString(ClassStringType {
            keyword: Keyword::from_token(stream.consume()?, stream.file_id()),
            parameter: parse_single_generic_parameter_or_none(stream)?,
        }),
        TypeTokenKind::InterfaceString => Type::InterfaceString(InterfaceStringType {
            keyword: Keyword::from_token(stream.consume()?, stream.file_id()),
            parameter: parse_single_generic_parameter_or_none(stream)?,
        }),
        TypeTokenKind::Callable => Type::Callable(CallableType {
            kind: CallableTypeKind::Callable,
            keyword: Keyword::from_token(stream.consume()?, stream.file_id()),
            specification: parse_optional_callable_type_specifications(stream)?,
        }),
        TypeTokenKind::PureCallable => Type::Callable(CallableType {
            kind: CallableTypeKind::PureCallable,
            keyword: Keyword::from_token(stream.consume()?, stream.file_id()),
            specification: parse_optional_callable_type_specifications(stream)?,
        }),
        TypeTokenKind::PureClosure => Type::Callable(CallableType {
            kind: CallableTypeKind::PureClosure,
            keyword: Keyword::from_token(stream.consume()?, stream.file_id()),
            specification: parse_optional_callable_type_specifications(stream)?,
        }),
        TypeTokenKind::QualifiedIdentifier => {
            let identifier = Identifier::from_token(stream.consume()?, stream.file_id());
            if stream.is_at(TypeTokenKind::ColonColon)? {
                let double_colon = stream.consume()?.span_for(stream.file_id());

                if stream.is_at(TypeTokenKind::Asterisk)? {
                    let asterisk = stream.consume()?.span_for(stream.file_id());

                    Type::MemberReference(MemberReferenceType {
                        class: identifier,
                        double_colon,
                        member: if stream.is_at(TypeTokenKind::Identifier)? {
                            MemberReferenceSelector::EndsWith(
                                asterisk,
                                Identifier::from_token(stream.eat(TypeTokenKind::Identifier)?, stream.file_id()),
                            )
                        } else {
                            MemberReferenceSelector::Wildcard(asterisk)
                        },
                    })
                } else {
                    let identifier = Identifier::from_token(stream.eat(TypeTokenKind::Identifier)?, stream.file_id());

                    Type::MemberReference(MemberReferenceType {
                        class: identifier,
                        double_colon,
                        member: if stream.is_at(TypeTokenKind::Asterisk)? {
                            MemberReferenceSelector::StartsWith(
                                identifier,
                                stream.consume()?.span_for(stream.file_id()),
                            )
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
                    keyword: Keyword::from_token(stream.consume()?, stream.file_id()),
                    specification: Some(parse_callable_type_specifications(stream)?),
                })
            } else {
                let identifier = Identifier::from_token(stream.consume()?, stream.file_id());
                if stream.is_at(TypeTokenKind::ColonColon)? {
                    let double_colon = stream.consume()?.span_for(stream.file_id());

                    if stream.is_at(TypeTokenKind::Asterisk)? {
                        let asterisk = stream.consume()?.span_for(stream.file_id());

                        Type::MemberReference(MemberReferenceType {
                            class: identifier,
                            double_colon,
                            member: if stream.is_at(TypeTokenKind::Identifier)? {
                                MemberReferenceSelector::EndsWith(
                                    asterisk,
                                    Identifier::from_token(stream.eat(TypeTokenKind::Identifier)?, stream.file_id()),
                                )
                            } else {
                                MemberReferenceSelector::Wildcard(asterisk)
                            },
                        })
                    } else {
                        let member_identifier =
                            Identifier::from_token(stream.eat(TypeTokenKind::Identifier)?, stream.file_id());

                        Type::MemberReference(MemberReferenceType {
                            class: identifier,
                            double_colon,
                            member: if stream.is_at(TypeTokenKind::Asterisk)? {
                                MemberReferenceSelector::StartsWith(
                                    member_identifier,
                                    stream.consume()?.span_for(stream.file_id()),
                                )
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
                    keyword: Keyword::from_token(stream.consume()?, stream.file_id()),
                    specification: Some(parse_callable_type_specifications(stream)?),
                })
            } else {
                let identifier = Identifier::from_token(stream.consume()?, stream.file_id());

                if stream.is_at(TypeTokenKind::ColonColon)? {
                    let double_colon = stream.consume()?.span_for(stream.file_id());

                    Type::MemberReference(MemberReferenceType {
                        class: identifier,
                        double_colon,
                        member: if stream.is_at(TypeTokenKind::Asterisk)? {
                            let asterisk = stream.consume()?.span_for(stream.file_id());

                            if stream.is_at(TypeTokenKind::Identifier)? {
                                MemberReferenceSelector::EndsWith(
                                    asterisk,
                                    Identifier::from_token(stream.eat(TypeTokenKind::Identifier)?, stream.file_id()),
                                )
                            } else {
                                MemberReferenceSelector::Wildcard(asterisk)
                            }
                        } else {
                            let identifier =
                                Identifier::from_token(stream.eat(TypeTokenKind::Identifier)?, stream.file_id());

                            if stream.is_at(TypeTokenKind::Asterisk)? {
                                MemberReferenceSelector::StartsWith(
                                    identifier,
                                    stream.consume()?.span_for(stream.file_id()),
                                )
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
        TypeTokenKind::Exclamation => {
            let exclamation = stream.consume()?.span_for(stream.file_id());
            let next = stream.peek()?;

            // Parse the class identifier (can be fully qualified, qualified, or local)
            let class = match next.kind {
                TypeTokenKind::Identifier
                | TypeTokenKind::QualifiedIdentifier
                | TypeTokenKind::FullyQualifiedIdentifier => {
                    Identifier::from_token(stream.consume()?, stream.file_id())
                }
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        vec![
                            TypeTokenKind::Identifier,
                            TypeTokenKind::QualifiedIdentifier,
                            TypeTokenKind::FullyQualifiedIdentifier,
                        ],
                        next.kind,
                        next.span_for(stream.file_id()),
                    ));
                }
            };

            // Parse `::`
            let double_colon = stream.eat(TypeTokenKind::ColonColon)?.span_for(stream.file_id());

            // Parse the alias name (can be identifier or keyword without `-`)
            let next = stream.peek()?;
            let alias = if next.kind.is_keyword() && !next.value.contains('-') {
                AliasName::Keyword(Keyword::from_token(stream.consume()?, stream.file_id()))
            } else if next.kind == TypeTokenKind::Identifier {
                AliasName::Identifier(Identifier::from_token(stream.consume()?, stream.file_id()))
            } else {
                return Err(ParseError::UnexpectedToken(
                    vec![TypeTokenKind::Identifier],
                    next.kind,
                    next.span_for(stream.file_id()),
                ));
            };

            Type::AliasReference(AliasReferenceType { exclamation, class, double_colon, alias })
        }
        TypeTokenKind::Whitespace | TypeTokenKind::SingleLineComment => {
            unreachable!("trivia tokens are skipped by the stream.")
        }
        TypeTokenKind::PartialLiteralString => {
            return Err(ParseError::UnclosedLiteralString(next.span_for(stream.file_id())));
        }
        _ => {
            return Err(ParseError::UnexpectedToken(vec![], next.kind, next.span_for(stream.file_id())));
        }
    };

    Ok(inner)
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

            Ok(LiteralIntOrFloatType::Int(LiteralIntType {
                span: token.span_for(stream.file_id()),
                value,
                raw: token.value,
            }))
        }
        TypeTokenKind::LiteralFloat => {
            let token = stream.consume()?;
            let value = parse_literal_float(token.value).unwrap_or_else(|| {
                unreachable!("lexer generated invalid float `{}`; this should never happen.", token.value)
            });

            Ok(LiteralIntOrFloatType::Float(LiteralFloatType {
                span: token.span_for(stream.file_id()),
                value: OrderedFloat(value),
                raw: token.value,
            }))
        }
        _ => Err(ParseError::UnexpectedToken(vec![], next.kind, next.span_for(stream.file_id()))),
    }
}
