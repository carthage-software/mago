//! Mixed-constraint joining: order-dependent state machine that
//! collapses any `Mixed` atom with the surrounding union into a
//! single canonical mixed flavour.

use crate::ty::atom::Atom;
use crate::ty::atom::kind::AtomKind;
use crate::ty::atom::payload::scalar::float::FloatAtom;
use crate::ty::atom::payload::scalar::mixed::MixedAtom;
use crate::ty::atom::payload::scalar::mixed::Truthiness;
use crate::ty::predicates::atom::is_falsy;
use crate::ty::predicates::atom::is_truthy;
use crate::ty::well_known::NULL;

/// When any `Mixed` kind appears in the input, the result is a
/// single mixed atom whose flavour is decided by walking the
/// input in original order:
///
/// - Vanilla `mixed` is the absorbing element: once seen, the result is
///   vanilla regardless of what follows.
/// - `truthy_mixed` / `falsy_mixed` / `nonnull_mixed` set their respective
///   flag if no contradiction has been seen yet (e.g. truthy seen after
///   any non-truthy non-mixed atom forces a generic mixed).
/// - Subsequent non-mixed atoms either strengthen the constraint (e.g.
///   truthy + truthy literal preserves truthy) or contradict it
///   (e.g. truthy + literal `"0"` collapses to nonnull).
///
/// Truthy / falsy results already encode their nullability semantically;
/// the explicit non-null flag is only set on the undetermined flavour.
///
/// Returns `None` when the input has no `Mixed` atom (caller
/// proceeds with the regular join). Returns `Some(atom)` with a
/// single mixed atom to emit.
pub fn apply_mixed_constraint_join<'arena>(atoms: &[Atom<'arena>]) -> Option<Atom<'arena>> {
    let mut state = MixedJoinState::default();

    for (index, &atom) in atoms.iter().enumerate() {
        if let Atom::Mixed(payload) = atom {
            process_mixed(payload, index, atoms, &mut state);
        } else {
            if state.generic {
                continue;
            }
            if state.falsy.unwrap_or(false) {
                if !is_falsy(atom) {
                    state.falsy = Some(false);
                    state.generic = true;
                }
                continue;
            }
            if state.truthy.unwrap_or(false) {
                if !is_truthy(atom) {
                    state.truthy = Some(false);
                    state.generic = true;
                }
                continue;
            }
            if state.non_null.unwrap_or(false) && atom == NULL {
                state.non_null = Some(false);
                state.generic = true;
            }
        }
    }

    if !state.has_mixed {
        return None;
    }

    let final_non_null = state.non_null.unwrap_or(false);
    let final_truthy = state.truthy.unwrap_or(false);
    let final_falsy = state.falsy.unwrap_or(false);

    let truthiness = if final_truthy && !final_falsy {
        Truthiness::Truthy
    } else if final_falsy && !final_truthy {
        Truthiness::Falsy
    } else {
        Truthiness::Undetermined
    };

    let payload = match truthiness {
        Truthiness::Truthy => MixedAtom::EMPTY.with_truthiness(Truthiness::Truthy),
        Truthiness::Falsy => MixedAtom::EMPTY.with_truthiness(Truthiness::Falsy),
        Truthiness::Undetermined => MixedAtom::EMPTY.with_is_non_null(final_non_null),
    };

    Some(Atom::Mixed(payload))
}

/// Walk state for the order-dependent mixed-constraint state machine.
/// Each `Option<bool>` axis is undecided (`None`) until the walk commits
/// it; `generic` records that a contradiction forced the vanilla flavour.
#[derive(Default)]
struct MixedJoinState {
    truthy: Option<bool>,
    falsy: Option<bool>,
    non_null: Option<bool>,
    generic: bool,
    has_mixed: bool,
    isset_from_loop: Option<bool>,
}

#[inline]
fn process_mixed(payload: MixedAtom, index: usize, atoms: &[Atom<'_>], state: &mut MixedJoinState) {
    if payload.is_isset_from_loop() {
        if state.generic {
            return;
        }
        if state.isset_from_loop.is_none() {
            state.isset_from_loop = Some(true);
        }
        state.has_mixed = true;
        return;
    }

    state.has_mixed = true;

    let payload_is_non_null = payload.is_non_null() || payload.truthiness() == Truthiness::Truthy;
    let is_vanilla = !payload_is_non_null && !payload.is_empty() && payload.truthiness() == Truthiness::Undetermined;
    if is_vanilla {
        state.falsy = Some(false);
        state.truthy = Some(false);
        state.isset_from_loop = Some(false);
        state.generic = true;
        return;
    }

    if payload.truthiness() == Truthiness::Truthy {
        if state.generic {
            return;
        }
        state.isset_from_loop = Some(false);

        if state.falsy.unwrap_or(false) {
            state.falsy = Some(false);
            state.generic = true;
            return;
        }

        if state.truthy.is_some() {
            return;
        }

        let has_non_truthy =
            atoms_seen_so_far_any(atoms, index, |atom| non_mixed_counts_for_truthy_check(atom) && !is_truthy(atom));
        if has_non_truthy {
            state.generic = true;
            return;
        }
        state.truthy = Some(true);
    } else {
        state.truthy = Some(false);
    }

    if payload.truthiness() == Truthiness::Falsy {
        if state.generic {
            return;
        }
        state.isset_from_loop = Some(false);

        if state.truthy.unwrap_or(false) {
            state.truthy = Some(false);
            state.generic = true;
            return;
        }

        if state.falsy.is_some() {
            return;
        }

        let has_non_falsy =
            atoms_seen_so_far_any(atoms, index, |atom| non_mixed_counts_for_falsy_check(atom) && !is_falsy(atom));
        if has_non_falsy {
            state.generic = true;
            return;
        }
        state.falsy = Some(true);
    } else {
        state.falsy = Some(false);
    }

    if payload_is_non_null {
        if state.generic {
            return;
        }
        state.isset_from_loop = Some(false);

        if atoms_seen_so_far_any(atoms, index, |atom| atom == NULL) {
            state.generic = true;
            return;
        }
        if state.falsy.unwrap_or(false) {
            state.falsy = Some(false);
            state.generic = true;
            return;
        }
        if state.non_null.is_none() {
            state.non_null = Some(true);
        }
    } else {
        state.non_null = Some(false);
    }
}

/// Whether `atom` (a non-mixed kind) counts as a value-types entry
/// that would contradict a `truthy_mixed` constraint. Integers and
/// float literals are excluded.
#[inline]
fn non_mixed_counts_for_truthy_check(atom: Atom<'_>) -> bool {
    match atom {
        Atom::Int(_) => false,
        Atom::Float(payload) => !matches!(payload, FloatAtom::Literal(_)),
        _ => true,
    }
}

#[inline]
fn non_mixed_counts_for_falsy_check(atom: Atom<'_>) -> bool {
    non_mixed_counts_for_truthy_check(atom)
}

#[inline]
fn atoms_seen_so_far_any<'arena>(
    atoms: &[Atom<'arena>],
    index: usize,
    predicate: impl Fn(Atom<'arena>) -> bool,
) -> bool {
    atoms[..index].iter().any(|&atom| atom.kind() != AtomKind::Mixed && predicate(atom))
}
