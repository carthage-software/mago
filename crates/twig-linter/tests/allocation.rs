#![allow(clippy::unwrap_used)]

mod support;

use std::alloc::Layout;
use std::borrow::Cow;
use std::cell::Cell;
use std::ptr::NonNull;

use mago_allocator::LocalArena;
use mago_allocator::alloc::AllocError;
use mago_allocator::alloc::Allocator;
use mago_allocator::alloc::Global;
use mago_database::file::File;
use mago_twig_linter::settings::Settings;

struct Counted<'counter>(&'counter Cell<usize>);

// SAFETY: allocations and frees go to Global without changing their layout or pointer.
unsafe impl Allocator for Counted<'_> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.0.set(self.0.get() + layout.size());
        Global.allocate(layout)
    }

    unsafe fn deallocate(&self, pointer: NonNull<u8>, layout: Layout) {
        // SAFETY: the caller supplies a live allocation and its original layout.
        unsafe { Global.deallocate(pointer, layout) };
    }
}

#[test]
fn borrows_a_large_text_buffer_instead_of_copying_it_into_the_arena() {
    let allocated = Cell::new(0);
    let arena = LocalArena::new_in(Counted(&allocated));
    let file = File::ephemeral(Cow::Borrowed(b"large.twig"), Cow::Owned(vec![b'x'; 1024 * 1024]));
    let only: [&str; 0] = [];
    assert!(support::lint(&arena, &file, &Settings::default(), &only).is_empty());
    assert!(allocated.get() < 16 * 1024, "{} bytes allocated for a1MiB text-only template", allocated.get());
}
