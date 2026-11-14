#![allow(clippy::unwrap_used)]

mod support;

use std::borrow::Cow;

use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_span::HasSpan;
use mago_text_edit::Safety;
use mago_twig_linter::settings::Settings;

#[test]
fn reports_exact_primary_ranges_and_fix_safety() {
    let source = include_str!("fixtures/positions.twig");
    for (code, text, occurrence, safety) in [
        ("twig-variable-name", "badName", 0, None),
        ("twig-macro-name", "badMacro", 0, None),
        ("twig-macro-argument-name", "badArg", 0, None),
        ("twig-duplicate-parameter", "badArg", 1, None),
        ("twig-duplicate-block", "repeated", 1, None),
        ("twig-duplicate-argument", "first", 1, None),
        ("twig-named-argument-separator", "=", 1, Some(Safety::Safe)),
        ("twig-duplicate-key", "'key'", 1, None),
        ("twig-no-dump", "dump", 0, None),
        ("twig-single-quotes", "\"<ok>\"", 0, Some(Safety::Safe)),
        ("twig-no-raw", "raw", 0, None),
        ("twig-duplicate-macro", "once", 1, None),
        ("twig-named-argument-name", "badNamed", 0, None),
        ("twig-deprecated-spaceless", "spaceless", 0, None),
        ("twig-deprecated-attribute", "attribute", 0, None),
        ("twig-no-autoescape-false", "autoescape", 0, None),
        ("twig-explicit-include-context", "include", 0, None),
        ("twig-no-include-ignore-missing", "ignore missing", 0, None),
        ("twig-prefer-include-function", "include", 0, None),
        ("twig-disallowed-functions", "blocked_function", 0, None),
        ("twig-disallowed-filters", "blocked_filter", 0, None),
        ("twig-disallowed-tags", "flush", 0, None),
        ("twig-no-nested-ternary", "?", 1, None),
        ("twig-constant-condition", "true", 0, None),
        ("twig-end-block-name", "endmacro", 0, Some(Safety::Safe)),
    ] {
        let arena = LocalArena::new();
        let file = File::ephemeral(Cow::Borrowed(b"positions.twig"), Cow::Borrowed(source.as_bytes()));
        let mut settings = Settings { version: "3.30".parse().unwrap(), ..Settings::default() };
        settings.rules.disallowed_functions.config.names = vec!["blocked_function".into()];
        settings.rules.disallowed_filters.config.names = vec!["blocked_filter".into()];
        settings.rules.disallowed_tags.config.names = vec!["flush".into()];
        let issues = support::lint(&arena, &file, &settings, &[code]);
        let issue = issues.iter().next().unwrap();
        let primary = issue.annotations.iter().find(|annotation| annotation.is_primary()).unwrap();
        let start = source.match_indices(text).nth(occurrence).unwrap().0 as u32;
        assert_eq!(
            (primary.span.start_offset(), primary.span.end_offset()),
            (start, start + text.len() as u32),
            "{code}"
        );
        let edits: Vec<_> = issue.edits.values().flatten().collect();
        if let Some(safety) = safety {
            assert!(!edits.is_empty());
            assert!(edits.iter().all(|edit| edit.safety == safety));
        } else {
            assert!(edits.is_empty());
        }
    }
}
