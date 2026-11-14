<?php

declare(strict_types=1);

/** @param non-empty-list<array{label: string}> $rows */
function label_lengths(array $rows): int
{
    foreach ($rows as &$row) {
        $row['length'] = strlen($row['label']);
    }

    unset($row);

    return $rows[0]['length'];
}

/** @param list<int> $ints */
function sum(array $ints): int
{
    return array_sum($ints);
}

/** @param list<int> $items */
function stringified(array $items): int
{
    foreach ($items as &$item) {
        $item = (string) $item;
    }

    unset($item);

    /** @mago-expect analysis:invalid-argument */
    return sum($items);
}

/**
 * @param list<int> $items
 * @return list<int>
 */
function skipped_items_keep_their_type(array $items): array
{
    foreach ($items as &$item) {
        if ($item > 5) {
            continue;
        }

        $item = $item * 2;
    }

    unset($item);

    return $items;
}
