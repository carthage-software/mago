use mago_database::file::HasFileId;

use crate::T;
use crate::cst::cst::ClosingTag;
use crate::cst::cst::FullOpeningTag;
use crate::cst::cst::OpeningTag;
use crate::cst::cst::ShortOpeningTag;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'arena> Parser<'_, 'arena> {
    pub(crate) fn parse_opening_tag(&mut self) -> Result<OpeningTag<'arena>, ParseError> {
        let token = self.stream.consume()?;

        Ok(match token.kind {
            T!["<?php"] => {
                OpeningTag::Full(FullOpeningTag { span: token.span_for(self.stream.file_id()), value: token.value })
            }
            T!["<?"] => OpeningTag::Short(ShortOpeningTag { span: token.span_for(self.stream.file_id()) }),
            _ => return Err(self.stream.unexpected(Some(token), &[T!["<?php"], T!["<?="]])),
        })
    }

    pub(crate) fn parse_closing_tag(&mut self) -> Result<ClosingTag, ParseError> {
        let span = self.stream.eat_span(T!["?>"])?;

        Ok(ClosingTag { span })
    }
}
