use bumpalo::Bump;
use bumpalo::collections::Vec;

use crate::document::Document;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Indentation<'arena> {
    Root,
    Indent,
    Alignment(&'arena str),
    Combined(Vec<'arena, Indentation<'arena>>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Break,
    Flat,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Command<'arena> {
    pub indentation: Indentation<'arena>,
    pub mode: Mode,
    pub document: Document<'arena>,
}

impl<'arena> Indentation<'arena> {
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
    pub fn get_value_in(&self, arena: &'arena Bump, use_tabs: bool, tab_width: usize) -> &'arena [u8] {
        match self {
            Indentation::Root => &[],
            Indentation::Indent => {
                if use_tabs {
                    b"\t"
                } else {
                    let mut spaces = Vec::with_capacity_in(tab_width, arena);
                    spaces.resize(tab_width, b' ');
                    spaces.into_bump_slice()
                }
            }
            Indentation::Alignment(value) => value.as_bytes(),
            Indentation::Combined(nested) => {
                let mut combined = Vec::new_in(arena);
                for i in nested {
                    combined.extend_from_slice(i.get_value_in(arena, use_tabs, tab_width));
                }
                combined.into_bump_slice()
            }
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

impl<'arena> Command<'arena> {
    pub fn new(indent: Indentation<'arena>, mode: Mode, document: Document<'arena>) -> Self {
        Self { indentation: indent, mode, document }
    }

    pub fn with_mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }
}
