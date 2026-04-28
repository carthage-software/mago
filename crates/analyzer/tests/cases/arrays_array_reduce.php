<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 */
function sum_all(array $xs): int
{
    return array_reduce($xs, static fn(int $acc, int $x): int => $acc + $x, 0);
}
