<?php

function test_impossible_case_value(): void
{
    $a = rand(0, 1) ? 'a' : 'b';

    switch ($a) {
        case 'a':
            break;
        case 'b':
            break;
        // @mago-expect analysis:never-matching-switch-case
        case 'c':
            echo 'impossible';
            break;
    }
}

function test_impossible_case_default(): void
{
    $a = rand(0, 1) ? 'a' : 'b';

    switch ($a) {
        case 'a':
            break;
        case 'b':
            break;
        // @mago-expect analysis:unreachable-switch-default
        default:
            echo 'impossible';
            break;
    }
}

function test_impossible_case_int_string_default(): void
{
    $a = rand(0, 1) ? 1 : 'two';

    switch ($a) {
        case 1:
            break;
        case 'two':
            break;
        // @mago-expect analysis:unreachable-switch-default
        default:
            echo 'impossible';
            break;
    }
}

/** @param 'a'|'b'|'c' $value */
function test_reachable_default(string $value): void
{
    switch ($value) {
        case 'a':
            echo $value;
            break;
        case 'b':
            echo $value;
            break;
        default:
            echo $value;
            // @mago-expect analysis:invalid-argument
            takes_switch_integer($value);
            break;
    }
    echo $value;
}

function takes_switch_integer(int $value): void
{
    echo $value;
}

/** @param 1|2 $value */
function test_coercible_case(int $value): void
{
    switch ($value) {
        case '1':
            echo $value;
            break;
        default:
            echo $value;
            break;
    }
}

/** @param 'a'|'b' $value */
function test_default_reachable_by_fallthrough(string $value): void
{
    switch ($value) {
        case 'a':
            break;
        case 'b':
            echo $value;
        default:
            echo $value;
            break;
    }
}

/** @param lowercase-string $value */
function test_lowercase_numeric_string(string $value): void
{
    switch ($value) {
        case '1E3':
            takes_switch_lowercase_string($value);
            break;
    }
}

/** @param uppercase-string $value */
function test_uppercase_numeric_string(string $value): void
{
    switch ($value) {
        case '1e3':
            takes_switch_uppercase_string($value);
            break;
    }
}

/** @param 0|9007199254740993 $value */
function test_large_integer_float_comparison(int $value): void
{
    switch ($value) {
        case 9007199254740992.0:
            takes_switch_integer($value);
            break;
    }
}

/** @param 0|9007199254740993 $value */
function test_large_integer_numeric_string_comparison(int $value): void
{
    switch ($value) {
        case '9007199254740992.0':
            takes_switch_large_integer($value);
            break;
    }
}

/** @param '9007199254740992'|'9007199254740994' $value */
function test_distinct_large_numeric_strings(string $value): void
{
    switch ($value) {
        // @mago-expect analysis:never-matching-switch-case
        case '9007199254740993':
            echo 'impossible';
            break;
    }
}

/** @param '1000'|'other' $value */
function test_numeric_string_notation(string $value): void
{
    switch ($value) {
        case '1E3':
            takes_switch_numeric_string($value);
            break;
    }
}

function test_float_numeric_string_notation(bool $flag): void
{
    $value = $flag ? 1.5 : 2.5;

    switch ($value) {
        case '15e-1':
            echo $value;
            break;
    }
}

function test_boolean_string_comparison(bool $value): void
{
    switch ($value) {
        case 'yes':
            echo $value;
            break;
        case '0':
            echo $value;
            break;
    }
}

/** @param 1|2 $value */
function test_distinct_integer_float_comparison(int $value): void
{
    switch ($value) {
        // @mago-expect analysis:never-matching-switch-case
        case 3.0:
            echo $value;
            break;
    }
}

/** @param non-empty-string $value */
function test_empty_string_against_non_empty_string(string $value): void
{
    switch ($value) {
        // @mago-expect analysis:never-matching-switch-case
        case '':
            echo $value;
            break;
    }
}

/** @param lowercase-string $value */
function takes_switch_lowercase_string(string $value): void
{
    echo $value;
}

/** @param uppercase-string $value */
function takes_switch_uppercase_string(string $value): void
{
    echo $value;
}

/** @param 9007199254740993 $value */
function takes_switch_large_integer(int $value): void
{
    echo $value;
}

/** @param '1000' $value */
function takes_switch_numeric_string(string $value): void
{
    echo $value;
}
