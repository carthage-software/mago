#![allow(clippy::unwrap_used)]

use std::borrow::Cow;

use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_formatter::Formatter;
use mago_formatter::settings::FormatSettings;
use mago_php_version::PHPVersion;

fn apply(source: &str, selection: &str, settings: FormatSettings) -> String {
    let arena = LocalArena::new();
    let formatter = Formatter::new(&arena, PHPVersion::PHP85, settings);
    let file = File::ephemeral(Cow::Borrowed(b"range.php"), source.as_bytes().to_vec().into());
    let start = source.find(selection).unwrap() as u32;
    let edit = formatter.format_range(&file, start..start + selection.len() as u32).unwrap().unwrap();
    assert!(
        edit.range.start <= start && edit.range.end >= start + selection.len() as u32,
        "replacement must cover the selected text"
    );
    let mut result = source.as_bytes().to_vec();
    result.splice(edit.range.start as usize..edit.range.end as usize, edit.new_text.iter().copied());
    String::from_utf8(result).unwrap()
}

#[test]
fn partial_expression_formats_the_statement_and_quotes() {
    let source = "<?php\n$before=  1;\n$selected= [\"value\" ,2 ];\n$after=  3;\n";
    let result = apply(source, "value", FormatSettings { single_quote: true, ..FormatSettings::default() });
    assert_eq!(result, "<?php\n$before=  1;\n$selected = ['value', 2];\n$after=  3;\n");
}

#[test]
fn class_member_uses_parent_indentation() {
    let source = "<?php\nclass A {\n public   int $value=1;\n public int $other=  2;\n}\n";
    assert_eq!(
        apply(source, "$value", FormatSettings::default()),
        "<?php\nclass A {\n    public int $value = 1;\n public int $other=  2;\n}\n"
    );
}

#[test]
fn leading_and_trailing_comments_stay_with_the_statement() {
    let source = "<?php\n$before=  1;\n// before selected\n$x=1; // after selected\n$after=  2;\n";
    assert_eq!(
        apply(source, "$x", FormatSettings::default()),
        "<?php\n$before=  1;\n// before selected\n$x = 1; // after selected\n$after=  2;\n"
    );
}

#[test]
fn selecting_an_import_formats_the_whole_import_group() {
    let source = "<?php\nuse Z\\B;use A\\C;\n$x=  1;\n";
    assert_eq!(apply(source, "Z\\B", FormatSettings::default()), "<?php\nuse A\\C;\nuse Z\\B;\n$x=  1;\n");
}

#[test]
fn template_php_preserves_surrounding_html() {
    let source = "<p><?php echo   \"yes\" ; ?></p>\n";
    assert_eq!(
        apply(source, "yes", FormatSettings { single_quote: true, ..FormatSettings::default() }),
        "<p><?php echo 'yes'; ?></p>\n"
    );
}

#[test]
fn narrow_groups_and_line_suffixes_keep_their_bounds() {
    let source = "<?php\nfunction f() {\n$x=call_function(111111,222222,333333); // suffix\n$after=  2;\n}\n";
    let result = apply(source, "call_function", FormatSettings { print_width: 30, ..FormatSettings::default() });
    assert!(result.starts_with("<?php\nfunction f() {\n    $x = call_function(\n"), "{result}");
    assert!(result.ends_with(" // suffix\n$after=  2;\n}\n"), "{result}");
    assert_eq!(result.matches("// suffix").count(), 1);
}

#[test]
fn heredoc_uses_formatter_indentation() {
    let source = "<?php\n$before=  1;\n$text=<<<TXT\nhello $name\nTXT;\n$after=  2;\n";
    assert_eq!(
        apply(source, "$text", FormatSettings::default()),
        "<?php\n$before=  1;\n$text = <<<TXT\n    hello $name\n    TXT;\n$after=  2;\n"
    );
    assert_eq!(
        apply(source, "$text", FormatSettings { indent_heredoc: false, ..FormatSettings::default() }),
        "<?php\n$before=  1;\n$text = <<<TXT\nhello $name\nTXT;\n$after=  2;\n"
    );
}

#[test]
fn invalid_ignored_and_unchanged_ranges_return_no_edit() {
    let arena = LocalArena::new();
    let formatter = Formatter::new(&arena, PHPVersion::PHP85, FormatSettings::default());
    let file = File::ephemeral(Cow::Borrowed(b"range.php"), Cow::Borrowed(b"<?php\n$x = 1;\n"));
    for (start, end) in [(7, 6), (0, 99), (99, 99), (5, 6), (6, 13), (6, 6), (5, 5), (14, 14)] {
        assert!(formatter.format_range(&file, start..end).unwrap().is_none(), "unexpected edit for {start}..{end}");
    }
    let html = File::ephemeral(Cow::Borrowed(b"range.php"), Cow::Borrowed(b"<p>text</p>"));
    assert!(formatter.format_range(&html, 3..7).unwrap().is_none(), "HTML-only selections should have no edit");
    let ignored =
        File::ephemeral(Cow::Borrowed(b"range.php"), Cow::Borrowed(b"<?php\n// @mago-format-ignore-next\n$x=1;"));
    assert!(formatter.format_range(&ignored, 33..35).unwrap().is_none(), "ignored statements should have no edit");
    let invalid = File::ephemeral(Cow::Borrowed(b"range.php"), Cow::Borrowed(b"<?php $x = ;"));
    formatter.format_range(&invalid, 6..8).unwrap_err();
}
