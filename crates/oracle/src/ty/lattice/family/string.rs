//! String family.
//!
//! Containers express constraints on three axes:
//!
//! - literal slot (`None` / `Unspecified` / `Value(v)`)
//! - casing (`Unspecified` / `Lowercase` / `Uppercase`)
//! - refinement flags (`NonEmpty`, `Truthy`, `Numeric`, `Callable`)
//!
//! The input must satisfy *every* constraint the container imposes. Each
//! constraint is satisfied either by an equivalent constraint on the input,
//! or by the input being a literal value that structurally implies it
//! (e.g. `"abc"` is non-empty by inspection).
//!
//! Class-like-string inputs are also accepted here: class names are
//! non-empty and not `"0"`, so they satisfy `non-empty` and `truthy`. They
//! do not satisfy casing or `callable` constraints by default.

use mago_flags::U8Flags;

use crate::ty::atom::Atom;
use crate::ty::atom::payload::scalar::string::StringAtom;
use crate::ty::atom::payload::scalar::string::StringCasing;
use crate::ty::atom::payload::scalar::string::StringLiteral;
use crate::ty::atom::payload::scalar::string::StringRefinementFlag;

#[inline]
#[must_use]
pub fn refines(input: Atom<'_>, container: Atom<'_>) -> bool {
    let Atom::String(container_payload) = container else {
        return false;
    };

    if let Atom::ClassLikeString(_) = input {
        return class_like_string_satisfies(*container_payload);
    }

    let Atom::String(input_payload) = input else {
        return false;
    };

    string_satisfies(*input_payload, *container_payload)
}

#[inline]
fn class_like_string_satisfies(container: StringAtom<'_>) -> bool {
    if !literal_constraint_admits_class_like(container.literal) {
        return false;
    }

    if container.casing != StringCasing::Unspecified {
        return false;
    }

    if container.flags.contains(StringRefinementFlag::Numeric) {
        return false;
    }

    !container.flags.contains(StringRefinementFlag::Callable)
}

#[inline]
const fn literal_constraint_admits_class_like(literal: StringLiteral<'_>) -> bool {
    matches!(literal, StringLiteral::None)
}

#[inline]
fn string_satisfies(input: StringAtom<'_>, container: StringAtom<'_>) -> bool {
    satisfies_literal(input.literal, container.literal)
        && satisfies_casing(input, container.casing)
        && satisfies_flags(input, container.flags)
}

#[inline]
fn satisfies_literal(input: StringLiteral<'_>, container: StringLiteral<'_>) -> bool {
    match (input, container) {
        (_, StringLiteral::None) => true,
        (StringLiteral::Value(_) | StringLiteral::Unspecified, StringLiteral::Unspecified) => true,
        (StringLiteral::Value(left), StringLiteral::Value(right)) => left == right,
        _ => false,
    }
}

#[inline]
fn satisfies_casing(input: StringAtom<'_>, container_casing: StringCasing) -> bool {
    match container_casing {
        StringCasing::Unspecified => true,
        StringCasing::Lowercase => match input.casing {
            StringCasing::Lowercase => true,
            _ => match input.literal {
                StringLiteral::Value(value) => value.as_bytes().iter().all(|byte| !byte.is_ascii_uppercase()),
                _ => false,
            },
        },
        StringCasing::Uppercase => match input.casing {
            StringCasing::Uppercase => true,
            _ => match input.literal {
                StringLiteral::Value(value) => value.as_bytes().iter().all(|byte| !byte.is_ascii_lowercase()),
                _ => false,
            },
        },
    }
}

#[inline]
fn satisfies_flags(input: StringAtom<'_>, container_flags: U8Flags<StringRefinementFlag>) -> bool {
    if container_flags.contains(StringRefinementFlag::NonEmpty) && !input_is_non_empty(input) {
        return false;
    }

    if container_flags.contains(StringRefinementFlag::Truthy) && !input_is_truthy(input) {
        return false;
    }

    if container_flags.contains(StringRefinementFlag::Numeric) && !input_is_numeric(input) {
        return false;
    }

    if container_flags.contains(StringRefinementFlag::Callable) && !input.flags.contains(StringRefinementFlag::Callable)
    {
        return false;
    }

    true
}

pub(super) fn input_is_non_empty(input: StringAtom<'_>) -> bool {
    if input.flags.contains(StringRefinementFlag::NonEmpty)
        || input.flags.contains(StringRefinementFlag::Truthy)
        || input.flags.contains(StringRefinementFlag::Numeric)
        || input.flags.contains(StringRefinementFlag::Callable)
    {
        return true;
    }

    match input.literal {
        StringLiteral::Value(value) => !value.is_empty(),
        _ => false,
    }
}

#[inline]
fn input_is_truthy(input: StringAtom<'_>) -> bool {
    if input.flags.contains(StringRefinementFlag::Truthy) {
        return true;
    }

    match input.literal {
        StringLiteral::Value(value) => {
            let bytes = value.as_bytes();
            !bytes.is_empty() && bytes != b"0"
        }
        _ => false,
    }
}

pub(super) fn input_is_numeric(input: StringAtom<'_>) -> bool {
    if input.flags.contains(StringRefinementFlag::Numeric) {
        return true;
    }

    match input.literal {
        StringLiteral::Value(value) => match core::str::from_utf8(value.as_bytes()) {
            Ok(text) => text.parse::<i64>().is_ok() || text.parse::<f64>().is_ok(),
            Err(_) => false,
        },
        _ => false,
    }
}
