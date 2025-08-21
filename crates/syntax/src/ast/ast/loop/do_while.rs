use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::statement::Statement;
use crate::ast::ast::terminator::Terminator;

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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct DoWhile<'arena> {
    pub r#do: Keyword<'arena>,
    pub statement: &'arena Statement<'arena>,
    pub r#while: Keyword<'arena>,
    pub left_parenthesis: Span,
    pub condition: &'arena Expression<'arena>,
    pub right_parenthesis: Span,
    pub terminator: Terminator<'arena>,
}

impl HasSpan for DoWhile<'_> {
    fn span(&self) -> Span {
        Span::between(self.r#do.span(), self.terminator.span())
    }
}
