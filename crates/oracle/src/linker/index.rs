use mago_allocator::Arena;
use mago_allocator::vec::Vec;

use crate::id::SymbolId;

/// Builds the `target.id`-sorted offset index that the member and inheritance
/// lists binary-search. `key(&entries[i])` yields the [`SymbolId`] entry `i` is
/// ordered by; the returned slice holds the offsets `0..entries.len()` sorted by
/// that key. The buffer is built directly in the output arena, so there is no
/// intermediate heap allocation.
pub(crate) fn sorted_offsets<'arena, A, T>(
    arena: &'arena A,
    entries: &[T],
    key: impl Fn(&T) -> SymbolId,
) -> &'arena [u32]
where
    A: Arena,
{
    let mut offsets = Vec::with_capacity_in(entries.len(), arena);
    offsets.extend(0..entries.len() as u32);
    offsets.sort_by_key(|&offset| key(&entries[offset as usize]));

    offsets.leak()
}

/// Sorts `ids` ascending, de-duplicates them, and copies the result into the
/// output arena - the shape the descendant and sealed-parent slices require for
/// binary-search membership. `ids` is a scratch-allocated buffer, so the only
/// lasting allocation is the returned slice.
pub(crate) fn sorted_unique_ids<'arena, A, B>(arena: &'arena A, mut ids: Vec<'_, SymbolId, B>) -> &'arena [SymbolId]
where
    A: Arena,
    B: Arena,
{
    ids.sort_unstable();
    ids.dedup();

    arena.alloc_slice_copy(&ids)
}
