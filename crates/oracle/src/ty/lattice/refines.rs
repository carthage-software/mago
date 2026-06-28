//! Refinement (subtype) relation: `refines(a, b)` is `true` iff every value
//! of type `a` is also a value of type `b` (i.e. `a <: b`).

use mago_allocator::Arena;
use mago_allocator::vec::Vec as ScratchVec;

use crate::symbol::SymbolTable;
use crate::ty::Type;
use crate::ty::atom::Atom;
use crate::ty::atom::kind::AtomKind;
use crate::ty::atom::payload::array::ArrayAtom;
use crate::ty::atom::payload::array::ArrayFlag;
use crate::ty::atom::payload::array::ListAtom;
use crate::ty::atom::payload::array::ListFlag;
use crate::ty::atom::payload::scalar::int::IntAtom;
use crate::ty::atom::payload::scalar::mixed::MixedAtom;
use crate::ty::atom::payload::scalar::string::StringCasing;
use crate::ty::atom::payload::scalar::string::StringLiteral;
use crate::ty::atom::payload::scalar::string::StringRefinementFlag;
use crate::ty::builder::TypeBuilder;
use crate::ty::lattice;
use crate::ty::lattice::CoercionCause;
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;
use crate::ty::lattice::family;
use crate::ty::lattice::overlaps::type_is_value_never;
use crate::ty::lattice::sealed::SealedResidual;
use crate::ty::well_known;

/// `true` iff `a <: b`: every runtime value of type `a` is also a value of
/// type `b` (i.e. `a` is a refinement / narrowing of `b`).
///
/// Implements the universal axioms (refl / Bot / Top), the union
/// dispatch (Union-L / Union-R), and the structural scalar lattice
/// (bool / int / float / string / class-like-string / resource /
/// array-key / numeric / scalar / object-any). Object hierarchy
/// queries flow through `symbols`; callable variance, array shape
/// rules, mixed-axis refinements, and template machinery layer in
/// family by family; what isn't implemented returns `false`
/// conservatively.
#[inline]
pub fn refines<'arena, S, A>(
    input: Type<'arena>,
    container: Type<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    if input == container && !options.ignore_null && !options.ignore_false {
        return true;
    }

    let input = expand_double_negation(input, builder);
    let container = expand_double_negation(container, builder);

    input
        .atoms
        .iter()
        .filter(|atom| {
            let skipped = (options.ignore_null && **atom == well_known::NULL)
                || (options.ignore_false && **atom == well_known::FALSE);
            !skipped
        })
        .all(|atom| {
            if container.atoms.contains(atom) {
                return true;
            }

            if atom_is_empty_container(*atom, symbols, builder) && union_admits_empty_container(container.atoms) {
                return true;
            }

            if container
                .atoms
                .iter()
                .any(|candidate| atom_refines(*atom, *candidate, symbols, options, report, builder))
            {
                return true;
            }

            if int_union_covers(*atom, container.atoms, builder) {
                return true;
            }

            if string_union_covers(*atom, container.atoms) {
                return true;
            }

            if bool_union_covers(*atom, container.atoms) {
                return true;
            }

            if mixed_union_covers(*atom, container.atoms) {
                return true;
            }

            if list_union_covers(*atom, container.atoms, symbols, options, report, builder) {
                return true;
            }

            if array_union_covers(*atom, container.atoms, symbols, options, report, builder) {
                return true;
            }

            if intersected_partition_covers(*atom, container.atoms, symbols, options, report, builder) {
                return true;
            }

            if negation_partition_covers(*atom, container.atoms, symbols, options, report, builder) {
                return true;
            }

            if generic_parameter_union_covers(*atom, container.atoms, symbols, options, report, builder) {
                return true;
            }

            if sealed_survivors_cover(*atom, container.atoms, symbols, options, report, builder) {
                return true;
            }

            if sealed_intersected_union_cover(*atom, container.atoms, symbols, options, report, builder) {
                return true;
            }

            false
        })
}

/// `T <: X ∪ ¬X` always (the union is `mixed`). Fires when the
/// container has a `Negated(Y)` atom whose inner `Y` is covered by
/// some other container atom, making the union `mixed`.
#[inline]
fn negation_partition_covers<'scratch, 'arena, S, A>(
    _input: Atom<'arena>,
    containers: &[Atom<'arena>],
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    if !containers.iter().any(|atom| matches!(atom, Atom::Negated(_))) {
        return false;
    }

    for &candidate in containers {
        let Atom::Negated(inner) = candidate else {
            continue;
        };

        let mut others: ScratchVec<'scratch, Atom<'arena>, S> = builder.scratch_vec();
        others.extend(containers.iter().copied().filter(|&other| other != candidate));
        if others.is_empty() {
            continue;
        }

        let others_type = builder.union_of(&others);
        if refines(*inner, others_type, symbols, options, report, builder) {
            return true;
        }
    }

    false
}

/// Recognize the partition identity
/// `Intersected(H, [!X1, .., !Xn]) ∪ X1 ∪ .. ∪ Xn ≡ H ∪ X1 ∪ .. ∪ Xn`.
/// When a container atom is `Intersected(H, [!X1, ..])` with all
/// negated inners `Xi` present elsewhere in the container, the
/// container's value-set equals the container with the Intersected
/// replaced by its head. Recurse on that reduced container; the
/// number of Intersected atoms strictly decreases each step.
///
/// A second pattern also reduces: `Intersected(H, [C…, !X]) | Y` where `Y`
/// refines `X` (or `Y` itself is `X`'s head, or the positive side of the
/// Intersected narrowed by `X` refines `Y`). The partition `!X | X` covers
/// everything, so the Intersected is again replaced by its head.
#[inline]
fn intersected_partition_covers<'scratch, 'arena, S, A>(
    input: Atom<'arena>,
    containers: &[Atom<'arena>],
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    if !containers.iter().any(|atom| matches!(atom, Atom::Intersected(_))) {
        return false;
    }

    for (index, &candidate) in containers.iter().enumerate() {
        let Atom::Intersected(payload) = candidate else {
            continue;
        };

        let mut all_negated = true;
        let mut all_inners_present = true;
        for &conjunct in payload.conjuncts {
            let Atom::Negated(negated_inner) = conjunct else {
                all_negated = false;
                break;
            };

            for &inner_atom in negated_inner.atoms {
                if !containers
                    .iter()
                    .enumerate()
                    .any(|(other_index, &other)| other_index != index && other == inner_atom)
                {
                    all_inners_present = false;
                    break;
                }
            }

            if !all_inners_present {
                break;
            }
        }

        if !all_negated || !all_inners_present {
            continue;
        }

        let mut reduced: ScratchVec<'scratch, Atom<'arena>, S> = builder.scratch_vec_from_slice(containers);
        reduced[index] = *payload.head;
        let input_type = builder.union_of(&[input]);
        let reduced_type = builder.union_of(&reduced);
        if refines(input_type, reduced_type, symbols, options, report, builder) {
            return true;
        }
    }

    for (index, &candidate) in containers.iter().enumerate() {
        let Atom::Intersected(payload) = candidate else {
            continue;
        };

        let mut non_negated: ScratchVec<'scratch, Atom<'arena>, S> = builder.scratch_vec();
        for &conjunct in payload.conjuncts {
            if !matches!(conjunct, Atom::Negated(_)) {
                non_negated.push(conjunct);
            }
        }

        let positive_atom =
            if non_negated.is_empty() { *payload.head } else { builder.intersected(*payload.head, &non_negated) };

        for &conjunct in payload.conjuncts {
            let Atom::Negated(inner) = conjunct else {
                continue;
            };

            let positive_type = builder.union_of(&[positive_atom]);
            let positive_meet_inner =
                crate::ty::meet::compute(positive_type, *inner, symbols, options, report, builder);

            let has_matching = containers.iter().enumerate().any(|(other_index, &other)| {
                if other_index == index {
                    return false;
                }

                if inner.atoms == [other] {
                    return true;
                }

                if let Atom::Intersected(other_payload) = other
                    && inner.atoms.first() == Some(other_payload.head)
                {
                    return true;
                }

                let other_type = builder.union_of(&[other]);
                refines(positive_meet_inner, other_type, symbols, options, report, builder)
            });

            if has_matching {
                let mut reduced: ScratchVec<'scratch, Atom<'arena>, S> = builder.scratch_vec_from_slice(containers);
                reduced[index] = *payload.head;
                let input_type = builder.union_of(&[input]);
                let reduced_type = builder.union_of(&reduced);
                if refines(input_type, reduced_type, symbols, options, report, builder) {
                    return true;
                }
            }
        }
    }

    false
}

/// `true` iff `atom` is a possibly-empty array or list whose only inhabitant
/// is the empty container `[]`, because a key, value, or element parameter is
/// uninhabited and so no entry can ever be stored. Such an atom is
/// semantically `[]` regardless of those (vacuous) parameters, so it is
/// covered by any union admitting the empty container - which a pointwise
/// key/value refinement would miss, since e.g. `array<0, A&D>` (with `A&D`
/// uninhabited) has key `0` that does not refine the empty array's `never`
/// key. Requires the symbol table to judge the parameter uninhabited, so this lives
/// in the comparison layer rather than in builder canonicalization.
#[inline]
pub(crate) fn atom_is_empty_container<'arena, S, A>(
    atom: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    match atom {
        Atom::Array(payload) => {
            if payload.flags.contains(ArrayFlag::NonEmpty) || payload.known_items.is_some() {
                return false;
            }

            payload.key_param.is_some_and(|ty| type_is_value_never(ty, symbols, builder))
                || payload.value_param.is_some_and(|ty| type_is_value_never(ty, symbols, builder))
        }
        Atom::List(payload) => {
            if payload.flags.contains(ListFlag::NonEmpty) || payload.known_elements.is_some() {
                return false;
            }

            type_is_value_never(payload.element_type, symbols, builder)
        }
        _ => false,
    }
}

/// `true` iff some atom in `containers` admits the empty container `[]`.
///
/// The empty array and the empty list are the *same* runtime value, so empty
/// coverage is a single cross-family question: any possibly-empty list or
/// array (with no required known element / item) admits `[]`. This is what
/// lets a possibly-empty array be covered when its empty case lands in an
/// empty-list piece of a union (e.g. `meet(array<int,float>, list<int>)`
/// yields `list<never>`) and its non-empty case lands in an array piece.
#[inline]
fn union_admits_empty_container(containers: &[Atom<'_>]) -> bool {
    containers.iter().copied().any(atom_admits_empty_container)
}

/// `true` iff the single atom `atom` contains the empty container `[]`: a
/// possibly-empty list or array with no required known element / item, or any
/// `iterable` (the empty array inhabits `iterable<K, V>` for every `K`, `V`).
#[inline]
pub(crate) fn atom_admits_empty_container(atom: Atom<'_>) -> bool {
    match atom {
        Atom::List(payload) => {
            !payload.flags.contains(ListFlag::NonEmpty)
                && payload.known_elements.is_none_or(|elements| elements.iter().all(|element| element.optional))
        }
        Atom::Array(payload) => {
            !payload.flags.contains(ArrayFlag::NonEmpty)
                && payload.known_items.is_none_or(|items| items.iter().all(|item| item.optional))
        }
        Atom::Iterable(_) => true,
        _ => false,
    }
}

/// True iff a possibly-empty `array<K, V>` input is covered by the union of
/// `containers`. An `array<K, V>` is the empty array together with the
/// non-empty arrays, so it refines the union exactly when (1) some container
/// admits the empty container and (2) the non-empty arrays refine the union.
///
/// The non-empty part is checked by a real recursive [`refines`], so it is
/// covered by a single container (`array<K, V> <: array<K', V'>` when
/// `K <: K'` and `V <: V'`) or by the `!B | B` intersected-partition split -
/// never by a pointwise product of the containers' parameters. The pointwise
/// product is unsound: it would accept `array<int, object>` into
/// `array<int(0), mixed> | array<int, int>` even though `[5 => obj]` inhabits
/// neither member. A non-empty input needs no split and is left to the
/// caller's single-container `atom_refines` checks (recursing here would loop).
#[inline]
fn array_union_covers<'arena, S, A>(
    input: Atom<'arena>,
    containers: &[Atom<'arena>],
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let Atom::Array(input_payload) = input else {
        return false;
    };

    if input_payload.known_items.is_some() || input_payload.flags.contains(ArrayFlag::NonEmpty) {
        return false;
    }

    if input_payload.key_param.is_none() || input_payload.value_param.is_none() {
        return false;
    }

    if !union_admits_empty_container(containers) {
        return false;
    }

    let non_empty = builder.array(ArrayAtom { flags: input_payload.flags.with(ArrayFlag::NonEmpty), ..*input_payload });
    let non_empty_type = builder.union_of(&[non_empty]);
    let containers_type = builder.union_of(containers);

    refines(non_empty_type, containers_type, symbols, options, report, builder)
}

/// True iff a possibly-empty `list<E>` input is covered by the union of
/// `containers`, by the same empty/non-empty split as [`array_union_covers`]:
/// the empty list must be admitted by some container, and the non-empty lists
/// must refine the union via a real recursive [`refines`] rather than a
/// pointwise element union (`list<A | B>` does not refine `list<A> | list<B>`,
/// since `[a, b]` inhabits neither).
#[inline]
fn list_union_covers<'arena, S, A>(
    input: Atom<'arena>,
    containers: &[Atom<'arena>],
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let Atom::List(input_payload) = input else {
        return false;
    };

    if input_payload.known_elements.is_some() || input_payload.flags.contains(ListFlag::NonEmpty) {
        return false;
    }

    if !union_admits_empty_container(containers) {
        return false;
    }

    let non_empty = builder.list(ListAtom { flags: input_payload.flags.with(ListFlag::NonEmpty), ..*input_payload });
    let non_empty_type = builder.union_of(&[non_empty]);
    let containers_type = builder.union_of(containers);

    refines(non_empty_type, containers_type, symbols, options, report, builder)
}

/// Expand `!!X` atom shapes inside a union to `X`'s atoms.
/// Single-atom `!!T` collapses at construction; multi-atom `T = a|b`
/// survives as `Negated(Negated(T))` and gets flattened here so
/// the structural dispatch sees the inner atoms.
#[inline]
fn expand_double_negation<'scratch, 'arena, S, A>(
    ty: Type<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    if !ty.atoms.iter().any(|&atom| is_double_negation(atom)) {
        return ty;
    }

    let mut expanded: ScratchVec<'scratch, Atom<'arena>, S> = builder.scratch_vec_with(ty.atoms.len());
    for &atom in ty.atoms {
        if let Atom::Negated(inner) = atom
            && let [Atom::Negated(inner_inner)] = inner.atoms
        {
            expanded.extend_from_slice(inner_inner.atoms);
        } else {
            expanded.push(atom);
        }
    }

    builder.union_of(&expanded)
}

#[inline]
fn is_double_negation(atom: Atom<'_>) -> bool {
    let Atom::Negated(inner) = atom else {
        return false;
    };

    matches!(inner.atoms, [Atom::Negated(_)])
}

/// True iff a single generic-parameter input `T extends X` is covered
/// by the union of all same-`T` atoms in the container. Each container
/// atom contributes its constraint; if their union covers `X`, the input
/// is in the container (just split across same-template narrowings).
#[inline]
fn generic_parameter_union_covers<'scratch, 'arena, S, A>(
    input: Atom<'arena>,
    containers: &[Atom<'arena>],
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let Atom::GenericParameter(input_payload) = input else {
        return false;
    };

    let mut container_constraints: ScratchVec<'scratch, Atom<'arena>, S> = builder.scratch_vec();
    for &candidate in containers {
        let Atom::GenericParameter(candidate_payload) = candidate else {
            continue;
        };

        if candidate_payload.name != input_payload.name
            || candidate_payload.defining_entity != input_payload.defining_entity
        {
            continue;
        }

        container_constraints.extend_from_slice(candidate_payload.constraint.atoms);
    }

    if container_constraints.is_empty() {
        return false;
    }

    let combined = builder.union_of(&container_constraints);
    refines(input_payload.constraint, combined, symbols, options, report, builder)
}

/// True iff a sealed named class `input` is covered by the union of
/// `containers` via its sealed inheritors. For sealed `Foo` with
/// inheritors `[Bar, Baz]`, `Foo <: containers` when `Bar <: containers`
/// and `Baz <: containers`.
#[inline]
fn sealed_survivors_cover<'arena, S, A>(
    input: Atom<'arena>,
    containers: &[Atom<'arena>],
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    if !matches!(input, Atom::Object(_)) {
        return false;
    }

    let residual = lattice::sealed::compute_residual(input, &[], symbols, options, report, builder);
    let surviving = match residual {
        SealedResidual::Surviving(surviving) => surviving,
        SealedResidual::FullyCovered => return true,
        SealedResidual::NotSealed => return false,
    };

    let containers_type = builder.union_of(containers);
    surviving.iter().all(|survivor| {
        let survivor_type = builder.union_of(&[*survivor]);
        refines(survivor_type, containers_type, symbols, options, report, builder)
    })
}

/// True iff the int range / literal `input` is fully covered by the union
/// of all int atoms in `containers`. Used as a precision fallback when
/// no single container atom accepts the input. The `UnspecifiedLiteral`
/// dominator is excluded on both sides because the lattice keeps it as a
/// distinct axis (`int <: literal-int` is intentionally false; treating
/// `literal-int` containers as unbounded coverage would silently break
/// that distinction). The broad `Unspecified` `int` input falls back here
/// when the disjuncts collectively cover the full integer range; required
/// for partition-style properties like `meet(a,b) ∪ subtract(a,b) ⊇ a`.
#[inline]
fn int_union_covers<'scratch, 'arena, S, A>(
    input: Atom<'_>,
    containers: &[Atom<'_>],
    builder: &TypeBuilder<'scratch, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let Atom::Int(input_payload) = input else {
        return false;
    };

    if !containers.iter().any(|atom| matches!(atom, Atom::Int(_))) {
        return false;
    }

    if matches!(input_payload, IntAtom::UnspecifiedLiteral) {
        return false;
    }

    let (input_lower, input_upper) = int_bounds_of(input_payload);

    let mut ranges: ScratchVec<'scratch, (Option<i64>, Option<i64>), S> = builder.scratch_vec();
    ranges.extend(containers.iter().filter_map(|candidate| match candidate {
        Atom::Int(IntAtom::UnspecifiedLiteral) => None,
        Atom::Int(payload) => Some(int_bounds_of(*payload)),
        _ => None,
    }));

    if ranges.is_empty() {
        return false;
    }

    ranges.sort_by(|left, right| match (left.0, right.0) {
        (None, None) => core::cmp::Ordering::Equal,
        (None, _) => core::cmp::Ordering::Less,
        (_, None) => core::cmp::Ordering::Greater,
        (Some(left_lower), Some(right_lower)) => left_lower.cmp(&right_lower),
    });

    let mut covered_up_to: Option<i64> = None;
    let mut started = false;

    for (lower, upper) in ranges {
        if !started {
            let starts_input = match (lower, input_lower) {
                (None, _) => true,
                (Some(_), None) => false,
                (Some(range_lower), Some(input_start)) => range_lower <= input_start,
            };

            if !starts_input {
                continue;
            }

            covered_up_to = match (input_lower, upper) {
                (Some(input_start), Some(range_upper)) if range_upper < input_start => continue,
                _ => upper,
            };

            started = true;
        } else {
            let connects = match (lower, covered_up_to) {
                (None, _) => true,
                (_, None) => true,
                (Some(range_lower), Some(covered)) => range_lower <= covered.saturating_add(1),
            };

            if !connects {
                return false;
            }

            covered_up_to = match (covered_up_to, upper) {
                (None, _) | (_, None) => None,
                (Some(covered), Some(range_upper)) => Some(covered.max(range_upper)),
            };
        }

        let covers_top = match (input_upper, covered_up_to) {
            (_, None) => true,
            (None, Some(_)) => false,
            (Some(input_end), Some(covered)) => input_end <= covered,
        };

        if covers_top {
            return true;
        }
    }

    false
}

/// True iff a broad `string` input is covered by the union of refined
/// string atoms in `containers`. Sufficient condition: the container
/// holds some atom that covers all non-empty strings AND some atom that
/// covers the empty string. Together that is the empty/non-empty
/// partition of `string`. Refined inputs (already non-empty,
/// truthy, etc.) bail; atom-wise refines is exact for them.
///
/// A qualifying non-empty cover carries the non-empty flag with literal
/// `None`/`Unspecified` and no casing / numeric / callable / truthy
/// refinement: `truthy-string` excludes the literal `"0"`, so it does NOT
/// cover all non-empty strings, and counting it here would falsely make
/// `string \ (truthy | empty)` collapse to `never`.
#[inline]
fn string_union_covers(input: Atom<'_>, containers: &[Atom<'_>]) -> bool {
    let Atom::String(input_payload) = input else {
        return false;
    };

    if !containers.iter().any(|atom| matches!(atom, Atom::String(_))) {
        return false;
    }

    let is_broad_string = matches!(input_payload.literal, StringLiteral::None)
        && input_payload.flags.is_empty()
        && matches!(input_payload.casing, StringCasing::Unspecified);
    if !is_broad_string {
        return false;
    }

    let mut covers_empty = false;
    let mut covers_non_empty = false;
    for &candidate in containers {
        let Atom::String(candidate_payload) = candidate else {
            continue;
        };

        if matches!(candidate_payload.literal, StringLiteral::Value(value) if value.is_empty()) {
            covers_empty = true;
        }

        if matches!(candidate_payload.literal, StringLiteral::None | StringLiteral::Unspecified)
            && candidate_payload.flags.contains(StringRefinementFlag::NonEmpty)
            && !candidate_payload.flags.contains(StringRefinementFlag::Truthy)
            && !candidate_payload.flags.contains(StringRefinementFlag::Numeric)
            && !candidate_payload.flags.contains(StringRefinementFlag::Callable)
            && matches!(candidate_payload.casing, StringCasing::Unspecified)
        {
            covers_non_empty = true;
        }
    }

    covers_empty && covers_non_empty
}

/// True iff broad `bool` is covered by the union of `true` and `false`
/// in `containers`. Mirrors [`int_union_covers`] for the bool axis.
#[inline]
fn bool_union_covers(input: Atom<'_>, containers: &[Atom<'_>]) -> bool {
    if input.kind() != AtomKind::Bool {
        return false;
    }

    let has_true = containers.iter().any(|atom| matches!(atom, Atom::True));
    let has_false = containers.iter().any(|atom| matches!(atom, Atom::False));
    has_true && has_false
}

/// True iff broad `mixed` is covered by `nonnull-mixed | null` in
/// `containers`. The null/non-null axis is the only structural
/// partition of `mixed` the lattice can recognize directly; deeper
/// coverage (e.g. `int | string | … = mixed`) needs an exhaustive
/// case-analysis we don't try here.
#[inline]
fn mixed_union_covers(input: Atom<'_>, containers: &[Atom<'_>]) -> bool {
    let Atom::Mixed(input_payload) = input else {
        return false;
    };

    if input_payload != MixedAtom::EMPTY {
        return false;
    }

    let has_null = containers.contains(&well_known::NULL);
    let has_non_null = containers.iter().any(|atom| matches!(atom, Atom::Mixed(payload) if payload.is_non_null()));
    has_null && has_non_null
}

#[inline]
fn int_bounds_of(payload: IntAtom<'_>) -> (Option<i64>, Option<i64>) {
    match payload {
        IntAtom::Unspecified | IntAtom::UnspecifiedLiteral => (None, None),
        IntAtom::Literal(value) => (Some(value), Some(value)),
        IntAtom::Range(range) => (range.lower(), range.upper()),
    }
}

/// `true` iff `a :> b`: every value of type `b` is also a value of type `a`
/// (`a` generalizes `b`). Equivalent to
/// `refines(b, a, symbols, options, report, builder)`.
#[inline]
pub fn generalizes<'arena, S, A>(
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
    refines(b, a, symbols, options, report, builder)
}

/// Decide whether one atom refines another.
///
/// Universal axioms first (refl, `never <: anything`, `anything <: mixed`),
/// then dispatch on the container's kind into a family-specific helper.
/// When the result is `false` and the input belongs to a "true-union" kind
/// (`mixed`, `array_key`, `bool`, `object_any`, `scalar`, `numeric`), the
/// [`CoercionCause::TrueUnionNarrow`] cause is recorded to flag that the
/// rejection was a narrowing, not an out-of-family mismatch. `mixed` inputs
/// additionally record [`CoercionCause::NestedMixed`]. `object_any`
/// inputs additionally record [`CoercionCause::ObjectAnyDown`].
///
/// The input-side `is_uninhabited` axiom deliberately has no
/// container-side short-circuit: an open `Foo & Bar` container can
/// pick up a common subclass via interfaces / traits, and the
/// container-intersection rule handles it via per-conjunct refinement.
/// The subtraction's atom-minus uses the symmetric check because subtract
/// has the inverse soundness needs.
#[inline]
pub(crate) fn atom_refines<'arena, S, A>(
    input: Atom<'arena>,
    container: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    if input == container {
        return true;
    }

    if input == well_known::NEVER {
        return true;
    }

    if lattice::is_uninhabited(input, symbols, builder) {
        return true;
    }

    if (input == well_known::VOID && container == well_known::NULL)
        || (input == well_known::NULL && container == well_known::VOID)
    {
        return true;
    }

    if container == well_known::MIXED {
        return true;
    }

    if let Atom::GenericParameter(input_payload) = input
        && !matches!(container, Atom::GenericParameter(_))
    {
        let constraint = input_payload.constraint;
        let container_type = builder.union_of(&[container]);
        let result = refines(constraint, container_type, symbols, options, report, builder);
        if !result && container != well_known::MIXED && constraint.atoms.contains(&well_known::MIXED) {
            report.causes.unset(CoercionCause::NestedMixed);
            report.add_cause(CoercionCause::TrueUnionNarrow);
            report.add_cause(CoercionCause::FromAsMixed);
        }

        return result;
    }

    if matches!(container, Atom::Intersected(_)) {
        return refines_container_intersected(input, container, symbols, options, report, builder);
    }

    if matches!(input, Atom::Intersected(_)) {
        if matches!(container, Atom::Mixed(_)) {
            return family::mixed::refines(input, container);
        }

        return refines_input_intersected(input, container, symbols, options, report, builder);
    }

    let result = dispatch_refines(input, container, symbols, options, report, builder);

    if result {
        if input.kind() == AtomKind::Int && container.kind() == AtomKind::Float {
            report.add_cause(CoercionCause::PhpRuntimeCoerce);
        }
    } else if is_true_union_kind(input.kind()) {
        report.add_cause(CoercionCause::TrueUnionNarrow);
        match input.kind() {
            AtomKind::Mixed => report.add_cause(CoercionCause::NestedMixed),
            AtomKind::ObjectAny => report.add_cause(CoercionCause::ObjectAnyDown),
            _ => {}
        }
    }

    result
}

#[inline]
fn refines_container_intersected<'arena, S, A>(
    input: Atom<'arena>,
    container: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let Atom::Intersected(payload) = container else {
        return false;
    };

    if !atom_refines(input, *payload.head, symbols, options, report, builder) {
        return false;
    }

    for &conjunct in payload.conjuncts {
        if !atom_refines(input, conjunct, symbols, options, report, builder) {
            return false;
        }
    }

    true
}

#[inline]
fn refines_input_intersected<'arena, S, A>(
    input: Atom<'arena>,
    container: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let Atom::Intersected(payload) = input else {
        return false;
    };

    if atom_refines(*payload.head, container, symbols, options, report, builder) {
        return true;
    }

    for &conjunct in payload.conjuncts {
        if atom_refines(conjunct, container, symbols, options, report, builder) {
            return true;
        }
    }

    if sealed_intersected_cover(input, container, symbols, options, report, builder) {
        return true;
    }

    false
}

/// Sealed-cover for an `Intersected(H sealed, conjuncts)` input against the
/// whole container union: `H ≡ ⋃ inheritors`, so the input equals the union
/// of each surviving inheritor intersected with the original conjuncts. The
/// input refines the container when every such `survivor & conjuncts` does.
///
/// Unlike [`sealed_intersected_cover`], which sees one container atom at a
/// time, this runs in the top-level union dispatch with all container atoms in
/// hand - necessary because the survivors typically land in *different*
/// container atoms (`C & E ≡ (A & E) | (B & E)`, with `A & E` and `B & E` in
/// separate union members). The conjuncts are carried onto each survivor so a
/// positive constraint like `& E` is not silently dropped.
#[inline]
fn sealed_intersected_union_cover<'scratch, 'arena, S, A>(
    input: Atom<'arena>,
    containers: &[Atom<'arena>],
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let Atom::Intersected(payload) = input else {
        return false;
    };

    if !matches!(payload.head, Atom::Object(_)) {
        return false;
    }

    let mut negated_inners: ScratchVec<'scratch, Type<'arena>, S> = builder.scratch_vec_with(payload.conjuncts.len());
    for &conjunct in payload.conjuncts {
        if let Atom::Negated(inner) = conjunct {
            negated_inners.push(*inner);
        }
    }

    let residual = lattice::sealed::compute_residual(*payload.head, &negated_inners, symbols, options, report, builder);
    let surviving = match residual {
        SealedResidual::Surviving(surviving) => surviving,
        SealedResidual::FullyCovered => return true,
        SealedResidual::NotSealed => return false,
    };

    let containers_type = builder.union_of(containers);
    surviving.iter().all(|&survivor| {
        let survivor_atom = builder.intersected(survivor, payload.conjuncts);
        let survivor_type = builder.union_of(&[survivor_atom]);
        refines(survivor_type, containers_type, symbols, options, report, builder)
    })
}

/// Sealed-cover: `Intersected(H sealed, conjuncts) <: container` when
/// `H`'s surviving inheritors each refine `container`.
#[inline]
fn sealed_intersected_cover<'scratch, 'arena, S, A>(
    input: Atom<'arena>,
    container: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let Atom::Intersected(payload) = input else {
        return false;
    };

    if !matches!(payload.head, Atom::Object(_)) {
        return false;
    }

    let mut negated_inners: ScratchVec<'scratch, Type<'arena>, S> = builder.scratch_vec_with(payload.conjuncts.len());
    for &conjunct in payload.conjuncts {
        if let Atom::Negated(inner) = conjunct {
            negated_inners.push(*inner);
        }
    }

    let residual = lattice::sealed::compute_residual(*payload.head, &negated_inners, symbols, options, report, builder);
    let surviving = match residual {
        SealedResidual::Surviving(surviving) => surviving,
        SealedResidual::FullyCovered => return true,
        SealedResidual::NotSealed => return false,
    };

    let container_type = builder.union_of(&[container]);
    surviving.iter().all(|survivor| {
        let survivor_type = builder.union_of(&[*survivor]);
        refines(survivor_type, container_type, symbols, options, report, builder)
    })
}

/// `true` for kinds whose values inhabit multiple disjoint sub-families:
/// narrowing one of these to a concrete sub-form is the standard PHP
/// "type-coerced" pattern that the lattice records via
/// [`CoercionCause::TrueUnionNarrow`].
#[inline]
const fn is_true_union_kind(kind: AtomKind) -> bool {
    matches!(
        kind,
        AtomKind::Mixed
            | AtomKind::ArrayKey
            | AtomKind::Bool
            | AtomKind::ObjectAny
            | AtomKind::Scalar
            | AtomKind::Numeric
    )
}

/// Dispatch on the container's kind into the owning family module.
///
/// `Intersected` containers are stripped and re-routed by the dedicated
/// intersection paths in [`atom_refines`] before this dispatch runs; an
/// `Intersected` reaching the final arm means the structural check has
/// already ruled the input out, so it answers `false` alongside the other
/// kinds with no widening container semantics.
#[inline]
fn dispatch_refines<'arena, S, A>(
    input: Atom<'arena>,
    container: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    if input.kind() == AtomKind::Negated {
        return family::negated::refines_input_negated(input, container, symbols, options, report, builder);
    }

    if container.kind() == AtomKind::Negated {
        return family::negated::refines_container_negated(input, container, symbols, options, report, builder);
    }

    match container.kind() {
        AtomKind::Bool => family::bool::refines(input, container),
        AtomKind::Resource => family::resource::refines(input, container),
        AtomKind::Int => family::int::refines(input, container),
        AtomKind::Float => family::float::refines(input, container),
        AtomKind::String => family::string::refines(input, container),
        AtomKind::ClassLikeString => {
            family::class_like_string::refines(input, container, symbols, options, report, builder)
        }
        AtomKind::ArrayKey => family::array_key::refines(input, container),
        AtomKind::Numeric => family::numeric::refines(input, container),
        AtomKind::Scalar => family::scalar::refines(input, container),
        AtomKind::ObjectAny => family::object::refines_object_any(input, container),
        AtomKind::Object | AtomKind::Enum | AtomKind::ObjectShape | AtomKind::HasMethod | AtomKind::HasProperty => {
            family::object::refines(input, container, symbols, options, report, builder)
        }
        AtomKind::Array | AtomKind::List => family::array::refines(input, container, symbols, options, report, builder),
        AtomKind::Iterable => family::iterable::refines(input, container, symbols, options, report, builder),
        AtomKind::Callable => family::callable::refines(input, container, symbols, options, report, builder),
        AtomKind::Mixed => family::mixed::refines(input, container),
        AtomKind::GenericParameter => family::generic::refines(input, container, symbols, options, report, builder),
        AtomKind::Variable
        | AtomKind::Reference
        | AtomKind::MemberReference
        | AtomKind::GlobalReference
        | AtomKind::Alias
        | AtomKind::Conditional
        | AtomKind::Derived => family::reference::refines(input, container),
        AtomKind::Null
        | AtomKind::Never
        | AtomKind::Void
        | AtomKind::Placeholder
        | AtomKind::True
        | AtomKind::False
        | AtomKind::Negated
        | AtomKind::Intersected => false,
    }
}
