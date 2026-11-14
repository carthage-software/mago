<?php

declare(strict_types=1);

/**
 * @param array{id: int, name?: string} $a
 */
function name_of(array $a): string
{
    $b = [...$a];

    return $b['name'] ?? 'none';
}

/**
 * @param array<string, int> $c
 */
function get(array $c): int
{
    $b = [...$c, 'x' => 1];

    return $b['y'] ?? 0;
}

/**
 * @param array<string, int> $c
 *
 * @return array{x: int}
 */
function sealed(array $c): array
{
    // @mago-expect analysis:invalid-return-statement
    return [...$c, 'x' => 1];
}

/**
 * @param array<int, string> $n
 */
function after_keyed(array $n): int
{
    $x = [...$n, 5];

    // @mago-expect analysis:invalid-return-statement
    return $x[0];
}

/**
 * @param list<string> $l
 */
function after_list(array $l): int
{
    $x = [...$l, ...[5]];

    // @mago-expect analysis:invalid-return-statement
    return $x[0];
}

/**
 * @param array{0: int, 1?: string} $t
 */
function third(array $t): int
{
    $x = [1, ...$t, 3];

    // @mago-expect analysis:invalid-return-statement
    return $x[2];
}
