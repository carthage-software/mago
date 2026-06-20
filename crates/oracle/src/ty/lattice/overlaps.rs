//! Overlap relation: `overlaps(a, b)` is `true` iff there exists a
//! runtime value `v` such that `v ∈ a ∩ b`.
//!
//! Symmetric: `overlaps(a, b) == overlaps(b, a)`. Distinct from
//! `refines`: `int<0,10>` and `int<5,15>` overlap (value 7 inhabits both)
//! without either refining the other. The type-returning meet (greatest
//! lower bound) lives in `crate::ty::meet`.
//!
//! Strategy: distribute over union (any atom pair on the two sides
//! that overlaps proves the whole types overlap), then for each atom
//! pair fall through these rules in order:
//!
//! 1. Reflexivity / Top / Bot axioms.
//! 2. Generic-parameter projection: `T` overlaps `X` iff `T`'s constraint
//!    overlaps `X`.
//! 3. Subsumption: `a <: b` or `b <: a` implies overlap.
//! 4. Family-specific positive overlap rules (e.g. range overlap, the
//!    string/class-like-string crossing, narrowed-mixed conservatism).
//!
//! When none of those fire we report disjoint. The rule set is incomplete
//! by design: adding a positive rule never weakens correctness, since the
//! relation is monotone in true outcomes; missing rules only cost
//! precision (a downstream narrowing returns `never` instead of a real
//! overlap).

use mago_allocator::Arena;
use mago_allocator::vec::Vec as ScratchVec;
use mago_flags::U8Flags;

use crate::path::Path;
use crate::symbol::SymbolTable;
use crate::symbol::class_like::ClassLikeKind;
use crate::symbol::part::generic::Variance;
use crate::ty::Type;
use crate::ty::atom::Atom;
use crate::ty::atom::kind::AtomKind;
use crate::ty::atom::payload::array::ArrayAtom;
use crate::ty::atom::payload::array::ArrayFlag;
use crate::ty::atom::payload::array::ListAtom;
use crate::ty::atom::payload::array::ListFlag;
use crate::ty::atom::payload::callable::CallableAtom;
use crate::ty::atom::payload::generic_parameter::DefiningEntity;
use crate::ty::atom::payload::generic_parameter::GenericParameterAtom;
use crate::ty::atom::payload::object::named::ObjectAtom;
use crate::ty::atom::payload::scalar::int::IntAtom;
use crate::ty::atom::payload::scalar::mixed::MixedAtom;
use crate::ty::atom::payload::scalar::mixed::Truthiness;
use crate::ty::atom::payload::scalar::string::StringCasing;
use crate::ty::atom::payload::scalar::string::StringLiteral;
use crate::ty::atom::payload::scalar::string::StringRefinementFlag;
use crate::ty::builder::TypeBuilder;
use crate::ty::lattice;
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;
use crate::ty::lattice::atom_admits_empty_container;
use crate::ty::lattice::atom_is_empty_container;
use crate::ty::lattice::atom_refines;
use crate::ty::lattice::family::mixed as mixed_family;
use crate::ty::well_known::INT;
use crate::ty::well_known::MIXED;
use crate::ty::well_known::NEVER;
use crate::ty::well_known::PLACEHOLDER;
use crate::ty::well_known::STRING;
use crate::ty::well_known::TYPE_ARRAY_KEY;
use crate::ty::well_known::TYPE_INT;
use crate::ty::well_known::TYPE_MIXED;

#[inline]
pub fn overlaps<'arena, S, A>(
    a: Type<'arena>,
    b: Type<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    a.atoms
        .iter()
        .any(|a_atom| b.atoms.iter().any(|b_atom| atom_overlaps(*a_atom, *b_atom, symbols, options, report, builder)))
}

/// Pairwise overlap over single atoms, applying the module-level rule
/// order.
///
/// Negations are answered through the subtraction: `!T` overlaps `X` iff
/// `X \ T ≢ ⊥` (some `X` value is outside `T`); the subtract side rejects
/// when `X <: T` and otherwise produces the surviving pieces. `!T ∩ !U`
/// is answered optimistically `true`: it is non-empty iff `T ∪ U ≢ mixed`,
/// which cannot be decided without exhaustive `mixed` enumeration.
///
/// `iterable` pairs always overlap: the empty iterator (`[]`, the empty
/// generator, …) inhabits `iterable<K, V>` for every `K`, `V`.
#[inline]
fn atom_overlaps<'arena, S, A>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    if a == NEVER || b == NEVER {
        return false;
    }

    if is_uninhabited(a, symbols, builder) || is_uninhabited(b, symbols, builder) {
        return false;
    }

    if a == b {
        return true;
    }

    if a == MIXED || b == MIXED || a == PLACEHOLDER || b == PLACEHOLDER {
        return true;
    }

    if let (Atom::GenericParameter(a_payload), Atom::GenericParameter(b_payload)) = (a, b) {
        let same = a_payload.name == b_payload.name && a_payload.defining_entity == b_payload.defining_entity;
        if same || generic_parameters_forward(a_payload, b_payload, symbols) {
            return overlaps(a_payload.constraint, b_payload.constraint, symbols, options, report, builder);
        }

        return false;
    }

    if let Atom::GenericParameter(payload) = a {
        let other = builder.union_of(&[b]);
        return overlaps(payload.constraint, other, symbols, options, report, builder);
    }

    if let Atom::GenericParameter(payload) = b {
        let other = builder.union_of(&[a]);
        return overlaps(payload.constraint, other, symbols, options, report, builder);
    }

    match (a, b) {
        (Atom::Negated(_), Atom::Negated(_)) => return true,
        (Atom::Negated(inner), other) | (other, Atom::Negated(inner)) => {
            let other_type = builder.union_of(&[other]);
            let surviving = crate::ty::subtract::compute(other_type, *inner, symbols, options, report, builder);
            return !surviving.is_never();
        }
        _ => {}
    }

    if let Atom::Intersected(payload) = a {
        if !atom_overlaps(*payload.head, b, symbols, options, report, builder) {
            return false;
        }

        for &conjunct in payload.conjuncts {
            if !atom_overlaps(conjunct, b, symbols, options, report, builder) {
                return false;
            }
        }

        return true;
    }

    if let Atom::Intersected(payload) = b {
        if !atom_overlaps(a, *payload.head, symbols, options, report, builder) {
            return false;
        }

        for &conjunct in payload.conjuncts {
            if !atom_overlaps(a, conjunct, symbols, options, report, builder) {
                return false;
            }
        }

        return true;
    }

    if (atom_is_empty_container(a, symbols, builder) && atom_admits_empty_container(b))
        || (atom_is_empty_container(b, symbols, builder) && atom_admits_empty_container(a))
    {
        return true;
    }

    if a.kind() == AtomKind::Object && b.kind() == AtomKind::Object {
        return object_overlap(a, b, symbols, options, report, builder);
    }

    if a.kind() == AtomKind::String && b.kind() == AtomKind::String {
        return string_overlap(a, b, symbols, options, report, builder);
    }

    if a.kind() == AtomKind::List && b.kind() == AtomKind::List {
        return list_overlap(a, b, symbols, options, report, builder);
    }

    if a.kind() == AtomKind::Array && b.kind() == AtomKind::Array {
        return array_overlap(a, b, symbols, options, report, builder);
    }

    if (a.kind() == AtomKind::List && b.kind() == AtomKind::Array)
        || (a.kind() == AtomKind::Array && b.kind() == AtomKind::List)
    {
        return list_array_overlap(a, b, symbols, options, report, builder);
    }

    if a.kind() == AtomKind::Callable && b.kind() == AtomKind::Callable {
        return callable_overlap(a, b);
    }

    if a.kind() == AtomKind::Iterable && b.kind() == AtomKind::Iterable {
        return true;
    }

    if (a.kind() == AtomKind::Iterable && b.kind() == AtomKind::Array)
        || (a.kind() == AtomKind::Array && b.kind() == AtomKind::Iterable)
    {
        return iterable_array_overlap(a, b, symbols, options, report, builder);
    }

    if (a.kind() == AtomKind::Iterable && b.kind() == AtomKind::List)
        || (a.kind() == AtomKind::List && b.kind() == AtomKind::Iterable)
    {
        return iterable_list_overlap(a, b, symbols, options, report, builder);
    }

    if matches!(
        (a.kind(), b.kind()),
        (AtomKind::HasMethod, AtomKind::HasMethod)
            | (AtomKind::HasProperty, AtomKind::HasProperty)
            | (AtomKind::HasMethod, AtomKind::HasProperty)
            | (AtomKind::HasProperty, AtomKind::HasMethod)
            | (AtomKind::ObjectShape, AtomKind::HasMethod)
            | (AtomKind::ObjectShape, AtomKind::HasProperty)
            | (AtomKind::HasMethod, AtomKind::ObjectShape)
            | (AtomKind::HasProperty, AtomKind::ObjectShape)
    ) {
        return true;
    }

    let (object_atom, structural_atom) = match (a.kind(), b.kind()) {
        (AtomKind::Object, AtomKind::HasMethod | AtomKind::HasProperty | AtomKind::ObjectShape) => (Some(a), Some(b)),
        (AtomKind::HasMethod | AtomKind::HasProperty | AtomKind::ObjectShape, AtomKind::Object) => (Some(b), Some(a)),
        _ => (None, None),
    };

    if let (Some(object), Some(structural)) = (object_atom, structural_atom) {
        return object_structural_overlap(object, structural, symbols);
    }

    if atom_refines(a, b, symbols, options, report, builder) || atom_refines(b, a, symbols, options, report, builder) {
        return true;
    }

    family_overlap(a, b)
}

/// `true` iff the two generic parameters are the same variable through
/// inheritance forwarding in *either* direction - then one is a subtype of
/// the other, so they share the subtype's values and overlap.
#[inline]
fn generic_parameters_forward<'arena, A>(
    a: &GenericParameterAtom<'arena>,
    b: &GenericParameterAtom<'arena>,
    symbols: &SymbolTable<'arena, A>,
) -> bool
where
    A: Arena,
{
    let (DefiningEntity::ClassLike(a_class), DefiningEntity::ClassLike(b_class)) =
        (a.defining_entity, b.defining_entity)
    else {
        return false;
    };

    symbols.template_parameter_forwards_to(a_class.id, a.name, b_class.id, b.name)
        || symbols.template_parameter_forwards_to(b_class.id, b.name, a_class.id, a.name)
}

#[inline]
fn object_structural_overlap<'arena, A>(
    object: Atom<'arena>,
    structural: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
) -> bool
where
    A: Arena,
{
    let Atom::Object(payload) = object else {
        return false;
    };

    let class = payload.name;
    !symbols.is_final(class.id) || class_satisfies_structural(class, structural, symbols)
}

#[inline]
fn class_satisfies_structural<'arena, A>(
    class: Path<'arena>,
    structural: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
) -> bool
where
    A: Arena,
{
    match structural {
        Atom::HasMethod(has_method) => symbols.class_has_method(class.id, has_method.method_name),
        Atom::HasProperty(has_property) => symbols.class_has_property(class.id, has_property.property_name),
        _ => true,
    }
}

/// Object × Object overlap. Two named classes share values when:
///
/// - They are the same class with type-arguments compatible under each
///   parameter's variance (invariant slots must value-equal, covariant
///   slots must overlap). Arguments are normalized first: arity-0 ignores
///   any explicit arguments, arity > 0 truncates over-supply and
///   default-fills under-supply, and a side carrying no `type_arguments`
///   denotes "any T" (the per-position check is skipped).
/// - One descends from the other (the descendant subset overlaps the
///   ancestor). The descendant's view of the ancestor's arguments must be
///   compatible under the ancestor's variance: an invariant argument
///   mismatch (e.g. `A<int(0)>` extending `B<T>` met with `B<int>`) makes
///   the intersection uninhabited, and overlap must reflect that or the
///   downstream meet (which performs the same check) would disagree.
///
/// Otherwise, in PHP's single-inheritance model, two unrelated nominal
/// classes cannot share a runtime instance, so we return `false`. This
/// is conservative: a future symbol-table surface for shared interfaces /
/// traits can lift the answer to `true`.
#[inline]
fn object_overlap<'scratch, 'arena, S, A>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let (Atom::Object(a_payload), Atom::Object(b_payload)) = (a, b) else {
        return false;
    };

    let combined = [a_payload.name, b_payload.name];
    if intersection_uninhabited_under_finality(&combined, symbols) {
        return false;
    }

    if intersection_has_unrelated_concrete_classes(&combined, symbols) {
        return false;
    }

    if a_payload.name != b_payload.name
        && let (Some(a_parent), Some(b_parent)) =
            (symbols.sealed_parent_of(a_payload.name.id), symbols.sealed_parent_of(b_payload.name.id))
        && a_parent == b_parent
        && !symbols.descends_from(a_payload.name.id, b_payload.name.id)
        && !symbols.descends_from(b_payload.name.id, a_payload.name.id)
    {
        return false;
    }

    if a_payload.name == b_payload.name {
        let arity = symbols.template_parameter_arity(a_payload.name.id);
        let one_side_bare = a_payload.type_arguments.is_none() || b_payload.type_arguments.is_none();
        let skip_for_coercion = options.template_default_coercion && one_side_bare;
        if arity > 0 && !skip_for_coercion {
            let a_supplied: &[Type<'arena>] = a_payload.type_arguments.unwrap_or_default();
            let b_supplied: &[Type<'arena>] = b_payload.type_arguments.unwrap_or_default();
            let fill = |index: usize| -> Type<'arena> {
                symbols
                    .template_parameter_at(a_payload.name.id, index)
                    .map(|parameter| parameter.constraint)
                    .unwrap_or(TYPE_MIXED)
            };

            for index in 0..arity {
                let a_argument = a_supplied.get(index).copied().unwrap_or_else(|| fill(index));
                let b_argument = b_supplied.get(index).copied().unwrap_or_else(|| fill(index));
                let variance = symbols
                    .template_parameter_at(a_payload.name.id, index)
                    .map_or(Variance::Invariant, |parameter| parameter.variance);
                match variance {
                    Variance::Invariant => {
                        let a_refines_b = lattice::refines(a_argument, b_argument, symbols, options, report, builder);
                        let b_refines_a = lattice::refines(b_argument, a_argument, symbols, options, report, builder);
                        if !a_refines_b || !b_refines_a {
                            return false;
                        }
                    }
                    Variance::Covariant => {
                        if !overlaps(a_argument, b_argument, symbols, options, report, builder) {
                            return false;
                        }
                    }
                    Variance::Contravariant => {}
                }
            }
        }
    }

    if a_payload.name != b_payload.name {
        let (descendant, ancestor) = if symbols.descends_from(a_payload.name.id, b_payload.name.id) {
            (a_payload, b_payload)
        } else if symbols.descends_from(b_payload.name.id, a_payload.name.id) {
            (b_payload, a_payload)
        } else {
            return true;
        };

        if !descendant_args_satisfy_ancestor(descendant, ancestor, symbols, options, report, builder) {
            return false;
        }
    }

    true
}

#[inline]
fn descendant_args_satisfy_ancestor<'arena, S, A>(
    descendant: &ObjectAtom<'arena>,
    ancestor: &ObjectAtom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let arity = symbols.template_parameter_arity(ancestor.name.id);
    if arity == 0 {
        return true;
    }

    let Some(ancestor_arguments) = ancestor.type_arguments else {
        return true;
    };

    if ancestor_arguments.len() != arity {
        return true;
    }

    let descendant_actuals: &[Type<'arena>] = descendant.type_arguments.unwrap_or_default();

    for (position, &ancestor_argument) in ancestor_arguments.iter().enumerate() {
        let Some(inherited) = crate::ty::lattice::family::object::resolve_inherited_argument(
            descendant.name,
            ancestor.name,
            position,
            symbols,
            builder,
            16,
        ) else {
            return true;
        };

        let resolved = crate::ty::template::substitute(
            inherited,
            &|parameter: &GenericParameterAtom<'arena>| -> Option<Type<'arena>> {
                if parameter.defining_entity != DefiningEntity::ClassLike(descendant.name) {
                    return None;
                }

                let actual_position = symbols.template_parameter_index(descendant.name.id, parameter.name)?;
                descendant_actuals.get(actual_position).copied()
            },
            builder,
        );
        let variance = symbols
            .template_parameter_at(ancestor.name.id, position)
            .map(|parameter| parameter.variance)
            .unwrap_or_default();
        let compatible = match variance {
            Variance::Invariant => {
                lattice::refines(resolved, ancestor_argument, symbols, options, report, builder)
                    && lattice::refines(ancestor_argument, resolved, symbols, options, report, builder)
            }
            Variance::Covariant => lattice::refines(resolved, ancestor_argument, symbols, options, report, builder),
            Variance::Contravariant => lattice::refines(ancestor_argument, resolved, symbols, options, report, builder),
        };
        if !compatible {
            return false;
        }
    }

    true
}

/// `true` iff `Foo & Bar & …` is provably uninhabited via the
/// symbol table's finality surface. A `final` class admits no subclass,
/// so for `F & O` to be inhabited `F` and `O` must be ancestor-
/// related; an unrelated `O` alongside a final `F` collapses the
/// intersection. Without a final witness we stay permissive
/// (return `false`).
#[inline]
fn intersection_uninhabited_under_finality<'arena, A>(classes: &[Path<'arena>], symbols: &SymbolTable<'arena, A>) -> bool
where
    A: Arena,
{
    classes.iter().any(|&final_candidate| {
        if !symbols.is_final(final_candidate.id) {
            return false;
        }

        classes.iter().any(|&other| {
            other != final_candidate
                && !symbols.descends_from(final_candidate.id, other.id)
                && !symbols.descends_from(other.id, final_candidate.id)
        })
    })
}

/// `true` iff two names in `classes` are unrelated concrete classes -
/// both `class_like_kind == Class`, neither descending from the other.
///
/// PHP classes are single-inheritance: a class extends exactly one class, so
/// any object's class-ancestry is a single chain and no object can descend
/// from two unrelated classes - their intersection is therefore empty.
/// Interfaces and traits are deliberately excluded (a class may implement /
/// use many, so an unrelated interface can still be bridged by a common
/// implementor), as is any name whose kind the symbol table does not know. Shared by
/// `object_overlap`, `object_uninhabited`, and meet's intersection
/// composition so overlap and meet always agree.
///
/// Soundness rests on the [`SymbolTable`] single-inheritance contract: for two
/// `Class`-kind names, `descends_from` forms a forest (no class has two
/// unrelated class ancestors).
#[inline]
fn intersection_has_unrelated_concrete_classes<'arena, A>(
    classes: &[Path<'arena>],
    symbols: &SymbolTable<'arena, A>,
) -> bool
where
    A: Arena,
{
    for (index, &left) in classes.iter().enumerate() {
        if symbols.class_like_kind(left.id) != Some(ClassLikeKind::Class) {
            continue;
        }

        for &right in &classes[index + 1..] {
            if symbols.class_like_kind(right.id) != Some(ClassLikeKind::Class) {
                continue;
            }

            if left != right && !symbols.descends_from(left.id, right.id) && !symbols.descends_from(right.id, left.id) {
                return true;
            }
        }
    }

    false
}

/// `true` for atoms that are structurally non-`never` but whose value
/// set is empty: `non-empty-list<never>`, `non-empty-array<…, never>`,
/// `Foo<never>` with a non-contravariant template, and any container
/// nested over a value-never type (e.g. `non-empty-list<B<never>>`).
/// The lattice can construct these but no runtime value inhabits
/// them, so `overlap` treats them as bottom.
#[inline]
fn list_uninhabited<'arena, S, A>(
    payload: &ListAtom<'arena>,
    intersections: Option<&[Atom<'arena>]>,
    symbols: &SymbolTable<'arena, A>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    if payload.flags.contains(ListFlag::NonEmpty)
        && payload.known_elements.is_none()
        && type_is_value_never(payload.element_type, symbols, builder)
    {
        return true;
    }

    if let Some(entries) = payload.known_elements {
        for entry in entries {
            if !entry.optional && type_is_value_never(entry.value, symbols, builder) {
                return true;
            }
        }
    }

    let stripped = builder.list(*payload);
    list_array_intersections_uninhabited_components(stripped, intersections, symbols, builder)
}

#[inline]
fn array_uninhabited<'arena, S, A>(
    payload: &ArrayAtom<'arena>,
    intersections: Option<&[Atom<'arena>]>,
    symbols: &SymbolTable<'arena, A>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    if payload.flags.contains(ArrayFlag::NonEmpty) {
        if let Some(key_type) = payload.key_param {
            let int_or_string = builder.union_of(&[INT, STRING]);
            if !overlaps(key_type, int_or_string, symbols, LatticeOptions::default(), &mut LatticeReport::new(), builder)
            {
                return true;
            }
        }

        let key_empty = payload.key_param.is_some_and(|ty| type_is_value_never(ty, symbols, builder));
        let value_empty = payload.value_param.is_some_and(|ty| type_is_value_never(ty, symbols, builder));
        if key_empty || value_empty {
            return true;
        }
    }

    if let Some(entries) = payload.known_items {
        for entry in entries {
            if !entry.optional && type_is_value_never(entry.value, symbols, builder) {
                return true;
            }
        }
    }

    let stripped = builder.array(*payload);
    list_array_intersections_uninhabited_components(stripped, intersections, symbols, builder)
}

#[inline]
fn object_uninhabited<'scratch, 'arena, S, A>(
    payload: &ObjectAtom<'arena>,
    intersections: Option<&[Atom<'arena>]>,
    symbols: &SymbolTable<'arena, A>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    if let Some(conjunct_list) = intersections {
        let mut classes: ScratchVec<'scratch, Path<'arena>, S> = builder.scratch_vec_from_slice(&[payload.name]);
        let mut structurals: ScratchVec<'scratch, Atom<'arena>, S> = builder.scratch_vec();
        let mut negations: ScratchVec<'scratch, Atom<'arena>, S> = builder.scratch_vec();
        for &conjunct in conjunct_list {
            match conjunct {
                Atom::Object(conjunct_payload) => classes.push(conjunct_payload.name),
                Atom::HasMethod(_) | Atom::HasProperty(_) => structurals.push(conjunct),
                Atom::Negated(_) => negations.push(conjunct),
                _ => {}
            }
        }

        if intersection_uninhabited_under_finality(&classes, symbols) {
            return true;
        }

        if intersection_has_unrelated_concrete_classes(&classes, symbols) {
            return true;
        }

        if sealed_siblings_disjoint(&classes, symbols) {
            return true;
        }

        for &negation in &negations {
            let Atom::Negated(inner) = negation else {
                continue;
            };

            for &class in &classes {
                let bare = builder.object(ObjectAtom { name: class, type_arguments: None, flags: U8Flags::empty() });
                let bare_type = builder.union_of(&[bare]);
                if lattice::refines(
                    bare_type,
                    *inner,
                    symbols,
                    LatticeOptions::default(),
                    &mut LatticeReport::new(),
                    builder,
                ) {
                    return true;
                }
            }
        }

        for &class in &classes {
            if !symbols.is_final(class.id) {
                continue;
            }

            for &structural in &structurals {
                let satisfied = match structural {
                    Atom::HasMethod(has_method) => symbols.class_has_method(class.id, has_method.method_name),
                    Atom::HasProperty(has_property) => symbols.class_has_property(class.id, has_property.property_name),
                    _ => true,
                };

                if !satisfied {
                    return true;
                }
            }
        }
    }

    let Some(type_arguments) = payload.type_arguments else {
        return false;
    };

    if symbols.sealed_direct_inheritors(payload.name.id).is_some() {
        let head = builder.object(ObjectAtom {
            name: payload.name,
            type_arguments: Some(type_arguments),
            flags: payload.flags,
        });
        let residual = lattice::sealed::compute_residual(
            head,
            &[],
            symbols,
            LatticeOptions::default(),
            &mut LatticeReport::new(),
            builder,
        );
        if matches!(residual, lattice::sealed::SealedResidual::FullyCovered) {
            return true;
        }
    }

    type_arguments.iter().enumerate().any(|(index, &argument)| {
        if !type_is_value_never(argument, symbols, builder) {
            return false;
        }

        let variance = symbols
            .template_parameter_at(payload.name.id, index)
            .map_or(Variance::Contravariant, |parameter| parameter.variance);
        !matches!(variance, Variance::Contravariant)
    })
}

#[inline]
pub fn is_uninhabited<'arena, S, A>(
    atom: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    match atom {
        Atom::List(payload) => list_uninhabited(payload, None, symbols, builder),
        Atom::Array(payload) => array_uninhabited(payload, None, symbols, builder),
        Atom::Object(payload) => object_uninhabited(payload, None, symbols, builder),
        Atom::Intersected(payload) => {
            if matches!(*payload.head, Atom::Object(_))
                && sealed_cover_fully_excluded(*payload.head, payload.conjuncts, symbols, builder)
            {
                return true;
            }

            if intersected_negated_contradiction(*payload.head, payload.conjuncts, symbols, builder) {
                return true;
            }

            match *payload.head {
                Atom::Object(head_payload) => {
                    return object_uninhabited(head_payload, Some(payload.conjuncts), symbols, builder);
                }
                Atom::List(head_payload) => {
                    return list_uninhabited(head_payload, Some(payload.conjuncts), symbols, builder);
                }
                Atom::Array(head_payload) => {
                    return array_uninhabited(head_payload, Some(payload.conjuncts), symbols, builder);
                }
                _ => {}
            }

            if is_uninhabited(*payload.head, symbols, builder) {
                return true;
            }

            for &conjunct in payload.conjuncts {
                if is_uninhabited(conjunct, symbols, builder) {
                    return true;
                }
            }

            false
        }
        _ => false,
    }
}

/// `true` when every atom in `ty` is uninhabited or `ty` is the
/// canonical `never`. Used by [`is_uninhabited`] to recurse into
/// container element types.
#[inline]
pub(crate) fn type_is_value_never<'arena, S, A>(
    ty: Type<'arena>,
    symbols: &SymbolTable<'arena, A>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    if ty.is_never() {
        return true;
    }

    ty.atoms.iter().all(|atom| *atom == NEVER || is_uninhabited(*atom, symbols, builder))
}

/// `true` iff the intersection of `head` with `conjuncts` refines the
/// inner of any Negated conjunct, making `Intersected(H, C1, …, !T)`
/// uninhabited.
#[inline]
fn intersected_negated_contradiction<'scratch, 'arena, S, A>(
    head: Atom<'arena>,
    conjuncts: &[Atom<'arena>],
    symbols: &SymbolTable<'arena, A>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let mut non_negated: ScratchVec<'scratch, Atom<'arena>, S> = builder.scratch_vec_with(conjuncts.len());
    for &conjunct in conjuncts {
        if !matches!(conjunct, Atom::Negated(_)) {
            non_negated.push(conjunct);
        }
    }

    let positive_atom = if non_negated.is_empty() { head } else { builder.intersected(head, &non_negated) };
    let positive_type = builder.union_of(&[positive_atom]);

    for &conjunct in conjuncts {
        let Atom::Negated(inner) = conjunct else {
            continue;
        };

        if lattice::refines(positive_type, *inner, symbols, LatticeOptions::default(), &mut LatticeReport::new(), builder)
        {
            return true;
        }
    }

    false
}

/// `true` iff `Intersected(H, conjuncts)` has a sealed head `H`
/// and every direct inheritor of `H` is covered by some Negated
/// conjunct, making the Intersected uninhabited.
#[inline]
fn sealed_cover_fully_excluded<'scratch, 'arena, S, A>(
    head: Atom<'arena>,
    conjuncts: &[Atom<'arena>],
    symbols: &SymbolTable<'arena, A>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let mut negated_inners: ScratchVec<'scratch, Type<'arena>, S> = builder.scratch_vec_with(conjuncts.len());
    for &conjunct in conjuncts {
        if let Atom::Negated(inner) = conjunct {
            negated_inners.push(*inner);
        }
    }

    if negated_inners.is_empty() {
        return false;
    }

    matches!(
        crate::ty::lattice::sealed::compute_residual(
            head,
            &negated_inners,
            symbols,
            LatticeOptions::default(),
            &mut LatticeReport::new(),
            builder,
        ),
        crate::ty::lattice::sealed::SealedResidual::FullyCovered
    )
}

/// `Callable × Callable` overlap. A function value has a fixed
/// arity at runtime, so two callable types with different parameter
/// counts cannot share any value. Same-arity (or one side `Any`)
/// callables share at least the always-throwing function (`return
/// never`), which trivially satisfies any return type.
#[inline]
fn callable_overlap(a: Atom<'_>, b: Atom<'_>) -> bool {
    let (Atom::Callable(CallableAtom::Signature(a_signature)), Atom::Callable(CallableAtom::Signature(b_signature))) =
        (a, b)
    else {
        return true;
    };

    let a_arity = a_signature.parameters.map_or(0, |parameters| parameters.len());
    let b_arity = b_signature.parameters.map_or(0, |parameters| parameters.len());
    a_arity == b_arity
}

/// `String × String` overlap: defer to the meet rule. Two refined
/// string axes (`numeric-string`, `lowercase-string`, etc.) admit a
/// non-empty intersection unless their literal/casing/flags are
/// jointly unsatisfiable, which `string_meet` already decides.
#[inline]
fn string_overlap<'arena, S, A>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let _ = (symbols, options, report);
    crate::ty::meet::family::string::string_meet(a, b, builder).is_some()
}

/// `true` iff `stripped` (a bare List or Array atom) carries an
/// intersection chain containing a `Negated(T)` whose inner `T` already
/// contains the stripped head; in that case the whole intersection is
/// empty. Mirrors the negated-class arm of [`is_uninhabited`] for
/// objects.
#[inline]
fn list_array_intersections_uninhabited_components<'arena, S, A>(
    stripped: Atom<'arena>,
    intersections: Option<&[Atom<'arena>]>,
    symbols: &SymbolTable<'arena, A>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let Some(conjuncts) = intersections else {
        return false;
    };

    let stripped_type = builder.union_of(&[stripped]);
    let options = LatticeOptions::default();
    for &conjunct in conjuncts {
        if let Atom::Negated(inner) = conjunct {
            let mut report = LatticeReport::new();
            if lattice::refines(stripped_type, *inner, symbols, options, &mut report, builder) {
                return true;
            }
        }
    }

    false
}

/// `list<X> ∩ list<Y>` shares the empty list `[]` only when neither
/// side requires non-empty. When at least one side requires non-empty,
/// the element types must overlap for any concrete value to inhabit
/// both sets.
#[inline]
fn list_overlap<'arena, S, A>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let (Atom::List(a_payload), Atom::List(b_payload)) = (a, b) else {
        return false;
    };

    if !a_payload.flags.contains(ListFlag::NonEmpty) && !b_payload.flags.contains(ListFlag::NonEmpty) {
        return true;
    }

    overlaps(a_payload.element_type, b_payload.element_type, symbols, options, report, builder)
}

/// `iterable<K,V> ∩ array<K',V'>` shares the empty array unless the
/// array is non-empty; otherwise the iterable's K must admit some
/// of the array's keys and V must admit some of the array's values.
#[inline]
fn iterable_array_overlap<'arena, S, A>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let (iterable_atom, array_atom) = if a.kind() == AtomKind::Iterable { (a, b) } else { (b, a) };
    let Atom::Iterable(iterable_payload) = iterable_atom else {
        return false;
    };
    let Atom::Array(array_payload) = array_atom else {
        return false;
    };

    if !array_payload.flags.contains(ArrayFlag::NonEmpty) {
        return true;
    }

    let array_key = array_payload.key_param.unwrap_or(TYPE_ARRAY_KEY);
    let array_value = array_payload.value_param.unwrap_or(TYPE_MIXED);
    overlaps(iterable_payload.key_type, array_key, symbols, options, report, builder)
        && overlaps(iterable_payload.value_type, array_value, symbols, options, report, builder)
}

/// `iterable<K,V> ∩ list<E>`. The empty list `[]` is an empty iterator and
/// inhabits every `iterable<K, V>`, so a possibly-empty list always overlaps.
/// A non-empty list shares a value only when `int` fits `K` (the list's keys
/// are `int`) and `V` overlaps the list element type.
#[inline]
fn iterable_list_overlap<'arena, S, A>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let (iterable_atom, list_atom) = if a.kind() == AtomKind::Iterable { (a, b) } else { (b, a) };
    let Atom::Iterable(iterable_payload) = iterable_atom else {
        return false;
    };
    let Atom::List(list_payload) = list_atom else {
        return false;
    };

    if !list_payload.flags.contains(ListFlag::NonEmpty) {
        return true;
    }

    if !lattice::refines(TYPE_INT, iterable_payload.key_type, symbols, options, report, builder) {
        return false;
    }

    overlaps(iterable_payload.value_type, list_payload.element_type, symbols, options, report, builder)
}

/// `list<E> ∩ array<K, V>` shares the empty list `[]` (which is also
/// the empty array) unless either side demands non-empty. With at
/// least one non-empty side, the array's key constraint must accept
/// `int` (lists are int-keyed) and `E ∩ V` must overlap.
#[inline]
fn list_array_overlap<'arena, S, A>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let (list_atom, array_atom) = if a.kind() == AtomKind::List { (a, b) } else { (b, a) };
    let Atom::List(list_payload) = list_atom else {
        return false;
    };
    let Atom::Array(array_payload) = array_atom else {
        return false;
    };

    if !list_payload.flags.contains(ListFlag::NonEmpty) && !array_payload.flags.contains(ArrayFlag::NonEmpty) {
        return true;
    }

    if let Some(array_key_param) = array_payload.key_param
        && !lattice::refines(TYPE_INT, array_key_param, symbols, options, report, builder)
    {
        return false;
    }

    let array_value = array_payload.value_param.unwrap_or(TYPE_MIXED);
    overlaps(list_payload.element_type, array_value, symbols, options, report, builder)
}

/// `array<K,V> ∩ array<K',V'>` mirrors `list_overlap`: the empty
/// array `[]` is shared only when neither side demands non-empty.
#[inline]
fn array_overlap<'arena, S, A>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let (Atom::Array(a_payload), Atom::Array(b_payload)) = (a, b) else {
        return false;
    };

    if !a_payload.flags.contains(ArrayFlag::NonEmpty) && !b_payload.flags.contains(ArrayFlag::NonEmpty) {
        return true;
    }

    match (a_payload.key_param, b_payload.key_param, a_payload.value_param, b_payload.value_param) {
        (Some(a_key), Some(b_key), Some(a_value), Some(b_value)) => {
            overlaps(a_key, b_key, symbols, options, report, builder)
                && overlaps(a_value, b_value, symbols, options, report, builder)
        }
        _ => true,
    }
}

/// Family-specific positive overlap rules for atom pairs that neither
/// subsumption nor the structural dispatch decided.
///
/// True-union dominator pairs (`scalar`, `numeric`, `array-key`) share
/// at least `int`, so every cross-pair overlaps.
#[inline]
fn family_overlap(a: Atom<'_>, b: Atom<'_>) -> bool {
    if a.kind() == AtomKind::Int && b.kind() == AtomKind::Int {
        return int_overlap(a, b);
    }

    if a.kind() == AtomKind::Mixed || b.kind() == AtomKind::Mixed {
        return mixed_overlap(a, b);
    }

    let pair = (a.kind(), b.kind());
    if matches!(pair, (AtomKind::String, AtomKind::ClassLikeString) | (AtomKind::ClassLikeString, AtomKind::String)) {
        return string_class_like_string_overlap(a, b);
    }

    if matches!(pair, (AtomKind::Numeric, AtomKind::String) | (AtomKind::String, AtomKind::Numeric)) {
        return numeric_string_overlap(a, b);
    }

    if matches!(
        pair,
        (AtomKind::Scalar, AtomKind::Numeric)
            | (AtomKind::Numeric, AtomKind::Scalar)
            | (AtomKind::Scalar, AtomKind::ArrayKey)
            | (AtomKind::ArrayKey, AtomKind::Scalar)
            | (AtomKind::ArrayKey, AtomKind::Numeric)
            | (AtomKind::Numeric, AtomKind::ArrayKey)
    ) {
        return true;
    }

    false
}

/// `numeric` × `string`: numeric strings inhabit both. A specific
/// string literal that isn't itself numeric (e.g. `'foo'`) rules the
/// overlap out: its value is not in `numeric`.
#[inline]
fn numeric_string_overlap(a: Atom<'_>, b: Atom<'_>) -> bool {
    let string_atom = if a.kind() == AtomKind::String { a } else { b };
    let Atom::String(payload) = string_atom else {
        return false;
    };

    match payload.literal {
        StringLiteral::Value(value) => {
            core::str::from_utf8(value).is_ok_and(|text| text.parse::<i64>().is_ok() || text.parse::<f64>().is_ok())
        }
        StringLiteral::None | StringLiteral::Unspecified => true,
    }
}

/// `String` × `ClassLikeString`: they overlap iff some string value
/// inhabits both. A class-like-string is always non-empty and (as a
/// PHP class name) carries no chars outside `[A-Za-z_0-9\\]`. A
/// literal string side rules out the overlap if its value isn't a
/// valid class name; a literal class-string side rules it out if its
/// fixed name conflicts with the string's literal/casing constraints.
#[inline]
fn string_class_like_string_overlap(a: Atom<'_>, b: Atom<'_>) -> bool {
    let string_atom = if a.kind() == AtomKind::String { a } else { b };
    let Atom::String(payload) = string_atom else {
        return false;
    };

    if let StringLiteral::Value(value) = payload.literal {
        return is_valid_class_name(value);
    }

    if payload.flags.contains(StringRefinementFlag::Numeric) || payload.flags.contains(StringRefinementFlag::Callable) {
        return false;
    }

    matches!(payload.casing, StringCasing::Unspecified)
}

#[inline]
fn is_valid_class_name(bytes: &[u8]) -> bool {
    let len = bytes.len();
    if len == 0 || bytes[len - 1] == b'\\' {
        return false;
    }

    let mut index = usize::from(bytes[0] == b'\\');
    if index >= len {
        return false;
    }

    let mut part_start = true;
    while index < len {
        let byte = bytes[index];
        if byte == b'\\' {
            if part_start {
                return false;
            }

            part_start = true;
        } else if part_start {
            if !(byte.is_ascii_alphabetic() || byte == b'_') {
                return false;
            }

            part_start = false;
        } else if !(byte.is_ascii_alphanumeric() || byte == b'_' || byte >= 0x80) {
            return false;
        }

        index += 1;
    }

    !part_start
}

/// Narrowed-mixed overlap: each side's axis flags must be jointly
/// satisfiable by some runtime value the other side admits. Vanilla
/// `mixed` is already absorbed by the Top axiom, so at least one side
/// here carries a non-trivial axis.
#[inline]
fn mixed_overlap(a: Atom<'_>, b: Atom<'_>) -> bool {
    let (mixed, other) = if a.kind() == AtomKind::Mixed { (a, b) } else { (b, a) };
    let Atom::Mixed(mixed_payload) = mixed else {
        return true;
    };

    if !mixed_axes_compatible(mixed_payload, other) {
        return false;
    }

    if let Atom::Mixed(other_payload) = other
        && !mixed_axes_compatible(other_payload, mixed)
    {
        return false;
    }

    true
}

/// `true` iff two distinct names in `names` share the same sealed
/// parent: distinct direct inheritors of one sealed class are disjoint.
#[inline]
fn sealed_siblings_disjoint<'arena, A>(names: &[Path<'arena>], symbols: &SymbolTable<'arena, A>) -> bool
where
    A: Arena,
{
    if names.len() < 2 {
        return false;
    }

    for first_index in 0..names.len() {
        for second_index in first_index + 1..names.len() {
            if names[first_index] == names[second_index] {
                continue;
            }

            if let (Some(first_parent), Some(second_parent)) =
                (symbols.sealed_parent_of(names[first_index].id), symbols.sealed_parent_of(names[second_index].id))
                && first_parent == second_parent
            {
                return true;
            }
        }
    }

    false
}

#[inline]
fn mixed_axes_compatible(payload: MixedAtom, other: Atom<'_>) -> bool {
    if payload.is_non_null() && !mixed_family::is_non_null(other) {
        return false;
    }

    let other_truthiness = mixed_family::truthiness_of(other);
    match payload.truthiness() {
        Truthiness::Truthy if other_truthiness == Truthiness::Falsy => return false,
        Truthiness::Falsy if other_truthiness == Truthiness::Truthy => return false,
        _ => {}
    }

    if payload.is_empty() && other_truthiness == Truthiness::Truthy {
        return false;
    }

    true
}

/// Intervals (with absorption: `int` and `literal-int` are unbounded) on
/// either side overlap iff `max(lo_a, lo_b) ≤ min(hi_a, hi_b)`. An open
/// bound on either side is treated as `±∞`.
#[inline]
fn int_overlap(a: Atom<'_>, b: Atom<'_>) -> bool {
    let (Atom::Int(a_payload), Atom::Int(b_payload)) = (a, b) else {
        return false;
    };

    let (a_lower, a_upper) = int_bounds(a_payload);
    let (b_lower, b_upper) = int_bounds(b_payload);

    let lower = match (a_lower, b_lower) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    };
    let upper = match (a_upper, b_upper) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    };

    match (lower, upper) {
        (Some(lower_bound), Some(upper_bound)) => lower_bound <= upper_bound,
        _ => true,
    }
}

#[inline]
fn int_bounds(payload: IntAtom<'_>) -> (Option<i64>, Option<i64>) {
    match payload {
        IntAtom::Unspecified | IntAtom::UnspecifiedLiteral => (None, None),
        IntAtom::Literal(value) => (Some(value), Some(value)),
        IntAtom::Range(range) => (range.lower(), range.upper()),
    }
}
