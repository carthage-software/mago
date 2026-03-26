use crate::T;
use crate::ast::ast::HaltCompiler;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_halt_compiler(&mut self) -> Result<HaltCompiler<'arena>, ParseError> {
        Ok(HaltCompiler {
            halt_compiler: self.expect_one_of_keyword(&[T!["__halt_compiler"]])?,
            left_parenthesis: self.stream.eat_span(T!["("])?,
            right_parenthesis: self.stream.eat_span(T![")"])?,
            terminator: self.parse_terminator()?,
        })
    }
}
