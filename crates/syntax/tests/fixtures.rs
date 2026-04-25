//! Fixture-based parser tests.

mod runner {
    use std::borrow::Cow;

    use bumpalo::Bump;

    use mago_database::file::File;
    use mago_syntax::parser::parse_file;

    pub fn parse_file_test(name: &'static str, code: &'static str, expected_errors: usize) {
        let arena = Bump::new();
        let file = File::ephemeral(Cow::Borrowed(name), Cow::Borrowed(code));
        let program = parse_file(&arena, &file);
        let actual = program.errors.len();
        if actual != expected_errors {
            panic!(
                "Test case '{name}' expected {expected_errors} parse error(s), got {actual}.\nErrors: {:#?}",
                program.errors,
            );
        }
    }
}

/// Generates a test that parses `tests/fixtures/<name>.php` and asserts the
/// resulting parse-error count matches `$expected_errors`.
///
/// - `test_parsing!(file_name, 0)` - file must parse with no errors.
/// - `test_parsing!(file_name, N)` - file must produce exactly N errors.
macro_rules! test_parsing {
    ($name:ident, $expected_errors:expr) => {
        #[test]
        fn $name() {
            let content = include_str!(concat!("fixtures/", stringify!($name), ".php"));
            crate::runner::parse_file_test(stringify!($name), content, $expected_errors);
        }
    };
}

test_parsing!(hello_world, 0);

// Fixtures lifted from nikic/PHP-Parser's test/code/parser/. Each one is
// invalid PHP at the syntax level (verified with `php -l`); the assertion is
// that mago keeps rejecting them. If a count drifts after a parser change,
// triage whether the new behavior is correct before bumping the number.
test_parsing!(error_handling_eof_error, 1);
test_parsing!(error_handling_lexer_errors, 1);
test_parsing!(error_handling_recovery, 3);
test_parsing!(expr_alternative_array_syntax, 22);
test_parsing!(expr_fetch_and_call_args, 5);
test_parsing!(expr_first_class_callables, 6);
test_parsing!(expr_new_without_class, 2);
test_parsing!(expr_uvs_global_non_simple_var_error, 3);
test_parsing!(scalar_encapsed_neg_var_offset, 6);
test_parsing!(scalar_float, 1);
test_parsing!(scalar_int, 14);
test_parsing!(scalar_invalid_octal, 1);
test_parsing!(scalar_number_separators, 8);
test_parsing!(stmt_class_short_echo_as_identifier, 8);
test_parsing!(stmt_halt_compiler_invalid_syntax, 1);
test_parsing!(stmt_halt_compiler_outermost_scope, 1);
test_parsing!(stmt_namespace_group_use_errors, 1);
