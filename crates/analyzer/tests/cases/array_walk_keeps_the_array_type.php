<?php

declare(strict_types=1);

/**
 * @param list<int> $numbers
 * @return list<int>
 */
function bump(array $numbers): array
{
    array_walk($numbers, function (int &$number): void {
        $number++;
    });

    return $numbers;
}

/**
 * @param list<int> $numbers
 * @return list<int>
 */
function show(array $numbers): array
{
    array_walk($numbers, function (int $n): string {
        echo $n;

        return (string) $n;
    });

    return $numbers;
}

/**
 * @param array<string, int> $numbers
 * @return array<string, int>
 */
function bump_recursive(array $numbers): array
{
    array_walk_recursive($numbers, function (int &$number): void {
        $number++;
    });

    return $numbers;
}
