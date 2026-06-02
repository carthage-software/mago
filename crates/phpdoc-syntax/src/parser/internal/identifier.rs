use crate::cst::identifier::Identifier;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_identifier(&mut self) -> Result<Identifier<'arena>, ParseError> {
        let token = self.stream.eat(TokenKind::Identifier)?;

        Ok(Identifier::from_token(token, self.file_id()))
    }
}
