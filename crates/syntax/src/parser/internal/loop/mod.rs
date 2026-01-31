use crate::T;
use crate::ast::ast::Break;
use crate::ast::ast::Continue;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

pub mod do_while;
pub mod r#for;
pub mod foreach;
pub mod r#while;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_continue(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Continue<'arena>, ParseError> {
        Ok(Continue {
            r#continue: self.expect_keyword(stream, T!["continue"])?,
            level: match stream.lookahead(0)?.map(|t| t.kind) {
                Some(T![";" | "?>"]) => None,
                _ => Some(self.parse_expression(stream)?),
            },
            terminator: self.parse_terminator(stream)?,
        })
    }

    pub(crate) fn parse_break(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<Break<'arena>, ParseError> {
        Ok(Break {
            r#break: self.expect_keyword(stream, T!["break"])?,
            level: match stream.lookahead(0)?.map(|t| t.kind) {
                Some(T![";" | "?>"]) => None,
                _ => Some(self.parse_expression(stream)?),
            },
            terminator: self.parse_terminator(stream)?,
        })
    }
}
