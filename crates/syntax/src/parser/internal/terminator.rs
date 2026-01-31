use crate::T;
use crate::ast::ast::ClosingTag;
use crate::ast::ast::Terminator;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;
use crate::token::TokenKind;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_optional_terminator(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Option<Terminator<'arena>>, ParseError> {
        let next = stream.lookahead(0)?;
        Ok(match next.map(|t| t.kind) {
            Some(T![";" | "?>"]) => Some(self.parse_terminator(stream)?),
            _ => None,
        })
    }

    pub(crate) fn parse_terminator(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Terminator<'arena>, ParseError> {
        if stream.is_at(TokenKind::Semicolon)? {
            let token = stream.consume()?;

            return Ok(Terminator::Semicolon(token.span));
        }

        if stream.is_at(TokenKind::CloseTag)? {
            let closing_tag_token = stream.consume()?;
            let closing_tag = ClosingTag { span: closing_tag_token.span };

            let next = stream.lookahead(0)?;
            if matches!(next.map(|t| t.kind), Some(T!["<?php" | "<?"])) {
                return Ok(Terminator::TagPair(closing_tag, self.parse_opening_tag(stream)?));
            } else {
                return Ok(Terminator::ClosingTag(closing_tag));
            }
        }

        let Some(next) = stream.lookahead(0)? else {
            return Err(stream.unexpected(None, T![";", "?>"]));
        };

        self.errors.push(stream.unexpected(Some(next), T![";", "?>"]));

        Ok(Terminator::Missing(next.span))
    }
}
