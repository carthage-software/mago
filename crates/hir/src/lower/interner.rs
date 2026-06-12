use mago_allocator::Arena;
use mago_allocator::collections::HashSet;

#[derive(Debug)]
pub(crate) struct Interner<'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    arena: &'arena A,
    names: HashSet<'scratch, &'arena [u8], S>,
}

impl<'scratch, 'arena, S, A> Interner<'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn new(arena: &'arena A, scratch: &'scratch S) -> Interner<'scratch, 'arena, S, A> {
        Interner { arena, names: HashSet::new_in(scratch) }
    }

    pub(crate) fn intern(&mut self, bytes: &[u8]) -> &'arena [u8] {
        match self.names.get(bytes) {
            Some(interned) => interned,
            None => {
                let interned = &*self.arena.alloc_slice_copy(bytes);
                self.names.insert(interned);

                interned
            }
        }
    }
}
