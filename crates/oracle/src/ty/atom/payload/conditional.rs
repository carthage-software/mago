use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;

use crate::ty::Type;

/// `(T is U ? X : Y)`: a conditional type, resolved during expansion once the
/// subject is known.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ConditionalAtom<'arena> {
    pub subject: Type<'arena>,
    pub target: Type<'arena>,
    pub then: Type<'arena>,
    pub otherwise: Type<'arena>,
    pub negated: bool,
}

impl Display for ConditionalAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let op = if self.negated { " is not " } else { " is " };

        write!(f, "({}{}{} ? {} : {})", self.subject, op, self.target, self.then, self.otherwise)
    }
}

impl CopyInto for ConditionalAtom<'_> {
    type Output<'arena> = ConditionalAtom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ConditionalAtom {
            subject: self.subject.copy_into(arena),
            target: self.target.copy_into(arena),
            then: self.then.copy_into(arena),
            otherwise: self.otherwise.copy_into(arena),
            negated: self.negated,
        }
    }
}
