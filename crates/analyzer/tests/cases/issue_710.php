<?php

declare(strict_types=1);

/**
 * @param non-empty-list<string>|null $value
 */
function checkCount(null|array $value): string
{
    if ($value === null || count($value) > 42) {
        return 'null or more than 42 items';
    }

    return 'valid: 1-42 items';
}

/**
 * @param non-empty-list<int> $items
 */
function checkMinimum(array $items): string
{
    if (count($items) >= 10) {
        return 'at least 10 items';
    }

    return 'less than 10 items';
}

/**
 * @param non-empty-list<string> $values
 */
function processLargeList(array $values): int
{
    if (count($values) > 100) {
        return count($values);
    }

    return 0;
}
