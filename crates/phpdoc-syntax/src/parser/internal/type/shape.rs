use mago_allocator::Arena;
use mago_span::Position;
use mago_span::Span;
use mago_syntax_core::utils::parse_literal_integer;

use crate::cst::identifier::Identifier;
use crate::cst::r#type::ShapeKey;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

const SHAPE_KEY_SCAN_LIMIT: usize = 48;

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    pub(crate) fn scan_for_shape_field_key(&mut self) -> Result<bool, ParseError> {
        let mut depth_angle: u32 = 0;

        for index in 0..SHAPE_KEY_SCAN_LIMIT {
            let Some(token) = self.stream.lookahead(index) else {
                return Ok(false);
            };

            match token.kind {
                TokenKind::Colon if depth_angle == 0 => return Ok(true),
                TokenKind::Question
                    if depth_angle == 0
                        && self.stream.lookahead(index + 1).is_some_and(|t| t.kind == TokenKind::Colon) =>
                {
                    return Ok(true);
                }
                TokenKind::Comma
                | TokenKind::RightBrace
                | TokenKind::LeftBrace
                | TokenKind::LeftParenthesis
                | TokenKind::RightParenthesis
                | TokenKind::LeftBracket
                | TokenKind::RightBracket
                | TokenKind::Ellipsis
                    if depth_angle == 0 =>
                {
                    return Ok(false);
                }
                TokenKind::LeftAngleBracket => depth_angle += 1,
                TokenKind::RightAngleBracket if depth_angle > 0 => depth_angle -= 1,
                _ => {}
            }
        }

        Ok(false)
    }

    pub(crate) fn parse_shape_field_key(&mut self) -> Result<ShapeKey<'arena>, ParseError> {
        if self.stream.is_at(TokenKind::SingleQuotedString) || self.stream.is_at(TokenKind::DoubleQuotedString) {
            let token = self.stream.consume()?;
            let value = token.value.get(1..token.value.len().saturating_sub(1)).unwrap_or(&[]);

            return Ok(ShapeKey::String { value, span: token.span_for(self.file_id()) });
        }

        if self.stream.is_at(TokenKind::LiteralInteger) {
            let token = self.stream.consume()?;
            let raw_value = parse_literal_integer(token.value).unwrap_or(0);
            let value = i64::try_from(raw_value).unwrap_or(0);

            return Ok(ShapeKey::Integer { value, span: token.span_for(self.file_id()) });
        }

        if self.stream.is_at(TokenKind::LiteralFloat) {
            let token = self.stream.consume()?;

            return Ok(ShapeKey::String { value: token.value, span: token.span_for(self.file_id()) });
        }

        if (self.stream.is_at(TokenKind::Minus) || self.stream.is_at(TokenKind::Plus))
            && self
                .stream
                .lookahead(1)
                .is_some_and(|t| matches!(t.kind, TokenKind::LiteralInteger | TokenKind::LiteralFloat))
        {
            let sign_token = self.stream.consume()?;
            let is_negative = sign_token.kind == TokenKind::Minus;
            let token = self.stream.consume()?;
            let end = Position::new(token.start.offset + token.value.len() as u32);
            let span = Span::new(self.file_id(), sign_token.start, end);

            if token.kind == TokenKind::LiteralInteger {
                let raw_value = parse_literal_integer(token.value).unwrap_or(0);
                let magnitude = i64::try_from(raw_value).unwrap_or(0);
                let value = if is_negative { magnitude.checked_neg().unwrap_or(0) } else { magnitude };

                return Ok(ShapeKey::Integer { value, span });
            }

            return Ok(ShapeKey::String { value: self.stream.raw_between(sign_token.start, end), span });
        }

        if self.stream.is_at(TokenKind::Identifier)
            && self.stream.lookahead(1).is_some_and(|t| t.kind == TokenKind::ColonColon)
            && self.is_at_member_identifier_at(2)
        {
            let class_token = self.stream.consume()?;
            let class_name = Identifier::from_token(class_token, self.file_id());
            let double_colon = self.stream.eat_span(TokenKind::ColonColon)?;
            let constant_token = self.eat_member_identifier()?;
            let constant_name = Identifier::from_token(constant_token, self.file_id());
            let span = class_name.span.join(constant_name.span);

            return Ok(ShapeKey::ClassLikeConstant { class_name, double_colon, constant_name, span });
        }

        let token = self.stream.eat(TokenKind::Identifier)?;

        Ok(ShapeKey::String { value: token.value, span: token.span_for(self.file_id()) })
    }
}
