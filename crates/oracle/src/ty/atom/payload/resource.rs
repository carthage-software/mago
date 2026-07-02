use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;

/// `resource`, `open-resource`, `closed-resource`.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ResourceAtom {
    Any,
    Open,
    Closed,
}

impl Display for ResourceAtom {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(match self {
            ResourceAtom::Any => "resource",
            ResourceAtom::Open => "open-resource",
            ResourceAtom::Closed => "closed-resource",
        })
    }
}

impl CopyInto for ResourceAtom {
    type Output<'arena> = ResourceAtom;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}
