use std::alloc::GlobalAlloc;
use std::alloc::Layout;
use std::alloc::System;

/// A bump-down allocator. Allocates from a fixed-size system block, never frees.
///
/// When the block is exhausted, the [`Interner`](crate::interner::Interner) keeps the
/// old block around (so all previously-handed-out pointers remain valid forever) and
/// installs a fresh, larger block. Memory leaks by design; the interner backs handles
/// that are intended to live for the lifetime of the process.
///
/// Allocating downward is a minor perf win on multithreaded workloads; see
/// <https://fitzgeraldnick.com/2019/11/01/always-bump-downwards.html>.
pub(crate) struct LeakyBumpAlloc {
    layout: Layout,
    start: *mut u8,
    end: *mut u8,
    ptr: *mut u8,
}

impl LeakyBumpAlloc {
    #[allow(clippy::expect_used)]
    pub(crate) fn new(capacity: usize, alignment: usize) -> LeakyBumpAlloc {
        // Construction-time invariant on the interner side: alignment is a power of two and
        // capacity is bounded — a Layout failure here would be a programmer bug, not runtime
        // input, so abort cleanly via `expect`.
        let layout = Layout::from_size_align(capacity, alignment).expect("invalid layout for word arena");
        // SAFETY: `layout` was just constructed by `Layout::from_size_align` and verified above.
        let start = unsafe { System.alloc(layout) };
        if start.is_null() {
            std::alloc::handle_alloc_error(layout);
        }

        // SAFETY: `System.alloc` returned a valid block of `layout.size()` bytes starting at
        // `start`; `start.add(layout.size())` is the one-past-end pointer of that block, which
        // is a valid pointer per the offset rules.
        let end = unsafe { start.add(layout.size()) };
        LeakyBumpAlloc { layout, start, end, ptr: end }
    }

    /// Allocates `num_bytes` from the block, returning the new pointer. Aborts the
    /// process on OOM; we hold the interner mutex while allocating and panicking
    /// across that boundary leaves the mutex poisoned and the cache wedged.
    #[allow(clippy::expect_used)]
    pub(crate) unsafe fn allocate(&mut self, num_bytes: usize) -> *mut u8 {
        let ptr = self.ptr as usize;
        // `expect` is fine: a numeric underflow here means we've handed out more memory than
        // a pointer-sized integer can address, which is impossible while the program is alive.
        let new_ptr = ptr.checked_sub(num_bytes).expect("word arena bump underflow");
        let new_ptr = new_ptr & !(self.layout.align() - 1);
        let start = self.start as usize;
        if new_ptr < start {
            std::process::abort();
        }

        // SAFETY: `ptr - new_ptr` is non-negative (checked_sub above), and `new_ptr` is still
        // within `[start, end)` (we aborted otherwise), so the resulting pointer stays inside
        // the block we allocated in `new`.
        self.ptr = unsafe { self.ptr.sub(ptr - new_ptr) };
        self.ptr
    }

    pub(crate) fn allocated(&self) -> usize {
        self.end as usize - self.ptr as usize
    }

    pub(crate) fn capacity(&self) -> usize {
        self.layout.size()
    }
}

// SAFETY: the allocator holds only owned pointers into its own backing block. There is no
// shared mutable state; the surrounding interner serialises calls to `allocate` with a mutex.
unsafe impl Send for LeakyBumpAlloc {}
