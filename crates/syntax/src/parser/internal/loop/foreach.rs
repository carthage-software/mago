use crate::T;
use crate::ast::ast::Foreach;
use crate::ast::ast::ForeachBody;
use crate::ast::ast::ForeachColonDelimitedBody;
use crate::ast::ast::ForeachKeyValueTarget;
use crate::ast::ast::ForeachTarget;
use crate::ast::ast::ForeachValueTarget;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_foreach(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Foreach<'arena>, ParseError> {
        Ok(Foreach {
            foreach: self.expect_keyword(stream, T!["foreach"])?,
            left_parenthesis: stream.eat(T!["("])?.span,
            expression: self.arena.alloc(self.parse_expression(stream)?),
            r#as: self.expect_keyword(stream, T!["as"])?,
            target: self.parse_foreach_target(stream)?,
            right_parenthesis: stream.eat(T![")"])?.span,
            body: self.parse_foreach_body(stream)?,
        })
    }

    fn parse_foreach_target(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<ForeachTarget<'arena>, ParseError> {
        let key_or_value = self.arena.alloc(self.parse_expression(stream)?);

        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["=>"]) => ForeachTarget::KeyValue(ForeachKeyValueTarget {
                key: key_or_value,
                double_arrow: stream.consume()?.span,
                value: self.arena.alloc(self.parse_expression(stream)?),
            }),
            _ => ForeachTarget::Value(ForeachValueTarget { value: key_or_value }),
        })
    }

    fn parse_foreach_body(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<ForeachBody<'arena>, ParseError> {
        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T![":"]) => ForeachBody::ColonDelimited(self.parse_foreach_colon_delimited_body(stream)?),
            _ => ForeachBody::Statement(self.arena.alloc(self.parse_statement(stream)?)),
        })
    }

    fn parse_foreach_colon_delimited_body(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<ForeachColonDelimitedBody<'arena>, ParseError> {
        Ok(ForeachColonDelimitedBody {
            colon: stream.eat(T![":"])?.span,
            statements: {
                let mut statements = self.new_vec();
                loop {
                    if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["endforeach"])) {
                        break;
                    }

                    statements.push(self.parse_statement(stream)?);
                }

                Sequence::new(statements)
            },
            end_foreach: self.expect_keyword(stream, T!["endforeach"])?,
            terminator: self.parse_terminator(stream)?,
        })
    }
}
