use mago_span::Span;

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
                            name: Box::new(parse_shape_field_key(stream)?),
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

pub fn parse_shape_field_key<'input>(stream: &mut TypeTokenStream<'input>) -> Result<Type<'input>, ParseError> {
    let (is_next_literal_int, is_next_colon) = if let Some(next) = stream.lookahead(1)? {
        (next.kind == TypeTokenKind::LiteralInteger, next.kind == TypeTokenKind::Colon)
    } else {
        (false, false)
    };

    let is_negated_or_posited = stream.is_at(TypeTokenKind::Plus)? || stream.is_at(TypeTokenKind::Minus)?;

    let is_literal_string = stream.is_at(TypeTokenKind::LiteralString)? && is_next_colon;
    let is_literal_integer = stream.is_at(TypeTokenKind::LiteralInteger)? && is_next_colon;
    let is_negated_or_posited_integer = is_negated_or_posited && is_next_literal_int;

    if is_literal_integer || is_literal_string || is_negated_or_posited_integer {
        return parse_type(stream);
    }

    let mut starting_position = None;
    let mut ending_position = None;
    loop {
        let current = stream.peek()?;

        if current.kind == TypeTokenKind::Colon
            || (current.kind == TypeTokenKind::Question
                && stream.lookahead(1)?.is_some_and(|t| t.kind == TypeTokenKind::Colon))
        {
            if starting_position.is_none() {
                return Err(ParseError::UnexpectedToken(
                    vec![TypeTokenKind::LiteralString, TypeTokenKind::LiteralInteger],
                    current.kind,
                    current.span,
                ));
            }

            break;
        }

        let token = stream.consume()?;

        if starting_position.is_none() {
            starting_position = Some(token.span.start);
        }

        ending_position = Some(token.span.end);
    }

    let Some(start) = starting_position else {
        return Err(ParseError::UnexpectedToken(
            vec![TypeTokenKind::LiteralString, TypeTokenKind::LiteralInteger],
            TypeTokenKind::Colon,
            stream.peek()?.span,
        ));
    };

    let Some(end) = ending_position else {
        return Err(ParseError::UnexpectedToken(
            vec![TypeTokenKind::LiteralString, TypeTokenKind::LiteralInteger],
            TypeTokenKind::Colon,
            stream.peek()?.span,
        ));
    };

    Ok(Type::Reference(ReferenceType {
        identifier: Identifier {
            span: Span::new(start, end),
            value: stream.lexer.slice_in_range(start.offset, end.offset),
        },
        parameters: None,
    }))
}
