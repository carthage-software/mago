#![allow(clippy::unwrap_used)]

mod support;

use std::borrow::Cow;

use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_twig_linter::settings::Settings;

#[test]
fn versioned_rules_require_a_proven_supported_minimum_version() {
    assert_eq!(Settings::default().version.to_string(), "3.0");
    for version in ["3.0", "3.11", "3.12", "3.14", "3.15", "3.28", "3.29", "3.30"] {
        let settings = Settings { version: version.parse().unwrap(), ..Settings::default() };
        for (code, source, minor, count) in [
            ("twig-named-argument-separator", "{{ range(low = 1, high = 2) }}", 12, 2),
            ("twig-deprecated-spaceless", "{{ value|spaceless }}", 12, 1),
            ("twig-deprecated-attribute", "{{ attribute(value, 'name') }}", 15, 1),
            ("twig-duplicate-macro", "{% macro name() %}{% endmacro %}{% macro name() %}{% endmacro %}", 29, 1),
        ] {
            let arena = LocalArena::new();
            let only = [code];
            let file = File::ephemeral(Cow::Borrowed(b"version.twig"), Cow::Borrowed(source.as_bytes()));
            let issues = support::lint(&arena, &file, &settings, &only);
            assert_eq!(issues.len(), if settings.version.supports(minor) { count } else { 0 }, "{code}, {version}");
        }
    }
}
