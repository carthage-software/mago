//! Parser fuzz target.
//!
//! Feeds arbitrary bytes to `parse_file` and asserts that the parser does
//! not panic, abort, or stack-overflow regardless of input. Parse errors are
//! a normal outcome and are intentionally ignored — only crashes are bugs.

#![no_main]

use std::borrow::Cow;

use bumpalo::Bump;
use libfuzzer_sys::fuzz_target;

use mago_database::file::File;
use mago_syntax::parser::parse_file;

fuzz_target!(|data: &[u8]| {
    // Reject bytes that aren't valid UTF-8: the parser's input layer expects a `&str`.
    let Ok(src) = std::str::from_utf8(data) else {
        return;
    };

    let arena = Bump::new();
    let file = File::ephemeral(Cow::Borrowed("fuzz.php"), Cow::Borrowed(src));
    let _ = parse_file(&arena, &file);
});
