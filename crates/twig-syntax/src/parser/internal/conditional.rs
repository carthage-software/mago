use mago_span::Position;

use crate::ast::Conditional;
use crate::ast::Expression;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigTokenKind;

impl<'input, 'arena> Parser<'input, 'arena> {
    /// Parse a conditional expression starting at `?`.
    ///
    /// Accepts every Twig form:
    /// * `cond ? then : else`
    /// * `cond ? then`
    /// * `cond ? : else`
    pub(crate) fn parse_conditional(
        &mut self,
        condition: Expression<'arena>,
    ) -> Result<Expression<'arena>, ParseError> {
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

    /// Parse the Elvis conditional form `a ?: c`. The lexer fuses `?:`
    /// into a single `QuestionColon` token; we split its span into the
    /// question-mark half and the colon half so the AST still reports
    /// both positions.
    pub(crate) fn parse_elvis(&mut self, condition: Expression<'arena>) -> Result<Expression<'arena>, ParseError> {
        let qc_tok = self.stream.consume()?;
        let full = self.stream.span_of(&qc_tok);
        let mid = Position::new(full.start.offset + 1);
        let question_mark = self.stream.span(full.start, mid);
        let colon = self.stream.span(mid, full.end);
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
