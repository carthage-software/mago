use serde::Deserialize;
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct Continue {
    pub r#continue: Keyword,
    pub level: Option<Expression>,
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct Break {
    pub r#break: Keyword,
    pub level: Option<Expression>,
    pub terminator: Terminator,
}

impl HasSpan for Continue {
    fn span(&self) -> Span {
        self.r#continue.span().join(self.terminator.span())
    }
}

impl HasSpan for Break {
    fn span(&self) -> Span {
        self.r#break.span().join(self.terminator.span())
    }
}
