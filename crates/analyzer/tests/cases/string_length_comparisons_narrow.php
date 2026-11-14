<?php

declare(strict_types=1);

/** @return non-empty-string */
function identical_to_zero(string $s): string
{
    if (strlen($s) === 0) {
        return 'default';
    }

    return $s;
}

/** @return non-empty-string */
function loosely_not_zero(string $s): string
{
    return strlen($s) != 0 ? $s : 'default';
}

/** @return non-empty-string */
function zero_on_the_left(string $s): string
{
    return 0 !== strlen($s) ? $s : 'default';
}

/** @return non-empty-string */
function multibyte_not_empty(string $s): string
{
    return mb_strlen($s) > 0 ? $s : 'default';
}

/** @return non-empty-string */
function multibyte_identical_to_zero(string $s): string
{
    if (mb_strlen($s) === 0) {
        return 'default';
    }

    return $s;
}

/** @return non-empty-string */
function multibyte_long(string $s): string
{
    return mb_strlen($s) > 5 ? 'long' : $s . 'x';
}

function else_of_exact_length(string|false $packed): string
{
    if ($packed !== false && strlen($packed) === 4) {
        return 'v4';
    } elseif ($packed !== false && strlen($packed) === 16) {
        return 'v6';
    }

    return 'name';
}
