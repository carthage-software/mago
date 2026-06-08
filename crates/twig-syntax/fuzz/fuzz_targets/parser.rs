//! Twig parser fuzz target. Asserts no panic / abort / stack-overflow on
//! arbitrary input. Parse errors are an expected outcome and are dropped.

#![no_main]

use std::borrow::Cow;

use mago_allocator::LocalArena;
use libfuzzer_sys::fuzz_target;

use mago_database::file::File;
use mago_twig_syntax::parser::parse_file;

fuzz_target!(|data: &[u8]| {
    let arena = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"fuzz.twig"), Cow::Owned(data.to_vec()));
    let _ = parse_file(&arena, &file);
});
