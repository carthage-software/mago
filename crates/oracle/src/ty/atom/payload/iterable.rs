use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;

use crate::ty::Type;

/// `iterable<K, V>`.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IterableAtom<'arena> {
    pub key_type: Type<'arena>,
    pub value_type: Type<'arena>,
}

impl Display for IterableAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "iterable<{}, {}>", self.key_type, self.value_type)
    }
}

impl CopyInto for IterableAtom<'_> {
    type Output<'arena> = IterableAtom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        IterableAtom { key_type: self.key_type.copy_into(arena), value_type: self.value_type.copy_into(arena) }
    }
}
