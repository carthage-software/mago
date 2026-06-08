use crate::ast::Expression;
use crate::ast::Filter;
use crate::error::ParseError;
use crate::parser::Parser;
use mago_allocator::prelude::*;

impl<'arena, A> Parser<'_, 'arena, A>
where
    A: Arena,
{
    /// Parse a filter suffix: `operand | name ( args? )`.  The leading
    /// `|` must already have been consumed by the caller.
    pub(crate) fn parse_filter(
        &mut self,
        operand: Expression<'arena>,
        pipe: mago_span::Span,
    ) -> Result<Expression<'arena>, ParseError<'arena>> {
        let name = self.expect_identifier(b"expected filter name after `|`")?;
        let argument_list = self.parse_optional_argument_list()?;
        Ok(Expression::Filter(Filter { operand: self.alloc(operand), pipe, name, argument_list }))
    }
}
