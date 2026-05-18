use bumpalo::collections::Vec;
use bumpalo::vec;

use crate::parser::Parser;

impl<'arena> Parser<'_, 'arena> {
    /// Creates a new empty vector in the parser's arena.
    #[inline]
    #[must_use]
    pub fn new_vec<T>(&self) -> Vec<'arena, T> {
        Vec::new_in(self.arena)
    }

    /// Creates a new vector with a single value in the parser's arena.
    #[inline]
    #[must_use]
    pub fn new_vec_of<T>(&self, value: T) -> Vec<'arena, T> {
        vec![in self.arena; value]
    }

    /// Allocates a byte slice in the parser's arena.
    #[inline]
    #[must_use]
    pub fn bytes(&self, bytes: &[u8]) -> &'arena [u8] {
        self.arena.alloc_slice_copy(bytes)
    }
}
