<?php

declare(strict_types=1);

/** @return array<string, int> */
function fetch(?string $key): array
{
    return ['entry' => $key === null ? 0 : 1];
}

function reads(): int
{
    $keys = null;

    /** @mago-expect analysis:null-array-access */
    $total = isset(fetch($keys['key'])['entry']) ? 1 : 0;

    /** @mago-expect analysis:null-array-access */
    return $total + (fetch($keys['key'])['entry'] ?? 0);
}
