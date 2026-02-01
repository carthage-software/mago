use crate::T;
use crate::ast::ast::ClosingTag;
use crate::ast::ast::Terminator;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TokenKind;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_optional_terminator(&mut self) -> Result<Option<Terminator<'arena>>, ParseError> {
        let next = self.stream.lookahead(0)?;
        Ok(match next.map(|t| t.kind) {
            Some(T![";" | "?>"]) => Some(self.parse_terminator()?),
            _ => None,
        })
    }

    pub(crate) fn parse_terminator(&mut self) -> Result<Terminator<'arena>, ParseError> {
        if self.stream.is_at(TokenKind::Semicolon)? {
            let token = self.stream.consume()?;

            return Ok(Terminator::Semicolon(token.span));
        }

        if self.stream.is_at(TokenKind::CloseTag)? {
            let closing_tag_token = self.stream.consume()?;
            let closing_tag = ClosingTag { span: closing_tag_token.span };

            let next = self.stream.lookahead(0)?;
            if matches!(next.map(|t| t.kind), Some(T!["<?php" | "<?"])) {
                return Ok(Terminator::TagPair(closing_tag, self.parse_opening_tag()?));
            } else {
                return Ok(Terminator::ClosingTag(closing_tag));
            }
        }

        let Some(next) = self.stream.lookahead(0)? else {
            return Err(self.stream.unexpected(None, T![";", "?>"]));
        };

        self.errors.push(self.stream.unexpected(Some(next), T![";", "?>"]));

        Ok(Terminator::Missing(next.span))
    }
}
