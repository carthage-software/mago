<?php

declare(strict_types=1);

/**
 * @param 'a'|'b' $key
 *
 * @return array{'a'?: float, 'b'?: float}
 */
function one_of_two(string $key, float $value): array
{
    $out = [];
    $out[$key] = $value;

    return $out;
}

/**
 * @param 'a'|'b' $key
 *
 * @return array{'a': float, 'b': float}
 */
function claims_both(string $key, float $value): array
{
    $out = [];
    $out[$key] = $value;

    // @mago-expect analysis:invalid-return-statement
    return $out;
}

/**
 * @param 'a'|'b' $key
 *
 * @return array{'a': int|string, 'b': int|string}
 */
function overwrite_one_of_two(string $key): array
{
    $out = ['a' => 1, 'b' => 2];
    $out[$key] = 'x';

    return $out;
}

/** @return array{'a': string} */
function single_key(): array
{
    $out = ['a' => 1];
    $out['a'] = 'x';

    return $out;
}

function literal_or_int(int $value): bool
{
    $out = [];
    $out[$value > 0 ? 'pos' : $value] = 1;

    return isset($out['pos']);
}
