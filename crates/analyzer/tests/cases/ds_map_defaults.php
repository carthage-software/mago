<?php

declare(strict_types=1);

/**
 * @param Ds\Map<int, string> $map
 * @throws OutOfBoundsException
 */
function getWithoutDefault(Ds\Map $map): string
{
    return $map->get(1);
}

/**
 * @param Ds\Map<int, string> $map
 * @throws OutOfBoundsException
 */
function getWithNullDefault(Ds\Map $map): ?string
{
    return $map->get(1, null);
}

/**
 * @param Ds\Map<int, string> $map
 * @throws OutOfBoundsException
 */
function getWithIntegerDefault(Ds\Map $map): string|int
{
    return $map->get(1, 0);
}

/**
 * @param Ds\Map<int, string> $map
 * @throws OutOfBoundsException
 */
function getWithNullableDefault(Ds\Map $map, ?int $default): string|int|null
{
    return $map->get(key: 1, default: $default);
}

/**
 * @param Ds\Map<int, string> $map
 * @throws OutOfBoundsException
 */
function removeWithoutDefault(Ds\Map $map): string
{
    return $map->remove(1);
}

/**
 * @param Ds\Map<int, string> $map
 * @throws OutOfBoundsException
 */
function removeWithNullDefault(Ds\Map $map): ?string
{
    return $map->remove(1, null);
}

/**
 * @param Ds\Map<int, string> $map
 * @throws OutOfBoundsException
 */
function removeWithIntegerDefault(Ds\Map $map): string|int
{
    return $map->remove(1, 0);
}

/**
 * @param Ds\Map<int, string> $map
 * @throws OutOfBoundsException
 */
function removeWithNullableDefault(Ds\Map $map, ?int $default): string|int|null
{
    return $map->remove(key: 1, default: $default);
}
