#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::borrow::Cow;

use mago_allocator::LocalArena;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::scanner::scan_program;
use mago_database::file::File;
use mago_names::resolver::NameResolver;
use mago_php_version::PHPVersion;
use mago_syntax::parser::parse_file;

fn scan(source: &str) -> CodebaseMetadata {
    let arena = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"callback-catches.php"), Cow::Owned(source.as_bytes().to_vec()));
    let program = parse_file(&arena, &file);
    assert!(!program.has_errors(), "parse failed: {:?}", program.errors);
    let resolved_names = NameResolver::new(&arena).resolve(program);

    scan_program(&arena, &file, program, &resolved_names, PHPVersion::LATEST)
}

fn callback_parameter(
    function: &FunctionLikeMetadata,
) -> &mago_codex::metadata::parameter::FunctionLikeParameterMetadata {
    function.parameters.first().expect("expected a callback parameter")
}

#[test]
fn integer_switch_scans_the_matching_callback_case() {
    let codebase = scan(
        "<?php
        /**
         * @param callable $callback
         * @param-immediately-invoked-callable $callback
         */
        function invoke_callback(callable $callback): void {
            switch (2) {
                case 3:
                    break;
                case 2:
                    try {
                        $callback();
                    } catch (RuntimeException) {
                    }
                    break;
            }
        }
        ",
    );
    let function = codebase.get_function(b"invoke_callback").expect("expected function metadata");
    let catch_summaries = callback_parameter(function)
        .callable_exception_catches
        .as_ref()
        .expect("expected a catch summary for the matching switch case");

    assert_eq!(catch_summaries.len(), 1);
    assert_eq!(catch_summaries[0].catch_groups.len(), 1);
}

#[test]
fn elseif_after_guaranteed_true_does_not_add_callback_catch_summary() {
    let codebase = scan(
        "<?php
        /**
         * @param callable $callback
         * @param-immediately-invoked-callable $callback
         */
        function invoke_callback(callable $callback, bool $condition): void {
            if (false) {
            } elseif ($condition) {
            } elseif (true) {
            } elseif (false) {
                try {
                    $callback();
                } catch (RuntimeException) {
                }
            }
        }
        ",
    );
    let function = codebase.get_function(b"invoke_callback").expect("expected function metadata");

    assert!(callback_parameter(function).callable_exception_catches.is_none());
}
