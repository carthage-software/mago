use crate::T;
use crate::ast::ast::ClosingTag;
use crate::ast::ast::FullOpeningTag;
use crate::ast::ast::OpeningTag;
use crate::ast::ast::ShortOpeningTag;
use crate::error::ParseError;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_opening_tag<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<OpeningTag<'arena>, ParseError> {
    let token = utils::expect_one_of(stream, &[T!["<?php"], T!["<?="], T!["<?"]])?;

    Ok(match token.kind {
        T!["<?php"] => OpeningTag::Full(FullOpeningTag { span: token.span, value: token.value }),
        T!["<?"] => OpeningTag::Short(ShortOpeningTag { span: token.span }),
        _ => unreachable!(),
    })
}

pub fn parse_closing_tag(stream: &mut TokenStream<'_, '_>) -> Result<ClosingTag, ParseError> {
    let span = utils::expect_span(stream, T!["?>"])?;

    Ok(ClosingTag { span })
}
