use crate::T;
use crate::ast::ast::ClosingTag;
use crate::ast::ast::FullOpeningTag;
use crate::ast::ast::OpeningTag;
use crate::ast::ast::ShortOpeningTag;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_opening_tag(&mut self) -> Result<OpeningTag<'arena>, ParseError> {
        let token = self.stream.consume()?;

        Ok(match token.kind {
            T!["<?php"] => OpeningTag::Full(FullOpeningTag { span: token.span, value: token.value }),
            T!["<?"] => OpeningTag::Short(ShortOpeningTag { span: token.span }),
            _ => return Err(self.stream.unexpected(Some(token), &[T!["<?php"], T!["<?="]])),
        })
    }

    pub(crate) fn parse_closing_tag(&mut self) -> Result<ClosingTag, ParseError> {
        let span = self.stream.eat(T!["?>"])?.span;

        Ok(ClosingTag { span })
    }
}
