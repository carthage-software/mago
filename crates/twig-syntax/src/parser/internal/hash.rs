use crate::ast::Expression;
use crate::ast::HashMap;
use crate::ast::HashMapEntry;
use crate::ast::Name;
use crate::ast::Number;
use crate::ast::Parenthesized;
use crate::ast::StringLiteral;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigTokenKind;

impl<'input, 'arena> Parser<'input, 'arena> {
    /// Parse a hash map literal: `{ a: 1, 'b': 2, ...rest }`.
    pub(crate) fn parse_hash_map(&mut self) -> Result<Expression<'arena>, ParseError> {
        let result = self.parse_comma_separated_sequence(
            TwigTokenKind::LeftBrace,
            TwigTokenKind::RightBrace,
            Self::parse_hash_map_entry,
        )?;

        Ok(Expression::HashMap(HashMap {
            left_brace: result.open,
            entries: result.sequence,
            right_brace: result.close,
        }))
    }

    /// Parse a single hash map entry - a spread (`...expr`), a shorthand
    /// (`name`, `{ a, b }`), or a full `key: value` pair.
    pub(crate) fn parse_hash_map_entry(&mut self) -> Result<HashMapEntry<'arena>, ParseError> {
        if let Some(dots_tok) = self.stream.try_consume(TwigTokenKind::DotDotDot)? {
            let ellipsis = Some(self.stream.span_of(&dots_tok));
            let value = self.parse_expression()?;
            return Ok(HashMapEntry { ellipsis, key: None, colon: None, value });
        }

        let key_tok = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[TwigTokenKind::Name]))?;

        // Shorthand entry: `{ a, b }` becomes key `'a'`, value `a`.
        if key_tok.kind == TwigTokenKind::Name {
            let next = self.stream.peek_kind(1)?;
            if matches!(next, Some(TwigTokenKind::Comma) | Some(TwigTokenKind::RightBrace) | None) {
                self.stream.consume()?;
                let span = self.stream.span_of(&key_tok);
                let key = Expression::String(StringLiteral { raw: key_tok.value, span });
                let value = Expression::Name(Name { name: key_tok.value, span });
                return Ok(HashMapEntry { ellipsis: None, key: Some(key), colon: None, value });
            }
        }

        let key: Expression<'arena> = match key_tok.kind {
            TwigTokenKind::Name | TwigTokenKind::StringSingleQuoted | TwigTokenKind::StringDoubleQuoted => {
                self.stream.consume()?;
                Expression::String(StringLiteral { raw: key_tok.value, span: self.stream.span_of(&key_tok) })
            }
            TwigTokenKind::Number => {
                self.stream.consume()?;
                let is_float = key_tok.value.contains('.');
                Expression::Number(Number { raw: key_tok.value, is_float, span: self.stream.span_of(&key_tok) })
            }
            TwigTokenKind::LeftParen => {
                let lp_tok = self.stream.consume()?;
                let left_parenthesis = self.stream.span_of(&lp_tok);
                let inner = self.parse_expression()?;
                let rp_tok = self.stream.expect_kind(TwigTokenKind::RightParen, "expected `)`")?;
                let right_parenthesis = self.stream.span_of(&rp_tok);
                Expression::Parenthesized(Parenthesized {
                    left_parenthesis,
                    inner: self.alloc(inner),
                    right_parenthesis,
                })
            }
            _ => {
                return Err(ParseError::UnexpectedToken(
                    format!("invalid hash key: {:?} {:?}", key_tok.kind, key_tok.value),
                    self.stream.span_of(&key_tok),
                ));
            }
        };

        let colon_tok = self.stream.expect_kind(TwigTokenKind::Colon, "expected `:`")?;
        let colon = Some(self.stream.span_of(&colon_tok));
        let value = self.parse_expression()?;
        Ok(HashMapEntry { ellipsis: None, key: Some(key), colon, value })
    }
}
