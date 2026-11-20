<?php

declare(strict_types=1);

namespace App\ShadowedBuiltins;

function is_a(object $_object, string $_class): bool
{
    return true;
}

function in_array(mixed $_needle, array $_haystack, bool $_strict = false): bool
{
    return true;
}

function shadowed_is_a_keeps_type(object $value): object
{
    if (is_a($value, \Exception::class)) {
        return $value;
    }

    return $value;
}

function shadowed_in_array_keeps_type(int|string $value): int|string
{
    if (in_array($value, ['a', 'b'], true)) {
        return $value === 'a' ? 1 : $value;
    }

    return $value;
}
