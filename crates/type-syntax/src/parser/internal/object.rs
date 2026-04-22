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
pub fn parse_object_type<'arena>(stream: &mut TypeTokenStream<'arena>) -> Result<Type<'arena>, ParseError> {
    let keyword = Keyword::from_token(stream.eat(TypeTokenKind::Object)?, stream.file_id());
    if !stream.is_at(TypeTokenKind::LeftBrace)? {
        return Ok(Type::Object(ObjectType { keyword, properties: None }));
    }

    Ok(Type::Object(ObjectType {
        keyword,
        properties: Some(ObjectProperties {
            left_brace: stream.eat(TypeTokenKind::LeftBrace)?.span_for(stream.file_id()),
            fields: {
                let mut fields = stream.new_bvec::<ShapeField<'arena>>();
                while !stream.is_at(TypeTokenKind::RightBrace)? && !stream.is_at(TypeTokenKind::Ellipsis)? {
                    let has_key = crate::parser::internal::array_like::scan_for_shape_field_key(stream)?;

                    let key = if has_key {
                        let shape_key = parse_shape_field_key(stream)?;
                        let question_mark = if stream.is_at(TypeTokenKind::Question)? {
                            Some(stream.consume()?.span_for(stream.file_id()))
                        } else {
                            None
                        };
                        let colon = stream.eat(TypeTokenKind::Colon)?.span_for(stream.file_id());
                        Some(ShapeFieldKey { key: shape_key, question_mark, colon })
                    } else {
                        None
                    };
                    let value_ty = parse_type(stream)?;
                    let value = stream.alloc(value_ty);
                    let comma = if stream.is_at(TypeTokenKind::Comma)? {
                        Some(stream.consume()?.span_for(stream.file_id()))
                    } else {
                        None
                    };
                    let field = ShapeField { key, value, comma };

                    if field.comma.is_none() {
                        fields.push(field);
                        break;
                    }

                    fields.push(field);
                }

                mago_syntax_core::ast::Sequence::new(fields)
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
