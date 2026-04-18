use mago_pattern::test_matches;
use mago_pattern::test_no_match;

test_matches! {
    name = surface_backtick_snippet_matches,
    pattern = "`eval(^x)`",
    code = "<?php eval($danger);",
    expect = 1,
}

test_no_match! {
    name = surface_backtick_snippet_no_false_positive,
    pattern = "`eval(^x)`",
    code = "<?php $eval = 1; echo $eval;",
}

test_matches! {
    name = surface_not_in_where_filters_literally,
    pattern = "`eval(^x)` where { not `eval('safe')` }",
    code = "<?php eval($user);",
    expect = 1,
}

test_matches! {
    name = surface_and_block_two_patterns,
    pattern = "and { `eval(^x)`, `eval(^x)` }",
    code = "<?php eval($a);",
    expect = 1,
}

test_matches! {
    name = surface_or_block_alternatives,
    pattern = "or { `eval(^x)`, `system(^x)` }",
    code = "<?php eval($a); system($b); echo 'c';",
    expect = 2,
}

test_matches! {
    name = surface_parentheses_group,
    pattern = "(`eval(^x)`)",
    code = "<?php eval($a);",
    expect = 1,
}

test_matches! {
    name = surface_where_with_one_constraint,
    pattern = "`eval(^x)` where { `eval(^x)` }",
    code = "<?php eval($a);",
    expect = 1,
}

test_matches! {
    name = surface_subtype_operator_both_match,
    pattern = "`eval(^x)` <: `eval(^x)`",
    code = "<?php eval($a);",
    expect = 1,
}

test_matches! {
    name = surface_nested_snippet_matches_eval_arg,
    pattern = "`eval(^x)`",
    code = "<?php eval('echo 1;');",
    expect = 1,
}

test_matches! {
    name = surface_contains_eval_anywhere,
    pattern = "contains `eval(^x)`",
    code = "<?php eval($a);",
    expect = 4,
}

test_matches! {
    name = surface_maybe_always_matches,
    pattern = "maybe `eval(^x)`",
    code = "<?php echo 1;",
    expect = 11,
}

test_matches! {
    name = surface_line_comment_ignored,
    pattern = "// this is a comment
                `eval(^x)`",
    code = "<?php eval($a);",
    expect = 1,
}
