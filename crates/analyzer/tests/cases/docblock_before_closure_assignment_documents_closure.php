<?php

declare(strict_types=1);

function by_reference(): int
{
    /** @param-out array<array-key, int> $arr */
    $fill = static function (array &$arr): void {
        if (!$arr) {
            $arr = ['total' => 0, 'sent' => 0];
        }
    };

    $stats = ['key' => []];
    $fill($stats['key']);

    return $stats['key']['total'] ?? 0;
}

function plain_param(): int
{
    /** @param list<int> $items */
    $sum = static function (array $items): int {
        $total = 0;
        foreach ($items as $item) {
            $total += $item;
        }

        return $total;
    };

    return $sum([1, 2]);
}
