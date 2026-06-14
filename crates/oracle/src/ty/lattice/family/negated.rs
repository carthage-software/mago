//! Negated family: `!T`, the complement of `T` against `mixed`.
//!
//! Semantically `!T` = `mixed \ T`. The lattice rules fall out from
//! that definition:
//!
//! - **`X <: !T`** iff every value of `X` is outside `T`. Decided by
//!   `meet(X, T) ≡ ⊥` - exactly the absence of overlap. Cross-kind cases
//!   such as `int <: !string` follow directly from the disjoint meet.
//! - **`!T <: X`** iff `X` covers `mixed \ T`, i.e. `T ∪ X ≡ mixed`.
//!   `X = mixed` is trivially true; `X = !U` reduces by contravariance to
//!   `U <: T`; otherwise the question becomes `refines(mixed, T ∪ X)`,
//!   which the recognised total-partition rules (e.g.
//!   `lowercase-string | !lowercase-string ≡ string`) decide. When no
//!   partition covers `mixed`, the answer is a sound `false`.
//! - **`!T <: !U`** iff `U <: T` (contravariance through negation).
//!
//! The dispatch sees these via the standard refines path: `input` of
//! kind `Negated` enters [`refines_input_negated`], `container` of
//! kind `Negated` enters [`refines_container_negated`].

use mago_allocator::Arena;

use crate::ty::atom::Atom;
use crate::ty::builder::TypeBuilder;
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;
use crate::ty::well_known;
use crate::world::World;

/// `X <: !T` iff `meet(X, T) ≡ ⊥`. The check is by structural
/// disjointness: if `X` has no value in common with `T`, every
/// value of `X` lies outside `T`, satisfying the negation.
#[inline]
pub fn refines_container_negated<'arena, S, A, W>(
    input: Atom<'arena>,
    container: Atom<'arena>,
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let Atom::Negated(inner) = container else {
        return false;
    };

    let input_type = builder.union_of(&[input]);

    crate::ty::meet::compute(input_type, *inner, world, options, report, builder).is_never()
}

/// `!T <: X` iff `mixed \ T <: X` iff `T ∪ X ≡ mixed`.
///
/// Three paths: `X = mixed` is trivially true; `X = !U` reduces by
/// contravariance to `U <: T`; otherwise we ask `refines(MIXED, T ∪ X)`
/// and let the recognized partitions drive the answer.
#[inline]
pub fn refines_input_negated<'arena, S, A, W>(
    input: Atom<'arena>,
    container: Atom<'arena>,
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    if container == well_known::MIXED {
        return true;
    }

    let Atom::Negated(input_inner) = input else {
        return false;
    };

    if let Atom::Negated(container_inner) = container {
        return crate::ty::lattice::refines(*container_inner, *input_inner, world, options, report, builder);
    }

    let mut union_atoms = builder.scratch_vec_from_slice(input_inner.atoms);
    union_atoms.push(container);
    let union_type = builder.union_of(&union_atoms);

    crate::ty::lattice::refines(well_known::TYPE_MIXED, union_type, world, options, report, builder)
}
