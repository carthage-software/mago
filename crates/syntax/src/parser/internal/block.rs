use crate::T;
use crate::ast::ast::Block;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_block(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<Block<'arena>, ParseError> {
        let left_brace = stream.eat(T!["{"])?.span;
        let mut statements = self.new_vec();

        loop {
            match stream.lookahead(0)?.map(|t| t.kind) {
                Some(T!["}"]) => break,
                Some(_) => match self.parse_statement(stream) {
                    Ok(statement) => statements.push(statement),
                    Err(err) => self.errors.push(err),
                },
                None => {
                    // EOF without closing brace
                    return Err(stream.unexpected(None, &[T!["}"]]));
                }
            }
        }

        let right_brace = stream.eat(T!["}"])?.span;

        Ok(Block { left_brace, statements: Sequence::new(statements), right_brace })
    }
}
