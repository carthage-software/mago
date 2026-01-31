use ordered_float::OrderedFloat;

use mago_syntax_core::utils::parse_literal_float;
use mago_syntax_core::utils::parse_literal_integer;
use mago_syntax_core::utils::parse_literal_string_in;

use crate::T;
use crate::ast::ast::Keyword;
use crate::ast::ast::Literal;
use crate::ast::ast::LiteralFloat;
use crate::ast::ast::LiteralInteger;
use crate::ast::ast::LiteralString;
use crate::ast::ast::LiteralStringKind;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_literal(&mut self) -> Result<Literal<'arena>, ParseError> {
        let token = self.stream.consume()?;

        Ok(match &token.kind {
            T![LiteralFloat] => Literal::Float(LiteralFloat {
                span: token.span,
                raw: token.value,
                value: OrderedFloat(parse_literal_float(token.value).unwrap_or_else(|| {
                    unreachable!("lexer generated invalid float `{}`; this should never happen.", token.value)
                })),
            }),
            T![LiteralInteger] => Literal::Integer(LiteralInteger {
                span: token.span,
                raw: token.value,
                value: parse_literal_integer(token.value),
            }),
            T!["true"] => Literal::True(Keyword { span: token.span, value: token.value }),
            T!["false"] => Literal::False(Keyword { span: token.span, value: token.value }),
            T!["null"] => Literal::Null(Keyword { span: token.span, value: token.value }),
            T![LiteralString] => Literal::String(LiteralString {
                kind: Some(if token.value.starts_with('"') {
                    LiteralStringKind::DoubleQuoted
                } else {
                    LiteralStringKind::SingleQuoted
                }),
                span: token.span,
                raw: token.value,
                value: parse_literal_string_in(self.arena, token.value, None, true),
            }),
            T![PartialLiteralString] => {
                let kind = if token.value.starts_with('"') {
                    LiteralStringKind::DoubleQuoted
                } else {
                    LiteralStringKind::SingleQuoted
                };

                return Err(ParseError::UnclosedLiteralString(kind, token.span));
            }
            _ => {
                return Err(self.stream.unexpected(
                    Some(token),
                    T!["true", "false", "null", LiteralFloat, LiteralInteger, LiteralString, PartialLiteralString],
                ));
            }
        })
    }
}
