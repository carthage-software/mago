#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_span::Span;

use mago_allocator::copy::CopyInto;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Name<'arena> {
    pub span: Span,
    pub value: &'arena [u8],
}

impl HasSpan for Name<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl CopyInto for Name<'_> {
    type Output<'arena> = Name<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Name { span: self.span, value: arena.alloc_slice_copy(self.value) }
    }
}
