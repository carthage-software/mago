use crate::T;
use crate::ast::ast::*;
use crate::error::ParseError;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_opening_tag(stream: &mut TokenStream<'_, '_>) -> Result<OpeningTag, ParseError> {
    let token = utils::expect_one_of(stream, &[T!["<?php"], T!["<?="], T!["<?"]])?;

    Ok(match token.kind {
        T!["<?php"] => OpeningTag::Full(FullOpeningTag { span: token.span, value: token.value }),
        T!["<?="] => OpeningTag::Echo(EchoOpeningTag { span: token.span }),
        T!["<?"] => OpeningTag::Short(ShortOpeningTag { span: token.span }),
        _ => unreachable!(),
    })
}

pub fn parse_closing_tag(stream: &mut TokenStream<'_, '_>) -> Result<ClosingTag, ParseError> {
    let span = utils::expect_span(stream, T!["?>"])?;

    Ok(ClosingTag { span })
}
