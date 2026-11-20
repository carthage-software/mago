<?php

declare(strict_types=1);

/**
 * @param array<int, string> $values
 */
function possibly_false_index_read(array $values, int|false $key): string
{
    /** @mago-expect analysis:possibly-false-array-index */
    return $values[$key];
}

/**
 * @param array<int, string> $values
 */
function possibly_false_index_unset(array $values): void
{
    $found = array_search('d', $values, true);
    /** @mago-expect analysis:possibly-false-array-index */
    unset($values[$found]);
}

/**
 * @param array<int, string> $values
 */
function checked_false_index(array $values): string
{
    $found = array_search('d', $values, true);
    if ($found === false) {
        return '';
    }

    return $values[$found];
}
