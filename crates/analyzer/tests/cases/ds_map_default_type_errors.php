<?php

declare(strict_types=1);

/**
 * @param Ds\Map<int, string> $map
 * @throws OutOfBoundsException
 */
function getNullDefaultIsNullable(Ds\Map $map): string
{
    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $map->get(1, null);
}

/**
 * @param Ds\Map<int, string> $map
 * @throws OutOfBoundsException
 */
function getIntegerDefaultIsIncluded(Ds\Map $map): string
{
    // @mago-expect analysis:invalid-return-statement
    return $map->get(1, 0);
}

/**
 * @param Ds\Map<int, string> $map
 * @throws OutOfBoundsException
 */
function getNullableDefaultIsIncluded(Ds\Map $map, ?int $default): string
{
    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $map->get(key: 1, default: $default);
}

/**
 * @param Ds\Map<int, string> $map
 * @throws OutOfBoundsException
 */
function removeNullDefaultIsNullable(Ds\Map $map): string
{
    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $map->remove(1, null);
}

/**
 * @param Ds\Map<int, string> $map
 * @throws OutOfBoundsException
 */
function removeIntegerDefaultIsIncluded(Ds\Map $map): string
{
    // @mago-expect analysis:invalid-return-statement
    return $map->remove(1, 0);
}

/**
 * @param Ds\Map<int, string> $map
 * @throws OutOfBoundsException
 */
function removeNullableDefaultIsIncluded(Ds\Map $map, ?int $default): string
{
    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $map->remove(key: 1, default: $default);
}
