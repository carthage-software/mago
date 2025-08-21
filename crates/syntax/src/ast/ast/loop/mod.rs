use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::terminator::Terminator;

pub mod do_while;
pub mod r#for;
pub mod foreach;
pub mod r#while;

/// Represents a continue statement in PHP.
///
/// Example:
///
/// ```php
/// <?php
///
/// for ($i = 0; $i < 10; $i++) {
///   if ($i === 5) {
///     continue;
///   }
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Continue<'arena> {
    pub r#continue: Keyword<'arena>,
    pub level: Option<Expression<'arena>>,
    pub terminator: Terminator<'arena>,
}

/// Represents a break statement in PHP.
///
/// Example:
///
/// ```php
/// <?php
///
/// for ($i = 0; $i < 10; $i++) {
///   if ($i === 5) {
///     break;
///   }
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Break<'arena> {
    pub r#break: Keyword<'arena>,
    pub level: Option<Expression<'arena>>,
    pub terminator: Terminator<'arena>,
}

impl HasSpan for Continue<'_> {
    fn span(&self) -> Span {
        self.r#continue.span().join(self.terminator.span())
    }
}

impl HasSpan for Break<'_> {
    fn span(&self) -> Span {
        self.r#break.span().join(self.terminator.span())
    }
}
