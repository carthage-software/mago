//! Type-syntax parser fuzz target.

#![no_main]

use bumpalo::Bump;
use libfuzzer_sys::fuzz_target;

use mago_database::file::FileId;
use mago_span::Position;
use mago_span::Span;
use mago_type_syntax::parse_str;

fuzz_target!(|data: &[u8]| {
    let Ok(src) = std::str::from_utf8(data) else {
        return;
    };

    let arena = Bump::new();
    let span = Span::new(FileId::zero(), Position::new(0), Position::new(src.len() as u32));
    let owned = bumpalo::collections::String::from_str_in(src, &arena).into_bump_str();
    let _ = parse_str(&arena, span, owned);
});
