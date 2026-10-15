<?php

declare(strict_types=1);

/**
 * @param array{'foo': true}|array{'bar': int} $arr
 */
function shape_union(array $arr): int
{
    if (!array_key_exists('foo', $arr)) {
        return $arr['bar'];
    }

    return 0;
}

/**
 * @param array{'foo': true}|list<never> $arr
 */
function list_fallback(array $arr): void
{
    if (!array_key_exists('foo', $arr)) {
    }
}

shape_union(['bar' => 1]);
list_fallback([]);

/**
 * @param array{'foo'?: string, 'bar': int} $arr
 * @return array{'bar': int}
 */
function optional_key_removal(array $arr): array
{
    if (!array_key_exists('foo', $arr)) {
        return $arr;
    }

    return ['bar' => $arr['bar']];
}

optional_key_removal(['bar' => 1]);

/**
 * @param array{'foo': true, 'baz': string}|array{'foo'?: string, 'bar': int} $arr
 * @return array{'bar': int}
 */
function shape_union_with_optional(array $arr): array
{
    if (!array_key_exists('foo', $arr)) {
        return $arr;
    }

    return ['bar' => 0];
}

shape_union_with_optional(['bar' => 1]);
