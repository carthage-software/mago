use mago_database::file::HasFileId;
use mago_span::Position;
use mago_span::Span;
use mago_syntax_core::utils::parse_literal_integer;

use crate::ast::ArrayType;
use crate::ast::AssociativeArrayType;
use crate::ast::Identifier;
use crate::ast::ListType;
use crate::ast::NonEmptyArrayType;
use crate::ast::NonEmptyListType;
use crate::ast::ShapeAdditionalFields;
use crate::ast::ShapeField;
use crate::ast::ShapeFieldKey;
use crate::ast::ShapeKey;
use crate::ast::ShapeType;
use crate::ast::ShapeTypeKind;
use crate::ast::Type;
use crate::error::ParseError;
use crate::parser::internal::eat_member_identifier;
use crate::parser::internal::generic::parse_generic_parameters_or_none;
use crate::parser::internal::is_at_member_identifier_at;
use crate::parser::internal::parse_type;
use crate::parser::internal::stream::TypeTokenStream;
use crate::token::TypeTokenKind;

/// Hard upper bound on the lookahead depth used when scanning for a
/// shape field's key. Must be less than the stream's lookahead capacity.
const SHAPE_KEY_SCAN_LIMIT: usize = 48;

/// Look ahead from the current stream position to determine whether the
/// next shape field starts with a `key:` / `key?:` prefix.
///
/// The scan tracks `<>` depth so that commas inside a generic parameter
/// list are not mistaken for the field terminator (e.g. the `,` in
/// `foo: array<int, string>`). `(`, `[`, and `{` remain hard terminators:
/// a `(` inside a field value signals a callable type (whose own `:` for
/// the return type must not be confused with a shape-key colon), a `[`
/// signals slice/array-access syntax, and a `{` would start a new shape.
/// A hard cap of [`SHAPE_KEY_SCAN_LIMIT`] prevents a pathological input
/// from exceeding the stream buffer's lookahead capacity; hitting the
/// cap conservatively returns `false` (treat the field as keyless).
#[inline]
pub fn scan_for_shape_field_key<'arena>(stream: &mut TypeTokenStream<'arena>) -> Result<bool, ParseError> {
    let mut depth_angle: u32 = 0;

    for i in 0..SHAPE_KEY_SCAN_LIMIT {
        let Some(token) = stream.lookahead(i)? else {
            return Ok(false);
        };
        match token.kind {
            TypeTokenKind::Colon if depth_angle == 0 => return Ok(true),
            TypeTokenKind::Question
                if depth_angle == 0 && stream.lookahead(i + 1)?.is_some_and(|t| t.kind == TypeTokenKind::Colon) =>
            {
                return Ok(true);
            }
            TypeTokenKind::Comma
            | TypeTokenKind::RightBrace
            | TypeTokenKind::LeftBrace
            | TypeTokenKind::LeftParenthesis
            | TypeTokenKind::RightParenthesis
            | TypeTokenKind::LeftBracket
            | TypeTokenKind::RightBracket
            | TypeTokenKind::Ellipsis
                if depth_angle == 0 =>
            {
                return Ok(false);
            }
            TypeTokenKind::LessThan => depth_angle += 1,
            TypeTokenKind::GreaterThan if depth_angle > 0 => depth_angle -= 1,
            _ => {}
        }
    }

    Ok(false)
}

#[inline]
pub fn parse_array_like_type<'arena>(stream: &mut TypeTokenStream<'arena>) -> Result<Type<'arena>, ParseError> {
    let next = stream.peek()?;
    let (keyword, kind) = match next.kind {
        TypeTokenKind::Array => {
            let keyword = stream.consume_keyword()?;
            if !stream.is_at(TypeTokenKind::LeftBrace)? {
                return Ok(Type::Array(ArrayType { keyword, parameters: parse_generic_parameters_or_none(stream)? }));
            }

            (keyword, ShapeTypeKind::Array)
        }
        TypeTokenKind::NonEmptyArray => {
            let keyword = stream.consume_keyword()?;
            if !stream.is_at(TypeTokenKind::LeftBrace)? {
                return Ok(Type::NonEmptyArray(NonEmptyArrayType {
                    keyword,
                    parameters: parse_generic_parameters_or_none(stream)?,
                }));
            }

            (keyword, ShapeTypeKind::NonEmptyArray)
        }
        TypeTokenKind::AssociativeArray => {
            let keyword = stream.consume_keyword()?;
            if !stream.is_at(TypeTokenKind::LeftBrace)? {
                return Ok(Type::AssociativeArray(AssociativeArrayType {
                    keyword,
                    parameters: parse_generic_parameters_or_none(stream)?,
                }));
            }

            (keyword, ShapeTypeKind::AssociativeArray)
        }
        TypeTokenKind::List => {
            let keyword = stream.consume_keyword()?;
            if !stream.is_at(TypeTokenKind::LeftBrace)? {
                return Ok(Type::List(ListType { keyword, parameters: parse_generic_parameters_or_none(stream)? }));
            }

            (keyword, ShapeTypeKind::List)
        }
        TypeTokenKind::NonEmptyList => {
            let keyword = stream.consume_keyword()?;
            if !stream.is_at(TypeTokenKind::LeftBrace)? {
                return Ok(Type::NonEmptyList(NonEmptyListType {
                    keyword,
                    parameters: parse_generic_parameters_or_none(stream)?,
                }));
            }

            (keyword, ShapeTypeKind::NonEmptyList)
        }
        _ => {
            return Err(ParseError::UnexpectedToken(
                vec![
                    TypeTokenKind::Array,
                    TypeTokenKind::NonEmptyArray,
                    TypeTokenKind::AssociativeArray,
                    TypeTokenKind::List,
                    TypeTokenKind::NonEmptyList,
                ],
                next.kind,
                next.span_for(stream.file_id()),
            ));
        }
    };

    Ok(Type::Shape(ShapeType {
        kind,
        keyword,
        left_brace: stream.eat_span(TypeTokenKind::LeftBrace)?,
        fields: {
            let mut fields = stream.new_bvec::<ShapeField<'arena>>();
            while !stream.is_at(TypeTokenKind::RightBrace)? && !stream.is_at(TypeTokenKind::Ellipsis)? {
                let has_key = scan_for_shape_field_key(stream)?;

                let key = if has_key {
                    let shape_key = parse_shape_field_key(stream)?;
                    let question_mark =
                        if stream.is_at(TypeTokenKind::Question)? { Some(stream.consume_span()?) } else { None };
                    let colon = stream.eat_span(TypeTokenKind::Colon)?;
                    Some(ShapeFieldKey { key: shape_key, question_mark, colon })
                } else {
                    None
                };
                let value_ty = parse_type(stream)?;
                let value = stream.alloc(value_ty);
                let comma = if stream.is_at(TypeTokenKind::Comma)? { Some(stream.consume_span()?) } else { None };
                let field = ShapeField { key, value, comma };

                if field.comma.is_none() {
                    fields.push(field);
                    break;
                }

                fields.push(field);
            }

            mago_syntax_core::ast::Sequence::new(fields)
        },
        additional_fields: {
            if stream.is_at(TypeTokenKind::Ellipsis)? {
                Some(ShapeAdditionalFields {
                    ellipsis: stream.consume_span()?,
                    parameters: parse_generic_parameters_or_none(stream)?,
                    comma: if stream.is_at(TypeTokenKind::Comma)? { Some(stream.consume_span()?) } else { None },
                })
            } else {
                None
            }
        },
        right_brace: stream.eat_span(TypeTokenKind::RightBrace)?,
    }))
}

pub fn parse_shape_field_key<'arena>(stream: &mut TypeTokenStream<'arena>) -> Result<ShapeKey<'arena>, ParseError> {
    if stream.is_at(TypeTokenKind::LiteralString)? {
        let token = stream.consume()?;
        let value = &token.value[1..token.value.len() - 1];

        return Ok(ShapeKey::String { value, span: token.span_for(stream.file_id()) });
    }

    if stream.is_at(TypeTokenKind::LiteralInteger)? {
        let token = stream.consume()?;
        let raw_value = parse_literal_integer(token.value).unwrap_or_else(|| {
            unreachable!("lexer generated invalid integer `{}`; this should never happen.", token.value)
        });
        let value = i64::try_from(raw_value).map_err(|_| {
            ParseError::UnexpectedToken(
                vec![TypeTokenKind::LiteralInteger],
                token.kind,
                token.span_for(stream.file_id()),
            )
        })?;

        return Ok(ShapeKey::Integer { value, span: token.span_for(stream.file_id()) });
    }

    if (stream.is_at(TypeTokenKind::Plus)? || stream.is_at(TypeTokenKind::Minus)?)
        && stream
            .lookahead(1)?
            .is_some_and(|t| t.kind == TypeTokenKind::LiteralInteger || t.kind == TypeTokenKind::LiteralFloat)
    {
        let sign_token = stream.consume()?;
        let is_negative = sign_token.kind == TypeTokenKind::Minus;

        if stream.is_at(TypeTokenKind::LiteralInteger)? {
            let token = stream.consume()?;
            let raw_value = parse_literal_integer(token.value).unwrap_or_else(|| {
                unreachable!("lexer generated invalid integer `{}`; this should never happen.", token.value)
            });
            let value = i64::try_from(raw_value).map_err(|_| {
                ParseError::UnexpectedToken(
                    vec![TypeTokenKind::LiteralInteger],
                    token.kind,
                    token.span_for(stream.file_id()),
                )
            })?;
            let value = if is_negative {
                value.checked_neg().ok_or_else(|| {
                    ParseError::UnexpectedToken(
                        vec![TypeTokenKind::LiteralInteger],
                        token.kind,
                        token.span_for(stream.file_id()),
                    )
                })?
            } else {
                value
            };

            return Ok(ShapeKey::Integer { value, span: Span::new(stream.file_id(), sign_token.start, token.end()) });
        } else if stream.is_at(TypeTokenKind::LiteralFloat)? {
            let token = stream.consume()?;
            return Ok(ShapeKey::String {
                value: stream.lexer.slice_in_range(sign_token.start.offset, token.end().offset),
                span: Span::new(stream.file_id(), sign_token.start, token.end()),
            });
        }
    }

    if stream.is_at(TypeTokenKind::LiteralFloat)? {
        let token = stream.consume()?;
        return Ok(ShapeKey::String { value: token.value, span: token.span_for(stream.file_id()) });
    }

    // Check for class-like constant key pattern: ClassName::CONSTANT_NAME
    if (stream.is_at(TypeTokenKind::Identifier)?
        || stream.is_at(TypeTokenKind::QualifiedIdentifier)?
        || stream.is_at(TypeTokenKind::FullyQualifiedIdentifier)?)
        && stream.lookahead(1)?.is_some_and(|t| t.kind == TypeTokenKind::ColonColon)
        && is_at_member_identifier_at(stream, 2)?
    {
        let class_token = stream.consume()?;
        let class_name = Identifier::from_token(class_token, stream.file_id());
        let double_colon = stream.eat_span(TypeTokenKind::ColonColon)?;
        let constant_token = eat_member_identifier(stream)?;
        let constant_name = Identifier::from_token(constant_token, stream.file_id());
        let span = class_name.span.join(constant_name.span);

        return Ok(ShapeKey::ClassLikeConstant { class_name, double_colon, constant_name, span });
    }

    let mut key_parts = Vec::new();
    let mut start_offset = None;
    let mut end_offset = None;

    loop {
        let current = stream.peek()?;

        if current.kind == TypeTokenKind::Colon
            || (current.kind == TypeTokenKind::Question
                && stream.lookahead(1)?.is_some_and(|t| t.kind == TypeTokenKind::Colon))
        {
            break;
        }

        match current.kind {
            TypeTokenKind::Comma
            | TypeTokenKind::RightBrace
            | TypeTokenKind::LeftBrace
            | TypeTokenKind::LeftParenthesis
            | TypeTokenKind::RightParenthesis
            | TypeTokenKind::LeftBracket
            | TypeTokenKind::RightBracket
            | TypeTokenKind::Ellipsis => {
                break;
            }
            _ => {}
        }

        let token = stream.consume()?;

        if start_offset.is_none() {
            start_offset = Some(token.start.offset);
        }
        end_offset = Some(token.end().offset);

        key_parts.push(token.value);
    }

    if key_parts.is_empty() {
        let token = stream.peek()?;
        return Err(ParseError::UnexpectedToken(
            vec![TypeTokenKind::LiteralString, TypeTokenKind::LiteralInteger, TypeTokenKind::Identifier],
            token.kind,
            token.span_for(stream.file_id()),
        ));
    }

    // Combine all parts into a single string key
    let start = start_offset.unwrap();
    let end = end_offset.unwrap();
    let key_value = stream.lexer.slice_in_range(start, end);

    Ok(ShapeKey::String {
        value: key_value,
        span: Span::new(stream.file_id(), Position::new(start), Position::new(end)),
    })
}
