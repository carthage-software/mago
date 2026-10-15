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
