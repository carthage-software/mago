use bumpalo::collections::Vec as BVec;

use mago_span::Span;

use crate::cst::inherit_doc::InheritDoc;
use crate::error::ParseError;
use crate::parser::PHPDocParser;

impl<'arena> PHPDocParser<'arena> {
    #[inline]
    pub(crate) fn new_vec<T>(&self) -> BVec<'arena, T> {
        BVec::new_in(self.arena)
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
    pub(crate) fn take_errors(&mut self) -> BVec<'arena, ParseError> {
        std::mem::replace(&mut self.errors, BVec::new_in(self.arena))
    }

    #[inline]
    pub(crate) fn record_inherit_doc(&mut self, span: Span) {
        self.inherit_docs.push(InheritDoc { span });
    }

    #[inline]
    pub(crate) fn take_inherit_docs(&mut self) -> BVec<'arena, InheritDoc> {
        std::mem::replace(&mut self.inherit_docs, BVec::new_in(self.arena))
    }
}
