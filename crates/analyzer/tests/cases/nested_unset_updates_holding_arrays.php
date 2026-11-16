<?php

declare(strict_types=1);

/**
 * @param array{a: int, b: array{c: int, d: int}} $values
 *
 * @return array{a: int, b: array{d: int}}
 */
function remove_nested(array $values): array
{
    unset($values['b']['c']);

    return $values;
}

/**
 * @param array{a: int, b: array{c: int, d: int}} $values
 */
function read_removed(array $values): int
{
    unset($values['b']['c']);

    return $values['b']['c']; // @mago-expect analysis:undefined-string-array-index,invalid-return-statement
}

/**
 * @param array{items: list{array{id: int, name: string}}} $values
 *
 * @return array{items: list{array{id: int}}}
 */
function remove_in_list(array $values): array
{
    unset($values['items'][0]['name']);

    return $values;
}
