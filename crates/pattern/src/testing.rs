//! Test helpers for `mago-pattern` so tests can focus on the pattern itself.
//!
//! The macros use `$crate::testing::*` helpers to keep the macro body small and keep the
//! logic testable without going through the macro surface every time.

use std::borrow::Cow;
use std::path::Path;

use bumpalo::Bump;
use mago_database::file::File;
use mago_syntax::parser::parse_file;

use crate::MagoIndex;
use crate::Match;

/// Compile a pattern + run it against `code`, returning every match.
///
/// Panics with a useful message if compilation fails; test macros call this unconditionally.
pub fn run_query(pattern_src: &str, code: &str) -> Vec<Match> {
    let pattern_arena = Bump::new();
    let compiled = match crate::compile(&pattern_arena, pattern_src) {
        Ok(c) => c,
        Err(err) => panic!("pattern failed to compile: {err}\n  pattern: {pattern_src:?}"),
    };

    let target_arena = Bump::new();
    let file = File::ephemeral(Cow::Borrowed("<test>"), Cow::Owned(code.to_string()));
    let program = parse_file(&target_arena, &file);
    let source = target_arena.alloc_str(file.contents.as_ref());
    let index = MagoIndex::new(program, source);
    match crate::query(&compiled, &index, Path::new("<test>")) {
        Ok(matches) => matches,
        Err(err) => panic!("pattern query failed: {err}\n  pattern: {pattern_src:?}\n  code: {code:?}"),
    }
}

/// Returns `true` if every expected `(name, value)` pair appears in any of `matches`'s captures.
/// For tests that don't care which match a capture came from; convenient when a pattern
/// matches exactly once and the test wants to assert a specific binding.
pub fn has_capture(matches: &[Match], name: &str, value: &str) -> bool {
    matches.iter().any(|m| m.captures.iter().any(|(n, v)| n == name && v == value))
}

/// Asserts that `matches` contains exactly `expected` entries, panicking with the actual
/// capture table on mismatch to make failures useful.
#[track_caller]
pub fn assert_match_count(matches: &[Match], expected: usize) {
    if matches.len() != expected {
        let actual: Vec<String> = matches
            .iter()
            .map(|m| {
                let caps: Vec<String> = m.captures.iter().map(|(k, v)| format!("^{k}={v:?}")).collect();
                format!("  - bytes {}..{} {{ {} }}", m.range.start, m.range.end, caps.join(", "))
            })
            .collect();
        panic!("expected {expected} match(es), got {}:\n{}", matches.len(), actual.join("\n"));
    }
}

/// Asserts that every `(variable_name, expected_value)` pair is present in at least one
/// match's capture list.
#[track_caller]
pub fn assert_has_captures(matches: &[Match], expected: &[(&str, &str)]) {
    for &(name, value) in expected {
        if !has_capture(matches, name, value) {
            let actual: Vec<String> =
                matches.iter().flat_map(|m| m.captures.iter().map(|(k, v)| format!("^{k}={v:?}"))).collect();
            panic!("expected capture `^{name}={value:?}` not found in matches; saw:\n  {}", actual.join("\n  "));
        }
    }
}

/// Run a rewriting pattern against `before` source, apply all emitted rewrites in
/// reverse order, and return the resulting text. Used by the `test_replaces!` macro.
pub fn run_replace(pattern_src: &str, before: &str) -> String {
    let matches = run_query(pattern_src, before);
    let mut all_rewrites: Vec<crate::Rewrite> = matches.iter().flat_map(|m| m.rewrites.iter().cloned()).collect();
    // Apply from the end so earlier ranges aren't invalidated.
    all_rewrites.sort_by_key(|r| std::cmp::Reverse(r.range.start));
    let mut out = before.to_string();
    for r in &all_rewrites {
        let start = r.range.start.min(out.len());
        let end = r.range.end.min(out.len());
        if start > end {
            continue;
        }
        out.replace_range(start..end, &r.replacement);
    }
    out
}

/// Asserts that applying `pattern` to `before` produces exactly `after`.
#[track_caller]
pub fn assert_replaces(pattern: &str, before: &str, expected: &str) {
    let actual = run_replace(pattern, before);
    if actual != expected {
        panic!(
            "replacement mismatch\n  pattern:  {pattern:?}\n  before:   {before:?}\n  expected: {expected:?}\n  actual:   {actual:?}",
        );
    }
}

/// A single case macro: asserts a given pattern matches `code` exactly `expect` times.
///
/// ```ignore
/// test_matches! {
///     name = eval_is_banned,
///     pattern = "eval(^code)",
///     code = "<?php eval($x);",
///     expect = 1,
/// }
/// ```
#[macro_export]
macro_rules! test_matches {
    (
        $(name = $name:ident,)?
        pattern = $pattern:expr,
        code = $code:expr,
        expect = $expect:expr $(,)?
    ) => {
        $crate::__test_matches_impl!($($name)?, $pattern, $code, $expect);
    };
}

/// Asserts that the pattern matches `code` exactly `expect` times AND that each listed
/// `(var, value)` capture is present somewhere.
///
/// ```ignore
/// test_matches_with_captures! {
///     name = eval_with_capture,
///     pattern = "eval(^code)",
///     code = "<?php eval($foo);",
///     expect = 1,
///     captures = [("code", "$foo")],
/// }
/// ```
#[macro_export]
macro_rules! test_matches_with_captures {
    (
        $(name = $name:ident,)?
        pattern = $pattern:expr,
        code = $code:expr,
        expect = $expect:expr,
        captures = [ $(($k:expr, $v:expr)),* $(,)? ] $(,)?
    ) => {
        $crate::__test_matches_captures_impl!(
            $($name)?, $pattern, $code, $expect, [$(($k, $v)),*]
        );
    };
}

/// Asserts that the pattern does NOT match `code`.
///
/// ```ignore
/// test_no_match! {
///     name = eval_missing,
///     pattern = "eval(^x)",
///     code = "<?php echo 'hi';",
/// }
/// ```
#[macro_export]
macro_rules! test_no_match {
    (
        $(name = $name:ident,)?
        pattern = $pattern:expr,
        code = $code:expr $(,)?
    ) => {
        $crate::__test_matches_impl!($($name)?, $pattern, $code, 0);
    };
}

/// Asserts that applying a rewriting `pattern` to `before` produces exactly `after`.
///
/// ```ignore
/// test_replaces! {
///     name = eval_to_safe_eval,
///     pattern = "`eval(^x)` => `safe_eval(^x)`",
///     before = "<?php eval($code);",
///     after = "<?php safe_eval($code);",
/// }
/// ```
#[macro_export]
macro_rules! test_replaces {
    (
        $(name = $name:ident,)?
        pattern = $pattern:expr,
        before = $before:expr,
        after = $after:expr $(,)?
    ) => {
        $crate::__test_replaces_impl!($($name)?, $pattern, $before, $after);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __test_replaces_impl {
    ($name:ident, $pattern:expr, $before:expr, $after:expr) => {
        #[test]
        fn $name() {
            $crate::testing::assert_replaces($pattern, $before, $after);
        }
    };
    (, $pattern:expr, $before:expr, $after:expr) => {
        $crate::testing::assert_replaces($pattern, $before, $after);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __test_matches_impl {
    ($name:ident, $pattern:expr, $code:expr, $expect:expr) => {
        #[test]
        fn $name() {
            let matches = $crate::testing::run_query($pattern, $code);
            $crate::testing::assert_match_count(&matches, $expect);
        }
    };
    (, $pattern:expr, $code:expr, $expect:expr) => {
        let matches = $crate::testing::run_query($pattern, $code);
        $crate::testing::assert_match_count(&matches, $expect);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __test_matches_captures_impl {
    ($name:ident, $pattern:expr, $code:expr, $expect:expr, [$(($k:expr, $v:expr)),*]) => {
        #[test]
        fn $name() {
            let matches = $crate::testing::run_query($pattern, $code);
            $crate::testing::assert_match_count(&matches, $expect);
            $crate::testing::assert_has_captures(&matches, &[$(($k, $v)),*]);
        }
    };
    (, $pattern:expr, $code:expr, $expect:expr, [$(($k:expr, $v:expr)),*]) => {
        let matches = $crate::testing::run_query($pattern, $code);
        $crate::testing::assert_match_count(&matches, $expect);
        $crate::testing::assert_has_captures(&matches, &[$(($k, $v)),*]);
    };
}
