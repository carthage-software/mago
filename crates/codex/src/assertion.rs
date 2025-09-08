use std::borrow::Cow;
use std::hash::Hash;
use std::hash::Hasher;

use ahash::AHasher;
use mago_atom::Atom;
use mago_atom::atom;
use mago_atom::concat_atom;
use mago_atom::i64_atom;
use mago_atom::usize_atom;
use serde::Deserialize;
use serde::Serialize;

use crate::metadata::CodebaseMetadata;
use crate::ttype::TType;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::key::ArrayKey;
use crate::ttype::template::TemplateResult;
use crate::ttype::template::inferred_type_replacer;
use crate::ttype::union::TUnion;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub enum Assertion {
    Any,
    IsType(TAtomic),
    IsNotType(TAtomic),
    Falsy,
    Truthy,
    IsIdentical(TAtomic),
    IsNotIdentical(TAtomic),
    IsEqual(TAtomic),
    IsNotEqual(TAtomic),
    IsEqualIsset,
    IsIsset,
    IsNotIsset,
    HasStringArrayAccess,
    HasIntOrStringArrayAccess,
    ArrayKeyExists,
    ArrayKeyDoesNotExist,
    InArray(TUnion),
    NotInArray(TUnion),
    HasArrayKey(ArrayKey),
    DoesNotHaveArrayKey(ArrayKey),
    HasNonnullEntryForKey(ArrayKey),
    DoesNotHaveNonnullEntryForKey(ArrayKey),
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

impl Assertion {
    pub fn to_atom(&self) -> Atom {
        match self {
            Assertion::Any => atom("any"),
            Assertion::Falsy => atom("falsy"),
            Assertion::Truthy => atom("truthy"),
            Assertion::IsEqualIsset => atom("=isset"),
            Assertion::IsIsset => atom("isset"),
            Assertion::IsNotIsset => atom("!isset"),
            Assertion::HasStringArrayAccess => atom("=string-array-access"),
            Assertion::HasIntOrStringArrayAccess => atom("=int-or-string-array-access"),
            Assertion::ArrayKeyExists => atom("array-key-exists"),
            Assertion::ArrayKeyDoesNotExist => atom("!array-key-exists"),
            Assertion::EmptyCountable => atom("empty-countable"),
            Assertion::Empty => atom("empty"),
            Assertion::NonEmpty => atom("non-empty"),
            Assertion::Countable => atom("countable"),
            Assertion::NotCountable(_) => atom("!countable"),
            Assertion::IsType(atomic) => atomic.get_id(),
            Assertion::IsNotType(atomic) => concat_atom!("!", atomic.get_id()),
            Assertion::IsIdentical(atomic) => concat_atom!("=", atomic.get_id()),
            Assertion::IsNotIdentical(atomic) => concat_atom!("!=", atomic.get_id()),
            Assertion::IsEqual(atomic) => concat_atom!("~", atomic.get_id()),
            Assertion::IsNotEqual(atomic) => concat_atom!("!~", atomic.get_id()),
            Assertion::InArray(union) => concat_atom!("=in-array-", union.get_id()),
            Assertion::NotInArray(union) => concat_atom!("!=in-array-", union.get_id()),
            Assertion::HasArrayKey(key) => concat_atom!("=has-array-key-", key.to_atom()),
            Assertion::DoesNotHaveArrayKey(key) => concat_atom!("!=has-array-key-", key.to_atom()),
            Assertion::HasNonnullEntryForKey(key) => concat_atom!("=has-nonnull-entry-for-", key.to_atom()),
            Assertion::DoesNotHaveNonnullEntryForKey(key) => {
                concat_atom!("!=has-nonnull-entry-for-", key.to_atom())
            }
            Assertion::HasExactCount(number) => concat_atom!("has-exactly-", usize_atom(*number)),
            Assertion::HasAtLeastCount(number) => concat_atom!("has-at-least-", usize_atom(*number)),
            Assertion::DoesNotHaveExactCount(number) => concat_atom!("!has-exactly-", usize_atom(*number)),
            Assertion::DoesNotHasAtLeastCount(number) => concat_atom!("has-at-most-", usize_atom(*number)),
            Assertion::IsLessThan(number) => concat_atom!("is-less-than-", i64_atom(*number)),
            Assertion::IsLessThanOrEqual(number) => concat_atom!("is-less-than-or-equal-", i64_atom(*number)),
            Assertion::IsGreaterThan(number) => concat_atom!("is-greater-than-", i64_atom(*number)),
            Assertion::IsGreaterThanOrEqual(number) => concat_atom!("is-greater-than-or-equal-", i64_atom(*number)),
            Assertion::NonEmptyCountable(negatable) => {
                if *negatable {
                    atom("non-empty-countable")
                } else {
                    atom("=non-empty-countable")
                }
            }
        }
    }

    pub fn to_hash(&self) -> u64 {
        let mut state = AHasher::default();
        self.to_atom().hash(&mut state);
        state.finish()
    }

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

    pub fn has_isset(&self) -> bool {
        matches!(
            self,
            Assertion::IsIsset | Assertion::ArrayKeyExists | Assertion::HasStringArrayAccess | Assertion::IsEqualIsset
        )
    }

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

    pub fn has_literal_value(&self) -> bool {
        match self {
            Assertion::IsIdentical(atomic)
            | Assertion::IsNotIdentical(atomic)
            | Assertion::IsType(atomic)
            | Assertion::IsNotType(atomic)
            | Assertion::IsEqual(atomic)
            | Assertion::IsNotEqual(atomic) => {
                atomic.is_literal_int()
                    || atomic.is_literal_float()
                    || atomic.is_known_literal_string()
                    || atomic.is_literal_class_string()
            }

            _ => false,
        }
    }

    pub fn has_integer(&self) -> bool {
        match self {
            Assertion::IsIdentical(atomic)
            | Assertion::IsNotIdentical(atomic)
            | Assertion::IsType(atomic)
            | Assertion::IsNotType(atomic)
            | Assertion::IsEqual(atomic)
            | Assertion::IsNotEqual(atomic) => atomic.is_int(),
            _ => false,
        }
    }

    pub fn has_literal_string(&self) -> bool {
        match self {
            Assertion::IsIdentical(atomic)
            | Assertion::IsNotIdentical(atomic)
            | Assertion::IsType(atomic)
            | Assertion::IsNotType(atomic)
            | Assertion::IsEqual(atomic)
            | Assertion::IsNotEqual(atomic) => atomic.is_known_literal_string(),

            _ => false,
        }
    }

    pub fn has_literal_int(&self) -> bool {
        match self {
            Assertion::IsIdentical(atomic)
            | Assertion::IsNotIdentical(atomic)
            | Assertion::IsType(atomic)
            | Assertion::IsNotType(atomic)
            | Assertion::IsEqual(atomic)
            | Assertion::IsNotEqual(atomic) => atomic.is_literal_int(),

            _ => false,
        }
    }

    pub fn has_literal_float(&self) -> bool {
        match self {
            Assertion::IsIdentical(atomic)
            | Assertion::IsNotIdentical(atomic)
            | Assertion::IsType(atomic)
            | Assertion::IsNotType(atomic)
            | Assertion::IsEqual(atomic)
            | Assertion::IsNotEqual(atomic) => atomic.is_literal_float(),

            _ => false,
        }
    }

    pub fn with_type(&self, atomic: TAtomic) -> Self {
        match self {
            Assertion::IsType(_) => Assertion::IsType(atomic),
            Assertion::IsNotType(_) => Assertion::IsNotType(atomic),
            Assertion::IsIdentical(_) => Assertion::IsIdentical(atomic),
            Assertion::IsNotIdentical(_) => Assertion::IsNotIdentical(atomic),
            Assertion::IsEqual(_) => Assertion::IsEqual(atomic),
            Assertion::IsNotEqual(_) => Assertion::IsNotEqual(atomic),
            _ => self.clone(),
        }
    }

    pub fn get_type(&self) -> Option<&TAtomic> {
        match self {
            Assertion::IsIdentical(atomic)
            | Assertion::IsNotIdentical(atomic)
            | Assertion::IsType(atomic)
            | Assertion::IsNotType(atomic)
            | Assertion::IsEqual(atomic)
            | Assertion::IsNotEqual(atomic) => Some(atomic),
            _ => None,
        }
    }

    pub fn get_type_mut(&mut self) -> Option<&mut TAtomic> {
        match self {
            Assertion::IsIdentical(atomic)
            | Assertion::IsNotIdentical(atomic)
            | Assertion::IsType(atomic)
            | Assertion::IsNotType(atomic)
            | Assertion::IsEqual(atomic)
            | Assertion::IsNotEqual(atomic) => Some(atomic),
            _ => None,
        }
    }

    pub fn resolve_templates(&self, codebase: &CodebaseMetadata, template_result: &TemplateResult) -> Vec<Self> {
        match self {
            Assertion::IsType(atomic) => {
                let union = TUnion::from_single(Cow::Owned(atomic.clone()));
                let resolved_union = inferred_type_replacer::replace(&union, template_result, codebase);

                let mut result = vec![];
                for resolved_atomic in resolved_union.types.into_owned() {
                    result.push(Assertion::IsType(resolved_atomic));
                }

                if result.is_empty() {
                    result.push(Assertion::IsType(TAtomic::Never));
                }

                result
            }
            Assertion::IsNotType(atomic) => {
                let union = TUnion::from_single(Cow::Owned(atomic.clone()));
                let resolved_union = inferred_type_replacer::replace(&union, template_result, codebase);

                let mut result = vec![];
                for resolved_atomic in resolved_union.types.into_owned() {
                    result.push(Assertion::IsNotType(resolved_atomic));
                }

                if result.is_empty() {
                    result.push(Assertion::IsNotType(TAtomic::Never));
                }

                result
            }
            Assertion::InArray(union) => {
                let resolved_union = inferred_type_replacer::replace(union, template_result, codebase);

                vec![Assertion::InArray(resolved_union)]
            }
            Assertion::NotInArray(union) => {
                let resolved_union = inferred_type_replacer::replace(union, template_result, codebase);

                vec![Assertion::NotInArray(resolved_union)]
            }
            _ => {
                vec![self.clone()]
            }
        }
    }

    pub fn is_negation_of(&self, other: &Assertion) -> bool {
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

    pub fn get_negation(&self) -> Self {
        match self {
            Assertion::Any => Assertion::Any,
            Assertion::Falsy => Assertion::Truthy,
            Assertion::IsType(atomic) => Assertion::IsNotType(atomic.clone()),
            Assertion::IsNotType(atomic) => Assertion::IsType(atomic.clone()),
            Assertion::Truthy => Assertion::Falsy,
            Assertion::IsIdentical(atomic) => Assertion::IsNotIdentical(atomic.clone()),
            Assertion::IsNotIdentical(atomic) => Assertion::IsIdentical(atomic.clone()),
            Assertion::IsEqual(atomic) => Assertion::IsNotEqual(atomic.clone()),
            Assertion::IsNotEqual(atomic) => Assertion::IsEqual(atomic.clone()),
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
            Assertion::InArray(union) => Assertion::NotInArray(union.clone()),
            Assertion::NotInArray(union) => Assertion::InArray(union.clone()),
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
