//! `Iterable` family subtract: key/value narrowing via `Negated`
//! intersection conjuncts, mirroring [`super::list::list_minus`].

use mago_allocator::Arena;

use crate::ty::atom::Atom;
use crate::ty::builder::TypeBuilder;

/// `iterable<K1, V1> \ iterable<K2, V2>`. When key/value parameters
/// match exactly, the residue is empty. Otherwise return `a & !b` via
/// the [`Intersected`](crate::ty::atom::kind::AtomKind::Intersected)
/// wrapper.
pub(in crate::ty::subtract) fn iterable_minus<'arena, S, A>(
    input: Atom<'arena>,
    removed: Atom<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Vec<Atom<'arena>>>
where
    S: Arena,
    A: Arena,
{
    let (Atom::Iterable(input_payload), Atom::Iterable(removed_payload)) = (input, removed) else {
        return None;
    };

    if input_payload.key_type == removed_payload.key_type && input_payload.value_type == removed_payload.value_type {
        return Some(Vec::new());
    }

    let removed_type = builder.union_of(&[removed]);
    let negated = builder.negated(removed_type);
    Some(vec![builder.intersected(input, &[negated])])
}
