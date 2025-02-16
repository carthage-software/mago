use bumpalo::boxed::Box;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;
use crate::ast::keyword::Keyword;
use crate::ast::statement::Statement;
use crate::ast::terminator::Terminator;

/// Represents a do-while statement in PHP.
///
/// Example:
///
/// ```php
/// <?php
///
/// $i = 0;
/// do {
///   echo $i;
///   $i++;
/// } while ($i < 10);
/// ```
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct DoWhile<'a> {
    pub r#do: Keyword,
    pub statement: Box<'a, Statement<'a>>,
    pub r#while: Keyword,
    pub left_parenthesis: Span,
    pub condition: Box<'a, Expression<'a>>,
    pub right_parenthesis: Span,
    pub terminator: Terminator,
}

impl HasSpan for DoWhile<'_> {
    fn span(&self) -> Span {
        Span::between(self.r#do.span(), self.terminator.span())
    }
}
