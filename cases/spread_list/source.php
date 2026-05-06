<?php

declare(strict_types=1);

/**
 * @param list<int> $integers
 * @return non-empty-list<int>
 */
function x1(array $integers): array
{
    return [1, ...$integers];
}

/**
 * @param non-empty-list<int> $integers
 * @return non-empty-list<int>
 */
function x2(array $integers): array
{
    return [...$integers];
}

/**
 * @param list{1, 2} $integers
 * @return non-empty-list<int>
 */
function x3(array $integers): array
{
    return [...$integers];
}

/**
 * @param list{} $integers
 * @return non-empty-list<int>
 */
function x4(array $integers): array
{
    return [1, ...$integers];
}
