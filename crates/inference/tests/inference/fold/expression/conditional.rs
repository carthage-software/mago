test_inference! {
    name = ternary_and_elvis_literals,
    cases = {
        "<?php 1 ? 'a' : 'b';" => "string('a')",
        "<?php 0 ? 'a' : 'b';" => "string('b')",
        "<?php $y ? 'a' : 'b';" => "string('a')|string('b')",
        "<?php 0 ?: 'x';" => "string('x')",
        "<?php 1 ?: 'x';" => "int(1)",
        "<?php '' ?: 'y';" => "string('y')",
    }
}

test_inference! {
    name = ternary_unions_both_branches,
    cases = {
        "<?php /** @var 1|2 */ $a = 1; $a > 0 ? 'yes' : 'no';" => "string('no')|string('yes')",
    }
}

test_inference! {
    name = ternary_narrows_branch_environments,
    cases = {
        "<?php /** @var 1|2 */ $a = 1; $a === 1 ? $a : $a;" => "int(1)|int(2)",
    }
}

test_inference! {
    name = ternary_arithmetic_distributes_over_narrowed_union,
    cases = {
        "<?php /** @var 1|2 */ $a = 1; ($a === 1 || $a === 2) ? ($a + 1) : 99;" => "int(2)|int(3)",
    }
}

test_inference! {
    name = ternary_with_always_true_condition_kills_else,
    cases = {
        "<?php /** @var 1|2 */ $a = 1; $c = ($a === 1 || ($b = 3)) ? ($a + 1) : ($a + $b); $c;" => "int(2)|int(3)",
    }
}

test_inference! {
    name = ternary_with_reachable_else_unions_all_outcomes,
    cases = {
        "<?php /** @var 1|2 */ $a = 1; /** @var bool */ $x = true; ($a === 1 || $x) ? ($a + 1) : 99;"
            => "int(2)|int(3)|int(99)",
    }
}

test_inference! {
    name = elvis_drops_falsy_from_condition,
    cases = {
        "<?php /** @var string|null */ $a = null; $a ?: 'default';" => "string&!string('')&!string('0')",
    }
}
