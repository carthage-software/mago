#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_span::Span;

use mago_allocator::copy::CopyInto;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Identifier<'arena> {
    pub span: Span,
    pub value: &'arena [u8],
    pub kind: IdentifierKind,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum IdentifierKind {
    Local,
    Qualified,
    FullyQualified,
}

impl<'arena> Identifier<'arena> {
    #[inline]
    #[must_use]
    pub const fn is_local(&self) -> bool {
        matches!(self.kind, IdentifierKind::Local)
    }

    #[inline]
    #[must_use]
    pub const fn is_qualified(&self) -> bool {
        matches!(self.kind, IdentifierKind::Qualified)
    }

    #[inline]
    #[must_use]
    pub const fn is_fully_qualified(&self) -> bool {
        matches!(self.kind, IdentifierKind::FullyQualified)
    }

    #[inline]
    #[must_use]
    pub fn last_segment(&self) -> &'arena [u8] {
        match memchr::memrchr(b'\\', self.value) {
            Some(pos) => &self.value[pos + 1..],
            None => self.value,
        }
    }
}

impl HasSpan for Identifier<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl CopyInto for Identifier<'_> {
    type Output<'arena> = Identifier<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Identifier { span: self.span, value: arena.alloc_slice_copy(self.value), kind: self.kind }
    }
}
