//! Generic-parameter subtract: narrow `T`'s constraint by removing
//! the right-hand side from its bound.

use mago_allocator::Arena;
use mago_allocator::vec::Vec as ScratchVec;

use crate::ty::atom::Atom;
use crate::ty::atom::payload::generic_parameter::GenericParameterAtom;
use crate::ty::builder::TypeBuilder;
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;
use crate::ty::Type;
use crate::world::World;

/// `(T of X) \ Y`: narrow `T`'s constraint by removing `Y` from its
/// bound. When the new constraint is empty (every value of `T` was in
/// `Y`), the result is `[]` (impossible). When the same-`T` rule fires
/// (`(T of X) \ (T of Y) → T of (X \ Y)`), both sides agree on the
/// parameter identity. Otherwise the rhs is treated as a plain type
/// the constraint must shed.
pub(in crate::ty::subtract) fn generic_parameter_minus<'scratch, 'arena, S, A, W>(
    input: Atom<'arena>,
    removed: Atom<'arena>,
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
    out: &mut ScratchVec<'scratch, Atom<'arena>, S>,
) -> bool
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let Atom::GenericParameter(input_payload) = input else {
        return false;
    };

    let other_constraint: Type<'arena> = if let Atom::GenericParameter(removed_payload) = removed {
        if input_payload.name != removed_payload.name
            || input_payload.defining_entity != removed_payload.defining_entity
        {
            return false;
        }

        removed_payload.constraint
    } else {
        builder.union_of(&[removed])
    };

    let new_constraint =
        crate::ty::subtract::compute(input_payload.constraint, other_constraint, world, options, report, builder);
    if new_constraint.is_never() {
        return true;
    }

    let narrowed = builder.generic_parameter(GenericParameterAtom { constraint: new_constraint, ..*input_payload });
    out.push(narrowed);
    true
}
