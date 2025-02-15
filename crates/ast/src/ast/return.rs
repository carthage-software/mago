use serde::Deserialize;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;
use crate::ast::keyword::Keyword;
use crate::ast::terminator::Terminator;

/// Represents a PHP `return` statement.
///
/// # Examples
///
/// ```php
/// <?php
///
/// function example(): int {
///     return 1;
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct Return {
    pub r#return: Keyword,
    pub value: Option<Expression>,
    pub terminator: Terminator,
}

impl HasSpan for Return {
    fn span(&self) -> Span {
        self.r#return.span().join(self.terminator.span())
    }
}
