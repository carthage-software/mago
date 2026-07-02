//! `HasMethod` / `HasProperty` family subtract: equal-name pairs
//! collapse to bottom; otherwise return `a & !b` via the
//! [`Intersected`](crate::ty::atom::kind::AtomKind::Intersected) wrapper.

use mago_allocator::Arena;
use mago_allocator::vec::Vec as ScratchVec;

use crate::ty::atom::Atom;
use crate::ty::builder::TypeBuilder;

pub(in crate::ty::subtract) fn has_method_minus<'scratch, 'arena, S, A>(
    input: Atom<'arena>,
    removed: Atom<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
    out: &mut ScratchVec<'scratch, Atom<'arena>, S>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let (Atom::HasMethod(input_payload), Atom::HasMethod(removed_payload)) = (input, removed) else {
        return false;
    };

    if input_payload.method_name == removed_payload.method_name {
        return true;
    }

    let removed_type = builder.union_of(&[removed]);
    let negated = builder.negated(removed_type);
    let intersected = builder.intersected(input, &[negated]);
    out.push(intersected);
    true
}

pub(in crate::ty::subtract) fn has_property_minus<'scratch, 'arena, S, A>(
    input: Atom<'arena>,
    removed: Atom<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
    out: &mut ScratchVec<'scratch, Atom<'arena>, S>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let (Atom::HasProperty(input_payload), Atom::HasProperty(removed_payload)) = (input, removed) else {
        return false;
    };

    if input_payload.property_name == removed_payload.property_name {
        return true;
    }

    let removed_type = builder.union_of(&[removed]);
    let negated = builder.negated(removed_type);
    let intersected = builder.intersected(input, &[negated]);
    out.push(intersected);
    true
}
