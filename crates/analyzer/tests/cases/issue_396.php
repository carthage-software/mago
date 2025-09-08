<?php

declare(strict_types=1);

/**
 * @param list<int> $y
 * @return array<int, int>
 */
function y(array $y): array
{
    unset($y[0]);
    return $y;
}

/**
 * @param list<int> $x
 * @return list<int>
 *
 * @mago-expect analysis:invalid-return-statement
 */
function x(array $x): array
{
    unset($x[0]);
    return $x;
}
