<?php

declare(strict_types=1);

namespace test;

function loose_zero_eliminates_null_int(?int $i, string $value): string
{
    if ($i == 0) {
        return 'zero or null';
    }

    return $value . $i;
}

function loose_false_eliminates_null_bool(?bool $flag, string $value): string
{
    if ($flag == false) {
        return 'false or null';
    }

    return $value . ($flag ? 'yes' : 'no');
}

function loose_empty_string_eliminates_null(?string $s): string
{
    if ($s == '') {
        return 'empty or null';
    }

    return $s;
}

function loose_empty_string_eliminates_false(string|bool $x): string
{
    if ($x == '') {
        return 'falsy';
    }

    return $x === true ? '1' : ($x === false ? '0' : $x);
}

function loose_zero_eliminates_false_and_null(int|false|null $value): int
{
    if ($value == 0) {
        return 0;
    }

    return $value;
}

function strict_zero_keeps_null(?int $i): string
{
    if ($i === 0) {
        return 'exactly zero';
    }

    return 'value: ' . $i;
}
