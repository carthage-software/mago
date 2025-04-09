use crate::T;
use crate::ast::ast::*;
use crate::error::ParseError;
use crate::parser::internal::identifier::parse_local_identifier;
use crate::parser::internal::terminator::parse_terminator;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_goto(stream: &mut TokenStream<'_, '_>) -> Result<Goto, ParseError> {
    Ok(Goto {
        goto: utils::expect_keyword(stream, T!["goto"])?,
        label: parse_local_identifier(stream)?,
        terminator: parse_terminator(stream)?,
    })
}

pub fn parse_label(stream: &mut TokenStream<'_, '_>) -> Result<Label, ParseError> {
    Ok(Label { name: parse_local_identifier(stream)?, colon: utils::expect_span(stream, T![":"])? })
}
