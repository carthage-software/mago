use std::hash::BuildHasher;
use std::hash::Hash;

#[cfg(feature = "serde")]
use serde::Serialize;

use foldhash::fast::FixedState;

use crate::ty::Atom;
use crate::ty::Type;
use crate::ty::atom::payload::array::ArrayKey;

pub mod set;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Assertion<'arena> {
    Any,
    IsType(Atom<'arena>),
    IsNotType(Atom<'arena>),
    Falsy,
    Truthy,
    IsIdentical(Atom<'arena>),
    IsNotIdentical(Atom<'arena>),
    IsEqual(Atom<'arena>),
    IsNotEqual(Atom<'arena>),
    IsEqualIsset,
    IsIsset,
    IsNotIsset,
    HasStringArrayAccess,
    HasIntOrStringArrayAccess,
    ArrayKeyExists,
    ArrayKeyDoesNotExist,
    InArray(Type<'arena>),
    NotInArray(Type<'arena>),
    HasArrayKey(ArrayKey<'arena>),
    DoesNotHaveArrayKey(ArrayKey<'arena>),
    HasNonnullEntryForKey(ArrayKey<'arena>),
    DoesNotHaveNonnullEntryForKey(ArrayKey<'arena>),
    Empty,
    NonEmpty,
    NonEmptyCountable(bool),
    EmptyCountable,
    HasExactCount(usize),
    HasAtLeastCount(usize),
    DoesNotHaveExactCount(usize),
    DoesNotHasAtLeastCount(usize),
    IsLessThan(i64),
    IsLessThanOrEqual(i64),
    IsGreaterThan(i64),
    IsGreaterThanOrEqual(i64),
    Countable,
    NotCountable(bool),
}

impl<'arena> Assertion<'arena> {
    #[must_use]
    pub fn to_hash(&self) -> u64 {
        FixedState::default().hash_one(self)
    }

    #[must_use]
    pub fn is_negation(&self) -> bool {
        matches!(
            self,
            Assertion::Falsy
                | Assertion::IsNotType(_)
                | Assertion::IsNotEqual(_)
                | Assertion::IsNotIdentical(_)
                | Assertion::IsNotIsset
                | Assertion::NotInArray(..)
                | Assertion::ArrayKeyDoesNotExist
                | Assertion::DoesNotHaveArrayKey(_)
                | Assertion::DoesNotHaveExactCount(_)
                | Assertion::DoesNotHaveNonnullEntryForKey(_)
                | Assertion::DoesNotHasAtLeastCount(_)
                | Assertion::EmptyCountable
                | Assertion::Empty
                | Assertion::NotCountable(_)
        )
    }

    #[must_use]
    pub fn has_isset(&self) -> bool {
        matches!(
            self,
            Assertion::IsIsset | Assertion::ArrayKeyExists | Assertion::HasStringArrayAccess | Assertion::IsEqualIsset
        )
    }

    #[must_use]
    pub fn has_non_isset_equality(&self) -> bool {
        matches!(
            self,
            Assertion::InArray(_)
                | Assertion::HasIntOrStringArrayAccess
                | Assertion::HasStringArrayAccess
                | Assertion::IsIdentical(_)
                | Assertion::IsEqual(_)
        )
    }

    #[must_use]
    pub fn has_equality(&self) -> bool {
        matches!(
            self,
            Assertion::InArray(_)
                | Assertion::HasIntOrStringArrayAccess
                | Assertion::HasStringArrayAccess
                | Assertion::IsEqualIsset
                | Assertion::IsIdentical(_)
                | Assertion::IsNotIdentical(_)
                | Assertion::IsEqual(_)
                | Assertion::IsNotEqual(_)
                | Assertion::HasExactCount(_)
        )
    }

    #[must_use]
    pub fn with_type(&self, atom: Atom<'arena>) -> Self {
        match self {
            Assertion::IsType(_) => Assertion::IsType(atom),
            Assertion::IsNotType(_) => Assertion::IsNotType(atom),
            Assertion::IsIdentical(_) => Assertion::IsIdentical(atom),
            Assertion::IsNotIdentical(_) => Assertion::IsNotIdentical(atom),
            Assertion::IsEqual(_) => Assertion::IsEqual(atom),
            Assertion::IsNotEqual(_) => Assertion::IsNotEqual(atom),
            _ => *self,
        }
    }

    #[must_use]
    pub fn get_type(&self) -> Option<Atom<'arena>> {
        match self {
            Assertion::IsIdentical(atomic)
            | Assertion::IsNotIdentical(atomic)
            | Assertion::IsType(atomic)
            | Assertion::IsNotType(atomic)
            | Assertion::IsEqual(atomic)
            | Assertion::IsNotEqual(atomic) => Some(*atomic),
            _ => None,
        }
    }

    #[must_use]
    pub fn is_negation_of(&self, other: &Assertion<'arena>) -> bool {
        match self {
            Assertion::Any => false,
            Assertion::Falsy => matches!(other, Assertion::Truthy),
            Assertion::Truthy => matches!(other, Assertion::Falsy),
            Assertion::IsType(atomic) => match other {
                Assertion::IsNotType(other_atomic) => other_atomic == atomic,
                _ => false,
            },
            Assertion::IsNotType(atomic) => match other {
                Assertion::IsType(other_atomic) => other_atomic == atomic,
                _ => false,
            },
            Assertion::IsIdentical(atomic) => match other {
                Assertion::IsNotIdentical(other_atomic) => other_atomic == atomic,
                _ => false,
            },
            Assertion::IsNotIdentical(atomic) => match other {
                Assertion::IsIdentical(other_atomic) => other_atomic == atomic,
                _ => false,
            },
            Assertion::IsEqual(atomic) => match other {
                Assertion::IsNotEqual(other_atomic) => other_atomic == atomic,
                _ => false,
            },
            Assertion::IsNotEqual(atomic) => match other {
                Assertion::IsEqual(other_atomic) => other_atomic == atomic,
                _ => false,
            },
            Assertion::IsEqualIsset => false,
            Assertion::IsIsset => matches!(other, Assertion::IsNotIsset),
            Assertion::IsNotIsset => matches!(other, Assertion::IsIsset),
            Assertion::HasStringArrayAccess => false,
            Assertion::HasIntOrStringArrayAccess => false,
            Assertion::ArrayKeyExists => matches!(other, Assertion::ArrayKeyDoesNotExist),
            Assertion::ArrayKeyDoesNotExist => matches!(other, Assertion::ArrayKeyExists),
            Assertion::HasArrayKey(str) => match other {
                Assertion::DoesNotHaveArrayKey(other_str) => other_str == str,
                _ => false,
            },
            Assertion::DoesNotHaveArrayKey(str) => match other {
                Assertion::HasArrayKey(other_str) => other_str == str,
                _ => false,
            },
            Assertion::HasNonnullEntryForKey(str) => match other {
                Assertion::DoesNotHaveNonnullEntryForKey(other_str) => other_str == str,
                _ => false,
            },
            Assertion::DoesNotHaveNonnullEntryForKey(str) => match other {
                Assertion::HasNonnullEntryForKey(other_str) => other_str == str,
                _ => false,
            },
            Assertion::InArray(union) => match other {
                Assertion::NotInArray(other_union) => other_union == union,
                _ => false,
            },
            Assertion::NotInArray(union) => match other {
                Assertion::InArray(other_union) => other_union == union,
                _ => false,
            },
            Assertion::Empty => matches!(other, Assertion::NonEmpty),
            Assertion::NonEmpty => matches!(other, Assertion::Empty),
            Assertion::NonEmptyCountable(negatable) => {
                if *negatable {
                    matches!(other, Assertion::EmptyCountable)
                } else {
                    false
                }
            }
            Assertion::EmptyCountable => matches!(other, Assertion::NonEmptyCountable(true)),
            Assertion::HasExactCount(number) => match other {
                Assertion::DoesNotHaveExactCount(other_number) => other_number == number,
                _ => false,
            },
            Assertion::DoesNotHaveExactCount(number) => match other {
                Assertion::HasExactCount(other_number) => other_number == number,
                _ => false,
            },
            Assertion::HasAtLeastCount(number) => match other {
                Assertion::DoesNotHasAtLeastCount(other_number) => other_number == number,
                _ => false,
            },
            Assertion::DoesNotHasAtLeastCount(number) => match other {
                Assertion::HasAtLeastCount(other_number) => other_number == number,
                _ => false,
            },
            Assertion::IsLessThan(number) => match other {
                Assertion::IsGreaterThanOrEqual(other_number) => other_number == number,
                _ => false,
            },
            Assertion::IsLessThanOrEqual(number) => match other {
                Assertion::IsGreaterThan(other_number) => other_number == number,
                _ => false,
            },
            Assertion::IsGreaterThan(number) => match other {
                Assertion::IsLessThanOrEqual(other_number) => other_number == number,
                _ => false,
            },
            Assertion::IsGreaterThanOrEqual(number) => match other {
                Assertion::IsLessThan(other_number) => other_number == number,
                _ => false,
            },
            Assertion::Countable => matches!(other, Assertion::NotCountable(negatable) if *negatable),
            Assertion::NotCountable(_) => matches!(other, Assertion::Countable),
        }
    }

    #[must_use]
    pub fn get_negation(&self) -> Self {
        match self {
            Assertion::Any => Assertion::Any,
            Assertion::Falsy => Assertion::Truthy,
            Assertion::IsType(atomic) => Assertion::IsNotType(*atomic),
            Assertion::IsNotType(atomic) => Assertion::IsType(*atomic),
            Assertion::Truthy => Assertion::Falsy,
            Assertion::IsIdentical(atomic) => Assertion::IsNotIdentical(*atomic),
            Assertion::IsNotIdentical(atomic) => Assertion::IsIdentical(*atomic),
            Assertion::IsEqual(atomic) => Assertion::IsNotEqual(*atomic),
            Assertion::IsNotEqual(atomic) => Assertion::IsEqual(*atomic),
            Assertion::IsIsset => Assertion::IsNotIsset,
            Assertion::IsNotIsset => Assertion::IsIsset,
            Assertion::Empty => Assertion::NonEmpty,
            Assertion::NonEmpty => Assertion::Empty,
            Assertion::NonEmptyCountable(negatable) => {
                if *negatable {
                    Assertion::EmptyCountable
                } else {
                    Assertion::Any
                }
            }
            Assertion::EmptyCountable => Assertion::NonEmptyCountable(true),
            Assertion::ArrayKeyExists => Assertion::ArrayKeyDoesNotExist,
            Assertion::ArrayKeyDoesNotExist => Assertion::ArrayKeyExists,
            Assertion::InArray(union) => Assertion::NotInArray(*union),
            Assertion::NotInArray(union) => Assertion::InArray(*union),
            Assertion::HasExactCount(size) => Assertion::DoesNotHaveExactCount(*size),
            Assertion::DoesNotHaveExactCount(size) => Assertion::HasExactCount(*size),
            Assertion::HasAtLeastCount(size) => Assertion::DoesNotHasAtLeastCount(*size),
            Assertion::DoesNotHasAtLeastCount(size) => Assertion::HasAtLeastCount(*size),
            Assertion::HasArrayKey(str) => Assertion::DoesNotHaveArrayKey(*str),
            Assertion::DoesNotHaveArrayKey(str) => Assertion::HasArrayKey(*str),
            Assertion::HasNonnullEntryForKey(str) => Assertion::DoesNotHaveNonnullEntryForKey(*str),
            Assertion::DoesNotHaveNonnullEntryForKey(str) => Assertion::HasNonnullEntryForKey(*str),
            Assertion::HasStringArrayAccess => Assertion::Any,
            Assertion::HasIntOrStringArrayAccess => Assertion::Any,
            Assertion::IsEqualIsset => Assertion::Any,
            Assertion::IsLessThan(number) => Assertion::IsGreaterThanOrEqual(*number),
            Assertion::IsLessThanOrEqual(number) => Assertion::IsGreaterThan(*number),
            Assertion::IsGreaterThan(number) => Assertion::IsLessThanOrEqual(*number),
            Assertion::IsGreaterThanOrEqual(number) => Assertion::IsLessThan(*number),
            Assertion::Countable => Assertion::NotCountable(true),
            Assertion::NotCountable(_) => Assertion::Countable,
        }
    }
}
