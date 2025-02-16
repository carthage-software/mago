use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;
use crate::ast::keyword::Keyword;
use crate::ast::terminator::Terminator;

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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Continue<'a> {
    pub r#continue: Keyword,
    pub level: Option<Expression<'a>>,
    pub terminator: Terminator,
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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Break<'a> {
    pub r#break: Keyword,
    pub level: Option<Expression<'a>>,
    pub terminator: Terminator,
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
