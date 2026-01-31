use crate::T;
use crate::ast::ast::ClosingTag;
use crate::ast::ast::FullOpeningTag;
use crate::ast::ast::OpeningTag;
use crate::ast::ast::ShortOpeningTag;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_opening_tag(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<OpeningTag<'arena>, ParseError> {
        let token = stream.consume()?;

        Ok(match token.kind {
            T!["<?php"] => OpeningTag::Full(FullOpeningTag { span: token.span, value: token.value }),
            T!["<?"] => OpeningTag::Short(ShortOpeningTag { span: token.span }),
            _ => return Err(stream.unexpected(Some(token), &[T!["<?php"], T!["<?="]])),
        })
    }

    pub(crate) fn parse_closing_tag(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<ClosingTag, ParseError> {
        let span = stream.eat(T!["?>"])?.span;

        Ok(ClosingTag { span })
    }
}
