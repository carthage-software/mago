use crate::document::Document;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Indentation<'a> {
    Root,
    Indent,
    Alignment(&'a str),
    Combined(Vec<Indentation<'a>>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Break,
    Flat,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Command<'a> {
    pub indentation: Indentation<'a>,
    pub mode: Mode,
    pub document: Document<'a>,
}

impl Indentation<'_> {
    pub fn root() -> Self {
        Self::Root
    }

    #[must_use]
    #[inline(always)]
    pub const fn is_root(&self) -> bool {
        matches!(self, Self::Root)
    }

    #[must_use]
    #[inline(always)]
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

impl<'a> Command<'a> {
    pub fn new(indent: Indentation<'a>, mode: Mode, document: Document<'a>) -> Self {
        Self { indentation: indent, mode, document }
    }

    pub fn with_mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }
}
