//! `HasMethod` / `HasProperty` family subtract: equal-name pairs
//! collapse to bottom; otherwise return `a & !b` via the
//! [`Intersected`](crate::ty::atom::kind::AtomKind::Intersected) wrapper.

use mago_allocator::Arena;

use crate::ty::atom::Atom;
use crate::ty::builder::TypeBuilder;

pub(in crate::ty::subtract) fn has_method_minus<'arena, S, A>(
    input: Atom<'arena>,
    removed: Atom<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Vec<Atom<'arena>>>
where
    S: Arena,
    A: Arena,
{
    let (Atom::HasMethod(input_payload), Atom::HasMethod(removed_payload)) = (input, removed) else {
        return None;
    };

    if input_payload.method_name == removed_payload.method_name {
        return Some(Vec::new());
    }

    let removed_type = builder.union_of(&[removed]);
    let negated = builder.negated(removed_type);
    Some(vec![builder.intersected(input, &[negated])])
}

pub(in crate::ty::subtract) fn has_property_minus<'arena, S, A>(
    input: Atom<'arena>,
    removed: Atom<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Vec<Atom<'arena>>>
where
    S: Arena,
    A: Arena,
{
    let (Atom::HasProperty(input_payload), Atom::HasProperty(removed_payload)) = (input, removed) else {
        return None;
    };

    if input_payload.property_name == removed_payload.property_name {
        return Some(Vec::new());
    }

    let removed_type = builder.union_of(&[removed]);
    let negated = builder.negated(removed_type);
    Some(vec![builder.intersected(input, &[negated])])
}
