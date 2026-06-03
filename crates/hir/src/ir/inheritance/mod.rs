use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::identifier::Identifier;

pub mod annotation;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Implements<'arena> {
    pub span: Span,
    pub types: &'arena [Identifier<'arena>],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ExtendsOne<'arena> {
    pub span: Span,
    pub r#type: Identifier<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ExtendsOneOrMore<'arena> {
    pub span: Span,
    pub types: &'arena [Identifier<'arena>],
}

impl HasSpan for Implements<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for ExtendsOne<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for ExtendsOneOrMore<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
