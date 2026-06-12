#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::identifier::Identifier;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_slice_into;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Implements<'arena> {
    pub span: Span,
    pub types: &'arena [Identifier<'arena>],
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Extends<'arena> {
    pub span: Span,
    pub types: &'arena [Identifier<'arena>],
}

impl HasSpan for Implements<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for Extends<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl CopyInto for Implements<'_> {
    type Output<'arena> = Implements<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Implements { span: self.span, types: copy_slice_into(self.types, arena) }
    }
}

impl CopyInto for Extends<'_> {
    type Output<'arena> = Extends<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Extends { span: self.span, types: copy_slice_into(self.types, arena) }
    }
}
