use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;

use crate::var::Var;

/// A type variable bound to a PHP variable, e.g. `$x` in `@assert` clauses.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VariableAtom<'arena> {
    pub name: Var<'arena>,
}

impl Display for VariableAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&self.name.as_str_lossy())
    }
}

impl CopyInto for VariableAtom<'_> {
    type Output<'arena> = VariableAtom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        VariableAtom { name: self.name.copy_into(arena) }
    }
}
