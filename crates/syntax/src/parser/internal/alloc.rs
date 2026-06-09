use mago_allocator::prelude::*;

use crate::parser::Parser;

impl<'arena, A> Parser<'_, 'arena, A>
where
    A: Arena,
{
    /// Creates a new empty vector in the parser's arena.
    #[inline]
    #[must_use]
    pub fn new_vec<T>(&self) -> Vec<'arena, T, A> {
        Vec::new_in(self.arena)
    }

    /// Creates a new vector with a single value in the parser's arena.
    #[inline]
    #[must_use]
    pub fn new_vec_of<T>(&self, value: T) -> Vec<'arena, T, A> {
        let mut vector = Vec::new_in(self.arena);
        vector.push(value);
        vector
    }

    /// Allocates a byte slice in the parser's arena.
    #[inline]
    #[must_use]
    pub fn bytes(&self, bytes: &[u8]) -> &'arena [u8] {
        self.arena.alloc_slice_copy(bytes)
    }
}
