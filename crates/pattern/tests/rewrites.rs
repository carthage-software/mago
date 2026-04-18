use mago_pattern::test_replaces;

test_replaces! {
    name = eval_to_safe_eval,
    pattern = "`eval(^x)` => `safe_eval(^x)`",
    before = "<?php eval($code);",
    after = "<?php safe_eval($code);",
}

test_replaces! {
    name = eval_preserves_literal_argument,
    pattern = "`eval(^x)` => `safe_eval(^x)`",
    before = "<?php eval('echo 1;');",
    after = "<?php safe_eval('echo 1;');",
}

test_replaces! {
    name = swap_loose_for_strict_equality,
    pattern = "`^a == ^b` => `^a === ^b`",
    before = "<?php if ($x == 1) { echo 1; }",
    after = "<?php if ($x === 1) { echo 1; }",
}

test_replaces! {
    name = strlen_greater_than_zero_to_non_empty,
    pattern = "`strlen(^s) > 0` => `^s !== ''`",
    before = "<?php if (strlen($name) > 0) {}",
    after = "<?php if ($name !== '') {}",
}

test_replaces! {
    name = count_greater_than_zero_to_non_empty_array,
    pattern = "`count(^x) > 0` => `[] !== ^x`",
    before = "<?php if (count($items) > 0) {}",
    after = "<?php if ([] !== $items) {}",
}

test_replaces! {
    name = rewrite_method_call_receiver,
    pattern = "`^obj->log(^msg)` => `Logger::log(^obj, ^msg)`",
    before = "<?php $user->log('hello');",
    after = "<?php Logger::log($user, 'hello');",
}

test_replaces! {
    name = rewrite_all_occurrences,
    pattern = "`is_null(^x)` => `^x === null`",
    before = "<?php if (is_null($a)) {} if (is_null($b)) {} if (is_null($c)) {}",
    after = "<?php if ($a === null) {} if ($b === null) {} if ($c === null) {}",
}

test_replaces! {
    name = rewrite_to_string_literal,
    pattern = "`dd(^x)` => \"// debug removed\"",
    before = "<?php dd($state);",
    after = "<?php // debug removed;",
}
