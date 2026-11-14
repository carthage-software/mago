<?php

declare(strict_types=1);

function int_key(bool $c): null|string
{
    $a = [1 => 'Hello'];

    // @mago-expect analysis:possibly-undefined-int-array-index
    return $a[$c ? 1 : 0];
}

/**
 * @param 'x'|'y' $k
 * @param array{x: int} $a
 */
function string_key(string $k, array $a): null|int
{
    // @mago-expect analysis:possibly-undefined-string-array-index
    return $a[$k];
}

/**
 * @param 'x'|'y' $k
 * @param array{x: int} $a
 */
function in_coalesce(string $k, array $a): int
{
    return $a[$k] ?? 0;
}

/**
 * @param 'x'|'y' $k
 * @param array{x: int, y: int} $a
 */
function all_there(string $k, array $a): int
{
    return $a[$k];
}

/**
 * @param '0'|'1'|'' $path
 */
function set_when_missing(string $path): int
{
    $map = ['' => 0];
    if (!isset($map[$path])) {
        $map[$path] = 1;
    }

    return $map[$path];
}
