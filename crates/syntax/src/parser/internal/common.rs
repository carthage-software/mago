use mago_database::file::HasFileId;

use crate::ast::ast::Keyword;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TokenKind;

impl<'input, 'arena> Parser<'input, 'arena> {
    /// Expects and consumes a keyword token.
    #[inline]
    pub(crate) fn expect_keyword(&mut self, kind: TokenKind) -> Result<Keyword<'arena>, ParseError> {
        let token = self.stream.eat(kind)?;
        Ok(Keyword { span: token.span_for(self.stream.file_id()), value: token.value })
    }

    /// Optionally consumes a keyword token if present.
    #[inline]
    pub(crate) fn maybe_expect_keyword(&mut self, kind: TokenKind) -> Result<Option<Keyword<'arena>>, ParseError> {
        if self.stream.is_at(kind)? { Ok(Some(self.expect_keyword(kind)?)) } else { Ok(None) }
    }

    /// Consumes any token and returns it as a keyword.
    #[inline]
    pub(crate) fn expect_any_keyword(&mut self) -> Result<Keyword<'arena>, ParseError> {
        let token = self.stream.consume()?;
        Ok(Keyword { span: token.span_for(self.stream.file_id()), value: token.value })
    }

    /// Expects and consumes one of the given token kinds.
    #[inline]
    pub(crate) fn expect_one_of_keyword(&mut self, kinds: &[TokenKind]) -> Result<Keyword<'arena>, ParseError> {
        let token = self.stream.consume()?;
        if kinds.contains(&token.kind) {
            Ok(Keyword { span: token.span_for(self.stream.file_id()), value: token.value })
        } else {
            Err(self.stream.unexpected(Some(token), kinds))
        }
    }
}
