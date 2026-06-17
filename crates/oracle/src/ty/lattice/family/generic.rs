//! Generic-parameter family. Comparison rules for the case where the
//! *container* is a `T` (a `@template` reference).
//!
//! Rules :
//!
//! - **Same-T**: `T_a <: T_b` when both sides name the same parameter
//!   declared by the same defining entity AND `a`'s constraint refines
//!   `b`'s. Constraints can differ when one side has been narrowed by an
//!   earlier assertion (`T of (int|string)` narrowed to `T of int`); the
//!   narrower side refines the wider but not vice versa.
//! - **Inherited-T**: `T_C <: T_D` when `C` (transitively) extends `D` binding
//!   its own parameter `T_C` into `D`'s slot for `T_D` (`class C<TC> extends
//!   D<TC>`). In that case the two name the same variable, so a `T_C` value is
//!   a `T_D` value. The relation is decided by
//!   [`World::template_parameter_forwards_to`], which is transitively closed,
//!   keeping the derived subtyping transitive. It is one-directional: `T_D <:
//!   T_C` does not follow (a bare `D` could be specialised to anything).
//!
//! The dual rule (input is `T`, container is non-generic, refined through
//! `T`'s constraint) lives in `atom_refines` because it
//! must fire before the container-kind dispatch.

use mago_allocator::Arena;

use crate::ty::atom::Atom;
use crate::ty::atom::payload::generic_parameter::DefiningEntity;
use crate::ty::builder::TypeBuilder;
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;
use crate::world::World;

#[inline]
pub fn refines<'arena, S, A, W>(
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
    let Atom::GenericParameter(container_payload) = container else {
        return false;
    };

    let Atom::GenericParameter(input_payload) = input else {
        return false;
    };

    if input_payload.name == container_payload.name
        && input_payload.defining_entity == container_payload.defining_entity
    {
        return crate::ty::lattice::refines(
            input_payload.constraint,
            container_payload.constraint,
            world,
            options,
            report,
            builder,
        );
    }

    parameter_forwards(input_payload, container_payload, world)
}

/// Inherited-T: the input parameter is the same variable as the container
/// parameter through inheritance forwarding (`class C<TC> extends D<TC>`
/// makes `C::TC <: D::T`). Both must be class-defined parameters; the world
/// decides the (transitively closed) forwarding.
#[inline]
fn parameter_forwards<'arena, W>(
    input: &crate::ty::atom::payload::generic_parameter::GenericParameterAtom<'arena>,
    container: &crate::ty::atom::payload::generic_parameter::GenericParameterAtom<'arena>,
    world: &W,
) -> bool
where
    W: World<'arena>,
{
    let (DefiningEntity::ClassLike(input_class), DefiningEntity::ClassLike(container_class)) =
        (input.defining_entity, container.defining_entity)
    else {
        return false;
    };

    world.template_parameter_forwards_to(input_class.id, input.name, container_class.id, container.name)
}
