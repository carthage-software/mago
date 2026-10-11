<?php

declare(strict_types=1);

/**
 * @param 'object'|'array' $mode
 * @return ($mode is 'array' ? array<string, mixed> : stdClass)
 */
function fetch(string $mode = MODE_OBJECT): array|stdClass
{
    return $mode === 'array' ? [] : new stdClass();
}

/**
 * @param 'object'|'array' $mode
 * @return ($mode is 'array' ? array<string, mixed> : stdClass)
 */
function fetch_defined(string $mode = DEFINED_MODE_OBJECT): array|stdClass
{
    return $mode === 'array' ? [] : new stdClass();
}

/**
 * @param 'object'|'array' $mode
 * @return ($mode is 'array' ? array<string, mixed> : stdClass)
 */
function fetch_array(string $mode = MODE_ARRAY): array|stdClass
{
    return $mode === 'array' ? [] : new stdClass();
}

/**
 * @param 'object'|'array' $mode
 * @return ($mode is 'array' ? array<string, mixed> : stdClass)
 */
function fetch_named(int $unused = 0, string $mode = MODE_OBJECT): array|stdClass
{
    return $mode === 'array' ? [] : new stdClass();
}

/**
 * @template T of 'object'|'array'
 * @param T $mode
 * @return (T is 'array' ? array<string, mixed> : stdClass)
 */
function fetch_generic(string $mode = MODE_OBJECT_ALIAS): array|stdClass
{
    return $mode === 'array' ? [] : new stdClass();
}

final class Fetcher
{
    /**
     * @param 'object'|'array' $mode
     * @return ($mode is 'array' ? array<string, mixed> : stdClass)
     */
    public function fetch(string $mode = MODE_OBJECT): array|stdClass
    {
        return $mode === 'array' ? [] : new stdClass();
    }
}
