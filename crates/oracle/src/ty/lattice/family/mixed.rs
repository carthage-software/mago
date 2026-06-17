//! Mixed family. Container is a `mixed` atom carrying axis flags
//! (`is_non_null`, truthiness, `is_empty`, `is_isset_from_loop`).
//!
//! Vanilla `mixed` is handled by the universal Top axiom in
//! `atom_refines`; this family fires only
//! for narrowed mixed containers. The input refines the container iff
//! every axis the container constrains is implied by the input ; either
//! because the input is a `mixed` carrying at least the same flags, or
//! because the input's atom kind structurally guarantees the property
//! (e.g. an `int` is non-null, a named object is truthy, the empty string
//! is falsy).
//!
//! `isset_from_loop` is an analysis-internal marker (a value that flows
//! through a loop body): only an input that already carries the marker
//! satisfies a container demanding it.

use crate::ty::atom::Atom;
use crate::ty::atom::payload::array::ArrayFlag;
use crate::ty::atom::payload::array::ListFlag;
use crate::ty::atom::payload::scalar::float::FloatAtom;
use crate::ty::atom::payload::scalar::int::IntAtom;
use crate::ty::atom::payload::scalar::mixed::Truthiness;
use crate::ty::atom::payload::scalar::string::StringLiteral;
use crate::ty::atom::payload::scalar::string::StringRefinementFlag;
use crate::ty::well_known;

#[inline]
#[must_use]
pub fn refines(input: Atom<'_>, container: Atom<'_>) -> bool {
    let Atom::Mixed(container_payload) = container else {
        return false;
    };

    if container_payload.is_non_null() && !is_non_null(input) {
        return false;
    }

    match container_payload.truthiness() {
        Truthiness::Truthy => {
            if truthiness_of(input) != Truthiness::Truthy {
                return false;
            }
        }
        Truthiness::Falsy => {
            if truthiness_of(input) != Truthiness::Falsy {
                return false;
            }
        }
        Truthiness::Undetermined => {}
    }

    if container_payload.is_empty() && truthiness_of(input) != Truthiness::Falsy {
        return false;
    }

    if container_payload.is_isset_from_loop() {
        let Atom::Mixed(input_payload) = input else {
            return false;
        };

        if !input_payload.is_isset_from_loop() {
            return false;
        }
    }

    true
}

/// `true` iff `input` cannot be `null`.
#[inline]
pub(crate) fn is_non_null(input: Atom<'_>) -> bool {
    match input {
        Atom::Null | Atom::Void => false,
        Atom::Mixed(payload) => payload.is_non_null() || payload.truthiness() == Truthiness::Truthy,
        Atom::GenericParameter(payload) => payload.constraint.atoms.iter().all(|atom| is_non_null(*atom)),
        Atom::Intersected(payload) => {
            if is_non_null(*payload.head) {
                return true;
            }

            payload.conjuncts.iter().any(|conjunct| is_non_null(*conjunct))
        }
        _ => true,
    }
}

/// `true` iff `inner` (a `Negated` inner type's atoms) collectively
/// excludes every falsy witness for `head`'s atom kind.
#[inline]
fn conjuncts_exclude_falsy_witnesses(head: Atom<'_>, inner: &[Atom<'_>]) -> bool {
    match head {
        Atom::Int(_) => inner.contains(&well_known::INT_ZERO),
        Atom::Float(_) => {
            inner.iter().any(|atom| matches!(atom, Atom::Float(FloatAtom::Literal(literal)) if literal.value() == 0.0))
        }
        Atom::Bool => inner.contains(&well_known::FALSE),
        _ => false,
    }
}

/// Best-known truthiness of `input` as a single value. Returns
/// [`Truthiness::Undetermined`] when both possibilities remain open.
#[inline]
pub(crate) fn truthiness_of(input: Atom<'_>) -> Truthiness {
    match input {
        Atom::True => Truthiness::Truthy,
        Atom::False | Atom::Null => Truthiness::Falsy,
        Atom::Bool => Truthiness::Undetermined,
        Atom::ObjectAny
        | Atom::Object(_)
        | Atom::Enum(_)
        | Atom::Resource(_)
        | Atom::Callable(_)
        | Atom::ObjectShape(_)
        | Atom::HasMethod(_)
        | Atom::HasProperty(_) => Truthiness::Truthy,
        Atom::ClassLikeString(_) => Truthiness::Truthy,
        Atom::Int(payload) => match payload {
            IntAtom::Literal(0) => Truthiness::Falsy,
            IntAtom::Literal(_) => Truthiness::Truthy,
            IntAtom::Range(range) => match (range.lower(), range.upper()) {
                (Some(lower), _) if lower > 0 => Truthiness::Truthy,
                (_, Some(upper)) if upper < 0 => Truthiness::Truthy,
                _ => Truthiness::Undetermined,
            },
            _ => Truthiness::Undetermined,
        },
        Atom::Float(FloatAtom::Literal(literal)) => {
            if literal.value() == 0.0 {
                Truthiness::Falsy
            } else {
                Truthiness::Truthy
            }
        }
        Atom::Float(_) => Truthiness::Undetermined,
        Atom::String(payload) => {
            if input == well_known::EMPTY_STRING {
                return Truthiness::Falsy;
            }

            if payload.flags.contains(StringRefinementFlag::Truthy) {
                return Truthiness::Truthy;
            }

            match payload.literal {
                StringLiteral::Value(value) => {
                    let bytes = value;
                    if bytes.is_empty() || bytes == b"0" { Truthiness::Falsy } else { Truthiness::Truthy }
                }
                _ => Truthiness::Undetermined,
            }
        }
        Atom::Array(payload) => {
            if input == well_known::EMPTY_ARRAY {
                return Truthiness::Falsy;
            }

            if payload.flags.contains(ArrayFlag::NonEmpty) {
                return Truthiness::Truthy;
            }

            if payload.key_param.is_some_and(|key_param| key_param.is_never())
                && payload.value_param.is_some_and(|value_param| value_param.is_never())
                && payload.known_items.is_none()
            {
                return Truthiness::Falsy;
            }

            Truthiness::Undetermined
        }
        Atom::List(payload) => {
            if payload.flags.contains(ListFlag::NonEmpty) {
                return Truthiness::Truthy;
            }

            if payload.element_type.is_never() && payload.known_elements.is_none() {
                return Truthiness::Falsy;
            }

            Truthiness::Undetermined
        }
        Atom::Mixed(payload) => {
            if payload.is_empty() {
                return Truthiness::Falsy;
            }

            payload.truthiness()
        }
        Atom::Intersected(payload) => {
            let head_truthiness = truthiness_of(*payload.head);
            if head_truthiness == Truthiness::Truthy || head_truthiness == Truthiness::Falsy {
                return head_truthiness;
            }

            let mut all_truthy_via_excluded_falsy = false;
            for conjunct in payload.conjuncts {
                if let Atom::Negated(inner) = conjunct
                    && conjuncts_exclude_falsy_witnesses(*payload.head, inner.atoms)
                {
                    all_truthy_via_excluded_falsy = true;
                }

                let conjunct_truthiness = truthiness_of(*conjunct);
                if conjunct_truthiness == Truthiness::Truthy {
                    return Truthiness::Truthy;
                }

                if conjunct_truthiness == Truthiness::Falsy {
                    return Truthiness::Falsy;
                }
            }

            if all_truthy_via_excluded_falsy { Truthiness::Truthy } else { Truthiness::Undetermined }
        }
        Atom::GenericParameter(payload) => {
            let mut accumulated: Option<Truthiness> = None;
            for atom in payload.constraint.atoms {
                let truthiness = truthiness_of(*atom);
                accumulated = Some(match accumulated {
                    None => truthiness,
                    Some(previous) if previous == truthiness => previous,
                    _ => return Truthiness::Undetermined,
                });
            }

            accumulated.unwrap_or(Truthiness::Undetermined)
        }
        _ => Truthiness::Undetermined,
    }
}
