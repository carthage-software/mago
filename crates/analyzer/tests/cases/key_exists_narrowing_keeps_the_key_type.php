<?php

declare(strict_types=1);

function key_exists_takes_int(int $i): int
{
    return $i;
}

function key_exists_literal_keys(int $key): int
{
    if (array_key_exists($key, ['a' => 1, 'b' => 2])) {
        return key_exists_takes_int($key);
    }

    return 0;
}

/**
 * @param array<string, string> $items
 */
function key_exists_generic_keys(int $key, array $items): int
{
    if (array_key_exists($key, $items)) {
        return key_exists_takes_int($key);
    }

    if (isset($items[$key])) {
        return key_exists_takes_int($key);
    }

    return 0;
}

/**
 * @param list<string> $strings
 */
function key_exists_in_array(int $key, array $strings): int
{
    if (in_array($key, $strings, true)) {
        return key_exists_takes_int($key);
    }

    return 0;
}

function key_exists_float_key(float $key): string
{
    $items = [971 => 'x', 972 => 'y'];

    if (array_key_exists($key, $items)) {
        // @mago-expect analysis:impossible-condition
        if (is_int($key)) {
            return 'int';
        }
    }

    return 'float';
}

function key_exists_float_or_string_key(float|string $key): string
{
    $items = [971 => 'x', 972 => 'y'];

    if (array_key_exists($key, $items) && !is_float($key)) {
        return $items[$key];
    }

    return 'z';
}
