use crate::harness::*;

test_inference! {
    name = arithmetic_literals,
    cases = {
        "<?php 1 + 123;" => "int(124)",
        "<?php 10 - 4;" => "int(6)",
        "<?php 6 * 7;" => "int(42)",
        "<?php 6 / 2;" => "int(3)",
        "<?php 7 / 2;" => "float(3.5)",
        "<?php 7 % 3;" => "int(1)",
        "<?php 2 ** 10;" => "int(1024)",
        "<?php 1.5 + 2;" => "float(3.5)",
        "<?php 1 / 0;" => "never",
    }
}

test_inference! {
    name = bitwise_and_concat_literals,
    cases = {
        "<?php 6 & 3;" => "int(2)",
        "<?php 5 | 2;" => "int(7)",
        "<?php 5 ^ 1;" => "int(4)",
        "<?php 1 << 4;" => "int(16)",
        "<?php 'a' . 'b';" => "string('ab')",
        "<?php 'x' . 1;" => "string('x1')",
    }
}

test_inference! {
    name = comparison_and_logical_literals,
    cases = {
        "<?php 1 < 2;" => "true",
        "<?php 2 <= 1;" => "false",
        "<?php 1 === 1;" => "true",
        "<?php 1 === 1.0;" => "false",
        "<?php 1 == 1.0;" => "true",
        "<?php true && false;" => "false",
        "<?php true || false;" => "true",
        "<?php 1 <=> 2;" => "int(-1)",
    }
}

test_inference! {
    name = php_coercion_truth_table,
    cases = {
        "<?php [] . ' 123' . 0x01;" => "string('Array 1231')",
        "<?php true . 'x';" => "string('1x')",
        "<?php false . 'x';" => "string('x')",
        "<?php null . 'x';" => "string('x')",
        "<?php '123' + 1;" => "int(124)",
        "<?php ' 123' + 1;" => "int(124)",
        "<?php '1.5' + 1;" => "float(2.5)",
        "<?php '123abc' + 1;" => "int(124)",
        "<?php '0x1A' + 1;" => "int(1)",
        "<?php '1e3' + 1;" => "float(1001)",
        "<?php true + 1;" => "int(2)",
        "<?php null + 1;" => "int(1)",
        "<?php '5' & '3';" => "string('1')",
        "<?php '5' | '2';" => "string('7')",
        "<?php '10' & 6;" => "int(2)",
        "<?php '123' <=> 124;" => "int(-1)",
        "<?php '1' == 1;" => "true",
        "<?php '01' == '1';" => "true",
    }
}

test_inference! {
    name = array_addition,
    cases = {
        "<?php [1] + [2];" => "list{0: int(1)}",
        "<?php [] + [1];" => "list{0: int(1)}",
        "<?php [1, 2] + [3, 4, 5];" => "list{0: int(1), 1: int(2), 2: int(5)}",
        "<?php ['a' => 1] + ['b' => 2];" => "array{'a': int(1), 'b': int(2)}",
        "<?php ['a' => 1, 'b' => 2] + ['b' => 3, 'c' => 4];" => "array{'a': int(1), 'b': int(2), 'c': int(4)}",
    }
}

test_inference! {
    name = never_propagation,
    cases = {
        "<?php 'abc' + 1;" => "never",
        "<?php [1] - 2;" => "never",
        "<?php [1] + 1;" => "never",
        "<?php [1] * 2;" => "never",
        "<?php ('abc' + 1) + 2;" => "never",
        "<?php (int) ('abc' + 1);" => "never",
        "<?php ['abc' + 1];" => "never",
        "<?php match ('abc' + 1) { 1 => 'x' };" => "never",
    }
}

test_inference! {
    name = assignment_in_short_circuit_is_conditional,
    cases = {
        "<?php $b = 'x'; $cond || ($b = 5); $b;" => "int(5)|string('x')",
    }
}

test_inference! {
    name = tdd_folds_multi_variable_contradiction,
    cases = {
        "<?php ($a || $b) && (!$a && !$b);" => "false",
    }
}

test_inference! {
    name = tdd_folds_single_variable_tautology,
    cases = {
        "<?php $a || !$a;" => "true",
    }
}

test_inference! {
    name = or_narrows_right_operand_when_left_is_false,
    code = "<?php /** @var string|null */ $a = null; $a === null || $a === null;",
    expect = |ir| {
        assert_eq!(get_last_binary(ir).right.meta.to_string(), "false");
    }
}

test_inference! {
    name = and_narrows_right_operand_when_left_is_true,
    code = "<?php /** @var string|null */ $a = null; $a === null && $a === null;",
    expect = |ir| {
        assert_eq!(get_last_binary(ir).right.meta.to_string(), "true");
    }
}

test_inference! {
    name = disjoint_identical_is_false_without_narrowing,
    code = "<?php $x = 'a'; true && $x === null;",
    expect = |ir| {
        assert_eq!(get_last_binary(ir).right.meta.to_string(), "false");
    }
}

test_inference! {
    name = pipe_operator_applies_the_right_operand,
    def = "<?php function shout(string $s): string { return $s; }",
    cases = {
        "<?php $f = fn(int $x): string => ''; $r = 5 |> $f; $r;" => "string",
        "<?php $r = 'x' |> 'shout'; $r;" => "string",
    }
}
