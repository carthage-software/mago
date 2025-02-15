use serde::Deserialize;
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct DoWhile {
    pub r#do: Keyword,
    pub statement: Box<Statement>,
    pub r#while: Keyword,
    pub left_parenthesis: Span,
    pub condition: Box<Expression>,
    pub right_parenthesis: Span,
    pub terminator: Terminator,
}

impl HasSpan for DoWhile {
    fn span(&self) -> Span {
        Span::between(self.r#do.span(), self.terminator.span())
    }
}
