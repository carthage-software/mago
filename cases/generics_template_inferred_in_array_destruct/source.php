<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param T $a
 * @param T $b
 *
 * @return array{T, T}
 */
function gen_pair_arr(mixed $a, mixed $b): array
{
    return [$a, $b];
}

[$x, $y] = gen_pair_arr(1, 2);
echo $x + $y;
