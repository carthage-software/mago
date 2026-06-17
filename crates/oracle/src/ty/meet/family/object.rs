//! Compositional object intersection. Two paths:
//!
//! - Different classes (or one with no shared `name`): glue them as a
//!   single intersection-bearing object (`Foo & Bar`), choosing the
//!   canonical-smallest participant as the head.
//! - Same class, different generic arguments: merge args pointwise
//!   under the world-declared variance. Invariant args meet (must
//!   agree); covariant args meet; contravariant args join. If any
//!   invariant slot meets to `never`, the whole intersection is
//!   uninhabitable and we return `None`.

use mago_allocator::Arena;
use mago_allocator::vec::Vec as ScratchVec;

use crate::path::Path;
use crate::symbol::part::generic::Variance;
use crate::ty::Type;
use crate::ty::atom::Atom;
use crate::ty::atom::kind::AtomKind;
use crate::ty::atom::payload::generic_parameter::DefiningEntity;
use crate::ty::atom::payload::generic_parameter::GenericParameterAtom;
use crate::ty::atom::payload::object::named::ObjectAtom;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeKind;
use crate::ty::builder::TypeBuilder;
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;
use crate::ty::well_known;
use crate::world::TemplateParameter;
use crate::world::World;

pub(in crate::ty::meet) fn compose_object_intersection<'scratch, 'arena, S, A, W>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let same_class_merged = merge_same_class_participants(&[a, b], world, options, report, builder)?;
    let reconciled = reconcile_descendant_participants(same_class_merged, world, options, report, builder)?;

    finalize_object_composition(&reconciled, world, builder)
}

/// Reconcile pairs of object participants where one nominally
/// descends the other. The descendant's view of the ancestor (via
/// `World::inherited_template_argument`) must be compatible with the
/// ancestor's args under the ancestor's variance; if not, the
/// intersection is uninhabited (`None`). When compatible, the
/// ancestor is redundant (the descendant is strictly more specific)
/// and we drop it from the merged list.
#[inline]
fn reconcile_descendant_participants<'scratch, 'arena, S, A, W>(
    mut merged: ScratchVec<'scratch, Atom<'arena>, S>,
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<ScratchVec<'scratch, Atom<'arena>, S>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let mut keep: ScratchVec<'scratch, bool, S> = builder.scratch_vec_with(merged.len());
    keep.resize(merged.len(), true);

    for descendant_index in 0..merged.len() {
        if !keep[descendant_index] {
            continue;
        }

        let Atom::Object(descendant_payload) = merged[descendant_index] else {
            continue;
        };

        for ancestor_index in 0..merged.len() {
            if descendant_index == ancestor_index || !keep[ancestor_index] {
                continue;
            }

            let Atom::Object(ancestor_payload) = merged[ancestor_index] else {
                continue;
            };

            if descendant_payload.name == ancestor_payload.name {
                continue;
            }

            if !world.descends_from(descendant_payload.name.id, ancestor_payload.name.id) {
                continue;
            }

            if !descendant_args_satisfy_ancestor(
                *descendant_payload,
                *ancestor_payload,
                world,
                options,
                report,
                builder,
            ) {
                return None;
            }

            keep[ancestor_index] = false;
        }
    }

    let mut keep = keep.into_iter();
    merged.retain(|_| keep.next().unwrap_or(false));

    Some(merged)
}

/// `true` iff `negated_atom` (a `Negated` conjunct) excludes every
/// instance of class `class_name`. Fires for negations of a bare-named
/// ancestor of `class_name`: every instance of `class_name` is also an
/// instance of the ancestor, so the negation rules them all out.
#[inline]
fn negation_excludes_class<'arena, W>(negated_atom: Atom<'arena>, class_name: Path<'_>, world: &W) -> bool
where
    W: World<'arena>,
{
    let Atom::Negated(inner) = negated_atom else {
        return false;
    };

    if !inner.kinds.contains(AtomKind::Object) {
        return false;
    }

    inner.atoms.iter().any(|&inner_atom| {
        let Atom::Object(inner_payload) = inner_atom else {
            return false;
        };

        world.descends_from(class_name.id, inner_payload.name.id)
    })
}

/// Project `descendant`'s view of `ancestor` through the world's
/// inherited-template-argument rule and substitute `descendant`'s
/// actual args, then check each position against `ancestor`'s args
/// under `ancestor`'s variance.
#[inline]
fn descendant_args_satisfy_ancestor<'arena, S, A, W>(
    descendant: ObjectAtom<'arena>,
    ancestor: ObjectAtom<'arena>,
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
    let arity = world.template_parameter_arity(ancestor.name.id);
    if arity == 0 {
        return true;
    }

    let ancestor_args: &[Type<'arena>] = match ancestor.type_arguments {
        Some(arguments) => arguments,
        None => return true,
    };
    if ancestor_args.len() != arity {
        return true;
    }

    let descendant_actuals: &[Type<'arena>] = descendant.type_arguments.unwrap_or(&[]);

    for (position, &ancestor_arg) in ancestor_args.iter().enumerate() {
        let Some(inherited) = world.inherited_template_argument(descendant.name.id, ancestor.name.id, position) else {
            return true;
        };

        let resolver = |payload: &GenericParameterAtom<'arena>| -> Option<Type<'arena>> {
            if payload.defining_entity != DefiningEntity::ClassLike(descendant.name) {
                return None;
            }

            let parameter_position = world.template_parameter_index(descendant.name.id, payload.name)?;
            descendant_actuals.get(parameter_position).copied()
        };
        let substituted = crate::ty::template::substitute(inherited, &resolver, builder);

        let variance = world
            .template_parameter_at(ancestor.name.id, position)
            .map(|parameter: TemplateParameter<'arena>| parameter.variance)
            .unwrap_or_default();

        let compatible = match variance {
            Variance::Invariant => {
                crate::ty::lattice::refines(substituted, ancestor_arg, world, options, report, builder)
                    && crate::ty::lattice::refines(ancestor_arg, substituted, world, options, report, builder)
            }
            Variance::Covariant => {
                crate::ty::lattice::refines(substituted, ancestor_arg, world, options, report, builder)
            }
            Variance::Contravariant => {
                crate::ty::lattice::refines(ancestor_arg, substituted, world, options, report, builder)
            }
        };
        if !compatible {
            return false;
        }
    }

    true
}

/// `object{...} ∩ has-method<m>` (or `has-property<p>` /
/// `ObjectShape`): the shape never guarantees the structural,
/// and its known properties may be optional or the shape unsealed,
/// so the intersection composes through the
/// [`Intersected`](AtomKind::Intersected) wrapper rather than
/// dropping the structural.
pub(in crate::ty::meet) fn compose_shape_with_structural<'arena, S, A>(
    shape: Atom<'arena>,
    structural: Atom<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
{
    Some(builder.intersected(shape, &[structural]))
}

/// Compose a nominal object atom with a structural conjunct
/// (`HasMethod`, `HasProperty`, `ObjectShape`). An unknown class
/// might gain the structural feature via a subclass, so the
/// intersection stays alive. A final class that doesn't satisfy
/// the structural collapses to `None`. When the world already
/// records that a positive class in the intersection has the
/// method/property, the redundant conjunct is dropped.
pub(in crate::ty::meet) fn compose_object_with_structural<'scratch, 'arena, S, A, W>(
    object: Atom<'arena>,
    structural: Atom<'arena>,
    world: &W,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let Atom::Object(object_payload) = object else {
        return None;
    };

    if structural_uninhabited_under_finality(&[object_payload.name], structural, world) {
        return None;
    }

    let drop_as_redundant = matches!(structural.kind(), AtomKind::HasMethod | AtomKind::HasProperty)
        && class_satisfies_structural(object_payload.name, structural, world);
    if drop_as_redundant {
        return Some(object);
    }

    finalize_object_composition(&[object, structural], world, builder)
}

/// `final C & HasMethod(m)` is uninhabited when `C` is final and the
/// world says it lacks `m`: a final class admits no subclass that
/// could add the member. The check fires only for nominal classes
/// the world declares final; open-world classes always keep the
/// structural intersection.
#[inline]
fn structural_uninhabited_under_finality<'arena, W>(
    classes: &[Path<'arena>],
    structural: Atom<'arena>,
    world: &W,
) -> bool
where
    W: World<'arena>,
{
    classes.iter().any(|&class| world.is_final(class.id) && !class_satisfies_structural(class, structural, world))
}

#[inline]
fn class_satisfies_structural<'arena, W>(class: Path<'_>, structural: Atom<'arena>, world: &W) -> bool
where
    W: World<'arena>,
{
    match structural {
        Atom::HasMethod(payload) => world.class_has_method(class.id, payload.method_name),
        Atom::HasProperty(payload) => world.class_has_property(class.id, payload.property_name),
        _ => true,
    }
}

/// Glue the merged participants into a single intersection-bearing
/// atom. Negated conjuncts are checked first: a negation that excludes
/// a positive class, or whose inner accepts another conjunct in the
/// same intersection (the `X & !X` shape), makes the composition
/// uninhabited.
#[inline]
fn finalize_object_composition<'scratch, 'arena, S, A, W>(
    merged: &[Atom<'arena>],
    world: &W,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let mut object_parts: ScratchVec<'scratch, Atom<'arena>, S> = builder.scratch_vec();
    let mut other_parts: ScratchVec<'scratch, Atom<'arena>, S> = builder.scratch_vec();
    for &atom in merged {
        if atom.kind() == AtomKind::Object {
            object_parts.push(atom);
        } else {
            other_parts.push(atom);
        }
    }

    if !single_inheritance_consistent(&object_parts, world, builder) {
        return None;
    }

    let has_negated = other_parts.iter().any(|atom| atom.kind() == AtomKind::Negated);
    if has_negated {
        for &negated in other_parts.iter().filter(|atom| atom.kind() == AtomKind::Negated) {
            for &object in &object_parts {
                let Atom::Object(object_payload) = object else {
                    continue;
                };

                if negation_excludes_class(negated, object_payload.name, world) {
                    return None;
                }
            }

            let Atom::Negated(negated_inner) = negated else {
                continue;
            };
            for &positive in &other_parts {
                if positive == negated {
                    continue;
                }

                let positive_type = builder.union_of(&[positive]);
                if crate::ty::lattice::refines(
                    positive_type,
                    *negated_inner,
                    world,
                    LatticeOptions::default(),
                    &mut LatticeReport::new(),
                    builder,
                ) {
                    return None;
                }
            }
        }
    }

    object_parts.sort();
    object_parts.dedup();
    other_parts.sort();
    other_parts.dedup();

    let mut object_iterator = object_parts.into_iter();
    let head = object_iterator.next()?;
    let mut conjuncts: ScratchVec<'scratch, Atom<'arena>, S> = builder.scratch_vec();
    conjuncts.extend(object_iterator);
    conjuncts.extend(other_parts);

    Some(builder.intersected(head, &conjuncts))
}

/// `Foo & Bar & …` is inhabitable when no finality witness rules it
/// out. A `final` class in the intersection has only itself as a
/// possible witness, so it must descend every other class in the
/// intersection. When that fails, the type is uninhabited and
/// compose collapses to `None`. Without a final witness we
/// optimistically allow the composition (PHP's open world might
/// supply a common subclass via interfaces / traits).
#[inline]
fn single_inheritance_consistent<'scratch, 'arena, S, A, W>(
    objects: &[Atom<'arena>],
    world: &W,
    builder: &TypeBuilder<'scratch, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let mut names: ScratchVec<'scratch, Path<'arena>, S> = builder.scratch_vec_with(objects.len());
    names.extend(objects.iter().filter_map(|atom| match atom {
        Atom::Object(payload) => Some(payload.name),
        _ => None,
    }));
    for &final_candidate in &names {
        if !world.is_final(final_candidate.id) {
            continue;
        }

        for &other in &names {
            if other == final_candidate {
                continue;
            }

            if !world.descends_from(final_candidate.id, other.id) && !world.descends_from(other.id, final_candidate.id)
            {
                return false;
            }
        }
    }

    for (index, &left) in names.iter().enumerate() {
        if world.class_like_kind(left.id) != Some(ClassLikeKind::Class) {
            continue;
        }

        for &right in &names[index + 1..] {
            if world.class_like_kind(right.id) != Some(ClassLikeKind::Class) {
                continue;
            }

            if left != right && !world.descends_from(left.id, right.id) && !world.descends_from(right.id, left.id) {
                return false;
            }
        }
    }

    true
}

#[inline]
fn merge_same_class_participants<'scratch, 'arena, S, A, W>(
    participants: &[Atom<'arena>],
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<ScratchVec<'scratch, Atom<'arena>, S>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let mut out: ScratchVec<'scratch, Atom<'arena>, S> = builder.scratch_vec_with(participants.len());

    for &atom in participants {
        let Atom::Object(payload) = atom else {
            out.push(atom);
            continue;
        };

        let mut absorbed = false;
        for slot in &mut out {
            let Atom::Object(existing) = *slot else {
                continue;
            };

            if existing.name != payload.name {
                continue;
            }

            let merged_args = merge_args(*existing, *payload, world, options, report, builder)?;
            *slot =
                builder.object(ObjectAtom { name: payload.name, type_arguments: merged_args, flags: payload.flags });
            absorbed = true;
            break;
        }

        if !absorbed {
            out.push(atom);
        }
    }

    Some(out)
}

/// Merge the type arguments of two same-named object atoms pointwise
/// under each position's declared variance. Arity-0 classes can carry
/// meaningless explicit args, so both sides collapse to the bare
/// nominal form regardless of how each side was constructed. Invariant
/// slots require the args to be mutually refining (value-equal): a mere
/// non-empty intersection isn't enough, since `B<int|enum>` and
/// `B<int>` admit no shared `B` instance when `T` is pinned exactly.
/// Both sides are first normalized to exactly `arity` positions
/// (over-supply truncated, under-supply default-filled).
#[inline]
fn merge_args<'scratch, 'arena, S, A, W>(
    a: ObjectAtom<'arena>,
    b: ObjectAtom<'arena>,
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<Option<&'arena [Type<'arena>]>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let arity = world.template_parameter_arity(a.name.id);

    if arity == 0 {
        return Some(None);
    }

    if a.type_arguments.is_none() && b.type_arguments.is_none() {
        return Some(None);
    }

    if a.type_arguments.is_none() || b.type_arguments.is_none() {
        let all_contravariant = (0..arity).all(|index| {
            matches!(
                world.template_parameter_at(a.name.id, index).map(|parameter| parameter.variance),
                Some(Variance::Contravariant)
            )
        });

        // A raw side carries no lower bound to join, so an all-contravariant
        // raw meet stays raw. With coercion on, a raw side meets to the pinned
        // arguments verbatim; without it (sound default) the raw side is
        // default-filled and merged pointwise below so the result is a genuine
        // lower bound of the pinned side.
        if all_contravariant {
            return Some(None);
        }

        if options.template_default_coercion {
            let arguments = a.type_arguments.or(b.type_arguments).unwrap_or_default();
            return Some(Some(arguments));
        }
    }

    let a_supplied: &[Type<'arena>] = a.type_arguments.unwrap_or_default();
    let b_supplied: &[Type<'arena>] = b.type_arguments.unwrap_or_default();
    let fill = |index: usize| -> Type<'arena> {
        world
            .template_parameter_at(a.name.id, index)
            .and_then(|parameter| parameter.upper_bound)
            .unwrap_or(well_known::TYPE_MIXED)
    };
    let mut a_args: ScratchVec<'scratch, Type<'arena>, S> = builder.scratch_vec_with(arity);
    a_args.extend((0..arity).map(|index| a_supplied.get(index).copied().unwrap_or_else(|| fill(index))));
    let mut b_args: ScratchVec<'scratch, Type<'arena>, S> = builder.scratch_vec_with(arity);
    b_args.extend((0..arity).map(|index| b_supplied.get(index).copied().unwrap_or_else(|| fill(index))));

    let mut merged: ScratchVec<'scratch, Type<'arena>, S> = builder.scratch_vec_with(arity);
    for (index, (&a_arg, &b_arg)) in a_args.iter().zip(b_args.iter()).enumerate() {
        let variance =
            world.template_parameter_at(a.name.id, index).map_or(Variance::Invariant, |parameter| parameter.variance);
        let arg = match variance {
            Variance::Covariant => crate::ty::meet::compute(a_arg, b_arg, world, options, report, builder),
            Variance::Invariant => {
                let a_refines_b = crate::ty::lattice::refines(a_arg, b_arg, world, options, report, builder);
                let b_refines_a = crate::ty::lattice::refines(b_arg, a_arg, world, options, report, builder);
                if !a_refines_b || !b_refines_a {
                    return None;
                }

                a_arg
            }
            Variance::Contravariant => {
                let mut atoms: ScratchVec<'scratch, Atom<'arena>, S> = builder.scratch_vec_from_slice(a_arg.atoms);
                atoms.extend_from_slice(b_arg.atoms);

                builder.union_of(&atoms)
            }
        };

        if matches!(variance, Variance::Covariant) && arg.is_never() {
            return None;
        }

        merged.push(arg);
    }

    Some(Some(builder.types(&merged)))
}
