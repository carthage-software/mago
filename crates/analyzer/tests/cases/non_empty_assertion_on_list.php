<?php

declare(strict_types=1);

/**
 * @psalm-assert non-empty-list $things
 *
 * @throws InvalidArgumentException
 */
function assertNonEmptyList(array $things): void
{
    if ($things === [] || !array_is_list($things)) {
        throw new InvalidArgumentException('empty');
    }
}

/**
 * @psalm-assert non-empty-array $things
 *
 * @throws InvalidArgumentException
 */
function assertNonEmptyArray(array $things): void
{
    if ($things === []) {
        throw new InvalidArgumentException('empty');
    }
}

/**
 * @param list<string> $list
 *
 * @throws InvalidArgumentException
 */
function firstKeyOfList(array $list): int
{
    assertNonEmptyList($list);

    return array_key_first($list);
}

/**
 * @param list<string> $list
 *
 * @throws InvalidArgumentException
 */
function firstKeyOfArray(array $list): int
{
    assertNonEmptyArray($list);

    return array_key_first($list);
}

/**
 * @param array<int, string> $map
 *
 * @throws InvalidArgumentException
 */
function firstKeyOfMap(array $map): int
{
    assertNonEmptyArray($map);

    return array_key_first($map);
}
