use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::terminator::Terminator;

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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Return<'arena> {
    pub r#return: Keyword<'arena>,
    pub value: Option<Expression<'arena>>,
    pub terminator: Terminator<'arena>,
}

impl HasSpan for Return<'_> {
    fn span(&self) -> Span {
        self.r#return.span().join(self.terminator.span())
    }
}
