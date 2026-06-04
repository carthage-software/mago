#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Focused tests that every `ParseError` and `SyntaxError` variant has a
//! minimal trigger and is distinguished by variant (not just `Err(_)`).

#[path = "common/mod.rs"]
mod common;

use crate::common::parse;
use crate::common::tokenize;
use bumpalo::Bump;
use mago_twig_syntax::error::ParseError;
use mago_twig_syntax::error::SyntaxError;

use crate::common::rejects_with;

#[test]
fn syntax_unexpected_character() {
    let err = tokenize("{{ @ }}").unwrap_err();
    assert!(matches!(err, SyntaxError::UnexpectedCharacter(..)), "got {err:?}");
}

#[test]
fn syntax_unclosed_verbatim() {
    let err = tokenize("{% verbatim %}body").unwrap_err();
    assert!(matches!(err, SyntaxError::UnclosedVerbatim(..)), "got {err:?}");
}

#[test]
fn syntax_unclosed_comment() {
    let err = tokenize("{# no close").unwrap_err();
    assert!(matches!(err, SyntaxError::UnclosedComment(..)), "got {err:?}");
}

#[test]
fn syntax_unclosed_single_quoted_string() {
    let err = tokenize("{{ 'hello }}").unwrap_err();
    assert!(matches!(err, SyntaxError::UnclosedString(..)), "got {err:?}");
}

#[test]
fn syntax_unclosed_double_quoted_string() {
    let err = tokenize(r#"{{ "hello }}"#).unwrap_err();
    assert!(matches!(err, SyntaxError::UnclosedString(..)), "got {err:?}");
}

#[test]
fn syntax_unclosed_bracket_paren() {
    let err = tokenize("{{ f(1 }}").unwrap_err();
    assert!(matches!(err, SyntaxError::UnclosedBracket(..) | SyntaxError::UnmatchedBracket(..)), "got {err:?}");
}

#[test]
fn syntax_unclosed_bracket_square() {
    let err = tokenize("{{ [1 }}").unwrap_err();
    assert!(matches!(err, SyntaxError::UnclosedBracket(..) | SyntaxError::UnmatchedBracket(..)), "got {err:?}");
}

#[test]
fn syntax_unclosed_bracket_at_eof() {
    let err = tokenize("{{ f(1").unwrap_err();
    assert!(matches!(err, SyntaxError::UnclosedBracket(..)), "got {err:?}");
}

#[test]
fn syntax_unmatched_bracket_close_paren() {
    let err = tokenize("{{ a ) }}").unwrap_err();
    assert!(matches!(err, SyntaxError::UnmatchedBracket(..)), "got {err:?}");
}

#[test]
fn syntax_unmatched_bracket_close_square() {
    let err = tokenize("{{ a ] }}").unwrap_err();
    assert!(matches!(err, SyntaxError::UnmatchedBracket(..)), "got {err:?}");
}

#[test]
fn syntax_unclosed_tag_block() {
    let err = tokenize("{% if a ").unwrap_err();
    assert!(matches!(err, SyntaxError::UnclosedTag(..)), "got {err:?}");
}

#[test]
fn syntax_unclosed_tag_variable() {
    let err = tokenize("{{ a").unwrap_err();
    assert!(matches!(err, SyntaxError::UnclosedTag(..)), "got {err:?}");
}

#[test]
fn parse_unexpected_token_basic() {
    rejects_with("{{ + }}", |e| matches!(e, ParseError::UnexpectedToken(..) | ParseError::UnexpectedEof(..)));
}

#[test]
fn parse_unexpected_eof_in_if() {
    rejects_with("{% if a %}body", |e| {
        matches!(e, ParseError::UnexpectedEof(..) | ParseError::UnexpectedToken(..) | ParseError::SyntaxError(_))
    });
}

#[test]
fn parse_unexpected_eof_in_for() {
    rejects_with("{% for x in xs %}body", |e| {
        matches!(e, ParseError::UnexpectedEof(..) | ParseError::UnexpectedToken(..) | ParseError::SyntaxError(_))
    });
}

// NOTE: for cross-kind mismatches like `{% if %}...{% endfor %}` the parser
// currently swallows the `endfor` as an unknown opaque tag, and then fails at
// EOF while still hunting for `endif`.  That produces `UnexpectedEof` rather
// than `MismatchedEndTag`.  The `MismatchedEndTag` variant is reached when the
// closing name-disambiguation is for a specific block (`{% endblock name %}`).

#[test]
fn parse_mismatched_end_tag_if_endfor_yields_eof() {
    rejects_with("{% if a %}body{% endfor %}", |e| {
        matches!(e, ParseError::UnexpectedEof(..) | ParseError::MismatchedEndTag { .. })
    });
}

#[test]
fn parse_mismatched_end_tag_for_endif_yields_eof() {
    rejects_with("{% for x in xs %}body{% endif %}", |e| {
        matches!(e, ParseError::UnexpectedEof(..) | ParseError::MismatchedEndTag { .. })
    });
}

#[test]
fn parse_mismatched_end_tag_block_endmacro_yields_eof() {
    rejects_with("{% block b %}body{% endmacro %}", |e| {
        matches!(e, ParseError::UnexpectedEof(..) | ParseError::MismatchedEndTag { .. })
    });
}

#[test]
fn parse_mismatched_end_tag_macro_endblock_yields_eof() {
    rejects_with("{% macro m() %}body{% endblock %}", |e| {
        matches!(e, ParseError::UnexpectedEof(..) | ParseError::MismatchedEndTag { .. })
    });
}

#[test]
fn parse_mismatched_end_tag_apply_endfor_yields_eof() {
    rejects_with("{% apply upper %}body{% endfor %}", |e| {
        matches!(e, ParseError::UnexpectedEof(..) | ParseError::MismatchedEndTag { .. })
    });
}

#[test]
fn parse_mismatched_block_name_in_endblock() {
    rejects_with("{% block main %}x{% endblock other %}", |e| matches!(e, ParseError::MismatchedEndTag { .. }));
}

#[test]
fn parse_mismatched_name_in_endmacro() {
    rejects_with("{% macro foo() %}x{% endmacro bar %}", |e| matches!(e, ParseError::MismatchedEndTag { .. }));
}

#[test]
fn parse_empty_parens_rejected() {
    rejects_with("{{ () }}", |e| matches!(e, ParseError::UnexpectedToken(..) | ParseError::Message(..)));
}

#[test]
fn parse_missing_in_in_for() {
    rejects_with("{% for x list %}{% endfor %}", |e| matches!(e, ParseError::UnexpectedToken(..)));
}

#[test]
fn parse_missing_as_in_import() {
    rejects_with("{% import 'm.twig' %}", |e| {
        matches!(e, ParseError::UnexpectedToken(..) | ParseError::UnexpectedEof(..))
    });
}

#[test]
fn parse_missing_import_in_from() {
    rejects_with("{% from 'm.twig' %}", |e| {
        matches!(e, ParseError::UnexpectedToken(..) | ParseError::UnexpectedEof(..))
    });
}

#[test]
fn parse_deprecated_unknown_option() {
    rejects_with("{% deprecated 'msg' unknown='x' %}", |e| matches!(e, ParseError::UnexpectedToken(..)));
}

#[test]
fn parse_guard_unknown_kind() {
    rejects_with("{% guard what constant %}body{% endguard %}", |e| matches!(e, ParseError::UnexpectedToken(..)));
}

#[test]
fn parse_stray_variable_close_is_raw_text() {
    crate::common::parses("x }}");
}

#[test]
fn parse_macro_missing_parens() {
    rejects_with("{% macro foo %}body{% endmacro %}", |e| matches!(e, ParseError::UnexpectedToken(..)));
}

#[test]
fn parse_error_has_span() {
    let arena = Bump::new();
    let tpl = parse(&arena, "{% if a %}body");
    let err = tpl.errors.first().expect("expected a deferred error");
    use mago_span::HasSpan;
    let _ = err.span();
}

#[test]
fn parse_error_implements_display() {
    let arena = Bump::new();
    let tpl = parse(&arena, "{% if a %}body");
    let err = tpl.errors.first().expect("expected a deferred error");
    let msg = err.message();
    assert!(!msg.is_empty());
}

fn run_with_large_stack<F: FnOnce() + Send + 'static>(f: F) {
    std::thread::Builder::new()
        .stack_size(128 * 1024 * 1024)
        .spawn(f)
        .expect("spawn parser thread")
        .join()
        .expect("parser thread must not abort (no stack overflow)");
}

#[test]
#[cfg_attr(miri, ignore)]
fn parse_deeply_nested_brackets_does_not_overflow() {
    run_with_large_stack(|| {
        let mut src = String::from("{{ ");
        src.push_str(&"[".repeat(5000));
        src.push_str(" }}");
        rejects_with(&src, |e| matches!(e, ParseError::RecursionLimitExceeded(..)));
    });
}

#[test]
#[cfg_attr(miri, ignore)]
fn parse_deeply_nested_parens_does_not_overflow() {
    run_with_large_stack(|| {
        let mut src = String::from("{{ ");
        src.push_str(&"(".repeat(5000));
        src.push_str(" }}");
        rejects_with(&src, |e| matches!(e, ParseError::RecursionLimitExceeded(..)));
    });
}

#[test]
#[cfg_attr(miri, ignore)]
fn parse_fuzzer_nested_array_argument_does_not_overflow() {
    run_with_large_stack(|| {
        let mut src = String::from("{{ date(date = ");
        src.push_str(&"[".repeat(5000));
        src.push_str(") }}");
        let arena = Bump::new();
        let _ = parse(&arena, &src);
    });
}

#[test]
#[cfg_attr(miri, ignore)]
fn parse_deeply_nested_blocks_does_not_overflow() {
    run_with_large_stack(|| {
        let mut src = String::new();
        for _ in 0..5000 {
            src.push_str("{% if x %}");
        }
        let arena = Bump::new();
        let _ = parse(&arena, &src);
    });
}
