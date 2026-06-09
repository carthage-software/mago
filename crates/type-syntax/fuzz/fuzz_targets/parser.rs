//! Type-syntax parser fuzz target.

#![no_main]

use mago_allocator::LocalArena;
use libfuzzer_sys::fuzz_target;

use mago_database::file::FileId;
use mago_span::Position;
use mago_span::Span;
use mago_type_syntax::parse_str;

fuzz_target!(|data: &[u8]| {
    let arena = LocalArena::new();
    let span = Span::new(FileId::zero(), Position::new(0), Position::new(data.len() as u32));
    let owned = arena.alloc_slice_copy(data);
    let _ = parse_str(&arena, span, owned);
});
