use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;

use crate::path::Path;

/// `enum(Foo)`, `enum(Foo::Bar)`: an enum type, optionally narrowed to a
/// single case.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EnumAtom<'arena> {
    pub name: Path<'arena>,
    pub case: Option<&'arena [u8]>,
}

impl Display for EnumAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.case {
            Some(case) => write!(f, "enum({}::{})", self.name, String::from_utf8_lossy(case)),
            None => write!(f, "enum({})", self.name),
        }
    }
}

impl CopyInto for EnumAtom<'_> {
    type Output<'arena> = EnumAtom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        let case = self.case.map(|case| -> &'arena [u8] { arena.alloc_slice_copy(case) });

        EnumAtom { name: self.name.copy_into(arena), case }
    }
}
