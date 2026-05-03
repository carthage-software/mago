<?php

declare(strict_types=1);

namespace test;

function loose_zero_eliminates_null_int(?int $i, string $value): string
{
    /** @mago-expect analysis:possibly-null-operand */
    if ($i == 0) {
        return 'zero or null';
    }

    return $value . $i;
}

function loose_false_eliminates_null_bool(?bool $flag, string $value): string
{
    /** @mago-expect analysis:possibly-null-operand */
    /** @mago-expect analysis:false-operand */
    if ($flag == false) {
        return 'false or null';
    }

    /** @mago-expect analysis:redundant-condition */
    return $value . ($flag ? 'yes' : 'no');
}

function loose_empty_string_eliminates_null(?string $s): string
{
    /** @mago-expect analysis:possibly-null-operand */
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
    /** @mago-expect analysis:possibly-null-operand */
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

    /** @mago-expect analysis:possibly-null-operand */
    return 'value: ' . $i;
}
