use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;

use crate::path::Path;

/// `Foo::Bar` where `Bar` is a `@type` alias declared on `Foo`. Expansion
/// resolves it to the alias body.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AliasAtom<'arena> {
    pub class_name: Path<'arena>,
    pub alias_name: &'arena [u8],
}

impl Display for AliasAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}::{}", self.class_name, String::from_utf8_lossy(self.alias_name))
    }
}

impl CopyInto for AliasAtom<'_> {
    type Output<'arena> = AliasAtom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        AliasAtom { class_name: self.class_name.copy_into(arena), alias_name: arena.alloc_slice_copy(self.alias_name) }
    }
}
