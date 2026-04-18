use bumpalo::Bump;
use mago_pattern::CompileError;
use mago_pattern::compile;
use mago_pattern::test_matches_with_captures;
use mago_pattern::test_replaces;

mod named_dots_capture {
    use super::*;

    test_matches_with_captures! {
        name = named_dots_captures_remaining_args,
        pattern = "foo(^first, ^...rest)",
        code = "<?php foo($a, $b, $c);",
        expect = 1,
        captures = [
            ("first", "$a"),
            ("rest", "$b, $c"),
        ],
    }

    test_matches_with_captures! {
        name = named_dots_with_single_absorbed_arg,
        pattern = "foo(^first, ^...rest)",
        code = "<?php foo($a, $b);",
        expect = 1,
        captures = [
            ("first", "$a"),
            ("rest", "$b"),
        ],
    }

    test_matches_with_captures! {
        name = named_dots_captures_literals,
        pattern = "log(^level, ^...values)",
        code = "<?php log('info', 'hello', 42, $ctx);",
        expect = 1,
        captures = [
            ("level", "'info'"),
            ("values", "'hello', 42, $ctx"),
        ],
    }

    test_matches_with_captures! {
        name = named_dots_between_fixed_args,
        pattern = "f(^head, ^...middle, ^tail)",
        code = "<?php f(1, 2, 3, 4, 5);",
        expect = 1,
        captures = [
            ("head", "1"),
            ("middle", "2, 3, 4"),
            ("tail", "5"),
        ],
    }
}

mod rewrite_with_named_dots {
    use super::*;

    test_replaces! {
        name = rewrite_drops_first_arg,
        pattern = "`shell_exec(^cmd, ^...rest)` => `exec(^...rest)`",
        before = "<?php shell_exec('ls', '-la', $env);",
        after = "<?php exec('-la', $env);",
    }

    test_replaces! {
        name = rewrite_empty_sequence_renders_empty,
        pattern = "`shell_exec(^cmd, ^...rest)` => `exec(^...rest)`",
        before = "<?php shell_exec('whoami');",
        after = "<?php exec();",
    }

    test_replaces! {
        name = rewrite_single_absorbed_element,
        pattern = "`shell_exec(^cmd, ^...rest)` => `exec(^...rest)`",
        before = "<?php shell_exec('echo', $message);",
        after = "<?php exec($message);",
    }

    test_replaces! {
        name = rewrite_moves_sequence_to_different_position,
        pattern = "`wrap(^first, ^...middle, ^last)` => `unwrap(^last, ^...middle, ^first)`",
        before = "<?php wrap(1, 2, 3, 4);",
        after = "<?php unwrap(4, 2, 3, 1);",
    }

    test_replaces! {
        name = plain_reference_to_named_dots_splices_the_list,
        pattern = "`shell_exec(^cmd, ^...rest)` => `exec(^rest)`",
        before = "<?php shell_exec('echo', 'a', 'b');",
        after = "<?php exec('a', 'b');",
    }

    test_replaces! {
        name = rewrite_applies_to_every_match_in_file,
        pattern = "`shell_exec(^cmd, ^...rest)` => `exec(^...rest)`",
        before = "<?php shell_exec('a', $x); shell_exec('b'); shell_exec('c', $y, $z);",
        after = "<?php exec($x); exec(); exec($y, $z);",
    }
}

mod rhs_variable_validation {
    use super::*;

    fn compile_error(pattern: &str) -> String {
        let arena = Bump::new();
        match compile(&arena, pattern) {
            Ok(_) => panic!("expected pattern to fail compiling, but it succeeded: {pattern:?}"),
            Err(err) => err.to_string(),
        }
    }

    #[test]
    fn rhs_references_unbound_plain_variable() {
        let msg = compile_error("`foo(^x)` => `bar(^y)`");
        assert!(msg.contains("`^y`"), "diagnostic should name the offending variable, got: {msg}");
        assert!(msg.contains("not bound by the left-hand side"), "got: {msg}");
    }

    #[test]
    fn rhs_references_unbound_sequence_variable() {
        let msg = compile_error("`foo(^...rest)` => `bar(^...other)`");
        assert!(msg.contains("`^...other`"), "diagnostic should name the offending variable, got: {msg}");
        assert!(msg.contains("not bound by the left-hand side"), "got: {msg}");
    }

    #[test]
    fn rhs_bare_variable_token_unbound() {
        let msg = compile_error("`foo(^x)` => ^y");
        assert!(msg.contains("`^y`"), "got: {msg}");
        assert!(msg.contains("not bound by the left-hand side"), "got: {msg}");
    }

    #[test]
    fn pattern_without_rewrite_does_not_need_rhs_validation() {
        let arena = Bump::new();
        compile(&arena, "foo(^x, ^...rest)").expect("non-rewrite pattern always compiles");
    }

    #[test]
    fn rhs_can_reference_any_lhs_bound_variable() {
        let arena = Bump::new();
        compile(&arena, "`f(^a, ^b, ^...rest)` => `g(^b, ^a, ^...rest)`").expect("every RHS variable is bound on LHS");
    }

    #[test]
    fn rhs_error_preserves_compile_error_variant() {
        let arena = Bump::new();
        let Err(err) = compile(&arena, "`foo(^x)` => `bar(^nope)`") else {
            panic!("expected compile to fail");
        };
        assert!(matches!(err, CompileError::SurfaceError(_)), "expected SurfaceError, got {err:?}");
    }
}
