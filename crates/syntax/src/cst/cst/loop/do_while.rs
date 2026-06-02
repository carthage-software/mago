use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::cst::expression::Expression;
use crate::cst::cst::keyword::Keyword;
use crate::cst::cst::statement::Statement;
use crate::cst::cst::terminator::Terminator;

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
