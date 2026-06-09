//! Collecting iterators directly into an arena.

use core::hash::BuildHasher;
use core::hash::Hash;

use crate::Arena;
use crate::collections::HashMap;
use crate::collections::HashSet;
use crate::vec::Vec;

/// Construct a collection from an iterator, allocating its storage in an arena.
///
/// The arena-equivalent of [`FromIterator`]: the implementing collection decides
/// what allocator handle it needs via [`Alloc`](FromIteratorIn::Alloc).
pub trait FromIteratorIn<T> {
    /// The allocator handle the collection is built into (e.g. `&'arena A`).
    type Alloc;

    /// Builds `Self` from `iter`, allocating into `alloc`.
    fn from_iter_in<I>(iter: I, alloc: Self::Alloc) -> Self
    where
        I: IntoIterator<Item = T>;
}

impl<'arena, T, A: Arena> FromIteratorIn<T> for Vec<'arena, T, A> {
    type Alloc = &'arena A;

    #[inline]
    fn from_iter_in<I>(iter: I, alloc: Self::Alloc) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut collection = Self::new_in(alloc);
        collection.extend(iter);
        collection
    }
}

impl<'arena, K, V, A, S> FromIteratorIn<(K, V)> for HashMap<'arena, K, V, A, S>
where
    K: Eq + Hash,
    S: BuildHasher + Default,
    A: Arena,
{
    type Alloc = &'arena A;

    #[inline]
    fn from_iter_in<I>(iter: I, alloc: Self::Alloc) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let mut collection = Self::with_hasher_in(S::default(), alloc);
        collection.extend(iter);
        collection
    }
}

impl<'arena, T, A, S> FromIteratorIn<T> for HashSet<'arena, T, A, S>
where
    T: Eq + Hash,
    S: BuildHasher + Default,
    A: Arena,
{
    type Alloc = &'arena A;

    #[inline]
    fn from_iter_in<I>(iter: I, alloc: Self::Alloc) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut collection = Self::with_hasher_in(S::default(), alloc);
        collection.extend(iter);
        collection
    }
}

/// Collect an iterator into an arena-allocated collection.
///
/// The arena-equivalent of [`Iterator::collect`]:
///
/// ```ignore
/// let evens: Vec<i32, A> = (0..10).filter(|n| n % 2 == 0).collect_in(arena);
/// ```
pub trait CollectIn: Iterator + Sized {
    /// Collects this iterator into `C`, allocating its storage in `alloc`.
    #[inline]
    fn collect_in<C>(self, alloc: C::Alloc) -> C
    where
        C: FromIteratorIn<Self::Item>,
    {
        C::from_iter_in(self, alloc)
    }
}

impl<I: Iterator> CollectIn for I {}

#[cfg(feature = "rayon")]
pub use parallel::ParallelCollectIn;

#[cfg(feature = "rayon")]
mod parallel {
    use rayon::iter::IndexedParallelIterator;
    use rayon::iter::IntoParallelRefMutIterator;
    use rayon::iter::ParallelIterator;

    use crate::Arena;
    use crate::vec::Vec;

    /// Collect an indexed parallel iterator into an arena-allocated [`Vec`], with
    /// zero global allocations.
    ///
    /// The destination is allocated once, in the arena, at the iterator's exact
    /// length; each worker writes its element directly into its uninitialized slot.
    /// There is no intermediate buffer.
    pub trait ParallelCollectIn: IndexedParallelIterator
    where
        Self::Item: Send,
    {
        /// Collects this parallel iterator into an arena-allocated [`Vec`].
        fn collect_in<A>(self, arena: &A) -> Vec<'_, Self::Item, A>
        where
            A: Arena + Sync,
        {
            let length = self.len();
            let mut collection = Vec::with_capacity_in(length, arena);

            collection.spare_capacity_mut()[..length].par_iter_mut().zip(self).for_each(|(slot, item)| {
                slot.write(item);
            });

            // SAFETY: every one of the first `length` slots was initialized above.
            unsafe { collection.set_len(length) };

            collection
        }
    }

    impl<I: IndexedParallelIterator> ParallelCollectIn for I where I::Item: Send {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::LocalArena;

    #[test]
    fn collect_in_vec() {
        let arena = LocalArena::new();
        let collected: Vec<'_, i32, LocalArena> = (0..5).collect_in(&arena);

        assert_eq!(collected.as_slice(), &[0, 1, 2, 3, 4]);
    }

    #[test]
    fn collect_in_hashset_deduplicates() {
        let arena = LocalArena::new();
        let collected: HashSet<'_, i32, LocalArena> = [1, 2, 2, 3].into_iter().collect_in(&arena);

        assert_eq!(collected.len(), 3);
        assert!(collected.contains(&2));
    }

    #[test]
    fn collect_in_hashmap() {
        let arena = LocalArena::new();
        let collected: HashMap<'_, i32, i32, LocalArena> = [(1, 10), (2, 20)].into_iter().collect_in(&arena);

        assert_eq!(collected.get(&2), Some(&20));
    }
}
