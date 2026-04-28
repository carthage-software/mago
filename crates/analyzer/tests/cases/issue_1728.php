<?php

declare(strict_types=1);

/**
 * @param array<int, array{count: int, ...<string, mixed>}> $res
 */
function issue_1728_bump(array $res, int $id): int
{
    if (!array_key_exists($id, $res)) {
        $res[$id] = ['count' => 0, 'foo' => 'bar'];
    }

    $res[$id]['count']++;

    return $res[$id]['count'];
}

issue_1728_bump([
    [
        'count' => 1,
        'someField' => 'sdf',
    ],
], id: 1);
