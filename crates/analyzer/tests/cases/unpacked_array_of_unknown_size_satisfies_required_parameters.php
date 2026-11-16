<?php

declare(strict_types=1);

function pair(int $first, int $second): int
{
    return $first + $second;
}

/**
 * @param array<int> $values
 * @param list{int} $single
 */
function spreads(array $values, array $single, Traversable $traversable): int
{
    $total = pair(...$values);
    // @mago-expect analysis:too-few-arguments
    $total += pair(...$single);
    // @mago-expect analysis:too-few-arguments
    $total += pair(...[1]);
    $total += pair(...$traversable);

    return $total + pair(...[1, 2]);
}
