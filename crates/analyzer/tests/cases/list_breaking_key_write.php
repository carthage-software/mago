<?php

declare(strict_types=1);

/**
 * @param list<int> $list
 * @return list<int>
 */
function list_breaking_string_key(array $list, string $key): array
{
    $list[$key] = 1;

    // @mago-expect analysis:invalid-return-statement
    return $list;
}

/**
 * @param list<int> $list
 * @param negative-int $key
 * @return list<int>
 */
function list_breaking_negative_key(array $list, int $key): array
{
    $list[$key] = 1;

    // @mago-expect analysis:invalid-return-statement
    return $list;
}

/**
 * @param list<int> $list
 * @param non-negative-int $key
 * @return list<int>
 */
function list_keeping_index_key(array $list, int $key): array
{
    $list[$key] = 1;

    return $list;
}
