//! Atom-level transformation primitives over a [`Type`].
//!
//! Four pure functions ([`map`], [`flat_map`], [`filter_map`],
//! [`filter`]) traverse a type **structurally**: the closure is
//! invoked at every atom position, including atoms buried inside
//! nested type-carriers (object type-arguments, list element types,
//! keyed-array keys/values/known items, iterable key/value,
//! class-like-string constraints, conditional branches, derived
//! operands, generic-parameter constraints, callable signatures).
//!
//! These primitives are the shared structural walker used by
//! [`crate::widen`] and (in a follow-up refactor) by
//! [`crate::expand`], [`crate::ty::template::standin`], and
//! [`crate::ty::template::substitute`].
//!
//! # Order
//!
//! Post-order. Nested types are rebuilt first, then the closure is
//! invoked on the (possibly rebuilt) atom. The closure therefore sees
//! an atom whose nested types have already been transformed; useful
//! when the decision depends on the final shape of the children.
//!
//! # Cost model
//!
//! At each type-level the walker accumulates results in a single
//! `Vec<Atom>` and commits them through the builder exactly once.
//! Nested type-levels each commit once for their level. A type with
//! `N` top-level atoms that each become `K` atoms after [`flat_map`]
//! costs **one** top-level union construction (not `N`) plus the
//! per-nested-level constructions dictated by the structure.
//!
//! When the closure returns each atom unchanged at every level (and
//! no recursion observed a change), the original [`Type`] is returned
//! verbatim ([`Type::ptr_eq`]-identical); the builder is not touched
//! at all.

mod walk;

use mago_allocator::Arena;

use crate::ty::Type;
use crate::ty::atom::Atom;
use crate::ty::builder::TypeBuilder;

use self::walk::Outcome;
use self::walk::walk;

/// Apply `transform` to every atom in `ty`, recursively descending
/// into nested types. The closure runs in post-order (after the atom's
/// nested types have been transformed).
///
/// Returns the original [`Type`] unchanged when the closure returned
/// each atom identical at every level.
#[inline]
pub fn map<'arena, S, A, F>(
    ty: Type<'arena>,
    mut transform: F,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
    F: FnMut(Atom<'arena>) -> Atom<'arena>,
{
    walk(
        ty,
        &mut |atom, _builder| {
            let replaced = transform(atom);
            if replaced == atom { Outcome::Unchanged } else { Outcome::Single(replaced) }
        },
        builder,
    )
}

/// Apply `transform` to every atom in `ty` exactly like [`map`], with
/// the builder threaded into the closure so it can construct
/// replacement atoms mid-walk.
#[inline]
pub(crate) fn map_with_builder<'scratch, 'arena, S, A, F>(
    ty: Type<'arena>,
    mut transform: F,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
    F: FnMut(Atom<'arena>, &mut TypeBuilder<'scratch, 'arena, S, A>) -> Atom<'arena>,
{
    walk(
        ty,
        &mut |atom, builder| {
            let replaced = transform(atom, builder);
            if replaced == atom { Outcome::Unchanged } else { Outcome::Single(replaced) }
        },
        builder,
    )
}

pub(crate) fn flat_map_with_builder<'scratch, 'arena, S, A, F>(
    ty: Type<'arena>,
    mut transform: F,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
    F: FnMut(Atom<'arena>, &mut TypeBuilder<'scratch, 'arena, S, A>) -> Vec<Atom<'arena>>,
{
    walk(
        ty,
        &mut |atom, builder| {
            let collected = transform(atom, builder);
            match collected.as_slice() {
                [only] if *only == atom => Outcome::Unchanged,
                [only] => Outcome::Single(*only),
                [] => Outcome::Drop,
                _ => Outcome::Many(collected),
            }
        },
        builder,
    )
}

/// Apply `transform` to every atom, replacing each with zero or more
/// atoms. Returning an empty iterator drops the atom from the
/// surrounding union (collapses to `never` if the level becomes
/// empty).
#[inline]
pub fn flat_map<'arena, S, A, F, I>(
    ty: Type<'arena>,
    mut transform: F,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
    F: FnMut(Atom<'arena>) -> I,
    I: IntoIterator<Item = Atom<'arena>>,
{
    walk(
        ty,
        &mut |atom, _builder| {
            let collected: Vec<Atom<'arena>> = transform(atom).into_iter().collect();
            match collected.as_slice() {
                [only] if *only == atom => Outcome::Unchanged,
                [only] => Outcome::Single(*only),
                [] => Outcome::Drop,
                _ => Outcome::Many(collected),
            }
        },
        builder,
    )
}

/// Apply `transform` to every atom, dropping any atom for which
/// `transform` returns `None`.
#[inline]
pub fn filter_map<'arena, S, A, F>(
    ty: Type<'arena>,
    mut transform: F,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
    F: FnMut(Atom<'arena>) -> Option<Atom<'arena>>,
{
    walk(
        ty,
        &mut |atom, _builder| match transform(atom) {
            Some(replaced) if replaced == atom => Outcome::Unchanged,
            Some(replaced) => Outcome::Single(replaced),
            None => Outcome::Drop,
        },
        builder,
    )
}

/// Drop every atom for which `predicate` returns `false`.
#[inline]
pub fn filter<'arena, S, A, F>(
    ty: Type<'arena>,
    mut predicate: F,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
    F: FnMut(&Atom<'arena>) -> bool,
{
    walk(ty, &mut |atom, _builder| if predicate(&atom) { Outcome::Unchanged } else { Outcome::Drop }, builder)
}
