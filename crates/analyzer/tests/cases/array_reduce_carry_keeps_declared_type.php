<?php

declare(strict_types=1);

/**
 * @param list<string> $parts
 *
 * @return array<array-key, mixed>
 */
function array_reduce_carry_collect(array $parts): array
{
    return array_reduce($parts, static fn(array $carry, string $part): array => [...$carry, $part], []);
}

/**
 * @param list<string> $parts
 *
 * @return array<array-key, mixed>
 */
function array_reduce_carry_collectWithFirst(array $parts): array
{
    return array_reduce(
        $parts,
        static fn(array $carry, string $part): array => [...$carry, $part, $carry[0] ?? $part],
        [],
    );
}

/**
 * @param list<int> $numbers
 *
 * @return array<array-key, mixed>
 */
function array_reduce_carry_count3(array $numbers): array
{
    $r = array_reduce(
        $numbers,
        static fn(array $carry, int $_number): array => ['n' => (int) ($carry['n'] ?? 0) + 1],
        ['n' => 0],
    );

    return $r;
}
