<?php

declare(strict_types=1);

function test(array $data, string $key): mixed
{
    if (array_key_exists($key, $data)) {
        return $data[$key];
    }

    return null;
}

function test_isset(array $data, string $key): mixed
{
    if (isset($data[$key])) {
        return $data[$key];
    }

    return null;
}

function test_alias(array $data, string $key): mixed
{
    if (key_exists($key, $data)) {
        return $data[$key];
    }

    return null;
}

/**
 * Unlike `isset()`, `array_key_exists()` keeps `null` values: the access stays `?int`.
 *
 * @param array<string, ?int> $arr
 */
function keeps_null(array $arr, string $key): ?int
{
    if (array_key_exists($key, $arr)) {
        return $arr[$key];
    }

    return null;
}

// The narrowing only applies to the exact key on the exact array that was checked.

function no_check(array $data, string $key): mixed
{
    /** @mago-expect analysis:possibly-undefined-string-array-index */
    return $data[$key];
}

function different_key(array $data, string $key, string $other): mixed
{
    if (array_key_exists($other, $data)) {
        /** @mago-expect analysis:possibly-undefined-string-array-index */
        return $data[$key];
    }

    return null;
}

function different_array(array $data, array $other, string $key): mixed
{
    if (array_key_exists($key, $other)) {
        /** @mago-expect analysis:possibly-undefined-string-array-index */
        return $data[$key];
    }

    return null;
}
