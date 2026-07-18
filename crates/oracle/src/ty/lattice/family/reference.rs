//! Indirection family: `Variable`, `Reference`, `MemberReference`,
//! `GlobalReference`, `Alias`, `Conditional`, `Derived`.
//!
//! These atoms are normally **resolved by the analyser before
//! subtyping is consulted**: a `Reference` becomes the
//! type recorded for that name in `Γ`, an `Alias` substitutes its body,
//! a `Derived` evaluates once its inputs are concrete. Two unresolved
//! atoms refine each other only by *structural identity*.
//!
//! Structural equality on [`Atom`] makes that identity check direct, and
//! the universal reflexivity axiom in
//! `atom_refines` catches
//! `input == container` before this family is even consulted. As a result,
//! this file's only job is to keep the dispatch honest:
//!
//! - Same-kind input with a *different* payload differs structurally;
//!   without resolution we cannot decide subtyping, so the answer is
//!   `false`.
//! - Cross-kind input refines an indirection container only via
//!   resolution, which the analyser must run beforehand. Until then,
//!   `false`.
//!
//! Returning `false` is sound: it just means "the lattice cannot prove
//! this without resolution". A downstream analyser that resolves the
//! atom and re-asks gets the real answer.

use crate::ty::atom::Atom;
use crate::ty::atom::kind::AtomKind;

#[inline]
#[must_use]
pub fn refines(input: Atom<'_>, container: Atom<'_>) -> bool {
    if !is_indirection_kind(container.kind()) {
        return false;
    }

    if input == container {
        return true;
    }

    false
}

#[inline]
const fn is_indirection_kind(kind: AtomKind) -> bool {
    matches!(
        kind,
        AtomKind::Variable
            | AtomKind::Reference
            | AtomKind::MemberReference
            | AtomKind::GlobalReference
            | AtomKind::Alias
            | AtomKind::Conditional
            | AtomKind::Derived
    )
}
