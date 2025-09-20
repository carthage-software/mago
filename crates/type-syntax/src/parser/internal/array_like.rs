use mago_database::file::HasFileId;
use mago_span::Position;
use mago_span::Span;
use mago_syntax_core::utils::parse_literal_integer;

use crate::ast::*;
use crate::error::ParseError;
use crate::parser::internal::generic::parse_generic_parameters_or_none;
use crate::parser::internal::parse_type;
use crate::parser::internal::stream::TypeTokenStream;
use crate::token::TypeTokenKind;

#[inline]
pub fn parse_array_like_type<'input>(stream: &mut TypeTokenStream<'input>) -> Result<Type<'input>, ParseError> {
    let next = stream.peek()?;
    let (keyword, kind) = match next.kind {
        TypeTokenKind::Array => {
            let keyword = Keyword::from(stream.consume()?);
            if !stream.is_at(TypeTokenKind::LeftBrace)? {
                return Ok(Type::Array(ArrayType { keyword, parameters: parse_generic_parameters_or_none(stream)? }));
            }

            (keyword, ShapeTypeKind::Array)
        }
        TypeTokenKind::NonEmptyArray => {
            let keyword = Keyword::from(stream.consume()?);
            if !stream.is_at(TypeTokenKind::LeftBrace)? {
                return Ok(Type::NonEmptyArray(NonEmptyArrayType {
                    keyword,
                    parameters: parse_generic_parameters_or_none(stream)?,
                }));
            }

            (keyword, ShapeTypeKind::NonEmptyArray)
        }
        TypeTokenKind::AssociativeArray => {
            let keyword = Keyword::from(stream.consume()?);
            if !stream.is_at(TypeTokenKind::LeftBrace)? {
                return Ok(Type::AssociativeArray(AssociativeArrayType {
                    keyword,
                    parameters: parse_generic_parameters_or_none(stream)?,
                }));
            }

            (keyword, ShapeTypeKind::AssociativeArray)
        }
        TypeTokenKind::List => {
            let keyword = Keyword::from(stream.consume()?);
            if !stream.is_at(TypeTokenKind::LeftBrace)? {
                return Ok(Type::List(ListType { keyword, parameters: parse_generic_parameters_or_none(stream)? }));
            }

            (keyword, ShapeTypeKind::List)
        }
        TypeTokenKind::NonEmptyList => {
            let keyword = Keyword::from(stream.consume()?);
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
                next.span,
            ));
        }
    };

    Ok(Type::Shape(ShapeType {
        kind,
        keyword,
        left_brace: stream.eat(TypeTokenKind::LeftBrace)?.span,
        fields: {
            let mut fields = Vec::new();
            while !stream.is_at(TypeTokenKind::RightBrace)? && !stream.is_at(TypeTokenKind::Ellipsis)? {
                let has_key = {
                    let mut found_key = false;
                    // Scan ahead to determine if a key is present before the value type.
                    for i in 0.. {
                        let Some(token) = stream.lookahead(i)? else {
                            // Reached the end of the stream, so no key was found.
                            break;
                        };

                        match token.kind {
                            // If we find a colon, we know a key is present.
                            TypeTokenKind::Colon => {
                                found_key = true;
                                break;
                            }
                            TypeTokenKind::Question => {
                                // If we find a question mark, it could indicate a key,
                                // if the following token is a colon.
                                if stream.lookahead(i + 1)?.is_some_and(|t| t.kind == TypeTokenKind::Colon) {
                                    found_key = true;
                                    break;
                                } else {
                                    // If the question mark is not followed by a colon,
                                    // it could be part of the key.
                                    continue;
                                }
                            }
                            // If we find any of these tokens, what came before must have
                            // been a full value type, not a key.
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
                            // Any other token is part of a potential key, so keep scanning.
                            _ => continue,
                        }
                    }

                    found_key
                };

                let field = ShapeField {
                    key: if has_key {
                        Some(ShapeFieldKey {
                            key: parse_shape_field_key(stream)?,
                            question_mark: if stream.is_at(TypeTokenKind::Question)? {
                                Some(stream.consume()?.span)
                            } else {
                                None
                            },
                            colon: stream.eat(TypeTokenKind::Colon)?.span,
                        })
                    } else {
                        None
                    },
                    value: Box::new(parse_type(stream)?),
                    comma: if stream.is_at(TypeTokenKind::Comma)? { Some(stream.consume()?.span) } else { None },
                };

                if field.comma.is_none() {
                    fields.push(field);
                    break;
                }

                fields.push(field);
            }

            fields
        },
        additional_fields: {
            if !stream.is_at(TypeTokenKind::Ellipsis)? {
                None
            } else {
                Some(ShapeAdditionalFields {
                    ellipsis: stream.consume()?.span,
                    parameters: parse_generic_parameters_or_none(stream)?,
                })
            }
        },
        right_brace: stream.eat(TypeTokenKind::RightBrace)?.span,
    }))
}

pub fn parse_shape_field_key<'input>(stream: &mut TypeTokenStream<'input>) -> Result<ShapeKey<'input>, ParseError> {
    if stream.is_at(TypeTokenKind::LiteralString)? {
        let token = stream.consume()?;
        let value = &token.value[1..token.value.len() - 1];

        return Ok(ShapeKey::String { value, span: token.span });
    }

    if stream.is_at(TypeTokenKind::LiteralInteger)? {
        let token = stream.consume()?;
        let value = parse_literal_integer(token.value).unwrap_or_else(|| {
            unreachable!("lexer generated invalid integer `{}`; this should never happen.", token.value)
        }) as i64;

        return Ok(ShapeKey::Integer { value, span: token.span });
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
            let value = parse_literal_integer(token.value).unwrap_or_else(|| {
                unreachable!("lexer generated invalid integer `{}`; this should never happen.", token.value)
            }) as i64;

            return Ok(ShapeKey::Integer {
                value: if is_negative { -value } else { value },
                span: Span::new(stream.file_id(), sign_token.span.start, token.span.end),
            });
        } else if stream.is_at(TypeTokenKind::LiteralFloat)? {
            let token = stream.consume()?;
            return Ok(ShapeKey::String {
                value: stream.lexer.slice_in_range(sign_token.span.start.offset, token.span.end.offset),
                span: Span::new(stream.file_id(), sign_token.span.start, token.span.end),
            });
        }
    }

    if stream.is_at(TypeTokenKind::LiteralFloat)? {
        let token = stream.consume()?;
        return Ok(ShapeKey::String { value: token.value, span: token.span });
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
            start_offset = Some(token.span.start.offset);
        }
        end_offset = Some(token.span.end.offset);

        key_parts.push(token.value);
    }

    if key_parts.is_empty() {
        return Err(ParseError::UnexpectedToken(
            vec![TypeTokenKind::LiteralString, TypeTokenKind::LiteralInteger, TypeTokenKind::Identifier],
            stream.peek()?.kind,
            stream.peek()?.span,
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
