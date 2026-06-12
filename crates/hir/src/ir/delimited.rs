#[cfg(feature = "serde")]
use serde::Serialize;

use std::ops::Deref;
use std::slice::Iter;

use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_span::Span;

use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_slice_into;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Delimited<'arena, T> {
    pub span: Span,
    pub items: &'arena [T],
}

impl<'arena, T> Delimited<'arena, T> {
    #[must_use]
    pub fn as_slice(&self) -> &'arena [T] {
        self.items
    }
}

impl<T> Deref for Delimited<'_, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.items
    }
}

impl<'arena, T> IntoIterator for Delimited<'arena, T> {
    type Item = &'arena T;
    type IntoIter = Iter<'arena, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl<'arena, T> IntoIterator for &Delimited<'arena, T> {
    type Item = &'arena T;
    type IntoIter = Iter<'arena, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl<T> HasSpan for Delimited<'_, T> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<T> CopyInto for Delimited<'_, T>
where
    T: CopyInto,
{
    type Output<'arena> = Delimited<'arena, T::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Delimited { span: self.span, items: copy_slice_into(self.items, arena) }
    }
}
