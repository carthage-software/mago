<?php

declare(strict_types=1);

/**
 * @param non-empty-list<int> $integers
 * @return non-empty-list<int|float>
 */
function multiply_by(array $integers, int|float $by): array
{
    array_walk($integers, fn(int $value, int|float $x): int|float => $value * $x, $by);

    return $integers;
}

/**
 * @param non-empty-list<int> $integers
 * @return non-empty-list<int>
 */
function multiply_by_two(array $integers): array
{
    array_walk($integers, fn(int $value): int => $value * 2);

    return $integers;
}
