#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_span::Span;

use mago_allocator::copy::CopyInto;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Visibility {
    pub span: Span,
    pub kind: VisibilityKind,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum VisibilityKind {
    Public,
    Protected,
    Private,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Modifier {
    pub span: Span,
    pub kind: ModifierKind,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
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

impl CopyInto for Visibility {
    type Output<'arena> = Visibility;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Visibility { span: self.span, kind: self.kind }
    }
}

impl CopyInto for VisibilityKind {
    type Output<'arena> = VisibilityKind;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl CopyInto for Modifier {
    type Output<'arena> = Modifier;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Modifier { span: self.span, kind: self.kind }
    }
}

impl CopyInto for ModifierKind {
    type Output<'arena> = ModifierKind;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
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
