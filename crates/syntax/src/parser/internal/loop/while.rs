use crate::T;
use crate::ast::ast::While;
use crate::ast::ast::WhileBody;
use crate::ast::ast::WhileColonDelimitedBody;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_while(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<While<'arena>, ParseError> {
        Ok(While {
            r#while: self.expect_keyword(stream, T!["while"])?,
            left_parenthesis: stream.eat(T!["("])?.span,
            condition: self.arena.alloc(self.parse_expression(stream)?),
            right_parenthesis: stream.eat(T![")"])?.span,
            body: self.parse_while_body(stream)?,
        })
    }

    fn parse_while_body(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<WhileBody<'arena>, ParseError> {
        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T![":"]) => WhileBody::ColonDelimited(self.parse_while_colon_delimited_body(stream)?),
            _ => WhileBody::Statement(self.arena.alloc(self.parse_statement(stream)?)),
        })
    }

    fn parse_while_colon_delimited_body(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<WhileColonDelimitedBody<'arena>, ParseError> {
        Ok(WhileColonDelimitedBody {
            colon: stream.eat(T![":"])?.span,
            statements: {
                let mut statements = self.new_vec();
                loop {
                    if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["endwhile"])) {
                        break;
                    }

                    statements.push(self.parse_statement(stream)?);
                }

                Sequence::new(statements)
            },
            end_while: self.expect_keyword(stream, T!["endwhile"])?,
            terminator: self.parse_terminator(stream)?,
        })
    }
}
