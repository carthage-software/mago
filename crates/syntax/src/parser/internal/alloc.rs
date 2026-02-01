use bumpalo::collections::Vec;
use bumpalo::vec;

use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    /// Creates a new empty vector in the parser's arena.
    #[inline]
    pub fn new_vec<T>(&self) -> Vec<'arena, T> {
        Vec::new_in(self.arena)
    }

    /// Creates a new vector with a single value in the parser's arena.
    #[inline]
    pub fn new_vec_of<T>(&self, value: T) -> Vec<'arena, T> {
        vec![in self.arena; value]
    }

    /// Allocates a string in the parser's arena.
    #[inline]
    pub fn str(&self, string: &str) -> &'arena str {
        self.arena.alloc_str(string)
    }
}
