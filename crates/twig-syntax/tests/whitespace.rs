//! Focused tests for whitespace-control markers.

#[path = "common/mod.rs"]
mod common;

use mago_twig_syntax::token::TwigTokenKind;

use crate::common::kinds;
use crate::common::lex;
use crate::common::parse_and_roundtrip;
use crate::common::roundtrip;

fn first_value_for(src: &str, kind: TwigTokenKind) -> String {
    lex(src).into_iter().find(|t| t.kind == kind).map(|t| t.value.to_string()).expect("kind not found")
}

fn first_comment(src: &str) -> String {
    lex(src)
        .into_iter()
        .find(|t| t.kind == TwigTokenKind::Comment)
        .map(|t| t.value.to_string())
        .expect("comment not found")
}

#[test]
fn block_open_trim() {
    assert_eq!(first_value_for("{%- do 1 %}", TwigTokenKind::OpenBlockDash), "{%-");
}

#[test]
fn block_close_trim() {
    assert_eq!(first_value_for("{% do 1 -%}", TwigTokenKind::CloseBlockDash), "-%}");
}

#[test]
fn block_both_sides_trim() {
    parse_and_roundtrip("{%- if a -%}b{%- endif -%}");
    assert_eq!(first_value_for("{%- if a -%}b{% endif %}", TwigTokenKind::OpenBlockDash), "{%-");
    assert_eq!(first_value_for("{%- if a -%}b{% endif %}", TwigTokenKind::CloseBlockDash), "-%}");
}

#[test]
fn variable_open_trim() {
    assert_eq!(first_value_for("{{- x }}", TwigTokenKind::OpenVariableDash), "{{-");
}

#[test]
fn variable_close_trim() {
    assert_eq!(first_value_for("{{ x -}}", TwigTokenKind::CloseVariableDash), "-}}");
}

#[test]
fn variable_both_sides_trim() {
    parse_and_roundtrip("{{- x -}}");
}

#[test]
fn comment_open_trim() {
    assert!(first_comment("{#- c #}").starts_with("{#-"));
}

#[test]
fn comment_close_trim() {
    assert!(first_comment("{# c -#}").ends_with("-#}"));
}

#[test]
fn comment_both_sides_trim() {
    parse_and_roundtrip("{#- c -#}");
}

#[test]
fn block_open_line_trim() {
    assert_eq!(first_value_for("{%~ do 1 %}", TwigTokenKind::OpenBlockTilde), "{%~");
}

#[test]
fn block_close_line_trim() {
    assert_eq!(first_value_for("{% do 1 ~%}", TwigTokenKind::CloseBlockTilde), "~%}");
}

#[test]
fn variable_open_line_trim() {
    assert_eq!(first_value_for("{{~ x }}", TwigTokenKind::OpenVariableTilde), "{{~");
}

#[test]
fn variable_close_line_trim() {
    assert_eq!(first_value_for("{{ x ~}}", TwigTokenKind::CloseVariableTilde), "~}}");
}

#[test]
fn comment_open_line_trim() {
    assert!(first_comment("{#~ c #}").starts_with("{#~"));
}

#[test]
fn comment_close_line_trim() {
    assert!(first_comment("{# c ~#}").ends_with("~#}"));
}

#[test]
fn asymmetric_block_only_open() {
    parse_and_roundtrip("{%- if a %}body{% endif %}");
}

#[test]
fn asymmetric_block_only_close() {
    parse_and_roundtrip("{% if a -%}body{% endif %}");
}

#[test]
fn asymmetric_variable_only_open() {
    parse_and_roundtrip("{{- x }}");
}

#[test]
fn asymmetric_variable_only_close() {
    parse_and_roundtrip("{{ x -}}");
}

#[test]
fn trim_around_for_loop_output() {
    parse_and_roundtrip("{% for i in list %}{{- i -}}{% endfor %}");
}

#[test]
fn trim_around_if_bodies() {
    parse_and_roundtrip("{%- if a -%}A{%- else -%}B{%- endif -%}");
}

#[test]
fn trim_preserves_raw_text_surrounding_space() {
    roundtrip("pre  {%- do 1 -%}  mid  {%- do 2 -%}  post");
}

#[test]
fn whitespace_kinds_outside_expressions_are_raw_text() {
    let k = kinds("  {% do 1 %}  ");
    let outside_whitespace = k
        .windows(2)
        .any(|pair| matches!(pair[0], TwigTokenKind::RawText) && matches!(pair[1], TwigTokenKind::Whitespace));
    assert!(!outside_whitespace);
}
