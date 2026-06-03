use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::argument::Argument;
use crate::ir::identifier::Identifier;

/// Represents a single attribute.
///
/// Example: `Foo` in `#[Foo]`, `Bar(1)` in `#[Bar(1)]`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Attribute<'arena, S, D, E> {
    pub span: Span,
    pub class: Identifier<'arena>,
    pub arguments: &'arena [Argument<'arena, S, D, E>],
}

impl<S, D, E> HasSpan for Attribute<'_, S, D, E> {
    fn span(&self) -> Span {
        self.span
    }
}
