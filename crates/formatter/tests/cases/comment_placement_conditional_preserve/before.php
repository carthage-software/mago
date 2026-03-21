<?php

// Before condition in parenthesized ternary
$a = (/* before condition */ $condition ? $then : $else);

// Line comment trailing condition
$b = $condition // trailing condition
    ? $then
    : $else;

// Block comment between `?` and then
$c = $condition
    ? /* block comment */
    value_function('arg')
    : $else;

// Line comment trailing then
$d = $condition
    ? $then // trailing then
    : $else;

// Own-line line comment between `:` and else
$e = $condition
    ? $then
    :
    // leading else
    $else;

// Line comment trailing else
$f = is_type($value)
    ? $value
    : ($flag_a ? CONST_A : 0) | ($flag_b ? CONST_B : 0); // trailing

// Elvis with block comment between condition and `?`
$g = $condition /* before ?: */ ?: $fallback;

// Elvis with block comment between `?` and `:`
$h = $condition ?/* between ?: */: $fallback;

// Elvis with block comment after `:`
$i = $condition ?:
    /* block before fallback */
    $fallback;

// Elvis with trailing line comment
$j = $condition ?: $fallback; // elvis trailing

// Multiple comments in same zone
$k = $condition
    ? $then // first
    // second
    : $else;
