//! Focused tests for the Twig lexer.

#[path = "common/mod.rs"]
mod common;

use mago_twig_syntax::error::SyntaxError;
use mago_twig_syntax::token::TwigTokenKind;

use crate::common::find_first_token;
use crate::common::kinds;
use crate::common::kinds_and_values;
use crate::common::lex;
use crate::common::roundtrip;
use crate::common::tokenize;

#[test]
fn plain_text_is_raw_text() {
    let toks = lex("hello world");
    assert_eq!(toks.len(), 1);
    assert_eq!(toks[0].kind, TwigTokenKind::RawText);
    assert_eq!(toks[0].value, "hello world");
}

#[test]
fn empty_input_yields_no_tokens() {
    let toks = lex("");
    assert!(toks.is_empty());
}

#[test]
fn raw_text_preserves_newlines_verbatim() {
    let src = "line1\nline2\nline3";
    roundtrip(src);
    let toks = lex(src);
    assert_eq!(toks[0].kind, TwigTokenKind::RawText);
    assert_eq!(toks[0].value, src);
}

#[test]
fn raw_text_may_contain_bytes_that_look_like_tag_opens_partial() {
    roundtrip("{ not a tag }");
    roundtrip("just a { brace");
}

#[test]
fn transition_data_to_block_and_back() {
    let k = kinds("x {% do 1 %} y");
    let k_significant: Vec<_> = k.into_iter().filter(|x| !matches!(x, TwigTokenKind::Whitespace)).collect();
    assert_eq!(
        k_significant,
        vec![
            TwigTokenKind::RawText,
            TwigTokenKind::OpenBlock,
            TwigTokenKind::Name,
            TwigTokenKind::Number,
            TwigTokenKind::CloseBlock,
            TwigTokenKind::RawText,
        ]
    );
}

#[test]
fn transition_data_to_variable_and_back() {
    let k: Vec<_> = kinds("x {{ y }} z").into_iter().filter(|x| !matches!(x, TwigTokenKind::Whitespace)).collect();
    assert_eq!(
        k,
        vec![
            TwigTokenKind::RawText,
            TwigTokenKind::OpenVariable,
            TwigTokenKind::Name,
            TwigTokenKind::CloseVariable,
            TwigTokenKind::RawText,
        ]
    );
}

#[test]
fn transition_data_to_comment_and_back() {
    let k = kinds("x {# c #} y");
    assert!(k.contains(&TwigTokenKind::Comment));
}

#[test]
fn transition_variable_to_dq_string_and_back() {
    let toks = lex(r#"{{ "hello" }}"#);
    let kinds: Vec<_> = toks.iter().map(|t| t.kind).filter(|k| !matches!(k, TwigTokenKind::Whitespace)).collect();
    assert_eq!(
        kinds,
        vec![TwigTokenKind::OpenVariable, TwigTokenKind::StringDoubleQuoted, TwigTokenKind::CloseVariable]
    );
}

#[test]
fn transition_variable_to_interpolating_string() {
    let toks = lex(r#"{{ "a#{b}c" }}"#);
    let kinds: Vec<_> = toks.iter().map(|t| t.kind).filter(|k| !matches!(k, TwigTokenKind::Whitespace)).collect();
    assert_eq!(
        kinds,
        vec![
            TwigTokenKind::OpenVariable,
            TwigTokenKind::DoubleQuoteStart,
            TwigTokenKind::StringPart,
            TwigTokenKind::InterpolationStart,
            TwigTokenKind::Name,
            TwigTokenKind::InterpolationEnd,
            TwigTokenKind::StringPart,
            TwigTokenKind::DoubleQuoteEnd,
            TwigTokenKind::CloseVariable,
        ]
    );
}

#[test]
fn nested_interpolation_tokens() {
    let src = r#"{{ "a#{"b#{c}d"}e" }}"#;
    roundtrip(src);
    let toks = lex(src);
    let dq_start = toks.iter().filter(|t| t.kind == TwigTokenKind::DoubleQuoteStart).count();
    let dq_end = toks.iter().filter(|t| t.kind == TwigTokenKind::DoubleQuoteEnd).count();
    assert_eq!(dq_start, 2);
    assert_eq!(dq_end, 2);
}

#[test]
fn nested_interpolation_with_concat() {
    let src = r#"{{ "a #{ x ~ '!' } b" }}"#;
    roundtrip(src);
    let toks = lex(src);
    assert!(toks.iter().any(|t| t.kind == TwigTokenKind::InterpolationStart));
    assert!(toks.iter().any(|t| t.kind == TwigTokenKind::InterpolationEnd));
}

#[test]
fn single_quoted_string_is_atomic() {
    let toks = lex(r#"{{ 'foo' }}"#);
    let s = toks.iter().find(|t| t.kind == TwigTokenKind::StringSingleQuoted).unwrap();
    assert_eq!(s.value, "'foo'");
}

#[test]
fn single_quoted_string_with_escape() {
    let src = r#"{{ 'it\'s' }}"#;
    roundtrip(src);
    let toks = lex(src);
    let s = toks.iter().find(|t| t.kind == TwigTokenKind::StringSingleQuoted).unwrap();
    assert_eq!(s.value, r#"'it\'s'"#);
}

#[test]
fn double_quoted_string_with_escape() {
    let src = r#"{{ "a\"b" }}"#;
    roundtrip(src);
    let toks = lex(src);
    let s = toks.iter().find(|t| t.kind == TwigTokenKind::StringDoubleQuoted).unwrap();
    assert_eq!(s.value, r#""a\"b""#);
}

#[test]
fn double_quoted_without_interpolation_is_atomic() {
    let toks = lex(r#"{{ "plain" }}"#);
    assert!(toks.iter().any(|t| t.kind == TwigTokenKind::StringDoubleQuoted && t.value == r#""plain""#));
    assert!(!toks.iter().any(|t| t.kind == TwigTokenKind::StringPart));
    assert!(!toks.iter().any(|t| t.kind == TwigTokenKind::DoubleQuoteStart));
}

#[test]
fn double_quoted_with_interpolation_is_structured() {
    let toks = lex(r#"{{ "hi #{name}" }}"#);
    assert!(toks.iter().any(|t| t.kind == TwigTokenKind::DoubleQuoteStart));
    assert!(toks.iter().any(|t| t.kind == TwigTokenKind::DoubleQuoteEnd));
    assert!(toks.iter().any(|t| t.kind == TwigTokenKind::StringPart));
    assert!(toks.iter().any(|t| t.kind == TwigTokenKind::InterpolationStart));
    assert!(toks.iter().any(|t| t.kind == TwigTokenKind::InterpolationEnd));
}

#[test]
fn whitespace_control_block_open() {
    let t = find_first_token("{%- if a %}b{% endif %}", |t| t.kind == TwigTokenKind::OpenBlockDash).unwrap();
    assert_eq!(t.value, "{%-");
}

#[test]
fn whitespace_control_block_close() {
    let toks = lex("{% if a -%}b{% endif %}");
    let ends: Vec<_> = toks.iter().filter(|t| t.kind == TwigTokenKind::CloseBlockDash).collect();
    assert!(ends.iter().any(|t| t.value == "-%}"), "expected `-%}}` marker to be emitted, got {ends:?}");
}

#[test]
fn whitespace_control_variable_open() {
    let t = find_first_token("{{- x }}", |t| t.kind == TwigTokenKind::OpenVariableDash).unwrap();
    assert_eq!(t.value, "{{-");
}

#[test]
fn whitespace_control_variable_close() {
    let t = find_first_token("{{ x -}}", |t| t.kind == TwigTokenKind::CloseVariableDash).unwrap();
    assert_eq!(t.value, "-}}");
}

#[test]
fn whitespace_control_comment_open() {
    let t = find_first_token("{#- c #}", |t| t.kind == TwigTokenKind::Comment).unwrap();
    assert!(t.value.starts_with("{#-"));
}

#[test]
fn whitespace_control_comment_close() {
    let t = find_first_token("{# c -#}", |t| t.kind == TwigTokenKind::Comment).unwrap();
    assert!(t.value.ends_with("-#}"));
}

#[test]
fn line_trim_tilde_block_open() {
    let t = find_first_token("{%~ if a %}b{% endif %}", |t| t.kind == TwigTokenKind::OpenBlockTilde).unwrap();
    assert_eq!(t.value, "{%~");
}

#[test]
fn line_trim_tilde_variable_close() {
    let t = find_first_token("{{ x ~}}", |t| t.kind == TwigTokenKind::CloseVariableTilde).unwrap();
    assert_eq!(t.value, "~}}");
}

#[test]
fn verbatim_body_is_preserved_as_verbatim_text() {
    let toks = lex("{% verbatim %}{{ foo }}{% endverbatim %}");
    let body = toks.iter().find(|t| t.kind == TwigTokenKind::VerbatimText).unwrap();
    assert_eq!(body.value, "{{ foo }}");
}

#[test]
fn raw_alias_preserved_as_verbatim_text() {
    let toks = lex("{% raw %}{{ foo }}{% endraw %}");
    let body = toks.iter().find(|t| t.kind == TwigTokenKind::VerbatimText).unwrap();
    assert_eq!(body.value, "{{ foo }}");
}

#[test]
fn verbatim_empty_body_emits_no_verbatim_text() {
    let toks = lex("{% verbatim %}{% endverbatim %}");
    assert!(!toks.iter().any(|t| t.kind == TwigTokenKind::VerbatimText));
}

#[test]
fn verbatim_whitespace_around_tag_roundtrips() {
    roundtrip("x {% verbatim %}y{% endverbatim %} z");
}

#[test]
fn comment_body_may_contain_close_looking_bytes() {
    let src = "{# body with %} inside #}";
    roundtrip(src);
    let toks = lex(src);
    let c = toks.iter().find(|t| t.kind == TwigTokenKind::Comment).unwrap();
    assert_eq!(c.value, src);
}

#[test]
fn comment_body_may_span_multiple_lines() {
    let src = "{# a\nb\nc #}";
    roundtrip(src);
    let toks = lex(src);
    let c = toks.iter().find(|t| t.kind == TwigTokenKind::Comment).unwrap();
    assert!(c.value.contains('\n'));
}

#[test]
fn empty_comment_body_is_ok() {
    roundtrip("{##}");
    let toks = lex("{##}");
    let c = toks.iter().find(|t| t.kind == TwigTokenKind::Comment).unwrap();
    assert_eq!(c.value, "{##}");
}

#[test]
fn inline_comment_inside_expression() {
    let toks = lex("{{ a # trailing\n + b }}");
    let c = toks.iter().find(|t| t.kind == TwigTokenKind::InlineComment).unwrap();
    assert_eq!(c.value, "# trailing");
}

#[test]
fn inline_comment_with_trailing_newline_terminates_at_newline() {
    let toks = lex("{{ a # to-eol\n }}").into_iter().collect::<Vec<_>>();
    let c = toks.iter().find(|t| t.kind == TwigTokenKind::InlineComment).unwrap();
    assert_eq!(c.value, "# to-eol");
}

fn first_kind_of(src: &str, kind: TwigTokenKind) -> String {
    lex(src).into_iter().find(|t| t.kind == kind).map(|t| t.value.to_string()).unwrap_or_default()
}

#[test]
fn operator_equal_equal() {
    assert_eq!(first_kind_of("{{ a == b }}", TwigTokenKind::EqualEqual), "==");
}

#[test]
fn operator_bang_equal() {
    assert_eq!(first_kind_of("{{ a != b }}", TwigTokenKind::BangEqual), "!=");
}

#[test]
fn operator_less_equal() {
    assert_eq!(first_kind_of("{{ a <= b }}", TwigTokenKind::LessThanEqual), "<=");
}

#[test]
fn operator_greater_equal() {
    assert_eq!(first_kind_of("{{ a >= b }}", TwigTokenKind::GreaterThanEqual), ">=");
}

#[test]
fn operator_spaceship() {
    assert_eq!(first_kind_of("{{ a <=> b }}", TwigTokenKind::Spaceship), "<=>");
}

#[test]
fn operator_identity_triple_equal() {
    assert_eq!(first_kind_of("{{ a === b }}", TwigTokenKind::EqualEqualEqual), "===");
}

#[test]
fn operator_non_identity_bang_double_equal() {
    assert_eq!(first_kind_of("{{ a !== b }}", TwigTokenKind::BangEqualEqual), "!==");
}

#[test]
fn operator_floor_div() {
    assert_eq!(first_kind_of("{{ a // b }}", TwigTokenKind::SlashSlash), "//");
}

#[test]
fn operator_pow() {
    assert_eq!(first_kind_of("{{ a ** b }}", TwigTokenKind::AsteriskAsterisk), "**");
}

#[test]
fn operator_range() {
    assert_eq!(first_kind_of("{{ 1..10 }}", TwigTokenKind::DotDot), "..");
}

#[test]
fn operator_null_coalesce() {
    assert_eq!(first_kind_of("{{ a ?? b }}", TwigTokenKind::QuestionQuestion), "??");
}

#[test]
fn operator_elvis() {
    assert_eq!(first_kind_of("{{ a ?: b }}", TwigTokenKind::QuestionColon), "?:");
}

#[test]
fn operator_null_safe() {
    assert_eq!(first_kind_of("{{ a?.b }}", TwigTokenKind::QuestionDot), "?.");
}

#[test]
fn operator_spread() {
    let toks = lex("{{ f(...xs) }}");
    assert!(toks.iter().any(|t| t.kind == TwigTokenKind::DotDotDot && t.value == "..."));
}

#[test]
fn operator_arrow() {
    assert_eq!(first_kind_of("{{ v => v + 1 }}", TwigTokenKind::FatArrow), "=>");
}

#[test]
fn operator_b_and() {
    assert_eq!(first_kind_of("{{ a b-and b }}", TwigTokenKind::BAnd), "b-and");
}

#[test]
fn operator_b_or() {
    assert_eq!(first_kind_of("{{ a b-or b }}", TwigTokenKind::BOr), "b-or");
}

#[test]
fn operator_b_xor() {
    assert_eq!(first_kind_of("{{ a b-xor b }}", TwigTokenKind::BXor), "b-xor");
}

#[test]
fn operator_not_in() {
    let toks = lex("{{ a not in b }}");
    assert!(toks.iter().any(|t| t.kind == TwigTokenKind::NotIn));
}

#[test]
fn operator_starts_with() {
    let toks = lex("{{ 'a' starts with 'a' }}");
    assert!(toks.iter().any(|t| t.kind == TwigTokenKind::StartsWith));
}

#[test]
fn operator_ends_with() {
    let toks = lex("{{ 'a' ends with 'a' }}");
    assert!(toks.iter().any(|t| t.kind == TwigTokenKind::EndsWith));
}

#[test]
fn operator_has_some() {
    let toks = lex("{{ xs has some ys }}");
    assert!(toks.iter().any(|t| t.kind == TwigTokenKind::HasSome));
}

#[test]
fn operator_has_every() {
    let toks = lex("{{ xs has every ys }}");
    assert!(toks.iter().any(|t| t.kind == TwigTokenKind::HasEvery));
}

#[test]
fn operator_same_as() {
    let toks = lex("{{ a is same as(b) }}");
    assert!(toks.iter().any(|t| t.kind == TwigTokenKind::SameAs));
}

#[test]
fn operator_divisible_by() {
    let toks = lex("{{ a is divisible by(3) }}");
    assert!(toks.iter().any(|t| t.kind == TwigTokenKind::DivisibleBy));
}

#[test]
fn word_operator_and() {
    assert_eq!(first_kind_of("{{ a and b }}", TwigTokenKind::And), "and");
}

#[test]
fn word_operator_or() {
    assert_eq!(first_kind_of("{{ a or b }}", TwigTokenKind::Or), "or");
}

#[test]
fn word_operator_xor() {
    assert_eq!(first_kind_of("{{ a xor b }}", TwigTokenKind::Xor), "xor");
}

#[test]
fn word_operator_matches() {
    assert_eq!(first_kind_of("{{ a matches '/x/' }}", TwigTokenKind::Matches), "matches");
}

#[test]
fn number_integer() {
    let toks = lex("{{ 123 }}");
    let n = toks.iter().find(|t| t.kind == TwigTokenKind::Number).unwrap();
    assert_eq!(n.value, "123");
}

#[test]
fn number_float() {
    let toks = lex("{{ 3.14 }}");
    let n = toks.iter().find(|t| t.kind == TwigTokenKind::Number).unwrap();
    assert_eq!(n.value, "3.14");
}

#[test]
fn number_with_underscore_separators() {
    let toks = lex("{{ 1_000_000 }}");
    let n = toks.iter().find(|t| t.kind == TwigTokenKind::Number).unwrap();
    assert_eq!(n.value, "1_000_000");
}

#[test]
fn number_with_exponent() {
    let toks = lex("{{ 1.5e10 }}");
    let n = toks.iter().find(|t| t.kind == TwigTokenKind::Number).unwrap();
    assert!(n.value.starts_with("1.5e"), "got {:?}", n.value);
}

#[test]
fn names_may_contain_underscores_and_digits() {
    let toks = lex("{{ _foo_bar1 }}");
    assert!(toks.iter().any(|t| t.kind == TwigTokenKind::Name && t.value == "_foo_bar1"));
}

#[test]
fn name_after_dot_is_not_a_word_operator() {
    let toks = lex("{{ a.not }}");
    let idx = toks.iter().position(|t| t.kind == TwigTokenKind::Dot).unwrap();
    let after_dot = &toks[idx + 1..].iter().find(|t| t.kind != TwigTokenKind::Whitespace).unwrap();
    assert_eq!(after_dot.kind, TwigTokenKind::Name);
    assert_eq!(after_dot.value, "not");
}

#[test]
fn name_after_pipe_is_not_a_word_operator() {
    let toks = lex("{{ x|in(xs) }}");
    let idx = toks.iter().position(|t| t.kind == TwigTokenKind::Pipe).unwrap();
    let after_pipe = &toks[idx + 1..].iter().find(|t| t.kind != TwigTokenKind::Whitespace).unwrap();
    assert_eq!(after_pipe.kind, TwigTokenKind::Name);
    assert_eq!(after_pipe.value, "in");
}

#[test]
fn punctuation_pipe_dot_colon_comma_question() {
    let src = "{{ a|b.c, d ? e : f }}";
    let toks = lex(src);
    assert!(toks.iter().any(|t| t.kind == TwigTokenKind::Pipe && t.value == "|"));
    assert!(toks.iter().any(|t| t.kind == TwigTokenKind::Dot && t.value == "."));
    assert!(toks.iter().any(|t| t.kind == TwigTokenKind::Colon && t.value == ":"));
    assert!(toks.iter().any(|t| t.kind == TwigTokenKind::Comma && t.value == ","));
    assert!(toks.iter().any(|t| t.kind == TwigTokenKind::Question && t.value == "?"));
}

#[test]
fn brace_emits_left_brace_for_hash_literal_open() {
    let toks = lex("{{ {} }}");
    let opener = toks.iter().find(|t| t.value == "{" && t.kind == TwigTokenKind::LeftBrace).unwrap();
    assert_eq!(opener.kind, TwigTokenKind::LeftBrace);
}

#[test]
fn span_of_mid_template_name_matches_offsets() {
    let src = "Hello, {{ name }}!";
    let toks = lex(src);
    let name = toks.iter().find(|t| t.kind == TwigTokenKind::Name).unwrap();
    assert_eq!(name.value, "name");
    assert_eq!(name.start.offset as usize, src.find("name").unwrap());
    assert_eq!(name.end().offset as usize, src.find("name").unwrap() + "name".len());
}

#[test]
fn span_of_variable_start_covers_exact_bytes() {
    let src = "x {{ a }}";
    let toks = lex(src);
    let vs = toks.iter().find(|t| t.kind == TwigTokenKind::OpenVariable).unwrap();
    assert_eq!(&src[vs.start.offset as usize..vs.end().offset as usize], "{{");
}

#[test]
fn span_of_raw_text_is_exact_slice() {
    let src = "abc{% do 1 %}";
    let toks = lex(src);
    let rt = toks.iter().find(|t| t.kind == TwigTokenKind::RawText).unwrap();
    assert_eq!(&src[rt.start.offset as usize..rt.end().offset as usize], "abc");
}

#[test]
fn unterminated_block_open_is_syntax_error() {
    let err = tokenize("{% if a ").unwrap_err();
    assert!(matches!(err, SyntaxError::UnclosedTag(..) | SyntaxError::UnclosedBracket(..)), "got {err:?}");
}

#[test]
fn unterminated_variable_open_is_syntax_error() {
    let err = tokenize("{{ a").unwrap_err();
    assert!(matches!(err, SyntaxError::UnclosedTag(..)), "got {err:?}");
}

#[test]
fn unterminated_comment_open_is_syntax_error() {
    let err = tokenize("{# no close").unwrap_err();
    assert!(matches!(err, SyntaxError::UnclosedComment(..)), "got {err:?}");
}

#[test]
fn unterminated_single_quoted_string_is_syntax_error() {
    let err = tokenize("{{ 'hello }}").unwrap_err();
    assert!(matches!(err, SyntaxError::UnclosedString(..)), "got {err:?}");
}

#[test]
fn unterminated_double_quoted_string_is_syntax_error() {
    let err = tokenize(r#"{{ "hello }}"#).unwrap_err();
    assert!(matches!(err, SyntaxError::UnclosedString(..)), "got {err:?}");
}

#[test]
fn unterminated_interpolating_double_quoted_string_is_syntax_error() {
    let err = tokenize(r#"{{ "x #{ y "#).unwrap_err();
    assert!(
        matches!(
            err,
            SyntaxError::UnclosedBracket(..) | SyntaxError::UnclosedTag(..) | SyntaxError::UnclosedString(..)
        ),
        "got {err:?}"
    );
}

#[test]
fn unterminated_verbatim_is_syntax_error() {
    let err = tokenize("{% verbatim %}contents with no end").unwrap_err();
    assert!(matches!(err, SyntaxError::UnclosedVerbatim(..)), "got {err:?}");
}

#[test]
fn unmatched_closing_paren_is_syntax_error() {
    let err = tokenize("{{ a ) }}").unwrap_err();
    assert!(matches!(err, SyntaxError::UnmatchedBracket(..)), "got {err:?}");
}

#[test]
fn unexpected_character_inside_expression() {
    let err = tokenize("{{ @ }}").unwrap_err();
    assert!(matches!(err, SyntaxError::UnexpectedCharacter(..)), "got {err:?}");
}

#[test]
fn lossless_roundtrip_mixed_content() {
    let srcs = [
        "hello",
        "{{ a }}",
        "{% if a %}x{% endif %}",
        "{# c #}",
        "{% verbatim %}{{ raw }}{% endverbatim %}",
        r#"{{ "a #{b} c" }}"#,
        "{{- x -}}",
        "{%- if a -%}b{%- endif -%}",
        "{%~ if a ~%}b{%~ endif ~%}",
    ];
    for s in srcs {
        roundtrip(s);
    }
}

#[test]
fn token_values_for_simple_print() {
    let kvs = kinds_and_values("{{ x }}");
    let names: Vec<_> = kvs.into_iter().filter(|(k, _)| !matches!(k, TwigTokenKind::Whitespace)).collect();
    assert_eq!(
        names,
        vec![
            (TwigTokenKind::OpenVariable, "{{".to_string()),
            (TwigTokenKind::Name, "x".to_string()),
            (TwigTokenKind::CloseVariable, "}}".to_string()),
        ]
    );
}

#[test]
fn trivia_classification() {
    assert!(TwigTokenKind::Whitespace.is_trivia());
    assert!(TwigTokenKind::InlineComment.is_trivia());
    assert!(!TwigTokenKind::Name.is_trivia());
    assert!(!TwigTokenKind::Plus.is_trivia());
    assert!(!TwigTokenKind::RawText.is_trivia());
}

#[test]
fn string_classification() {
    assert!(TwigTokenKind::StringSingleQuoted.is_string());
    assert!(TwigTokenKind::StringDoubleQuoted.is_string());
    assert!(!TwigTokenKind::StringPart.is_string());
    assert!(!TwigTokenKind::Name.is_string());
}
