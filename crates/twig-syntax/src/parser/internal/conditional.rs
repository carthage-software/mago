use mago_allocator::prelude::*;
use mago_span::Position;

use crate::cst::Conditional;
use crate::cst::Expression;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigTokenKind;

impl<'arena, A> Parser<'_, 'arena, A>
where
    A: Arena,
{
    /// Parse a conditional expression starting at `?`.
    ///
    /// Accepts every Twig form:
    /// * `cond ? then : else`
    /// * `cond ? then`
    /// * `cond ? : else`
    pub(crate) fn parse_conditional(
        &mut self,
        condition: Expression<'arena>,
    ) -> Result<Expression<'arena>, ParseError<'arena>> {
        let q_tok = self.stream.consume()?;
        let question_mark = self.stream.span_of(&q_tok);

        // `a ? : c`: colon directly after the `?`, no `then` branch.
        if let Some(c_tok) = self.stream.try_consume(TwigTokenKind::Colon)? {
            let colon = self.stream.span_of(&c_tok);
            let else_expr = self.parse_expression()?;
            return Ok(Expression::Conditional(Conditional {
                condition: self.alloc(condition),
                question_mark,
                then: None,
                colon: Some(colon),
                r#else: Some(self.alloc(else_expr)),
            }));
        }

        let then_expr = self.parse_expression()?;
        let then_ref: &'arena Expression<'arena> = self.alloc(then_expr);

        // Optional `: else`: Twig accepts `a ? b` with no else.
        if let Some(c_tok) = self.stream.try_consume(TwigTokenKind::Colon)? {
            let colon = self.stream.span_of(&c_tok);
            let else_expr = self.parse_expression()?;
            return Ok(Expression::Conditional(Conditional {
                condition: self.alloc(condition),
                question_mark,
                then: Some(then_ref),
                colon: Some(colon),
                r#else: Some(self.alloc(else_expr)),
            }));
        }

        Ok(Expression::Conditional(Conditional {
            condition: self.alloc(condition),
            question_mark,
            then: Some(then_ref),
            colon: None,
            r#else: None,
        }))
    }

    /// Parse `a ?: c` or `a ? : c`, keeping each punctuation span.
    pub(crate) fn parse_elvis(
        &mut self,
        condition: Expression<'arena>,
    ) -> Result<Expression<'arena>, ParseError<'arena>> {
        let qc_tok = self.stream.consume()?;
        let full = self.stream.span_of(&qc_tok);
        let question_mark = self.stream.span(full.start, Position::new(full.start.offset + 1));
        let colon = self.stream.span(Position::new(full.end.offset - 1), full.end);
        let else_expr = self.parse_expression()?;
        Ok(Expression::Conditional(Conditional {
            condition: self.alloc(condition),
            question_mark,
            then: None,
            colon: Some(colon),
            r#else: Some(self.alloc(else_expr)),
        }))
    }
}
