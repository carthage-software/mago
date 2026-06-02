use mago_span::Span;

use crate::cst::r#type::IndexAccessType;
use crate::cst::r#type::Type;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_index_access_type(
        &mut self,
        target: &'arena Type<'arena>,
        left_bracket: Span,
    ) -> Result<Type<'arena>, ParseError> {
        let index = self.parse_type()?;
        let index = self.alloc(index);
        let right_bracket = self.stream.eat_span(TokenKind::RightBracket)?;

        Ok(Type::IndexAccess(IndexAccessType { target, left_bracket, index, right_bracket }))
    }
}
