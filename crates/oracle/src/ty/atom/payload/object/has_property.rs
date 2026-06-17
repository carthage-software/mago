use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;

/// `has-property<'foo'>`: any object exposing a property with the given name.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HasPropertyAtom<'arena> {
    pub property_name: &'arena [u8],
}

impl Display for HasPropertyAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "has-property<'{}'>", String::from_utf8_lossy(self.property_name))
    }
}

impl CopyInto for HasPropertyAtom<'_> {
    type Output<'arena> = HasPropertyAtom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        HasPropertyAtom { property_name: arena.alloc_slice_copy(self.property_name) }
    }
}
