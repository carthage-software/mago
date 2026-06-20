use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;

use crate::path::Path;
use crate::symbol::class_like::ClassLikeKind;
use crate::ty::Type;

/// `class-string`, `interface-string<T>`, `enum-string`, `trait-string`,
/// `class-string('Foo')`.
///
/// `kind` selects which class-like family the string names; `specifier`
/// carries the rest.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ClassLikeStringAtom<'arena> {
    pub kind: ClassLikeKind,
    pub specifier: ClassLikeStringSpecifier<'arena>,
}

/// What is known about the value of this class-like-string beyond its kind.
///
/// `Generic` carries just a constraint type; the constraint itself contains a
/// [`GenericParameterAtom`](crate::ty::atom::payload::generic_parameter::GenericParameterAtom)
/// names the template parameter and its scope.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ClassLikeStringSpecifier<'arena> {
    Any,
    Literal { value: Path<'arena> },
    OfType { constraint: Type<'arena> },
    Generic { constraint: Type<'arena> },
}

impl ClassLikeKind {
    #[inline]
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            ClassLikeKind::Class => "class-string",
            ClassLikeKind::Interface => "interface-string",
            ClassLikeKind::Enum => "enum-string",
            ClassLikeKind::Trait => "trait-string",
        }
    }
}

impl Display for ClassLikeStringAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.specifier {
            ClassLikeStringSpecifier::Any => f.write_str(self.kind.as_str()),
            ClassLikeStringSpecifier::Literal { value } => write!(f, "class-string('{}')", value),
            ClassLikeStringSpecifier::OfType { constraint } | ClassLikeStringSpecifier::Generic { constraint } => {
                write!(f, "{}<{}>", self.kind.as_str(), constraint)
            }
        }
    }
}

impl CopyInto for ClassLikeStringAtom<'_> {
    type Output<'arena> = ClassLikeStringAtom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ClassLikeStringAtom { kind: self.kind, specifier: self.specifier.copy_into(arena) }
    }
}

impl CopyInto for ClassLikeStringSpecifier<'_> {
    type Output<'arena> = ClassLikeStringSpecifier<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match *self {
            ClassLikeStringSpecifier::Any => ClassLikeStringSpecifier::Any,
            ClassLikeStringSpecifier::Literal { value } => {
                ClassLikeStringSpecifier::Literal { value: value.copy_into(arena) }
            }
            ClassLikeStringSpecifier::OfType { constraint } => {
                ClassLikeStringSpecifier::OfType { constraint: constraint.copy_into(arena) }
            }
            ClassLikeStringSpecifier::Generic { constraint } => {
                ClassLikeStringSpecifier::Generic { constraint: constraint.copy_into(arena) }
            }
        }
    }
}
