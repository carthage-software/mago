#![allow(clippy::unwrap_used)]

mod support;

use std::borrow::Cow;

use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_span::HasSpan;
use mago_twig_linter::settings::Settings;

#[test]
fn raw_reports_each_filter_without_edits_that_could_join_tokens() {
    let source = include_bytes!("fixtures/raw_boundaries.twig");
    let arena = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"raw.twig"), Cow::Borrowed(source));
    let issues = support::lint(&arena, &file, &Settings::default(), &["twig-no-raw"]);
    assert_eq!(issues.len(), 6);
    for issue in issues.iter() {
        assert_eq!(issue.code.as_deref(), Some("twig-no-raw"));
        assert!(issue.edits.is_empty());
        let primary = issue.annotations.iter().find(|annotation| annotation.is_primary()).unwrap();
        assert_eq!(&source[primary.span.start_offset() as usize..primary.span.end_offset() as usize], b"raw");
    }
}

#[test]
fn legacy_raw_withholds_all_edits_but_keeps_diagnostics() {
    let source = include_str!("fixtures/legacy_raw.twig");
    let only = ["twig-named-argument-separator", "twig-single-quotes", "twig-end-block-name"];
    for legacy in [true, false] {
        for only in [only.as_slice(), ["strict-types"].as_slice()] {
            let source = if legacy { source.to_owned() } else { source.replace("raw", "verbatim") };
            let arena = LocalArena::new();
            let file = File::ephemeral(Cow::Borrowed(b"legacy.twig"), Cow::Owned(source.into_bytes()));
            let settings = Settings { version: "3.30".parse().unwrap(), ..Settings::default() };
            let issues = support::lint(&arena, &file, &settings, only);
            assert_eq!(issues.len(), if only.len() == 1 { 1 } else { 11 });
            assert_eq!(issues.iter().any(|issue| !issue.edits.is_empty()), !legacy);
        }
    }
}
