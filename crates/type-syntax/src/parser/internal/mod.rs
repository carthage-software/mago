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
use crate::ast::GlobalWildcardSelector;
use crate::ast::GlobalWildcardType;
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
use crate::ast::LiteralFloatType;
use crate::ast::LiteralIntOrFloatType;
use crate::ast::LiteralIntType;
use crate::ast::LiteralStringType;
use crate::ast::MemberReferenceSelector;
use crate::ast::MemberReferenceType;
use crate::ast::NegatedType;
use crate::ast::NewType;
use crate::ast::NullableType;
use crate::ast::ParenthesizedType;
use crate::ast::PositedType;
use crate::ast::PropertiesOfFilter;
use crate::ast::PropertiesOfType;
use crate::ast::ReferenceType;
use crate::ast::SliceType;
use crate::ast::TemplateTypeType;
use crate::ast::TrailingPipeType;
use crate::ast::TraitStringType;
use crate::ast::Type;
use crate::ast::UnionType;
use crate::ast::ValueOfType;
use crate::ast::VariableType;
use crate::ast::WildcardKind;
use crate::ast::WildcardType;
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
use crate::token::TypeToken;
use crate::token::TypeTokenKind;

pub mod array_like;
pub mod callable;
pub mod generic;
pub mod object;
pub mod stream;

/// Parses a complete type expression, including unions, intersections, and conditionals.
#[inline]
pub fn parse_type<'arena>(stream: &mut TypeTokenStream<'arena>) -> Result<Type<'arena>, ParseError> {
    parse_type_with_precedence(stream, TypePrecedence::Lowest)
}

/// Returns `true` when the current token can act as a member-name identifier.
///
/// Accepts plain `Identifier` tokens, the `new` keyword when it is *not* followed by `<`, and
/// any single-word reserved keyword whose source text has no hyphen.
pub(crate) fn is_at_member_identifier(stream: &mut TypeTokenStream<'_>) -> Result<bool, ParseError> {
    is_at_member_identifier_at(stream, 0)
}

/// Like [`is_at_member_identifier`] but peeks `offset` tokens ahead instead of at the current
/// position. `offset == 0` is equivalent to [`is_at_member_identifier`].
pub(crate) fn is_at_member_identifier_at(stream: &mut TypeTokenStream<'_>, offset: usize) -> Result<bool, ParseError> {
    let Some(token) = stream.lookahead(offset)? else {
        return Ok(false);
    };

    match token.kind {
        TypeTokenKind::Identifier => Ok(true),
        TypeTokenKind::New => Ok(stream.lookahead(offset + 1)?.is_none_or(|t| t.kind != TypeTokenKind::LessThan)),
        kind if kind.is_keyword() && !token.value.as_bytes().contains(&b'-') => Ok(true),
        _ => Ok(false),
    }
}

/// Consumes the next token as a member-name identifier. Accepts `Identifier`, the `new`
/// keyword when it is not followed by `<`, or any non-hyphenated reserved keyword. Errors
/// with `UnexpectedToken` expecting `Identifier` otherwise, so the diagnostic matches the
/// traditional call site.
pub(crate) fn eat_member_identifier<'arena>(
    stream: &mut TypeTokenStream<'arena>,
) -> Result<TypeToken<'arena>, ParseError> {
    if is_at_member_identifier(stream)? { stream.consume() } else { stream.eat(TypeTokenKind::Identifier) }
}

/// Returns `true` when the current token would legitimately close an enclosing construct
/// (e.g. `,`, `)`, `>`, `}`, `]`, `:`, `;`, `=`, EOF). Used to detect a trailing `|` in a
/// union, so that `int|string|` or `array{0: int|string|}` parses leniently rather than
/// erroring on the empty right-hand operand.
#[inline]
fn is_at_union_closing_token(stream: &mut TypeTokenStream<'_>) -> Result<bool, ParseError> {
    Ok(match stream.lookahead(0)?.map(|t| t.kind) {
        None => true,
        Some(kind) => matches!(
            kind,
            TypeTokenKind::Comma
                | TypeTokenKind::RightParenthesis
                | TypeTokenKind::GreaterThan
                | TypeTokenKind::RightBrace
                | TypeTokenKind::RightBracket
                | TypeTokenKind::Colon
                | TypeTokenKind::Equals
        ),
    })
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
pub fn parse_type_with_precedence<'arena>(
    stream: &mut TypeTokenStream<'arena>,
    min_precedence: TypePrecedence,
) -> Result<Type<'arena>, ParseError> {
    let mut inner = parse_primary_type(stream)?;

    loop {
        let is_inner_nullable = matches!(inner, Type::Nullable(_));

        inner = match stream.lookahead(0)?.map(|t| t.kind) {
            // Union types: T|U (and `T|` with a trailing pipe).
            Some(TypeTokenKind::Pipe) if !is_inner_nullable && min_precedence <= TypePrecedence::Union => {
                let pipe = stream.consume_span()?;

                if is_at_union_closing_token(stream)? {
                    return Ok(Type::TrailingPipe(TrailingPipeType { inner: stream.alloc(inner), pipe }));
                }

                let right = parse_type_with_precedence(stream, TypePrecedence::Union)?;
                if let Type::TrailingPipe(trailing) = right {
                    return Ok(Type::TrailingPipe(TrailingPipeType {
                        inner: stream.alloc(Type::Union(UnionType {
                            left: stream.alloc(inner),
                            pipe,
                            right: trailing.inner,
                        })),
                        pipe: trailing.pipe,
                    }));
                }

                Type::Union(UnionType { left: stream.alloc(inner), pipe, right: stream.alloc(right) })
            }
            // Intersection types: T&U
            Some(TypeTokenKind::Ampersand)
                if !is_inner_nullable
                    && min_precedence <= TypePrecedence::Intersection
                    && !stream
                        .lookahead(1)?
                        .is_some_and(|t| matches!(t.kind, TypeTokenKind::Variable | TypeTokenKind::Ellipsis)) =>
            {
                let left = stream.alloc(inner);
                let ampersand = stream.consume_span()?;
                let rhs = parse_type_with_precedence(stream, TypePrecedence::Intersection)?;
                let right = stream.alloc(rhs);
                Type::Intersection(IntersectionType { left, ampersand, right })
            }
            // Conditional types: T is U ? V : W
            Some(TypeTokenKind::Is) if !is_inner_nullable && min_precedence <= TypePrecedence::Conditional => {
                let subject = stream.alloc(inner);
                let is = stream.consume_keyword()?;
                let not = if stream.is_at(TypeTokenKind::Not)? { Some(stream.consume_keyword()?) } else { None };
                let target_ty = parse_type_with_precedence(stream, TypePrecedence::Conditional)?;
                let target = stream.alloc(target_ty);
                let question_mark = stream.eat_span(TypeTokenKind::Question)?;
                let then_ty = parse_type_with_precedence(stream, TypePrecedence::Conditional)?;
                let then = stream.alloc(then_ty);
                let colon = stream.eat_span(TypeTokenKind::Colon)?;
                let otherwise_ty = parse_type_with_precedence(stream, TypePrecedence::Conditional)?;
                let otherwise = stream.alloc(otherwise_ty);
                Type::Conditional(ConditionalType { subject, is, not, target, question_mark, then, colon, otherwise })
            }
            // Postfix operations: T[], T[K]
            Some(TypeTokenKind::LeftBracket) if min_precedence <= TypePrecedence::Postfix => {
                let left_bracket = stream.consume_span()?;

                if stream.is_at(TypeTokenKind::RightBracket)? {
                    let inner_ref = stream.alloc(inner);
                    let right_bracket = stream.consume_span()?;
                    Type::Slice(SliceType { inner: inner_ref, left_bracket, right_bracket })
                } else {
                    let target = stream.alloc(inner);
                    let index_ty = parse_type(stream)?;
                    let index = stream.alloc(index_ty);
                    let right_bracket = stream.eat_span(TypeTokenKind::RightBracket)?;
                    Type::IndexAccess(IndexAccessType { target, left_bracket, index, right_bracket })
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
fn parse_primary_type<'arena>(stream: &mut TypeTokenStream<'arena>) -> Result<Type<'arena>, ParseError> {
    let next = stream.peek()?;
    let inner = match next.kind {
        TypeTokenKind::Variable => Type::Variable(VariableType::from_token(stream.consume()?, stream.file_id())),
        TypeTokenKind::Question => {
            let question_mark = stream.consume_span()?;
            let inner_ty = parse_type(stream)?;
            Type::Nullable(NullableType { question_mark, inner: stream.alloc(inner_ty) })
        }
        TypeTokenKind::LeftParenthesis => {
            let left_parenthesis = stream.consume_span()?;
            let inner_ty = parse_type(stream)?;
            let inner = stream.alloc(inner_ty);
            let right_parenthesis = stream.eat_span(TypeTokenKind::RightParenthesis)?;
            Type::Parenthesized(ParenthesizedType { left_parenthesis, inner, right_parenthesis })
        }
        TypeTokenKind::Asterisk => {
            let token = stream.consume()?;
            let asterisk_span = token.span_for(stream.file_id());

            if is_at_member_identifier(stream)? {
                let identifier = Identifier::from_token(eat_member_identifier(stream)?, stream.file_id());
                Type::GlobalWildcardReference(GlobalWildcardType {
                    selector: GlobalWildcardSelector::EndsWith(asterisk_span, identifier),
                })
            } else {
                Type::Wildcard(WildcardType { span: asterisk_span, kind: WildcardKind::Asterisk })
            }
        }
        TypeTokenKind::Mixed => Type::Mixed(stream.consume_keyword()?),
        TypeTokenKind::NonEmptyMixed => Type::NonEmptyMixed(stream.consume_keyword()?),
        TypeTokenKind::Null => Type::Null(stream.consume_keyword()?),
        TypeTokenKind::Void => Type::Void(stream.consume_keyword()?),
        TypeTokenKind::Never => Type::Never(stream.consume_keyword()?),
        TypeTokenKind::Resource => Type::Resource(stream.consume_keyword()?),
        TypeTokenKind::ClosedResource => Type::ClosedResource(stream.consume_keyword()?),
        TypeTokenKind::OpenResource => Type::OpenResource(stream.consume_keyword()?),
        TypeTokenKind::True => Type::True(stream.consume_keyword()?),
        TypeTokenKind::False => Type::False(stream.consume_keyword()?),
        TypeTokenKind::Bool | TypeTokenKind::Boolean => Type::Bool(stream.consume_keyword()?),
        TypeTokenKind::Float | TypeTokenKind::Real | TypeTokenKind::Double => Type::Float(stream.consume_keyword()?),
        TypeTokenKind::Int | TypeTokenKind::Integer => {
            let keyword = stream.consume_keyword()?;

            if stream.is_at(TypeTokenKind::LessThan)? {
                Type::IntRange(IntRangeType {
                    keyword,
                    less_than: stream.consume_span()?,
                    min: if stream.is_at(TypeTokenKind::Minus)? {
                        let minus = stream.consume_span()?;
                        let token = stream.eat(TypeTokenKind::LiteralInteger)?;
                        // `parse_literal_integer` only fails on inputs the lexer would already have
                        // rejected; fall back to `0` defensively rather than panicking.
                        let value = parse_literal_integer(token.value).unwrap_or(0);

                        IntOrKeyword::NegativeInt {
                            minus,
                            int: LiteralIntType { span: token.span_for(stream.file_id()), value, raw: token.value },
                        }
                    } else if stream.is_at(TypeTokenKind::LiteralInteger)? {
                        let token = stream.consume()?;
                        #[allow(clippy::unreachable)]
                        let value = parse_literal_integer(token.value).unwrap_or_else(|| {
                            unreachable!("lexer generated invalid integer `{}`; this should never happen.", token.value)
                        });

                        IntOrKeyword::Int(LiteralIntType {
                            span: token.span_for(stream.file_id()),
                            value,
                            raw: token.value,
                        })
                    } else if stream.is_at(TypeTokenKind::Int)? || stream.is_at(TypeTokenKind::Integer)? {
                        IntOrKeyword::Keyword(stream.consume_keyword()?)
                    } else {
                        IntOrKeyword::Keyword(stream.eat_keyword(TypeTokenKind::Min)?)
                    },
                    comma: stream.eat_span(TypeTokenKind::Comma)?,
                    max: if stream.is_at(TypeTokenKind::Minus)? {
                        let minus = stream.consume_span()?;
                        let token = stream.eat(TypeTokenKind::LiteralInteger)?;
                        // `parse_literal_integer` only fails on inputs the lexer would already have
                        // rejected; fall back to `0` defensively rather than panicking.
                        let value = parse_literal_integer(token.value).unwrap_or(0);

                        IntOrKeyword::NegativeInt {
                            minus,
                            int: LiteralIntType { span: token.span_for(stream.file_id()), value, raw: token.value },
                        }
                    } else if stream.is_at(TypeTokenKind::LiteralInteger)? {
                        let token = stream.consume()?;
                        #[allow(clippy::unreachable)]
                        let value = parse_literal_integer(token.value).unwrap_or_else(|| {
                            unreachable!("lexer generated invalid integer `{}`; this should never happen.", token.value)
                        });

                        IntOrKeyword::Int(LiteralIntType {
                            span: token.span_for(stream.file_id()),
                            value,
                            raw: token.value,
                        })
                    } else if stream.is_at(TypeTokenKind::Int)? || stream.is_at(TypeTokenKind::Integer)? {
                        IntOrKeyword::Keyword(stream.consume_keyword()?)
                    } else {
                        IntOrKeyword::Keyword(stream.eat_keyword(TypeTokenKind::Max)?)
                    },
                    greater_than: stream.eat_span(TypeTokenKind::GreaterThan)?,
                })
            } else {
                Type::Int(keyword)
            }
        }
        TypeTokenKind::PositiveInt => Type::PositiveInt(stream.consume_keyword()?),
        TypeTokenKind::NegativeInt => Type::NegativeInt(stream.consume_keyword()?),
        TypeTokenKind::NonPositiveInt => Type::NonPositiveInt(stream.consume_keyword()?),
        TypeTokenKind::NonNegativeInt => Type::NonNegativeInt(stream.consume_keyword()?),
        TypeTokenKind::NonZeroInt => Type::NonZeroInt(stream.consume_keyword()?),
        TypeTokenKind::String => Type::String(stream.consume_keyword()?),
        TypeTokenKind::CallableString => Type::CallableString(stream.consume_keyword()?),
        TypeTokenKind::LowercaseCallableString => Type::LowercaseCallableString(stream.consume_keyword()?),
        TypeTokenKind::UppercaseCallableString => Type::UppercaseCallableString(stream.consume_keyword()?),
        TypeTokenKind::NumericString => Type::NumericString(stream.consume_keyword()?),
        TypeTokenKind::NonEmptyString => Type::NonEmptyString(stream.consume_keyword()?),
        TypeTokenKind::NonEmptyLowercaseString => Type::NonEmptyLowercaseString(stream.consume_keyword()?),
        TypeTokenKind::LowercaseString => Type::LowercaseString(stream.consume_keyword()?),
        TypeTokenKind::NonEmptyUppercaseString => Type::NonEmptyUppercaseString(stream.consume_keyword()?),
        TypeTokenKind::UppercaseString => Type::UppercaseString(stream.consume_keyword()?),
        TypeTokenKind::TruthyString => Type::TruthyString(stream.consume_keyword()?),
        TypeTokenKind::NonFalsyString => Type::NonFalsyString(stream.consume_keyword()?),
        TypeTokenKind::Object => parse_object_type(stream)?,
        TypeTokenKind::NoReturn | TypeTokenKind::NeverReturn | TypeTokenKind::NeverReturns | TypeTokenKind::Nothing => {
            Type::Never(stream.consume_keyword()?)
        }
        TypeTokenKind::KeyOf => Type::KeyOf(KeyOfType {
            keyword: stream.consume_keyword()?,
            parameter: parse_single_generic_parameter(stream)?,
        }),
        TypeTokenKind::ValueOf => Type::ValueOf(ValueOfType {
            keyword: stream.consume_keyword()?,
            parameter: parse_single_generic_parameter(stream)?,
        }),
        TypeTokenKind::IntMask => Type::IntMask(IntMaskType {
            keyword: stream.consume_keyword()?,
            parameters: parse_generic_parameters(stream)?,
        }),
        TypeTokenKind::IntMaskOf => Type::IntMaskOf(IntMaskOfType {
            keyword: stream.consume_keyword()?,
            parameter: parse_single_generic_parameter(stream)?,
        }),
        TypeTokenKind::New => Type::New(NewType {
            keyword: stream.consume_keyword()?,
            parameter: parse_single_generic_parameter(stream)?,
        }),
        TypeTokenKind::TemplateType => Type::TemplateType(TemplateTypeType {
            keyword: stream.consume_keyword()?,
            parameters: parse_generic_parameters(stream)?,
        }),
        TypeTokenKind::Scalar => Type::Scalar(stream.consume_keyword()?),
        TypeTokenKind::Numeric => Type::Numeric(stream.consume_keyword()?),
        TypeTokenKind::ArrayKey => Type::ArrayKey(stream.consume_keyword()?),
        TypeTokenKind::StringableObject => Type::StringableObject(stream.consume_keyword()?),
        TypeTokenKind::UnspecifiedLiteralInt => Type::UnspecifiedLiteralInt(stream.consume_keyword()?),
        TypeTokenKind::UnspecifiedLiteralString => Type::UnspecifiedLiteralString(stream.consume_keyword()?),
        TypeTokenKind::UnspecifiedLiteralFloat => Type::UnspecifiedLiteralFloat(stream.consume_keyword()?),
        TypeTokenKind::NonEmptyUnspecifiedLiteralString => {
            Type::NonEmptyUnspecifiedLiteralString(stream.consume_keyword()?)
        }
        TypeTokenKind::PropertiesOf => Type::PropertiesOf(PropertiesOfType {
            filter: PropertiesOfFilter::All,
            keyword: stream.consume_keyword()?,
            parameter: parse_single_generic_parameter(stream)?,
        }),
        TypeTokenKind::PublicPropertiesOf => Type::PropertiesOf(PropertiesOfType {
            filter: PropertiesOfFilter::Public,
            keyword: stream.consume_keyword()?,
            parameter: parse_single_generic_parameter(stream)?,
        }),
        TypeTokenKind::PrivatePropertiesOf => Type::PropertiesOf(PropertiesOfType {
            filter: PropertiesOfFilter::Private,
            keyword: stream.consume_keyword()?,
            parameter: parse_single_generic_parameter(stream)?,
        }),
        TypeTokenKind::ProtectedPropertiesOf => Type::PropertiesOf(PropertiesOfType {
            filter: PropertiesOfFilter::Protected,
            keyword: stream.consume_keyword()?,
            parameter: parse_single_generic_parameter(stream)?,
        }),
        TypeTokenKind::Array
        | TypeTokenKind::NonEmptyArray
        | TypeTokenKind::AssociativeArray
        | TypeTokenKind::List
        | TypeTokenKind::NonEmptyList => parse_array_like_type(stream)?,
        TypeTokenKind::Iterable => Type::Iterable(IterableType {
            keyword: stream.consume_keyword()?,
            parameters: parse_generic_parameters_or_none(stream)?,
        }),
        TypeTokenKind::LiteralFloat => {
            let token = stream.consume()?;
            #[allow(clippy::unreachable)]
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
            #[allow(clippy::unreachable)]
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
        TypeTokenKind::Minus => {
            Type::Negated(NegatedType { minus: stream.consume_span()?, number: parse_literal_number_type(stream)? })
        }
        TypeTokenKind::Plus => {
            Type::Posited(PositedType { plus: stream.consume_span()?, number: parse_literal_number_type(stream)? })
        }
        TypeTokenKind::EnumString => Type::EnumString(EnumStringType {
            keyword: stream.consume_keyword()?,
            parameter: parse_single_generic_parameter_or_none(stream)?,
        }),
        TypeTokenKind::TraitString => Type::TraitString(TraitStringType {
            keyword: stream.consume_keyword()?,
            parameter: parse_single_generic_parameter_or_none(stream)?,
        }),
        TypeTokenKind::ClassString => Type::ClassString(ClassStringType {
            keyword: stream.consume_keyword()?,
            parameter: parse_single_generic_parameter_or_none(stream)?,
        }),
        TypeTokenKind::InterfaceString => Type::InterfaceString(InterfaceStringType {
            keyword: stream.consume_keyword()?,
            parameter: parse_single_generic_parameter_or_none(stream)?,
        }),
        TypeTokenKind::Callable => Type::Callable(CallableType {
            kind: CallableTypeKind::Callable,
            keyword: stream.consume_keyword()?,
            specification: parse_optional_callable_type_specifications(stream)?,
        }),
        TypeTokenKind::PureCallable => Type::Callable(CallableType {
            kind: CallableTypeKind::PureCallable,
            keyword: stream.consume_keyword()?,
            specification: parse_optional_callable_type_specifications(stream)?,
        }),
        TypeTokenKind::PureClosure => Type::Callable(CallableType {
            kind: CallableTypeKind::PureClosure,
            keyword: stream.consume_keyword()?,
            specification: parse_optional_callable_type_specifications(stream)?,
        }),
        TypeTokenKind::QualifiedIdentifier => {
            let identifier = Identifier::from_token(stream.consume()?, stream.file_id());
            if stream.is_at(TypeTokenKind::ColonColon)? {
                let double_colon = stream.consume_span()?;

                if stream.is_at(TypeTokenKind::Asterisk)? {
                    let asterisk = stream.consume_span()?;

                    Type::MemberReference(MemberReferenceType {
                        class: identifier,
                        double_colon,
                        member: if is_at_member_identifier(stream)? {
                            MemberReferenceSelector::EndsWith(
                                asterisk,
                                Identifier::from_token(eat_member_identifier(stream)?, stream.file_id()),
                            )
                        } else {
                            MemberReferenceSelector::Wildcard(asterisk)
                        },
                    })
                } else {
                    let identifier = Identifier::from_token(eat_member_identifier(stream)?, stream.file_id());

                    Type::MemberReference(MemberReferenceType {
                        class: identifier,
                        double_colon,
                        member: if stream.is_at(TypeTokenKind::Asterisk)? {
                            MemberReferenceSelector::StartsWith(identifier, stream.consume_span()?)
                        } else {
                            MemberReferenceSelector::Identifier(identifier)
                        },
                    })
                }
            } else {
                Type::Reference(ReferenceType { identifier, parameters: parse_generic_parameters_or_none(stream)? })
            }
        }
        TypeTokenKind::Identifier if next.value == "_" => {
            let token = stream.consume()?;

            Type::Wildcard(WildcardType { span: token.span_for(stream.file_id()), kind: WildcardKind::Underscore })
        }
        TypeTokenKind::Identifier => {
            if next.value.eq_ignore_ascii_case("Closure")
                && matches!(stream.lookahead(1)?.map(|t| t.kind), Some(TypeTokenKind::LeftParenthesis))
            {
                Type::Callable(CallableType {
                    kind: CallableTypeKind::Closure,
                    keyword: stream.consume_keyword()?,
                    specification: Some(parse_callable_type_specifications(stream)?),
                })
            } else {
                let identifier = Identifier::from_token(stream.consume()?, stream.file_id());
                if stream.is_at(TypeTokenKind::ColonColon)? {
                    let double_colon = stream.consume_span()?;

                    if stream.is_at(TypeTokenKind::Asterisk)? {
                        let asterisk = stream.consume_span()?;

                        Type::MemberReference(MemberReferenceType {
                            class: identifier,
                            double_colon,
                            member: if is_at_member_identifier(stream)? {
                                MemberReferenceSelector::EndsWith(
                                    asterisk,
                                    Identifier::from_token(eat_member_identifier(stream)?, stream.file_id()),
                                )
                            } else {
                                MemberReferenceSelector::Wildcard(asterisk)
                            },
                        })
                    } else {
                        let member_identifier =
                            Identifier::from_token(eat_member_identifier(stream)?, stream.file_id());

                        Type::MemberReference(MemberReferenceType {
                            class: identifier,
                            double_colon,
                            member: if stream.is_at(TypeTokenKind::Asterisk)? {
                                MemberReferenceSelector::StartsWith(member_identifier, stream.consume_span()?)
                            } else {
                                MemberReferenceSelector::Identifier(member_identifier)
                            },
                        })
                    }
                } else if stream.is_at(TypeTokenKind::Asterisk)? {
                    let asterisk_span = stream.consume_span()?;
                    Type::GlobalWildcardReference(GlobalWildcardType {
                        selector: GlobalWildcardSelector::StartsWith(identifier, asterisk_span),
                    })
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
                    keyword: stream.consume_keyword()?,
                    specification: Some(parse_callable_type_specifications(stream)?),
                })
            } else {
                let identifier = Identifier::from_token(stream.consume()?, stream.file_id());

                if stream.is_at(TypeTokenKind::ColonColon)? {
                    let double_colon = stream.consume_span()?;

                    Type::MemberReference(MemberReferenceType {
                        class: identifier,
                        double_colon,
                        member: if stream.is_at(TypeTokenKind::Asterisk)? {
                            let asterisk = stream.consume_span()?;

                            if is_at_member_identifier(stream)? {
                                MemberReferenceSelector::EndsWith(
                                    asterisk,
                                    Identifier::from_token(eat_member_identifier(stream)?, stream.file_id()),
                                )
                            } else {
                                MemberReferenceSelector::Wildcard(asterisk)
                            }
                        } else {
                            let identifier = Identifier::from_token(eat_member_identifier(stream)?, stream.file_id());

                            if stream.is_at(TypeTokenKind::Asterisk)? {
                                MemberReferenceSelector::StartsWith(identifier, stream.consume_span()?)
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
            let exclamation = stream.consume_span()?;
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
            let double_colon = stream.eat_span(TypeTokenKind::ColonColon)?;

            // Parse the alias name (can be identifier or keyword without `-`)
            let next = stream.peek()?;
            let alias = if next.kind.is_keyword() && !next.value.contains('-') {
                AliasName::Keyword(stream.consume_keyword()?)
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
            // Trivia tokens are skipped by the stream; if one slips through it's a stream bug,
            // so surface it as an unexpected-token error rather than panicking.
            return Err(ParseError::UnexpectedToken(vec![], next.kind, next.span_for(stream.file_id())));
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

pub fn parse_literal_number_type<'arena>(
    stream: &mut TypeTokenStream<'arena>,
) -> Result<LiteralIntOrFloatType<'arena>, ParseError> {
    let next = stream.peek()?;

    match next.kind {
        TypeTokenKind::LiteralInteger => {
            let token = stream.consume()?;
            // Defensive fallback: see comment in `parse_atomic_type` above.
            let value = parse_literal_integer(token.value).unwrap_or(0);

            Ok(LiteralIntOrFloatType::Int(LiteralIntType {
                span: token.span_for(stream.file_id()),
                value,
                raw: token.value,
            }))
        }
        TypeTokenKind::LiteralFloat => {
            let token = stream.consume()?;
            // Defensive fallback: see comment in `parse_atomic_type` above.
            let value = parse_literal_float(token.value).unwrap_or(0.0);

            Ok(LiteralIntOrFloatType::Float(LiteralFloatType {
                span: token.span_for(stream.file_id()),
                value: OrderedFloat(value),
                raw: token.value,
            }))
        }
        _ => Err(ParseError::UnexpectedToken(vec![], next.kind, next.span_for(stream.file_id()))),
    }
}
