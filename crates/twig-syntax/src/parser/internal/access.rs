use mago_span::Span;

use crate::ast::Call;
use crate::ast::Expression;
use crate::ast::GetAttribute;
use crate::ast::GetItem;
use crate::ast::Identifier;
use crate::ast::MethodCall;
use crate::ast::Name;
use crate::ast::Number;
use crate::ast::Slice;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::is_keyword_usable_as_name;
use crate::parser::stream::looks_like_identifier;
use crate::token::TwigTokenKind;

impl<'arena> Parser<'_, 'arena> {
    /// Parse `object . attribute` or `object ?. attribute`. If the
    /// attribute is immediately followed by `(`, the result is a
    /// [`MethodCall`]; otherwise a [`GetAttribute`]. An `object.(expr)`
    /// form becomes a `GetAttribute` with a non-name attribute, optionally
    /// wrapped in a `Call` if followed by arguments.
    pub(crate) fn parse_dot_access(
        &mut self,
        object: Expression<'arena>,
        dot: Span,
        null_safe: bool,
    ) -> Result<Expression<'arena>, ParseError> {
        let token = self
            .stream
            .lookahead(0)?
            .ok_or_else(|| self.stream.unexpected(None, &[TwigTokenKind::Name, TwigTokenKind::Number]))?;

        let (attribute, method): (Expression<'arena>, Option<Identifier<'arena>>) = match token.kind {
            TwigTokenKind::LeftParen => {
                self.stream.consume()?;
                let inner = self.parse_expression()?;
                self.stream.expect_kind(TwigTokenKind::RightParen, "expected `)`")?;
                (inner, None)
            }
            TwigTokenKind::Name => {
                self.stream.consume()?;
                let ident = self.identifier_from(&token);
                (Expression::Name(Name { name: token.value, span: self.stream.span_of(&token) }), Some(ident))
            }
            TwigTokenKind::Number => {
                self.stream.consume()?;
                let is_float = token.value.contains('.');
                (Expression::Number(Number { raw: token.value, is_float, span: self.stream.span_of(&token) }), None)
            }
            kind if is_keyword_usable_as_name(kind) && looks_like_identifier(token.value) => {
                self.stream.consume()?;
                let ident = self.identifier_from(&token);
                (Expression::Name(Name { name: token.value, span: self.stream.span_of(&token) }), Some(ident))
            }
            _ => {
                return Err(ParseError::UnexpectedToken(
                    "expected attribute name or number after `.`".to_string(),
                    self.stream.span_of(&token),
                ));
            }
        };

        if self.stream.is_at(TwigTokenKind::LeftParen)? {
            let argument_list = self.parse_argument_list()?;
            if let Some(method) = method {
                return Ok(Expression::MethodCall(MethodCall {
                    object: self.alloc(object),
                    dot,
                    null_safe,
                    method,
                    argument_list,
                }));
            }

            let get = Expression::GetAttribute(GetAttribute {
                object: self.alloc(object),
                dot,
                null_safe,
                attribute: self.alloc(attribute),
            });

            return Ok(Expression::Call(Call { callee: self.alloc(get), argument_list }));
        }

        Ok(Expression::GetAttribute(GetAttribute {
            object: self.alloc(object),
            dot,
            null_safe,
            attribute: self.alloc(attribute),
        }))
    }

    /// Parse `object [ ... ]`. Chooses between [`GetItem`] (`a[b]`) and
    /// [`Slice`] (`a[:b]`, `a[b:]`, `a[b:c]`, `a[:]`) based on whether a
    /// colon appears between the brackets.
    pub(crate) fn parse_bracket_access(
        &mut self,
        object: Expression<'arena>,
    ) -> Result<Expression<'arena>, ParseError> {
        let lb_tok = self.stream.consume()?;
        let left_bracket = self.stream.span_of(&lb_tok);

        if let Some(c_tok) = self.stream.try_consume(TwigTokenKind::Colon)? {
            let colon = self.stream.span_of(&c_tok);
            let length = self.parse_optional_expression_until(TwigTokenKind::RightBracket)?;
            let rb_tok = self.stream.expect_kind(TwigTokenKind::RightBracket, "expected `]`")?;
            let right_bracket = self.stream.span_of(&rb_tok);
            return Ok(Expression::Slice(Slice {
                object: self.alloc(object),
                left_bracket,
                start: None,
                colon,
                length,
                right_bracket,
            }));
        }

        let first = self.parse_expression()?;
        if let Some(c_tok) = self.stream.try_consume(TwigTokenKind::Colon)? {
            let colon = self.stream.span_of(&c_tok);
            let start: &Expression<'arena> = self.alloc(first);
            let length = self.parse_optional_expression_until(TwigTokenKind::RightBracket)?;
            let rb_tok = self.stream.expect_kind(TwigTokenKind::RightBracket, "expected `]`")?;
            let right_bracket = self.stream.span_of(&rb_tok);
            return Ok(Expression::Slice(Slice {
                object: self.alloc(object),
                left_bracket,
                start: Some(start),
                colon,
                length,
                right_bracket,
            }));
        }

        let rb_tok = self.stream.expect_kind(TwigTokenKind::RightBracket, "expected `]`")?;
        let right_bracket = self.stream.span_of(&rb_tok);
        Ok(Expression::GetItem(GetItem {
            object: self.alloc(object),
            left_bracket,
            index: self.alloc(first),
            right_bracket,
        }))
    }

    /// Parse an expression unless the next token is `terminator`.
    fn parse_optional_expression_until(
        &mut self,
        terminator: TwigTokenKind,
    ) -> Result<Option<&'arena Expression<'arena>>, ParseError> {
        if self.stream.is_at(terminator)? {
            return Ok(None);
        }
        let expr = self.parse_expression()?;
        Ok(Some(self.alloc(expr)))
    }
}
