use crate::T;
use crate::ast::ast::Block;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_block(&mut self) -> Result<Block<'arena>, ParseError> {
        let left_brace = self.stream.eat_span(T!["{"])?;
        let mut statements = self.new_vec();

        loop {
            match self.stream.peek_kind(0)? {
                Some(T!["}"]) => break,
                Some(_) => {
                    let position_before = self.stream.current_position();
                    match self.parse_statement() {
                        Ok(statement) => statements.push(statement),
                        Err(err) => self.errors.push(err),
                    }
                    // Safety: prevent infinite loop if statement parsing didn't advance
                    if self.stream.current_position() == position_before
                        && let Ok(Some(token)) = self.stream.lookahead(0)
                    {
                        if token.kind == T!["}"] {
                            break;
                        }
                        self.errors.push(self.stream.unexpected(Some(token), &[]));
                        let _ = self.stream.consume();
                    }
                }
                None => {
                    // EOF without closing brace
                    return Err(self.stream.unexpected(None, &[T!["}"]]));
                }
            }
        }

        let right_brace = self.stream.eat_span(T!["}"])?;

        Ok(Block { left_brace, statements: Sequence::new(statements), right_brace })
    }
}
