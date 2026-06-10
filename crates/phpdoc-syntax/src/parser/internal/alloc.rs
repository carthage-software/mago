use mago_allocator::Arena;
use mago_allocator::vec::Vec;

use crate::error::ParseError;
use crate::parser::PHPDocParser;

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    #[inline]
    pub(crate) fn new_vec<T>(&self) -> Vec<'arena, T, A> {
        Vec::new_in(self.arena)
    }

    #[inline]
    pub(crate) fn alloc<T>(&self, value: T) -> &'arena T {
        self.arena.alloc(value)
    }

    #[inline]
    pub(crate) fn record_error(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    #[inline]
    pub(crate) fn take_errors(&mut self) -> Vec<'arena, ParseError, A> {
        std::mem::replace(&mut self.errors, Vec::new_in(self.arena))
    }
}
