use mago_allocator::Arena;

use crate::document::Document;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Indentation<'arena> {
    depth: usize,
    alignment: Option<&'arena [u8]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Break,
    Flat,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Command<'arena, A>
where
    A: Arena,
{
    pub indentation: Indentation<'arena>,
    pub mode: Mode,
    pub document: CommandDocument<'arena, A>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CommandDocument<'arena, A: Arena> {
    Document(&'arena Document<'arena, A>),
    Array(&'arena [Document<'arena, A>]),
}

impl<A: Arena> Copy for CommandDocument<'_, A> {}

impl<A: Arena> Clone for CommandDocument<'_, A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'arena> Indentation<'arena> {
    pub const fn root() -> Self {
        Self { depth: 0, alignment: None }
    }

    pub const fn aligned(value: &'arena [u8]) -> Self {
        Self { depth: 0, alignment: Some(value) }
    }

    pub const fn indented(self) -> Self {
        Self { depth: self.depth + 1, alignment: self.alignment }
    }

    #[must_use]
    #[inline]
    pub const fn is_root(&self) -> bool {
        self.depth == 0 && self.alignment.is_none()
    }

    #[must_use]
    #[inline]
    pub const fn depth(&self) -> usize {
        self.depth
    }

    #[must_use]
    #[inline]
    pub const fn alignment(&self) -> Option<&'arena [u8]> {
        self.alignment
    }
}

impl Mode {
    pub fn is_break(self) -> bool {
        self == Self::Break
    }

    pub fn is_flat(self) -> bool {
        self == Self::Flat
    }
}

impl<'arena, A> Command<'arena, A>
where
    A: Arena,
{
    pub fn new(indent: Indentation<'arena>, mode: Mode, document: &'arena Document<'arena, A>) -> Self {
        Self { indentation: indent, mode, document: CommandDocument::Document(document) }
    }

    pub fn array(indent: Indentation<'arena>, mode: Mode, documents: &'arena [Document<'arena, A>]) -> Self {
        Self { indentation: indent, mode, document: CommandDocument::Array(documents) }
    }
}
