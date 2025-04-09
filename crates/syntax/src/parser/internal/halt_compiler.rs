use crate::T;
use crate::ast::ast::*;
use crate::error::ParseError;
use crate::parser::internal::terminator::parse_terminator;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_halt_compiler(stream: &mut TokenStream<'_, '_>) -> Result<HaltCompiler, ParseError> {
    Ok(HaltCompiler {
        halt_compiler: utils::expect_one_of_keyword(stream, &[T!["__halt_compiler"]])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        right_parenthesis: utils::expect_span(stream, T![")"])?,
        terminator: parse_terminator(stream)?,
    })
}
