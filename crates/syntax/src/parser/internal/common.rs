use crate::ast::ast::Keyword;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;
use crate::token::TokenKind;

impl<'arena> Parser<'arena> {
    /// Expects and consumes a keyword token.
    #[inline]
    pub(crate) fn expect_keyword(
        &self,
        stream: &mut TokenStream<'_, 'arena>,
        kind: TokenKind,
    ) -> Result<Keyword<'arena>, ParseError> {
        let token = stream.eat(kind)?;
        Ok(Keyword { span: token.span, value: token.value })
    }

    /// Optionally consumes a keyword token if present.
    #[inline]
    pub(crate) fn maybe_expect_keyword(
        &self,
        stream: &mut TokenStream<'_, 'arena>,
        kind: TokenKind,
    ) -> Result<Option<Keyword<'arena>>, ParseError> {
        if stream.is_at(kind)? { Ok(Some(self.expect_keyword(stream, kind)?)) } else { Ok(None) }
    }

    /// Consumes any token and returns it as a keyword.
    #[inline]
    pub(crate) fn expect_any_keyword(
        &self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Keyword<'arena>, ParseError> {
        let token = stream.consume()?;
        Ok(Keyword { span: token.span, value: token.value })
    }

    /// Expects and consumes one of the given token kinds.
    #[inline]
    pub(crate) fn expect_one_of_keyword(
        &self,
        stream: &mut TokenStream<'_, 'arena>,
        kinds: &[TokenKind],
    ) -> Result<Keyword<'arena>, ParseError> {
        let token = stream.consume()?;
        if kinds.contains(&token.kind) {
            Ok(Keyword { span: token.span, value: token.value })
        } else {
            Err(stream.unexpected(Some(token), kinds))
        }
    }
}
