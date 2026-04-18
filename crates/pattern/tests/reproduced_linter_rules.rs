use mago_pattern::test_matches;
use mago_pattern::test_matches_with_captures;
use mago_pattern::test_no_match;

mod safety {
    use super::*;

    test_matches! {
        name = no_eval_detects_direct_call,
        pattern = "eval(^code)",
        code = "<?php eval('echo 1;');",
        expect = 1,
    }

    test_matches! {
        name = no_eval_detects_with_variable,
        pattern = "eval(^code)",
        code = "<?php eval($user_input);",
        expect = 1,
    }

    test_no_match! {
        name = no_eval_ignores_unrelated_calls,
        pattern = "eval(^code)",
        code = "<?php json_decode($x); $eval = 1; echo $eval;",
    }

    test_matches! {
        name = no_eval_finds_both_when_nested,
        pattern = "eval(^code)",
        code = "<?php eval($a); eval($b);",
        expect = 2,
    }

    test_matches_with_captures! {
        name = no_eval_captures_exact_argument,
        pattern = "eval(^code)",
        code = "<?php eval($userInput);",
        expect = 1,
        captures = [("code", "$userInput")],
    }
}

mod php_deprecations {
    use super::*;

    test_matches! {
        name = php_extract_banned,
        pattern = "extract(^arr)",
        code = "<?php extract($data);",
        expect = 1,
    }

    test_matches! {
        name = php_compact_call,
        pattern = "compact(^a, ^b)",
        code = "<?php $r = compact('user', 'role');",
        expect = 1,
    }

    test_matches! {
        name = php_func_get_args,
        pattern = "func_get_args()",
        code = "<?php function f() { $a = func_get_args(); }",
        expect = 1,
    }

    test_matches! {
        name = php_prefer_random_int_over_mt_rand,
        pattern = "mt_rand(^a, ^b)",
        code = "<?php $x = mt_rand(1, 100);",
        expect = 1,
    }
}

mod psl_preferences {
    use super::*;

    test_matches! {
        name = psl_array_map_detected,
        pattern = "array_map(^fn, ^arr)",
        code = "<?php $r = array_map('strtoupper', $words);",
        expect = 1,
    }

    test_matches! {
        name = psl_array_filter_detected,
        pattern = "array_filter(^arr, ^fn)",
        code = "<?php $r = array_filter($items, fn($i) => $i > 0);",
        expect = 1,
    }

    test_matches! {
        name = psl_strlen_detected,
        pattern = "strlen(^s)",
        code = "<?php $n = strlen($input);",
        expect = 1,
    }

    test_matches! {
        name = psl_strpos_detected,
        pattern = "strpos(^hay, ^needle)",
        code = "<?php $i = strpos('hello world', 'world');",
        expect = 1,
    }

    test_matches! {
        name = psl_intdiv_detected,
        pattern = "intdiv(^a, ^b)",
        code = "<?php $q = intdiv(10, 3);",
        expect = 1,
    }

    test_matches! {
        name = psl_random_int_detected,
        pattern = "random_int(^a, ^b)",
        code = "<?php $n = random_int(0, 100);",
        expect = 1,
    }

    test_matches! {
        name = psl_preg_match_detected,
        pattern = "preg_match(^pattern, ^subject)",
        code = "<?php preg_match('/foo/', $s);",
        expect = 1,
    }

    test_matches! {
        name = psl_preg_split_detected,
        pattern = "preg_split(^pattern, ^subject)",
        code = "<?php $parts = preg_split('/,/', $csv);",
        expect = 1,
    }

    test_matches! {
        name = psl_sleep_detected,
        pattern = "sleep(^seconds)",
        code = "<?php sleep(5);",
        expect = 1,
    }

    test_matches! {
        name = psl_printf_detected,
        pattern = "printf(^fmt, ^arg)",
        code = "<?php printf(\"%s\\n\", $name);",
        expect = 1,
    }
}

mod best_practices_extra {
    use super::*;

    test_matches! {
        name = prefer_explode_trivial_pattern,
        pattern = "preg_split(^pat, ^sub)",
        code = "<?php preg_split('/ /', $line);",
        expect = 1,
    }
}

mod clarity_extra {
    use super::*;

    test_matches! {
        name = strict_assertion_loose_equality,
        pattern = "assert(^a == ^b)",
        code = "<?php assert($x == 1);",
        expect = 1,
    }

    test_no_match! {
        name = strict_assertion_strict_equality_skipped,
        pattern = "assert(^a == ^b)",
        code = "<?php assert($x === 1);",
    }

    test_matches! {
        name = identity_loose_equality_detected,
        pattern = "^a == ^b",
        code = "<?php if ($x == $y) { echo 1; }",
        expect = 1,
    }

    test_matches! {
        name = identity_not_equal_detected,
        pattern = "^a != ^b",
        code = "<?php if ($x != null) { echo 1; }",
        expect = 1,
    }

    test_no_match! {
        name = identity_strict_equality_not_loose,
        pattern = "^a == ^b",
        code = "<?php if ($x === $y) { echo 1; }",
    }
}

mod security {
    use super::*;

    test_matches! {
        name = no_superglobal_get_direct_read,
        pattern = "$_GET",
        code = "<?php echo $_GET['id'];",
        expect = 1,
    }
}

mod best_practices {}

mod clarity {
    use super::*;

    test_matches! {
        name = assert_without_description_matches,
        pattern = "assert(^cond)",
        code = "<?php assert($x > 0);",
        expect = 1,
    }

    test_no_match! {
        name = assert_with_description_skipped,
        pattern = "assert(^cond)",
        code = "<?php assert($x > 0, 'x must be positive');",
    }

    test_matches! {
        name = assert_with_description_positive,
        pattern = "assert(^cond, ^msg)",
        code = "<?php assert($x > 0, 'positive');",
        expect = 1,
    }
}

mod deprecation {
    use super::*;

    test_matches! {
        name = deprecated_create_function,
        pattern = "create_function(^args, ^body)",
        code = "<?php $fn = create_function('$x', 'return $x*2;');",
        expect = 1,
    }

    test_matches! {
        name = deprecated_each,
        pattern = "each(^arr)",
        code = "<?php while ($pair = each($items)) {}",
        expect = 1,
    }
}

mod consistency {}

mod redundancy {
    use super::*;

    test_matches! {
        name = parenthesized_variable_is_redundant,
        pattern = "(^x)",
        code = "<?php $y = ($a);",
        expect = 1,
    }
}

mod correctness {}

mod simplifications {
    use super::*;

    test_matches! {
        name = count_greater_than_zero,
        pattern = "count(^x) > 0",
        code = "<?php if (count($items) > 0) { echo 1; }",
        expect = 1,
    }

    test_matches! {
        name = count_strict_equals_zero,
        pattern = "count(^x) === 0",
        code = "<?php if (count($items) === 0) { echo 'empty'; }",
        expect = 1,
    }

    test_matches! {
        name = is_null_call,
        pattern = "is_null(^x)",
        code = "<?php if (is_null($v)) { echo 'yes'; }",
        expect = 1,
    }

    test_matches! {
        name = strlen_greater_than_zero,
        pattern = "strlen(^x) > 0",
        code = "<?php if (strlen($s) > 0) {}",
        expect = 1,
    }

    test_matches! {
        name = array_key_exists,
        pattern = "array_key_exists(^k, ^arr)",
        code = "<?php array_key_exists('foo', $map);",
        expect = 1,
    }
}

mod instantiation {
    use super::*;

    test_matches! {
        name = new_datetime_detected,
        pattern = "new DateTime(^fmt)",
        code = "<?php $d = new DateTime('now');",
        expect = 1,
    }

    test_matches_with_captures! {
        name = new_datetime_captures_format,
        pattern = "new DateTime(^fmt)",
        code = "<?php $d = new DateTime('2024-01-01');",
        expect = 1,
        captures = [("fmt", "'2024-01-01'")],
    }

    test_matches! {
        name = new_exception_detected,
        pattern = "new Exception(^msg)",
        code = "<?php throw new Exception('boom');",
        expect = 1,
    }
}

mod access {
    use super::*;

    test_matches! {
        name = method_call_with_metavar_receiver,
        pattern = "^obj->foo()",
        code = "<?php $x->foo(); $y->foo(); $z->bar();",
        expect = 2,
    }

    test_matches_with_captures! {
        name = method_call_captures_receiver,
        pattern = "^obj->log()",
        code = "<?php $logger->log();",
        expect = 1,
        captures = [("obj", "$logger")],
    }

    test_matches! {
        name = null_safe_method_call_with_metavar,
        pattern = "^obj?->value()",
        code = "<?php $maybe?->value();",
        expect = 1,
    }

    test_matches! {
        name = method_call_with_metavar_method_and_receiver,
        pattern = "^obj->^method()",
        code = "<?php $x->foo(); $y->bar();",
        expect = 2,
    }

    test_matches! {
        name = array_subscript_with_metavar_base,
        pattern = "^arr['key']",
        code = "<?php echo $data['key']; echo $other['key'];",
        expect = 2,
    }

    test_matches! {
        name = static_method_call_with_metavar_arg,
        pattern = "Log::error(^msg)",
        code = "<?php Log::error('oops');",
        expect = 1,
    }

    test_matches_with_captures! {
        name = log_error_captures,
        pattern = "Log::error(^msg)",
        code = "<?php Log::error($diagnostic);",
        expect = 1,
        captures = [("msg", "$diagnostic")],
    }

    test_matches! {
        name = class_constant_access,
        pattern = "Status::ACTIVE",
        code = "<?php if ($s === Status::ACTIVE) {}",
        expect = 1,
    }
}

mod framework_specific {
    use super::*;

    test_matches! {
        name = wp_create_nonce,
        pattern = "wp_create_nonce(^name)",
        code = "<?php $n = wp_create_nonce('action');",
        expect = 1,
    }

    test_matches! {
        name = laravel_db_select,
        pattern = "DB::select(^sql)",
        code = "<?php $rows = DB::select('select * from users');",
        expect = 1,
    }

    test_matches! {
        name = laravel_dd,
        pattern = "dd(^x)",
        code = "<?php dd($user);",
        expect = 1,
    }

    test_matches! {
        name = var_dump_leftover,
        pattern = "var_dump(^x)",
        code = "<?php var_dump($state);",
        expect = 1,
    }

    test_matches! {
        name = print_r_leftover,
        pattern = "print_r(^x)",
        code = "<?php print_r($state);",
        expect = 1,
    }

    test_matches! {
        name = print_r_two_args,
        pattern = "print_r(^x, ^as_string)",
        code = "<?php echo print_r($state, true);",
        expect = 1,
    }
}

mod identities {
    use super::*;

    test_matches! {
        name = strict_equal_to_null,
        pattern = "^x === null",
        code = "<?php if ($v === null) echo 1;",
        expect = 1,
    }

    test_matches! {
        name = strict_not_equal_to_false,
        pattern = "^x !== false",
        code = "<?php if ($result !== false) echo 1;",
        expect = 1,
    }

    test_matches! {
        name = null_coalesce_with_literal,
        pattern = "^x ?? ^default",
        code = "<?php echo $name ?? 'guest';",
        expect = 1,
    }
}

mod arithmetic {
    use super::*;

    test_matches! {
        name = self_ge_comparison,
        pattern = "^x >= ^x",
        code = "<?php if ($a >= $a) echo 1;",
        expect = 1,
    }

    test_matches! {
        name = subtract_zero_is_redundant,
        pattern = "^x - 0",
        code = "<?php $y = $a - 0;",
        expect = 1,
    }

    test_matches! {
        name = multiply_by_one_is_redundant,
        pattern = "^x * 1",
        code = "<?php $y = $a * 1;",
        expect = 1,
    }

    test_matches! {
        name = or_with_literal_true,
        pattern = "^x || true",
        code = "<?php if ($cond || true) echo 1;",
        expect = 1,
    }
}

#[test]
fn engine_capability_status() {}
