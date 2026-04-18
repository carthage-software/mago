use mago_pattern::test_matches;
use mago_pattern::test_matches_with_captures;
use mago_pattern::test_no_match;

test_matches! {
    name = array_key_value_with_metavar_value,
    pattern = "['f' => ^bar]",
    code = "<?php $x = ['f' => $value];",
    expect = 1,
}

test_matches! {
    name = array_literal_exact_key_exact_value,
    pattern = "['f' => 1]",
    code = "<?php $x = ['f' => 1]; $y = ['g' => 1];",
    expect = 1,
}

test_matches_with_captures! {
    name = array_key_value_captures_value,
    pattern = "['f' => ^bar]",
    code = "<?php $x = ['f' => $user];",
    expect = 1,
    captures = [("bar", "$user")],
}

test_matches! {
    name = array_key_value_many_entries_with_metavar_everywhere,
    pattern = "[^k => ^v]",
    code = "<?php $x = ['a' => 1]; $y = ['b' => 2]; $z = ['c' => 3];",
    expect = 3,
}

test_matches! {
    name = match_arm_arrow_preserved,
    pattern = "match (^x) { 1 => 'one' }",
    code = "<?php $r = match ($status) { 1 => 'one' };",
    expect = 1,
}

test_matches! {
    name = arrow_function_syntax,
    pattern = "fn() => ^body",
    code = "<?php $f = fn() => 42;",
    expect = 1,
}

test_matches! {
    name = arrow_function_with_param,
    pattern = "fn($^x) => ^body",
    code = "<?php $f = fn($a) => $a * 2;",
    expect = 1,
}

test_no_match! {
    name = array_wrong_key_literal,
    pattern = "['expected' => ^v]",
    code = "<?php $x = ['other' => 1];",
}

test_no_match! {
    name = match_wrong_arm_value,
    pattern = "match (^x) { 1 => 'one' }",
    code = "<?php $r = match ($s) { 1 => 'ONE' };",
}
