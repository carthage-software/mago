use mago_interner::ThreadedInterner;
use serde::Deserialize;
use serde::Serialize;

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
    fn get_id(&self, _interner: Option<&ThreadedInterner>) -> String {
        match self.closed {
            Some(true) => "closed-resource".to_string(),
            Some(false) => "open-resource".to_string(),
            None => "resource".to_string(),
        }
    }
}

impl Default for TResource {
    fn default() -> Self {
        Self::new(None)
    }
}
