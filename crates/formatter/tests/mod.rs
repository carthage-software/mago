use mago_formatter::settings::FormatSettings;
use mago_interner::ThreadedInterner;
use mago_parser::parse_source;
use mago_source::SourceCategory;
use mago_source::SourceManager;

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
    let interner = ThreadedInterner::new();
    let manager = SourceManager::new(interner.clone());

    let code_id = manager.insert_content("code.php", code.as_ref(), SourceCategory::default());
    let code_source = manager.load(&code_id).expect("Failed to load code source");
    let (code_program, error) = parse_source(&interner, &code_source);
    assert_eq!(error, None, "Error parsing code");
    let formatted_code = mago_formatter::format(&interner, &code_source, &code_program, settings);
    pretty_assertions::assert_eq!(expected, formatted_code, "Formatted code does not match expected");

    let formatted_code_id = manager.insert_content("formatted_code.php", formatted_code, SourceCategory::default());
    let formatted_code_source = manager.load(&formatted_code_id).expect("Failed to load formatted code source");
    let (formatted_code_program, error) = parse_source(&interner, &formatted_code_source);
    assert_eq!(error, None, "Error parsing formatted code");
    let reformatted_code = mago_formatter::format(&interner, &formatted_code_source, &formatted_code_program, settings);
    pretty_assertions::assert_eq!(expected, reformatted_code, "Reformatted code does not match expected");
}
