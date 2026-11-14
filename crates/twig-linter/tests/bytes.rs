#![allow(clippy::unwrap_used)]

mod support;

use std::borrow::Cow;

use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_twig_linter::settings::Settings;

#[test]
fn literal_mapping_keys_keep_non_utf8_bytes_and_numeric_string_boundaries() {
    for source in [
        b"{{ {'\xff': 1, '\\xff': 2} }}".as_slice(),
        b"{{ {(-0): 1, 0: 2} }}",
        b"{{ {1_000: 1, '1000': 2} }}",
        b"{{ {(1e3): 1, 1000: 2} }}",
        b"{{ {'a\\\r\nb': 1, 'a\\nb': 2} }}",
        "{{ {'café': 1, 'caf\\xc3\\xa9': 2} }}".as_bytes(),
    ] {
        let arena = LocalArena::new();
        let only = ["twig-duplicate-key"];
        let file = File::ephemeral(Cow::Borrowed(b"keys.twig"), Cow::Borrowed(source));
        let issues = support::lint(&arena, &file, &Settings::default(), &only);
        assert_eq!(issues.len(), 1, "{issues:?}");
        assert_eq!(issues.iter().next().unwrap().code.as_deref(), Some("twig-duplicate-key"));
    }
    for source in [
        b"{{ {0: 1, '-0': 2} }}".as_slice(),
        b"{{ {1: 1, '01': 2} }}",
        b"{{ {1: 1, '1.0': 2} }}",
        b"{{ {true: 1, 1: 2} }}",
        b"{{ {null: 1, '': 2} }}",
        "{{ {'café': 1, 'cafe\u{301}': 2} }}".as_bytes(),
    ] {
        let arena = LocalArena::new();
        let only = ["twig-duplicate-key"];
        let file = File::ephemeral(Cow::Borrowed(b"keys.twig"), Cow::Borrowed(source));
        assert!(support::lint(&arena, &file, &Settings::default(), &only).is_empty());
    }
}
