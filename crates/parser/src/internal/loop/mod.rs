use mago_ast::ast::*;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression;
use crate::internal::terminator::parse_terminator;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

pub mod do_while;
pub mod r#for;
pub mod foreach;
pub mod r#while;

#[inline]
pub fn parse_continue<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<Continue<'i>, ParseError> {
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

#[inline]
pub fn parse_break<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<Break<'i>, ParseError> {
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
