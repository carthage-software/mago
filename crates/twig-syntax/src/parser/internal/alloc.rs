use mago_allocator::prelude::*;

use crate::parser::Parser;

impl<'arena, A> Parser<'_, 'arena, A>
where
    A: Arena,
{
    /// A fresh arena-backed vector.
    #[inline]
    pub(crate) fn new_vec<T>(&self) -> Vec<'arena, T, A> {
        Vec::new_in(self.arena)
    }

    /// Arena-allocate a value and return an `&'arena T` reference.
    #[inline]
    pub(crate) fn alloc<T>(&self, value: T) -> &'arena T {
        self.arena.alloc(value)
    }
}
