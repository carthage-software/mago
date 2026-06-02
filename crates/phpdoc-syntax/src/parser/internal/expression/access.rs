use crate::cst::expression::ClassLikeConstantAccessExpression;
use crate::cst::expression::ConstantAccessExpression;
use crate::cst::expression::ConstantExpression;
use crate::cst::identifier::Identifier;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_identifier_constant(&mut self) -> Result<ConstantExpression<'arena>, ParseError> {
        let next = self.stream.peek()?;

        if next.value.eq_ignore_ascii_case(b"true") {
            return Ok(ConstantExpression::True(self.parse_keyword()?));
        }

        if next.value.eq_ignore_ascii_case(b"false") {
            return Ok(ConstantExpression::False(self.parse_keyword()?));
        }

        if next.value.eq_ignore_ascii_case(b"null") {
            return Ok(ConstantExpression::Null(self.parse_keyword()?));
        }

        if next.value.eq_ignore_ascii_case(b"array")
            && self.stream.lookahead(1).is_some_and(|t| t.kind == TokenKind::LeftParenthesis)
        {
            let keyword = self.parse_keyword()?;
            let left_delimiter = self.stream.eat_span(TokenKind::LeftParenthesis)?;

            return self.parse_array_constant(Some(keyword), left_delimiter, TokenKind::RightParenthesis);
        }

        let file_id = self.file_id();
        let identifier = Identifier::from_token(self.stream.consume()?, file_id);

        if self.stream.is_at(TokenKind::ColonColon) {
            let double_colon = self.stream.consume_span()?;
            let constant = if self.stream.is_at(TokenKind::Asterisk) {
                Identifier::from_token(self.stream.consume()?, file_id)
            } else {
                Identifier::from_token(self.stream.eat(TokenKind::Identifier)?, file_id)
            };

            Ok(ConstantExpression::ClassLikeConstantAccess(ClassLikeConstantAccessExpression {
                class: identifier,
                double_colon,
                constant,
            }))
        } else {
            Ok(ConstantExpression::ConstantAccess(ConstantAccessExpression { name: identifier }))
        }
    }
}
