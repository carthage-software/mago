use mago_database::file::HasFileId;

use crate::ast::Keyword;
use crate::ast::ShapeField;
use crate::ast::ShapeFieldKey;
use crate::ast::Type;
use crate::ast::object::ObjectProperties;
use crate::ast::object::ObjectType;
use crate::error::ParseError;
use crate::parser::internal::array_like::parse_shape_field_key;
use crate::parser::internal::parse_type;
use crate::parser::internal::stream::TypeTokenStream;
use crate::token::TypeTokenKind;

#[inline]
pub fn parse_object_type<'input>(stream: &mut TypeTokenStream<'input>) -> Result<Type<'input>, ParseError> {
    let keyword = Keyword::from_token(stream.eat(TypeTokenKind::Object)?, stream.file_id());
    if !stream.is_at(TypeTokenKind::LeftBrace)? {
        return Ok(Type::Object(ObjectType { keyword, properties: None }));
    }

    Ok(Type::Object(ObjectType {
        keyword,
        properties: Some(ObjectProperties {
            left_brace: stream.eat(TypeTokenKind::LeftBrace)?.span_for(stream.file_id()),
            fields: {
                let mut fields = Vec::new();
                while !stream.is_at(TypeTokenKind::RightBrace)? && !stream.is_at(TypeTokenKind::Ellipsis)? {
                    let has_key = crate::parser::internal::array_like::scan_for_shape_field_key(stream)?;

                    let field = ShapeField {
                        key: if has_key {
                            Some(ShapeFieldKey {
                                key: parse_shape_field_key(stream)?,
                                question_mark: if stream.is_at(TypeTokenKind::Question)? {
                                    Some(stream.consume()?.span_for(stream.file_id()))
                                } else {
                                    None
                                },
                                colon: stream.eat(TypeTokenKind::Colon)?.span_for(stream.file_id()),
                            })
                        } else {
                            None
                        },
                        value: Box::new(parse_type(stream)?),
                        comma: if stream.is_at(TypeTokenKind::Comma)? {
                            Some(stream.consume()?.span_for(stream.file_id()))
                        } else {
                            None
                        },
                    };

                    if field.comma.is_none() {
                        fields.push(field);
                        break;
                    }

                    fields.push(field);
                }

                fields
            },
            ellipsis: if stream.is_at(TypeTokenKind::Ellipsis)? {
                Some(stream.consume()?.span_for(stream.file_id()))
            } else {
                None
            },
            right_brace: stream.eat(TypeTokenKind::RightBrace)?.span_for(stream.file_id()),
        }),
    }))
}
