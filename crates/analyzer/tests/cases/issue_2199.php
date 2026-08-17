<?php

declare(strict_types=1);

/** @return 'a'|'b'|null */
function knownStringKeys(string $key): ?string
{
    $values = ['a' => 1, 'b' => 2];

    if (array_key_exists($key, $values)) {
        return $key;
    }

    return null;
}

/** @return 'x'|'y'|null */
function directArray(string $key): ?string
{
    if (array_key_exists($key, ['x' => 1, 'y' => 2])) {
        return $key;
    }

    return null;
}

/**
 * @param int|string $key
 * @param array<string, int> $values
 */
function genericStringKeys(int|string $key, array $values): ?string
{
    if (array_key_exists($key, $values)) {
        return $key;
    }

    return null;
}

/**
 * @param int|string $key
 * @return 1|2|'1'|'2'|null
 */
function knownIntegerKeys(int|string $key): int|string|null
{
    $values = [1 => 'a', 2 => 'b'];

    if (!array_key_exists($key, $values)) {
        return null;
    }

    return $key;
}

/**
 * @param array<int, string> $values
 * @return numeric-string|null
 */
function genericIntegerKeys(string $key, array $values): ?string
{
    if (array_key_exists($key, $values)) {
        return $key;
    }

    return null;
}

/** @return true|null */
function coercedBooleanKey(bool $key): ?bool
{
    if (array_key_exists($key, [1 => 'a'])) {
        return $key;
    }

    return null;
}
