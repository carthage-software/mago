use mago_allocator::Arena;
use mago_allocator::vec::Vec;

use crate::document::Document;
use crate::internal::utils::string_width;

#[derive(Debug, PartialEq, Eq)]
pub enum Indentation<'arena, A>
where
    A: Arena,
{
    Root,
    Indent,
    Alignment(&'arena [u8]),
    Combined(Vec<'arena, Indentation<'arena, A>, A>),
}

impl<A> Clone for Indentation<'_, A>
where
    A: Arena,
{
    fn clone(&self) -> Self {
        match self {
            Self::Root => Self::Root,
            Self::Indent => Self::Indent,
            Self::Alignment(value) => Self::Alignment(value),
            Self::Combined(nested) => Self::Combined(nested.clone()),
        }
    }
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
    pub indentation: Indentation<'arena, A>,
    pub mode: Mode,
    pub document: Document<'arena, A>,
}

impl<'arena, A> Indentation<'arena, A>
where
    A: Arena,
{
    pub fn root() -> Self {
        Self::Root
    }

    #[must_use]
    #[inline]
    pub const fn is_root(&self) -> bool {
        matches!(self, Self::Root)
    }

    #[must_use]
    #[inline]
    pub fn get_value_in(&self, arena: &'arena A, use_tabs: bool, tab_width: usize) -> &'arena [u8] {
        match self {
            Indentation::Root => &[],
            Indentation::Indent => {
                if use_tabs {
                    b"\t"
                } else {
                    let mut spaces = Vec::with_capacity_in(tab_width, arena);
                    spaces.resize(tab_width, b' ');
                    spaces.leak()
                }
            }
            Indentation::Alignment(value) => value,
            Indentation::Combined(nested) => {
                let mut combined = Vec::new_in(arena);
                for i in nested {
                    combined.extend_from_slice(i.get_value_in(arena, use_tabs, tab_width));
                }
                combined.leak()
            }
        }
    }

    #[must_use]
    pub fn get_width_in(&self, tab_width: usize) -> usize {
        match self {
            Indentation::Root => 0,
            Indentation::Indent => tab_width,
            Indentation::Alignment(value) => string_width(value),
            Indentation::Combined(nested) => nested.iter().map(|i| i.get_width_in(tab_width)).sum(),
        }
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
    pub fn new(indent: Indentation<'arena, A>, mode: Mode, document: Document<'arena, A>) -> Self {
        Self { indentation: indent, mode, document }
    }

    pub fn with_mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }
}
