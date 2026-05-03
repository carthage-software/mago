use bumpalo::collections::Vec as BVec;

use crate::parser::Parser;

impl<'arena> Parser<'_, 'arena> {
    /// A fresh arena-backed vector.
    #[inline]
    pub(crate) fn new_vec<T>(&self) -> BVec<'arena, T> {
        BVec::new_in(self.arena)
    }

    /// Arena-allocate a value and return an `&'arena T` reference.
    #[inline]
    pub(crate) fn alloc<T>(&self, value: T) -> &'arena T {
        self.arena.alloc(value)
    }
}
