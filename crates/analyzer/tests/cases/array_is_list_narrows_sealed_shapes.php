<?php

declare(strict_types=1);

/**
 * @param array{a?: string} $a
 * @return array{}
 */
function only_string_keys(array $a): array
{
    if (array_is_list($a)) {
        return $a;
    }

    return [];
}

/**
 * @param array{0?: string, a?: int} $b
 * @return list{0?: string}
 */
function optional_int_key(array $b): array
{
    if (array_is_list($b)) {
        return $b;
    }

    return [];
}

/**
 * @param array{0?: string, 1: int} $c
 * @return list{string, int}
 */
function set_key_after_optional(array $c): array
{
    if (array_is_list($c)) {
        return $c;
    }

    return ['a', 1];
}

/**
 * @param array{0?: string, 2?: int} $d
 * @return list{0?: string}
 */
function gap_drops_later_keys(array $d): array
{
    if (array_is_list($d)) {
        return $d;
    }

    return [];
}

/**
 * @param array{1: int, a: string} $e
 */
function never_a_list(array $e): void
{
    /** @mago-expect analysis:impossible-type-comparison,impossible-condition */
    if (array_is_list($e)) {
        echo 'never';
    }
}
