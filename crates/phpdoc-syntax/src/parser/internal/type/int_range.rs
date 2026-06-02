use mago_syntax_core::utils::parse_literal_integer;

use crate::cst::keyword::Keyword;
use crate::cst::r#type::IntOrKeyword;
use crate::cst::r#type::IntRangeType;
use crate::cst::r#type::LiteralIntType;
use crate::cst::r#type::Type;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_int_range_type(&mut self, keyword: Keyword<'arena>) -> Result<Type<'arena>, ParseError> {
        let less_than = self.stream.eat_span(TokenKind::LeftAngleBracket)?;
        let min = self.parse_int_or_keyword()?;
        let comma = self.stream.eat_span(TokenKind::Comma)?;
        let max = self.parse_int_or_keyword()?;
        let greater_than = self.stream.eat_span(TokenKind::RightAngleBracket)?;

        Ok(Type::IntRange(IntRangeType { keyword, less_than, min, comma, max, greater_than }))
    }

    fn parse_int_or_keyword(&mut self) -> Result<IntOrKeyword<'arena>, ParseError> {
        let file_id = self.file_id();

        if self.stream.is_at(TokenKind::Minus) {
            let minus = self.stream.consume_span()?;
            let token = self.stream.eat(TokenKind::LiteralInteger)?;
            let value = parse_literal_integer(token.value).unwrap_or(0);

            Ok(IntOrKeyword::NegativeInt {
                minus,
                int: LiteralIntType { span: token.span_for(file_id), value, raw: token.value },
            })
        } else if self.stream.is_at(TokenKind::LiteralInteger) {
            let token = self.stream.consume()?;
            let value = parse_literal_integer(token.value).unwrap_or(0);

            Ok(IntOrKeyword::Int(LiteralIntType { span: token.span_for(file_id), value, raw: token.value }))
        } else if self.stream.is_at(TokenKind::Identifier) {
            Ok(IntOrKeyword::Keyword(self.parse_keyword()?))
        } else {
            Err(ParseError::UnexpectedToken(self.stream.peek()?.span_for(file_id)))
        }
    }
}
