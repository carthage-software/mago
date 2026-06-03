use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::identifier::Identifier;

pub mod annotation;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Type<'arena> {
    pub span: Span,
    pub kind: TypeKind<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "kind", content = "value")]
pub enum TypeKind<'arena> {
    Named(Identifier<'arena>),
    Union(&'arena [TypeKind<'arena>]),
    Intersection(&'arena [TypeKind<'arena>]),
    Null,
    Array,
    Callable,
    Static,
    Self_,
    Parent,
    Void,
    Never,
    Float,
    Bool(Option<bool>),
    Integer,
    String,
    Object,
    Mixed,
    Iterable,
}

impl HasSpan for Type<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
