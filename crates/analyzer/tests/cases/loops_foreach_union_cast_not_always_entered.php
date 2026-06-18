<?php

declare(strict_types=1);

/**
 * `(array) $fn` with `$fn: string|array<string>` is not guaranteed to enter the
 * loop (an empty array skips the body), so `if (!$cols)` is reachable.
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
 * A generic `iterable` may be empty, so the loop is not guaranteed to enter.
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

/**
 * Opposite direction: a *pure* string cast always yields a one-element array, so
 * the loop always runs — `impossible-condition` must still fire (no over-suppression).
 */
function foreach_pure_string_cast_always_enters(string $fn): string
{
    $cols = [];
    foreach ((array) $fn as $f) {
        $cols[] = $f;
    }

    /** @mago-expect analysis:impossible-condition */
    if (!$cols) {
        return '';
    }

    return implode(',', $cols);
}
