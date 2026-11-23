<?php

declare(strict_types=1);

/**
 * @param array{a: int, b: int} $values
 * @param 'a'|'b' $key
 */
function reference_to_required_union_key(array $values, string $key): int
{
    $value = &$values[$key];

    return $value + 1;
}

/**
 * @param array{a: int, b: int} $values
 * @param 'a'|'b' $key
 */
function reference_to_required_union_key_in_condition(array $values, string $key): int
{
    if (($value = &$values[$key]) > 0) {
        return $value;
    }

    return 0;
}
