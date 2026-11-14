<?php

declare(strict_types=1);

/** @param list{0?: string, 1?: string} $optional */
function first_optional(array $optional): string
{
    return $optional[0] ?? '';
}

/** @param list{0: string, 1: int} $pair */
function second_of_pair(array $pair): int
{
    return $pair[1];
}

/** @param list{0: string, int} $mixed */
function mixed_keys(array $mixed): int
{
    return $mixed[1];
}

/** @return list{0: string, 1: int} */
function make_pair(): array
{
    return ['a', 1];
}
