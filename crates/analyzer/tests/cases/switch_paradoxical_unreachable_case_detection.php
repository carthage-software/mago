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
