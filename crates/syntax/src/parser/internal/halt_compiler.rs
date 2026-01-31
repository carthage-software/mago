use crate::T;
use crate::ast::ast::HaltCompiler;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_halt_compiler(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<HaltCompiler<'arena>, ParseError> {
        Ok(HaltCompiler {
            halt_compiler: self.expect_one_of_keyword(stream, &[T!["__halt_compiler"]])?,
            left_parenthesis: stream.eat(T!["("])?.span,
            right_parenthesis: stream.eat(T![")"])?.span,
            terminator: self.parse_terminator(stream)?,
        })
    }
}
