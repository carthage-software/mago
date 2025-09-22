use crate::ast::object::ObjectProperties;
use crate::ast::object::ObjectType;
use crate::ast::*;
use crate::error::ParseError;
use crate::parser::internal::array_like::parse_shape_field_key;
use crate::parser::internal::parse_type;
use crate::parser::internal::stream::TypeTokenStream;
use crate::token::TypeTokenKind;

#[inline]
pub fn parse_object_type<'input>(stream: &mut TypeTokenStream<'input>) -> Result<Type<'input>, ParseError> {
    let keyword = Keyword::from(stream.eat(TypeTokenKind::Object)?);
    if !stream.is_at(TypeTokenKind::LeftBrace)? {
        return Ok(Type::Object(ObjectType { keyword, properties: None }));
    }

    Ok(Type::Object(ObjectType {
        keyword,
        properties: Some(ObjectProperties {
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
            ellipsis: if stream.is_at(TypeTokenKind::Ellipsis)? { Some(stream.consume()?.span) } else { None },
            right_brace: stream.eat(TypeTokenKind::RightBrace)?.span,
        }),
    }))
}
