use crate::T;
use crate::ast::ast::*;
use crate::error::ParseError;
use crate::parser::internal::tag::parse_opening_tag;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_optional_terminator<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Option<Terminator<'arena>>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T![";" | "?>"]) => Some(parse_terminator(stream)?),
        _ => None,
    })
}

pub fn parse_terminator<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Terminator<'arena>, ParseError> {
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
