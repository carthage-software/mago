use crate::T;
use crate::ast::ast::While;
use crate::ast::ast::WhileBody;
use crate::ast::ast::WhileColonDelimitedBody;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_while(&mut self) -> Result<While<'arena>, ParseError> {
        Ok(While {
            r#while: self.expect_keyword(T!["while"])?,
            left_parenthesis: self.stream.eat(T!["("])?.span,
            condition: self.arena.alloc(self.parse_expression()?),
            right_parenthesis: self.stream.eat(T![")"])?.span,
            body: self.parse_while_body()?,
        })
    }

    fn parse_while_body(&mut self) -> Result<WhileBody<'arena>, ParseError> {
        Ok(match self.stream.lookahead(0)?.map(|t| t.kind) {
            Some(T![":"]) => WhileBody::ColonDelimited(self.parse_while_colon_delimited_body()?),
            _ => WhileBody::Statement(self.arena.alloc(self.parse_statement()?)),
        })
    }

    fn parse_while_colon_delimited_body(&mut self) -> Result<WhileColonDelimitedBody<'arena>, ParseError> {
        Ok(WhileColonDelimitedBody {
            colon: self.stream.eat(T![":"])?.span,
            statements: {
                let mut statements = self.new_vec();
                loop {
                    if matches!(self.stream.lookahead(0)?.map(|t| t.kind), Some(T!["endwhile"])) {
                        break;
                    }

                    statements.push(self.parse_statement()?);
                }

                Sequence::new(statements)
            },
            end_while: self.expect_keyword(T!["endwhile"])?,
            terminator: self.parse_terminator()?,
        })
    }
}
