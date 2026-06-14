//! `Iterable` family subtract: key/value narrowing via `Negated`
//! intersection conjuncts, mirroring [`super::list::list_minus`].

use mago_allocator::Arena;
use mago_allocator::vec::Vec as ScratchVec;

use crate::ty::atom::Atom;
use crate::ty::builder::TypeBuilder;

/// `iterable<K1, V1> \ iterable<K2, V2>`. When key/value parameters
/// match exactly, the residue is empty. Otherwise return `a & !b` via
/// the [`Intersected`](crate::ty::atom::kind::AtomKind::Intersected)
/// wrapper.
pub(in crate::ty::subtract) fn iterable_minus<'scratch, 'arena, S, A>(
    input: Atom<'arena>,
    removed: Atom<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
    out: &mut ScratchVec<'scratch, Atom<'arena>, S>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let (Atom::Iterable(input_payload), Atom::Iterable(removed_payload)) = (input, removed) else {
        return false;
    };

    if input_payload.key_type == removed_payload.key_type && input_payload.value_type == removed_payload.value_type {
        return true;
    }

    let removed_type = builder.union_of(&[removed]);
    let negated = builder.negated(removed_type);
    let intersected = builder.intersected(input, &[negated]);
    out.push(intersected);
    true
}
