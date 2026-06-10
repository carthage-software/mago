use crate::cst::identifier::Identifier;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;
use mago_allocator::Arena;

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_identifier(&mut self) -> Result<Identifier<'arena>, ParseError> {
        let token = self.stream.eat(TokenKind::Identifier)?;

        Ok(Identifier::from_token(token, self.file_id()))
    }
}
