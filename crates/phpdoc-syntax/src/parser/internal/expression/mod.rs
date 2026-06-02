use crate::cst::expression::ConstantExpression;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

pub(crate) mod access;
pub(crate) mod array;
pub(crate) mod literal;
pub(crate) mod unary;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_constant_expression(&mut self) -> Result<ConstantExpression<'arena>, ParseError> {
        let next = self.stream.peek()?;

        match next.kind {
            TokenKind::Minus => self.parse_negated_constant(),
            TokenKind::Plus => self.parse_posited_constant(),
            TokenKind::LiteralInteger
            | TokenKind::LiteralFloat
            | TokenKind::SingleQuotedString
            | TokenKind::DoubleQuotedString => self.parse_constant_literal(),
            TokenKind::LeftBracket => self.parse_bracket_array_constant(),
            TokenKind::Identifier => self.parse_identifier_constant(),
            _ => Err(ParseError::UnexpectedToken(next.span_for(self.file_id()))),
        }
    }
}
