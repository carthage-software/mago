use mago_syntax_core::cst::Sequence;

use crate::cst::r#type::ShapeField;
use crate::cst::r#type::ShapeFieldKey;
use crate::cst::r#type::Type;
use crate::cst::r#type::object::ObjectProperties;
use crate::cst::r#type::object::ObjectType;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_object_type(&mut self) -> Result<Type<'arena>, ParseError> {
        let keyword = self.parse_keyword()?;
        if !self.stream.is_at(TokenKind::LeftBrace) {
            return Ok(Type::Object(ObjectType { keyword, properties: None }));
        }

        let left_brace = self.stream.eat_span(TokenKind::LeftBrace)?;

        let mut fields = self.new_vec::<ShapeField<'arena>>();
        while !self.stream.is_at(TokenKind::RightBrace) && !self.stream.is_at(TokenKind::Ellipsis) {
            let key = if self.scan_for_shape_field_key()? {
                let shape_key = self.parse_shape_field_key()?;
                let question_mark =
                    if self.stream.is_at(TokenKind::Question) { Some(self.stream.consume_span()?) } else { None };
                let colon = self.stream.eat_span(TokenKind::Colon)?;

                Some(ShapeFieldKey { key: shape_key, question_mark, colon })
            } else {
                None
            };

            let value = self.parse_type()?;
            let value = self.alloc(value);
            let comma = if self.stream.is_at(TokenKind::Comma) { Some(self.stream.consume_span()?) } else { None };
            let has_comma = comma.is_some();
            fields.push(ShapeField { key, value, comma });

            if !has_comma {
                break;
            }
        }

        let ellipsis = if self.stream.is_at(TokenKind::Ellipsis) { Some(self.stream.consume_span()?) } else { None };
        let right_brace = self.stream.eat_span(TokenKind::RightBrace)?;

        Ok(Type::Object(ObjectType {
            keyword,
            properties: Some(ObjectProperties { left_brace, fields: Sequence::new(fields), ellipsis, right_brace }),
        }))
    }
}
