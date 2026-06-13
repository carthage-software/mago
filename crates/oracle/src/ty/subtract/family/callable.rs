//! `Callable` family subtract: equal callables collapse to bottom;
//! otherwise wrap as `Intersected(a, [Negated(b)])` so the narrowing
//! survives.

use mago_allocator::Arena;

use crate::ty::atom::Atom;
use crate::ty::builder::TypeBuilder;

pub(in crate::ty::subtract) fn callable_minus<'arena, S, A>(
    input: Atom<'arena>,
    removed: Atom<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Vec<Atom<'arena>>>
where
    S: Arena,
    A: Arena,
{
    if input == removed {
        return Some(Vec::new());
    }

    let removed_type = builder.union_of(&[removed]);
    let negated = builder.negated(removed_type);
    Some(vec![builder.intersected(input, &[negated])])
}
