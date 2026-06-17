//! Generic-parameter family meet: narrow `T`'s constraint when an
//! assertion eliminates part of its declared bound.
//!
//! `T of X ∧ Y` is a fresh `T` whose constraint is `X ∩ Y` (computed
//! recursively via the lattice meet). When the narrowed constraint is
//! empty, the result is `None` (impossible; no value of T can satisfy
//! both the original bound and the assertion). When `T of X` already
//! refines `Y`, the subsumption rule in [`crate::ty::meet`] short-circuits
//! before we get here, so this rule fires only for genuine narrowings.
//!
//! Same-`T` meets (`T of X ∧ T of Y`) intersect both constraints, since
//! both sides describe the same parameter under different bounds.

use mago_allocator::Arena;

use crate::ty::atom::Atom;
use crate::ty::atom::payload::generic_parameter::DefiningEntity;
use crate::ty::atom::payload::generic_parameter::GenericParameterAtom;
use crate::ty::builder::TypeBuilder;
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;
use crate::world::World;

pub(in crate::ty::meet) fn generic_parameter_meet<'arena, S, A, W>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let (template, other_constraint) = match (a, b) {
        (Atom::GenericParameter(a_payload), Atom::GenericParameter(b_payload)) => {
            if a_payload.name != b_payload.name || a_payload.defining_entity != b_payload.defining_entity {
                // Distinct parameters: when one is the same variable as the
                // other through inheritance forwarding, that one is the
                // subtype and so the meet. Otherwise they are disjoint.
                if parameter_forwards(a_payload, b_payload, world) {
                    return Some(a);
                }
                if parameter_forwards(b_payload, a_payload, world) {
                    return Some(b);
                }

                return None;
            }

            (a_payload, b_payload.constraint)
        }
        (Atom::GenericParameter(a_payload), _) => (a_payload, builder.union_of(&[b])),
        (_, Atom::GenericParameter(b_payload)) => (b_payload, builder.union_of(&[a])),
        _ => return None,
    };

    let new_constraint =
        crate::ty::meet::compute(template.constraint, other_constraint, world, options, report, builder);
    if new_constraint.is_never() {
        return None;
    }

    Some(builder.generic_parameter(GenericParameterAtom { constraint: new_constraint, ..*template }))
}

/// `true` iff `from` is the same variable as `to` through inheritance
/// forwarding - then `from <: to`, so `from` is their meet.
#[inline]
fn parameter_forwards<'arena, W>(
    from: &GenericParameterAtom<'arena>,
    to: &GenericParameterAtom<'arena>,
    world: &W,
) -> bool
where
    W: World<'arena>,
{
    let (DefiningEntity::ClassLike(from_class), DefiningEntity::ClassLike(to_class)) =
        (from.defining_entity, to.defining_entity)
    else {
        return false;
    };

    world.template_parameter_forwards_to(from_class.id, from.name, to_class.id, to.name)
}
