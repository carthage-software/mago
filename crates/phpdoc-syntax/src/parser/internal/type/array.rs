use mago_syntax_core::ast::Sequence;

use crate::cst::r#type::ArrayType;
use crate::cst::r#type::AssociativeArrayType;
use crate::cst::r#type::ListType;
use crate::cst::r#type::NonEmptyArrayType;
use crate::cst::r#type::NonEmptyListType;
use crate::cst::r#type::ShapeAdditionalFields;
use crate::cst::r#type::ShapeField;
use crate::cst::r#type::ShapeFieldKey;
use crate::cst::r#type::ShapeType;
use crate::cst::r#type::ShapeTypeKind;
use crate::cst::r#type::Type;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_array_like_type(&mut self) -> Result<Type<'arena>, ParseError> {
        let next = self.stream.peek()?;
        let keyword = self.parse_keyword()?;

        let kind = if next.value.eq_ignore_ascii_case(b"non-empty-array") {
            if !self.stream.is_at(TokenKind::LeftBrace) {
                return Ok(Type::NonEmptyArray(NonEmptyArrayType {
                    keyword,
                    parameters: self.parse_generic_parameters_or_none()?,
                }));
            }

            ShapeTypeKind::NonEmptyArray
        } else if next.value.eq_ignore_ascii_case(b"associative-array") {
            if !self.stream.is_at(TokenKind::LeftBrace) {
                return Ok(Type::AssociativeArray(AssociativeArrayType {
                    keyword,
                    parameters: self.parse_generic_parameters_or_none()?,
                }));
            }

            ShapeTypeKind::AssociativeArray
        } else if next.value.eq_ignore_ascii_case(b"non-empty-list") {
            if !self.stream.is_at(TokenKind::LeftBrace) {
                return Ok(Type::NonEmptyList(NonEmptyListType {
                    keyword,
                    parameters: self.parse_generic_parameters_or_none()?,
                }));
            }

            ShapeTypeKind::NonEmptyList
        } else if next.value.eq_ignore_ascii_case(b"list") {
            if !self.stream.is_at(TokenKind::LeftBrace) {
                return Ok(Type::List(ListType { keyword, parameters: self.parse_generic_parameters_or_none()? }));
            }

            ShapeTypeKind::List
        } else {
            if !self.stream.is_at(TokenKind::LeftBrace) {
                return Ok(Type::Array(ArrayType { keyword, parameters: self.parse_generic_parameters_or_none()? }));
            }

            ShapeTypeKind::Array
        };

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

        let additional_fields = if self.stream.is_at(TokenKind::Ellipsis) {
            let ellipsis = self.stream.consume_span()?;
            let parameters = self.parse_generic_parameters_or_none()?;
            let comma = if self.stream.is_at(TokenKind::Comma) { Some(self.stream.consume_span()?) } else { None };

            Some(ShapeAdditionalFields { ellipsis, parameters, comma })
        } else {
            None
        };

        let right_brace = self.stream.eat_span(TokenKind::RightBrace)?;

        Ok(Type::Shape(ShapeType {
            kind,
            keyword,
            left_brace,
            fields: Sequence::new(fields),
            additional_fields,
            right_brace,
        }))
    }
}
