//! Per-atom truthiness / falsiness / literal classifiers.
//!
//! Each function returns a 2-state guarantee on a single atom:
//!
//! - [`is_truthy`] - every value the atom admits is truthy.
//! - [`is_falsy`] - every value the atom admits is falsy.
//! - [`could_be_truthy`] - at least one value could be truthy.
//! - [`could_be_falsy`] - at least one value could be falsy.
//!
//! [`is_literal`] reports whether the atom represents a single literal value.

use crate::ty::atom::Atom;
use crate::ty::atom::kind::AtomKind;
use crate::ty::atom::payload::array::ArrayFlag;
use crate::ty::atom::payload::array::ListFlag;
use crate::ty::atom::payload::resource::ResourceAtom;
use crate::ty::atom::payload::scalar::float::FloatAtom;
use crate::ty::atom::payload::scalar::int::IntAtom;
use crate::ty::atom::payload::scalar::mixed::Truthiness;
use crate::ty::atom::payload::scalar::string::StringLiteral;
use crate::ty::atom::payload::scalar::string::StringRefinementFlag;
use crate::ty::well_known;

/// Every value of `atom` is guaranteed truthy at runtime.
pub(crate) fn is_truthy(atom: Atom<'_>) -> bool {
    match atom {
        Atom::True => true,
        Atom::Object(_)
        | Atom::Enum(_)
        | Atom::ObjectShape(_)
        | Atom::HasMethod(_)
        | Atom::HasProperty(_)
        | Atom::ObjectAny
        | Atom::Callable(_)
        | Atom::ClassLikeString(_) => true,
        Atom::Resource(payload) => match payload {
            ResourceAtom::Open | ResourceAtom::Any => true,
            ResourceAtom::Closed => false,
        },
        Atom::Int(payload) => match payload {
            IntAtom::Literal(value) => value != 0,
            IntAtom::Range(range) => match (range.lower(), range.upper()) {
                (Some(lower), _) if lower > 0 => true,
                (_, Some(upper)) if upper < 0 => true,
                _ => false,
            },
            IntAtom::Unspecified | IntAtom::UnspecifiedLiteral => false,
        },
        Atom::Float(payload) => match payload {
            FloatAtom::Literal(literal) => literal.value() != 0.0,
            FloatAtom::Unspecified | FloatAtom::UnspecifiedLiteral => false,
        },
        Atom::String(payload) => {
            if atom == well_known::EMPTY_STRING {
                return false;
            }

            if payload.flags.contains(StringRefinementFlag::Truthy) {
                return true;
            }

            match payload.literal {
                StringLiteral::Value(value) => {
                    let bytes = value.as_bytes();
                    !bytes.is_empty() && bytes != b"0"
                }
                StringLiteral::None | StringLiteral::Unspecified => false,
            }
        }
        Atom::Array(payload) => {
            if atom == well_known::EMPTY_ARRAY {
                return false;
            }

            if payload.flags.contains(ArrayFlag::NonEmpty) {
                return true;
            }

            payload.known_items.is_some_and(|entries| entries.iter().any(|entry| !entry.optional))
        }
        Atom::List(payload) => {
            if payload.flags.contains(ListFlag::NonEmpty) {
                return true;
            }

            payload.known_elements.is_some_and(|entries| entries.iter().any(|entry| !entry.optional))
        }
        Atom::Mixed(payload) => payload.truthiness() == Truthiness::Truthy,
        Atom::GenericParameter(payload) => {
            let constraint = payload.constraint;
            !constraint.atoms.is_empty() && constraint.atoms.iter().all(|inner| is_truthy(*inner))
        }
        _ => false,
    }
}

/// Every value of `atom` is guaranteed falsy at runtime.
pub(crate) fn is_falsy(atom: Atom<'_>) -> bool {
    match atom {
        Atom::False | Atom::Null | Atom::Void => true,
        Atom::Resource(payload) => matches!(payload, ResourceAtom::Closed),
        Atom::Int(payload) => matches!(payload, IntAtom::Literal(0)),
        Atom::Float(payload) => matches!(payload, FloatAtom::Literal(literal) if literal.value() == 0.0),
        Atom::String(payload) => {
            if atom == well_known::EMPTY_STRING {
                return true;
            }

            match payload.literal {
                StringLiteral::Value(value) => value.is_empty(),
                StringLiteral::None | StringLiteral::Unspecified => false,
            }
        }
        Atom::Array(payload) => {
            if atom == well_known::EMPTY_ARRAY {
                return true;
            }

            if payload.known_items.is_some() {
                return false;
            }

            if let (Some(key), Some(value)) = (payload.key_param, payload.value_param)
                && !key.is_never()
                && !value.is_never()
            {
                return false;
            }

            !payload.flags.contains(ArrayFlag::NonEmpty)
        }
        Atom::List(payload) => {
            payload.known_elements.is_none()
                && payload.element_type.is_never()
                && !payload.flags.contains(ListFlag::NonEmpty)
        }
        Atom::Mixed(payload) => payload.truthiness() == Truthiness::Falsy,
        Atom::GenericParameter(payload) => {
            let constraint = payload.constraint;
            !constraint.atoms.is_empty() && constraint.atoms.iter().all(|inner| is_falsy(*inner))
        }
        _ => false,
    }
}

/// At least one value of `atom` could be truthy. `never` and `void` have no
/// truthy values.
pub(super) fn could_be_truthy(atom: Atom<'_>) -> bool {
    if matches!(atom.kind(), AtomKind::Never | AtomKind::Void) {
        return false;
    }

    !is_falsy(atom)
}

/// At least one value of `atom` could be falsy. `never` has no values at
/// all.
pub(super) fn could_be_falsy(atom: Atom<'_>) -> bool {
    if atom.kind() == AtomKind::Never {
        return false;
    }

    !is_truthy(atom)
}

/// `true` iff `atom` represents a single literal value.
pub(super) fn is_literal(atom: Atom<'_>) -> bool {
    match atom {
        Atom::True | Atom::False | Atom::Null | Atom::Void => true,
        Atom::Int(payload) => matches!(payload, IntAtom::Literal(_)),
        Atom::Float(payload) => matches!(payload, FloatAtom::Literal(_)),
        Atom::String(payload) => matches!(payload.literal, StringLiteral::Value(_)),
        _ => false,
    }
}
