use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::atom;

use crate::ttype::TType;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TResource {
    pub closed: Option<bool>,
}

impl TResource {
    #[inline]
    pub const fn new(closed: Option<bool>) -> Self {
        Self { closed }
    }

    #[inline]
    pub const fn closed() -> Self {
        Self::new(Some(true))
    }

    #[inline]
    pub const fn open() -> Self {
        Self::new(Some(false))
    }

    #[inline]
    pub const fn is_closed(&self) -> bool {
        matches!(self.closed, Some(true))
    }

    #[inline]
    pub const fn is_open(&self) -> bool {
        matches!(self.closed, Some(false))
    }
}

impl TType for TResource {
    fn needs_population(&self) -> bool {
        false
    }

    fn is_expandable(&self) -> bool {
        false
    }

    fn get_id(&self) -> Atom {
        match self.closed {
            Some(true) => atom("closed-resource"),
            Some(false) => atom("open-resource"),
            None => atom("resource"),
        }
    }
}

impl Default for TResource {
    fn default() -> Self {
        Self::new(None)
    }
}
