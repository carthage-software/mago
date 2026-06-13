use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;

use crate::name::Name;

/// `has-property<'foo'>`: any object exposing a property with the given name.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HasPropertyAtom<'arena> {
    pub property_name: Name<'arena>,
}

impl Display for HasPropertyAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "has-property<'{}'>", self.property_name.as_str_lossy())
    }
}

impl CopyInto for HasPropertyAtom<'_> {
    type Output<'arena> = HasPropertyAtom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        HasPropertyAtom { property_name: self.property_name.copy_into(arena) }
    }
}
