//! Lattice meet (greatest lower bound): the type-returning intersection.
//!
//! Two entry points:
//!
//! - [`narrow`] is the primary operation. It runs the meet and
//!   classifies the result for assertion-driven narrowing: `Impossible`
//!   when the inputs are disjoint, `Redundant` when the input already
//!   refines the narrowing (the assertion adds no information),
//!   `Narrowed` when the result is strictly more specific.
//! - [`compute`] is a thin wrapper that throws away the classification
//!   and returns just the meet's [`Type`]. Use it when you want the
//!   intersection of two types unrelated to assertions (e.g. computing
//!   `A ∧ B` to feed into a later operation).
//!
//! In type-lattice terms, `compute(A, B)` is the greatest lower bound
//! (meet, ⊓) of `A` and `B` under this crate's subtype order, paired
//! with the union join in [`crate::ty::join`].
//!
//! # Strategy
//!
//! Intersection distributes over union: for each
//! atom on either side we compute pairwise atom meets, drop the
//! disjoint pairs, and union the surviving atoms.
//!
//! Atom-pair meet walks these rules in order:
//!
//! 1. Reflexivity / `never` / `mixed` / `placeholder`.
//! 2. Subsumption: if either side refines the other, the more specific
//!    one is the meet.
//! 3. Family-specific positive rules in the family submodules (integer ranges,
//!    string axes + numeric-string crossing, list / keyed-array shape
//!    composition, compositional object intersections).
//! 4. Otherwise the pair is treated as disjoint (`None`).
//!
//! # Soundness vs precision
//!
//! Returning [`well_known::TYPE_NEVER`] for a pair that actually overlaps
//! is a precision loss but never an unsoundness: `never <: anything` so
//! the lower-bound axiom holds. As family rules grow, what previously
//! collapsed to `never` becomes the real meet; every step is monotone
//! in result precision. The same precision debt feeds the classifier in
//! [`narrow`]: an unhandled overlap pair will be misreported as
//! `Impossible`, never as a false `Redundant`/`Narrowed`.

use mago_allocator::Arena;
use mago_allocator::vec::Vec as ScratchVec;
use mago_flags::U8Flags;

use crate::ty::Type;
use crate::ty::atom::Atom;
use crate::ty::atom::kind::AtomKind;
use crate::ty::atom::payload::array::ArrayAtom;
use crate::ty::atom::payload::array::ArrayFlag;
use crate::ty::atom::payload::array::ListAtom;
use crate::ty::atom::payload::array::ListFlag;
use crate::ty::atom::payload::scalar::mixed::MixedAtom;
use crate::ty::atom::payload::scalar::mixed::Truthiness;
use crate::ty::atom::payload::scalar::string::StringAtom;
use crate::ty::atom::payload::scalar::string::StringCasing;
use crate::ty::atom::payload::scalar::string::StringLiteral;
use crate::ty::atom::payload::scalar::string::StringRefinementFlag;
use crate::ty::builder::TypeBuilder;
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;
use crate::ty::lattice::is_uninhabited;
use crate::ty::lattice::overlaps;
use crate::ty::lattice::refines;
use crate::ty::lattice::sealed::SealedResidual;
use crate::ty::lattice::sealed::compute_residual;
use crate::ty::meet::family::generic;
use crate::ty::well_known;
use crate::world::World;

pub(crate) mod family;

/// Outcome of [`narrow`], classifying an assertion-driven meet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MeetOutcome<'arena> {
    /// The input and the narrowing have no values in common. The
    /// assertion `input is σ` cannot hold for any value of `input`.
    Impossible,
    /// The input is already a subtype of the narrowing; the assertion
    /// adds no information. Carries the (unchanged) input.
    Redundant(Type<'arena>),
    /// The narrowing strictly refined the input. Carries the new type.
    Narrowed(Type<'arena>),
}

impl<'arena> MeetOutcome<'arena> {
    /// Extract the resulting [`Type`], mapping `Impossible` to
    /// [`well_known::TYPE_NEVER`].
    #[inline]
    #[must_use]
    pub fn into_type(self) -> Type<'arena> {
        match self {
            Self::Impossible => well_known::TYPE_NEVER,
            Self::Redundant(ty) | Self::Narrowed(ty) => ty,
        }
    }
}

/// Compute `input ∧ narrowing` and classify the outcome for
/// assertion-driven diagnostics.
///
/// `input` is the existing type; `narrowing` is the type asserted at
/// the use site (e.g. the right-hand side of `instanceof`). Both
/// `result <: input` and `result <: narrowing` always hold for the
/// `Narrowed` and `Redundant` variants; `Impossible` corresponds to
/// `result ≡ ⊥`.
#[inline]
pub fn narrow<'scratch, 'arena, S, A, W>(
    input: Type<'arena>,
    narrowing: Type<'arena>,
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> MeetOutcome<'arena>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    if input == narrowing {
        return MeetOutcome::Redundant(input);
    }

    let mut atoms: ScratchVec<'scratch, Atom<'arena>, S> =
        builder.scratch_vec_with(input.atoms.len().saturating_mul(narrowing.atoms.len()));

    let any_negated = input.kinds.contains(AtomKind::Negated) || narrowing.kinds.contains(AtomKind::Negated);
    let any_mixed = input.kinds.contains(AtomKind::Mixed) || narrowing.kinds.contains(AtomKind::Mixed);

    for &input_atom in input.atoms {
        for &narrowing_atom in narrowing.atoms {
            if any_negated && (input_atom.kind() == AtomKind::Negated || narrowing_atom.kind() == AtomKind::Negated) {
                negated_atom_meet_multi(input_atom, narrowing_atom, world, options, report, builder, &mut atoms);
                continue;
            }

            if cross_dominator_meet(input_atom, narrowing_atom, &mut atoms) {
                continue;
            }

            if any_mixed && narrowed_mixed_meet_multi(input_atom, narrowing_atom, world, builder, &mut atoms) {
                continue;
            }

            if let Some(met) = atom_meet(input_atom, narrowing_atom, world, options, report, builder) {
                atoms.push(met);
            }
        }
    }

    if atoms.is_empty() {
        return MeetOutcome::Impossible;
    }

    let result = builder.union_of(&atoms);
    if result == input { MeetOutcome::Redundant(input) } else { MeetOutcome::Narrowed(result) }
}

/// Compute `A ∧ B`: the largest type whose values are in both `A` and `B`.
///
/// Returns [`well_known::TYPE_NEVER`] when the two are disjoint (or when
/// no rule yet describes their overlap; precision can only grow).
///
/// This is a thin wrapper over [`narrow`] for callers that don't need
/// the assertion classification.
#[inline]
pub fn compute<'arena, S, A, W>(
    a: Type<'arena>,
    b: Type<'arena>,
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    narrow(a, b, world, options, report, builder).into_type()
}

/// Pairwise atom meet. `int ∧ float` is treated as disjoint even though
/// `int <: float` holds as a one-directional PHP parameter-coercion
/// rule: an `int(0)` runtime value is not a member of `float`, so the
/// value-level intersection must never be re-introduced through the
/// coercion-aware `refines` subsumption short-circuit.
#[inline]
fn atom_meet<'arena, S, A, W>(
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
    if is_uninhabited(a, world, builder) || is_uninhabited(b, world, builder) {
        return None;
    }

    if a == b {
        return Some(a);
    }

    if a == well_known::NEVER || b == well_known::NEVER {
        return None;
    }

    if a == well_known::MIXED || a == well_known::PLACEHOLDER {
        if is_uninhabited(b, world, builder) {
            return None;
        }

        return Some(b);
    }

    if b == well_known::MIXED || b == well_known::PLACEHOLDER {
        if is_uninhabited(a, world, builder) {
            return None;
        }

        return Some(a);
    }

    if matches!((a.kind(), b.kind()), (AtomKind::Int, AtomKind::Float) | (AtomKind::Float, AtomKind::Int)) {
        return None;
    }

    if a.kind() == AtomKind::Negated || b.kind() == AtomKind::Negated {
        return negated_atom_meet(a, b, world, options, report, builder);
    }

    if a.kind() == AtomKind::Intersected || b.kind() == AtomKind::Intersected {
        return intersected_atom_meet(a, b, world, options, report, builder);
    }

    if a.kind() == AtomKind::Mixed || b.kind() == AtomKind::Mixed {
        let met = narrowed_mixed_meet(a, b, world, builder);
        return normalise_meet_result(met, world, builder);
    }

    let a_type = builder.union_of(&[a]);
    let b_type = builder.union_of(&[b]);

    if refines(a_type, b_type, world, options, report, builder) {
        return normalise_meet_result(Some(a), world, builder);
    }

    if refines(b_type, a_type, world, options, report, builder) {
        return normalise_meet_result(Some(b), world, builder);
    }

    if a.kind() == AtomKind::GenericParameter || b.kind() == AtomKind::GenericParameter {
        let met = generic::generic_parameter_meet(a, b, world, options, report, builder);
        return normalise_meet_result(met, world, builder);
    }

    let met = family_atom_meet(a, b, world, options, report, builder);
    normalise_meet_result(met, world, builder)
}

/// If the synthesised atom is uninhabited (e.g. sealed-class
/// intersection with all inheritors negated), collapse to `None`
/// so the caller treats it as the empty meet.
#[inline]
fn normalise_meet_result<'arena, S, A, W>(
    result: Option<Atom<'arena>>,
    world: &W,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    match result {
        Some(atom) if is_uninhabited(atom, world, builder) => None,
        other => other,
    }
}

/// Cross-dominator pair meet: `(ArrayKey, Numeric)` shares `int`
/// and `numeric-string` but neither dominates, so subsumption
/// can't fire. `(Scalar, *)` already collapses via subsumption.
/// Pushes the shared constituents into `out` and returns `true` when
/// the pair is a cross-dominator; returns `false` to fall through.
#[inline]
fn cross_dominator_meet<'arena, S>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    out: &mut ScratchVec<'_, Atom<'arena>, S>,
) -> bool
where
    S: Arena,
{
    if matches!((a.kind(), b.kind()), (AtomKind::ArrayKey, AtomKind::Numeric) | (AtomKind::Numeric, AtomKind::ArrayKey))
    {
        out.push(well_known::INT);
        out.push(well_known::NUMERIC_STRING);
        return true;
    }

    false
}

/// Multi-atom variant of [`negated_atom_meet`] used by [`narrow`]:
/// pushes every surviving atom into `out` (e.g.
/// `meet(non-negative-int, !int(1))` yields `[int(0), int<2,∞>]`).
#[inline]
fn negated_atom_meet_multi<'scratch, 'arena, S, A, W>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
    out: &mut ScratchVec<'scratch, Atom<'arena>, S>,
) where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    if a.kind() == AtomKind::Negated && b.kind() == AtomKind::Negated {
        let met = negated_pair_meet(a, b, world, options, report, builder);
        out.push(met);
        return;
    }

    let (positive, negated_inner) = match (a, b) {
        (Atom::Negated(inner), other) | (other, Atom::Negated(inner)) => (other, *inner),
        _ => return,
    };

    let positive_type = builder.union_of(&[positive]);
    let surviving = crate::ty::subtract::compute(positive_type, negated_inner, world, options, report, builder);
    if surviving.is_never() {
        return;
    }

    out.extend_from_slice(surviving.atoms);
}

/// `meet(!T, !U) ≡ !(T ∪ U)`. When `T <: U` the union collapses
/// to `U` and the result is `!U`; symmetric for `U <: T`.
#[inline]
fn negated_pair_meet<'scratch, 'arena, S, A, W>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Atom<'arena>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let (Atom::Negated(a_inner), Atom::Negated(b_inner)) = (a, b) else {
        return well_known::NEVER;
    };

    if refines(*a_inner, *b_inner, world, options, report, builder) {
        return b;
    }

    if refines(*b_inner, *a_inner, world, options, report, builder) {
        return a;
    }

    let mut union_atoms: ScratchVec<'scratch, Atom<'arena>, S> = builder.scratch_vec_from_slice(a_inner.atoms);
    union_atoms.extend_from_slice(b_inner.atoms);
    let union_type = builder.union_of(&union_atoms);

    builder.negated(union_type)
}

/// `meet` with a `Negated` participant. `meet(X, !T)` ≡
/// `subtract(X, T)`; `meet(!T, !U)` ≡ `!(T ∪ U)`. Returning a single
/// [`Atom`] constrains the surviving form: when `subtract`
/// produces multiple atoms (e.g. `int \ int(0) → negative-int |
/// positive-int`), we union them under a single negated atom only
/// when both operands were negated; otherwise we conservatively
/// drop to `None` and let the caller (via the loop in `narrow`)
/// fall back through other meet pairs. A future refactor of
/// `atom_meet` to return `Vec<Atom>` would make this exact.
#[inline]
fn negated_atom_meet<'arena, S, A, W>(
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
    if a.kind() == AtomKind::Negated && b.kind() == AtomKind::Negated {
        return Some(negated_pair_meet(a, b, world, options, report, builder));
    }

    let (positive, negated_inner) = match (a, b) {
        (Atom::Negated(inner), other) | (other, Atom::Negated(inner)) => (other, *inner),
        _ => return None,
    };

    let positive_type = builder.union_of(&[positive]);
    let surviving = crate::ty::subtract::compute(positive_type, negated_inner, world, options, report, builder);
    if surviving.is_never() {
        return None;
    }

    if let [single] = surviving.atoms {
        return Some(*single);
    }

    None
}

#[inline]
fn intersected_atom_meet<'scratch, 'arena, S, A, W>(
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
    let result = match (a, b) {
        (Atom::Intersected(a_payload), Atom::Intersected(b_payload)) => {
            let head = atom_meet(*a_payload.head, *b_payload.head, world, options, report, builder)?;
            let mut all_conjuncts: ScratchVec<'scratch, Atom<'arena>, S> = builder.scratch_vec_from_slice(a_payload.conjuncts);
            all_conjuncts.extend_from_slice(b_payload.conjuncts);

            builder.intersected(head, &all_conjuncts)
        }
        (Atom::Intersected(payload), other) | (other, Atom::Intersected(payload)) => {
            let head = atom_meet(*payload.head, other, world, options, report, builder)?;

            builder.intersected(head, payload.conjuncts)
        }
        _ => return None,
    };

    if let Some(canonical) = canonicalise_intersected(result, world, options, report, builder) {
        return Some(canonical);
    }

    if is_uninhabited(result, world, builder) {
        return None;
    }

    Some(result)
}

/// Drop redundant negated conjuncts and collapse sealed-cover residuals.
/// A negated conjunct `!X` is redundant when `head` is disjoint from `X`
/// (`!overlaps(head, X)`), meaning the head already satisfies the negation.
/// After dropping redundancies, a sealed-cover single-survivor residual
/// replaces the Intersected with the bare inheritor.
#[inline]
fn canonicalise_intersected<'scratch, 'arena, S, A, W>(
    atom: Atom<'arena>,
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
    let Atom::Intersected(payload) = atom else {
        return None;
    };

    let head_is_object = payload.head.kind() == AtomKind::Object;
    let head_type = if head_is_object { Some(builder.union_of(&[*payload.head])) } else { None };

    let mut kept: ScratchVec<'scratch, Atom<'arena>, S> = builder.scratch_vec_with(payload.conjuncts.len());
    let mut negated_inners: ScratchVec<'scratch, Type<'arena>, S> = builder.scratch_vec_with(payload.conjuncts.len());
    for &conjunct in payload.conjuncts {
        if let Atom::Negated(inner) = conjunct {
            if let Some(head) = head_type
                && !overlaps(head, *inner, world, options, report, builder)
            {
                continue;
            }

            negated_inners.push(*inner);
        }

        kept.push(conjunct);
    }

    if kept.is_empty() {
        return Some(*payload.head);
    }

    if payload.head.kind() == AtomKind::Object && !negated_inners.is_empty() {
        let residual = compute_residual(*payload.head, &negated_inners, world, options, report, builder);
        match residual {
            SealedResidual::Surviving(survivors) if survivors.len() == 1 => {
                let survivor = survivors.first().copied()?;
                let survivor_type = builder.union_of(&[survivor]);
                let mut residual_conjuncts: ScratchVec<'scratch, Atom<'arena>, S> =
                    builder.scratch_vec_with(kept.len());
                for &conjunct in &kept {
                    let still_applies = match conjunct {
                        Atom::Negated(inner) => overlaps(survivor_type, *inner, world, options, report, builder),
                        _ => true,
                    };

                    if still_applies {
                        residual_conjuncts.push(conjunct);
                    }
                }

                return if residual_conjuncts.is_empty() {
                    Some(survivor)
                } else {
                    Some(builder.intersected(survivor, &residual_conjuncts))
                };
            }
            SealedResidual::FullyCovered => {
                return Some(well_known::NEVER);
            }
            _ => {}
        }
    }

    if kept.len() == payload.conjuncts.len() {
        return None;
    }

    Some(builder.intersected(*payload.head, &kept))
}

/// `meet(narrowed-mixed, X)` where `narrowed-mixed` is `truthy-mixed`,
/// `falsy-mixed`, or `non-null-mixed`. Returns `X` filtered by the
/// flag, expressed via the universal `Intersected` / `Negated`
/// machinery and PHP truthiness semantics for each atom kind.
#[inline]
fn narrowed_mixed_meet<'scratch, 'arena, S, A, W>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    world: &W,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let mut pieces: ScratchVec<'scratch, Atom<'arena>, S> = builder.scratch_vec();
    if !narrowed_mixed_meet_multi(a, b, world, builder, &mut pieces) {
        return None;
    }

    match pieces.as_slice() {
        [single] => Some(*single),
        _ => None,
    }
}

/// Multi-atom variant of [`narrowed_mixed_meet`]. Pushes the surviving
/// atoms into `out` and returns `true` when one side is a `Mixed` atom;
/// returns `false` (pushing nothing) when neither side is `Mixed` so
/// the caller falls through. A `true` with no pushed atoms is the empty
/// meet.
#[inline]
fn narrowed_mixed_meet_multi<'scratch, 'arena, S, A, W>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    world: &W,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
    out: &mut ScratchVec<'scratch, Atom<'arena>, S>,
) -> bool
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    if let (Atom::Mixed(a_payload), Atom::Mixed(b_payload)) = (a, b) {
        let merged_truthiness = match (a_payload.truthiness(), b_payload.truthiness()) {
            (Truthiness::Truthy, Truthiness::Falsy) | (Truthiness::Falsy, Truthiness::Truthy) => {
                return true;
            }
            (Truthiness::Truthy, _) | (_, Truthiness::Truthy) => Truthiness::Truthy,
            (Truthiness::Falsy, _) | (_, Truthiness::Falsy) => Truthiness::Falsy,
            (Truthiness::Undetermined, Truthiness::Undetermined) => Truthiness::Undetermined,
        };

        let merged = MixedAtom::EMPTY
            .with_is_non_null(a_payload.is_non_null() || b_payload.is_non_null())
            .with_is_empty(a_payload.is_empty() || b_payload.is_empty())
            .with_is_isset_from_loop(a_payload.is_isset_from_loop() || b_payload.is_isset_from_loop())
            .with_truthiness(merged_truthiness);

        out.push(Atom::Mixed(merged));
        return true;
    }

    let ((Atom::Mixed(mixed_payload), other) | (other, Atom::Mixed(mixed_payload))) = (a, b) else {
        return false;
    };

    if mixed_payload == MixedAtom::EMPTY {
        if !is_uninhabited(other, world, builder) {
            out.push(other);
        }

        return true;
    }

    if mixed_payload.is_non_null() && other == well_known::NULL {
        return true;
    }

    let start = out.len();
    narrow_by_truthiness(other, mixed_payload.truthiness(), builder, out);
    if mixed_payload.is_non_null() {
        let negated_null = builder.negated(well_known::TYPE_NULL);
        for index in start..out.len() {
            out[index] = builder.intersected(out[index], &[negated_null]);
        }
    }

    true
}

/// Narrow `other` by PHP truthiness, pushing the surviving atoms into
/// `out`. Pushes nothing when the kind is incompatible with the
/// requested truthiness (e.g. `Object` is always truthy, so falsy
/// narrowing yields the empty set), and pushes `other` unchanged when
/// truthiness is undetermined.
#[inline]
fn narrow_by_truthiness<'scratch, 'arena, S, A>(
    other: Atom<'arena>,
    truthiness: Truthiness,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
    out: &mut ScratchVec<'scratch, Atom<'arena>, S>,
) where
    S: Arena,
    A: Arena,
{
    if matches!(truthiness, Truthiness::Undetermined) {
        out.push(other);
        return;
    }

    match (other, truthiness) {
        (Atom::Null | Atom::False, Truthiness::Truthy) => {}
        (Atom::True, Truthiness::Falsy) => {}
        (
            Atom::Object(_)
            | Atom::ObjectAny
            | Atom::Enum(_)
            | Atom::ObjectShape(_)
            | Atom::HasMethod(_)
            | Atom::HasProperty(_)
            | Atom::Resource(_)
            | Atom::Callable(_)
            | Atom::ClassLikeString(_),
            Truthiness::Falsy,
        ) => {}
        (Atom::Bool, Truthiness::Truthy) => out.push(well_known::TRUE),
        (Atom::Bool, Truthiness::Falsy) => out.push(well_known::FALSE),
        (Atom::Int(_), Truthiness::Truthy) => {
            let zero_type = builder.union_of(&[well_known::INT_ZERO]);
            let negated_zero = builder.negated(zero_type);
            let intersected = builder.intersected(other, &[negated_zero]);

            out.push(intersected);
        }
        (Atom::Int(_), Truthiness::Falsy) => out.push(well_known::INT_ZERO),
        (Atom::Float(_), Truthiness::Truthy) => {
            let zero = Atom::float_literal(0.0);
            let zero_type = builder.union_of(&[zero]);
            let negated_zero = builder.negated(zero_type);
            let intersected = builder.intersected(other, &[negated_zero]);

            out.push(intersected);
        }
        (Atom::Float(_), Truthiness::Falsy) => out.push(Atom::float_literal(0.0)),
        (Atom::String(payload), Truthiness::Truthy) => {
            let truthy = narrow_string_truthy(payload, builder);

            out.push(truthy);
        }
        (Atom::String(payload), Truthiness::Falsy) => narrow_string_falsy(payload, builder, out),
        (Atom::List(_) | Atom::Array(_) | Atom::Iterable(_), Truthiness::Truthy) => {
            let non_empty = force_non_empty(other, builder);

            out.push(non_empty);
        }
        (Atom::List(_) | Atom::Array(_), Truthiness::Falsy) => {
            if let Some(empty) = falsy_collection(other, builder) {
                out.push(empty);
            }
        }
        (Atom::Iterable(_), Truthiness::Falsy) => {}
        _ => match (crate::ty::lattice::family::mixed::truthiness_of(other), truthiness) {
            (Truthiness::Truthy, Truthiness::Falsy) | (Truthiness::Falsy, Truthiness::Truthy) => {}
            _ => out.push(other),
        },
    }
}

/// `falsy ∩ list/array<X>` is the empty collection singleton when the
/// input allows empty (`non_empty=false` and not sealed-non-empty),
/// otherwise the empty set (non-empty collections are all truthy).
#[inline]
fn falsy_collection<'arena, S, A>(
    atom: Atom<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
{
    match atom {
        Atom::List(payload) => {
            if payload.flags.contains(ListFlag::NonEmpty) {
                return None;
            }

            Some(builder.list(ListAtom {
                element_type: well_known::TYPE_NEVER,
                known_elements: None,
                known_count: None,
                flags: U8Flags::empty(),
            }))
        }
        Atom::Array(payload) => {
            if payload.flags.contains(ArrayFlag::NonEmpty) {
                return None;
            }

            Some(well_known::EMPTY_ARRAY)
        }
        _ => None,
    }
}

#[inline]
fn narrow_string_truthy<'arena, S, A>(
    payload: &'arena StringAtom<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Atom<'arena>
where
    S: Arena,
    A: Arena,
{
    if let StringLiteral::Value(value) = payload.literal {
        let bytes = value.as_bytes();
        if bytes.is_empty() || bytes == b"0" {
            return well_known::NEVER;
        }

        return Atom::String(payload);
    }

    let mut flags = payload.flags;
    flags.set(StringRefinementFlag::Truthy);
    flags.set(StringRefinementFlag::NonEmpty);

    builder.string(StringAtom { flags, ..*payload })
}

#[inline]
fn narrow_string_falsy<'scratch, 'arena, S, A>(
    payload: &'arena StringAtom<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
    out: &mut ScratchVec<'scratch, Atom<'arena>, S>,
) where
    S: Arena,
    A: Arena,
{
    if let StringLiteral::Value(value) = payload.literal {
        let bytes = value.as_bytes();
        if bytes.is_empty() || bytes == b"0" {
            out.push(Atom::String(payload));
        }

        return;
    }

    if payload.flags.contains(StringRefinementFlag::NonEmpty) && !zero_string_compatible(*payload) {
        return;
    }

    if payload.flags.contains(StringRefinementFlag::Truthy) {
        return;
    }

    let numeric = payload.flags.contains(StringRefinementFlag::Numeric);
    if !payload.flags.contains(StringRefinementFlag::NonEmpty) {
        let empty = builder.string_literal(b"");
        if string_falsy_piece_retained(payload.casing, numeric, empty) {
            out.push(empty);
        }
    }

    if zero_string_compatible(*payload) {
        let zero = builder.string_literal(b"0");
        if string_falsy_piece_retained(payload.casing, numeric, zero) {
            out.push(zero);
        }
    }
}

/// The retain predicate for [`narrow_string_falsy`]'s candidate pieces:
/// keep a string literal only when its casing matches the source's
/// casing refinement and, for a numeric-refined source, the literal
/// parses as an integer. Non-string or non-literal pieces are kept.
#[inline]
fn string_falsy_piece_retained(casing: StringCasing, numeric: bool, piece: Atom<'_>) -> bool {
    let Atom::String(piece_payload) = piece else {
        return true;
    };
    let StringLiteral::Value(piece_value) = piece_payload.literal else {
        return true;
    };

    casing_compatible(casing, piece_value.as_bytes())
        && (!numeric || core::str::from_utf8(piece_value.as_bytes()).is_ok_and(|text| text.parse::<i64>().is_ok()))
}

#[inline]
fn zero_string_compatible(payload: StringAtom<'_>) -> bool {
    if payload.flags.contains(StringRefinementFlag::Truthy) {
        return false;
    }

    casing_compatible(payload.casing, b"0")
}

#[inline]
fn casing_compatible(casing: StringCasing, bytes: &[u8]) -> bool {
    let has_lower = bytes.iter().any(u8::is_ascii_lowercase);
    let has_upper = bytes.iter().any(u8::is_ascii_uppercase);
    match casing {
        StringCasing::Unspecified => true,
        StringCasing::Lowercase => !has_upper,
        StringCasing::Uppercase => !has_lower,
    }
}

#[inline]
fn force_non_empty<'arena, S, A>(atom: Atom<'arena>, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Atom<'arena>
where
    S: Arena,
    A: Arena,
{
    match atom {
        Atom::List(payload) => {
            let mut flags = payload.flags;
            flags.set(ListFlag::NonEmpty);

            builder.list(ListAtom { flags, ..*payload })
        }
        Atom::Array(payload) => {
            let mut flags = payload.flags;
            flags.set(ArrayFlag::NonEmpty);

            builder.array(ArrayAtom { flags, ..*payload })
        }
        _ => atom,
    }
}

#[inline]
fn family_atom_meet<'arena, S, A, W>(
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
    match (a.kind(), b.kind()) {
        (AtomKind::Int, AtomKind::Int) => family::int::int_meet(a, b, builder),
        (AtomKind::String, AtomKind::String) => family::string::string_meet(a, b, builder),
        (AtomKind::Numeric, AtomKind::String) | (AtomKind::String, AtomKind::Numeric) => {
            family::string::numeric_string_meet(a, b, builder)
        }
        (AtomKind::List, AtomKind::List) => family::array::list_meet(a, b, world, options, report, builder),
        (AtomKind::Array, AtomKind::Array) => family::array::keyed_array_meet(a, b, world, options, report, builder),
        (AtomKind::List, AtomKind::Array) | (AtomKind::Array, AtomKind::List) => {
            family::array::list_array_meet(a, b, world, options, report, builder)
        }
        (AtomKind::Iterable, AtomKind::Iterable) => {
            family::iterable::iterable_meet(a, b, world, options, report, builder)
        }
        (AtomKind::Callable, AtomKind::Callable) => {
            family::callable::callable_meet(a, b, world, options, report, builder)
        }
        (AtomKind::HasMethod, AtomKind::HasMethod) => family::has_member::has_method_meet(a, b, builder),
        (AtomKind::HasProperty, AtomKind::HasProperty) => family::has_member::has_property_meet(a, b, builder),
        (AtomKind::HasMethod, AtomKind::HasProperty) | (AtomKind::HasProperty, AtomKind::HasMethod) => {
            family::has_member::has_method_property_meet(a, b, builder)
        }
        (AtomKind::Object, AtomKind::Object) => {
            family::object::compose_object_intersection(a, b, world, options, report, builder)
        }
        (AtomKind::Object, AtomKind::HasMethod)
        | (AtomKind::Object, AtomKind::HasProperty)
        | (AtomKind::Object, AtomKind::ObjectShape) => {
            family::object::compose_object_with_structural(a, b, world, builder)
        }
        (AtomKind::HasMethod, AtomKind::Object)
        | (AtomKind::HasProperty, AtomKind::Object)
        | (AtomKind::ObjectShape, AtomKind::Object) => {
            family::object::compose_object_with_structural(b, a, world, builder)
        }
        (AtomKind::ObjectShape, AtomKind::HasMethod | AtomKind::HasProperty) => {
            family::object::compose_shape_with_structural(a, b, builder)
        }
        (AtomKind::HasMethod | AtomKind::HasProperty, AtomKind::ObjectShape) => {
            family::object::compose_shape_with_structural(b, a, builder)
        }
        (AtomKind::Iterable, AtomKind::Array) => {
            family::array::iterable_array_meet(a, b, world, options, report, builder)
        }
        (AtomKind::Array, AtomKind::Iterable) => {
            family::array::iterable_array_meet(b, a, world, options, report, builder)
        }
        (AtomKind::Iterable, AtomKind::List) => {
            family::array::iterable_list_meet(a, b, world, options, report, builder)
        }
        (AtomKind::List, AtomKind::Iterable) => {
            family::array::iterable_list_meet(b, a, world, options, report, builder)
        }
        _ => None,
    }
}
