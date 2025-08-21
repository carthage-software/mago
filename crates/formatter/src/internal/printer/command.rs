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

impl Indentation<'_> {
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
    pub fn get_value(&self, use_tabs: bool, tab_width: usize) -> String {
        match self {
            Indentation::Root => String::new(),
            Indentation::Indent => {
                if use_tabs {
                    "\t".to_string()
                } else {
                    " ".repeat(tab_width)
                }
            }
            Indentation::Alignment(value) => value.to_string(),
            Indentation::Combined(nested) => nested.iter().map(|i| i.get_value(use_tabs, tab_width)).collect(),
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
