<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param array<string, T> $map
 * @param string $key
 *
 * @return T|null
 */
function gen_get_or_null(array $map, string $key): mixed
{
    return $map[$key] ?? null;
}

/** @var array<string, int> $m */
$m = ['a' => 1, 'b' => 2];
$v = gen_get_or_null($m, 'a');
if (null !== $v) {
    echo $v + 1;
}
