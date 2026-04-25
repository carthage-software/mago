//! Twig parser fuzz target. Asserts no panic / abort / stack-overflow on
//! arbitrary input. Parse errors are an expected outcome and are dropped.

#![no_main]

use std::borrow::Cow;

use bumpalo::Bump;
use libfuzzer_sys::fuzz_target;

use mago_database::file::File;
use mago_twig_syntax::parser::parse_file;

fuzz_target!(|data: &[u8]| {
    let Ok(src) = std::str::from_utf8(data) else {
        return;
    };

    let arena = Bump::new();
    let file = File::ephemeral(Cow::Borrowed("fuzz.twig"), Cow::Borrowed(src));
    let _ = parse_file(&arena, &file);
});
