#![allow(clippy::unwrap_used, clippy::use_debug)]

use std::borrow::Cow;

use mago_allocator::LocalArena;
use mago_database::file::FileId;
use mago_syntax_core::input::Input;
use mago_twig_formatter::Formatter;
use mago_twig_formatter::settings::EndOfLine;
use mago_twig_formatter::settings::FormatSettings;
use mago_twig_syntax::lexer::TwigLexer;
use mago_twig_syntax::settings::LexerSettings;
use mago_twig_syntax::token::TwigTokenKind;

mod support;

fn check(source: &[u8], settings: FormatSettings) {
    let arena = LocalArena::new();
    let formatter = Formatter::new(&arena, settings);
    let output = formatter.format_code(Cow::Borrowed(b"test.twig"), Cow::Owned(source.to_vec())).unwrap();
    support::assert_preserves_syntax(source, output);
    let again = formatter.format_code(Cow::Borrowed(b"test.twig"), Cow::Owned(output.to_vec())).unwrap();
    assert_eq!(output, again, "Formatting must be idempotent");
}

#[test]
fn long_expression_chains_do_not_overflow() {
    for expression in [
        format!("{}1", "1 + ".repeat(2_000)),
        format!("x{}", "|default(0)".repeat(2_000)),
        format!("x{}", ".method()".repeat(2_000)),
        format!("x{}", "[0]".repeat(2_000)),
    ] {
        for source in [
            format!("{{{{ {expression} }}}}"),
            format!("{{% block long {expression} %}}"),
            format!("{{% set x = {expression} %}}"),
        ] {
            check(source.as_bytes(), FormatSettings { print_width: 30, ..FormatSettings::default() });
        }
    }
}

#[test]
fn very_long_chains_fit_a_small_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let source = format!("{{{{ x{} }}}}", "|f".repeat(100_000));
            check(source.as_bytes(), FormatSettings::default());
            let source = format!("{{{{ {}1 }}}}", "1 + ".repeat(100_000));
            check(source.as_bytes(), FormatSettings::default());
            let source = format!("{{{{ {} }}}}", "\"#{x}\" ".repeat(100_000));
            check(source.as_bytes(), FormatSettings::default());
            let source = format!("{{{{ x{} => 1 }}}}", "|f".repeat(100_000));
            let arena = LocalArena::new();
            Formatter::new(&arena, FormatSettings::default())
                .format_code(Cow::Borrowed(b"deep.twig"), Cow::Owned(source.into_bytes()))
                .unwrap_err();
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn deeply_nested_groups_and_comments_are_stable() {
    let source = format!("{{{{ {}x{} }}}}", "(".repeat(24), ")".repeat(24));
    let commented = source.replace('x', "x # keep\n");
    for source in [source, commented] {
        check(source.as_bytes(), FormatSettings { print_width: 30, ..FormatSettings::default() });
    }
}

#[test]
fn many_tags_on_one_line_keep_columns_and_template_text() {
    for tag in ["{{ name }}", "{# a comment #}"] {
        let source = format!("  <pre>{}</pre>\n", tag.repeat(5_000));
        for print_width in [30, 1_000_000] {
            check(source.as_bytes(), FormatSettings { print_width, ..FormatSettings::default() });
        }
    }
}

#[test]
fn excessive_nesting_returns_an_error() {
    let arena = LocalArena::new();
    let formatter = Formatter::new(&arena, FormatSettings::default());
    for source in [
        format!("{{{{ {}x{} }}}}", "(".repeat(5000), ")".repeat(5000)),
        format!("{}x{}", "{% if x %}".repeat(5000), "{% endif %}".repeat(5000)),
        format!("{}x{}", "{% for x in y %}".repeat(5000), "{% endfor %}".repeat(5000)),
    ] {
        formatter.format_code(Cow::Borrowed(b"deep.twig"), Cow::Owned(source.into_bytes())).unwrap_err();
    }
}

#[test]
fn newlines_inside_tags_follow_the_setting() {
    let source = include_bytes!("cases/wrapping/before.twig");
    let expected = include_str!("cases/wrapping/after.twig");
    for (setting, newline) in [(EndOfLine::Lf, "\n"), (EndOfLine::Crlf, "\r\n"), (EndOfLine::Cr, "\r")] {
        let source = String::from_utf8_lossy(source).replace('\n', newline);
        let expected = expected.replace('\n', newline);
        for end_of_line in [setting, EndOfLine::Auto] {
            let arena = LocalArena::new();
            let formatter =
                Formatter::new(&arena, FormatSettings { print_width: 30, end_of_line, ..FormatSettings::default() });
            let output = formatter
                .format_code(Cow::Borrowed(b"line-endings.twig"), Cow::Owned(source.as_bytes().to_vec()))
                .unwrap();
            assert_eq!(output, expected.as_bytes());
        }
    }
}

#[test]
fn inline_comments_keep_their_original_line_endings() {
    let source = include_str!("cases/commented_expressions/before.twig");
    for setting in [EndOfLine::Lf, EndOfLine::Crlf, EndOfLine::Cr] {
        for input_newline in ["\n", "\r\n", "\r"] {
            let input = source.replace('\n', input_newline);
            let arena = LocalArena::new();
            let formatter =
                Formatter::new(&arena, FormatSettings { end_of_line: setting, ..FormatSettings::default() });
            let output =
                formatter.format_code(Cow::Borrowed(b"comments.twig"), Cow::Owned(input.as_bytes().to_vec())).unwrap();
            let again = formatter.format_code(Cow::Borrowed(b"comments.twig"), Cow::Owned(output.to_vec())).unwrap();
            assert_eq!(output, again);
            assert_eq!(output, input.as_bytes());
        }
    }
}

fn expression(seed: u64, depth: u8) -> String {
    if depth == 0 {
        return ["x", "1", "'a  b'", "true", "null", "[1, 2]"][(seed as usize) % 6].to_string();
    }

    let next = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
    let left = expression(next, depth - 1);
    let right = expression(next.rotate_left(17), depth - 1);
    match seed % 9 {
        0 => format!("({left} + {right})"),
        1 => format!("({left} ?? {right})"),
        2 => format!("f({left}, named: {right})"),
        3 => format!("[{left}, {right}]"),
        4 => format!("{{'left': {left}, 'right': {right}}}"),
        5 => format!("(x ? {left} : {right})"),
        6 => format!("({left})|default({right})"),
        7 => format!("({left} is same as({right}))"),
        _ => format!("((x, y) => {left})"),
    }
}

#[test]
fn generated_expressions_keep_tokens_at_tiny_widths() {
    for seed in 0..64 {
        let source = format!("{{{{ {} }}}}", expression(seed, 3));
        let mut lexer = TwigLexer::new(Input::new(FileId::zero(), source.as_bytes()), LexerSettings);
        let mut gaps = Vec::new();
        while let Some(token) = lexer.advance() {
            let token = token.unwrap();
            if token.kind == TwigTokenKind::Whitespace {
                gaps.push(token.end().offset as usize);
            }
        }

        let mut variants = vec![source.clone()];
        for index in [0, gaps.len() / 2, gaps.len() - 1] {
            let mut commented = source.clone();
            commented.insert_str(gaps[index], "# keep\n");
            variants.push(commented);
        }

        for source in variants {
            for print_width in [0, 1, 8, 30, 120] {
                for (use_tabs, tab_width) in [(false, 0), (false, 2), (true, 0), (true, 4)] {
                    check(
                        source.as_bytes(),
                        FormatSettings { print_width, use_tabs, tab_width, ..FormatSettings::default() },
                    );
                }
            }
        }
    }
}
