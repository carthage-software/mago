use mago_database::file::HasFileId;

use crate::ast::Identifier;
use crate::ast::Keyword;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::is_keyword_usable_as_name;
use crate::token::TwigToken;
use crate::token::TwigTokenKind;

impl<'input, 'arena> Parser<'input, 'arena> {
    /// Convert a token into a [`Keyword`].
    #[inline]
    pub(crate) fn keyword_from(&self, token: &TwigToken<'arena>) -> Keyword<'arena> {
        Keyword { span: token.span_for(self.stream.file_id()), value: token.value }
    }

    /// Convert a token into an [`Identifier`].
    #[inline]
    pub(crate) fn identifier_from(&self, token: &TwigToken<'arena>) -> Identifier<'arena> {
        Identifier { span: token.span_for(self.stream.file_id()), value: token.value }
    }

    /// Expect a plain `Name` token and wrap it as an [`Identifier`].
    #[inline]
    pub(crate) fn expect_identifier(&mut self, what: &str) -> Result<Identifier<'arena>, ParseError> {
        let token = self.stream.expect_kind(TwigTokenKind::Name, what)?;
        Ok(self.identifier_from(&token))
    }

    /// Accept either a `Name` or any word-keyword that is also usable as
    /// an identifier (e.g. `in`, `is`, `and`, `matches`, `divisible`).
    #[inline]
    pub(crate) fn expect_flexible_identifier(&mut self, what: &str) -> Result<Identifier<'arena>, ParseError> {
        match self.stream.lookahead(0)? {
            Some(token) if token.kind == TwigTokenKind::Name || is_keyword_usable_as_name(token.kind) => {
                self.stream.consume()?;
                Ok(self.identifier_from(&token))
            }
            Some(token) => Err(ParseError::UnexpectedToken(what.to_string(), token.span_for(self.stream.file_id()))),
            None => {
                Err(ParseError::UnexpectedEof(self.stream.file_id(), what.to_string(), self.stream.current_position()))
            }
        }
    }

    /// Optionally consume a `Name` token with a specific literal value
    /// (e.g. `"with"`, `"only"`, `"as"`) and return it as a keyword.
    #[inline]
    pub(crate) fn try_consume_name_keyword(&mut self, name: &str) -> Result<Option<Keyword<'arena>>, ParseError> {
        match self.stream.try_consume_name(name)? {
            Some(token) => Ok(Some(self.keyword_from(&token))),
            None => Ok(None),
        }
    }
}
