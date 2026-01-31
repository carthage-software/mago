use crate::T;
use crate::ast::ast::Block;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_block(&mut self) -> Result<Block<'arena>, ParseError> {
        let left_brace = self.stream.eat(T!["{"])?.span;
        let mut statements = self.new_vec();

        loop {
            match self.stream.lookahead(0)?.map(|t| t.kind) {
                Some(T!["}"]) => break,
                Some(_) => match self.parse_statement() {
                    Ok(statement) => statements.push(statement),
                    Err(err) => self.errors.push(err),
                },
                None => {
                    // EOF without closing brace
                    return Err(self.stream.unexpected(None, &[T!["}"]]));
                }
            }
        }

        let right_brace = self.stream.eat(T!["}"])?.span;

        Ok(Block { left_brace, statements: Sequence::new(statements), right_brace })
    }
}
