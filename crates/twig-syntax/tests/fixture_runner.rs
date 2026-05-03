#![allow(clippy::unwrap_used, clippy::expect_used, clippy::else_if_without_else)]

//! Drives the crate's parser against the upstream Twig fixture corpus.
//!
//! Each fixture is a `.test` file with the format popularised by Twig's own
//! test harness:
//!
//! ```text
//! --TEST--
//! <short description>
//! --TEMPLATE--
//! <template body>
//! --DATA-- (optional, ignored)
//! ...
//! --EXPECT-- (optional)
//! ...
//! --EXCEPTION-- (optional)
//! <error message>
//! ```
//!
//! A fixture is classified as **must-parse** (no `--EXCEPTION--` section) or
//! **exception** (has an `--EXCEPTION--` section).  For exception fixtures we
//! further classify the exception as a **parse-time** error (which we expect
//! to reject) or a **run-time** error (which we expect to accept, since our
//! parser is purely syntactic).

#[path = "common/mod.rs"]
mod common;

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use bumpalo::Bump;

use crate::common::parse;
use crate::common::roundtrip_tokens;

/// Heuristics for classifying an `--EXCEPTION--` message as a parse-time
/// error (we must reject) vs a run-time / compile-time error (we accept).
fn is_parse_time_error(message: &str) -> bool {
    let lo = message.to_ascii_lowercase();
    if lo.contains("string template") {
        return false;
    }

    const PARSE_TIME_NEEDLES: &[&str] = &[
        "unexpected token",
        "unexpected end of template",
        "unexpected character",
        "unclosed comment",
        "unclosed \"verbatim\"",
        "unclosed \"if\"",
        "unclosed \"for\"",
        "unclosed \"block\"",
        "unclosed \"",
        "expected name or number",
        "expected closing",
        "a block must start with a tag name",
        "a mapping key must be",
        "a sequence element",
        "a mapping value",
        "a list of arguments",
        "cannot have a multi-target",
        "must have the same number of variables",
        "expecting closing tag",
        "unexpected \"",
        "tag (expecting",
        "expected end of",
    ];

    const RUN_TIME_NEEDLES: &[&str] = &[
        "unknown \"",
        "variable",
        "template",
        "block",
        "does not contain",
        "does not exist",
        "macro",
        "does not define",
        "argument",
        "filter",
        "function",
        "test",
        "enum",
        "property",
        "method",
        "attribute",
        "cannot include self",
        "cannot extend",
        "extends tags are forbidden",
        "outside twig blocks",
        "outside of a block",
        "parent_without_extends",
    ];

    for needle in PARSE_TIME_NEEDLES {
        if lo.contains(needle) {
            let any_runtime = RUN_TIME_NEEDLES.iter().any(|rn| lo.contains(rn));
            if !any_runtime {
                return true;
            }

            return lo.contains("unexpected token")
                || lo.contains("unexpected character")
                || lo.contains("unclosed")
                || lo.contains("expected");
        }
    }

    false
}

struct Fixture {
    path: PathBuf,
    template: String,
    exception: Option<String>,
    has_secondary_templates: bool,
}

enum FixtureEntry {
    Ok(Fixture),
    SecondaryOnly { path: PathBuf, exception: Option<String> },
    ReadError { path: PathBuf, error: String },
}

fn parse_fixture(path: &Path) -> FixtureEntry {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => return FixtureEntry::ReadError { path: path.to_path_buf(), error: e.to_string() },
    };

    let mut template: Option<String> = None;
    let mut exception: Option<String> = None;
    let mut has_secondary_templates = false;
    let mut current_tag: Option<String> = None;
    let mut current_body = String::new();
    for line in content.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\n', '\r']);
        if trimmed.starts_with("--") && trimmed.ends_with("--") && trimmed.len() >= 4 {
            if let Some(tag) = current_tag.take() {
                commit_section(&tag, std::mem::take(&mut current_body), &mut template, &mut exception);
            }

            let name = &trimmed[2..trimmed.len() - 2];
            if name.starts_with("TEMPLATE(") {
                has_secondary_templates = true;
            }

            current_tag = Some(name.to_string());
        } else if current_tag.is_some() {
            current_body.push_str(line);
        }
    }

    if let Some(tag) = current_tag.take() {
        commit_section(&tag, current_body, &mut template, &mut exception);
    }

    match template {
        Some(template) => {
            FixtureEntry::Ok(Fixture { path: path.to_path_buf(), template, exception, has_secondary_templates })
        }
        None if has_secondary_templates => FixtureEntry::SecondaryOnly { path: path.to_path_buf(), exception },
        None => FixtureEntry::ReadError {
            path: path.to_path_buf(),
            error: "no `--TEMPLATE--` section and no `--TEMPLATE(name)--` sections".into(),
        },
    }
}

fn commit_section(tag: &str, body: String, template: &mut Option<String>, exception: &mut Option<String>) {
    match tag {
        "TEMPLATE" => {
            let body = body.trim_end_matches('\n').to_string();
            *template = Some(body);
        }
        "EXCEPTION" => {
            let body = body.trim_end_matches('\n').to_string();
            *exception = Some(body);
        }
        _ => {}
    }
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

struct Collected {
    fixtures: Vec<Fixture>,
    excluded: Vec<(PathBuf, String)>,
    total_files: usize,
}

fn collect_fixtures() -> Collected {
    let mut fixtures = Vec::new();
    let mut excluded = Vec::new();
    let mut total = 0;
    collect_recursive(&fixtures_dir(), &mut fixtures, &mut excluded, &mut total);
    fixtures.sort_by(|a, b| a.path.cmp(&b.path));
    excluded.sort_by(|a, b| a.0.cmp(&b.0));
    Collected { fixtures, excluded, total_files: total }
}

fn collect_recursive(
    dir: &Path,
    fixtures: &mut Vec<Fixture>,
    excluded: &mut Vec<(PathBuf, String)>,
    total: &mut usize,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_recursive(&p, fixtures, excluded, total);
        } else if p.extension().map(|x| x == "test").unwrap_or(false) {
            *total += 1;
            match parse_fixture(&p) {
                FixtureEntry::Ok(fx) => fixtures.push(fx),
                FixtureEntry::SecondaryOnly { path, exception } => {
                    let reason = format!(
                        "only secondary `--TEMPLATE(name)--` sections (no main `--TEMPLATE--`); exception={:?}",
                        exception.as_deref().unwrap_or("")
                    );
                    excluded.push((path, reason));
                }
                FixtureEntry::ReadError { path, error } => {
                    excluded.push((path, format!("malformed fixture: {error}")));
                }
            }
        }
    }
}

fn verbose() -> bool {
    std::env::var("TWIG_FIXTURE_VERBOSE").is_ok()
}

#[test]
fn fixtures_match_upstream_accept_reject() {
    let collected = collect_fixtures();
    assert!(!collected.fixtures.is_empty(), "no fixtures found in {}", fixtures_dir().display());

    let mut expected_accept = 0;
    let mut expected_reject = 0;
    let mut agree_accept = 0;
    let mut agree_reject = 0;
    let mut mismatches: Vec<String> = Vec::new();

    for fx in &collected.fixtures {
        let must_reject = fx.exception.as_deref().map(is_parse_time_error).unwrap_or(false);
        let must_reject = must_reject && !fx.has_secondary_templates;
        if must_reject {
            expected_reject += 1;
        } else {
            expected_accept += 1;
        }

        let arena = Bump::new();
        let tpl = parse(&arena, &fx.template);
        let accepted = !tpl.has_errors();
        let relative = fx.path.strip_prefix(fixtures_dir()).unwrap_or(&fx.path).display().to_string();
        match (must_reject, accepted) {
            (true, false) => agree_reject += 1,
            (false, true) => agree_accept += 1,
            (false, false) => {
                let first_error = tpl.errors.first().map(|e| e.to_string()).unwrap_or_default();
                mismatches.push(format!("expected to PARSE but rejected: {relative} :: {first_error}"));
            }
            (true, true) => {
                mismatches.push(format!(
                    "expected syntactic REJECT but parsed: {relative} :: exception={:?}",
                    fx.exception.as_deref().unwrap_or("")
                ));
            }
        }
    }

    let parsed = collected.fixtures.len();
    let excluded = collected.excluded.len();
    let total_files = collected.total_files;
    let summary = format!(
        "twig fixtures: total_files={total_files} parsed={parsed} excluded={excluded} expected_accept={expected_accept} expected_reject={expected_reject} agree_accept={agree_accept} agree_reject={agree_reject} disagree={}",
        mismatches.len()
    );
    if !mismatches.is_empty() {
        let slice: &[String] = if verbose() {
            &mismatches
        } else {
            let take = mismatches.len().min(10);
            &mismatches[..take]
        };
        panic!("{summary}\n  {}", slice.join("\n  "));
    }

    assert_eq!(
        agree_accept + agree_reject + excluded,
        total_files,
        "{summary}: fixtures not fully accounted for; excluded entries: {:?}",
        collected.excluded,
    );

    if verbose() && !collected.excluded.is_empty() {
        let lines: Vec<String> = collected
            .excluded
            .iter()
            .map(|(p, r)| format!("excluded: {} :: {}", p.strip_prefix(fixtures_dir()).unwrap_or(p).display(), r))
            .collect();
        panic!("verbose: {summary}\n  {}", lines.join("\n  "));
    }
}

#[test]
fn excluded_fixtures_are_explicit_and_self_consistent() {
    let collected = collect_fixtures();
    for (p, reason) in &collected.excluded {
        assert!(p.exists(), "excluded fixture does not exist: {}", p.display());
        assert!(!reason.is_empty(), "excluded fixture {} has empty reason", p.display());
    }

    let relative: Vec<String> = collected
        .excluded
        .iter()
        .map(|(p, _)| p.strip_prefix(fixtures_dir()).unwrap_or(p).display().to_string().replace('\\', "/"))
        .collect();

    assert_eq!(
        relative,
        vec!["tags/embed/error_line.test".to_string()],
        "the set of explicitly-excluded fixtures changed; review and update this test",
    );

    let arena = Bump::new();
    let tpl = parse(&arena, "");
    assert!(!tpl.has_errors(), "empty template should parse without errors");
    assert!(tpl.statements.is_empty());
}

#[test]
fn fixtures_roundtrip_for_accepted_templates() {
    let collected = collect_fixtures();
    let mut disagreements: Vec<String> = Vec::new();
    let mut tested = 0;
    for fx in &collected.fixtures {
        if fx.exception.is_some() {
            continue;
        }

        let arena = Bump::new();
        if parse(&arena, &fx.template).has_errors() {
            continue;
        }

        tested += 1;
        match roundtrip_tokens(&fx.template) {
            Ok(out) if out == fx.template => {}
            Ok(_) => {
                let rel = fx.path.strip_prefix(fixtures_dir()).unwrap_or(&fx.path).display().to_string();
                disagreements.push(format!("round-trip mismatch: {rel}"));
            }
            Err(e) => {
                let rel = fx.path.strip_prefix(fixtures_dir()).unwrap_or(&fx.path).display().to_string();
                disagreements.push(format!("tokenize error: {rel} :: {e}"));
            }
        }
    }

    assert!(
        disagreements.is_empty(),
        "{} round-trip failures out of {} tested:\n  {}",
        disagreements.len(),
        tested,
        disagreements.iter().take(10).cloned().collect::<Vec<_>>().join("\n  ")
    );
}
