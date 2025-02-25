use mago_formatter::settings::FormatSettings;
use mago_interner::ThreadedInterner;
use mago_parser::parse_source;
use mago_php_version::PHPVersion;
use mago_source::Source;

pub mod comment;
pub mod format;
pub mod parens;

/// Test that the given code is formatted to the expected result.
///
/// This function will parse the given code, format it, parse the formatted code, and then format it again
/// to ensure that the formatter is idempotent.
///
/// # Arguments
///
/// * `code` - The code to format
/// * `expected` - The expected result of formatting the code
/// * `settings` - The settings to use when formatting the code
pub fn test_format(code: impl AsRef<str>, expected: &str, settings: FormatSettings) {
    test_format_with_version(code, expected, PHPVersion::PHP84, settings);
}

/// Test that the given code is formatted to the expected result.
///
/// This function will parse the given code, format it, parse the formatted code, and then format it again
/// to ensure that the formatter is idempotent.
///
/// # Arguments
///
/// * `code` - The code to format
/// * `expected` - The expected result of formatting the code
/// * `php_version` - The PHP version to use when formatting the code
/// * `settings` - The settings to use when formatting the code
pub fn test_format_with_version(
    code: impl AsRef<str>,
    expected: &str,
    php_version: PHPVersion,
    settings: FormatSettings,
) {
    let interner = ThreadedInterner::new();

    let code_source = Source::standalone(&interner, "code.php", code.as_ref());
    let (code_program, error) = parse_source(&interner, &code_source);
    assert_eq!(error, None, "Error parsing code");
    let formatted_code = mago_formatter::format(&interner, &code_source, &code_program, php_version, settings);
    pretty_assertions::assert_eq!(expected, formatted_code, "Formatted code does not match expected");

    let formatted_code_source = Source::standalone(&interner, "formatted_code.php", &formatted_code);
    let (formatted_code_program, error) = parse_source(&interner, &formatted_code_source);
    assert_eq!(error, None, "Error parsing formatted code");
    let reformatted_code =
        mago_formatter::format(&interner, &formatted_code_source, &formatted_code_program, php_version, settings);
    pretty_assertions::assert_eq!(expected, reformatted_code, "Reformatted code does not match expected");
}
