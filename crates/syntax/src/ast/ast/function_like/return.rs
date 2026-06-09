use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::type_hint::Hint;

/// Represents a function-like return type hint in PHP.
#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FunctionLikeReturnTypeHint<'arena> {
    pub colon: Span,
    pub hint: Hint<'arena>,
}

impl HasSpan for FunctionLikeReturnTypeHint<'_> {
    fn span(&self) -> Span {
        Span::between(self.colon, self.hint.span())
    }
}
