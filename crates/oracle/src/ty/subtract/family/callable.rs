//! `Callable` family subtract: equal callables collapse to bottom;
//! otherwise wrap as `Intersected(a, [Negated(b)])` so the narrowing
//! survives.

use mago_allocator::Arena;
use mago_allocator::vec::Vec as ScratchVec;

use crate::ty::atom::Atom;
use crate::ty::builder::TypeBuilder;

pub(in crate::ty::subtract) fn callable_minus<'scratch, 'arena, S, A>(
    input: Atom<'arena>,
    removed: Atom<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
    out: &mut ScratchVec<'scratch, Atom<'arena>, S>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    if input == removed {
        return true;
    }

    let removed_type = builder.union_of(&[removed]);
    let negated = builder.negated(removed_type);
    let intersected = builder.intersected(input, &[negated]);
    out.push(intersected);
    true
}
