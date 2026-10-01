<?php

declare(strict_types=1);

namespace Issue2235;

use function array_multisort;

use const SORT_ASC;
use const SORT_DESC;
use const SORT_NATURAL;
use const SORT_NUMERIC;

function sortMultipleArrays(): void
{
    $timestamps = $ids = $usernames = [];

    array_multisort($timestamps, SORT_ASC, $ids, SORT_DESC, SORT_NUMERIC, $usernames, SORT_NATURAL);
    array_multisort($timestamps, $ids, SORT_ASC, $usernames);
    array_multisort($timestamps, getArray());
    array_multisort($timestamps, [3, 2, 1]);
    array_multisort([3, 2, 1]);
}

/** @return array<array-key, mixed> */
function getArray(): array
{
    return [];
}

function preserveFlagType(int $flag): int
{
    $values = [];
    array_multisort($values, $flag);

    return $flag;
}

/** @param list<string> $labels */
function preserveArrayType(array $labels): string
{
    $values = [];
    array_multisort($values, $labels);

    return $labels[0] ?? '';
}
