//! Sealed-class lattice rules: when a named class is declared sealed
//! (its set of direct inheritors is closed by the language engine), the
//! lattice can prove identities that open-hierarchy reasoning cannot reach.

use mago_allocator::Arena;
use mago_allocator::vec::Vec as ScratchVec;
use mago_flags::U8Flags;

use crate::path::Path;
use crate::symbol::SymbolTable;
use crate::symbol::class_like::part::inheritance::InheritedType;
use crate::symbol::part::generic::Variance;
use crate::ty::Type;
use crate::ty::atom::Atom;
use crate::ty::atom::payload::object::named::ObjectAtom;
use crate::ty::builder::TypeBuilder;
use crate::ty::lattice;
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;
use crate::ty::predicates::contains_template_anywhere;
use crate::ty::well_known;

/// The result of asking "what survives of `H`'s sealed cover after
/// these negation conjuncts filter out some inheritors?". The surviving
/// inheritors live on the builder's scratch arena.
#[derive(Debug, Clone)]
pub(crate) enum SealedResidual<'scratch, 'arena, S>
where
    S: Arena,
{
    NotSealed,
    FullyCovered,
    Surviving(ScratchVec<'scratch, Atom<'arena>, S>),
}

const DEPTH_CAP: usize = 16;

pub(crate) fn compute_residual<'scratch, 'arena, S, A>(
    head: Atom<'arena>,
    negated_inners: &[Type<'arena>],
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> SealedResidual<'scratch, 'arena, S>
where
    S: Arena,
    A: Arena,
{
    let Atom::Object(head_payload) = head else {
        return SealedResidual::NotSealed;
    };

    let class_name = head_payload.name;

    let Some(inheritors) = symbols.sealed_direct_inheritors(class_name.id) else {
        return SealedResidual::NotSealed;
    };

    let mut visited: ScratchVec<'scratch, Path<'arena>, S> = builder.scratch_vec_with(8);
    visited.push(class_name);

    compute_residual_impl(head, negated_inners, inheritors, symbols, options, report, builder, &mut visited, 0)
}

/// One level of the sealed cover: each inheritor is either covered by a
/// negation conjunct, survives as-is, or (when itself sealed) recurses into
/// its own cover.
///
/// A cycle in the sealing graph, or a recursion past [`DEPTH_CAP`], is
/// unresolvable: the walk bails with [`SealedResidual::NotSealed`] rather
/// than emitting self-referential survivors, and the bail-out propagates up
/// instead of being patched into a partial cover - otherwise downstream
/// refines / overlaps consumers would loop forever asking the same
/// question. Callers fall back to non-sealed reasoning.
#[inline]
fn compute_residual_impl<'scratch, 'arena, S, A>(
    head: Atom<'arena>,
    negated_inners: &[Type<'arena>],
    inheritors: &[InheritedType<'arena>],
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
    visited: &mut ScratchVec<'scratch, Path<'arena>, S>,
    depth: usize,
) -> SealedResidual<'scratch, 'arena, S>
where
    S: Arena,
    A: Arena,
{
    if depth > DEPTH_CAP {
        return SealedResidual::NotSealed;
    }

    let Atom::Object(head_payload) = head else {
        return SealedResidual::NotSealed;
    };

    let mut surviving: ScratchVec<'scratch, Atom<'arena>, S> = builder.scratch_vec();

    for inheritor in inheritors {
        let inheritor = inheritor.target;
        if !inheritor_admits_head_arguments(*head_payload, inheritor, symbols, options, report, builder) {
            continue;
        }

        let inheritor_atom = build_inheritor_atom(*head_payload, inheritor, symbols, builder);

        let covered = negated_inners.iter().any(|negated| {
            let inheritor_type = builder.union_of(&[inheritor_atom]);
            lattice::refines(inheritor_type, *negated, symbols, options, report, builder)
        });

        if covered {
            continue;
        }

        if let Some(grandchildren) = symbols.sealed_direct_inheritors(inheritor.id) {
            if visited.contains(&inheritor) {
                return SealedResidual::NotSealed;
            }

            visited.push(inheritor);

            let child_residual = compute_residual_impl(
                inheritor_atom,
                negated_inners,
                grandchildren,
                symbols,
                options,
                report,
                builder,
                visited,
                depth + 1,
            );

            visited.pop();

            match child_residual {
                SealedResidual::FullyCovered => {}
                SealedResidual::NotSealed => return SealedResidual::NotSealed,
                SealedResidual::Surviving(children) => surviving.extend(children),
            }
        } else {
            surviving.push(inheritor_atom);
        }
    }

    if surviving.is_empty() { SealedResidual::FullyCovered } else { SealedResidual::Surviving(surviving) }
}

/// `true` iff `inheritor`, viewed through its declared extension of the
/// sealed head class, can carry the head's `type_arguments`. A sealed class
/// equals the union of its inheritors, so an inheritor that pins the head's
/// parameter to an incompatible type (e.g. `A extends C<mixed>` against a head
/// `C<int(0)>` with an invariant `T`) shares no instance with the head and
/// contributes nothing to the cover. A position where the inheritor forwards
/// the parameter (its inherited argument still mentions a template) can take
/// any value, so it is always compatible.
#[inline]
fn inheritor_admits_head_arguments<'arena, S, A>(
    head_payload: ObjectAtom<'arena>,
    inheritor: Path<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let Some(head_arguments) = head_payload.type_arguments else {
        return true;
    };

    let arity = symbols.template_parameter_arity(head_payload.name.id);
    if arity == 0 || head_arguments.len() != arity {
        return true;
    }

    for (position, &head_argument) in head_arguments.iter().enumerate() {
        let Some(inherited) = symbols.inherited_template_argument(inheritor.id, head_payload.name.id, position) else {
            continue;
        };

        if contains_template_anywhere(inherited) {
            continue;
        }

        let variance = symbols
            .template_parameter_at(head_payload.name.id, position)
            .map(|parameter| parameter.variance)
            .unwrap_or_default();

        let compatible = match variance {
            Variance::Covariant => lattice::refines(inherited, head_argument, symbols, options, report, builder),
            Variance::Contravariant => lattice::refines(head_argument, inherited, symbols, options, report, builder),
            Variance::Invariant => {
                lattice::refines(inherited, head_argument, symbols, options, report, builder)
                    && lattice::refines(head_argument, inherited, symbols, options, report, builder)
            }
        };

        if !compatible {
            return false;
        }
    }

    true
}

#[inline]
fn build_inheritor_atom<'arena, S, A>(
    head_payload: ObjectAtom<'arena>,
    inheritor: Path<'arena>,
    symbols: &SymbolTable<'arena, A>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Atom<'arena>
where
    S: Arena,
    A: Arena,
{
    let arity = symbols.template_parameter_arity(inheritor.id);

    let type_arguments = if let Some(head_arguments) = head_payload.type_arguments
        && arity != 0
    {
        let mut projected = builder.scratch_vec_with(arity);
        for position in 0..arity {
            let argument = symbols
                .inherited_template_argument(inheritor.id, head_payload.name.id, position)
                .unwrap_or_else(|| head_arguments.get(position).copied().unwrap_or(well_known::TYPE_MIXED));
            projected.push(argument);
        }

        Some(builder.types(&projected))
    } else {
        None
    };

    builder.object(ObjectAtom { name: inheritor, type_arguments, flags: U8Flags::empty() })
}
