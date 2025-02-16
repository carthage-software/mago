use mago_ast::ast::*;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::tag::parse_opening_tag;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

#[inline]
pub fn parse_optional_terminator(stream: &mut TokenStream<'_, '_>) -> Result<Option<Terminator>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T![";" | "?>"]) => Some(parse_terminator(stream)?),
        _ => None,
    })
}

#[inline]
pub fn parse_terminator(stream: &mut TokenStream<'_, '_>) -> Result<Terminator, ParseError> {
    let token = utils::expect_one_of(stream, T![";", "?>"])?;

    match token.kind {
        T![";"] => Ok(Terminator::Semicolon(token.span)),
        T!["?>"] => {
            let closing_tag = ClosingTag { span: token.span };

            if matches!(utils::maybe_peek(stream)?.map(|t| t.kind), Some(T!["<?php" | "<?=" | "<?"])) {
                Ok(Terminator::TagPair(closing_tag, parse_opening_tag(stream)?))
            } else {
                Ok(Terminator::ClosingTag(closing_tag))
            }
        }
        _ => unreachable!(),
    }
}
