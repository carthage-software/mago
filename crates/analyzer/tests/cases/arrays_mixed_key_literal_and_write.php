<?php

declare(strict_types=1);

namespace App\MixedKey;

/**
 * @return array<array-key, int>
 */
function from_literal(mixed $id): array
{
    // @mago-expect analysis:mixed-array-index
    return [$id => 1];
}

/**
 * @return array<array-key, int>
 */
function from_write(mixed $id): array
{
    $result = [];
    // @mago-expect analysis:mixed-array-index
    $result[$id] = 1;

    return $result;
}

/**
 * @return array<array-key, int>
 */
function from_invalid_key(\stdClass $key): array
{
    // @mago-expect analysis:invalid-array-element-key
    return [$key => 1];
}

/**
 * @return array<array-key, int>
 */
function from_invalid_key_write(\stdClass $key): array
{
    $result = [];
    // @mago-expect analysis:invalid-array-index
    $result[$key] = 1;

    // The rejected write still widens the inferred key type: genuine mismatch.
    /** @mago-expect analysis:invalid-return-statement */
    return $result;
}

function from_scalar_base(int $x, mixed $id): void
{
    // @mago-expect analysis:invalid-array-access
    $x[$id] = 1;
}
