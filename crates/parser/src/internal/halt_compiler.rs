use fennec_ast::ast::*;
use fennec_token::T;

use crate::error::ParseError;
use crate::internal::terminator::parse_terminator;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

pub fn parse_halt_compiler<'a, 'i>(stream: &mut TokenStream<'a, 'i>) -> Result<HaltCompiler, ParseError> {
    Ok(HaltCompiler {
        halt_compiler: utils::expect_one_of_keyword(stream, &[T!["__halt_compiler"]])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        right_parenthesis: utils::expect_span(stream, T![")"])?,
        terminator: parse_terminator(stream)?,
    })
}
