use mago_span::HasSpan;

use crate::ast::ArrowFunction;
use crate::ast::Expression;
use crate::ast::Identifier;
use crate::ast::Parenthesized;
use crate::ast::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigTokenKind;

impl<'input, 'arena> Parser<'input, 'arena> {
    /// Parse a `(` group - which may be a parenthesised expression, an
    /// arrow function's parameter list, or an empty-parameter arrow.
    /// Handles: `()`, `( expr )`, `( name ) => body`, `(a, b) => body`,
    /// `() => body`.
    pub(crate) fn parse_group(&mut self) -> Result<Expression<'arena>, ParseError> {
        let lp_tok = self.stream.consume()?;
        let left_parenthesis = self.stream.span_of(&lp_tok);

        if self.stream.is_at(TwigTokenKind::RightParen)? {
            let rp_tok = self.stream.consume()?;
            let right_parenthesis = self.stream.span_of(&rp_tok);
            if let Some(arrow_tok) = self.stream.try_consume(TwigTokenKind::FatArrow)? {
                let fat_arrow = self.stream.span_of(&arrow_tok);
                let body = self.parse_expression()?;
                return Ok(Expression::ArrowFunction(ArrowFunction {
                    left_parenthesis: Some(left_parenthesis),
                    parameters: TokenSeparatedSequence::new(self.new_vec(), self.new_vec()),
                    right_parenthesis: Some(right_parenthesis),
                    fat_arrow,
                    body: self.alloc(body),
                }));
            }
            let next = self.stream.lookahead(0)?;
            return Err(self.stream.unexpected(next, &[]));
        }

        let inner = self.parse_expression()?;

        if self.stream.is_at(TwigTokenKind::Comma)? {
            return self.finish_arrow_parameters(left_parenthesis, inner);
        }

        let rp_tok = self.stream.expect_kind(TwigTokenKind::RightParen, "expected `)`")?;
        let right_parenthesis = self.stream.span_of(&rp_tok);

        if let Some(arrow_tok) = self.stream.try_consume(TwigTokenKind::FatArrow)?
            && let Expression::Name(ref n) = inner
        {
            let fat_arrow = self.stream.span_of(&arrow_tok);
            let mut param_nodes = self.new_vec();
            param_nodes.push(Identifier { span: n.span, value: n.name });
            let body = self.parse_expression()?;
            let parameters = TokenSeparatedSequence::new(param_nodes, self.new_vec());
            return Ok(Expression::ArrowFunction(ArrowFunction {
                left_parenthesis: Some(left_parenthesis),
                parameters,
                right_parenthesis: Some(right_parenthesis),
                fat_arrow,
                body: self.alloc(body),
            }));
        }

        Ok(Expression::Parenthesized(Parenthesized { left_parenthesis, inner: self.alloc(inner), right_parenthesis }))
    }

    fn finish_arrow_parameters(
        &mut self,
        left_parenthesis: mago_span::Span,
        inner: Expression<'arena>,
    ) -> Result<Expression<'arena>, ParseError> {
        let mut parameters = self.new_vec();
        let mut commas = self.new_vec();
        let Expression::Name(ref n) = inner else {
            return Err(ParseError::UnexpectedToken(
                "arrow function parameters must be simple names".to_string(),
                inner.span(),
            ));
        };
        parameters.push(Identifier { span: n.span, value: n.name });

        while let Some(comma) = self.stream.try_consume(TwigTokenKind::Comma)? {
            commas.push(comma);
            if self.stream.is_at(TwigTokenKind::RightParen)? {
                break;
            }
            let name_tok = self.stream.expect_kind(TwigTokenKind::Name, "expected parameter name")?;
            parameters.push(self.identifier_from(&name_tok));
        }

        let rp_tok = self.stream.expect_kind(TwigTokenKind::RightParen, "expected `)`")?;
        let right_parenthesis = self.stream.span_of(&rp_tok);

        let arrow_tok = match self.stream.try_consume(TwigTokenKind::FatArrow)? {
            Some(token) => token,
            None => {
                let next = self.stream.lookahead(0)?;
                return Err(self.stream.unexpected(next, &[TwigTokenKind::FatArrow]));
            }
        };
        let fat_arrow = self.stream.span_of(&arrow_tok);
        let body = self.parse_expression()?;
        let parameters = TokenSeparatedSequence::new(parameters, commas);
        Ok(Expression::ArrowFunction(ArrowFunction {
            left_parenthesis: Some(left_parenthesis),
            parameters,
            right_parenthesis: Some(right_parenthesis),
            fat_arrow,
            body: self.alloc(body),
        }))
    }

    /// Build the parameter list for a trailing-arrow function produced
    /// from a bare expression (`v => body` or `(v) => body` parsed as a
    /// parenthesised expression before the `=>`).
    pub(crate) fn arrow_parameters_from(
        &self,
        expression: &Expression<'arena>,
    ) -> TokenSeparatedSequence<'arena, Identifier<'arena>> {
        let mut nodes = self.new_vec();
        let commas = self.new_vec();
        match expression {
            Expression::Name(n) => nodes.push(Identifier { span: n.span, value: n.name }),
            Expression::Parenthesized(g) => {
                if let Expression::Name(n) = g.inner {
                    nodes.push(Identifier { span: n.span, value: n.name });
                }
            }
            _ => {}
        }
        TokenSeparatedSequence::new(nodes, commas)
    }
}
