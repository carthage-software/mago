use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Visibility {
    pub span: Span,
    pub kind: VisibilityKind,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum VisibilityKind {
    Public,
    Protected,
    Private,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Modifier {
    pub span: Span,
    pub kind: ModifierKind,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum ModifierKind {
    Static,
    Final,
    Abstract,
    Readonly,
    Public,
    PublicSet,
    Protected,
    ProtectedSet,
    Private,
    PrivateSet,
}

impl HasSpan for Visibility {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for Modifier {
    fn span(&self) -> Span {
        self.span
    }
}
