//! Lattice difference: `A \ B` is the type whose values are in `A` but not in `B`.
//!
//! Pairs with [`crate::ty::meet`] the way negative narrowing pairs with
//! positive narrowing: `if ($x !== null)` produces `subtract(T_x, null)`.
//!
//! Two entry points:
//!
//! - [`narrow`] is the primary operation. It runs the difference and
//!   classifies the result for assertion-driven narrowing:
//!   `Impossible` when `input ⊆ σ` (the negation can never hold),
//!   `Redundant` when `input # σ` (the negation is trivially true and
//!   adds nothing), `Narrowed` when the result is strictly smaller.
//! - [`compute`] is a thin wrapper that returns just the resulting
//!   [`Type`], mapping `Impossible` to [`well_known::TYPE_NEVER`].
//!
//! The operation is *partial*: when no rule
//! describes the precise difference, the input is returned unchanged.
//! Returning a superset of the true difference is sound; the
//! soundness invariants are
//!
//! - `result <: A` (no value escapes the original),
//! - `result ∧ B ≡ ⊥` *if precise*, `result ⊇ A \ B` always.
//!
//! # Strategy
//!
//! Difference distributes over union on the left and intersects with the
//! complement on the right:
//!
//! ```text
//! (α ∨ β) \ γ  ≡  (α \ γ) ∨ (β \ γ)
//! α \ (β ∨ γ)  ≡  (α \ β) \ γ  ≡  (α \ γ) \ β
//! ```
//!
//! So for each atom in `A` we fold over the atoms in `B`, subtracting
//! one at a time and accumulating the surviving pieces.
//!
//! Atom-pair difference walks these rules in order:
//!
//! 1. `α <: β` ⇒ `⊥` (every `α`-value is a `β`-value).
//! 2. `α # β` (disjoint) ⇒ `α` (subtraction is identity).
//! 3. Family-specific positive rule (e.g. integer-range split).
//! 4. Otherwise return `α` unchanged (conservative fallback).

mod family;

use mago_allocator::Arena;

use crate::ty::Type;
use crate::ty::atom::Atom;
use crate::ty::atom::kind::AtomKind;
use crate::ty::atom::payload::array::ArrayAtom;
use crate::ty::atom::payload::array::ArrayFlag;
use crate::ty::atom::payload::array::ListAtom;
use crate::ty::atom::payload::array::ListFlag;
use crate::ty::builder::TypeBuilder;
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;
use crate::ty::lattice::is_uninhabited;
use crate::ty::lattice::overlaps;
use crate::ty::lattice::refines;
use crate::ty::lattice::sealed;
use crate::ty::lattice::sealed::SealedResidual;
use crate::ty::meet;
use crate::ty::subtract::family::array;
use crate::ty::subtract::family::callable;
use crate::ty::subtract::family::dominator;
use crate::ty::subtract::family::generic;
use crate::ty::subtract::family::has_member;
use crate::ty::subtract::family::int;
use crate::ty::subtract::family::iterable;
use crate::ty::subtract::family::list;
use crate::ty::subtract::family::object;
use crate::ty::subtract::family::string;
use crate::ty::well_known;
use crate::ty::well_known::BOOL;
use crate::ty::well_known::FALSE;
use crate::ty::well_known::MIXED;
use crate::ty::well_known::NEVER;
use crate::ty::well_known::NON_NULL_MIXED;
use crate::ty::well_known::NULL;
use crate::ty::well_known::TRUE;
use crate::world::World;

/// Outcome of [`narrow`], classifying an assertion-driven difference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SubtractOutcome<'arena> {
    /// `input ⊆ σ`: every value of the input also satisfies the
    /// predicate being negated, so the negative assertion can never
    /// hold. The result is `never`.
    Impossible,
    /// `input # σ` (already disjoint): the input has no values in
    /// common with the predicate, so the negation is trivially true
    /// and adds no information. Carries the (unchanged) input.
    Redundant(Type<'arena>),
    /// The subtraction strictly narrowed the input. Carries the new
    /// type.
    Narrowed(Type<'arena>),
}

impl<'arena> SubtractOutcome<'arena> {
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

/// Compute `input \ narrowing` and classify the outcome for
/// assertion-driven diagnostics.
///
/// `input` is the existing type; `narrowing` is the type the negative
/// assertion is removing (e.g. the right-hand side of
/// `!($x instanceof Foo)`).
///
/// `result <: input` always; `result ∧ narrowing ≡ ⊥` when the family
/// rules cover every surviving atom precisely.
#[inline]
pub fn narrow<'arena, S, A, W>(
    input: Type<'arena>,
    narrowing: Type<'arena>,
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> SubtractOutcome<'arena>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    if input == narrowing {
        return SubtractOutcome::Impossible;
    }

    let mut atoms: Vec<Atom<'arena>> = Vec::with_capacity(input.atoms.len());

    let mut current_scratch: Vec<Atom<'arena>> = Vec::new();
    let mut next_scratch: Vec<Atom<'arena>> = Vec::new();
    for &atom in input.atoms {
        let pieces =
            subtract_all(atom, narrowing, world, options, report, builder, &mut current_scratch, &mut next_scratch);

        atoms.extend(pieces.iter().copied());
    }

    if atoms.is_empty() {
        return SubtractOutcome::Impossible;
    }

    canonicalise_sealed_residuals(&mut atoms, world, options, report, builder);

    let result = builder.union_of(&atoms);

    if result == input { SubtractOutcome::Redundant(input) } else { SubtractOutcome::Narrowed(result) }
}

#[inline]
fn canonicalise_sealed_residuals<'arena, S, A, W>(
    atoms: &mut Vec<Atom<'arena>>,
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    if atoms.is_empty() {
        return;
    }

    let mut index = 0;
    while index < atoms.len() {
        let atom = atoms[index];
        let Atom::Intersected(payload) = atom else {
            index += 1;
            continue;
        };

        let conjuncts = payload.conjuncts;
        let head = *payload.head;
        let head_type = if matches!(head, Atom::Object(_)) { Some(builder.union_of(&[head])) } else { None };

        let mut kept: Vec<Atom<'arena>> = Vec::with_capacity(conjuncts.len());
        let mut negated_inners: Vec<Type<'arena>> = Vec::with_capacity(conjuncts.len());
        for &conjunct in conjuncts {
            if let Atom::Negated(inner) = conjunct {
                if let Some(head_type) = head_type
                    && !overlaps(head_type, *inner, world, options, report, builder)
                {
                    continue;
                }
                negated_inners.push(*inner);
            }
            kept.push(conjunct);
        }

        if kept.is_empty() {
            atoms[index] = head;
            index += 1;
            continue;
        }

        if matches!(head, Atom::Object(_)) && !negated_inners.is_empty() {
            let residual = sealed::compute_residual(head, &negated_inners, world, options, report, builder);
            match residual {
                SealedResidual::FullyCovered => {
                    atoms.remove(index);
                    continue;
                }
                SealedResidual::Surviving(survivors) => {
                    if let [survivor] = survivors.as_slice() {
                        let survivor = *survivor;
                        let survivor_type = builder.union_of(&[survivor]);
                        let mut residual_conjuncts: Vec<Atom<'arena>> = Vec::with_capacity(kept.len());
                        for &conjunct in &kept {
                            let still_applies = match conjunct {
                                Atom::Negated(inner) => {
                                    overlaps(survivor_type, *inner, world, options, report, builder)
                                }
                                _ => true,
                            };

                            if still_applies {
                                residual_conjuncts.push(conjunct);
                            }
                        }

                        atoms[index] = if residual_conjuncts.is_empty() {
                            survivor
                        } else {
                            builder.intersected(survivor, &residual_conjuncts)
                        };

                        index += 1;
                        continue;
                    }
                }
                SealedResidual::NotSealed => {}
            }
        }

        if kept.len() != conjuncts.len() {
            atoms[index] = builder.intersected(head, &kept);
        }
        index += 1;
    }
}

/// Compute `input \ removed`: the largest representable type whose
/// values are in `input` but not in `removed`. Thin wrapper over
/// [`narrow`] for callers that don't need the assertion classification.
#[inline]
pub fn compute<'arena, S, A, W>(
    input: Type<'arena>,
    removed: Type<'arena>,
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
    narrow(input, removed, world, options, report, builder).into_type()
}

/// Apply `α \ β₁ \ β₂ \ … \ βₙ` by folding over the atoms of
/// `narrowing`, then drain to empty when the surviving atoms refine the
/// full `narrowing` union (the per-atom fold sees one container at a
/// time and can stall on partition-style coverage).
///
/// `current_scratch` and `next_scratch` are reused per call; the
/// function clears and refills them, returning a borrowed view into
/// `current_scratch` for the caller to copy out.
#[inline]
fn subtract_all<'scratch, 'arena, S, A, W>(
    atom: Atom<'arena>,
    narrowing: Type<'arena>,
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    current_scratch: &'scratch mut Vec<Atom<'arena>>,
    next_scratch: &mut Vec<Atom<'arena>>,
) -> &'scratch [Atom<'arena>]
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    current_scratch.clear();
    current_scratch.push(atom);
    for &removed in narrowing.atoms {
        if current_scratch.is_empty() {
            break;
        }

        next_scratch.clear();
        for &surviving in current_scratch.iter() {
            next_scratch.extend(atom_minus(surviving, removed, world, options, report, builder));
        }

        core::mem::swap(current_scratch, next_scratch);
    }

    if !current_scratch.is_empty() {
        let current_type = builder.union_of(current_scratch);
        if refines(current_type, narrowing, world, options, report, builder) {
            current_scratch.clear();
        }
    }

    current_scratch.as_slice()
}

/// One step of the atom-pair rule walk described in the
/// [module documentation](self). Negation routes through the duality
/// with the meet: `subtract(X, !T)` ≡ `meet(X, T)` and
/// `subtract(!T, X)` ≡ `!(T ∪ X)`.
pub(in crate::ty::subtract) fn atom_minus<'arena, S, A, W>(
    input: Atom<'arena>,
    removed: Atom<'arena>,
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Vec<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    if input == removed || input == NEVER {
        return Vec::new();
    }

    if removed == NEVER {
        return vec![input];
    }

    if removed == MIXED {
        return Vec::new();
    }

    if is_uninhabited(removed, world, builder) {
        return vec![input];
    }

    if is_uninhabited(input, world, builder) {
        return Vec::new();
    }

    if let Atom::Intersected(payload) = input {
        let input_type = builder.union_of(&[input]);
        let removed_type = builder.union_of(&[removed]);
        if refines(input_type, removed_type, world, options, report, builder) {
            return Vec::new();
        }

        if !overlaps(input_type, removed_type, world, options, report, builder) {
            return vec![input];
        }

        let head_pieces = atom_minus(*payload.head, removed, world, options, report, builder);

        return head_pieces.into_iter().map(|head| builder.intersected(head, payload.conjuncts)).collect();
    }

    if let Atom::Negated(inner) = removed {
        let input_type = builder.union_of(&[input]);
        let kept = meet::compute(input_type, *inner, world, options, report, builder);

        return kept.atoms.to_vec();
    }

    if let Atom::Negated(inner) = input {
        let mut union_atoms: Vec<Atom<'arena>> = inner.atoms.to_vec();
        union_atoms.push(removed);
        let union_type = builder.union_of(&union_atoms);
        return vec![builder.negated(union_type)];
    }

    if input == MIXED {
        let removed_type = builder.union_of(&[removed]);
        return vec![builder.negated(removed_type)];
    }

    if input == NON_NULL_MIXED {
        let union_type = builder.union_of(&[NULL, removed]);
        return vec![builder.negated(union_type)];
    }

    if matches!(input, Atom::ClassLikeString(_)) && matches!(removed, Atom::ClassLikeString(_)) && input != removed {
        let removed_type = builder.union_of(&[removed]);
        let negated = builder.negated(removed_type);
        return vec![builder.intersected(input, &[negated])];
    }

    if let Some(pieces) = list::sealed_list_residue(input, removed, world, options, report, builder) {
        return pieces;
    }

    let input_type = builder.union_of(&[input]);
    let removed_type = builder.union_of(&[removed]);

    if refines(input_type, removed_type, world, options, report, builder) {
        return Vec::new();
    }

    if !overlaps(input_type, removed_type, world, options, report, builder) {
        return vec![input];
    }

    if matches!(input, Atom::GenericParameter(_)) {
        return generic::generic_parameter_minus(input, removed, world, options, report, builder)
            .unwrap_or_else(|| vec![input]);
    }

    if let Some(pieces) = dominator::true_union_minus(input, removed, world, options, report, builder) {
        return pieces;
    }

    if let Some(pieces) = object::object_descendant_minus(input, removed, world, builder) {
        return pieces;
    }

    if let Some(pieces) = family_atom_minus(input, removed, builder) {
        return pieces;
    }

    if matches!(removed, Atom::Intersected(_)) {
        return vec![input];
    }

    if scalar_supports_intersected_subtract(input.kind()) {
        let negated = builder.negated(removed_type);
        return vec![builder.intersected(input, &[negated])];
    }

    vec![input]
}

#[inline]
const fn scalar_supports_intersected_subtract(kind: AtomKind) -> bool {
    matches!(
        kind,
        AtomKind::String
            | AtomKind::Int
            | AtomKind::Float
            | AtomKind::Bool
            | AtomKind::Numeric
            | AtomKind::ArrayKey
            | AtomKind::ClassLikeString
            | AtomKind::Resource
            | AtomKind::Scalar
            | AtomKind::Enum
    )
}

#[inline]
fn family_atom_minus<'arena, S, A>(
    input: Atom<'arena>,
    removed: Atom<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Vec<Atom<'arena>>>
where
    S: Arena,
    A: Arena,
{
    if matches!(input, Atom::Int(_)) && matches!(removed, Atom::Int(_)) {
        return Some(int::int_minus(input, removed, builder));
    }

    if input == BOOL && removed == TRUE {
        return Some(vec![FALSE]);
    }

    if input == BOOL && removed == FALSE {
        return Some(vec![TRUE]);
    }

    if matches!(input, Atom::String(_)) && matches!(removed, Atom::String(_)) {
        return string::string_minus(input, removed, builder);
    }

    if matches!(input, Atom::List(_)) && matches!(removed, Atom::List(_)) {
        return list::list_minus(input, removed, builder);
    }

    if matches!(input, Atom::Array(_)) && matches!(removed, Atom::Array(_)) {
        return array::array_minus(input, removed, builder);
    }

    if matches!(input, Atom::List(_)) && matches!(removed, Atom::Iterable(_)) {
        return list::list_minus_iterable(input, removed, builder);
    }

    if matches!(input, Atom::Array(_)) && matches!(removed, Atom::Iterable(_)) {
        return array::array_minus_iterable(input, removed, builder);
    }

    if let (Atom::List(input_payload), Atom::Array(removed_payload)) = (input, removed) {
        let removed_allows_empty = !removed_payload.flags.contains(ArrayFlag::NonEmpty);
        let head = if removed_allows_empty {
            builder.list(ListAtom { flags: input_payload.flags.with(ListFlag::NonEmpty), ..*input_payload })
        } else {
            input
        };

        let removed_type = builder.union_of(&[removed]);
        let negated = builder.negated(removed_type);
        return Some(vec![builder.intersected(head, &[negated])]);
    }

    if let (Atom::Array(input_payload), Atom::List(removed_payload)) = (input, removed) {
        let removed_allows_empty = !removed_payload.flags.contains(ListFlag::NonEmpty);
        let head = if removed_allows_empty {
            builder.array(ArrayAtom { flags: input_payload.flags.with(ArrayFlag::NonEmpty), ..*input_payload })
        } else {
            input
        };

        let removed_type = builder.union_of(&[removed]);
        let negated = builder.negated(removed_type);
        return Some(vec![builder.intersected(head, &[negated])]);
    }

    if matches!(input, Atom::Iterable(_)) && matches!(removed, Atom::Iterable(_)) {
        return iterable::iterable_minus(input, removed, builder);
    }

    if matches!(input, Atom::Callable(_)) && matches!(removed, Atom::Callable(_)) {
        return callable::callable_minus(input, removed, builder);
    }

    if matches!(input, Atom::HasMethod(_)) && matches!(removed, Atom::HasMethod(_)) {
        return has_member::has_method_minus(input, removed, builder);
    }

    if matches!(input, Atom::HasProperty(_)) && matches!(removed, Atom::HasProperty(_)) {
        return has_member::has_property_minus(input, removed, builder);
    }

    None
}
