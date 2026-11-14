<?php

declare(strict_types=1);

/**
 * @param array<string, int> $map
 *
 * @return list<int>
 */
function push_unshift_int_keys(array $map): array
{
    array_unshift($map, 7);

    $values = [];
    foreach ($map as $k => $v) {
        if (is_int($k)) {
            $values[] = $v;
        }
    }

    return $values;
}

/**
 * @param array<string, int> $map
 *
 * @return array<string, int>
 */
function push_unshift_wrong(array $map): array
{
    array_push($map, 7);

    // @mago-expect analysis:invalid-return-statement
    return $map;
}

function implode_needs_separator(): void
{
    // @mago-expect analysis:too-few-arguments
    echo implode();
}
