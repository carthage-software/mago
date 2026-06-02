use crate::cst::variable::Variable;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_variable(&mut self) -> Result<Variable<'arena>, ParseError> {
        if self.stream.is_at(TokenKind::Variable) || self.stream.is_at(TokenKind::ThisVariable) {
            let token = self.stream.consume()?;

            Ok(Variable { span: token.span_for(self.file_id()), value: token.value })
        } else {
            Err(ParseError::UnexpectedToken(self.stream.peek()?.span_for(self.file_id())))
        }
    }
}
