use mago_word::Word;
use mago_word::word;

use crate::ttype::TType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct TResource {
    pub closed: Option<bool>,
}

impl TResource {
    #[inline]
    #[must_use]
    pub const fn new(closed: Option<bool>) -> Self {
        Self { closed }
    }

    #[inline]
    #[must_use]
    pub const fn closed() -> Self {
        Self::new(Some(true))
    }

    #[inline]
    #[must_use]
    pub const fn open() -> Self {
        Self::new(Some(false))
    }

    #[inline]
    #[must_use]
    pub const fn is_closed(&self) -> bool {
        matches!(self.closed, Some(true))
    }

    #[inline]
    #[must_use]
    pub const fn is_open(&self) -> bool {
        matches!(self.closed, Some(false))
    }
}

impl TType for TResource {
    fn get_id(&self) -> Word {
        match self.closed {
            Some(true) => word("closed-resource"),
            Some(false) => word("open-resource"),
            None => word("resource"),
        }
    }
}

impl Default for TResource {
    fn default() -> Self {
        Self::new(None)
    }
}
