<?php

declare(strict_types=1);

/**
 * Iterating `(array) $fn` where `$fn` is `string|array<string>` is NOT guaranteed
 * to enter the loop: if `$fn` is an empty array the body never runs, so `$cols`
 * can still be empty afterwards. The `if (!$cols)` check must not be flagged as an
 * impossible condition.
 *
 * @param string|array<string> $fn
 */
function foreach_union_cast(array|string $fn): string
{
    $cols = [];
    foreach ((array) $fn as $f) {
        $cols[] = $f;
    }

    if (!$cols) {
        return '';
    }

    return implode(',', $cols);
}

/**
 * A generic `iterable` may be empty too, so the loop is not guaranteed to enter.
 *
 * @param iterable<int> $items
 */
function foreach_iterable_may_be_empty(iterable $items): string
{
    $seen = [];
    foreach ($items as $item) {
        $seen[] = $item;
    }

    if (!$seen) {
        return 'empty';
    }

    return (string) count($seen);
}
