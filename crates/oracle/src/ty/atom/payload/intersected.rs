use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::copy::copy_slice_into;

use crate::ty::atom::Atom;

/// `head & conj1 & conj2 & …`: the universal intersection wrapper.
/// `conjuncts` is sorted and non-empty by construction.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IntersectedAtom<'arena> {
    pub head: &'arena Atom<'arena>,
    pub conjuncts: &'arena [Atom<'arena>],
}

impl Display for IntersectedAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if self.head.has_intersection_types() {
            write!(f, "({})", self.head)?;
        } else {
            Display::fmt(self.head, f)?;
        }

        for conjunct in self.conjuncts {
            if conjunct.has_intersection_types() {
                write!(f, "&({conjunct})")?;
            } else {
                write!(f, "&{conjunct}")?;
            }
        }

        Ok(())
    }
}

impl CopyInto for IntersectedAtom<'_> {
    type Output<'arena> = IntersectedAtom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        IntersectedAtom { head: copy_ref_into(self.head, arena), conjuncts: copy_slice_into(self.conjuncts, arena) }
    }
}
