use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;

/// `has-method<'foo'>`: any object exposing a method with the given name.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HasMethodAtom<'arena> {
    pub method_name: &'arena [u8],
}

impl Display for HasMethodAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "has-method<'{}'>", String::from_utf8_lossy(self.method_name))
    }
}

impl CopyInto for HasMethodAtom<'_> {
    type Output<'arena> = HasMethodAtom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        HasMethodAtom { method_name: arena.alloc_slice_copy(self.method_name) }
    }
}
