use crate::T;
use crate::ast::ast::*;
use crate::error::ParseError;
use crate::parser::internal::expression::parse_expression;
use crate::parser::internal::terminator::parse_terminator;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub mod do_while;
pub mod r#for;
pub mod foreach;
pub mod r#while;

pub fn parse_continue<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Continue<'arena>, ParseError> {
    Ok(Continue {
        r#continue: utils::expect_keyword(stream, T!["continue"])?,
        level: if !matches!(utils::peek(stream)?.kind, T![";" | "?>"]) {
            Some(parse_expression(stream)?)
        } else {
            None
        },
        terminator: parse_terminator(stream)?,
    })
}

pub fn parse_break<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Break<'arena>, ParseError> {
    Ok(Break {
        r#break: utils::expect_keyword(stream, T!["break"])?,
        level: if !matches!(utils::peek(stream)?.kind, T![";" | "?>"]) {
            Some(parse_expression(stream)?)
        } else {
            None
        },
        terminator: parse_terminator(stream)?,
    })
}
