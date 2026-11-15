//! Keep CLI allocations on mimalloc and use jemalloc for the language server.

use std::alloc::GlobalAlloc;
use std::alloc::Layout;
use std::ffi::c_char;
use std::ffi::c_void;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use mimalloc::MiMalloc;
use tikv_jemallocator::Jemalloc;

// Jemalloc reads this pointer before main, so its settings must be static.
#[unsafe(export_name = "_rjem_malloc_conf")]
static MALLOC_CONF: Option<&'static c_char> = Some(unsafe {
    // SAFETY: The C string has static storage and a trailing NUL.
    &*c"narenas:1,tcache:false,dirty_decay_ms:0,muzzy_decay_ms:0,retain:false".as_ptr()
});

unsafe extern "C" {
    fn mi_is_in_heap_region(pointer: *const c_void) -> bool;
}

pub(crate) struct Allocator {
    language_server: AtomicBool,
}

impl Allocator {
    pub(crate) const fn new() -> Self {
        Self { language_server: AtomicBool::new(false) }
    }

    pub(crate) fn enable_language_server(&self) {
        // Never switch back: jemalloc allocations can outlive the server.
        self.language_server.store(true, Ordering::Release);
    }

    #[inline]
    fn owns_mimalloc(&self, pointer: *mut u8) -> bool {
        !self.language_server.load(Ordering::Acquire)
            // SAFETY: This lookup accepts foreign pointers and reads mimalloc's page map.
            || unsafe { mi_is_in_heap_region(pointer.cast()) }
    }
}

// SAFETY: Each allocation keeps its allocator for both reallocation and freeing.
unsafe impl GlobalAlloc for Allocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: The caller supplies a valid allocation layout.
        unsafe {
            if self.language_server.load(Ordering::Acquire) { Jemalloc.alloc(layout) } else { MiMalloc.alloc(layout) }
        }
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: The caller supplies a valid allocation layout.
        unsafe {
            if self.language_server.load(Ordering::Acquire) {
                Jemalloc.alloc_zeroed(layout)
            } else {
                MiMalloc.alloc_zeroed(layout)
            }
        }
    }

    #[inline]
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: The caller supplies a live allocation; the lookup selects its owner.
        unsafe {
            if self.owns_mimalloc(pointer) {
                MiMalloc.dealloc(pointer, layout)
            } else {
                Jemalloc.dealloc(pointer, layout)
            }
        }
    }

    #[inline]
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: The caller supplies a live allocation and valid size; keep its owner.
        unsafe {
            if self.owns_mimalloc(pointer) {
                MiMalloc.realloc(pointer, layout, new_size)
            } else {
                Jemalloc.realloc(pointer, layout, new_size)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocations_keep_their_owner_after_language_server_switch() {
        let allocator = Allocator::new();

        // SAFETY: All layouts are nonzero; check each allocation before access and free it once.
        unsafe {
            let old = [(32, 8), (3 * 1024 * 1024, 8), (64, 64 * 1024)].map(|(size, alignment)| {
                let layout = Layout::from_size_align(size, alignment).unwrap();
                let to_free = allocator.alloc(layout);
                let to_grow = allocator.alloc(layout);
                assert!(!to_free.is_null() && !to_grow.is_null());
                assert!(mi_is_in_heap_region(to_free.cast()));
                assert!(mi_is_in_heap_region(to_grow.cast()));
                assert_eq!(to_grow.addr() % alignment, 0);
                to_grow.write_bytes(0x5a, size);
                (layout, to_free, to_grow)
            });

            allocator.enable_language_server();
            for (layout, to_free, to_grow) in old {
                allocator.dealloc(to_free, layout);
                let grown = allocator.realloc(to_grow, layout, layout.size() * 2);
                assert!(!grown.is_null());
                assert!(mi_is_in_heap_region(grown.cast()));
                assert_eq!(grown.addr() % layout.align(), 0);
                assert!(std::slice::from_raw_parts(grown, layout.size()).iter().all(|byte| *byte == 0x5a));
                allocator.dealloc(grown, Layout::from_size_align(layout.size() * 2, layout.align()).unwrap());
            }

            for (size, alignment, zeroed) in [(37, 1, false), (64, 4096, true)] {
                let layout = Layout::from_size_align(size, alignment).unwrap();
                let pointer = if zeroed { allocator.alloc_zeroed(layout) } else { allocator.alloc(layout) };
                assert!(!pointer.is_null());
                assert!(!mi_is_in_heap_region(pointer.cast()));
                assert_eq!(pointer.addr() % alignment, 0);
                if zeroed {
                    assert!(std::slice::from_raw_parts(pointer, size).iter().all(|byte| *byte == 0));
                }
                pointer.write_bytes(0x6b, size);
                let grown = allocator.realloc(pointer, layout, size * 2);
                assert!(!grown.is_null());
                assert!(!mi_is_in_heap_region(grown.cast()));
                assert_eq!(grown.addr() % alignment, 0);
                assert!(std::slice::from_raw_parts(grown, size).iter().all(|byte| *byte == 0x6b));
                allocator.dealloc(grown, Layout::from_size_align(size * 2, alignment).unwrap());
            }
        }
    }

    #[test]
    fn allocations_keep_their_owner_across_threads() {
        let allocator = Allocator::new();
        let layout = Layout::from_size_align(32, 8).unwrap();

        // SAFETY: Expose each live allocation's provenance for one transfer, then free it once.
        unsafe {
            let old = allocator.alloc(layout);
            assert!(!old.is_null());
            let old_address = old.expose_provenance();
            allocator.enable_language_server();

            let new_address = std::thread::scope(|scope| {
                let allocator = &allocator;
                scope
                    .spawn(move || {
                        let old = std::ptr::with_exposed_provenance_mut::<u8>(old_address);
                        assert!(mi_is_in_heap_region(old.cast()));
                        allocator.dealloc(old, layout);

                        let new = allocator.alloc(layout);
                        assert!(!new.is_null());
                        assert!(!mi_is_in_heap_region(new.cast()));
                        new.write_bytes(0x6b, layout.size());
                        new.expose_provenance()
                    })
                    .join()
                    .unwrap()
            });
            let new = std::ptr::with_exposed_provenance_mut::<u8>(new_address);
            assert!(std::slice::from_raw_parts(new, layout.size()).iter().all(|byte| *byte == 0x6b));
            allocator.dealloc(new, layout);
        }
    }
}
