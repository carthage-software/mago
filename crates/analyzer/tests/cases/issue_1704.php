<?php

declare(strict_types=1);

/**
 * @param array<int, array{id: int}> $items
 * @return array<int, array{count: int, ...<string, mixed>}>
 */
function aggregate(array $items): array
{
    /** @var array<int, array{count: int, ...<string, mixed>}> $res */
    $res = [];

    foreach ($items as $item) {
        $id = $item['id'];

        if (!array_key_exists($id, $res)) {
            $res[$id] = [
                ...$item,
                'count' => 0,
            ];
        }

        $res[$id]['count']++;
    }

    return $res;
}
