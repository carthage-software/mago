use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;

use crate::name::Name;

/// `enum(Foo)`, `enum(Foo::Bar)`: an enum type, optionally narrowed to a
/// single case.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EnumAtom<'arena> {
    pub name: Name<'arena>,
    pub case: Option<Name<'arena>>,
}

impl Display for EnumAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.case {
            Some(case) => write!(f, "enum({}::{})", self.name.as_str_lossy(), case.as_str_lossy()),
            None => write!(f, "enum({})", self.name.as_str_lossy()),
        }
    }
}

impl CopyInto for EnumAtom<'_> {
    type Output<'arena> = EnumAtom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        EnumAtom { name: self.name.copy_into(arena), case: self.case.map(|case| case.copy_into(arena)) }
    }
}
