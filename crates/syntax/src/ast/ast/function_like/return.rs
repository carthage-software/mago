use serde::Deserialize;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::type_hint::Hint;

/// Represents a function-like return type hint in PHP.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct FunctionLikeReturnTypeHint {
    pub colon: Span,
    pub hint: Hint,
}

impl HasSpan for FunctionLikeReturnTypeHint {
    fn span(&self) -> Span {
        Span::between(self.colon, self.hint.span())
    }
}
