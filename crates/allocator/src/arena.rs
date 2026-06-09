//! The bump arenas and the [`Arena`] allocation trait.

use core::alloc::Layout;
use core::fmt;
use core::ptr::NonNull;

use crate::alloc::AllocError;
use crate::alloc::Allocator;
use crate::alloc::Global;
use crate::boxed::Box;
use crate::vec::Vec;

/// A single-threaded bump arena.
///
/// Fastest of the three; `Send` but not `Sync`. Use it for per-task scratch and
/// any single-threaded pass.
pub struct LocalArena<A: Allocator = Global>(blink_alloc::BlinkAlloc<A>);

/// A thread-safe bump arena (`Send + Sync`).
///
/// Share `&SharedArena` across threads (for example, one arena for a whole
/// parallel compilation); every worker allocates into it and the results live for
/// the shared borrow. For contention-free per-worker scratch, take a
/// [`ScopedArena`] with [`scoped`](SharedArena::scoped).
pub struct SharedArena<A: Allocator = Global>(blink_alloc::SyncBlinkAlloc<A>);

/// A thread-local view into a [`SharedArena`], handed out by
/// [`SharedArena::scoped`].
///
/// It draws blocks from its parent [`SharedArena`] and hands out allocations
/// locally, without the atomic contention of allocating through the shared arena
/// directly. Allocations are tied to the scope's borrow rather than to the parent
/// arena, so a `ScopedArena` is for **scratch that does not escape the scope**;
/// anything that must outlive the scope should be allocated into the
/// [`SharedArena`] itself.
pub struct ScopedArena<'shared, A: Allocator = Global>(blink_alloc::LocalBlinkAlloc<'shared, A>);

impl LocalArena<Global> {
    /// Creates new arena allocator that uses global allocator
    /// to allocate memory chunks.
    ///
    /// See [`LocalArena::new_in`] for using custom allocator.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self::new_in(Global)
    }

    /// Creates new arena allocator that uses global allocator
    /// to allocate memory chunks.
    /// With this method you can specify initial chunk size.
    ///
    /// See [`LocalArena::new_in`] for using custom allocator.
    #[inline]
    #[must_use]
    pub const fn with_chunk_size(chunk_size: usize) -> Self {
        Self::with_chunk_size_in(chunk_size, Global)
    }
}

impl<A: Allocator> LocalArena<A> {
    /// Creates a new arena that uses `arena` to allocate its memory chunks.
    #[inline]
    #[must_use]
    pub const fn new_in(arena: A) -> Self {
        Self(blink_alloc::BlinkAlloc::new_in(arena))
    }

    /// Creates a new, empty arena whose chunks are at least `chunk_size` bytes.
    #[inline]
    #[must_use]
    pub const fn with_chunk_size_in(chunk_size: usize, arena: A) -> Self {
        Self(blink_alloc::BlinkAlloc::with_chunk_size_in(chunk_size, arena))
    }

    /// Frees every allocation at once, keeping the arena's chunks for reuse.
    #[inline]
    pub fn reset(&mut self) {
        self.0.reset();
    }
}

impl Default for LocalArena {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl SharedArena<Global> {
    /// Creates new arena allocator that uses global allocator
    /// to allocate memory chunks.
    ///
    /// See [`SharedArena::new_in`] for using custom allocator.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self::new_in(Global)
    }

    /// Creates new arena allocator that uses global allocator
    /// to allocate memory chunks.
    /// With this method you can specify initial chunk size.
    ///
    /// See [`SharedArena::new_in`] for using custom allocator.
    #[inline]
    #[must_use]
    pub const fn with_chunk_size(chunk_size: usize) -> Self {
        Self::with_chunk_size_in(chunk_size, Global)
    }
}

impl<A: Allocator> SharedArena<A> {
    /// Creates new arena allocator that uses the given allocator
    /// to allocate memory chunks.
    #[inline]
    #[must_use]
    pub const fn new_in(arena: A) -> Self {
        Self(blink_alloc::SyncBlinkAlloc::new_in(arena))
    }

    /// Creates a new, empty arena whose chunks are at least `chunk_size` bytes.
    #[inline]
    #[must_use]
    pub const fn with_chunk_size_in(chunk_size: usize, arena: A) -> Self {
        Self(blink_alloc::SyncBlinkAlloc::with_chunk_size_in(chunk_size, arena))
    }

    /// Returns a thread-local [`ScopedArena`] drawing from this arena.
    ///
    /// Each worker in a parallel pass can take its own scope and allocate scratch
    /// without contending on the shared bump pointer.
    #[inline]
    pub fn scoped(&self) -> ScopedArena<'_, A> {
        ScopedArena(self.0.local())
    }

    /// Frees every allocation at once, keeping the arena's chunks for reuse.
    #[inline]
    pub fn reset(&mut self) {
        self.0.reset();
    }
}

impl Default for SharedArena {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Allocator> ScopedArena<'_, A> {
    /// Frees the scope's allocations, returning unused blocks to the parent arena.
    #[inline]
    pub fn reset(&mut self) {
        self.0.reset();
    }
}

macro_rules! forward_allocator {
    () => {
        #[inline]
        fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
            Allocator::allocate(&self.0, layout)
        }

        #[inline]
        fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
            Allocator::allocate_zeroed(&self.0, layout)
        }

        #[inline]
        unsafe fn deallocate(&self, pointer: NonNull<u8>, layout: Layout) {
            // SAFETY: the caller upholds `deallocate`'s contract; forwarded verbatim.
            unsafe { Allocator::deallocate(&self.0, pointer, layout) }
        }

        #[inline]
        unsafe fn grow(
            &self,
            pointer: NonNull<u8>,
            old_layout: Layout,
            new_layout: Layout,
        ) -> Result<NonNull<[u8]>, AllocError> {
            // SAFETY: the caller upholds `grow`'s contract; forwarded verbatim.
            unsafe { Allocator::grow(&self.0, pointer, old_layout, new_layout) }
        }

        #[inline]
        unsafe fn grow_zeroed(
            &self,
            pointer: NonNull<u8>,
            old_layout: Layout,
            new_layout: Layout,
        ) -> Result<NonNull<[u8]>, AllocError> {
            // SAFETY: the caller upholds `grow_zeroed`'s contract; forwarded verbatim.
            unsafe { Allocator::grow_zeroed(&self.0, pointer, old_layout, new_layout) }
        }

        #[inline]
        unsafe fn shrink(
            &self,
            pointer: NonNull<u8>,
            old_layout: Layout,
            new_layout: Layout,
        ) -> Result<NonNull<[u8]>, AllocError> {
            // SAFETY: the caller upholds `shrink`'s contract; forwarded verbatim.
            unsafe { Allocator::shrink(&self.0, pointer, old_layout, new_layout) }
        }
    };
}

// SAFETY: every method forwards unchanged to the inner blink-alloc allocator,
// which is itself a sound `Allocator`; the newtype adds no behavior of its own.
unsafe impl<A: Allocator> Allocator for LocalArena<A> {
    forward_allocator!();
}

// SAFETY: see the `LocalArena` impl above; this only forwards to the inner allocator.
unsafe impl<A: Allocator> Allocator for SharedArena<A> {
    forward_allocator!();
}

// SAFETY: see the `LocalArena` impl above; this only forwards to the inner allocator.
unsafe impl<A: Allocator> Allocator for ScopedArena<'_, A> {
    forward_allocator!();
}

struct ArenaFormatter<'buffer, 'arena, A: Allocator + ?Sized> {
    buffer: &'buffer mut Vec<'arena, u8, A>,
}

impl<A: Allocator + ?Sized> fmt::Write for ArenaFormatter<'_, '_, A> {
    #[inline]
    fn write_str(&mut self, fragment: &str) -> fmt::Result {
        self.buffer.extend_from_slice(fragment.as_bytes());

        Ok(())
    }
}

/// Ergonomic, `bumpalo`-style allocation on top of any [`Allocator`].
///
/// Every method returns a reference (or mutable slice) tied to the borrow of the
/// arena, so the allocation escapes the call and lives for the arena's lifetime.
/// Implemented for every [`Allocator`] (including the three arenas and shared
/// references to them) via a blanket impl.
pub trait Arena: Allocator {
    /// Allocates `value` and returns a unique reference to it.
    fn alloc<T>(&self, value: T) -> &mut T;

    /// Allocates the result of `f`, evaluated directly into the arena slot.
    fn alloc_with<T>(&self, f: impl FnOnce() -> T) -> &mut T;

    /// Copies `src` into the arena and returns the copy.
    fn alloc_str(&self, src: &str) -> &mut str;

    /// Formats `arguments` directly into the arena and returns the resulting string.
    ///
    /// The arena-equivalent of [`format!`], with no intermediate global allocation:
    ///
    /// ```
    /// use mago_allocator::prelude::*;
    ///
    /// let arena = LocalArena::new();
    /// let greeting = arena.alloc_fmt(format_args!("{}-{}", "v", 2));
    /// assert_eq!(greeting, "v-2");
    /// ```
    fn alloc_fmt(&self, arguments: fmt::Arguments<'_>) -> &mut str;

    /// Copies a slice of `Copy` values into the arena.
    fn alloc_slice_copy<T>(&self, src: &[T]) -> &mut [T]
    where
        T: Copy;

    /// Clones a slice of `Clone` values into the arena.
    fn alloc_slice_clone<T>(&self, src: &[T]) -> &mut [T]
    where
        T: Clone;

    /// Allocates `len` copies of `value`.
    fn alloc_slice_fill_copy<T>(&self, len: usize, value: T) -> &mut [T]
    where
        T: Copy;

    /// Allocates `len` clones of `value`.
    fn alloc_slice_fill_clone<T>(&self, len: usize, value: &T) -> &mut [T]
    where
        T: Clone;

    /// Allocates `len` elements, each produced by `f(index)`.
    fn alloc_slice_fill_with<T>(&self, len: usize, f: impl FnMut(usize) -> T) -> &mut [T];

    /// Allocates `len` default-constructed elements.
    fn alloc_slice_fill_default<T>(&self, len: usize) -> &mut [T]
    where
        T: Default;

    /// Collects `iter` into the arena and returns the resulting slice.
    fn alloc_slice_fill_iter<T>(&self, iter: impl IntoIterator<Item = T>) -> &mut [T];
}

impl<A: Allocator + ?Sized> Arena for A {
    #[inline]
    fn alloc<T>(&self, value: T) -> &mut T {
        Box::leak(Box::new_in(value, self))
    }

    #[inline]
    fn alloc_with<T>(&self, f: impl FnOnce() -> T) -> &mut T {
        Box::leak(Box::new_in(f(), self))
    }

    #[inline]
    fn alloc_str(&self, src: &str) -> &mut str {
        let bytes = self.alloc_slice_copy(src.as_bytes());

        // SAFETY: `bytes` is a byte-for-byte copy of a valid `&str`.
        unsafe { core::str::from_utf8_unchecked_mut(bytes) }
    }

    #[inline]
    fn alloc_fmt(&self, arguments: fmt::Arguments<'_>) -> &mut str {
        let mut buffer = Vec::new_in(self);
        let mut formatter = ArenaFormatter { buffer: &mut buffer };
        let _ = fmt::write(&mut formatter, arguments);

        let bytes = buffer.leak();

        // SAFETY: `fmt::write` only ever feeds valid UTF-8 to `write_str`, so the
        // accumulated bytes are valid UTF-8.
        unsafe { core::str::from_utf8_unchecked_mut(bytes) }
    }

    #[inline]
    fn alloc_slice_copy<T>(&self, src: &[T]) -> &mut [T]
    where
        T: Copy,
    {
        let mut vec = Vec::with_capacity_in(src.len(), self);
        vec.extend_from_slice(src);
        vec.leak()
    }

    #[inline]
    fn alloc_slice_clone<T>(&self, src: &[T]) -> &mut [T]
    where
        T: Clone,
    {
        let mut vec = Vec::with_capacity_in(src.len(), self);
        vec.extend(src.iter().cloned());
        vec.leak()
    }

    #[inline]
    fn alloc_slice_fill_copy<T>(&self, len: usize, value: T) -> &mut [T]
    where
        T: Copy,
    {
        let mut vec = Vec::with_capacity_in(len, self);
        vec.resize(len, value);
        vec.leak()
    }

    #[inline]
    fn alloc_slice_fill_clone<T>(&self, len: usize, value: &T) -> &mut [T]
    where
        T: Clone,
    {
        let mut vec = Vec::with_capacity_in(len, self);
        vec.resize_with(len, || value.clone());
        vec.leak()
    }

    #[inline]
    fn alloc_slice_fill_with<T>(&self, len: usize, mut f: impl FnMut(usize) -> T) -> &mut [T] {
        let mut vec = Vec::with_capacity_in(len, self);
        for index in 0..len {
            vec.push(f(index));
        }
        vec.leak()
    }

    #[inline]
    fn alloc_slice_fill_default<T>(&self, len: usize) -> &mut [T]
    where
        T: Default,
    {
        self.alloc_slice_fill_with(len, |_| T::default())
    }

    #[inline]
    fn alloc_slice_fill_iter<T>(&self, iter: impl IntoIterator<Item = T>) -> &mut [T] {
        let mut vec = Vec::new_in(self);
        vec.extend(iter);
        vec.leak()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::HashMap;
    use crate::vec::Vec;

    #[test]
    fn local_alloc_roundtrips() {
        let arena = LocalArena::new();
        let value = arena.alloc(42u32);

        assert_eq!(*value, 42);
    }

    #[test]
    fn alloc_str_copies() {
        let arena = LocalArena::new();
        let copied = arena.alloc_str("hello");

        assert_eq!(copied, "hello");
    }

    #[test]
    fn alloc_fmt_formats_into_arena() {
        let arena = LocalArena::new();
        let formatted = arena.alloc_fmt(format_args!("{}+{}={}", 2, 3, 5));

        assert_eq!(formatted, "2+3=5");
    }

    #[test]
    fn format_in_macro_formats_into_arena() {
        let arena = LocalArena::new();
        let label = crate::format_in!(arena, "{}::{}", "App", "VERSION");

        assert_eq!(label, "App::VERSION");
    }

    #[test]
    fn alloc_slice_copy_copies() {
        let arena = LocalArena::new();
        let slice = arena.alloc_slice_copy(&[1, 2, 3]);

        assert_eq!(slice, &[1, 2, 3]);
    }

    #[test]
    fn alloc_slice_fill_with_invokes_closure() {
        let arena = LocalArena::new();
        let slice = arena.alloc_slice_fill_with(4, |index| index * 2);

        assert_eq!(slice, &[0, 2, 4, 6]);
    }

    #[test]
    fn shared_vec_grows_in_place() {
        let arena = SharedArena::new();
        let mut vec = Vec::new_in(&arena);
        for value in 0..1000 {
            vec.push(value);
        }

        assert_eq!(vec.len(), 1000);
        assert_eq!(vec.last(), Some(&999));
    }

    #[test]
    fn scoped_alloc_roundtrips() {
        let shared = SharedArena::new();
        let scoped = shared.scoped();
        let value = scoped.alloc(7u8);

        assert_eq!(*value, 7);
    }

    #[test]
    fn arena_hashmap_roundtrips() {
        let arena = SharedArena::new();
        let mut map = HashMap::new_in(&arena);
        map.insert(1u32, "one");

        assert_eq!(map.get(&1), Some(&"one"));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn arena_vec_serializes() {
        let arena = LocalArena::new();
        let mut numbers = Vec::new_in(&arena);
        numbers.extend([1, 2, 3]);

        let json = serde_json::to_string(&numbers);

        assert!(matches!(json.as_deref(), Ok("[1,2,3]")));
    }
}
