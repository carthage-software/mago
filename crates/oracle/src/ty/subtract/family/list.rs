//! `List` family subtract: empty-list elimination + element-type
//! narrowing via `Negated` intersection conjuncts.

use mago_allocator::Arena;
use mago_flags::U8Flags;

use crate::ty::atom::Atom;
use crate::ty::atom::payload::array::KnownElement;
use crate::ty::atom::payload::array::ListAtom;
use crate::ty::atom::payload::array::ListFlag;
use crate::ty::builder::TypeBuilder;
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;
use crate::ty::well_known::TYPE_NEVER;
use crate::world::World;

/// `list{A0, …, An} \ list{B0, …, Bn}` for two identically-shaped fixed
/// sealed lists (no tail, all positions required, matching indices and
/// non-empty flag).
///
/// A tuple is in the input but not the removed set exactly when it differs
/// from the removed shape in *at least one* position. The disjoint
/// decomposition by the *first* differing position gives one piece per
/// position `i`: positions before `i` are pinned to the intersection
/// `Aj ∩ Bj` (they agreed with the removed shape), position `i` is the
/// difference `Ai \ Bi`, and positions after `i` stay the full `Aj`. A piece
/// with any required `never` position (an empty difference at `i`, or an
/// empty intersection before it) contributes nothing and is dropped.
///
/// This is the staircase that the join's single-position merge folds back
/// together with `meet` to reconstruct the input, so subtract still
/// partitions. Returns `None` when the two lists are not the identical fixed
/// shape; the caller then keeps the input unchanged.
pub(in crate::ty::subtract) fn sealed_list_residue<'arena, S, A, W>(
    input: Atom<'arena>,
    removed: Atom<'arena>,
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Vec<Atom<'arena>>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let (Atom::List(input_payload), Atom::List(removed_payload)) = (input, removed) else {
        return None;
    };
    let (Some(input_elements), Some(removed_elements)) = (input_payload.known_elements, removed_payload.known_elements)
    else {
        return None;
    };

    if input_payload.element_type != TYPE_NEVER
        || removed_payload.element_type != TYPE_NEVER
        || input_payload.flags != removed_payload.flags
        || input_payload.known_count != removed_payload.known_count
        || input_elements.len() != removed_elements.len()
    {
        return None;
    }
    for (left, right) in input_elements.iter().zip(removed_elements) {
        if left.index != right.index || left.optional || right.optional {
            return None;
        }
    }

    let mut pieces: Vec<Atom<'arena>> = Vec::new();
    for first_out in 0..input_elements.len() {
        let mut new_elements: Vec<KnownElement<'arena>> = Vec::with_capacity(input_elements.len());
        let mut uninhabited = false;
        for (position, element) in input_elements.iter().enumerate() {
            let value = if position < first_out {
                crate::ty::meet::compute(element.value, removed_elements[position].value, world, options, report, builder)
            } else if position == first_out {
                crate::ty::subtract::compute(
                    element.value,
                    removed_elements[position].value,
                    world,
                    options,
                    report,
                    builder,
                )
            } else {
                element.value
            };

            if value.is_never() {
                uninhabited = true;
                break;
            }

            new_elements.push(KnownElement { value, ..*element });
        }

        if uninhabited {
            continue;
        }

        let known_elements = builder.known_elements(&new_elements);
        pieces.push(builder.list(ListAtom { known_elements: Some(known_elements), ..*input_payload }));
    }

    Some(pieces)
}

/// `list<E1> \ list<E2>` (or `\ non-empty-list<E2>`). The empty-list
/// singleton drops out when both sides allow empty; otherwise it
/// survives. The non-empty residue is tightened by attaching a
/// `Negated(removed)` conjunct when element types differ, so the
/// lattice can later detect contradictions like `(non-empty-list<X> &
/// Negated(list<Y>)) ∩ list<Y> ≡ ⊥` and prune the imprecise overlap.
/// When the element types are equal and only the input allows empty,
/// the non-empty values of the input are entirely covered by the
/// removed side, so only the empty piece survives.
pub(in crate::ty::subtract) fn list_minus<'arena, S, A>(
    input: Atom<'arena>,
    removed: Atom<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Vec<Atom<'arena>>>
where
    S: Arena,
    A: Arena,
{
    let (Atom::List(input_payload), Atom::List(removed_payload)) = (input, removed) else {
        return None;
    };

    if input_payload.known_elements.is_some() || removed_payload.known_elements.is_some() {
        return None;
    }

    let input_allows_empty = !input_payload.flags.contains(ListFlag::NonEmpty);
    let removed_allows_empty = !removed_payload.flags.contains(ListFlag::NonEmpty);

    let mut pieces: Vec<Atom<'arena>> = Vec::new();

    if input_allows_empty && !removed_allows_empty {
        pieces.push(empty_list(builder));
    }

    let non_empty_residue = ListAtom { flags: input_payload.flags.with(ListFlag::NonEmpty), ..*input_payload };
    if input_payload.element_type != removed_payload.element_type {
        let removed_type = builder.union_of(&[removed]);
        let negated = builder.negated(removed_type);
        let head = builder.list(non_empty_residue);
        pieces.push(builder.intersected(head, &[negated]));
    } else if input_allows_empty == removed_allows_empty {
        pieces.push(builder.list(non_empty_residue));
    }

    Some(pieces)
}

#[inline]
fn empty_list<'arena, S, A>(builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Atom<'arena>
where
    S: Arena,
    A: Arena,
{
    builder.list(ListAtom {
        element_type: TYPE_NEVER,
        known_elements: None,
        known_count: None,
        flags: U8Flags::empty(),
    })
}

/// `list<E> \ iterable<K, V>`: every iterable accepts the empty
/// iterator, so when the input allows the empty list it sits in the
/// removed side and gets removed. Element-type narrowing on the
/// non-empty pieces is captured by attaching a `Negated(iterable<K, V>)`
/// conjunct, the same way [`list_minus`] does for list-vs-list.
pub(in crate::ty::subtract) fn list_minus_iterable<'arena, S, A>(
    input: Atom<'arena>,
    removed: Atom<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Vec<Atom<'arena>>>
where
    S: Arena,
    A: Arena,
{
    let Atom::List(input_payload) = input else {
        return None;
    };

    if input_payload.known_elements.is_some() {
        return None;
    }

    let head = builder.list(ListAtom { flags: input_payload.flags.with(ListFlag::NonEmpty), ..*input_payload });
    let removed_type = builder.union_of(&[removed]);
    let negated = builder.negated(removed_type);
    Some(vec![builder.intersected(head, &[negated])])
}
