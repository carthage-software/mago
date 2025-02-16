use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::type_hint::Hint;

/// Represents a function-like return type hint in PHP.
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct FunctionLikeReturnTypeHint<'a> {
    pub colon: Span,
    pub hint: Hint<'a>,
}

impl HasSpan for FunctionLikeReturnTypeHint<'_> {
    fn span(&self) -> Span {
        Span::between(self.colon, self.hint.span())
    }
}
