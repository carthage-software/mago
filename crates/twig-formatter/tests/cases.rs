#![allow(clippy::unwrap_used, clippy::use_debug)]

use std::borrow::Cow;

use mago_allocator::LocalArena;
use mago_twig_formatter::Formatter;
use mago_twig_formatter::settings::EndOfLine;
use mago_twig_formatter::settings::FormatSettings;

mod support;

macro_rules! test_case {
    ($name:ident) => {
        #[test]
        fn $name() {
            let before = include_bytes!(concat!("cases/", stringify!($name), "/before.twig"));
            let after = include_bytes!(concat!("cases/", stringify!($name), "/after.twig"));
            let settings: FormatSettings = include!(concat!("cases/", stringify!($name), "/settings.inc"));
            let arena = LocalArena::new();
            let formatter = Formatter::new(&arena, settings);
            let result = formatter.format_code(Cow::Borrowed(b"case.twig"), Cow::Borrowed(before)).unwrap();
            support::assert_preserves_syntax(before, result);
            pretty_assertions::assert_eq!(String::from_utf8_lossy(after), String::from_utf8_lossy(result));
            let again = formatter.format_code(Cow::Borrowed(b"case.twig"), Cow::Owned(result.to_vec())).unwrap();
            assert_eq!(result, again, "Formatting must be idempotent");
        }
    };
}

test_case!(expressions);
test_case!(blocks);
test_case!(tags);
test_case!(whitespace_control);
test_case!(verbatim);
test_case!(comments);
test_case!(html);
test_case!(ignore);
test_case!(ignore_file);
test_case!(strings);
test_case!(tabs);
test_case!(preserve_indentation);
test_case!(wrapping);
test_case!(crlf);
test_case!(indent_html_context);
test_case!(indent_inline_chain);
test_case!(tabs_with_alignment);
test_case!(tab_width_boundary);
test_case!(empty_trim_comments);
test_case!(ignore_for_else);
test_case!(unknown_tags);
test_case!(guard_and_types);
test_case!(commented_expressions);
test_case!(mixed_line_endings);
test_case!(inline_tab_alignment);
test_case!(ignore_boundaries);
test_case!(keyword_names);
test_case!(word_operators_after_dots);
test_case!(reserved_names);
test_case!(option_order);
test_case!(keyword_separators);
test_case!(adjacent_strings);
test_case!(compound_word_names);
test_case!(mixed_tab_stops);
test_case!(legacy_raw);

#[test]
fn invalid_templates_are_rejected() {
    for source in [
        b"{% if a %}".as_slice(),
        b"{{ foo( }}",
        b"{# missing end",
        b"{% custom_tag foo",
        b"{{ true => 1 }}",
        b"{{ (a.b) => -1 }}",
        b"{{ 'a' 'b' }}",
        b"{{ \"#{name}x\" 'b' }}",
        b"{{ 'a' \"x#{name}\" }}",
        b"{% types value ? : 'string' %}",
    ] {
        let arena = LocalArena::new();
        let formatter = Formatter::new(&arena, FormatSettings::default());
        formatter.format_code(Cow::Borrowed(b"invalid.twig"), Cow::Borrowed(source)).unwrap_err();
    }
}

#[test]
fn preserves_non_utf8_text_and_strings() {
    let arena = LocalArena::new();
    let formatter = Formatter::new(&arena, FormatSettings::default());
    let result = formatter.format_code(Cow::Borrowed(b"bytes.twig"), Cow::Borrowed(b"\xff {{ '\xfe' }} \0")).unwrap();
    assert_eq!(result, b"\xff {{ '\xfe' }} \0");
}
