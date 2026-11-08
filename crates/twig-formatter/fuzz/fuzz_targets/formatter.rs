#![no_main]

use std::borrow::Cow;

use libfuzzer_sys::fuzz_target;
use mago_allocator::LocalArena;
use mago_twig_formatter::Formatter;
use mago_twig_formatter::settings::FormatSettings;

#[path = "../../tests/support/mod.rs"]
mod support;

fuzz_target!(|data: &[u8]| {
    for settings in
        [FormatSettings::default(), FormatSettings { print_width: 20, use_tabs: true, ..FormatSettings::default() }]
    {
        let arena = LocalArena::new();
        let formatter = Formatter::new(&arena, settings);
        if let Ok(formatted) = formatter.format_code(Cow::Borrowed(b"fuzz.twig"), Cow::Owned(data.to_vec())) {
            support::assert_preserves_syntax(data, formatted);
            let repeated = formatter
                .format_code(Cow::Borrowed(b"fuzz.twig"), Cow::Owned(formatted.to_vec()))
                .expect("formatted output must parse");
            assert_eq!(formatted, repeated, "formatting must be idempotent");
        }
    }
});
